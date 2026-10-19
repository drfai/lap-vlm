//! Extracts human-readable generation metadata from AI-created PNG files.
//!
//! The reader walks PNG chunks without decoding pixels. NovelAI's fallback
//! decodes only one PNG row at a time, keeping scans bounded in memory.

use flate2::read::{GzDecoder, ZlibDecoder};
use png::{ColorType, Decoder};
use serde_json::Value;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read, Seek, SeekFrom};
use std::path::Path;

const PNG_SIGNATURE: [u8; 8] = [137, 80, 78, 71, 13, 10, 26, 10];
const MAX_TEXT_CHUNK_BYTES: usize = 512 * 1024;
const MAX_COMMENT_BYTES: usize = 64 * 1024;
const MAX_CHUNKS: usize = 4096;
const MAX_STEALTH_BITS: usize = 15 * 8 + 32 + MAX_TEXT_CHUNK_BYTES * 8;

/// Returns AI generation metadata suitable for the user-facing file comment.
/// Unsupported, malformed, or non-AI PNGs simply return `None`.
pub fn extract_comment(file_path: &str) -> Option<String> {
    if !is_png(file_path) {
        return None;
    }

    let file = File::open(file_path).ok()?;
    let mut reader = BufReader::new(file);
    let mut signature = [0u8; 8];
    reader.read_exact(&mut signature).ok()?;
    if signature != PNG_SIGNATURE {
        return None;
    }

    let mut candidates: Vec<(u8, String)> = Vec::new();
    for _ in 0..MAX_CHUNKS {
        let mut header = [0u8; 8];
        if reader.read_exact(&mut header).is_err() {
            break;
        }
        let length = u32::from_be_bytes(header[..4].try_into().ok()?) as usize;
        let chunk_type = &header[4..8];

        if chunk_type == b"IEND" {
            break;
        }

        let is_text = matches!(chunk_type, b"tEXt" | b"zTXt" | b"iTXt");
        if !is_text || length > MAX_TEXT_CHUNK_BYTES {
            reader.seek(SeekFrom::Current((length as i64) + 4)).ok()?;
            continue;
        }

        let mut data = vec![0u8; length];
        reader.read_exact(&mut data).ok()?;
        let mut crc = [0u8; 4];
        reader.read_exact(&mut crc).ok()?;

        let text = match chunk_type {
            b"tEXt" => parse_text_chunk(&data),
            b"zTXt" => parse_compressed_text_chunk(&data),
            b"iTXt" => parse_international_text_chunk(&data),
            _ => None,
        };
        if let Some((keyword, value)) = text {
            if let Some((priority, comment)) = format_ai_metadata(&keyword, &value) {
                candidates.push((priority, comment));
            }
        }
    }

    candidates.sort_by(|left, right| right.0.cmp(&left.0));
    candidates
        .into_iter()
        .next()
        .map(|(_, comment)| comment)
        .or_else(|| extract_stealth_comment(file_path))
}

/// NovelAI stores a compressed metadata payload in the alpha channel's LSBs.
/// This is intentionally a fallback: normal PNG text metadata is cheaper to read.
fn extract_stealth_comment(file_path: &str) -> Option<String> {
    let file = File::open(file_path).ok()?;
    let decoder = Decoder::new(BufReader::new(file));
    let mut reader = decoder.read_info().ok()?;
    let info = reader.info();
    // NovelAI stealth metadata is embedded in 8-bit RGBA PNGs. Rejecting
    // other color types before decoding avoids a full pixel decode for the
    // majority of ordinary RGB/grayscale PNGs.
    if info.color_type != ColorType::Rgba
        || info.bit_depth != png::BitDepth::Eight
        || info.interlaced
    {
        return None;
    }
    let width = info.width;
    let height = info.height;
    let row_size = reader.output_line_size(width)?;
    let mut row = vec![0u8; row_size];
    let pixel_count = (width as usize).checked_mul(height as usize)?;
    let bit_limit = pixel_count.min(MAX_STEALTH_BITS);
    let mut alpha_bits = vec![0u8; bit_limit.div_ceil(8)];
    let mut y = 0usize;
    while let Some(_) = reader.read_row(&mut row).ok()? {
        for x in 0..width as usize {
            let alpha = row[x * 4 + 3] & 1;
            let bit_index = x * height as usize + y;
            if bit_index < bit_limit {
                alpha_bits[bit_index / 8] |= alpha << (7 - bit_index % 8);
            }
        }
        y += 1;
    }
    let mut reader = StealthBitReader::new(&alpha_bits, width, height);
    let magic = reader.read_bytes(b"stealth_pngcomp".len())?;
    if magic != b"stealth_pngcomp" {
        return None;
    }

    let compressed_len_bits = reader.read_u32()?;
    if compressed_len_bits % 8 != 0 {
        return None;
    }
    let compressed_len = (compressed_len_bits / 8) as usize;
    if compressed_len == 0 || compressed_len > MAX_TEXT_CHUNK_BYTES {
        return None;
    }
    let compressed = reader.read_bytes(compressed_len)?;

    let decoder = GzDecoder::new(compressed.as_slice());
    let mut decoded = Vec::with_capacity(compressed_len.min(MAX_COMMENT_BYTES));
    decoder
        .take((MAX_COMMENT_BYTES + 1) as u64)
        .read_to_end(&mut decoded)
        .ok()?;
    if decoded.len() > MAX_COMMENT_BYTES {
        return None;
    }

    let metadata = serde_json::from_slice::<Value>(&decoded).ok()?;
    let comment = metadata.get("Comment")?;
    if let Some(comment_json) = comment.as_str() {
        if let Ok(nested) = serde_json::from_str::<Value>(comment_json) {
            return format_json_value(&nested);
        }
        return normalize(comment_json);
    }
    format_json_value(comment)
}

