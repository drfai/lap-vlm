import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import ts from 'typescript';
import { computed, ref } from 'vue';

const source = fs.readFileSync(new URL('../src/common/rawDisplay.ts', import.meta.url), 'utf8');
const js = ts.transpile(source.replace(/^import .*;$/gm, '').replace(/export /g, ''), { target: ts.ScriptTarget.ES2022 });
const settings = { rawPreviewSource: 'embedded', rawRenderBrightness: 'original', groupRawJpegPairs: false, rawPairDisplaySource: 'jpeg' };
const { getRawDisplayOptions, rawDisplayKey, appendRawDisplayParams, nextRawPreviewMode } = new Function('useConfigStore', js + '; return { getRawDisplayOptions, rawDisplayKey, appendRawDisplayParams, nextRawPreviewMode };')(() => ({ settings }));

test('source and brightness generate four distinct cache keys and request policies', () => {
  const keys = new Set();
  for (const source of ['embedded', 'rendered']) {
    for (const brightness of ['original', 'brightened']) {
      settings.rawPreviewSource = source;
      settings.rawRenderBrightness = brightness;
      keys.add(rawDisplayKey());
      const params = new URLSearchParams();
      appendRawDisplayParams(params);
      assert.equal(params.get('rawPreviewMode'), source);
      assert.equal(params.get('rawAutoBright'), String(brightness === 'brightened'));
    }
  }
  assert.equal(keys.size, 4);
});

test('local RAW override removes paired JPEG preference without changing settings', () => {
  settings.groupRawJpegPairs = true;
  const before = { ...settings };
  const params = new URLSearchParams('rawPairDisplay=jpeg');
  appendRawDisplayParams(params, { mode: 'rendered', autoBright: false, preferPair: false });
  assert.equal(params.get('rawPairDisplay'), 'raw');
  assert.deepEqual(settings, before);
  assert.equal(getRawDisplayOptions().preferPair, true);
});

test('direct cycle skips unavailable embedded preview and returns from paired JPEG using defaults', () => {
  const defaults = { mode: 'embedded', autoBright: false, preferPair: true };
  assert.equal(nextRawPreviewMode('embedded', false, defaults), 'rendered');
  assert.equal(nextRawPreviewMode('rendered', false, defaults), 'brightened');
  assert.equal(nextRawPreviewMode('brightened', false, defaults), 'embedded');
  assert.equal(nextRawPreviewMode('rendered', true, defaults), 'brightened');
  assert.equal(nextRawPreviewMode('brightened', true, defaults), 'rendered');
  assert.equal(nextRawPreviewMode('pair', false, defaults), 'embedded');
  assert.equal(nextRawPreviewMode('pair', true, { ...defaults, autoBright: true }), 'brightened');
});

const imageSource = fs.readFileSync(new URL('../src/components/Image.vue', import.meta.url), 'utf8');
const selectionSource = ts.transpile(imageSource.slice(imageSource.indexOf('const rawModeSource ='), imageSource.indexOf('let rawAbortController')), { target: ts.ScriptTarget.ES2022 });

test('JPEG round trips restore the selected RAW mode before cycling again', () => {
  const defaults = { mode: 'embedded', autoBright: false, preferPair: true };
  const rawOverride = ref(null);
  const rawSource = ref('embedded');
  const rawRequestPending = ref(false);
  const requestedRawOptions = computed(() => rawOverride.value || defaults);
  const { switchRaw, selectRawPair, rawSourceLabel } = new Function('computed', 'rawOverride', 'rawSource', 'rawRequestPending', 'rawSelectionVersion', 'rawEmbeddedUnavailable', 'requestedRawOptions', 'rawPairLabel', 'nextRawPreviewMode', 'getRawDisplayOptions', 't', selectionSource + '; return { switchRaw, selectRawPair, rawSourceLabel };')(
    computed, rawOverride, rawSource, rawRequestPending, ref(0), ref(false), requestedRawOptions, ref('JPEG'), nextRawPreviewMode, () => defaults, key => key,
  );
  const finish = source => { rawSource.value = source; rawRequestPending.value = false; };
  // JPEG is highlighted by preferPair even if the last confirmed source
  // is embedded. The first click must select that badge mode, not cycle it.
  switchRaw();
  assert.equal(rawOverride.value.mode, 'embedded');
  finish('embedded');
  for (const mode of ['rendered', 'brightened', 'embedded']) {
    switchRaw();
    finish(mode);
    selectRawPair();
    // Returning before or after the JPEG response must preserve the RAW mode.
    switchRaw();
    assert.equal(rawOverride.value.mode, mode === 'embedded' ? 'embedded' : 'rendered');
    assert.equal(rawOverride.value.autoBright, mode === 'brightened');
    finish(mode);
    selectRawPair();
    finish('pair');
    switchRaw();
    assert.equal(rawOverride.value.mode, mode === 'embedded' ? 'embedded' : 'rendered');
    assert.equal(rawOverride.value.autoBright, mode === 'brightened');
    assert.equal(rawOverride.value.preferPair, false);
    finish(mode);
  }
  // A generic image fallback must not claim to be an embedded RAW preview.
  finish('image');
  assert.equal(rawSourceLabel.value, '');
  switchRaw();
  assert.equal(rawOverride.value.mode, 'rendered');
});

