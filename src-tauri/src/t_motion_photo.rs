//! Android Motion Photo detection.
//!
//! A Motion Photo (Samsung "Motion Photo", Google Camera / Pixel, Xiaomi,
//! OPPO, OnePlus, …) is a plain-looking JPEG whose bytes are followed by an
//! embedded MP4 video. Unlike Apple Live Photos the video is *not* a separate
//! sidecar file, so it travels with the JPEG automatically and only needs to be
//! detected and (for playback) extracted.
//!
//! Detection is byte-level and vendor-agnostic, mirroring the widely used
//! `sm_motion_photo` approach:
//!   1. A plain JPEG ends with the EOI marker `FF D9`; a motion photo does not
//!      (it ends in MP4 `mdat` bytes), so that is used as a fast reject.
//!   2. Otherwise we scan for the Samsung `MotionPhoto_Data` marker, then for an
//!      MP4 `ftyp` box (4-byte size + `ftyp` + 4-byte major brand).

use std::fs;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

/// Files smaller than this are never motion photos (the embedded video alone is
/// always larger).
const MIN_FILE_SIZE: u64 = 64 * 1024;
/// An embedded video smaller than this is treated as a false positive.
const MIN_VIDEO_SIZE: u64 = 1024;

/// XMP metadata (which holds the authoritative video length) lives in the
/// first APP segments of a JPEG, so a bounded header read resolves most motion
/// photos without loading the whole file.
const XMP_HEADER_READ_SIZE: usize = 1024 * 1024;

const MOTION_PHOTO_DATA_MARKER: &[u8] = b"MotionPhoto_Data";

/// ISOBMFF major brands produced by phone cameras. Android 12 changed the MP4
/// brand from `ftypmp4`/`ftypisom`; a few common variants are covered here.
fn is_mp4_brand(brand: &[u8]) -> bool {
    matches!(
        brand,
        b"isom"
            | b"iso2"
            | b"avc1"
            | b"mp41"
            | b"mp42"
            | b"iso4"
            | b"iso5"
            | b"iso6"
            | b"M4V "
            | b"M4VH"
            | b"MSNV"
            | b"dash"
            | b"3gp4"
            | b"3gp5"
    )
}

/// Returns the byte offset where the embedded MP4 begins, or `None` if the file
/// is not a JPEG motion photo. The video spans from `offset` to end-of-file.
pub fn detect_motion_photo(path: &Path) -> Option<u64> {
    let size = fs::metadata(path).ok()?.len();
    if size < MIN_FILE_SIZE {
        return None;
    }

    // Fast reject: a plain JPEG ends with EOI (`FF D9`). A motion photo has
    // appended MP4 bytes, so its last two bytes are the end of `mdat`, not EOI.
    let mut tail = [0u8; 2];
    {
        let mut f = fs::File::open(path).ok()?;
        f.seek(SeekFrom::End(-2)).ok()?;
        f.read_exact(&mut tail).ok()?;
    }
    if tail == [0xFF, 0xD9] {
        return None;
    }

    // Bounded header read first: XMP (the authoritative offset) is near the
    // start of the file, so most motion photos resolve without a full read.
    if let Some(header) = read_prefix(path, XMP_HEADER_READ_SIZE) {
        if let Some(offset) = find_xmp_video_offset(&header, size as usize) {
            if is_mp4_at_offset(path, offset as u64) && size - offset as u64 >= MIN_VIDEO_SIZE {
                return Some(offset as u64);
            }
        }
    }

    // Fallback: full scan for the Samsung marker / trailing ftyp box. Only
    // reached for files without XMP or with an invalid XMP offset.
    let data = fs::read(path).ok()?;
    let offset = find_video_offset(&data)?;
    if size - offset < MIN_VIDEO_SIZE {
        return None;
    }
    Some(offset)
}

/// Read up to `max` bytes from the start of `path`.
fn read_prefix(path: &Path, max: usize) -> Option<Vec<u8>> {
    let mut f = fs::File::open(path).ok()?;
    let mut buf = vec![0u8; max];
    let n = f.read(&mut buf).ok()?;
    buf.truncate(n);
    Some(buf)
}

