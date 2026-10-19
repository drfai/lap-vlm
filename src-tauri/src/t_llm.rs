//! 鏈湴 VLM 瑙嗚绱㈠紩妯″潡锛坒ork 鏂板锛夈€?//!
//! 閫氳繃 OpenAI 鍏煎鎺ュ彛璋冪敤鏈湴瑙嗚璇█妯″瀷锛圠M Studio / llama.cpp server锛夛紝
//! 涓烘瘡寮犵収鐗囩敓鎴愮粨鏋勫寲涓枃鎻忚堪锛屽啓鍏?SQLite 渚?FTS5 妫€绱€?//!
//! # 璁捐鍘熷垯
//!
//! 1. **涓嶈Е纰?CLIP 绱㈠紩銆?* `afiles.embeds` 浠嶅瓨 CLIP 鍥惧儚鍚戦噺锛?//!    `t_similar.rs` 鐨勭浉浼煎害鍒嗙粍瀹屽叏涓嶅彈褰卞搷銆傛湰妯″潡鍙柊澧炶〃銆?//! 2. **鏈嶅姟涓嶅湪绾挎椂闄嶇骇銆?* 鎵€鏈夎皟鐢ㄥ厛 health check锛涘け璐ュ彧褰卞搷鏈姛鑳斤紝
//!    涓嶅奖鍝嶆祻瑙堛€丆LIP 鎼滅储銆佷汉鑴歌仛绫汇€?//! 3. **鍙彇娑?+ 鍙柇鐐圭画璺戙€?* 鍙傜収 `t_face::run_face_indexing` 鐨勪笁浠跺锛?//!    鍙栨秷浠ょ墝 / 杩涘害鐘舵€?/ Tauri 浜嬩欢銆?//! 4. **涓茶鎵瑰鐞嗐€?* llama.cpp 鍗曞疄渚嬪苟鍙戜細浜掔浉鎺掗槦骞朵簤鏄惧瓨锛?//!    榛樿骞跺彂 1锛屽彲鐢遍厤缃皟鍒?2銆?//!
//! # 宸查獙璇佺殑绾︽潫锛堟潵鑷?POC锛岃 llm-poc/REPORT.md锛?//!
//! - 妯″瀷榛樿寮€鍚€濊€冩ā寮忥紝`reasoning_tokens` 浼氬悆鎺夊ぇ閲忛绠楀鑷磋緭鍑鸿鎴柇銆?//!   蹇呴』鍏虫€濊€冩垨鎶?max_tokens 璋冨埌 1024+銆?//! - 涓枃蹇呴』璧?bigram 鍒囧垎锛孎TS5 榛樿 unicode61 瀵瑰瀛楄瘝鍩烘湰鏃犳晥銆?//! - 鏌ヨ涓蹭笉鑳界敤绌烘牸杩炴帴鍚庣洿鎺?MATCH锛堥偅鏄煭璇涔夛級锛屽繀椤绘樉寮?OR銆?
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
    /// OpenAI 鍏煎绔偣鏍瑰湴鍧€锛屼笉鍚?/v1銆備緥锛歨ttp://127.0.0.1:1234
    pub base_url: String,
pub model: String,
pub api_key: String,
    /// 鏄惁鍏抽棴妯″瀷鐨勬€濊€冩ā寮忋€傚己鐑堝缓璁?true鈥斺€斿惁鍒?reasoning_tokens
pub disable_thinking: bool,
pub max_tokens: u32,
pub temperature: f32,
pub preview_max_px: u32,
pub concurrency: u32,
pub timeout_secs: u64,
pub retry_on_parse_error: u32,
pub album_ids: Vec<i64>,
    /// 鏄惁鍚敤绮剧‘閲嶆帓锛氬 top-N 鍊欓€変簩娆¤皟鐢?VLM 鍒ゆ柇鐩稿叧鎬с€?    ///
    /// 榛樿 false銆傚紑鍚悗姣忔鏂囨湰鎼滅储閮戒細澶氳姳 N 娆℃帹鐞嗚皟鐢?    /// 锛圢 = `rerank_top_n`锛夛紝鎱絾鍑嗐€傞€傚悎搴撳緢澶с€丆LIP+caption
