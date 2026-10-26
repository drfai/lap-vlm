//! [comment]
//! [comment]
//! [comment]
//! [comment]
//!
//! [comment]
//! [comment]
//! [comment]
//! [comment]
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tauri::{AppHandle, Emitter};

// ============================================================
// 閰嶇疆
// ============================================================
/// LLM 杩炴帴閰嶇疆銆傚瓨浜庣嫭绔嬫枃浠?`llm-config.json`锛岄伩鍏嶆敼鍔?Lap 鍘熸湁
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct LlmConfig {
pub enabled: bool,
    /// [comment]
    pub base_url: String,
pub model: String,
pub api_key: String,
    /// [comment]
pub disable_thinking: bool,
pub max_tokens: u32,
pub temperature: f32,
pub preview_max_px: u32,
pub concurrency: u32,
pub timeout_secs: u64,
pub retry_on_parse_error: u32,
pub album_ids: Vec<i64>,
    /// [comment]
    /// [comment]
pub enable_rerank: bool,
pub rerank_top_n: u32,
    /// [comment]
pub prompt_fields: PromptFields,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct PromptFields {
pub builtin_descs: std::collections::BTreeMap<String, String>,
pub custom_fields: Vec<CustomField>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct CustomField {
pub key: String,
pub desc: String,
pub kind: CustomFieldKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CustomFieldKind {
    String,
    Array,
    Number,
}

impl Default for PromptFields {
    fn default() -> Self {
        Self {
            builtin_descs: Default::default(),
            custom_fields: Vec::new(),
        }
    }
}

impl Default for CustomField {
    fn default() -> Self {
        Self {
            key: String::new(),
            desc: String::new(),
            kind: CustomFieldKind::String,
        }
    }
}

/// Built-in index fields. Keys are fixed (bound to DB columns);
/// only descriptions are editable via `builtin_descs` in the UI.
const BUILTIN_FIELDS: &[(&str, &str)] = &[
    ("caption", "\u{4e00}\u{5230}\u{4e24}\u{53e5}\u{5ba2}\u{89c2}\u{63cf}\u{8ff0}\u{ff1a}\u{753b}\u{9762}\u{91cc}\u{6709}\u{4ec0}\u{4e48}\u{3001}\u{5728}\u{505a}\u{4ec0}\u{4e48}\u{3001}\u{4ec0}\u{4e48}\u{73af}\u{5883}"),
    ("objects", "\u{753b}\u{9762}\u{4e2d}\u{53ef}\u{89c1}\u{7684}\u{4e3b}\u{8981}\u{7269}\u{4f53}\u{6216}\u{4eba}\u{ff0c}\u{9010}\u{9879}\u{5217}\u{51fa}"),
    ("scene", "\u{573a}\u{666f}\u{7c7b}\u{578b}\u{ff0c}\u{5982}\u{ff1a}\u{5ba4}\u{5185}\u{3001}\u{6237}\u{5916}\u{3001}\u{8857}\u{9053}\u{3001}\u{6d77}\u{8fb9}\u{3001}\u{5c71}\u{5730}\u{3001}\u{9910}\u{5385}\u{3001}\u{529e}\u{516c}\u{5ba4}"),
    ("action", "\u{753b}\u{9762}\u{4e2d}\u{6b63}\u{5728}\u{53d1}\u{751f}\u{7684}\u{52a8}\u{4f5c}\u{ff1b}\u{82e5}\u{65e0}\u{660e}\u{786e}\u{52a8}\u{4f5c}\u{586b} null"),
    ("relations", "\u{7269}\u{4f53}\u{6216}\u{4eba}\u{4e4b}\u{95f4}\u{7684}\u{7a7a}\u{95f4}\u{5173}\u{7cfb}\u{ff0c}\u{5982}\u{ff1a}\u{676f}\u{5b50}\u{653e}\u{5728}\u{684c}\u{4e0a}\u{3001}\u{4eba}\u{5750}\u{5728}\u{6905}\u{5b50}\u{4e0a}"),
    ("text", "\u{753b}\u{9762}\u{4e2d}\u{53ef}\u{8bfb}\u{7684}\u{6587}\u{5b57}\u{ff1b}\u{82e5}\u{65e0}\u{586b} null"),
    ("people_count", "\u{753b}\u{9762}\u{4e2d}\u{53ef}\u{5206}\u{8fa8}\u{7684}\u{4eba}\u{8138}\u{6570}\u{91cf}\u{ff0c}\u{6570}\u{5b57}\u{ff0c}\u{770b}\u{4e0d}\u{6e05}\u{586b} 0"),
];

/// Build the indexing prompt from the current config.
///
/// The structure is fixed (header constraint + field list + tail hard rules);
/// only field descriptions and custom fields come from user config,
/// so users cannot craft a destructive prompt.
pub fn build_index_prompt(fields: &PromptFields) -> String {
    let mut out = String::new();
    out.push_str("\u{4f60}\u{662f}\u{7167}\u{7247}\u{7d22}\u{5f15}\u{5668}\u{3002}\u{5206}\u{6790}\u{8fd9}\u{5f20}\u{7167}\u{7247}\u{ff0c}\u{53ea}\u{8f93}\u{51fa} JSON\u{ff0c}\u{4e0d}\u{8981}\u{8f93}\u{51fa}\u{4efb}\u{4f55}\u{5176}\u{4ed6}\u{6587}\u{5b57}\u{3002}\n");
    out.push_str("JSON \u{683c}\u{5f0f}\u{ff08}\u{6240}\u{6709}\u{5b57}\u{6bb5}\u{7528}\u{7b80}\u{4f53}\u{4e2d}\u{6587}\u{586b}\u{5199}\u{ff09}\u{ff1a}\n{\n");

    for (key, default_desc) in BUILTIN_FIELDS {
        let desc = fields
            .builtin_descs
            .get(*key)
            .filter(|d| !d.trim().is_empty())
            .map(|d| d.trim())
            .unwrap_or(default_desc);
        // FORK: 转义引号和反斜杠——用户描述含 " 会破坏 prompt 的 JSON 模板
        let desc = desc.replace('\\', "\\\\").replace('"', "\\\"");
        if *key == "people_count" {
            out.push_str(&format!("  \"{}\": {}\n", key, desc));
        } else if *key == "caption" || *key == "scene" || *key == "action" || *key == "text" {
            out.push_str(&format!("  \"{}\": \"{}\"\n", key, desc));
        } else {
            out.push_str(&format!("  \"{}\": [\"{}\"]\n", key, desc));
        }
    }

    for cf in &fields.custom_fields {
        if cf.key.trim().is_empty() || cf.desc.trim().is_empty() {
            continue;
        }
        let key = cf.key.trim();
        let desc = cf.desc.trim().replace('\\', "\\\\").replace('"', "\\\"");
        match cf.kind {
            CustomFieldKind::String => out.push_str(&format!("  \"{}\": \"{}\"\n", key, desc)),
            CustomFieldKind::Array => out.push_str(&format!("  \"{}\": [\"{}\"]\n", key, desc)),
            CustomFieldKind::Number => out.push_str(&format!("  \"{}\": {}\n", key, desc)),
        }
    }

    out.push_str("}\n");
    out.push_str("\u{786c}\u{6027}\u{8981}\u{6c42}\u{ff1a}\n");
    out.push_str("- \u{53ea}\u{63cf}\u{8ff0}\u{4f60}\u{786e}\u{5b9e}\u{770b}\u{5230}\u{7684}\u{5185}\u{5bb9}\u{3002}\u{4e0d}\u{786e}\u{5b9a}\u{5c31}\u{586b} null\u{ff0c}\u{7edd}\u{5bf9}\u{4e0d}\u{8981}\u{63a8}\u{6d4b}\u{4eba}\u{7269}\u{8eab}\u{4efd}\u{3001}\u{59d3}\u{540d}\u{3001}\u{7cbe}\u{786e}\u{5730}\u{70b9}\u{3001}\u{65f6}\u{95f4}\u{3002}\n");
    out.push_str("- \u{4e0d}\u{8981}\u{8bc4}\u{4ef7}\u{7167}\u{7247}\u{8d28}\u{91cf}\u{ff0c}\u{4e0d}\u{8981}\u{8f93}\u{51fa}\u{5efa}\u{8bae}\u{3002}\n");
    out.push_str("- \u{53ea}\u{8f93}\u{51fa} JSON \u{5bf9}\u{8c61}\u{672c}\u{8eab}\u{3002}");
    out
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            base_url: "http://127.0.0.1:1234".to_string(),
            model: String::new(),
            api_key: String::new(),
            disable_thinking: true,
            max_tokens: 1024,
            temperature: 0.1,
            preview_max_px: 768,
            concurrency: 1,
            timeout_secs: 300,
            retry_on_parse_error: 1,
            album_ids: Vec::new(),
            enable_rerank: false,
            // FORK: 默认 20 而非 30——精确搜索时每个候选一次推理（27B 约 1-3 秒），
            // 20 个候选约 20-60 秒，是"可接受的等待"上限。用户可在 llm-config.json 调整。
            rerank_top_n: 20,
            prompt_fields: PromptFields::default(),
        }
    }
}

fn llm_config_path() -> Result<std::path::PathBuf, String> {
    let dir = crate::t_config::get_app_data_dir()?;
    Ok(dir.join("llm-config.json"))
}

pub fn load_llm_config() -> Result<LlmConfig, String> {
    let path = llm_config_path()?;
    if !path.exists() {
        return Ok(LlmConfig::default());
    }
    let content =
        std::fs::read_to_string(&path).map_err(|e| format!("Failed to read llm config: {}", e))?;
    serde_json::from_str(&content).map_err(|e| format!("Failed to parse llm config: {}", e))
}

pub fn save_llm_config(cfg: &LlmConfig) -> Result<(), String> {
    let path = llm_config_path()?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("Failed to create config dir: {}", e))?;
    }
    let content = serde_json::to_string_pretty(cfg)
        .map_err(|e| format!("Failed to serialize llm config: {}", e))?;
    std::fs::write(&path, content).map_err(|e| format!("Failed to write llm config: {}", e))
}