struct StealthBitReader<'a> {
    alpha_bits: &'a [u8],
    width: u32,
    height: u32,
    x: u32,
    y: u32,
}

impl<'a> StealthBitReader<'a> {
    fn new(alpha_bits: &'a [u8], width: u32, height: u32) -> Self {
        Self {
            alpha_bits,
            width,
            height,
            x: 0,
            y: 0,
        }
    }

    fn read_bit(&mut self) -> Option<u8> {
        if self.x >= self.width || self.y >= self.height {
            return None;
        }
        // NovelAI flattens alpha.T, so pixels are traversed column-major.
        let bit_index = (self.x * self.height + self.y) as usize;
        let bit = (self.alpha_bits.get(bit_index / 8)? >> (7 - bit_index % 8)) & 1;
        self.y += 1;
        if self.y >= self.height {
            self.y = 0;
            self.x += 1;
        }
        Some(bit)
    }

    fn read_bytes(&mut self, length: usize) -> Option<Vec<u8>> {
        (0..length)
            .map(|_| (0..8).try_fold(0u8, |value, _| Some((value << 1) | self.read_bit()?)))
            .collect()
    }

    fn read_u32(&mut self) -> Option<u32> {
        Some(u32::from_be_bytes(self.read_bytes(4)?.try_into().ok()?))
    }
}

fn is_png(file_path: &str) -> bool {
    Path::new(file_path)
        .extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("png"))
}

fn parse_text_chunk(data: &[u8]) -> Option<(String, String)> {
    let separator = data.iter().position(|byte| *byte == 0)?;
    Some((
        decode_latin1(&data[..separator]),
        decode_latin1(&data[separator + 1..]),
    ))
}

fn parse_compressed_text_chunk(data: &[u8]) -> Option<(String, String)> {
    let separator = data.iter().position(|byte| *byte == 0)?;
    if data.get(separator + 1).copied()? != 0 {
        return None;
    }
    Some((
        decode_latin1(&data[..separator]),
        decompress_text(&data[separator + 2..], true)?,
    ))
}

fn parse_international_text_chunk(data: &[u8]) -> Option<(String, String)> {
    let keyword_end = data.iter().position(|byte| *byte == 0)?;
    let compression_flag = data.get(keyword_end + 1).copied()?;
    let compression_method = data.get(keyword_end + 2).copied()?;
    if compression_flag > 1 || (compression_flag == 1 && compression_method != 0) {
        return None;
    }

    let language_start = keyword_end + 3;
    let language_end = data[language_start..].iter().position(|byte| *byte == 0)? + language_start;
    let translated_start = language_end + 1;
    let translated_end = data[translated_start..]
        .iter()
        .position(|byte| *byte == 0)?
        + translated_start;
    let text = &data[translated_end + 1..];

    let value = if compression_flag == 1 {
        decompress_text(text, false)?
    } else {
        String::from_utf8(text.to_vec()).ok()?
    };
    Some((decode_latin1(&data[..keyword_end]), value))
}

fn decompress_text(data: &[u8], latin1: bool) -> Option<String> {
    let decoder = ZlibDecoder::new(data);
    let mut output = Vec::with_capacity(data.len().min(MAX_COMMENT_BYTES));
    decoder
        .take((MAX_COMMENT_BYTES + 1) as u64)
        .read_to_end(&mut output)
        .ok()?;
    if output.len() > MAX_COMMENT_BYTES {
        return None;
    }
    if latin1 {
        Some(decode_latin1(&output))
    } else {
        String::from_utf8(output).ok()
    }
}