pub enable_rerank: bool,
pub rerank_top_n: u32,
    /// FORK: 绱㈠紩 prompt 鐨勫瓧娈靛畾涔夛紙UI 鍙紪杈戯級銆?    ///
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
// 鐘舵€侊紙鐓ф妱 t_face 鐨勪笁浠跺妯″紡锛?// ============================================================

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

/// 妯″瀷杩斿洖鐨勭粨鏋勫寲鎻忚堪銆傚瓧娈靛悕涓?prompt 涓殑 JSON key 涓€涓€瀵瑰簲銆?///
/// # 涓轰粈涔堝瓧娈佃閫愪釜鍔?`deserialize_with`
///
/// 妯″瀷涓嶄繚璇佺被鍨嬫纭€侾OC + 瀹炴祴韪╄繃鐨勫潙锛?/// - `objects` / `relations` 鏈夋椂杩斿洖 `null` 鎴栧崟涓瓧绗︿覆鑰岄潪鏁扮粍
/// - `people_count` 鏈夋椂杩斿洖瀛楃涓?`"1"` 鎴?`null`
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
    /// FORK: 鑷畾涔夊瓧娈靛厹搴曘€?    ///
    /// 鐢ㄦ埛鏀逛簡 prompt銆佽妯″瀷杈撳嚭涓婇潰鐨勫瓧娈典箣澶栫殑鍐呭锛堟瘮濡?    /// `"colors": ["绾?,"钃?]`銆乣"weather": "鏅村ぉ"`锛夛紝娌℃湁杩欎釜瀛楁鐨勮瘽
    /// serde 浼氭妸瀹冧滑鍏ㄩ儴涓㈠純锛屾悳涓嶅埌銆?    ///
#[serde(flatten, deserialize_with = "de_extra")]
    pub extra: std::collections::BTreeMap<String, String>,
}

/// 瀹藉瑙ｆ瀽鑷畾涔夊瓧娈碉細鎶婁换鎰?JSON 鍊艰浆鎴愬彲绱㈠紩鐨勬枃鏈€?/// - 瀛楃涓?-> 鍘熸牱
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
// 涓枃 bigram 鍒囧垎
// ============================================================

fn is_cjk(ch: char) -> bool {
    matches!(ch as u32,
        0x4E00..=0x9FFF      // CJK Unified Ideographs
        | 0x3400..=0x4DBF    // Extension A
        | 0xF900..=0xFAFF    // Compatibility Ideographs
    )
}

