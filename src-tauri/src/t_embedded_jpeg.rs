//! Bounded JPEG preview extraction independent of the RAW decoder.
use std::io::{BufReader, Cursor, Read, Seek, SeekFrom};

use exif::{Exif, In, Tag};
use image::{ImageFormat, ImageReader};

const MAX_JPEG_BYTES: u64 = 64 * 1024 * 1024;
const MAX_JPEG_PIXELS: u64 = 100_000_000;

/// Follow only the bounded top-level TIFF directory chain. Some Sony files put
/// the full-size preview IFD beyond the usual 128 KiB metadata prefix.
pub fn read_tiff_metadata<R: Read + Seek>(source: &mut R) -> Result<Exif, String> {
    const MAX_METADATA: u64 = 1024 * 1024;
    let file_len = source.seek(SeekFrom::End(0)).map_err(|e| e.to_string())?;
    source.seek(SeekFrom::Start(0)).map_err(|e| e.to_string())?;
    let mut header = [0; 8];
    source.read_exact(&mut header).map_err(|e| e.to_string())?;
    let little = &header[..4] == b"II\x2a\x00";
    if !little && &header[..4] != b"MM\x00\x2a" {
        return Err("Not a classic TIFF file".into());
    }
    let read_u32 = |b: [u8; 4]| {
        if little {
            u32::from_le_bytes(b)
        } else {
            u32::from_be_bytes(b)
        }
    };
    let mut offset = u64::from(read_u32(header[4..8].try_into().unwrap()));
    let mut extent = file_len.min(128 * 1024);
    let mut visited = Vec::new();
    for _ in 0..8 {
        if offset == 0 || visited.contains(&offset) || offset + 2 > file_len.min(MAX_METADATA) {
            break;
        }
        visited.push(offset);
        source
            .seek(SeekFrom::Start(offset))
            .map_err(|e| e.to_string())?;
        let mut count = [0; 2];
        source.read_exact(&mut count).map_err(|e| e.to_string())?;
        let count = if little {
            u16::from_le_bytes(count)
        } else {
            u16::from_be_bytes(count)
        };
        let end = offset + 2 + u64::from(count) * 12 + 4;
        if end > file_len.min(MAX_METADATA) {
            break;
        }
        extent = extent.max(end);
        source
            .seek(SeekFrom::Start(end - 4))
            .map_err(|e| e.to_string())?;
        let mut next = [0; 4];
        source.read_exact(&mut next).map_err(|e| e.to_string())?;
        offset = u64::from(read_u32(next));
    }
    source.seek(SeekFrom::Start(0)).map_err(|e| e.to_string())?;
    let mut metadata = vec![0; extent as usize];
    source
        .read_exact(&mut metadata)
        .map_err(|e| e.to_string())?;
    exif::Reader::new()
        .continue_on_error(true)
        .read_raw(metadata)
        .or_else(|e| e.distill_partial_result(|_| {}))
        .map_err(|e| e.to_string())
}

pub struct Preview {
    pub data: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub orientation: i32,
}

