let activePreviewStop: (() => void) | null = null;

export function claimHoverPreview(stop: () => void) {
  if (activePreviewStop === stop) return;
  activePreviewStop?.();
  activePreviewStop = stop;
}

export function releaseHoverPreview(stop: () => void) {
  if (activePreviewStop === stop) activePreviewStop = null;
}
