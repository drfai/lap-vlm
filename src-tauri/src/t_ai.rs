/**
 * AI Engine module
 * Handles ONNX Runtime sessions and model inference.
 */
use crate::t_common;
use image::DynamicImage;
use ndarray::{Array, Array4};
use ort::{
    inputs,
    session::{Session, builder::GraphOptimizationLevel},
    value::TensorRef,
};
use reqwest::header::{CONTENT_RANGE, RANGE, USER_AGENT};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::{
    io::{ErrorKind, Read},
    path::{Path, PathBuf},
    sync::{
        Mutex,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
    time::Duration,
};
use tauri::{AppHandle, Emitter, Manager};
use tokenizers::Tokenizer;
use tokio::io::AsyncWriteExt;

pub struct AiEngine {
    text_model: Option<Session>,
    vision_model: Option<Session>,
    tokenizer: Option<Tokenizer>,
    text_model_kind: ImageSearchTextModel,
}

const AI_INTRA_THREADS: usize = 2;
const MULTILINGUAL_TEXT_MODEL_URL: &str =
    "https://github.com/julyx10/lap-binaries/releases/download/models/text_model.onnx";
const MULTILINGUAL_TOKENIZER_URL: &str =
    "https://github.com/julyx10/lap-binaries/releases/download/models/tokenizer.json";
const MULTILINGUAL_CHECKSUMS_URL: &str =
    "https://github.com/julyx10/lap-binaries/releases/download/models/sha256sums.txt";
const MULTILINGUAL_RELEASE_API_URL: &str =
    "https://api.github.com/repos/julyx10/lap-binaries/releases/tags/models";
const MULTILINGUAL_MODEL_CONNECT_TIMEOUT: Duration = Duration::from_secs(30);
static MULTILINGUAL_MODEL_DOWNLOAD_ID: AtomicU64 = AtomicU64::new(0);
static MULTILINGUAL_MODEL_INSTALLING: AtomicBool = AtomicBool::new(false);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ImageSearchTextModel {
    Default,
    Multilingual,
}

impl ImageSearchTextModel {
    pub fn from_i64(value: i64) -> Self {
        match value {
            1 => Self::Multilingual,
            _ => Self::Default,
        }
    }

    pub fn as_i64(self) -> i64 {
        match self {
            Self::Default => 0,
            Self::Multilingual => 1,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageSearchModelStatus {
    pub active_model: i64,
    pub multilingual_available: bool,
}

#[derive(Debug, Clone)]
struct TextModelPaths {
    model: PathBuf,
    tokenizer: PathBuf,
}

fn checksum_for_file(content: &str, expected_filename: &str) -> Result<String, String> {
    for line in content.lines().filter(|line| !line.trim().is_empty()) {
        let mut fields = line.split_whitespace();
        let Some(hash) = fields.next() else {
            return Err("Invalid multilingual model checksum file".to_string());
        };
        let Some(filename) = fields.next() else {
            return Err("Invalid multilingual model checksum file".to_string());
        };
        if fields.next().is_some()
            || hash.len() != 64
            || !hash.bytes().all(|byte| byte.is_ascii_hexdigit())
        {
            return Err("Invalid multilingual model checksum file".to_string());
        }
        if filename.trim_start_matches('*') == expected_filename {
            return Ok(hash.to_ascii_lowercase());
        }
    }
    Err(format!("Checksum file is missing {}", expected_filename))
}

struct MultilingualModelInstallGuard;

impl MultilingualModelInstallGuard {
    fn begin() -> Self {
        MULTILINGUAL_MODEL_INSTALLING.store(true, Ordering::SeqCst);
        Self
    }
}

impl Drop for MultilingualModelInstallGuard {
    fn drop(&mut self) {
        MULTILINGUAL_MODEL_INSTALLING.store(false, Ordering::SeqCst);
    }
}

fn file_sha256(path: &Path) -> Result<String, String> {
    let mut file = std::fs::File::open(path)
        .map_err(|e| format!("Failed to open {} for verification: {}", path.display(), e))?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 64 * 1024];

    loop {
        let read = file
            .read(&mut buffer)
            .map_err(|e| format!("Failed to read {} for verification: {}", path.display(), e))?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }

    Ok(format!("{:x}", hasher.finalize()))
}

fn restore_interrupted_multilingual_model(model_dir: &Path) {
    if MULTILINGUAL_MODEL_INSTALLING.load(Ordering::SeqCst) || model_dir.exists() {
        return;
    }
    let Some(parent) = model_dir.parent() else {
        return;
    };
    let Some(model_name) = model_dir.file_name().and_then(|name| name.to_str()) else {
        return;
    };
    let backup_prefix = format!("{}.backup.", model_name);
    let Ok(entries) = std::fs::read_dir(parent) else {
        return;
    };
    let Some(backup_dir) = entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .find(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with(&backup_prefix))
        })
    else {
        return;
    };
    let _ = std::fs::rename(backup_dir, model_dir);
}

impl AiEngine {
    pub fn new() -> Self {
        Self {
            text_model: None,
            vision_model: None,
            tokenizer: None,
            text_model_kind: ImageSearchTextModel::Default,
        }
    }

    pub fn load_models(&mut self, app: &AppHandle) -> Result<(), String> {
        if self.text_model.is_some() && self.vision_model.is_some() {
            return Ok(());
        }

        println!("Loading AI Models...");

        let resource_dir = Self::resource_model_dir(app)?;
        let vision_model_path = resource_dir.join(t_common::AI_VISION_MODEL);
        // Load Vision Model
        if self.vision_model.is_none() {
            let vision_model = Self::load_session(&vision_model_path, "vision")?;
            self.vision_model = Some(vision_model);
        }

        if self.text_model.is_none() {
            self.set_text_model(app, ImageSearchTextModel::Default)?;
        }

        println!("AI Models Loaded Successfully!");
        Ok(())
    }

    /// FORK: 诊断用——把当前 AI 引擎状态写到日志文件。
    /// GUI 应用的 stderr 无处可去，前端也不显示 check_ai_status，
    /// 导致"模型不可用"时完全看不到原因。
    pub fn write_diag(app: &AppHandle, engine: &AiEngine) {
        let resource_dir = Self::resource_model_dir(app)
            .map(|p| p.display().to_string())
            .unwrap_or_else(|e| format!("<err: {}>", e));
        let dll = onnxruntime_dll_path()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|e| format!("<err: {}>", e));
        let text = format!(
            "ai_loaded={} vision={} text={} tokenizer={}\nresource_dir={}\nonnxruntime_dll={}\ndll_exists={}\n",
            engine.is_loaded(),
            engine.vision_model.is_some(),
            engine.text_model.is_some(),
            engine.tokenizer.is_some(),
            resource_dir,
            dll,
            std::path::Path::new(&dll).exists(),
        );
        if let Ok(dir) = crate::t_config::get_app_data_dir() {
            let _ = std::fs::write(dir.join("ai-diag.log"), text);
        }
    }

    fn load_session(path: &Path, model_name: &str) -> Result<Session, String> {
        // FORK: ort rc.14 load-dynamic。用 init_from 显式指定 onnxruntime.dll 路径，
        // 不依赖 ORT_DYLIB_PATH / init() 的默认搜索（后者在 load-dynamic 下不可靠，
        // 会导致原生 AI 模型加载失败、报"模型不可用"）。
        //
        // Environment 必须进程级缓存：ONNX Runtime 全局只允许一个 Environment
        // per process，重复 init_from 会直接报 "only one environment is allowed
        // per process"，导致第二个模型（text）加载失败、AI 整体不可用。
        let env = ort_environment()?;
        Session::builder(&env)
            .map_err(|e| e.to_string())?
            .with_optimization_level(GraphOptimizationLevel::Level3)
            .map_err(|e| e.to_string())?
            .with_intra_threads(AI_INTRA_THREADS)
            .map_err(|e| e.to_string())?
            .commit_from_file(path)
            .map_err(|e| format!("Failed to load {} model from {:?}: {}", model_name, path, e))
    }

    fn resource_model_dir(app: &AppHandle) -> Result<PathBuf, String> {
        #[cfg(debug_assertions)]
        {
            let manifest_dir = env!("CARGO_MANIFEST_DIR");
            let dev_path = std::path::PathBuf::from(manifest_dir).join("resources/models");
            if dev_path.exists() {
                return Ok(dev_path);
            }
        }

        app.path()
            .resolve("models", tauri::path::BaseDirectory::Resource)
            .map_err(|e| format!("Failed to resolve resource path: {}", e))
    }

    fn multilingual_model_dir(_app: &AppHandle) -> Result<PathBuf, String> {
        crate::t_config::get_app_data_dir().map(|dir| dir.join("models").join("multilingual"))
    }

    fn text_model_paths(
        app: &AppHandle,
        model: ImageSearchTextModel,
    ) -> Result<TextModelPaths, String> {
        let model_dir = match model {
            ImageSearchTextModel::Default => Self::resource_model_dir(app)?,
            ImageSearchTextModel::Multilingual => Self::multilingual_model_dir(app)?,
        };

        Ok(TextModelPaths {
            model: model_dir.join(t_common::AI_TEXT_MODEL),
            tokenizer: model_dir.join(t_common::AI_TOKENIZER),
        })
    }

    pub fn is_multilingual_model_available(app: &AppHandle) -> bool {
        Self::text_model_paths(app, ImageSearchTextModel::Multilingual)
            .map(|paths| {
                if let Some(model_dir) = paths.model.parent() {
                    restore_interrupted_multilingual_model(model_dir);
                }
                paths.model.exists() && paths.tokenizer.exists()
            })
            .unwrap_or(false)
    }

    pub fn model_status(&self, app: &AppHandle) -> ImageSearchModelStatus {
        ImageSearchModelStatus {
            active_model: self.text_model_kind.as_i64(),
            multilingual_available: Self::is_multilingual_model_available(app),
        }
    }

    pub fn set_text_model(
        &mut self,
        app: &AppHandle,
        model: ImageSearchTextModel,
    ) -> Result<(), String> {
        if self.text_model.is_some() && self.text_model_kind == model {
            return Ok(());
        }

        let paths = Self::text_model_paths(app, model)?;
        if model == ImageSearchTextModel::Multilingual {
            if let Some(model_dir) = paths.model.parent() {
                restore_interrupted_multilingual_model(model_dir);
            }
        }
        if !paths.model.exists() || !paths.tokenizer.exists() {
            return Err(format!(
                "Image search model files are missing for {:?}",
                model
            ));
        }

        let tokenizer = Tokenizer::from_file(&paths.tokenizer)
            .map_err(|e| format!("Failed to load tokenizer from {:?}: {}", paths.tokenizer, e))?;
        let text_model = Self::load_session(&paths.model, "text")?;

        let previous_text_model = self.text_model.take();
        let previous_tokenizer = self.tokenizer.take();
        let previous_model_kind = self.text_model_kind;
        self.tokenizer = Some(tokenizer);
        self.text_model = Some(text_model);
        self.text_model_kind = model;
        if let Err(error) = self.ensure_embedding_dimensions_match() {
            self.text_model = previous_text_model;
            self.tokenizer = previous_tokenizer;
            self.text_model_kind = previous_model_kind;
            return Err(error);
        }
        Ok(())
    }

    fn ensure_embedding_dimensions_match(&mut self) -> Result<(), String> {
        if self.vision_model.is_none() {
            return Ok(());
        }
        let text_dim = self.encode_text("__lap_embedding_probe__")?.len();
        let vision_dim = self.run_vision_model(Array::zeros((1, 3, 224, 224)))?.len();
        if text_dim != vision_dim {
            return Err(format!(
                "Image search model is incompatible with the bundled vision model (text dimension {}, vision dimension {}). Please select or download a compatible model.",
                text_dim, vision_dim
            ));
        }
        Ok(())
    }

    pub fn is_loaded(&self) -> bool {
        self.text_model.is_some() && self.vision_model.is_some() && self.tokenizer.is_some()
    }

    pub fn encode_text(&mut self, text: &str) -> Result<Vec<f32>, String> {
        if !self.is_loaded() {
            return Err("AI models not loaded".to_string());
        }

        let tokenizer = self.tokenizer.as_ref().unwrap();
        let encoding = tokenizer
            .encode(text, true)
            .map_err(|e| format!("Tokenization error: {}", e))?;

        let input_ids = encoding.get_ids();
        let attention_mask = encoding.get_attention_mask();

        let input_ids_array = Array::from_shape_vec(
            (1, input_ids.len()),
            input_ids.iter().map(|&x| x as i64).collect(),
        )
        .map_err(|e| e.to_string())?;

        let input_ids_value = TensorRef::from_array_view(&input_ids_array).map_err(|e| e.to_string())?;
        let attention_mask_array = Array::from_shape_vec(
            (1, attention_mask.len()),
            attention_mask.iter().map(|&x| x as i64).collect(),
        )
        .map_err(|e| e.to_string())?;
        let attention_mask_value =
            TensorRef::from_array_view(&attention_mask_array).map_err(|e| e.to_string())?;

        let uses_attention_mask = self
            .text_model
            .as_ref()
            .unwrap()
            .inputs()
            .iter()
            .any(|input| input.name() == "attention_mask");

        let outputs = if uses_attention_mask {
            self.text_model.as_mut().unwrap().run(inputs![
                "input_ids" => input_ids_value,
                "attention_mask" => attention_mask_value,
            ])
        } else {
            self.text_model.as_mut().unwrap().run(inputs![
                "input_ids" => input_ids_value,
            ])
        }
        .map_err(|e| format!("Inference error: {}", e))?;

        let (embedding, first_token_only) = if let Some(vals) = outputs.get("pooler_output") {
            (vals, false)
        } else if let Some(vals) = outputs.get("text_embeds") {
            (vals, false)
        } else if let Some(vals) = outputs.get("last_hidden_state") {
            (vals, true)
        } else {
            (&outputs[0], true)
        };

        Self::extract_text_embedding(embedding, first_token_only)
    }

    fn extract_text_embedding(
        embedding: &ort::value::DynValue,
        first_token_only: bool,
    ) -> Result<Vec<f32>, String> {
        let (shape, embedding_data) = embedding
            .try_extract_tensor::<f32>()
            .map_err(|e| format!("Failed to extract tensor: {}", e))?;

        if first_token_only && shape.len() >= 3 {
            let hidden_size = shape
                .last()
                .copied()
                .filter(|dim| *dim > 0)
                .ok_or_else(|| format!("Invalid text embedding shape: {}", shape))?
                as usize;
            if embedding_data.len() < hidden_size {
                return Err(format!(
                    "Text embedding data is shorter than shape {}",
                    shape
                ));
            }
            return Ok(embedding_data[..hidden_size].to_vec());
        }

        Ok(embedding_data.to_vec())
    }

    pub fn encode_image(&mut self, image_path: &str) -> Result<Vec<f32>, String> {
        if !self.is_loaded() {
            return Err("AI models not loaded".to_string());
        }

        let image_input = self.preprocess_image(image_path)?;
        self.run_vision_model(image_input)
    }

    pub fn encode_image_from_bytes(&mut self, image_bytes: &[u8]) -> Result<Vec<f32>, String> {
        if !self.is_loaded() {
            return Err("AI models not loaded".to_string());
        }

        let img = image::load_from_memory(image_bytes)
            .map_err(|e| format!("Failed to load image from memory: {}", e))?;
        let image_input = self.preprocess_dynamic_image(img)?;

        self.run_vision_model(image_input)
    }

    fn run_vision_model(&mut self, image_input: Array4<f32>) -> Result<Vec<f32>, String> {
        let image_input_value = TensorRef::from_array_view(&image_input).map_err(|e| e.to_string())?;

        let outputs = self
            .vision_model
            .as_mut()
            .unwrap()
            .run(inputs![
                "pixel_values" => image_input_value,
            ])
            .map_err(|e| format!("Inference error: {}", e))?;

        let embedding = if let Some(vals) = outputs.get("pooler_output") {
            vals
        } else if let Some(vals) = outputs.get("image_embeds") {
            vals
        } else {
            &outputs[0]
        };

        let (_, embedding_data) = embedding
            .try_extract_tensor::<f32>()
            .map_err(|e| format!("Failed to extract tensor: {}", e))?;

        Ok(embedding_data.to_vec())
    }

    fn preprocess_image(&self, path: &str) -> Result<Array4<f32>, String> {
        let img = image::open(path).map_err(|e| format!("Failed to open image: {}", e))?;
        self.preprocess_dynamic_image(img)
    }

    fn preprocess_dynamic_image(&self, img: DynamicImage) -> Result<Array4<f32>, String> {
        // resize to 224x224
        let img = img.resize_exact(224, 224, image::imageops::FilterType::Triangle);
        let rgb_img = img.to_rgb8();

        // Normalize
        let mean = [0.48145466, 0.4578275, 0.40821073];
        let std = [0.26862954, 0.26130258, 0.27577711];

        let mut array = Array::zeros((1, 3, 224, 224));

        for (x, y, pixel) in rgb_img.enumerate_pixels() {
            let r = (pixel[0] as f32 / 255.0 - mean[0]) / std[0];
            let g = (pixel[1] as f32 / 255.0 - mean[1]) / std[1];
            let b = (pixel[2] as f32 / 255.0 - mean[2]) / std[2];

            array[[0, 0, y as usize, x as usize]] = r;
            array[[0, 1, y as usize, x as usize]] = g;
            array[[0, 2, y as usize, x as usize]] = b;
        }

        Ok(array)
    }
}

