import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import ts from 'typescript';

const source = fs.readFileSync(new URL('../src/components/MediaViewer.vue', import.meta.url), 'utf8');
const component = ts.transpile(source.slice(source.indexOf('const previewFullScreen ='), source.indexOf('const desktopAppWindow =')), { target: ts.ScriptTarget.ES2022 });
const manager = ts.transpile(fs.readFileSync(new URL('../src/common/previewWindow.ts', import.meta.url), 'utf8').replace(/^import .*;$/gm, '').replace(/export /g, ''), { target: ts.ScriptTarget.ES2022 });

function setup() {
  let nativeFull = false;
  let exitGate;
  let exitStarted;
  let focused = true;
  const calls = [];
  const listeners = new Set();
  const timers = new Map();
  let nextTimer = 0;
  const config = { mediaViewer: { isFullScreen: false } };
  const appWindow = {
    isFocused: async () => focused,
    isFullscreen: async () => nativeFull,
    isMaximized: async () => false,
    setFullscreen: async value => {
      if (!value && exitGate) { exitStarted(); await exitGate; }
      nativeFull = value;
      calls.push(['fullscreen', value]);
    },
    onResized: async callback => { listeners.add(callback); return () => listeners.delete(callback); },
  };
  const getCurrentWindow = () => appWindow;
  const getCurrentWebview = () => ({ setFocus: async () => calls.push(['focus']) });
  const lifecycle = new Function('getCurrentWindow', 'getCurrentWebview', 'window', 'Event', 'setTimeout', 'clearTimeout', manager + '; return { runPreviewWindowOperation, recoverPreviewWindowFocus };')(
    getCurrentWindow, getCurrentWebview, { dispatchEvent: e => calls.push([e.type]) }, Event,
    fn => { const id = ++nextTimer; timers.set(id, fn); return id; }, id => timers.delete(id),
  );
  const create = new Function('ref', 'computed', 'props', 'config', 'getCurrentWindow', 'runPreviewWindowOperation', 'recoverPreviewWindowFocus', 'emit', 'isWin', component + '; return { toggleFullScreen, exitPreviewFullScreen, dispose() { previewDisposed = true; stopPreviewResizeListener(); return exitPreviewFullScreen(); }, get full() { return previewFullScreen.value; } };');
  return {
    calls, config, timers, listeners,
    make: () => create(v => ({ value: v }), f => ({ get value() { return f(); } }), { mode: 0, isFullScreen: false }, config, getCurrentWindow, lifecycle.runPreviewWindowOperation, lifecycle.recoverPreviewWindowFocus, () => {}, false),
    get nativeFull() { return nativeFull; },
    unfocus: () => { focused = false; },
    resize: async () => { await Promise.all([...listeners].map(fn => fn())); await new Promise(resolve => setImmediate(resolve)); },
    blockExit() {
      const started = new Promise(resolve => { exitStarted = resolve; });
      let release;
      exitGate = new Promise(resolve => { release = resolve; });
      return { started, release };
    },
  };
}

test('old preview cleanup finishes before a reopened preview captures native state', async () => {
  const t = setup();
  const first = t.make(); await first.toggleFullScreen();
  const gate = t.blockExit();
  const closing = first.dispose(); await gate.started;
  const second = t.make(); const opening = second.toggleFullScreen();
  await Promise.resolve(); assert.equal(second.full, false);
  gate.release(); await closing; await opening;
  assert.equal(second.full, true); assert.equal(t.nativeFull, true);
  assert.equal(t.config.mediaViewer.isFullScreen, true);
  assert.equal(t.timers.size, 0, 'new session cancels old focus recovery');
  const focusCount = t.calls.filter(c => c[0] === 'focus').length;
  await t.resize(); assert.equal(t.calls.filter(c => c[0] === 'focus').length, focusCount);
});

test('closing fullscreen restores keyboard focus after component disposal and final resize', async () => {
  const t = setup(); const preview = t.make(); await preview.toggleFullScreen();
  await preview.dispose();
  assert.equal(t.nativeFull, false);
  assert.equal(t.config.mediaViewer.isFullScreen, true);
  assert.ok(t.calls.some(c => c[0] === 'preview-window-focus-restored'));
  const count = t.calls.filter(c => c[0] === 'focus').length;
  await t.resize(); assert.equal(t.calls.filter(c => c[0] === 'focus').length, count + 1);
  t.unfocus(); await t.resize(); assert.equal(t.calls.filter(c => c[0] === 'focus').length, count + 1);
  for (const fn of [...t.timers.values()]) fn();
  assert.equal(t.timers.size, 0); assert.equal(t.listeners.size, 0);
});

test('button exit remembers non-fullscreen and still restores keyboard focus', async () => {
  const t = setup(); const preview = t.make(); await preview.toggleFullScreen();
  await preview.exitPreviewFullScreen();
  assert.equal(preview.full, false); assert.equal(t.nativeFull, false);
  assert.equal(t.config.mediaViewer.isFullScreen, false);
  assert.ok(t.calls.some(c => c[0] === 'preview-window-focus-restored'));
});

test('a preview disposed while queued never enters fullscreen', async () => {
  const t = setup(); const preview = t.make();
  const opening = preview.toggleFullScreen(); const closing = preview.dispose();
  await opening; await closing;
  assert.equal(preview.full, false); assert.equal(t.nativeFull, false);
  assert.equal(t.calls.length, 0);
});


test('closing immediately after button exit preserves final-resize focus recovery', async () => {
  const t = setup(); const preview = t.make(); await preview.toggleFullScreen();
  await preview.exitPreviewFullScreen();
  assert.equal(t.timers.size, 1);
  await preview.dispose();
  assert.equal(t.timers.size, 1, 'no-op exit keeps the preceding recovery alive');
  assert.equal(t.listeners.size, 1);
  const count = t.calls.filter(c => c[0] === 'focus').length;
  await t.resize();
  assert.equal(t.calls.filter(c => c[0] === 'focus').length, count + 1);
  assert.equal(t.config.mediaViewer.isFullScreen, false);
  for (const fn of [...t.timers.values()]) fn();
  assert.equal(t.listeners.size, 0);
});

test('a disposed queued entry also preserves preceding focus recovery', async () => {
  const t = setup(); const first = t.make(); await first.toggleFullScreen();
  await first.dispose();
  const second = t.make();
  const opening = second.toggleFullScreen(); const closing = second.dispose();
  await opening; await closing;
  assert.equal(t.nativeFull, false);
  assert.equal(t.timers.size, 1);
  const count = t.calls.filter(c => c[0] === 'focus').length;
  await t.resize();
  assert.equal(t.calls.filter(c => c[0] === 'focus').length, count + 1);
});