/// 鎶婃枃鏈垏鎴愰€傚悎 FTS5 unicode61 绱㈠紩鐨?token 涓层€?///
/// 瑙勫垯锛?/// - CJK 姹夊瓧鎸?2 瀛楁粦鍔ㄧ獥鍙ｅ垏鍒嗭紙"鏃ヨ惤鏃跺垎" -> "鏃ヨ惤 钀芥椂 鏃跺垎"锛?/// - **濂囨暟闀垮害鏃惰ˉ杈撳嚭灏鹃儴鍗曞瓧**锛?娉曞浗鏂楃墰鐘? -> "娉曞浗 鍥芥枟 鏂楃墰 鐗涚姮 鐘?锛夛紝
///   鍚﹀垯鏈€鍚庝竴涓瓧姘歌繙鏃犳硶琚崟瀛楁煡璇㈠懡涓紝閫犳垚鍙洖鎹熷け銆?/// - 杩炵画鐨勫瓧姣?鏁板瓧淇濇寔涓哄崟涓?token锛屼繚鐣欏師濮嬪ぇ灏忓啓
///   锛團TS5 unicode61 鏈韩澶у皬鍐欎笉鏁忔劅锛屾棤闇€鍦ㄦ褰掍竴鍖栵級
/// - 鍏朵綑瀛楃浣滀负鍒嗛殧绗︿涪寮?///
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
/// **蹇呴』鐢?OR 杩炴帴锛屼笉鑳界敤绌烘牸銆?* FTS5 閲岀┖鏍兼槸 AND / 闅愬惈鐭璇箟锛?/// 鑰岀储寮曚晶姣忎釜 bigram 鏄嫭绔?token銆佹棤杩炵画浣嶇疆鍏崇郴锛岀敤绌烘牸杩炴帴浼?/// 瀵艰嚧鍏ㄩ儴鏌ヤ笉鍒帮紙POC 绗竴鐗堣俯杩囪繖涓潙锛岀粨鏋滃叏 0锛夈€?///
/// # 鍗曞瓧鏌ヨ鐨勫浐鏈夊眬闄?///
/// bigram 绱㈠紩閲屼笉瀛樺湪鍗曞瓧 token锛?涓€鍙尗" 鍒囨垚 "涓€鍙?鍙尗/鐚湪/..."锛夛紝
/// 鎵€浠?`MATCH '"鐚?'` 姘歌繙鏌ヤ笉鍒颁笢瑗裤€傞暱搴?鈮?2 鐨勬煡璇㈤渶璋冪敤鏂规敼璧?/// `search_caption_substring` 鐢?LIKE 鍏滃簳锛屽惁鍒欑敤鎴锋悳"鐚?"鐙?
pub fn query_to_fts(q: &str) -> String {
    let toks: Vec<String> = bigramize(q)
        .split_whitespace()
        .map(|t| format!("\"{}\"", t.replace('"', "")))
        .collect();
    if toks.is_empty() {
        // 绌烘煡璇細缁欎竴涓繀鐒朵笉鍖归厤鐨勮〃杈惧紡锛岄伩鍏?FTS5 璇硶閿欒
        return "\"\"".to_string();
    }
    toks.join(" OR ")
}

// ============================================================
// Prompt
// ============================================================

/// 绱㈠紩鐢?prompt 鐢?`build_index_prompt(&cfg.prompt_fields)` 鍔ㄦ€佺敓鎴?/// 锛堣涓婃柟 `BUILTIN_FIELDS` 涓?`PromptFields`锛夈€?///
/// 纭€ц姹傦紙澶撮儴"鍙緭鍑?JSON" + 灏鹃儴涓夋潯绾︽潫锛夊湪 `build_index_prompt` 閲?/// 鍐欐锛岀敤鎴锋敼涓嶄簡鈥斺€擯OC 瀹炴祴 23 寮?0 渚嬩汉鍚嶅湴鍚嶅够瑙夛紝caption 鍙俊搴?/// 鐩存帴鍐冲畾妫€绱㈣川閲忋€傛斁寮€鎺ㄦ祴浼氳绱㈠紩閲屽厖婊￠敊璇俊鎭€?
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
/// 澶辫触鍒嗙被锛堝墠绔鑳藉尯鍒嗭級锛?/// - 鏈嶅姟涓嶅湪绾?-> health_check 鍏堟嫤浣?/// - HTTP 閿欒 -> 甯︾姸鎬佺爜
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