/// Returns true if `offset` points at a valid MP4 `ftyp` box in `path`.
pub fn is_mp4_at_offset(path: &Path, offset: u64) -> bool {
    let mut f = match fs::File::open(path) {
        Ok(f) => f,
        Err(_) => return false,
    };
    if f.seek(SeekFrom::Start(offset)).is_err() {
        return false;
    }
    let mut buf = [0u8; 12];
    if f.read(&mut buf).map(|n| n < 12).unwrap_or(true) {
        return false;
    }
    let box_size = u32::from_be_bytes(buf[0..4].try_into().unwrap());
    box_size >= 16 && &buf[4..8] == b"ftyp" && is_mp4_brand(&buf[8..12])
}

fn find_video_offset(data: &[u8]) -> Option<u64> {
    // The Android specification stores the video item length in XMP. This is
    // the authoritative offset and avoids mistaking a later `ftyp` sequence
    // within the MP4 for the beginning of the video.
    if let Some(offset) = find_xmp_video_offset(data, data.len()) {
        if is_mp4_at(data, offset) {
            return Some(offset as u64);
        }
    }

    // Samsung: locate the last `MotionPhoto_Data` marker (the XMP also names
    // the marker, so the first hit is usually metadata, not the payload),
    // then the ftyp box right after it (some devices pad between the marker
    // and the MP4).
    if let Some(marker_pos) = find_subslice_last(data, MOTION_PHOTO_DATA_MARKER) {
        let start = marker_pos + MOTION_PHOTO_DATA_MARKER.len();
        let window_end = (marker_pos + MOTION_PHOTO_DATA_MARKER.len() + 64 * 1024).min(data.len());
        if let Some(ftyp) = find_ftyp_in_range(data, start, window_end) {
            return Some(ftyp as u64);
        }
    }

    // Google / cross-vendor: the last MP4 `ftyp` box in the file.
    find_last_ftyp(data, 0).map(|pos| pos as u64)
}

fn find_xmp_video_offset(data: &[u8], file_size: usize) -> Option<usize> {
    // Byte-level scan: the file can be tens of megabytes while the XMP
    // payload is tiny, so never materialize the whole file as a String.
    // `file_size` is the full file length (the buffer may be a bounded header).

    // Current Container XMP: the MotionPhoto item's length is the number of
    // bytes at the end of the file occupied by the MP4.
    let mut search_from = 0;
    while let Some(relative_pos) = find_subslice(&data[search_from..], b"MotionPhoto") {
        let pos = search_from + relative_pos;
        search_from = pos + b"MotionPhoto".len();
        let Some(tag) = enclosing_tag(data, pos) else {
            continue;
        };
        if let Some(length) = xmp_attribute_u64(tag, "ItemLength")
            .or_else(|| xmp_attribute_u64(tag, "Item:Length"))
        {
            return file_size.checked_sub(length as usize);
        }
    }

    // Legacy MicroVideoOffset has the same end-relative semantics.
    let name_pos = find_subslice(data, b"MicroVideoOffset")?;
    let offset = quoted_u64_at(data, name_pos + b"MicroVideoOffset".len())? as usize;
    file_size.checked_sub(offset)
}

/// The XML tag enclosing `pos`. The scan is bounded so binary garbage that
/// happens to contain the searched bytes cannot trigger a whole-file scan.
fn enclosing_tag(data: &[u8], pos: usize) -> Option<&str> {
    let start = pos.saturating_sub(4 * 1024);
    let tag_start = data[start..pos]
        .iter()
        .rposition(|&b| b == b'<')
        .map(|i| start + i)?;
    let end = (pos + 4 * 1024).min(data.len());
    let tag_end = data[pos..end]
        .iter()
        .position(|&b| b == b'>')
        .map(|i| pos + i)?;
    std::str::from_utf8(&data[tag_start..=tag_end]).ok()
}