/// Offsets are relative to the TIFF header: use the RAW file for TIFF RAWs,
/// or the EXIF TIFF buffer for other containers. Only decode JPEG headers while
/// choosing a candidate; read the selected payload once.
pub fn select<R: Read + Seek>(
    source: &mut R,
    exif: &Exif,
    thumbnail_size: Option<u32>,
) -> Result<Option<Preview>, String> {
    let file_len = source.seek(SeekFrom::End(0)).map_err(|e| e.to_string())?;
    let mut candidates = Vec::new();
    for index in 0..8 {
        let ifd = In(index);
        let value = |tag| exif.get_field(tag, ifd).and_then(|f| f.value.get_uint(0));
        let (offset, len) = match (
            value(Tag::JPEGInterchangeFormat),
            value(Tag::JPEGInterchangeFormatLength),
        ) {
            (Some(offset), Some(len)) => (u64::from(offset), u64::from(len)),
            _ => continue,
        };
        if !(4..=MAX_JPEG_BYTES).contains(&len)
            || offset.checked_add(len).is_none_or(|end| end > file_len)
        {
            continue;
        }
        source
            .seek(SeekFrom::Start(offset))
            .map_err(|e| e.to_string())?;
        let dimensions =
            ImageReader::with_format(BufReader::new((&mut *source).take(len)), ImageFormat::Jpeg)
                .into_dimensions();
        let (width, height) = match dimensions {
            Ok((w, h)) if w > 0 && h > 0 && u64::from(w) * u64::from(h) <= MAX_JPEG_PIXELS => {
                (w, h)
            }
            _ => continue,
        };
        let orientation = value(Tag::Orientation)
            .filter(|v| (1..=8).contains(v))
            .unwrap_or(1);
        candidates.push((offset, len, width, height, orientation));
    }

    // For thumbnails prefer the smallest sufficient image, otherwise the largest.
    // For viewing prefer the largest image, without consulting LibRaw dimensions.
    let selected = if let Some(size) = thumbnail_size {
        candidates
            .iter()
            .filter(|c| c.2.max(c.3) >= size)
            .min_by_key(|c| c.2.max(c.3))
            .or_else(|| candidates.iter().max_by_key(|c| c.2.max(c.3)))
    } else {
        candidates
            .iter()
            .max_by_key(|c| u64::from(c.2) * u64::from(c.3))
    };
    let &(offset, len, width, height, ifd_orientation) = match selected {
        Some(candidate) => candidate,
        None => return Ok(None),
    };
    source
        .seek(SeekFrom::Start(offset))
        .map_err(|e| e.to_string())?;
    let mut data = vec![0; len as usize];
    source.read_exact(&mut data).map_err(|e| e.to_string())?;
    let orientation = exif::Reader::new()
        .read_from_container(&mut Cursor::new(&data))
        .ok()
        .and_then(|e| {
            e.get_field(Tag::Orientation, In::PRIMARY)
                .and_then(|f| f.value.get_uint(0))
        })
        .filter(|v| (1..=8).contains(v))
        .unwrap_or(ifd_orientation) as i32;
    Ok(Some(Preview {
        data,
        width,
        height,
        orientation,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn jpeg(w: u32, h: u32) -> Vec<u8> {
        let mut out = Cursor::new(Vec::new());
        image::DynamicImage::new_rgb8(w, h)
            .write_to(&mut out, ImageFormat::Jpeg)
            .unwrap();
        out.into_inner()
    }

    fn fixture(big_endian: bool, images: &[(Vec<u8>, u16)]) -> Vec<u8> {
        let u16bytes = |n: u16| {
            if big_endian {
                n.to_be_bytes()
            } else {
                n.to_le_bytes()
            }
        };
        let u32bytes = |n: u32| {
            if big_endian {
                n.to_be_bytes()
            } else {
                n.to_le_bytes()
            }
        };
        let mut data = vec![0; 200_000];
        data[..4].copy_from_slice(if big_endian {
            b"MM\x00\x2a"
        } else {
            b"II\x2a\x00"
        });
        data[4..8].copy_from_slice(&u32bytes(8));
        for (i, (jpeg, orientation)) in images.iter().enumerate() {
            let start = 8 + i * 42;
            data[start..start + 2].copy_from_slice(&u16bytes(3));
            let offset = data.len() as u32;
            for (j, (tag, typ, value)) in [
                (0x112u16, 3u16, u32::from(*orientation)),
                (0x201, 4, offset),
                (0x202, 4, jpeg.len() as u32),
            ]
            .into_iter()
            .enumerate()
            {
                let at = start + 2 + j * 12;
                data[at..at + 2].copy_from_slice(&u16bytes(tag));
                data[at + 2..at + 4].copy_from_slice(&u16bytes(typ));
                data[at + 4..at + 8].copy_from_slice(&u32bytes(1));
                if typ == 3 {
                    data[at + 8..at + 10].copy_from_slice(&u16bytes(value as u16));
                } else {
                    data[at + 8..at + 12].copy_from_slice(&u32bytes(value));
                }
            }
            if i + 1 < images.len() {
                data[start + 38..start + 42].copy_from_slice(&u32bytes((start + 42) as u32));
            }
            data.extend_from_slice(jpeg);
        }
        data
    }

    fn metadata(data: &[u8]) -> Exif {
        exif::Reader::new()
            .read_raw(data[..128 * 1024].to_vec())
            .unwrap()
    }

    #[test]
    fn reads_beyond_metadata_buffer_and_selects_by_purpose() {
        for big_endian in [false, true] {
            let data = fixture(
                big_endian,
                &[(jpeg(16, 12), 1), (jpeg(160, 120), 6), (jpeg(640, 480), 8)],
            );
            let exif = metadata(&data);
            let large = select(&mut Cursor::new(&data), &exif, None)
                .unwrap()
                .unwrap();
            assert_eq!(
                (large.width, large.height, large.orientation),
                (640, 480, 8)
            );
            assert!(image::load_from_memory(&large.data).is_ok());
            let thumb = select(&mut Cursor::new(&data), &exif, Some(100))
                .unwrap()
                .unwrap();
            assert_eq!(
                (thumb.width, thumb.height, thumb.orientation),
                (160, 120, 6)
            );
            let undersized = select(&mut Cursor::new(&data), &exif, Some(1000))
                .unwrap()
                .unwrap();
            assert_eq!(undersized.width, 640);
        }
    }

    #[test]
    fn follows_preview_directory_beyond_128k_and_stops_cycles() {
        for big in [false, true] {
            let mut data = fixture(big, &[(jpeg(16, 12), 1), (jpeg(640, 480), 6)]);
            let next = if big {
                145074u32.to_be_bytes()
            } else {
                145074u32.to_le_bytes()
            };
            let directory = data[50..92].to_vec();
            data[145074..145116].copy_from_slice(&directory);
            data[46..50].copy_from_slice(&next);
            // A cycle must not cause unbounded reads.
            data[145112..145116].copy_from_slice(&next);
            let exif = read_tiff_metadata(&mut Cursor::new(&data)).unwrap();
            let selected = select(&mut Cursor::new(&data), &exif, None)
                .unwrap()
                .unwrap();
            assert_eq!(
                (selected.width, selected.height, selected.orientation),
                (640, 480, 6)
            );
        }
    }

    #[test]
    fn rejects_truncated_and_oversized_payloads() {
        let mut data = fixture(false, &[(jpeg(16, 12), 1)]);
        let exif = metadata(&data);
        data.pop();
        assert!(select(&mut Cursor::new(&data), &exif, None)
            .unwrap()
            .is_none());
        // JPEGInterchangeFormatLength exceeds the bounded allocation limit.
        data[42..46].copy_from_slice(&((MAX_JPEG_BYTES + 1) as u32).to_le_bytes());
        let exif = metadata(&data);
        assert!(select(&mut Cursor::new(&data), &exif, None)
            .unwrap()
            .is_none());
    }

    #[test]
    fn skips_invalid_and_excessive_dimension_headers() {
        let mut huge = jpeg(16, 12);
        let sof = huge.windows(2).position(|v| v == [0xff, 0xc0]).unwrap();
        huge[sof + 5..sof + 7].copy_from_slice(&20000u16.to_be_bytes());
        huge[sof + 7..sof + 9].copy_from_slice(&20000u16.to_be_bytes());
        let data = fixture(
            false,
            &[(b"not a jpeg".to_vec(), 1), (huge, 1), (jpeg(16, 12), 1)],
        );
        let result = select(&mut Cursor::new(&data), &metadata(&data), None)
            .unwrap()
            .unwrap();
        assert_eq!((result.width, result.height), (16, 12));
    }

    #[test]
    fn reads_bounded_metadata_and_selected_payload_without_scanning_raw() {
        struct Counted {
            cursor: Cursor<Vec<u8>>,
            bytes: usize,
        }
        impl Read for Counted {
            fn read(&mut self, b: &mut [u8]) -> std::io::Result<usize> {
                let n = self.cursor.read(b)?;
                self.bytes += n;
                Ok(n)
            }
        }
        impl Seek for Counted {
            fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
                self.cursor.seek(pos)
            }
        }
        let mut data = fixture(false, &[(jpeg(160, 120), 1), (jpeg(640, 480), 1)]);
        data.resize(8 * 1024 * 1024, 0);
        let mut source = Counted {
            cursor: Cursor::new(data),
            bytes: 0,
        };
        let metadata = read_tiff_metadata(&mut source).unwrap();
        let result = select(&mut source, &metadata, Some(100)).unwrap().unwrap();
        assert_eq!(result.width, 160);
        assert!(source.bytes < 256 * 1024, "read {} bytes", source.bytes);
    }

    #[test]
    fn jpeg_orientation_overrides_ifd_orientation() {
        let mut jpeg = jpeg(16, 12);
        let tiff = fixture(false, &[(Vec::new(), 3)]);
        let mut app1 = b"Exif\0\0".to_vec();
        app1.extend_from_slice(&tiff[..50]);
        let mut segment = vec![0xff, 0xe1];
        segment.extend_from_slice(&((app1.len() + 2) as u16).to_be_bytes());
        segment.extend(app1);
        jpeg.splice(2..2, segment);
        let data = fixture(false, &[(jpeg, 6)]);
        let result = select(&mut Cursor::new(&data), &metadata(&data), None)
            .unwrap()
            .unwrap();
        assert_eq!(result.orientation, 3);
    }
}