/// 鏋佺畝 HTTP POST 瀹㈡埛绔紙std::net锛屾棤寮傛杩愯鏃讹級銆?///
/// 璇存槑锛歚t_http.rs` 宸插湪鐢?reqwest锛屼絾閭ｆ槸澶氳瑷€妯″瀷涓嬭浇璺緞銆?/// 鏈ā鍧楀彧鐢ㄦ渶绠€鍗曠殑 POST + 璇?body锛岀敤 std 鎵嬪啓鍙伩鍏?reqwest 鐨?/// feature/閾炬帴宸紓銆傝嫢浣犵殑 fork 宸茬‘璁?reqwest 绋冲畾锛屽彲鎹㈠洖 reqwest銆?///
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
/// 2. **鎻愬彇绗竴涓?`{` 鍒版渶鍚庝竴涓?`}``** 鈥斺€?妯″瀷鏈夋椂鍦?JSON 鍓嶅悗甯﹁鏄庢枃瀛?///    锛?濂界殑锛岃繖鏄垎鏋愮粨鏋滐細" + JSON + "甯屾湜鏈夊府鍔?锛夛紝涓嶆彁鍙栦細鐩存帴瑙ｆ瀽澶辫触
/// 3. serde 鐨?`default` 鍏滃簳缂哄け瀛楁
pub fn parse_caption(raw: &str) -> Option<LlmCaption> {
    let mut t = raw.trim();

    // 1. 鍘绘帀 markdown 浠ｇ爜鍥存爮
    if t.starts_with("```") {
        let after = t.splitn(2, '\n').nth(1).unwrap_or("");
        t = after.trim_end().trim_end_matches("```").trim();
    }

    // 2. 鎴彇绗竴涓?{ 鍒版渶鍚庝竴涓?}锛屽蹇嶅墠鍚庣紑鏂囧瓧
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

        -- FORK: 澶辫触璁板綍琛ㄣ€傛病鏈夊畠锛岀储寮曞け璐ョ殑鏂囦欢涓嶅啓浠讳綍鐥曡抗锛?        -- pending_files 涓嬩竴杞張鎶婂悓涓€鎵规崬鍥炴潵 -> 鏃犻檺閲嶈瘯绌鸿浆銆?        -- 锛堝疄娴嬶細110 寮犵収鐗囪窇 1 灏忔椂锛宑urrent 绱埌 1340锛宖ailed 1331銆傦級
        -- fail_streak 璁板綍杩炵画澶辫触娆℃暟锛岃秴杩囬槇鍊间笉鍐嶈嚜鍔ㄩ噸璇曘€?        CREATE TABLE IF NOT EXISTS afile_llm_fail (
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
            "ensure llm fts (FTS5 鏈惎鐢? {}). 鑻?bundled SQLite 鏈紑 FTS5, \
             闇€鍦?build.rs 瀹氫箟 SQLITE_ENABLE_FTS5",
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

/// 娣峰悎妫€绱細鎶?VLM caption 绱㈠紩鐨勫懡涓粨鏋滃悎骞惰繘 CLIP 鐨勫垎鏁板垪琛ㄣ€?///
/// 杩欐槸甯﹂厤缃鏌ョ殑澶栧眰鍖呰銆傛牳蹇冨悎骞堕€昏緫鍦?`merge_with_caption_scores`锛?/// 鎶藉嚭鏉ユ槸涓轰簡鍙祴璇曪紙涓嶄緷璧栧叏灞€閰嶇疆鏂囦欢锛夈€?///
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

/// 鏍稿績鍚堝苟閫昏緫锛堢函鍑芥暟锛屼笉璇婚厤缃級銆?///
/// # 涓轰粈涔堢敤浣嶇疆鍔犳潈鑰屼笉鏄垎鏁扮浉鍔?///
/// CLIP 鐨勪綑寮︾浉浼煎害涓?FTS5 鐨?`rank` 閲忕翰瀹屽叏涓嶅悓锛堝墠鑰呮槸 [0,1] 鐨勪綑寮︼紝
/// 鍚庤€呮槸 FTS5 鍐呴儴鐨?BM25 鍙樹綋锛屽彲浠ユ槸浠绘剰璐熸暟锛夈€傜洿鎺ョ浉鍔犱細璁╂煇涓€鏂?/// 瀹屽叏涓诲锛屽疄闄呮晥鏋滀笉鍙帶銆?///
/// 杩欓噷鎶婁袱杈?*鍚勮嚜褰掍竴鍖栧埌 [0,1]** 鍐嶅姞鏉冿細
/// - CLIP 渚э細score 鏈韩灏辨槸 [0,1] 鐨勪綑寮︼紝鐩存帴鐢?/// - caption 渚э細鎸?FTS5 杩斿洖鐨勪綅缃嚎鎬ц“鍑忥紝绗?i 鍚嶅緱 (n-i)/n
///
/// # 涓轰粈涔?caption 鍙姞鍒嗕笉绛涢€?///
/// caption 鏄?琛ュ厖绾跨储"锛屼笉鏄?杩囨护鍣?銆侰LIP 鍛戒腑鐨勭粨鏋滃叏閮ㄤ繚鐣欙紝
/// caption 鍛戒腑鐨勬枃浠惰寰€涓婃彁 鈥斺€?杩欐牱鍗充娇 caption 婕忔爣鎴栬鏍囷紝
pub fn merge_with_caption_scores(
    conn: &rusqlite::Connection,
    query: &str,
    scores: Vec<(i64, f32)>,
) -> Vec<(i64, f32)> {
    // caption 鍛戒腑鍒楄〃锛堟寜 FTS5 rank 鎺掑簭锛夈€備笂闄愬彇 CLIP 缁撴灉鏁?+ 涓€瀹氫綑閲忥紝
    let cap_limit = (scores.len() as i64) + 64;
    let caption_ids = match search_caption(conn, query, cap_limit) {
        Ok(ids) => ids,
        Err(_) => return scores, // FTS 鏌ヨ澶辫触 -> 闈欓粯闄嶇骇
    };
    if caption_ids.is_empty() {
        return scores;
    }

const CLIP_WEIGHT: f32 = 0.7;
    const CAPTION_WEIGHT: f32 = 0.3;

    // caption 渚т綅缃垎锛氱 i 鍚嶏紙浠?0 寮€濮嬶級寰?(n-i)/n锛屽綊涓€鍖栧埌 [0,1]
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
            // 鍙湪 caption 涓懡涓殑鏂囦欢锛氬彧缁?caption 鍒嗭紝
            let cap = caption_score.get(id).copied().unwrap_or(0.0);
            merged.insert(*id, cap * CAPTION_WEIGHT);
        }
    }

    let mut result: Vec<(i64, f32)> = merged.into_iter().collect();
    result.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    result
}

