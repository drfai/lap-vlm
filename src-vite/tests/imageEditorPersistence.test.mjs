import test from 'node:test';
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import vm from 'node:vm';
import { createApp, nextTick, watch } from 'vue';
import { createPinia, defineStore } from 'pinia';
import { createPersistedState } from 'pinia-plugin-persistedstate';

const mainSource = await readFile(new URL('../src/main.js', import.meta.url), 'utf8');
const editorSource = await readFile(new URL('../src/views/ImageEditor.vue', import.meta.url), 'utf8');
const listenerStart = mainSource.indexOf("listen('image-editor-config-changed'");
const listener = mainSource.slice(listenerStart, mainSource.indexOf('\n})', listenerStart) + 3);
const watcherStart = editorSource.indexOf('watch(() => config.imageEditor,');
const watcher = editorSource.slice(watcherStart, editorSource.indexOf('}, { deep: true });', watcherStart) + '}, { deep: true });'.length);

test('main-window writes preserve editor preferences after cross-window sync', async () => {
  const data = new Map();
  const storage = { getItem: key => data.get(key) ?? null, setItem: (key, value) => data.set(key, value) };
  const useConfig = defineStore('configStore', {
    state: () => ({ imageEditor: { tab: 'edit', cropShape: 0, saveAs: 0, format: 0, quality: 0, custom: { brightness: 0 } }, settings: { language: 'en' } }),
    persist: true,
  });
  function windowStore() {
    const pinia = createPinia();
    pinia.use(createPersistedState({ storage }));
    createApp({}).use(pinia);
    return useConfig(pinia);
  }
  const main = windowStore();
  const editor = windowStore();
  const listeners = [];
  for (const [currentWindowLabel, config] of [['main', main], ['imageeditor', editor]]) {
    vm.runInNewContext(listener, { config, currentWindowLabel,
      listen: (_name, callback) => listeners.push(callback),
    });
  }
  let events = 0;
  const stop = vm.runInNewContext(watcher, { config: editor, watch,
    tauriEmit: async (_name, payload) => {
      events++;
      for (const callback of listeners) callback({ payload });
    },
  });
  try {
    Object.assign(editor.imageEditor, { tab: 'adjust', cropShape: 3, saveAs: 1, format: 2, quality: 1, custom: { brightness: 15 } });
    await nextTick();
    main.settings.language = 'zh';
    await nextTick();
    const reopened = windowStore();
    assert.deepEqual({ ...reopened.imageEditor }, { tab: 'adjust', cropShape: 3, saveAs: 1, format: 2, quality: 1, custom: { brightness: 15 } });
    assert.equal(events, 1);
    editor.imageEditor.custom.brightness = 25;
    await nextTick();
    main.settings.language = 'ja';
    await nextTick();
    assert.equal(windowStore().imageEditor.custom.brightness, 25);
    assert.equal(events, 2);
    assert.equal(reopened.settings.language, 'zh');
  } finally {
    stop();
  }
});