/// FORK: 定位 onnxruntime.dll（load-dynamic 需要）。
/// dev 用源码 resources 目录，prod 用 exe 同目录（resource_dir）。
/// 失败时返回 Err（不 panic），让 load_session 能把错误传回上层。
pub(crate) fn onnxruntime_dll_path() -> Result<std::path::PathBuf, String> {
    #[cfg(debug_assertions)]
    {
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        let dev = std::path::PathBuf::from(manifest_dir).join("resources").join("onnxruntime.dll");
        if dev.exists() {
            return Ok(dev);
        }
    }

    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let dll = exe
        .parent()
        .unwrap_or_else(|| std::path::Path::new("."))
        .join("onnxruntime.dll");
    if !dll.exists() {
        return Err(format!("onnxruntime.dll not found at {:?}", dll));
    }
    Ok(dll)
}

/// FORK: 设置 ORT_DYLIB_PATH, 让 load-dynamic 模式找到 onnxruntime.dll。
/// 模式与 t_video::init_ffmpeg_path 一致: dev 用源码目录, production 用资源目录。
/// 必须在首次加载 ONNX 模型前调用。
pub fn init_onnxruntime_path(app: &AppHandle) {
    let dll = std::path::Path::new("onnxruntime.dll");

    #[cfg(debug_assertions)]
    {
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        let dev_path = std::path::PathBuf::from(manifest_dir).join("resources").join(dll);
        if dev_path.exists() {
            // SAFETY: 单线程 setup 阶段, 无并发读写环境变量的风险。
            unsafe { std::env::set_var("ORT_DYLIB_PATH", &dev_path) };
            return;
        }
    }

    if let Ok(res_dir) = app.path().resource_dir() {
        let prod_path = res_dir.join(dll);
        if prod_path.exists() {
            // SAFETY: 同上, setup 阶段单线程调用。
            unsafe { std::env::set_var("ORT_DYLIB_PATH", &prod_path) };
        }
    }
}