fn decode_latin1(bytes: &[u8]) -> String {
    String::from_utf8(bytes.to_vec())
        .unwrap_or_else(|_| bytes.iter().map(|byte| char::from(*byte)).collect())
}

fn format_ai_metadata(keyword: &str, value: &str) -> Option<(u8, String)> {
    let keyword = keyword.trim().to_ascii_lowercase();
    match keyword.as_str() {
        // Automatic1111, Forge, and Fooocus use this plain-text representation.
        "parameters" => normalize(value).map(|comment| (100, comment)),
        // InvokeAI and other generators commonly use JSON under one of these keys.
        "sd-metadata" | "invokeai_metadata" | "metadata" | "comment" => {
            format_json_metadata(value).map(|comment| (90, comment))
        }
        // ComfyUI stores a JSON prompt graph under `prompt`; plain-text prompts are
        // also accepted for generators that use the same keyword.
        "prompt" => format_prompt_metadata(value).map(|comment| (80, comment)),
        _ => None,
    }
}

fn format_prompt_metadata(value: &str) -> Option<String> {
    let parsed = serde_json::from_str::<Value>(value).ok();
    if let Some(parsed) = parsed {
        let (positive, negative) = comfy_prompt_sections(&parsed);
        if !positive.is_empty() || !negative.is_empty() {
            let mut sections = Vec::new();
            if !positive.is_empty() {
                sections.push(format!("Prompt:\n{}", positive.join("\n\n")));
            }
            if !negative.is_empty() {
                sections.push(format!("Negative prompt:\n{}", negative.join("\n\n")));
            }
            return normalize(&sections.join("\n"));
        }
        return format_json_value(&parsed);
    }
    normalize(value).map(|prompt| format!("Prompt:\n{prompt}"))
}

fn format_json_metadata(value: &str) -> Option<String> {
    let parsed = serde_json::from_str::<Value>(value).ok()?;
    format_json_value(&parsed)
}

fn format_json_value(value: &Value) -> Option<String> {
    let object = value.as_object()?;
    let mut lines = Vec::new();
    append_json_string(object, "prompt", "Prompt", &mut lines);
    append_json_string(object, "positive_prompt", "Prompt", &mut lines);
    append_json_string(object, "negative_prompt", "Negative prompt", &mut lines);
    append_json_string(object, "uc", "Negative prompt", &mut lines);
    append_json_value(object, "model", "Model", &mut lines);
    append_json_value(object, "seed", "Seed", &mut lines);
    append_json_value(object, "steps", "Steps", &mut lines);
    append_json_value(object, "sampler", "Sampler", &mut lines);
    append_json_value(object, "sampler_name", "Sampler", &mut lines);
    append_json_value(object, "cfg_scale", "CFG scale", &mut lines);
    append_json_value(object, "cfg", "CFG scale", &mut lines);
    append_json_value(object, "scale", "CFG scale", &mut lines);
    normalize(&lines.join("\n"))
}

fn append_json_string(
    object: &serde_json::Map<String, Value>,
    key: &str,
    label: &str,
    lines: &mut Vec<String>,
) {
    if let Some(value) = object
        .get(key)
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
    {
        lines.push(format!("{label}:\n{}", value.trim()));
    }
}

fn append_json_value(
    object: &serde_json::Map<String, Value>,
    key: &str,
    label: &str,
    lines: &mut Vec<String>,
) {
    let Some(value) = object.get(key) else { return };
    let value = match value {
        Value::String(value) if !value.trim().is_empty() => value.trim().to_string(),
        Value::Number(value) => value.to_string(),
        Value::Bool(value) => value.to_string(),
        _ => return,
    };
    if !lines
        .iter()
        .any(|line| line.starts_with(&format!("{label}:")))
    {
        lines.push(format!("{label}: {value}"));
    }
}

fn comfy_prompt_sections(value: &Value) -> (Vec<String>, Vec<String>) {
    let Some(nodes) = value.as_object() else {
        return (Vec::new(), Vec::new());
    };
    let text_nodes: HashMap<String, String> = nodes
        .iter()
        .filter_map(|(node_id, node)| {
            let node = node.as_object()?;
            let class_type = node
                .get("class_type")
                .and_then(Value::as_str)
                .unwrap_or_default();
            if !class_type.contains("CLIPTextEncode") {
                return None;
            }
            let text = node
                .get("inputs")
                .and_then(Value::as_object)
                .and_then(|inputs| inputs.get("text"))
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|text| !text.is_empty())?;
            Some((node_id.clone(), text.to_string()))
        })
        .collect();

    let mut positive = Vec::new();
    let mut negative = Vec::new();
    let mut has_classified_prompt = false;
    for node in nodes.values() {
        let Some(node) = node.as_object() else {
            continue;
        };
        let Some(inputs) = node.get("inputs").and_then(Value::as_object) else {
            continue;
        };
        for (field, target) in [("positive", &mut positive), ("negative", &mut negative)] {
            let Some(link) = inputs
                .get(field)
                .and_then(Value::as_array)
                .and_then(|link| link.first())
                .and_then(Value::as_str)
            else {
                continue;
            };
            if let Some(text) = text_nodes.get(link) {
                target.push(text.clone());
                has_classified_prompt = true;
            }
        }
    }

    if !has_classified_prompt {
        positive.extend(text_nodes.into_values());
    }
    positive.sort();
    positive.dedup();
    negative.sort();
    negative.dedup();
    (positive, negative)
}