// ============================================================
// ============================================================

#[derive(Clone)]
pub struct LlmState(pub Arc<Mutex<LlmConfig>>);

impl Default for LlmState {
    fn default() -> Self {
        Self(Arc::new(Mutex::new(LlmConfig::default())))
    }
}

#[derive(Clone)]
pub struct LlmIndexCancellation(pub Arc<Mutex<bool>>);

#[derive(Clone)]
pub struct LlmIndexingStatus(pub Arc<Mutex<bool>>);

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LlmIndexProgress {
    pub current: u64,
    pub total: u64,
    pub succeeded: u64,
    pub failed: u64,
    pub phase: String,
#[serde(default)]
    pub last_error: String,
}

impl Default for LlmIndexProgress {
    fn default() -> Self {
        Self {
            current: 0,
            total: 0,
            succeeded: 0,
            failed: 0,
            phase: "idle".to_string(),
            last_error: String::new(),
        }
    }
}

#[derive(Clone)]
pub struct LlmIndexProgressState(pub Arc<Mutex<LlmIndexProgress>>);

// ============================================================
// 绱㈠紩浜х墿
// ============================================================

/// [comment]
/// [comment]
///
/// [comment]
/// [comment]
/// - `caption` / `scene` 鏈夋椂杩斿洖 `null`
///
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct LlmCaption {
    #[serde(deserialize_with = "de_str")]
    pub caption: String,
    #[serde(deserialize_with = "de_str_vec")]
    pub objects: Vec<String>,
    #[serde(deserialize_with = "de_str")]
    pub scene: String,
    #[serde(deserialize_with = "de_str")]
    pub action: String,
    #[serde(deserialize_with = "de_str_vec")]
    pub relations: Vec<String>,
#[serde(deserialize_with = "de_str")]
    pub text: String,
#[serde(alias = "people_count", deserialize_with = "de_i64")]
    pub people_count: i64,
    /// [comment]
    /// [comment]
    /// serde 浼氭妸瀹冧滑鍏ㄩ儴涓㈠純锛屾悳涓嶅埌銆?    ///
#[serde(flatten, deserialize_with = "de_extra")]
    pub extra: std::collections::BTreeMap<String, String>,
}

/// [comment]
/// - 鏁板瓧/甯冨皵 -> toString
/// - 鏁扮粍 -> 绌烘牸鎷兼帴锛堥€掑綊灞曞钩涓€灞傦級
/// - 瀵硅薄 -> 灞曞钩鍏跺€?/// - null -> 涓㈠純锛堜笉鍏?map锛岄伩鍏嶅櫔闊筹級
fn de_extra<'de, D>(d: D) -> Result<std::collections::BTreeMap<String, String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let v = serde_json::Value::deserialize(d)?;
    let mut out = std::collections::BTreeMap::new();
    if let serde_json::Value::Object(map) = v {
        for (k, val) in map {
            let text = value_to_text(&val);
            if !text.is_empty() {
                out.insert(k, text);
            }
        }
    }
    Ok(out)
}

pub fn value_to_text(v: &serde_json::Value) -> String {
    match v {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Number(n) => n.to_string(),
        serde_json::Value::Bool(b) => b.to_string(),
        serde_json::Value::Array(a) => a
            .iter()
            .map(value_to_text)
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join(" "),
        serde_json::Value::Object(o) => o
            .values()
            .map(value_to_text)
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join(" "),
        serde_json::Value::Null => String::new(),
    }
}

fn de_str<'de, D>(d: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let v = serde_json::Value::deserialize(d)?;
    Ok(match v {
        serde_json::Value::String(s) => s,
        serde_json::Value::Number(n) => n.to_string(),
        serde_json::Value::Bool(b) => b.to_string(),
        _ => String::new(),
    })
}

fn de_str_vec<'de, D>(d: D) -> Result<Vec<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let v = serde_json::Value::deserialize(d)?;
    Ok(match v {
        serde_json::Value::Array(a) => a
            .into_iter()
            .map(|e| match e {
                serde_json::Value::String(s) => s,
                serde_json::Value::Number(n) => n.to_string(),
                serde_json::Value::Bool(b) => b.to_string(),
                _ => String::new(),
            })
            .collect(),
        serde_json::Value::String(s) => vec![s],
        _ => Vec::new(),
    })
}

fn de_i64<'de, D>(d: D) -> Result<i64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let v = serde_json::Value::deserialize(d)?;
    Ok(match v {
        serde_json::Value::Number(n) => n.as_i64().unwrap_or(0),
        serde_json::Value::String(s) => s.trim().parse().unwrap_or(0),
        serde_json::Value::Bool(b) => i64::from(b),
        _ => 0,
    })
}

impl LlmCaption {
pub fn to_index_text(&self) -> String {
        let mut parts: Vec<String> = Vec::new();
        if !self.caption.trim().is_empty() {
            parts.push(self.caption.trim().to_string());
        }
        if !self.objects.is_empty() {
            parts.push(self.objects.join(" "));
        }
        if !self.scene.trim().is_empty() {
            parts.push(self.scene.trim().to_string());
        }
        if !self.action.trim().is_empty() {
            parts.push(self.action.trim().to_string());
        }
        if !self.relations.is_empty() {
            parts.push(self.relations.join(" "));
        }
        if !self.text.trim().is_empty() {
            parts.push(self.text.trim().to_string());
        }
        if self.people_count > 0 {
            parts.push(format!("people_count {}", self.people_count));
        }
        for (k, v) in &self.extra {
            if !v.trim().is_empty() {
                parts.push(format!("{} {}", k, v.trim()));
            }
        }
        parts.join(" ")
    }
}

// ============================================================
// [comment]
// ============================================================

fn is_cjk(ch: char) -> bool {
    matches!(ch as u32,
        0x4E00..=0x9FFF      // CJK Unified Ideographs
        | 0x3400..=0x4DBF    // Extension A
        | 0xF900..=0xFAFF    // Compatibility Ideographs
    )
}

/// [comment]
/// 瑙勫垯锛?/// - CJK 姹夊瓧鎸?2 瀛楁粦鍔ㄧ獥鍙ｅ垏鍒嗭紙"鏃ヨ惤鏃跺垎" -> "鏃ヨ惤 钀芥椂 鏃跺垎"锛?/// - **濂囨暟闀垮害鏃惰ˉ杈撳嚭灏鹃儴鍗曞瓧**锛?娉曞浗鏂楃墰鐘? -> "娉曞浗 鍥芥枟 鏂楃墰 鐗涚姮 鐘?锛夛紝
/// [comment]
/// [comment]
/// [comment]
pub fn bigramize(s: &str) -> String {
    let mut out: Vec<String> = Vec::new();
    let mut cjk_buf: Vec<char> = Vec::new();
    let mut latin_buf = String::new();

    fn flush_latin(buf: &mut String, out: &mut Vec<String>) {
        if !buf.is_empty() {
            out.push(std::mem::take(buf));
        }
    }
    fn flush_cjk(buf: &mut Vec<char>, out: &mut Vec<String>) {
        if buf.is_empty() {
            return;
        }
        if buf.len() == 1 {
            out.push(buf[0].to_string());
        } else {
            for w in buf.windows(2) {
                out.push(w.iter().collect());
            }
            if buf.len() % 2 == 1 {
                out.push(buf[buf.len() - 1].to_string());
            }
        }
        buf.clear();
    }

    for ch in s.chars() {
        if is_cjk(ch) {
            flush_latin(&mut latin_buf, &mut out);
            cjk_buf.push(ch);
        } else if ch.is_alphanumeric() {
            flush_cjk(&mut cjk_buf, &mut out);
            latin_buf.push(ch);
        } else {
            flush_latin(&mut latin_buf, &mut out);
            flush_cjk(&mut cjk_buf, &mut out);
        }
    }
    flush_latin(&mut latin_buf, &mut out);
    flush_cjk(&mut cjk_buf, &mut out);

    out.join(" ")
}

