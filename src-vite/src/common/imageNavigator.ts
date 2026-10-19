export interface NavigatorViewport {
  fileId?: number;
  scale: number;
  normX: number;
  normY: number;
  sourceWidth: number;
  sourceHeight: number;
  viewportWidth: number;
  viewportHeight: number;
  rotate: number;
  pannable: boolean;
}

export function navigatorLayout(viewport: NavigatorViewport, size: { width: number; height: number }) {
  const sourceWidth = viewport?.sourceWidth || 1;
  const sourceHeight = viewport?.sourceHeight || 1;
  const angle = (viewport?.rotate || 0) * Math.PI / 180;
  const cos = Math.abs(Math.cos(angle));
  const sin = Math.abs(Math.sin(angle));
  const width = sourceWidth * cos + sourceHeight * sin;
  const height = sourceWidth * sin + sourceHeight * cos;
  const ratio = Math.min(size.width / width, size.height / height);
  return { width: width * ratio, height: height * ratio, left: (size.width - width * ratio) / 2,
    top: (size.height - height * ratio) / 2, imageWidth: sourceWidth * ratio, imageHeight: sourceHeight * ratio, ratio };
}

export function navigatorPoint(viewport: NavigatorViewport, layout: ReturnType<typeof navigatorLayout>, x: number, y: number) {
  const dx = x - layout.left - layout.width / 2;
  const dy = y - layout.top - layout.height / 2;
  const angle = viewport.rotate * Math.PI / 180;
  return { normX: 0.5 + (dx * Math.cos(angle) + dy * Math.sin(angle)) / layout.imageWidth,
    normY: 0.5 + (-dx * Math.sin(angle) + dy * Math.cos(angle)) / layout.imageHeight };
}

export function navigatorBox(viewport: NavigatorViewport, layout: ReturnType<typeof navigatorLayout>) {
  const angle = viewport.rotate * Math.PI / 180;
  const dx = (viewport.normX - 0.5) * layout.imageWidth;
  const dy = (viewport.normY - 0.5) * layout.imageHeight;
  const width = viewport.viewportWidth / viewport.scale * layout.ratio;
  const height = viewport.viewportHeight / viewport.scale * layout.ratio;
  return { width, height, left: layout.width / 2 + dx * Math.cos(angle) - dy * Math.sin(angle) - width / 2,
    top: layout.height / 2 + dx * Math.sin(angle) + dy * Math.cos(angle) - height / 2 };
}