/// FORK: 进程级 ONNX Runtime Environment 缓存。
///
/// ONNX Runtime 全局只允许一个 Environment per process。Lap 会加载多个模型
/// （vision + text + 人脸的 det/embedding），每次 load_session 都 init_from
/// 会让第二次直接报 "only one environment is allowed per process"，
/// 导致 text 模型加载失败、AI 整体不可用。用 OnceLock 保证只初始化一次。
static ORT_ENV: std::sync::OnceLock<ort::environment::Environment> = std::sync::OnceLock::new();

/// 获取（或首次创建）全局 ONNX Runtime Environment。
pub(crate) fn ort_environment() -> Result<&'static ort::environment::Environment, String> {
    if let Some(env) = ORT_ENV.get() {
        return Ok(env);
    }
    let dll = onnxruntime_dll_path()?;
    let env = ort::init_from(&dll)
        .map_err(|e| format!("load onnxruntime.dll from {:?} failed: {}", dll, e))?
        .build()
        .map_err(|e| e.to_string())?;
    // 竞争时可能已被其他线程设置，用 get_or_init 的返回值为准
    Ok(ORT_ENV.get_or_init(|| env))
}

pub struct AiState(pub Mutex<AiEngine>);

async fn verify_downloaded_file(
    path: &Path,
    expected_size: Option<u64>,
    expected_sha256: &str,
) -> Result<(), String> {
    let metadata = tokio::fs::metadata(path)
        .await
        .map_err(|e| format!("Failed to inspect downloaded model file: {}", e))?;
    if let Some(expected_size) = expected_size {
        if metadata.len() != expected_size {
            return Err(format!(
                "Downloaded {} has unexpected size (expected {}, got {})",
                path.display(),
                expected_size,
                metadata.len()
            ));
        }
    }

    let path = path.to_owned();
    let hash_path = path.clone();
    let digest = tokio::task::spawn_blocking(move || file_sha256(&hash_path))
        .await
        .map_err(|e| format!("Failed to verify downloaded model file: {}", e))??;
    if digest != expected_sha256 {
        return Err(format!(
            "Downloaded {} failed integrity verification",
            path.display()
        ));
    }
    Ok(())
}

