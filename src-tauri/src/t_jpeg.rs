use image::RgbImage;
use std::ffi::CStr;
use std::os::raw::{c_char, c_int, c_ulong};

#[link(name = "lap_libraw_shim", kind = "static")]
unsafe extern "C" {
    fn lap_jpeg_encode_rgb8(
        rgb_data: *const u8,
        width: u32,
        height: u32,
        quality: c_int,
        out_data: *mut *mut u8,
        out_len: *mut c_ulong,
        err_buf: *mut c_char,
        err_buf_len: usize,
    ) -> c_int;
    fn lap_jpeg_decode_rgb8(
        file_path: *const c_char,
        target_width: u32,
        target_height: u32,
        out_width: *mut u32,
        out_height: *mut u32,
        out_data: *mut *mut u8,
        err_buf: *mut c_char,
        err_buf_len: usize,
    ) -> c_int;
    fn lap_jpeg_free_buffer(data: *mut u8);
}

pub fn decode_rgb8_scaled(
    file_path: &str,
    target_width: u32,
    target_height: u32,
) -> Result<(Vec<u8>, u32, u32), String> {
    let mut out_data: *mut u8 = std::ptr::null_mut();
    let mut out_width: u32 = 0;
    let mut out_height: u32 = 0;
    let mut err_buf = vec![0 as c_char; 512];

    let c_path = std::ffi::CString::new(file_path).map_err(|e| e.to_string())?;

    let ok = unsafe {
        lap_jpeg_decode_rgb8(
            c_path.as_ptr(),
            target_width,
            target_height,
            &mut out_width,
            &mut out_height,
            &mut out_data,
            err_buf.as_mut_ptr(),
            err_buf.len(),
        )
    };

    if ok == 0 {
        let err = unsafe { CStr::from_ptr(err_buf.as_ptr()) }
            .to_string_lossy()
            .trim()
            .to_string();
        return Err(if err.is_empty() {
            "libjpeg-turbo decode failed".to_string()
        } else {
            format!("libjpeg-turbo decode failed: {}", err)
        });
    }

    if out_data.is_null() || out_width == 0 || out_height == 0 {
        return Err("libjpeg-turbo returned empty pixels".to_string());
    }

    let bytes = unsafe {
        let len = (out_width as usize) * (out_height as usize) * 3;
        let slice = std::slice::from_raw_parts(out_data, len);
        let result = slice.to_vec();
        lap_jpeg_free_buffer(out_data);
        result
    };

    Ok((bytes, out_width, out_height))
}

pub fn encode_rgb8(rgb: &RgbImage, quality: u8) -> Result<Vec<u8>, String> {
    let mut out_data: *mut u8 = std::ptr::null_mut();
    let mut out_len: c_ulong = 0;
    let mut err_buf = vec![0 as c_char; 512];

    let ok = unsafe {
        lap_jpeg_encode_rgb8(
            rgb.as_raw().as_ptr(),
            rgb.width(),
            rgb.height(),
            quality as c_int,
            &mut out_data,
            &mut out_len,
            err_buf.as_mut_ptr(),
            err_buf.len(),
        )
    };

    if ok == 0 {
        let err = unsafe { CStr::from_ptr(err_buf.as_ptr()) }
            .to_string_lossy()
            .trim()
            .to_string();
        return Err(if err.is_empty() {
            "libjpeg-turbo encode failed".to_string()
        } else {
            format!("libjpeg-turbo encode failed: {}", err)
        });
    }

    if out_data.is_null() || out_len == 0 {
        return Err("libjpeg-turbo returned empty JPEG output".to_string());
    }

    let bytes = unsafe {
        let slice = std::slice::from_raw_parts(out_data, out_len as usize);
        let result = slice.to_vec();
        lap_jpeg_free_buffer(out_data);
        result
    };

    Ok(bytes)
}