const loaderSource = ts.transpile(imageSource.slice(imageSource.indexOf('async function loadRawImage'), imageSource.indexOf('watch(() => [scale.value')), { target: ts.ScriptTarget.ES2022 });

test('a superseded RAW response cannot replace the latest source or leak a blob URL', async () => {
  const requests = [];
  const urls = new Set();
  const load = new Function('fetch', 'props', 'getPreviewUrl', 'appendRawDisplayParams', 'requestedRawOptions', 'Image', 'rawObjectUrls', `let rawAbortController = null; const rawViewerId = 'test-viewer'; ${loaderSource}; return loadRawImage;`)(
    (_url, options) => new Promise(resolve => requests.push({ resolve, signal: options.signal })),
    { fileId: 42, fileVersion: 1 }, () => 'http://preview.localhost/test/42', appendRawDisplayParams,
    { value: { mode: 'rendered', autoBright: true, preferPair: false } },
    class { naturalWidth = 100; naturalHeight = 100; async decode() {} }, urls,
  );
  const older = load('photo.dng');
  const rejected = assert.rejects(older, /cancelled/);
  const newer = load('photo.dng');
  assert.equal(requests[0].signal.aborted, true);
  const response = () => ({ ok: true, blob: async () => new Blob(['image']), headers: new Headers({ 'X-Raw-Source': 'brightened', 'X-Raw-Embedded-Unavailable': 'true', 'X-Raw-Pair': 'JPEG' }) });
  requests[1].resolve(response());
  const latest = await newer;
  requests[0].resolve(response());
  await rejected;
  // preferPair is false: a response header must not expose a paired JPEG.
  assert.deepEqual(latest.raw, { source: 'brightened', unavailable: true, pair: null });
  assert.equal(urls.size, 1);
  URL.revokeObjectURL(latest.src);
});

const utilsSource = fs.readFileSync(new URL('../src/common/utils.ts', import.meta.url), 'utf8');
const previewUrlSource = ts.transpile(utilsSource.slice(utilsSource.indexOf('export function getPreviewUrl('), utilsSource.indexOf('export function shouldUseBackendPreview(')).replace('export ', ''), { target: ts.ScriptTarget.ES2022 });

test('editor URL selects the export pipeline regardless of RAW browsing settings', () => {
  const getPreviewUrl = new Function('isWin', '_thumbLibraryId', 'useUIStore', 'appendRawDisplayParams', previewUrlSource + '; return getPreviewUrl;')(
    false, 'test-library', () => ({ getFileVersion: () => 7 }), appendRawDisplayParams,
  );
  const urls = new Set();
  for (const source of ['embedded', 'rendered']) {
    for (const brightness of ['original', 'brightened']) {
      settings.rawPreviewSource = source;
      settings.rawRenderBrightness = brightness;
      settings.groupRawJpegPairs = true;
      const url = getPreviewUrl(42, 'photo.dng', false, 3, true);
      urls.add(url);
      const params = new URL(url).searchParams;
      assert.equal(params.get('forEditing'), 'true');
      assert.equal(params.has('rawPreviewMode'), false);
      assert.equal(params.has('rawAutoBright'), false);
      assert.equal(params.has('rawPairDisplay'), false);
      assert.equal(params.get('v'), '3');
      assert.equal(params.get('u'), '7');
    }
  }
  assert.equal(urls.size, 1);
  const browsing = new URL(getPreviewUrl(42, 'photo.dng')).searchParams;
  assert.equal(browsing.has('forEditing'), false);
  assert.equal(browsing.get('rawPairDisplay'), 'jpeg');
});