async fn get_remote_file_size(client: &reqwest::Client, url: &str) -> Option<u64> {
    let response = client
        .get(url)
        .header(RANGE, "bytes=0-0")
        .timeout(Duration::from_secs(20))
        .send()
        .await
        .ok()?
        .error_for_status()
        .ok()?;

    if let Some(content_range) = response.headers().get(CONTENT_RANGE) {
        let content_range = content_range.to_str().ok()?;
        if let Some((_, total)) = content_range.rsplit_once('/') {
            if total != "*" {
                return total.parse::<u64>().ok();
            }
        }
    }

    response.content_length()
}

async fn get_release_asset_total_size(
    client: &reqwest::Client,
    files: &[(&str, &str, &str)],
) -> Option<u64> {
    let response = client
        .get(MULTILINGUAL_RELEASE_API_URL)
        .header(USER_AGENT, "Lap")
        .timeout(Duration::from_secs(20))
        .send()
        .await
        .ok()?
        .error_for_status()
        .ok()?;
    let value = serde_json::from_str::<serde_json::Value>(&response.text().await.ok()?).ok()?;
    let assets = value.get("assets")?.as_array()?;
    let mut total_size = 0u64;

    for (_, filename, _) in files {
        let asset = assets
            .iter()
            .find(|asset| asset.get("name").and_then(|name| name.as_str()) == Some(*filename))?;
        total_size += asset.get("size")?.as_u64()?;
    }

    Some(total_size)
}