fn xmp_attribute_u64(tag: &str, attribute: &str) -> Option<u64> {
    let attribute_pos = tag.find(attribute)? + attribute.len();
    quoted_u64_at(tag.as_bytes(), attribute_pos)
}

/// Parse `= "123"` / `='123'` (whitespace tolerated) right after an attribute
/// name at `start`. Byte-based so it works on windows followed by binary data.
fn quoted_u64_at(data: &[u8], start: usize) -> Option<u64> {
    let mut i = start;
    while i < data.len() && data[i].is_ascii_whitespace() {
        i += 1;
    }
    if data.get(i) != Some(&b'=') {
        return None;
    }
    i += 1;
    while i < data.len() && data[i].is_ascii_whitespace() {
        i += 1;
    }
    let quote = *data.get(i)?;
    if quote != b'"' && quote != b'\'' {
        return None;
    }
    i += 1;
    let value_start = i;
    while i < data.len() && data[i] != quote {
        if !data[i].is_ascii_digit() {
            return None;
        }
        i += 1;
    }
    if i == value_start {
        return None;
    }
    std::str::from_utf8(&data[value_start..i]).ok()?.parse().ok()
}

fn is_mp4_at(data: &[u8], offset: usize) -> bool {
    offset.checked_add(4).is_some_and(|ftyp_pos| is_valid_ftyp_box(data, ftyp_pos))
}

fn has_jpeg_eoi_before(data: &[u8], offset: usize) -> bool {
    data[..offset.min(data.len())]
        .windows(2)
        .any(|marker| marker == [0xFF, 0xD9])
}

fn find_subslice(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() || haystack.len() < needle.len() {
        return None;
    }
    haystack.windows(needle.len()).position(|w| w == needle)
}

fn find_subslice_last(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() || haystack.len() < needle.len() {
        return None;
    }
    haystack.windows(needle.len()).rposition(|w| w == needle)
}

/// Find an `ftyp` box (4-byte size + `ftyp` + valid brand) in `data[start..end)`
/// and return the box start (i.e. the size field offset, not `ftyp` itself).
fn find_ftyp_in_range(data: &[u8], start: usize, end: usize) -> Option<usize> {
    let end = end.min(data.len());
    if start >= end {
        return None;
    }
    let mut i = start;
    while i + 8 <= end {
        if is_valid_ftyp_box(data, i) && has_jpeg_eoi_before(data, i - 4) {
            return Some(i.saturating_sub(4));
        }
        i += 1;
    }
    None
}

/// Find the last MP4 `ftyp` box in the file, returning the box start offset.
fn find_last_ftyp(data: &[u8], start: usize) -> Option<usize> {
    let mut last: Option<usize> = None;
    let mut i = start;
    while i + 8 <= data.len() {
        if is_valid_ftyp_box(data, i) && has_jpeg_eoi_before(data, i - 4) {
            last = Some(i.saturating_sub(4));
            i += 8;
            continue;
        }
        i += 1;
    }
    last
}