/// 鎶婄敤鎴锋煡璇㈣浆鎴?FTS5 MATCH 琛ㄨ揪寮忋€?///
/// [comment]
/// [comment]
/// [comment]
/// 鎵€浠?`MATCH '"鐚?'` 姘歌繙鏌ヤ笉鍒颁笢瑗裤€傞暱搴?鈮?2 鐨勬煡璇㈤渶璋冪敤鏂规敼璧?/// `search_caption_substring` 鐢?LIKE 鍏滃簳锛屽惁鍒欑敤鎴锋悳"鐚?"鐙?
pub fn query_to_fts(q: &str) -> String {
    let toks: Vec<String> = bigramize(q)
        .split_whitespace()
        .map(|t| format!("\"{}\"", t.replace('"', "")))
        .collect();
    if toks.is_empty() {
        // [comment]
        return "\"\"".to_string();
    }
    toks.join(" OR ")
}

// ============================================================
// Prompt
// ============================================================

/// [comment]
/// [comment]
/// prompt 鐗堟湰鍙枫€?///
/// FORK: prompt 版本号（索引/统计/失败记录共用）。
/// 字段定义变更通过清空索引触发重建（见 save_llm_config_cmd），
/// 因此版本号保持固定即可，无需进程内自增。
static PROMPT_VERSION: std::sync::atomic::AtomicI64 = std::sync::atomic::AtomicI64::new(1);

pub fn prompt_version() -> i64 {
    PROMPT_VERSION.load(std::sync::atomic::Ordering::SeqCst)
}

fn prompt_fields_eq(a: &PromptFields, b: &PromptFields) -> bool {
    match (serde_json::to_string(a), serde_json::to_string(b)) {
        (Ok(x), Ok(y)) => x == y,
        _ => false,
    }
}
// ============================================================
// HTTP 瀹㈡埛绔?// ============================================================

pub fn health_check(cfg: &LlmConfig) -> Result<(), String> {
    if cfg.base_url.trim().is_empty() {
        return Err("LLM base_url is empty".to_string());
    }
    let u = url::Url::parse(&cfg.base_url).map_err(|e| format!("bad base_url: {}", e))?;
    let host = u
        .host_str()
        .ok_or_else(|| "base_url has no host".to_string())?
        .to_string();
    let port = u.port().unwrap_or(80);
    std::net::TcpStream::connect_timeout(
        &format!("{}:{}", host, port)
            .parse()
            .map_err(|e| format!("bad address: {}", e))?,
        Duration::from_secs(5),
    )
    .map_err(|e| format!("LLM service unreachable at {}:{}: {}", host, port, e))?;
    Ok(())
}

#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<serde_json::Value>,
    max_tokens: u32,
    temperature: f32,
#[serde(skip_serializing_if = "Option::is_none")]
    chat_template_kwargs: Option<serde_json::Value>,
}

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<ChatChoice>,
}

#[derive(Deserialize)]
struct ChatChoice {
    message: ChatMessage,
}

#[derive(Deserialize)]
struct ChatMessage {
    content: Option<String>,
}

/// 璋冪敤 VLM 鐢熸垚 caption銆?///
/// [comment]
pub fn request_caption(cfg: &LlmConfig, image_b64: &str) -> Result<LlmCaption, String> {
    let url = format!("{}/v1/chat/completions", cfg.base_url.trim_end_matches('/'));

    let req = ChatRequest {
        model: cfg.model.clone(),
        messages: vec![serde_json::json!({
            "role": "user",
            "content": [
                { "type": "image_url", "image_url": { "url": format!("data:image/jpeg;base64,{}", image_b64) } },
                { "type": "text", "text": build_index_prompt(&cfg.prompt_fields) }
            ]
        })],
        max_tokens: cfg.max_tokens,
        temperature: cfg.temperature,
        chat_template_kwargs: if cfg.disable_thinking {
            Some(serde_json::json!({ "enable_thinking": false }))
        } else {
            None
        },
    };

    let body = serde_json::to_vec(&req).map_err(|e| format!("serialize request: {}", e))?;
    let resp = http_post(&url, &body, cfg)?;
    let parsed: ChatResponse =
        serde_json::from_slice(&resp).map_err(|e| format!("parse response: {}", e))?;

    let content = parsed
        .choices
        .first()
        .and_then(|c| c.message.content.clone())
        .unwrap_or_default();

    parse_caption(&content).ok_or_else(|| {
        let snippet: String = content.chars().take(200).collect();
        format!(
            "caption JSON parse failed (len={}, raw: {})",
            content.len(),
            snippet
        )
    })
}

/// [comment]
/// [comment]
fn http_post(url: &str, body: &[u8], cfg: &LlmConfig) -> Result<Vec<u8>, String> {
    use std::io::{Read, Write};
    use std::net::TcpStream;

    let u = url::Url::parse(url).map_err(|e| format!("bad url: {}", e))?;
    let host = u
        .host_str()
        .ok_or_else(|| "url has no host".to_string())?
        .to_string();
    let port = u.port().unwrap_or(80);
    let path = if u.path().is_empty() { "/" } else { u.path() };
    let target = match u.query() {
        Some(q) => format!("{}?{}", path, q),
        None => path.to_string(),
    };

    let stream = TcpStream::connect((host.as_str(), port))
        .map_err(|e| format!("connect {}:{} failed: {}", host, port, e))?;
    stream
        .set_read_timeout(Some(Duration::from_secs(cfg.timeout_secs)))
        .map_err(|e| format!("set read timeout: {}", e))?;
    stream
        .set_write_timeout(Some(Duration::from_secs(30)))
        .map_err(|e| format!("set write timeout: {}", e))?;

    let mut head = format!(
        "POST {} HTTP/1.1\r\nHost: {}:{}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n",
        target, host, port, body.len()
    );
    if !cfg.api_key.trim().is_empty() {
        head.push_str(&format!("Authorization: Bearer {}\r\n", cfg.api_key.trim()));
    }
    head.push_str("\r\n");

    let mut stream = stream;
    stream
        .write_all(head.as_bytes())
        .and_then(|_| stream.write_all(body))
        .map_err(|e| format!("write request: {}", e))?;
    stream.flush().ok();

    let mut raw = Vec::new();
    stream
        .read_to_end(&mut raw)
        .map_err(|e| format!("read response: {}", e))?;

    let split = find_subslice(&raw, b"\r\n\r\n")
        .ok_or_else(|| "malformed HTTP response".to_string())?;
    let head_str = String::from_utf8_lossy(&raw[..split]).to_string();
    let body_bytes = &raw[(split + 4).min(raw.len())..];

    let status = head_str
        .lines()
        .next()
        .unwrap_or("")
        .split_whitespace()
        .nth(1)
        .and_then(|s| s.parse::<u16>().ok())
        .unwrap_or(0);
    if status < 200 || status >= 300 {
        let snippet: String = String::from_utf8_lossy(body_bytes).chars().take(300).collect();
        return Err(format!("HTTP {}: {}", status, snippet));
    }

    if head_str.to_lowercase().contains("transfer-encoding: chunked") {
        Ok(dechunk(body_bytes))
    } else {
        Ok(body_bytes.to_vec())
    }
}

fn find_subslice(hay: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() || hay.len() < needle.len() {
        return None;
    }
    hay.windows(needle.len()).position(|w| w == needle)
}

fn dechunk(input: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < input.len() {
        let line_end = match find_subslice(&input[i..], b"\r\n") {
            Some(p) => i + p,
            None => break,
        };
        let size_hex = String::from_utf8_lossy(&input[i..line_end]);
        let size_hex = size_hex.split(';').next().unwrap_or("").trim();
        let size = usize::from_str_radix(size_hex, 16).unwrap_or(0);
        if size == 0 {
            break;
        }
        let data_start = line_end + 2;
        let data_end = (data_start + size).min(input.len());
        out.extend_from_slice(&input[data_start..data_end]);
        i = data_end + 2;
    }
    out
}

/// 瑙ｆ瀽妯″瀷杈撳嚭涓?LlmCaption銆?///
/// 瀹归敊绛栫暐锛堟寜 POC 涓庡疄娴嬭俯鍧戞暣鐞嗭級锛?/// 1. 鍘绘帀 ` ```json ` / ` ``` ` 鍥存爮
/// [comment]
/// [comment]
pub fn parse_caption(raw: &str) -> Option<LlmCaption> {
    let mut t = raw.trim();

    // 1. 鍘绘帀 markdown 浠ｇ爜鍥存爮
    if t.starts_with("```") {
        let after = t.splitn(2, '\n').nth(1).unwrap_or("");
        t = after.trim_end().trim_end_matches("```").trim();
    }

    // [comment]
    if let (Some(start), Some(end)) = (t.find('{'), t.rfind('}')) {
        if end > start {
            t = &t[start..=end];
        }
    }

    serde_json::from_str::<LlmCaption>(t).ok()
}

