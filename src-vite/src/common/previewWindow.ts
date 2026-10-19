import { getCurrentWindow } from '@tauri-apps/api/window';
import { getCurrentWebview } from '@tauri-apps/api/webview';

export const PREVIEW_WINDOW_FOCUS_RESTORED = 'preview-window-focus-restored';

// Module state belongs to the main webview, not to an individual preview.
let queue: Promise<unknown> = Promise.resolve();
let recoveryGeneration = 0;
let stopRecovery: (() => void) | undefined;

export function runPreviewWindowOperation<T>(
  operation: (cancelFocusRecovery: () => void) => Promise<T>,
): Promise<T> {
  const result = queue.then(() => operation(() => {
    // The caller first checks whether it will actually change the window.
    // A queued no-op must preserve recovery from the preceding operation.
    ++recoveryGeneration;
    stopRecovery?.();
    stopRecovery = undefined;
  }));
  queue = result.catch(() => {});
  return result;
}

export async function recoverPreviewWindowFocus() {
  const generation = ++recoveryGeneration;
  stopRecovery?.();
  const appWindow = getCurrentWindow();
  const restore = async () => {
    try {
      if (generation !== recoveryGeneration || !(await appWindow.isFocused())) return;
      if (generation !== recoveryGeneration) return;
      await getCurrentWebview().setFocus();
      if (generation === recoveryGeneration) {
        window.dispatchEvent(new Event(PREVIEW_WINDOW_FOCUS_RESTORED));
      }
    } catch (error) {
      console.error('Failed to restore preview window focus', error);
    }
  };
  // macOS's final resize follows the native fullscreen animation/style restore.
  // This listener survives preview unmount, but is cancelled before another
  // preview starts a window operation so it cannot interfere with that session.
  const unlisten = await appWindow.onResized(() => { void restore(); });
  if (generation !== recoveryGeneration) {
    unlisten();
    return;
  }
  const timer = setTimeout(() => {
    if (generation === recoveryGeneration) {
      stopRecovery?.();
      stopRecovery = undefined;
      ++recoveryGeneration;
    }
  }, 2000);
  stopRecovery = () => { clearTimeout(timer); unlisten(); };
  await restore();
}