/// 鍒ゆ柇鏌ヨ鏄惁闇€瑕佽蛋 LIKE 鍏滃簳銆?///
/// bigram 绱㈠紩閲屼笉瀛樺湪鍗曞瓧 token锛?涓€鍙尗" 鍒囨垚 "涓€鍙?鍙尗/鐚湪/..."锛夛紝
pub fn needs_substring_fallback(q: &str) -> bool {
    let cjk_count = q.chars().filter(|c| is_cjk(*c)).count();
    cjk_count <= 2
}

/// 瀛愪覆鍏滃簳妫€绱細鐩存帴鍦?`index_text` 涓婂仛 LIKE銆?///
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
    // 杞箟 LIKE 閫氶厤绗︼紝闃叉鐢ㄦ埛杈撳叆 % 鎴?_ 鏀瑰彉璇箟
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

/// 绮剧‘閲嶆帓锛氬 top-N 鍊欓€夌敤 VLM 閫愪釜鍒ゆ柇鏄惁鍖归厤鏌ヨ銆?///
/// 杩欐槸甯﹂厤缃鏌ョ殑澶栧眰鍖呰锛堜笌 `merge_caption_scores` 鍚屼竴妯″紡锛夛紝
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

/// 鏍稿績閲嶆帓閫昏緫锛堢函鍑芥暟锛屼笉璇婚厤缃紝渚夸簬娴嬭瘯锛夈€?///
/// # 涓轰粈涔堝€煎緱鍋?///
/// CLIP + caption 鍚堝苟鏄?绮楁帓"锛欳LIP 鎶撹瑙夌浉浼硷紝caption 鎶撴枃瀛楃嚎绱紝
/// 浣嗕袱鑰呴兘涓嶄細鐪熺殑"璇绘噦"鏌ヨ鎰忓浘銆傛瘮濡傛煡"绾㈣壊鑳屽寘鏀惧湪妞呭瓙鏃佽竟"锛?/// 绮楁帓鍙兘闈?caption 閲屾槸鍚﹀嚭鐜颁簡杩欎簺璇嶏紝鏃犳硶鐞嗚В绌洪棿鍏崇郴銆?///
/// VLM 閲嶆帓璁╂ā鍨嬬洿鎺ョ湅鍊欓€夌殑 caption + 鏌ヨ锛岃緭鍑?0-1 鐨勭浉鍏冲害鍒嗘暟銆?///
/// # 涓轰粈涔堥粯璁ゅ叧闂?///
/// 姣忔鏂囨湰鎼滅储澶氳姳 N 娆℃帹鐞嗚皟鐢紙N = `rerank_top_n`锛夈€?/// 27B 妯″瀷鍗曟绾?1-3 绉掞紝30 涓€欓€夊氨鏄?30-90 绉?鈥斺€?鎼滅储浠?鍗虫椂"
/// 鍙樻垚"瑕佺瓑"銆傛墍浠ヨ繖鏄彲閫夌殑绮剧‘妯″紡锛屼笉鏄粯璁よ矾寰勩€?///
/// # 娣峰悎鏉冮噸
///
/// `final = 鍘熷鍒?脳 0.3 + VLM 鍒ゆ柇 脳 0.7`銆備笉瀹屽叏涓㈠純鍘熷鍒嗭紝
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