// ============================================================
// 鏁版嵁搴?// ============================================================

pub fn ensure_schema(conn: &rusqlite::Connection) -> Result<(), String> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS afile_llm (
            file_id    INTEGER PRIMARY KEY,
            caption    TEXT NOT NULL DEFAULT '',
            objects    TEXT NOT NULL DEFAULT '',
            scene      TEXT NOT NULL DEFAULT '',
            action     TEXT NOT NULL DEFAULT '',
            relations  TEXT NOT NULL DEFAULT '',
            ocr_text   TEXT NOT NULL DEFAULT '',
            people_cnt INTEGER NOT NULL DEFAULT 0,
            index_text TEXT NOT NULL DEFAULT '',
            model      TEXT NOT NULL DEFAULT '',
            prompt_ver INTEGER NOT NULL DEFAULT 1,
            indexed_at INTEGER NOT NULL DEFAULT 0
        );
        CREATE INDEX IF NOT EXISTS idx_afile_llm_indexed ON afile_llm(indexed_at);",
    )
    .map_err(|e| format!("ensure llm schema: {}", e))?;

    conn.execute_batch(
        "CREATE VIRTUAL TABLE IF NOT EXISTS afile_llm_fts USING fts5(
            index_text,
            content='',
            contentless_delete=1,
            tokenize='unicode61'
        );

        -- FORK: failure log. Without it, failed files leave no trace and
        -- pending_files re-fetches them forever (observed: 110 photos, 1 hour,
        -- current=1340, failed=1331).
        -- fail_streak counts consecutive failures; past the cap no auto-retry.
        CREATE TABLE IF NOT EXISTS afile_llm_fail (
            file_id    INTEGER PRIMARY KEY,
            model      TEXT NOT NULL,
            prompt_ver INTEGER NOT NULL,
            error      TEXT NOT NULL DEFAULT '',
            fail_streak INTEGER NOT NULL DEFAULT 1,
            failed_at  INTEGER NOT NULL DEFAULT 0
        );",
    )
    .map_err(|e| {
        format!(
            "ensure llm fts (FTS5 not enabled? {}). If bundled SQLite lacks FTS5, \
             define SQLITE_ENABLE_FTS5 in build.rs",
            e
        )
    })?;

    conn.execute(
        "DELETE FROM afile_llm WHERE file_id NOT IN (SELECT id FROM afiles)",
        [],
    )
    .ok();
    conn.execute(
        "DELETE FROM afile_llm_fail WHERE file_id NOT IN (SELECT id FROM afiles)",
        [],
    )
    .ok();
    conn.execute(
        "DELETE FROM afile_llm_fts WHERE rowid NOT IN (SELECT id FROM afiles)",
        [],
    )
    .ok();

    Ok(())
}

const MAX_FAIL_STREAK: i64 = 3;

pub fn store_failure(
    conn: &rusqlite::Connection,
    file_id: i64,
    cfg: &LlmConfig,
    error: &str,
) -> Result<(), String> {
    ensure_schema(conn)?;
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    conn.execute(
        "INSERT INTO afile_llm_fail (file_id, model, prompt_ver, error, fail_streak, failed_at)
         VALUES (?1, ?2, ?3, ?4, 1, ?5)
         ON CONFLICT(file_id) DO UPDATE SET
             model=excluded.model,
             prompt_ver=excluded.prompt_ver,
             error=excluded.error,
             fail_streak=fail_streak + 1,
             failed_at=excluded.failed_at",
        rusqlite::params![file_id, cfg.model, prompt_version(), error, now],
    )
    .map_err(|e| format!("record llm failure: {}", e))?;
    Ok(())
}

pub fn clear_failure(conn: &rusqlite::Connection, file_id: i64) -> Result<(), String> {
    conn.execute(
        "DELETE FROM afile_llm_fail WHERE file_id = ?1",
        rusqlite::params![file_id],
    )
    .map_err(|e| format!("clear llm failure: {}", e))?;
    Ok(())
}

/// 鍙栧緟澶勭悊鏂囦欢锛氳繕娌＄储寮曠殑锛屾垨妯″瀷/prompt 鐗堟湰鍙樻洿闇€閲嶅缓鐨勩€?///
pub fn pending_files(
    conn: &rusqlite::Connection,
    cfg: &LlmConfig,
    limit: i64,
) -> Result<Vec<(i64, i64, String)>, String> {
    ensure_schema(conn)?;
    let mut stmt = conn
        .prepare(
            "SELECT a.id, a.file_type, f.path || '/' || a.name AS path
             FROM afiles a
             JOIN afolders f ON f.id = a.folder_id
             LEFT JOIN afile_llm l ON l.file_id = a.id
             LEFT JOIN afile_llm_fail x ON x.file_id = a.id
             WHERE a.file_type IN (1, 2, 3)
                AND (l.file_id IS NULL OR l.model != ?1 OR l.prompt_ver != ?2)
                AND (x.file_id IS NULL OR x.fail_streak < ?4)
             ORDER BY a.id
             LIMIT ?3",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(
            rusqlite::params![cfg.model, prompt_version(), limit.max(0), MAX_FAIL_STREAK],
            |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, String>(2)?,
                ))
            },
        )
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    Ok(rows)
}

pub fn store_caption(
    conn: &rusqlite::Connection,
    file_id: i64,
    cap: &LlmCaption,
    cfg: &LlmConfig,
) -> Result<(), String> {
    ensure_schema(conn)?;
    let index_text = bigramize(&cap.to_index_text());
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);

    let tx = conn
        .unchecked_transaction()
        .map_err(|e| format!("begin tx: {}", e))?;

    tx.execute(
        "DELETE FROM afile_llm_fts WHERE rowid = ?1",
        rusqlite::params![file_id],
    )
    .ok();

    tx.execute(
        "INSERT INTO afile_llm (
            file_id, caption, objects, scene, action, relations, ocr_text,
            people_cnt, index_text, model, prompt_ver, indexed_at
         ) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12)
         ON CONFLICT(file_id) DO UPDATE SET
            caption=excluded.caption, objects=excluded.objects,
            scene=excluded.scene, action=excluded.action,
            relations=excluded.relations, ocr_text=excluded.ocr_text,
            people_cnt=excluded.people_cnt, index_text=excluded.index_text,
            model=excluded.model, prompt_ver=excluded.prompt_ver,
            indexed_at=excluded.indexed_at",
        rusqlite::params![
            file_id,
            cap.caption,
            cap.objects.join(" "),
            cap.scene,
            cap.action,
            cap.relations.join(" "),
            cap.text,
            cap.people_count,
            index_text,
            cfg.model,
            prompt_version(),
            now,
        ],
    )
    .map_err(|e| format!("insert llm row: {}", e))?;

    tx.execute(
        "INSERT INTO afile_llm_fts (rowid, index_text) VALUES (?1, ?2)",
        rusqlite::params![file_id, index_text],
    )
    .map_err(|e| format!("insert llm fts: {}", e))?;

    tx.commit().map_err(|e| format!("commit tx: {}", e))?;
    Ok(())
}

/// [comment]
/// [comment]
/// # 闄嶇骇琛屼负
///
pub fn merge_caption_scores(
    conn: &rusqlite::Connection,
    params: &crate::t_sqlite::ImageSearchParams,
    scores: Vec<(i64, f32)>,
) -> Vec<(i64, f32)> {
    let query = params.search_text.trim();
    if query.is_empty() {
        return scores;
    }

    let cfg = match load_llm_config() {
        Ok(c) => c,
        Err(_) => return scores, // 閰嶇疆璇讳笉鍒?-> 闈欓粯闄嶇骇
    };
    if !cfg.enabled {
        return scores;
    }

    merge_with_caption_scores(conn, query, scores)
}