fn normalize(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }
    let mut end = trimmed.len().min(MAX_COMMENT_BYTES);
    while end > 0 && !trimmed.is_char_boundary(end) {
        end -= 1;
    }
    Some(trimmed[..end].to_string())
}

#[cfg(test)]
mod tests {
    use super::{extract_comment, format_ai_metadata, format_prompt_metadata, PNG_SIGNATURE};
    use flate2::write::GzEncoder;
    use flate2::Compression;
    use image::{Rgba, RgbaImage};
    use std::fs;
    use std::io::Write;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn preserves_automatic1111_parameters() {
        let metadata = format_ai_metadata("parameters", "cat\nNegative prompt: blur\nSteps: 20")
            .expect("parameters should be recognized");
        assert_eq!(metadata.1, "cat\nNegative prompt: blur\nSteps: 20");
    }

    #[test]
    fn extracts_comfy_text_nodes() {
        let metadata = format_prompt_metadata(
            r#"{
                "1":{"class_type":"CLIPTextEncode","inputs":{"text":"a mountain"}},
                "2":{"class_type":"CLIPTextEncode","inputs":{"text":"blurry"}},
                "3":{"class_type":"KSampler","inputs":{"positive":["1",0],"negative":["2",0]}}
            }"#,
        )
        .expect("ComfyUI prompt should be recognized");
        assert_eq!(metadata, "Prompt:\na mountain\nNegative prompt:\nblurry");
    }

    #[test]
    fn extracts_parameters_from_png_text_chunk() {
        let mut png = PNG_SIGNATURE.to_vec();
        let text = b"Title\0ignored";
        png.extend_from_slice(&(text.len() as u32).to_be_bytes());
        png.extend_from_slice(b"tEXt");
        png.extend_from_slice(text);
        png.extend_from_slice(&[0; 4]);
        png.extend_from_slice(&0u32.to_be_bytes());
        png.extend_from_slice(b"IDAT");
        png.extend_from_slice(&[0; 4]);
        let post_idat_text = b"parameters\0cat in space\nSteps: 20";
        png.extend_from_slice(&(post_idat_text.len() as u32).to_be_bytes());
        png.extend_from_slice(b"tEXt");
        png.extend_from_slice(post_idat_text);
        png.extend_from_slice(&[0; 4]);

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after the epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("lap-ai-png-{timestamp}.png"));
        fs::write(&path, png).expect("fixture should be written");

        let comment = extract_comment(path.to_str().expect("path should be UTF-8"));
        let _ = fs::remove_file(path);
        assert_eq!(comment.as_deref(), Some("cat in space\nSteps: 20"));
    }

    #[test]
    fn extracts_novelai_stealth_comment() {
        let metadata = br#"{"Comment":"{\"prompt\":\"a fox\",\"uc\":\"blurry\",\"steps\":20}"}"#;
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder
            .write_all(metadata)
            .expect("metadata should compress");
        let compressed = encoder.finish().expect("gzip should finish");

        let mut payload = b"stealth_pngcomp".to_vec();
        payload.extend_from_slice(&((compressed.len() as u32) * 8).to_be_bytes());
        payload.extend_from_slice(&compressed);

        let mut image = RgbaImage::from_pixel(256, 256, Rgba([0, 0, 0, 0]));
        let mut bit_index = 0usize;
        for x in 0..image.width() {
            for y in 0..image.height() {
                if bit_index >= payload.len() * 8 {
                    break;
                }
                let byte = payload[bit_index / 8];
                let bit = (byte >> (7 - (bit_index % 8))) & 1;
                image.get_pixel_mut(x, y).0[3] = bit;
                bit_index += 1;
            }
        }

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after the epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("lap-novelai-{timestamp}.png"));
        image.save(&path).expect("fixture should be written");

        let comment = extract_comment(path.to_str().expect("path should be UTF-8"));
        let _ = fs::remove_file(path);
        assert_eq!(
            comment.as_deref(),
            Some("Prompt:\na fox\nNegative prompt:\nblurry\nSteps: 20")
        );
    }
}