const RERANK_PROMPT_PREFIX: &str = "浣犳槸鐓х墖鎼滅储鐩稿叧鎬у垽鏂櫒銆傜粰瀹氱敤鎴风殑鎼滅储鏌ヨ鍜屼竴寮犵収鐗囩殑鍐呭鎻忚堪锛屽垽鏂繖寮犵収鐗囨槸鍚︽槸鐢ㄦ埛鎯虫壘鐨勩€俓n\n瑙勫垯锛歕n- 鍙緭鍑轰竴涓?0 鍒?1 涔嬮棿鐨勫皬鏁帮紝涓嶈杈撳嚭浠讳綍鍏朵粬鏂囧瓧\n- 1.0 = 瀹屽叏绗﹀悎鏌ヨ鎰忓浘\n- 0.5 = 閮ㄥ垎鐩稿叧锛屽彲鑳芥槸鐢ㄦ埛鎯崇湅鐨刓n- 0.0 = 瀹屽叏涓嶇浉鍏砛n- 鎷夸笉鍑嗘椂缁?0.3 宸﹀彸鐨勪腑闂村€硷紝涓嶈鏋佺\n\n";

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
/// 妯″瀷鍙兘杈撳嚭 "0.8"銆?.8"銆?0.8."銆?鐩稿叧搴︼細0.8"銆?鍒嗘暟鏄?0.6 鍒? 绛夛紝
/// 閮借瀹归敊銆傚仛娉曟槸鎵弿鏁翠釜瀛楃涓诧紝鎵惧嚭鎵€鏈夋暟瀛楀瓙涓诧紝鍙栫涓€涓惤鍦?/// [0,1] 鑼冨洿鍐呯殑銆?///
/// 涓や釜瀹规槗韪╃殑鍧戯紙閮芥湁鍥炲綊娴嬭瘯锛夛細
/// 1. **`.5` 杩欑鏃犲墠瀵奸浂褰㈠紡** 鈥斺€?璧风偣鏄?`.` 鑰岄潪鏁板瓧锛岀洿鎺ユ寜"鏁板瓧璧风偣"
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
            // 甯︾鍙蜂竴璧?parse: -0.3 寰楀埌 -0.3, 钀藉湪 [0,1] 澶? 鑷劧鎷掔粷
            if (0.0..=1.0).contains(&v) {
                return Some(v);
            }
        }
        // 澶辫触鏃惰烦鍒版湰 token 鏈熬, 閬垮厤鎶?-0.3 鎷嗘垚 -0.3 + 0.3 涓ゆ鍖归厤
        i = j.max(start + 1);
    }
    None
}

/// 妫€绱細杩斿洖鎸?rank / 鐩稿叧搴︽帓搴忕殑 file_id銆?///
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
            // FORK: JOIN afiles 杩囨护瀛ゅ効琛岋紙file_id 鍦?afiles 閲屽凡涓嶅瓨鍦級銆?            // 涓?JOIN 鐨勮瘽锛屽簱閲嶅缓/鍒犳枃浠跺悗 afile_llm 鐨勬畫鐣欎細璁?indexed
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
// 鍥剧墖棰勫鐞?// ============================================================