async fn get_download_total_size(client: &reqwest::Client, files: &[(&str, &str, &str)]) -> u64 {
    if let Some(total_size) = get_release_asset_total_size(client, files).await {
        return total_size;
    }

    let mut total_size = 0u64;
    for (url, _, _) in files {
        match get_remote_file_size(client, url).await {
            Some(file_size) => total_size += file_size,
            None => return 0,
        }
    }
    total_size
}

fn is_current_multilingual_download(download_id: u64) -> bool {
    MULTILINGUAL_MODEL_DOWNLOAD_ID.load(Ordering::SeqCst) == download_id
}

fn ensure_current_multilingual_download(download_id: u64, temp_dir: &Path) -> Result<(), String> {
    if is_current_multilingual_download(download_id) {
        return Ok(());
    }

    let _ = std::fs::remove_dir_all(temp_dir);
    Err("Download canceled".to_string())
}

async fn clean_multilingual_download_temp_dirs(model_dir: &Path) {
    let Some(parent) = model_dir.parent() else {
        return;
    };
    let Some(model_name) = model_dir.file_name().and_then(|name| name.to_str()) else {
        return;
    };
    let temp_prefix = format!("{}.download", model_name);
    let Ok(mut entries) = tokio::fs::read_dir(parent).await else {
        return;
    };

    while let Ok(Some(entry)) = entries.next_entry().await {
        let should_remove = entry
            .file_name()
            .to_str()
            .map(|name| name == temp_prefix || name.starts_with(&format!("{}.", temp_prefix)))
            .unwrap_or(false);
        if should_remove {
            let _ = tokio::fs::remove_dir_all(entry.path()).await;
        }
    }
}

