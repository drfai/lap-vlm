//! Native file drag-out via CrabNebula drag-rs. Original files are offered as copies.
use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
};

use tauri::{ipc::Channel, AppHandle, Emitter, Listener, Manager, Window};

const MAX_NATIVE_DRAG_FILES: usize = 1000;

static PENDING: std::sync::LazyLock<Mutex<HashMap<String, Arc<AtomicBool>>>> =
    std::sync::LazyLock::new(|| Mutex::new(HashMap::new()));

struct PendingDrag(String);
impl Drop for PendingDrag {
    fn drop(&mut self) {
        PENDING.lock().unwrap().remove(&self.0);
    }
}

// Retained by the native callback after asynchronous macOS/GTK startup.
struct DragSession<F: Fn(bool)> {
    notify: F,
    finished: AtomicBool,
}
impl<F: Fn(bool)> DragSession<F> {
    fn new(notify: F) -> Self {
        notify(true);
        Self {
            notify,
            finished: AtomicBool::new(false),
        }
    }
    fn finish(&self) {
        if !self.finished.swap(true, Ordering::SeqCst) {
            (self.notify)(false);
        }
    }
}
impl<F: Fn(bool)> Drop for DragSession<F> {
    fn drop(&mut self) {
        self.finish();
    }
}

#[tauri::command]
pub fn cancel_file_drag(window: Window, request_id: String) {
    // Scope cancellation to the requesting window.
    if let Some(flag) = PENDING
        .lock()
        .unwrap()
        .get(&format!("{}:{request_id}", window.label()))
    {
        flag.store(true, Ordering::SeqCst);
    }
}