/// 瑙嗛鎶藉抚鐨勬椂闂寸偣锛堢櫨鍒嗘瘮浣嶇疆锛夈€?///
const VIDEO_FRAME_PERCENTS: [u8; 3] = [10, 50, 90];

/// 涓鸿棰戞娊鍙栧甯у苟妯悜鎷兼垚涓€寮犲浘锛堜緵 VLM 涓€娆＄湅瀹岋級銆?///
/// 澶嶇敤 `t_video::get_video_thumbnail` 鈥斺€?瀹冨唴閮ㄥ凡澶勭悊 ffmpeg 璺緞銆?/// seek 绛栫暐銆佸け璐ラ檷绾с€傝繖閲屽彧璐熻矗鎸夊涓椂闂寸偣鎶藉彇骞舵嫾鎺ャ€?///
/// 鎷兼帴鑰岄潪鍙戝寮犲浘锛氬鏁?VLM 鐨?API 瀵瑰崟鏉℃秷鎭殑鍥剧墖鏁伴噺鏈夐檺鍒讹紝
async fn prepare_video_frames_b64(file_path: &str, cfg: &LlmConfig) -> Result<Option<String>, String> {
    use crate::t_video;

    let duration = t_video::get_video_duration(file_path).await.ok();
    // 澶煭鐨勮棰戞娊澶氬抚娌℃剰涔夛紙鍙兘姣忓抚閮戒竴鏍凤級锛岀洿鎺ョ敤灏侀潰
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

    // 缁熶竴楂樺害鍚庢í鍚戞嫾鎺ワ紝涓棿鐣?4px 榛戠紳鍒嗛殧
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
/// 妯″紡鏉ヨ嚜 `t_face::run_face_indexing`銆傝棰戝皝闈㈠抚鍚屾牱瀛樺湪
/// `athumbs` 琛ㄩ噷锛屾墍浠ヤ篃浼氳杩欐潯璺緞瑕嗙洊銆?///
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

/// 杩愯 VLM 绱㈠紩銆?///
/// 妯″紡瀹屽叏瀵圭収 `t_face::run_face_indexing`锛?/// - 鍙栨秷浠ょ墝妫€鏌?/// - 杩涘害閫氳繃 Tauri 浜嬩欢鎺ㄩ€侊紙姣?10 涓垨缁撴潫鏃讹級
/// - 鍗曚釜鏂囦欢澶辫触涓嶄腑鏂暣浣?/// - 缁撴潫鏃?emit `llm_index_finished`
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
        // reset 闂寘蹇呴』瀹氫箟鍦?spawn 鍐呴儴: 瀹氫箟鍦ㄥ灞備細鍥犲€熺敤 progress/status
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

        // 鏈嶅姟涓嶅湪绾?-> 绔嬪埢澶辫触锛屼笉杩涗富寰幆
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
            // 鍥犳鍙互瀹夊叏鍊熺敤 batch / cfg / cancel銆?            //
            // 涓轰粈涔堟瘡绾跨▼鐙珛寮€ DB 杩炴帴: Lap 鐨?SQLite 璺戝湪 WAL 妯″紡
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
                        // 绾跨▼鍐呭鐢ㄥ悓涓€涓繛鎺? 閬垮厤姣忔枃浠堕噸寮€
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
                                    // FORK: 鍐欏け璐ヨ褰曪紝鏂帀鏃犻檺閲嶈瘯
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

            // 姹囨€伙細淇濇寔鏂囦欢椤哄簭锛岃繘搴﹁鏁板湪涓荤嚎绋嬫洿鏂帮紙閬垮厤閿佺珵浜夛級
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
                // FORK: 姣?10 涓彂涓€娆¤繘搴︼紱涓€鏃﹀嚭鐜板け璐ョ珛鍒昏ˉ鍙戯紝
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
