import { test } from 'node:test';
import assert from 'node:assert/strict';
import { navigatorLayout, navigatorBox, navigatorPoint, type NavigatorViewport } from '../src/common/imageNavigator.ts';
const near = (actual: number, expected: number) => assert.ok(Math.abs(actual - expected) < 1e-8, `${actual} != ${expected}`);
const viewport: NavigatorViewport = { scale: 2, normX: 0.3, normY: 0.7,
  sourceWidth: 4000, sourceHeight: 2000, viewportWidth: 1000, viewportHeight: 800, rotate: 0, pannable: true };
for (const rotate of [0, 90, 180, 270, -90]) {
  test(`rotation ${rotate}: viewport center maps back to the same image content`, () => {
    const view = { ...viewport, rotate };
    const layout = navigatorLayout(view, { width: 320, height: 240 });
    const box = navigatorBox(view, layout);
    const point = navigatorPoint(view, layout, layout.left + box.left + box.width / 2, layout.top + box.top + box.height / 2);
    near(point.normX, view.normX); near(point.normY, view.normY);
    near(box.width, view.viewportWidth / view.scale * layout.ratio);
    near(box.height, view.viewportHeight / view.scale * layout.ratio);
    assert.ok(layout.width <= 320 + 1e-8 && layout.height <= 240 + 1e-8);
  });
}
test('letterboxing is excluded from navigator coordinates', () => {
  const view = { ...viewport, normX: 0.5, normY: 0.5 };
  const layout = navigatorLayout(view, { width: 300, height: 300 });
  near(layout.top, 75);
  const point = navigatorPoint(view, layout, 0, 75);
  near(point.normX, 0); near(point.normY, 0);
});
test('thumbnail and original resolutions describe the same viewport', () => {
  const thumbnail = { ...viewport, sourceWidth: 400, sourceHeight: 200, scale: 20 };
  const size = { width: 320, height: 240 };
  const originalBox = navigatorBox(viewport, navigatorLayout(viewport, size));
  const thumbnailBox = navigatorBox(thumbnail, navigatorLayout(thumbnail, size));
  for (const key of ['left', 'top', 'width', 'height'] as const) near(originalBox[key], thumbnailBox[key]);
});