/// [comment]
/// [comment]
/// CLIP 鐨勪綑寮︾浉浼煎害涓?FTS5 鐨?`rank` 閲忕翰瀹屽叏涓嶅悓锛堝墠鑰呮槸 [0,1] 鐨勪綑寮︼紝
/// [comment]
/// 杩欓噷鎶婁袱杈?*鍚勮嚜褰掍竴鍖栧埌 [0,1]** 鍐嶅姞鏉冿細
/// [comment]
///
/// [comment]
/// caption 鏄?琛ュ厖绾跨储"锛屼笉鏄?杩囨护鍣?銆侰LIP 鍛戒腑鐨勭粨鏋滃叏閮ㄤ繚鐣欙紝
/// [comment]
pub fn merge_with_caption_scores(
    conn: &rusqlite::Connection,
    query: &str,
    scores: Vec<(i64, f32)>,
) -> Vec<(i64, f32)> {
    // caption 鍛戒腑鍒楄〃锛堟寜 FTS5 rank 鎺掑簭锛夈€備笂闄愬彇 CLIP 缁撴灉鏁?+ 涓€瀹氫綑閲忥紝
    let cap_limit = (scores.len() as i64) + 64;
    let caption_ids = match search_caption(conn, query, cap_limit) {
        Ok(ids) => ids,
        Err(_) => return scores, // FTS query failed -> silent degrade
    };
    if caption_ids.is_empty() {
        return scores;
    }

const CLIP_WEIGHT: f32 = 0.7;
    const CAPTION_WEIGHT: f32 = 0.3;

    // [comment]
    let n = caption_ids.len() as f32;
    let mut caption_score: std::collections::HashMap<i64, f32> =
        std::collections::HashMap::with_capacity(caption_ids.len());
    for (i, id) in caption_ids.iter().enumerate() {
        let pos_score = (n - i as f32) / n;
        caption_score.insert(*id, pos_score);
    }

    let mut merged: std::collections::HashMap<i64, f32> =
        std::collections::HashMap::with_capacity(scores.len() + caption_ids.len());

    for (id, clip_score) in &scores {
        let cap = caption_score.get(id).copied().unwrap_or(0.0);
        merged.insert(*id, clip_score * CLIP_WEIGHT + cap * CAPTION_WEIGHT);
    }
    for id in &caption_ids {
        if !merged.contains_key(id) {
            // [comment]
            let cap = caption_score.get(id).copied().unwrap_or(0.0);
            merged.insert(*id, cap * CAPTION_WEIGHT);
        }
    }

    let mut result: Vec<(i64, f32)> = merged.into_iter().collect();
    result.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    result
}

/// [comment]
/// [comment]
pub fn needs_substring_fallback(q: &str) -> bool {
    let cjk_count = q.chars().filter(|c| is_cjk(*c)).count();
    cjk_count <= 2
}