fn validate_paths(paths: Vec<PathBuf>) -> Result<Vec<PathBuf>, String> {
    if paths.is_empty() {
        return Err("No files to drag".into());
    }
    let mut seen = HashSet::new();
    let mut files = Vec::new();
    for path in paths {
        if !path.is_absolute() {
            return Err("Drag paths must be absolute".into());
        }
        let metadata = std::fs::metadata(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        if !metadata.is_file() {
            return Err(format!("Not a file: {}", path.display()));
        }
        if seen.insert(path.clone()) {
            files.push(path);
        }
    }
    Ok(files)
}

#[tauri::command]
pub async fn start_file_drag(
    app: AppHandle,
    window: Window,
    paths: Vec<PathBuf>,
    preview: Vec<u8>,
    on_event: Channel<String>,
) -> Result<(), String> {
    validate_drag_count(paths.len())?;
    let destroyed_listener = Arc::new(Mutex::new(None));
    let listener_id = destroyed_listener.clone();
    let source = window.clone();
    let events = app.clone();
    let session = Arc::new(DragSession::new(move |active| {
        if !active {
            if let Some(id) = listener_id.lock().unwrap().take() {
                source.unlisten(id);
            }
        }
        let _ = events.emit("native-file-drag-active", active);
    }));
    let request_id = uuid::Uuid::new_v4().to_string();
    let key = format!("{}:{request_id}", window.label());
    let cancelled = Arc::new(AtomicBool::new(false));
    PENDING
        .lock()
        .unwrap()
        .insert(key.clone(), cancelled.clone());
    let _pending = PendingDrag(key);
    let weak_session = Arc::downgrade(&session);
    let cancel_on_destroy = cancelled.clone();
    *destroyed_listener.lock().unwrap() = Some(window.once("tauri://destroyed", move |_| {
        cancel_on_destroy.store(true, Ordering::SeqCst);
        if let Some(session) = weak_session.upgrade() {
            session.finish();
        }
    }));
    // Catch destruction that happened before the listener was registered.
    if app.get_webview_window(window.label()).is_none() {
        return Err("Drag source window was closed".into());
    }
    on_event
        .send(format!("Preparing:{request_id}"))
        .map_err(|e| e.to_string())?;
    let preview = normalize_drag_preview(&preview)?;
    let paths = tauri::async_runtime::spawn_blocking(move || validate_paths(paths))
        .await
        .map_err(|e| e.to_string())??;
    let (tx, rx) = tokio::sync::oneshot::channel();
    let native_session = session.clone();
    let startup_session = session.clone();
    app.run_on_main_thread(move || {
        let result = (|| -> Result<(), String> {
            if cancelled.load(Ordering::SeqCst) {
                let _ = on_event.send("Cancel".into());
                return Ok(());
            }
            #[cfg(target_os = "linux")]
            let handle = window.gtk_window().map_err(|e| e.to_string())?;
            #[cfg(not(target_os = "linux"))]
            let handle = window;
            drag::start_drag(
                &handle,
                drag::DragItem::Files(paths),
                drag::Image::Raw(preview),
                move |result, _| {
                    native_session.finish();
                    let _ = on_event.send(format!("{result:?}"));
                },
                drag::Options {
                    cancelled: Some(cancelled),
                    ..Default::default()
                },
            )
            .map_err(|e| e.to_string())
        })();
        if result.is_err() {
            startup_session.finish();
        }
        let _ = tx.send(result);
    })
    .map_err(|e| e.to_string())?;
    rx.await.map_err(|e| e.to_string())?
}

fn validate_drag_count(count: usize) -> Result<(), String> {
    if count > MAX_NATIVE_DRAG_FILES {
        return Err(format!(
            "At most {MAX_NATIVE_DRAG_FILES} files can be dragged out at once"
        ));
    }
    Ok(())
}

fn normalize_drag_preview(preview: &[u8]) -> Result<Vec<u8>, String> {
    if preview.len() > 1024 * 1024 {
        return Err("Drag preview too large".into());
    }
    let mut reader =
        image::ImageReader::with_format(std::io::Cursor::new(preview), image::ImageFormat::Png);
    let mut limits = image::Limits::default();
    limits.max_image_width = Some(256);
    limits.max_image_height = Some(256);
    reader.limits(limits);
    let img = reader
        .decode()
        .map_err(|e| format!("Invalid drag preview: {e}"))?;
    // Pass only the bounded decoded image to the platform decoder.
    let mut png = std::io::Cursor::new(Vec::new());
    img.write_to(&mut png, image::ImageFormat::Png)
        .map_err(|e| e.to_string())?;
    Ok(png.into_inner())
}

#[cfg(test)]
mod tests {
    use super::*;
    // Exercise Windows button mapping on every host.
    mod windows_mouse {
        include!("../vendor/drag/src/platform_impl/windows/mouse.rs");
    }
    // Exercise GTK rollback even on hosts without GTK libraries.
    mod gtk_startup {
        include!("../vendor/drag/src/startup.rs");
    }
    #[test]
    fn drag_count_accepts_limit_and_rejects_excess() {
        assert!(validate_drag_count(1000).is_ok());
        assert!(validate_drag_count(1001).is_err());
    }

    #[test]
    fn preview_is_reencoded_and_rejects_oversized_or_invalid_images() {
        let encode = |width, height| {
            let img = image::DynamicImage::ImageRgba8(image::RgbaImage::new(width, height));
            let mut png = std::io::Cursor::new(Vec::new());
            img.write_to(&mut png, image::ImageFormat::Png).unwrap();
            png.into_inner()
        };
        let original = encode(96, 96);
        let mut with_trailing_data = original.clone();
        with_trailing_data.extend_from_slice(b"untrusted trailing data");
        let normalized = normalize_drag_preview(&with_trailing_data).unwrap();
        assert_eq!(normalized, original);
        let decoded = image::load_from_memory(&normalized).unwrap();
        assert_eq!((decoded.width(), decoded.height()), (96, 96));
        assert!(normalize_drag_preview(&encode(257, 1)).is_err());
        assert!(normalize_drag_preview(&encode(1, 257)).is_err());
        assert!(normalize_drag_preview(&vec![0; 1024 * 1024 + 1]).is_err());
        assert!(normalize_drag_preview(b"not a PNG").is_err());
    }

    #[test]
    fn session_survives_async_native_start_and_cleans_startup_errors() {
        let events = Mutex::new(Vec::new());
        let session = Arc::new(DragSession::new(|active| {
            events.lock().unwrap().push(active)
        }));
        let callback = session.clone();
        drop(session); // macOS/GTK startup returns before the actual drop.
        assert_eq!(*events.lock().unwrap(), vec![true]);
        callback.finish();
        drop(callback);
        assert_eq!(*events.lock().unwrap(), vec![true, false]);
        drop(DragSession::new(|active| {
            events.lock().unwrap().push(active)
        }));
        assert_eq!(*events.lock().unwrap(), vec![true, false, true, false]);
    }

    #[test]
    fn destroyed_source_clears_state_without_frontend_and_late_completion_is_harmless() {
        let events = Mutex::new(Vec::new());
        let session = Arc::new(DragSession::new(|active| {
            events.lock().unwrap().push(active)
        }));
        let destroy_listener = Arc::downgrade(&session);
        // Rust receives window destruction while file validation is still pending.
        destroy_listener.upgrade().unwrap().finish();
        assert_eq!(*events.lock().unwrap(), vec![true, false]);
        let next = DragSession::new(|active| events.lock().unwrap().push(active));
        session.finish(); // The old backend request eventually completes.
        drop(session);
        assert_eq!(*events.lock().unwrap(), vec![true, false, true]);
        drop(next);
        assert_eq!(*events.lock().unwrap(), vec![true, false, true, false]);
    }

    #[test]
    fn rejects_empty_relative_missing_and_directory_paths() {
        assert!(validate_paths(vec![]).is_err());
        assert!(validate_paths(vec![PathBuf::from("relative.png")]).is_err());
        assert!(validate_paths(vec![std::env::temp_dir()]).is_err());
        assert!(validate_paths(vec![std::env::temp_dir().join("lap-missing-drag-file")]).is_err());
    }
    #[test]
    fn preserves_original_paths_and_deduplicates() {
        let file = std::env::temp_dir().join(format!("lap-drag-{}.png", uuid::Uuid::new_v4()));
        std::fs::write(&file, b"test").unwrap();
        let result = validate_paths(vec![file.clone(), file.clone()]);
        std::fs::remove_file(&file).unwrap();
        assert_eq!(result.unwrap(), vec![file]);
    }
}
