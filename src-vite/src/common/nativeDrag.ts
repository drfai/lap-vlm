import { Channel, invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

export const MAX_NATIVE_DRAG_FILES = 1000;

let active = false;
let sharedActive = false;
// Each WebView has its own module instance; share native drag state app-wide.
const sharedStateReady = listen<boolean>('native-file-drag-active', event => {
  sharedActive = event.payload;
});
// Native completion and DOM drop use separate event queues. Remember the source
// when it enters the webview, rather than rechecking only at drop time.
let returningToApp = false;

export function isNativeFileDragActive(): boolean { return active || sharedActive; }

export function isReturningNativeFileDrag(event: DragEvent): boolean {
  const dragging = isNativeFileDragActive();
  if (event.type === 'dragenter') returningToApp = dragging;
  else if (dragging) returningToApp = true;
  const returning = dragging || returningToApp;
  if (event.type === 'drop') returningToApp = false;
  return returning;
}

export function isWindowDragEdge(event: MouseEvent): boolean {
  const margin = 16;
  return event.clientX <= margin || event.clientY <= margin
    || event.clientX >= window.innerWidth - margin
    || event.clientY >= window.innerHeight - margin;
}

// Render synchronously from an already loaded thumbnail; no original decoding or fetch.
export function createDragPreview(image?: HTMLImageElement | null): number[] {
  const canvas = document.createElement('canvas');
  canvas.width = canvas.height = 96;
  const context = canvas.getContext('2d')!;
  context.fillStyle = '#64748b';
  context.fillRect(12, 12, 72, 72);
  if (image?.complete && image.naturalWidth) {
    const scale = Math.min(96 / image.naturalWidth, 96 / image.naturalHeight);
    const width = image.naturalWidth * scale;
    const height = image.naturalHeight * scale;
    try {
      context.clearRect(0, 0, 96, 96);
      context.drawImage(image, (96 - width) / 2, (96 - height) / 2, width, height);
      return Array.from(atob(canvas.toDataURL('image/png').split(',')[1]), c => c.charCodeAt(0));
    } catch { /* Cross-origin thumbnails use the generic preview below. */ }
  }
  const fallback = document.createElement('canvas');
  fallback.width = fallback.height = 96;
  const ctx = fallback.getContext('2d')!;
  ctx.fillStyle = '#64748b';
  ctx.fillRect(12, 12, 72, 72);
  return Array.from(atob(fallback.toDataURL('image/png').split(',')[1]), c => c.charCodeAt(0));
}

export async function startNativeFileDrag(paths: string[], preview: number[]): Promise<void> {
  if (paths.length > MAX_NATIVE_DRAG_FILES) throw new Error(`At most ${MAX_NATIVE_DRAG_FILES} files can be dragged out at once`);
  if (isNativeFileDragActive()) return;
  active = true;
  returningToApp = false;
  let released = false;
  let requestId: string | undefined;
  const cancel = () => {
    released = true;
    if (requestId) void invoke('cancel_file_drag', { requestId }).catch(console.error);
  };
  window.addEventListener('pointerup', cancel, true);
  window.addEventListener('pointercancel', cancel, true);
  window.addEventListener('blur', cancel);
  try {
    await sharedStateReady;
    await new Promise<void>((resolve, reject) => {
      const onEvent = new Channel<string>();
      onEvent.onmessage = message => {
        if (message.startsWith('Preparing:')) {
          requestId = message.slice('Preparing:'.length);
          if (released) cancel();
          return;
        }
        resolve();
      };
      invoke('start_file_drag', { paths, preview, onEvent }).catch(reject);
    });
  } finally {
    window.removeEventListener('pointerup', cancel, true);
    window.removeEventListener('pointercancel', cancel, true);
    window.removeEventListener('blur', cancel);
    active = false;
  }
}