/// [comment]
pub fn search_caption_substring(
    conn: &rusqlite::Connection,
    query: &str,
    limit: i64,
) -> Result<Vec<i64>, String> {
    ensure_schema(conn)?;
    let q = query.trim();
    if q.is_empty() {
        return Ok(Vec::new());
    }
    // [comment]
    let escaped = q.replace('\\', "\\\\").replace('%', "\\%").replace('_', "\\_");
    let pattern = format!("%{}%", escaped);
    let mut stmt = conn
        .prepare(
            "SELECT file_id FROM afile_llm
             WHERE index_text LIKE ?1 ESCAPE '\\'
             ORDER BY indexed_at DESC, file_id DESC
             LIMIT ?2",
        )
        .map_err(|e| e.to_string())?;
    let ids = stmt
        .query_map(rusqlite::params![pattern, limit.max(0)], |row| {
            row.get::<_, i64>(0)
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    Ok(ids)
}

/// [comment]
/// [comment]
/// 鏍稿績閫昏緫鍦?`rerank_with_cfg`銆?///
/// # 闄嶇骇
///
pub fn rerank_scores(
    conn: &rusqlite::Connection,
    params: &crate::t_sqlite::ImageSearchParams,
    scores: Vec<(i64, f32)>,
) -> Vec<(i64, f32)> {
    let query = params.search_text.trim();
    if query.is_empty() || scores.is_empty() {
        return scores;
    }
    // FORK: 三重开关——本次请求开启 + VLM 索引启用。
    // 前端「精确搜索」勾选框（params.enable_rerank）是唯一显式开关；
    // 配置里的 enable_rerank 只作为默认值，这里被请求参数覆盖，
    // 否则勾了也无效（配置默认 false 会把它拦掉）。
    if !params.enable_rerank {
        return scores;
    }
    let mut cfg = match load_llm_config() {
        Ok(c) => c,
        Err(_) => return scores,
    };
    if !cfg.enabled {
        return scores;
    }
    cfg.enable_rerank = true;
    rerank_with_cfg(conn, &cfg, query, scores)
}

/// [comment]
/// # 涓轰粈涔堝€煎緱鍋?///
/// [comment]
/// [comment]
/// [comment]
/// # 涓轰粈涔堥粯璁ゅ叧闂?///
/// [comment]
/// [comment]
/// # 娣峰悎鏉冮噸
///
/// [comment]
pub fn rerank_with_cfg(
    conn: &rusqlite::Connection,
    cfg: &LlmConfig,
    query: &str,
    scores: Vec<(i64, f32)>,
) -> Vec<(i64, f32)> {
    if !cfg.enable_rerank || !cfg.enabled {
        return scores;
    }
    let query = query.trim();
    if query.is_empty() || scores.is_empty() {
        return scores;
    }

    let top_n = cfg.rerank_top_n.clamp(1, 100) as usize;
    let (head, tail) = if scores.len() > top_n {
        (scores[..top_n].to_vec(), scores[top_n..].to_vec())
    } else {
        (scores.clone(), Vec::new())
    };

    let mut reranked: Vec<(i64, f32)> = Vec::with_capacity(head.len());
    for (id, orig_score) in &head {
        let caption = get_caption_text(conn, *id).unwrap_or_default();
        let relevance = score_relevance(cfg, query, &caption).unwrap_or(*orig_score);
        let final_score = orig_score * 0.3 + relevance * 0.7;
        reranked.push((*id, final_score));
    }

    reranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    reranked.extend(tail);
    reranked
}

fn get_caption_text(conn: &rusqlite::Connection, file_id: i64) -> Option<String> {
    conn.query_row(
        "SELECT caption, objects, scene, action, relations, ocr_text
         FROM afile_llm WHERE file_id = ?1",
        rusqlite::params![file_id],
        |row| {
            let caption: String = row.get(0)?;
            let objects: String = row.get(1)?;
            let scene: String = row.get(2)?;
            let action: String = row.get(3)?;
            let relations: String = row.get(4)?;
            let ocr: String = row.get(5)?;
            Ok([caption, objects, scene, action, relations, ocr].join(" "))
        },
    )
    .ok()
}

const RERANK_PROMPT_PREFIX: &str = "\u{4f60}\u{662f}\u{7167}\u{7247}\u{641c}\u{7d22}\u{76f8}\u{5173}\u{6027}\u{5224}\u{65ad}\u{5668}\u{3002}\u{7ed9}\u{5b9a}\u{7528}\u{6237}\u{7684}\u{641c}\u{7d22}\u{67e5}\u{8be2}\u{548c}\u{4e00}\u{5f20}\u{7167}\u{7247}\u{7684}\u{5185}\u{5bb9}\u{63cf}\u{8ff0}\u{ff0c}\u{5224}\u{65ad}\u{8fd9}\u{5f20}\u{7167}\u{7247}\u{662f}\u{5426}\u{662f}\u{7528}\u{6237}\u{60f3}\u{627e}\u{7684}\u{3002}

\u{89c4}\u{5219}\u{ff1a}
- \u{53ea}\u{8f93}\u{51fa}\u{4e00}\u{4e2a} 0 \u{5230} 1 \u{4e4b}\u{95f4}\u{7684}\u{5c0f}\u{6570}\u{ff0c}\u{4e0d}\u{8981}\u{8f93}\u{51fa}\u{4efb}\u{4f55}\u{5176}\u{4ed6}\u{6587}\u{5b57}
- 1.0 = \u{5b8c}\u{5168}\u{7b26}\u{5408}\u{67e5}\u{8be2}\u{610f}\u{56fe}
- 0.5 = \u{90e8}\u{5206}\u{76f8}\u{5173}\u{ff0c}\u{53ef}\u{80fd}\u{662f}\u{7528}\u{6237}\u{60f3}\u{770b}\u{7684}
- 0.0 = \u{5b8c}\u{5168}\u{4e0d}\u{76f8}\u{5173}
- \u{62ff}\u{4e0d}\u{51c6}\u{65f6}\u{7ed9} 0.3 \u{5de6}\u{53f3}\u{7684}\u{4e2d}\u{95f4}\u{503c}\u{ff0c}\u{4e0d}\u{8981}\u{6781}\u{7aef}

";

/// 璋冪敤 VLM 鍒ゆ柇鍗曚釜鍊欓€夌殑鐩稿叧搴︼紝杩斿洖 [0,1]銆?///
fn score_relevance(cfg: &LlmConfig, query: &str, caption: &str) -> Option<f32> {
    if caption.trim().is_empty() {
        return None;
    }

    let prompt = format!(
        "{}\u{641c}\u{7d22}\u{67e5}\u{8be2}\u{ff1a}{}
\u{7167}\u{7247}\u{5185}\u{5bb9}\u{63cf}\u{8ff0}\u{ff1a}{}

\u{76f8}\u{5173}\u{5ea6}\u{5206}\u{6570}\u{ff1a}",
        RERANK_PROMPT_PREFIX, query, caption
    );

    let payload = serde_json::json!({
        "model": cfg.model,
        "messages": [{ "role": "user", "content": prompt }],
        "max_tokens": 8,
        "temperature": 0.0,
    });
    let body = serde_json::to_vec(&payload).ok()?;
    let url = format!("{}/v1/chat/completions", cfg.base_url.trim_end_matches('/'));
    let resp = http_post(&url, &body, cfg).ok()?;
    let parsed: ChatResponse = serde_json::from_slice(&resp).ok()?;
    let text = parsed.choices.first()?.message.content.clone()?;

    parse_relevance(&text)
}

/// 浠庢ā鍨嬭緭鍑鸿В鏋?0-1 鐨勫垎鏁般€?///
/// [comment]
/// [comment]
/// 涓や釜瀹规槗韪╃殑鍧戯紙閮芥湁鍥炲綊娴嬭瘯锛夛細
/// [comment]
pub fn parse_relevance(raw: &str) -> Option<f32> {
    let chars: Vec<char> = raw.chars().collect();
    let n = chars.len();
    let mut i = 0;
    while i < n {
        let start = i;
        let mut j = i;
        let has_sign = chars[j] == '-' || chars[j] == '+';
        if has_sign {
            j += 1;
        }
        let digit_start = j < n && chars[j].is_ascii_digit();
        let dot_start = j + 1 < n && chars[j] == '.' && chars[j + 1].is_ascii_digit();
        if !digit_start && !dot_start {
            i += 1;
            continue;
        }
        while j < n && chars[j].is_ascii_digit() {
            j += 1;
        }
        if j < n && chars[j] == '.' {
            j += 1;
            while j < n && chars[j].is_ascii_digit() {
                j += 1;
            }
        }
        let token: String = chars[start..j].iter().collect();
        if let Ok(v) = token.parse::<f32>() {
            // [comment]
            if (0.0..=1.0).contains(&v) {
                return Some(v);
            }
        }
        // [comment]
        i = j.max(start + 1);
    }
    None
}

/// [comment]
pub fn search_caption(
    conn: &rusqlite::Connection,
    query: &str,
    limit: i64,
) -> Result<Vec<i64>, String> {
    ensure_schema(conn)?;

    if needs_substring_fallback(query) {
        return search_caption_substring(conn, query, limit);
    }

    let fts_q = query_to_fts(query);
    let mut stmt = conn
        .prepare(
            "SELECT rowid FROM afile_llm_fts
             WHERE afile_llm_fts MATCH ?1
             ORDER BY rank
             LIMIT ?2",
        )
        .map_err(|e| e.to_string())?;
    let ids = stmt
        .query_map(rusqlite::params![fts_q, limit.max(0)], |row| {
            row.get::<_, i64>(0)
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    Ok(ids)
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LlmIndexStats {
    pub indexed: i64,
    pub pending: i64,
    pub total: i64,
}

pub fn get_stats(conn: &rusqlite::Connection, cfg: &LlmConfig) -> Result<LlmIndexStats, String> {
    ensure_schema(conn)?;
    let total: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM afiles WHERE file_type IN (1,2,3)",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);
    let indexed: i64 = conn
        .query_row(
            // [comment]
            "SELECT COUNT(*) FROM afile_llm l
             JOIN afiles a ON a.id = l.file_id
             WHERE l.model = ?1 AND l.prompt_ver = ?2",
            rusqlite::params![cfg.model, prompt_version()],
            |r| r.get(0),
        )
        .unwrap_or(0);
    Ok(LlmIndexStats {
        indexed,
        pending: (total - indexed).max(0),
        total,
    })
}

// ============================================================
// ============================================================

/// [comment]
const VIDEO_FRAME_PERCENTS: [u8; 3] = [10, 50, 90];

/// [comment]
/// [comment]
/// [comment]
async fn prepare_video_frames_b64(file_path: &str, cfg: &LlmConfig) -> Result<Option<String>, String> {
    use crate::t_video;

    let duration = t_video::get_video_duration(file_path).await.ok();
    // [comment]
    let dur = duration.unwrap_or(0);
    if dur > 0 && dur < 3 {
        return Ok(None);
    }

    let frame_px = cfg.preview_max_px.max(64).min(512);
    let mut frames: Vec<image::RgbImage> = Vec::with_capacity(VIDEO_FRAME_PERCENTS.len());

    for percent in VIDEO_FRAME_PERCENTS {
        let bytes = t_video::get_video_thumbnail(file_path, frame_px, duration, Some(percent)).await;
        let bytes = match bytes {
            Ok(Some(b)) => b,
            _ => continue, // 鍗曞抚澶辫触涓嶉樆濉炲叾浠栧抚
        };
        match image::load_from_memory(&bytes) {
            Ok(img) => frames.push(img.to_rgb8()),
            Err(_) => continue,
        }
    }

    if frames.is_empty() {
        return Ok(None);
    }

    // [comment]
    let target_h = frames.iter().map(|f| f.height()).min().unwrap_or(frame_px).max(1);
    let gap = 4_u32;
    let scaled: Vec<image::RgbImage> = frames
        .iter()
        .map(|f| {
            let nw = ((f.width() as f32) * (target_h as f32 / f.height() as f32)).round().max(1.0) as u32;
            image::imageops::resize(f, nw, target_h, image::imageops::FilterType::Triangle)
        })
        .collect();
    let total_w: u32 = scaled.iter().map(|f| f.width()).sum::<u32>()
        + gap * (scaled.len().saturating_sub(1) as u32);

    let mut canvas = image::RgbImage::new(total_w.max(1), target_h);
    let mut x = 0_u32;
    for f in &scaled {
        for (px, py, pixel) in f.enumerate_pixels() {
            if x + px < canvas.width() {
                canvas.put_pixel(x + px, py, *pixel);
            }
        }
        x += f.width() + gap;
    }

    let mut buf = Vec::new();
    let mut cursor = std::io::Cursor::new(&mut buf);
    image::DynamicImage::ImageRgb8(canvas)
        .write_to(&mut cursor, image::ImageFormat::Jpeg)
        .map_err(|e| format!("encode video frames: {}", e))?;

    use base64::Engine;
    Ok(Some(base64::engine::general_purpose::STANDARD.encode(&buf)))
}

/// `prepare_video_frames_b64` 鐨勫悓姝ュ寘瑁呫€?///
fn prepare_video_frames_b64_sync(file_path: &str, cfg: &LlmConfig) -> Result<Option<String>, String> {
    let handle = tokio::runtime::Handle::try_current().map_err(|_| "No tokio runtime".to_string())?;
    tokio::task::block_in_place(|| handle.block_on(prepare_video_frames_b64(file_path, cfg)))
}

/// 璇诲彇骞跺帇缂╁浘鐗囦负 JPEG base64銆?///
/// 浼樺厛澶嶇敤 Lap 宸茬敓鎴愮殑缂╃暐鍥撅紝閬垮厤涓烘瘡寮犲浘瑙ｇ爜鍘熷浘 鈥斺€?杩欎釜浼樺寲
/// [comment]
/// [comment]
pub fn prepare_image_b64(
    file_id: i64,
    file_path: &str,
    cfg: &LlmConfig,
) -> Result<String, String> {
    let img = match crate::t_sqlite::AThumb::fetch(file_id) {
        Ok(Some(thumb)) if thumb.thumb_data.is_some() => {
            let bytes = thumb.thumb_data.as_ref().unwrap();
            image::load_from_memory(bytes).map_err(|e| format!("load cached thumb: {}", e))?
        }
        _ => image::open(file_path).map_err(|e| format!("open image {}: {}", file_path, e))?,
    };

    // 淇濇寔 DynamicImage 褰㈡€? resize 鍚庝粛鐢?DynamicImage 鐨?write_to,
    let rgb = img.to_rgb8();
    let (w, h) = rgb.dimensions();
    let max_px = cfg.preview_max_px.max(64) as f32;
    let scale = (max_px / w.max(h) as f32).min(1.0);
    let resized = if scale < 1.0 {
        let nw = (w as f32 * scale).round().max(1.0) as u32;
        let nh = (h as f32 * scale).round().max(1.0) as u32;
        image::DynamicImage::ImageRgb8(image::imageops::resize(
            &rgb,
            nw,
            nh,
            image::imageops::FilterType::Triangle,
        ))
    } else {
        image::DynamicImage::ImageRgb8(rgb)
    };

    let mut buf = Vec::new();
    let mut cursor = std::io::Cursor::new(&mut buf);
    resized
        .write_to(&mut cursor, image::ImageFormat::Jpeg)
        .map_err(|e| format!("encode jpeg: {}", e))?;

    use base64::Engine;
    Ok(base64::engine::general_purpose::STANDARD.encode(&buf))
}

// ============================================================
// 绱㈠紩璋冨害
// ============================================================

fn is_cancelled(cancel: &Arc<Mutex<bool>>) -> bool {
    cancel.lock().map(|c| *c).unwrap_or(false)
}

/// [comment]
/// [comment]
/// [comment]
pub fn run_llm_indexing(
    app_handle: AppHandle,
    llm_state: LlmState,
    cancel_token: LlmIndexCancellation,
    status_token: LlmIndexingStatus,
    progress_token: LlmIndexProgressState,
) -> Result<(), String> {
    let cancel = cancel_token.0.clone();
    let status = status_token.0.clone();
    let progress = progress_token.0.clone();

    {
        let mut running = status.lock().map_err(|e| e.to_string())?;
        if *running {
            return Err("LLM indexing is already running".to_string());
        }
        *running = true;
    }
    *cancel.lock().map_err(|e| e.to_string())? = false;

    // 閲嶇疆杩涘害锛堝湪 spawn 澶栧仛锛屼繚璇佸悓姝ョ敓鏁堬級
    {
        if let Ok(mut p) = progress.lock() {
            *p = LlmIndexProgress::default();
            p.phase = "indexing".to_string();
        }
    }

    tauri::async_runtime::spawn_blocking(move || {
        // [comment]
        let reset = || {
            if let Ok(mut p) = progress.lock() {
                *p = LlmIndexProgress::default();
                p.phase = "indexing".to_string();
            }
            if let Ok(mut r) = status.lock() {
                *r = false;
            }
        };

        let cfg = match llm_state.0.lock() {
            Ok(c) => c.clone(),
            Err(e) => {
                eprintln!("[llm] lock config failed: {}", e);
                let _ = app_handle.emit(
                    "llm_index_finished",
                    serde_json::json!({ "error": e.to_string(), "cancelled": false }),
                );
                reset();
                return;
            }
        };

        if !cfg.enabled {
            let _ = app_handle.emit(
                "llm_index_finished",
                serde_json::json!({ "error": "LLM indexing is disabled", "cancelled": false }),
            );
            reset();
            return;
        }

        // [comment]
        if let Err(e) = health_check(&cfg) {
            eprintln!("[llm] health check failed: {}", e);
            let _ = app_handle.emit(
                "llm_index_finished",
                serde_json::json!({ "error": e, "cancelled": false }),
            );
            reset();
            return;
        }

        let conn = match crate::t_sqlite::open_conn() {
            Ok(c) => c,
            Err(e) => {
                let _ = app_handle.emit(
                    "llm_index_finished",
                    serde_json::json!({ "error": e.to_string(), "cancelled": false }),
                );
                reset();
                return;
            }
        };

        const BATCH: i64 = 64;
        let mut current: u64 = 0;
        let mut succeeded: u64 = 0;
        let mut failed: u64 = 0;
        let mut cancelled_flag = false;

        loop {
            if is_cancelled(&cancel) {
                cancelled_flag = true;
                break;
            }

            let batch = match pending_files(&conn, &cfg, BATCH) {
                Ok(b) => b,
                Err(e) => {
                    eprintln!("[llm] pending_files failed: {}", e);
                    let _ = app_handle.emit(
                        "llm_index_finished",
                        serde_json::json!({ "error": e, "cancelled": false }),
                    );
                    reset();
                    return;
                }
            };

            if batch.is_empty() {
                break;
            }

            // 鎵瑰唴骞跺彂澶勭悊銆俿td::thread::scope 淇濊瘉绾跨▼鍦ㄦ壒缁撴潫鍓嶅叏閮?join,
            // [comment]
            // [comment]
            let workers = cfg.concurrency.clamp(1, 8) as usize;
            let chunk_size = ((batch.len() + workers - 1) / workers).max(1);
            let mut last_emit_failed: u64 = 0;

            let mut batch_results: Vec<(usize, bool)> = std::thread::scope(|scope| {
                let mut handles = Vec::new();
                let fail_errors: std::sync::Arc<std::sync::Mutex<Vec<(i64, String)>>> =
                    std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
                for (chunk_idx, chunk) in batch.chunks(chunk_size).enumerate() {
                    let cfg_ref = &cfg;
                    let cancel_ref = &cancel;
                    let fail_errors_ref = fail_errors.clone();
                    let handle = scope.spawn(move || {
                        let mut local: Vec<(usize, bool)> = Vec::with_capacity(chunk.len());
                        // [comment]
                        let thread_conn = crate::t_sqlite::open_conn().ok();
                        for (i, (file_id, file_type, path)) in chunk.iter().enumerate() {
                            if is_cancelled(cancel_ref) {
                                break;
                            }
                            let result: Result<(), String> = (|| {
                                let b64 = if *file_type == 2 {
                                    match prepare_video_frames_b64_sync(path, cfg_ref) {
                                        Ok(Some(b)) => b,
                                        Ok(None) => prepare_image_b64(*file_id, path, cfg_ref)?,
                                        Err(e) => {
                                            eprintln!("[llm] video frames {} failed: {}, fallback", file_id, e);
                                            prepare_image_b64(*file_id, path, cfg_ref)?
                                        }
                                    }
                                } else {
                                    prepare_image_b64(*file_id, path, cfg_ref)?
                                };
                                let cap = request_caption(cfg_ref, &b64)?;
                                let conn = thread_conn.as_ref().ok_or_else(|| "open db conn failed".to_string())?;
                                store_caption(conn, *file_id, &cap, cfg_ref)?;
                                clear_failure(conn, *file_id)?;
                                Ok(())
                            })();

                            match result {
                                Ok(()) => local.push((chunk_idx * chunk_size + i, true)),
                                Err(e) => {
                                    eprintln!("[llm] index file {} ({}) failed: {}", file_id, path, e);
                                    // [comment]
                                    if let Some(conn) = thread_conn.as_ref() {
                                        let _ = store_failure(conn, *file_id, cfg_ref, &e);
                                    }
                                    if let Ok(mut v) = fail_errors_ref.lock() {
                                        v.push((*file_id, e));
                                    }
                                    local.push((chunk_idx * chunk_size + i, false));
                                }
                            }
                        }
                        local
                    });
                    handles.push(handle);
                }
                let mut all = Vec::new();
                for h in handles {
                    if let Ok(local) = h.join() {
                        all.extend(local);
                    }
                }
                // 鎶婂け璐ュ師鍥犳殏瀛樺埌 progress锛屼緵姹囨€婚樁娈?emit
                if let Ok(v) = fail_errors.lock() {
                    if !v.is_empty() {
                        if let Ok(mut p) = progress.lock() {
                            p.last_error = v
                                .iter()
                                .rev()
                                .take(3)
                                .map(|(id, e)| format!("#{}: {}", id, e))
                                .collect::<Vec<_>>()
                                .join(" | ");
                        }
                    }
                }
                all
            });

            // [comment]
            batch_results.sort_by_key(|(idx, _)| *idx);
            for (_, ok) in batch_results {
                current += 1;
                if ok {
                    succeeded += 1;
                } else {
                    failed += 1;
                }
                if let Ok(mut p) = progress.lock() {
                    p.current = current;
                    p.succeeded = succeeded;
                    p.failed = failed;
                    p.phase = "indexing".to_string();
                }
                // [comment]
                if current % 10 == 0 || (failed > 0 && last_emit_failed != failed) {
                    if let Ok(p) = progress.lock() {
                        let _ = app_handle.emit("llm_index_progress", p.clone());
                    }
                    last_emit_failed = failed;
                }
            }

            if cancelled_flag {
                break;
            }
        }

        let _ = app_handle.emit(
            "llm_index_finished",
            serde_json::json!({
                "total": current,
                "succeeded": succeeded,
                "failed": failed,
                "cancelled": cancelled_flag
            }),
        );
        reset();
    });

    Ok(())
}

// ============================================================
// Tauri 鍛戒护
// ============================================================

#[tauri::command]
pub fn get_llm_config(state: tauri::State<LlmState>) -> Result<LlmConfig, String> {
    load_llm_config()
        .or_else(|_| state.0.lock().map(|c| c.clone()).map_err(|e| e.to_string()))
}

#[tauri::command]
pub fn save_llm_config_cmd(cfg: LlmConfig, state: tauri::State<LlmState>) -> Result<(), String> {
    // FORK: 字段定义变更时清空旧索引，触发全量重建（新字段才能被搜到）。
    let fields_changed = match state.0.lock() {
        Ok(cur) => !prompt_fields_eq(&cur.prompt_fields, &cfg.prompt_fields),
        Err(_) => false,
    };
    save_llm_config(&cfg)?;
    if fields_changed {
        if let Ok(conn) = crate::t_sqlite::open_conn() {
            let _ = ensure_schema(&conn);
            let _ = conn.execute("DELETE FROM afile_llm", []);
            let _ = conn.execute("DELETE FROM afile_llm_fts", []);
            let _ = conn.execute("DELETE FROM afile_llm_fail", []);
        }
    }
    if let Ok(mut cur) = state.0.lock() {
        *cur = cfg;
    }
    Ok(())
}

#[tauri::command]
pub fn test_llm_connection(state: tauri::State<LlmState>) -> Result<String, String> {
    let cfg = state.0.lock().map_err(|e| e.to_string())?.clone();
    health_check(&cfg)?;
    Ok(format!("connected to {} (model: {})", cfg.base_url, cfg.model))
}

#[tauri::command]
pub fn start_llm_indexing(
    app: AppHandle,
    llm_state: tauri::State<LlmState>,
    cancel: tauri::State<LlmIndexCancellation>,
    status: tauri::State<LlmIndexingStatus>,
    progress: tauri::State<LlmIndexProgressState>,
) -> Result<(), String> {
    run_llm_indexing(
        app,
        llm_state.inner().clone(),
        cancel.inner().clone(),
        status.inner().clone(),
        progress.inner().clone(),
    )
}

#[tauri::command]
pub fn cancel_llm_indexing(cancel: tauri::State<LlmIndexCancellation>) -> Result<(), String> {
    *cancel.0.lock().map_err(|e| e.to_string())? = true;
    Ok(())
}

#[tauri::command]
pub fn get_llm_index_stats(llm_state: tauri::State<LlmState>) -> Result<LlmIndexStats, String> {
    let cfg = llm_state.0.lock().map_err(|e| e.to_string())?.clone();
    let conn = crate::t_sqlite::open_conn().map_err(|e| e.to_string())?;
    get_stats(&conn, &cfg)
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LlmCaptionInfo {
    pub caption: String,
    pub objects: Vec<String>,
    pub scene: String,
    pub action: String,
    pub relations: Vec<String>,
    pub text: String,
    pub people_count: i64,
    pub indexed: bool,
}

#[tauri::command]
pub fn get_llm_caption(file_id: i64) -> Result<Option<LlmCaptionInfo>, String> {
    let conn = crate::t_sqlite::open_conn().map_err(|e| e.to_string())?;
    ensure_schema(&conn)?;

    let row = conn
        .query_row(
            "SELECT caption, objects, scene, action, relations, ocr_text, people_cnt
             FROM afile_llm WHERE file_id = ?1",
            rusqlite::params![file_id],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, String>(5)?,
                    row.get::<_, i64>(6)?,
                ))
            },
        )
        .ok();

    Ok(row.map(
        |(caption, objects, scene, action, relations, text, people_count)| LlmCaptionInfo {
            caption,
            objects: objects.split_whitespace().map(String::from).collect(),
            scene,
            action,
            relations: relations.split_whitespace().map(String::from).collect(),
            text,
            people_count,
            indexed: true,
        },
    ))
}

// ============================================================
// 娴嬭瘯
// ============================================================


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_caption_clean_json() {
        let raw = "{\"caption\":\"a cat sitting\",\"objects\":[\"cat\"],\"people_count\":1}";
        let c = parse_caption(raw).expect("parse");
        assert_eq!(c.caption, "a cat sitting");
        assert_eq!(c.objects, vec!["cat".to_string()]);
        assert_eq!(c.people_count, 1);
    }

    #[test]
    fn parse_caption_tolerates_wrong_types() {
        let c = parse_caption("{\"caption\":\"cat\",\"objects\":null,\"people_count\":null}").unwrap();
        assert!(c.objects.is_empty());
        assert_eq!(c.people_count, 0);
        let c = parse_caption("{\"caption\":\"p\",\"people_count\":\"2\"}").unwrap();
        assert_eq!(c.people_count, 2);
        let c = parse_caption("{\"caption\":null,\"objects\":[\"x\"]}").unwrap();
        assert_eq!(c.caption, "");
    }

    #[test]
    fn parse_caption_extracts_braces() {
        let raw = "here is the result: {\"caption\":\"dog\",\"scene\":\"outdoor\"} hope it helps";
        let c = parse_caption(raw).unwrap();
        assert_eq!(c.caption, "dog");
        assert_eq!(c.scene, "outdoor");
    }

    #[test]
    fn parse_caption_rejects_garbage() {
        assert!(parse_caption("I cannot help").is_none());
        assert!(parse_caption("").is_none());
        assert!(parse_caption("{\"caption\": \"unclosed").is_none());
    }

    #[test]
    fn custom_fields_survive_parse() {
        let raw = "{\"caption\":\"sunset\",\"colors\":[\"orange\",\"purple\"],\"weather\":\"clear\"}";
        let c = parse_caption(raw).unwrap();
        assert_eq!(c.extra.get("colors").map(|s| s.as_str()), Some("orange purple"));
        assert_eq!(c.extra.get("weather").map(|s| s.as_str()), Some("clear"));
        let idx = c.to_index_text();
        assert!(idx.contains("orange"));
        assert!(idx.contains("colors"));
    }

    #[test]
    fn bigramize_cjk_three_char() {
        let s = "\u{65e5}\u{843d}\u{65f6}";
        // 奇数长度追加尾部单字，让末字能被单字查询命中
        assert_eq!(bigramize(s), "\u{65e5}\u{843d} \u{843d}\u{65f6} \u{65f6}");
    }

    #[test]
    fn bigramize_mixed_latin_cjk() {
        let s = "Q3\u{732b}";
        assert_eq!(bigramize(s), "Q3 \u{732b}");
    }

    #[test]
    fn query_to_fts_uses_or() {
        let q = query_to_fts("\u{65e5}\u{843d}\u{65f6}");
        assert_eq!(q, "\"\u{65e5}\u{843d}\" OR \"\u{843d}\u{65f6}\" OR \"\u{65f6}\"");
        assert!(q.contains(" OR "));
    }

    #[test]
    fn parse_relevance_various() {
        assert_eq!(parse_relevance("0.8"), Some(0.8));
        assert_eq!(parse_relevance(".5"), Some(0.5));
        assert_eq!(parse_relevance("relevance: 0.75"), Some(0.75));
        assert_eq!(parse_relevance("-0.3"), None);
        assert_eq!(parse_relevance("1.5"), None);
        assert_eq!(parse_relevance("not relevant"), None);
    }

    #[test]
    fn build_index_prompt_includes_custom_fields() {
        let mut fields = PromptFields::default();
        fields.custom_fields.push(CustomField {
            key: "colors".to_string(),
            desc: "dominant colors".to_string(),
            kind: CustomFieldKind::Array,
        });
        fields.custom_fields.push(CustomField {
            key: "weather".to_string(),
            desc: "weather if outdoors".to_string(),
            kind: CustomFieldKind::String,
        });
        let p = build_index_prompt(&fields);
        // 自定义字段按类型出现在 prompt 里
        assert!(p.contains("\"colors\": [\"dominant colors\"]"), "array field: {}", p);
        assert!(p.contains("\"weather\": \"weather if outdoors\""), "string field: {}", p);
        // 内置字段仍在
        assert!(p.contains("\"caption\":"));
        assert!(p.contains("\"people_count\":"));
        // 硬性约束在
        assert!(p.contains("only output JSON") || p.contains("\u{53ea}\u{8f93}\u{51fa} JSON"));
    }

    #[test]
    fn build_index_prompt_uses_custom_builtin_desc() {
        let mut fields = PromptFields::default();
        fields.builtin_descs.insert("caption".to_string(), "custom caption desc".to_string());
        let p = build_index_prompt(&fields);
        assert!(p.contains("\"caption\": \"custom caption desc\""), "{}", p);
    }

    #[test]
    fn build_index_prompt_escapes_quotes_in_desc() {
        let mut fields = PromptFields::default();
        fields.custom_fields.push(CustomField {
            key: "note".to_string(),
            desc: "say \"hi\"".to_string(),
            kind: CustomFieldKind::String,
        });
        let p = build_index_prompt(&fields);
        // 引号必须被转义，否则破坏 prompt 的 JSON 模板
        assert!(p.contains("\\\"hi\\\""), "{}", p);
    }

    #[test]
    fn build_index_prompt_skips_empty_custom_fields() {
        let mut fields = PromptFields::default();
        fields.custom_fields.push(CustomField {
            key: String::new(),
            desc: "no key".to_string(),
            kind: CustomFieldKind::String,
        });
        fields.custom_fields.push(CustomField {
            key: "ok".to_string(),
            desc: "  ".to_string(),
            kind: CustomFieldKind::String,
        });
        let p = build_index_prompt(&fields);
        assert!(!p.contains("no key"));
        assert!(!p.contains("\"ok\""));
    }
}