pub async fn download_multilingual_text_model(app: AppHandle) -> Result<(), String> {
    let download_id = MULTILINGUAL_MODEL_DOWNLOAD_ID.fetch_add(1, Ordering::SeqCst) + 1;
    let model_dir = AiEngine::multilingual_model_dir(&app)?;
    clean_multilingual_download_temp_dirs(&model_dir).await;
    let temp_dir = model_dir.with_extension(format!("download.{}", download_id));
    match tokio::fs::remove_dir_all(&temp_dir).await {
        Ok(_) => {}
        Err(e) if e.kind() == ErrorKind::NotFound => {}
        Err(e) => return Err(format!("Failed to clean temporary download files: {}", e)),
    }
    tokio::fs::create_dir_all(&temp_dir)
        .await
        .map_err(|e| e.to_string())?;

    let files = [
        (
            MULTILINGUAL_TEXT_MODEL_URL,
            t_common::AI_TEXT_MODEL,
            "text_model",
        ),
        (
            MULTILINGUAL_TOKENIZER_URL,
            t_common::AI_TOKENIZER,
            "tokenizer",
        ),
    ];
    let client = reqwest::Client::builder()
        .connect_timeout(MULTILINGUAL_MODEL_CONNECT_TIMEOUT)
        .build()
        .map_err(|e| format!("Failed to create download client: {}", e))?;
    let expected_total = get_download_total_size(&client, &files).await;
    let checksums = client
        .get(MULTILINGUAL_CHECKSUMS_URL)
        .send()
        .await
        .map_err(|e| format!("Failed to download multilingual model checksums: {}", e))?
        .error_for_status()
        .map_err(|e| format!("Failed to download multilingual model checksums: {}", e))?
        .text()
        .await
        .map_err(|e| format!("Failed to read multilingual model checksums: {}", e))?;
    let files = files.map(|(url, filename, label)| {
        checksum_for_file(&checksums, filename).map(|checksum| (url, filename, label, checksum))
    });
    let files = files.into_iter().collect::<Result<Vec<_>, String>>()?;
    let total_files = files.len() as f64;
    let mut downloaded_total = 0u64;
    ensure_current_multilingual_download(download_id, &temp_dir)?;

    let _ = app.emit(
        "image_search_model_download_progress",
        serde_json::json!({
            "progress": 0,
            "downloadedBytes": 0,
            "totalBytes": expected_total,
            "downloadId": download_id,
            "file": "start",
        }),
    );

    for (index, (url, filename, label, expected_sha256)) in files.iter().enumerate() {
        ensure_current_multilingual_download(download_id, &temp_dir)?;
        let response = client
            .get(*url)
            .send()
            .await
            .map_err(|e| format!("Failed to download {}: {}", filename, e))?
            .error_for_status()
            .map_err(|e| format!("Failed to download {}: {}", filename, e))?;

        let path = temp_dir.join(filename);
        let mut file = tokio::fs::File::create(&path)
            .await
            .map_err(|e| e.to_string())?;
        let content_length = response.content_length().unwrap_or(0);
        let mut downloaded = 0u64;
        let mut response = response;
        while let Some(chunk) = response
            .chunk()
            .await
            .map_err(|e| format!("Failed to read {}: {}", filename, e))?
        {
            if chunk.is_empty() {
                continue;
            }
            ensure_current_multilingual_download(download_id, &temp_dir)?;
            file.write_all(&chunk).await.map_err(|e| e.to_string())?;
            downloaded += chunk.len() as u64;
            downloaded_total += chunk.len() as u64;

            let file_progress = if content_length > 0 {
                (downloaded as f64 / content_length as f64).min(1.0)
            } else {
                0.0
            };
            let progress = if expected_total > 0 {
                ((downloaded_total as f64 / expected_total as f64).min(1.0) * 100.0).round() as i64
            } else {
                (((index as f64 + file_progress) / total_files) * 100.0).round() as i64
            };
            let _ = app.emit(
                "image_search_model_download_progress",
                serde_json::json!({
                    "progress": progress,
                    "downloadedBytes": downloaded_total,
                    "totalBytes": expected_total,
                    "downloadId": download_id,
                    "file": label,
                }),
            );
        }
        file.flush().await.map_err(|e| e.to_string())?;
        drop(file);
        verify_downloaded_file(
            &path,
            (content_length > 0).then_some(content_length),
            expected_sha256,
        )
        .await
        .map_err(|e| {
            let _ = std::fs::remove_dir_all(&temp_dir);
            e
        })?;

        let progress = if expected_total > 0 {
            ((downloaded_total as f64 / expected_total as f64).min(1.0) * 100.0).round() as i64
        } else {
            ((((index + 1) as f64) / total_files) * 100.0).round() as i64
        };
        let _ = app.emit(
            "image_search_model_download_progress",
            serde_json::json!({
                "progress": progress,
                "downloadedBytes": downloaded_total,
                "totalBytes": expected_total,
                "downloadId": download_id,
                "file": label,
            }),
        );
    }

    ensure_current_multilingual_download(download_id, &temp_dir)?;
    let temp_text_model_path = temp_dir.join(t_common::AI_TEXT_MODEL);
    let temp_tokenizer_path = temp_dir.join(t_common::AI_TOKENIZER);
    Tokenizer::from_file(&temp_tokenizer_path).map_err(|e| {
        let _ = std::fs::remove_dir_all(&temp_dir);
        format!("Downloaded tokenizer is invalid: {}", e)
    })?;
    if let Err(e) = AiEngine::load_session(&temp_text_model_path, "text") {
        let _ = std::fs::remove_dir_all(&temp_dir);
        return Err(format!("Downloaded text model is invalid: {}", e));
    }

    ensure_current_multilingual_download(download_id, &temp_dir)?;
    let _install_guard = MultilingualModelInstallGuard::begin();
    let backup_dir = model_dir.with_extension(format!("backup.{}", download_id));
    let _ = tokio::fs::remove_dir_all(&backup_dir).await;
    let had_existing_model = model_dir.exists();
    if had_existing_model {
        tokio::fs::rename(&model_dir, &backup_dir)
            .await
            .map_err(|e| format!("Failed to prepare existing model for replacement: {}", e))?;
    }
    if !is_current_multilingual_download(download_id) {
        if had_existing_model {
            let _ = tokio::fs::rename(&backup_dir, &model_dir).await;
        }
        let _ = tokio::fs::remove_dir_all(&temp_dir).await;
        return Err("Download canceled".to_string());
    }
    if let Err(error) = tokio::fs::rename(&temp_dir, &model_dir).await {
        if had_existing_model {
            let _ = tokio::fs::rename(&backup_dir, &model_dir).await;
        }
        return Err(format!(
            "Failed to install downloaded multilingual model: {}",
            error
        ));
    }
    if !is_current_multilingual_download(download_id) {
        if had_existing_model {
            if let Err(error) = tokio::fs::rename(&model_dir, &temp_dir).await {
                return Err(format!(
                    "Failed to cancel multilingual model installation: {}",
                    error
                ));
            }
            if let Err(error) = tokio::fs::rename(&backup_dir, &model_dir).await {
                let _ = tokio::fs::rename(&temp_dir, &model_dir).await;
                return Err(format!(
                    "Failed to restore previous multilingual model: {}",
                    error
                ));
            }
            let _ = tokio::fs::remove_dir_all(&temp_dir).await;
        } else {
            let _ = tokio::fs::remove_dir_all(&model_dir).await;
        }
        return Err("Download canceled".to_string());
    }
    let _ = tokio::fs::remove_dir_all(&backup_dir).await;

    let _ = app.emit(
        "image_search_model_download_progress",
        serde_json::json!({
            "progress": 100,
            "downloadedBytes": downloaded_total,
            "totalBytes": expected_total,
            "downloadId": download_id,
            "file": "complete",
        }),
    );

    Ok(())
}

pub async fn cancel_multilingual_text_model_download(app: AppHandle) -> Result<(), String> {
    MULTILINGUAL_MODEL_DOWNLOAD_ID.fetch_add(1, Ordering::SeqCst);
    let model_dir = AiEngine::multilingual_model_dir(&app)?;
    clean_multilingual_download_temp_dirs(&model_dir).await;
    Ok(())
}