fn is_valid_ftyp_box(data: &[u8], ftyp_pos: usize) -> bool {
    if ftyp_pos < 4 || ftyp_pos + 8 > data.len() || &data[ftyp_pos..ftyp_pos + 4] != b"ftyp" {
        return false;
    }
    let box_start = ftyp_pos - 4;
    let box_size = u32::from_be_bytes(data[box_start..ftyp_pos].try_into().unwrap()) as usize;
    box_size >= 16 && box_start.checked_add(box_size).is_some_and(|end| end <= data.len())
        && is_mp4_brand(&data[ftyp_pos + 4..ftyp_pos + 8])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_google_style_ftyp_after_jpeg() {
        // Simulate a JPEG (ends in EOI) followed by an MP4 `ftyp` box.
        let mut data = vec![0u8; 64];
        data[62] = 0xFF;
        data[63] = 0xD9;
        // 4-byte box size + "ftyp" + "isom" brand + some payload.
        let mut mp4 = Vec::new();
        mp4.extend_from_slice(&20u32.to_be_bytes());
        mp4.extend_from_slice(b"ftypisom");
        mp4.extend_from_slice(&[0u8; 12]);
        let offset = data.len() as u64;
        data.extend_from_slice(&mp4);
        assert_eq!(find_video_offset(&data), Some(offset));
    }

    #[test]
    fn detects_samsung_marker_then_ftyp() {
        let mut data = vec![0u8; 16];
        data.extend_from_slice(&[0xFF, 0xD9]);
        data.extend_from_slice(b"MotionPhoto_Data");
        // padding then the MP4 box
        data.extend_from_slice(&[0u8; 8]);
        let offset = data.len() as u64;
        data.extend_from_slice(&32u32.to_be_bytes());
        data.extend_from_slice(b"ftypmp42");
        data.extend_from_slice(&[0u8; 24]);
        assert_eq!(find_video_offset(&data), Some(offset));
    }

    #[test]
    fn rejects_plain_jpeg_without_ftyp() {
        let mut data = vec![0u8; 128];
        data[126] = 0xFF;
        data[127] = 0xD9;
        assert_eq!(find_video_offset(&data), None);
    }

    #[test]
    fn rejects_ftyp_with_non_mp4_brand() {
        let mut data = vec![0u8; 64];
        data.extend_from_slice(b"ftypjunk");
        assert_eq!(find_last_ftyp(&data, 0), None);
    }

    #[test]
    fn rejects_ftyp_before_jpeg_eoi() {
        let mut data = Vec::new();
        data.extend_from_slice(&20u32.to_be_bytes());
        data.extend_from_slice(b"ftypisom");
        data.extend_from_slice(&[0u8; 12]);
        data.extend_from_slice(&[0xFF, 0xD9]);
        assert_eq!(find_video_offset(&data), None);
    }

    #[test]
    fn uses_xmp_motion_photo_item_length() {
        let video = [
            0, 0, 0, 20, b'f', b't', b'y', b'p', b'i', b's', b'o', b'm', 0, 0, 0, 0, 0, 0,
            0, 0,
        ];
        let mut data = format!(
            "<rdf:li GContainer:ItemSemantic=\"MotionPhoto\" GContainer:ItemLength=\"{}\"/>",
            video.len(),
        )
        .into_bytes();
        data.extend_from_slice(&[0xFF, 0xD9]);
        let offset = data.len() as u64;
        data.extend_from_slice(&video);
        // Assert on the XMP path directly: the data also contains a valid
        // ftyp-after-EOI pattern the fallback could match.
        assert_eq!(find_xmp_video_offset(&data, data.len()), Some(offset as usize));
        assert_eq!(find_video_offset(&data), Some(offset));
    }

    #[test]
    fn uses_legacy_micro_video_offset() {
        let video = [
            0, 0, 0, 20, b'f', b't', b'y', b'p', b'i', b's', b'o', b'm', 0, 0, 0, 0, 0, 0,
            0, 0,
        ];
        let mut data = format!("GCamera:MicroVideoOffset=\"{}\"", video.len()).into_bytes();
        data.extend_from_slice(&[0xFF, 0xD9]);
        let offset = data.len() as u64;
        data.extend_from_slice(&video);
        assert_eq!(find_xmp_video_offset(&data, data.len()), Some(offset as usize));
    }

    #[test]
    fn samsung_marker_in_xmp_is_skipped_for_payload_marker() {
        // The XMP names the marker before the payload marker actually appears.
        let mut data = b"<x:MotionPhoto_Data/>".to_vec();
        data.extend_from_slice(&[0u8; 8]);
        data.extend_from_slice(&[0xFF, 0xD9]);
        data.extend_from_slice(b"MotionPhoto_Data");
        data.extend_from_slice(&[0u8; 4]);
        let offset = data.len() as u64;
        data.extend_from_slice(&24u32.to_be_bytes());
        data.extend_from_slice(b"ftypmp42");
        data.extend_from_slice(&[0u8; 16]);
        assert_eq!(find_video_offset(&data), Some(offset));
    }
}
