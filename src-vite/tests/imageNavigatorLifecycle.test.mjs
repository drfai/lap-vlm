import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import ts from 'typescript';

// Exercise the component's real buffer-switch and viewport calculations,
// without requiring a Tauri window or asynchronous image decoding.
const source = fs.readFileSync(new URL('../src/components/Image.vue', import.meta.url), 'utf8');
const ready = source.slice(source.indexOf('const onImageReady ='), source.indexOf('const rotateView ='));
const viewport = source.slice(source.indexOf('function getViewportState()'), source.indexOf('// Expose methods'));
const script = ts.transpile(ready + viewport, { target: ts.ScriptTarget.ES2022 });

function setup(deferred = false) {
  const ref = value => ({ value });
  const state = {
    activeImage: ref(0),
    imageSize: ref([{ width: 4000, height: 2000 }, { width: 2000, height: 1000 }]),
    imageSizeRotated: ref([{ width: 4000, height: 2000 }, { width: 2000, height: 1000 }]),
    imageRotate: ref([0, 0]),
    scale: ref([1, 1]),
    position: ref([{ x: -1100, y: -800 }, { x: 0, y: 0 }]),
    containerSize: ref(deferred ? { width: 0, height: 0 } : { width: 1000, height: 800 }),
    isZoomFit: ref(false), isGrabbing: ref(true), isLoading: ref(true),
    noTransition: ref(false), suppressViewportEmit: ref(false), minScale: ref(0.1), maxScale: ref(10),
    gestureType: ref('none'), props: { fileId: 42 }, loadingTimeout: null,
  };
  const events = [];
  let resize;
  const dependencies = {
    ...state, emit: (event, value) => events.push({ event, value }), triggerRef: () => {},
    requestAnimationFrame: () => {}, setTimeout: () => {}, clearTimeout: () => {},
    watch: (_, callback) => { resize = callback; return () => {}; },
  };
  const readyImage = new Function(...Object.keys(dependencies), script + '; return onImageReady;')(...Object.values(dependencies));
  return { state, events, readyImage, resize() {
    state.containerSize.value = { width: 1000, height: 800 };
    resize(state.containerSize.value);
  } };
}

test('non-fit buffer switch publishes the new file viewport after silent restore', () => {
  const t = setup();
  t.readyImage(1);
  assert.equal(t.state.activeImage.value, 1);
  assert.equal(t.events.length, 1);
  assert.equal(t.events[0].event, 'viewport-change');
  assert.deepEqual(t.events[0].value, {
    fileId: 42, scale: 1, normX: 0.4, normY: 0.6, sourceWidth: 2000, sourceHeight: 1000,
    viewportWidth: 1000, viewportHeight: 800, rotate: 0, pannable: true,
  });
});

test('deferred layout publishes only after the new image has a viewport', () => {
  const t = setup(true);
  t.readyImage(1, true);
  assert.equal(t.state.isLoading.value, true, 'placeholder retains the loading indicator');
  assert.equal(t.events.length, 0);
  t.resize();
  assert.equal(t.events.length, 1);
  assert.equal(t.events[0].value.fileId, 42);
  assert.equal(t.events[0].value.sourceWidth, 2000);
  assert.equal(t.events[0].value.viewportWidth, 1000);
});
