import test from 'node:test';
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import vm from 'node:vm';
import { createApp, nextTick, watch } from 'vue';
import { createPinia, defineStore } from 'pinia';
import { createPersistedState } from 'pinia-plugin-persistedstate';

const mainSource = await readFile(new URL('../src/main.js', import.meta.url), 'utf8');
const montageSource = await readFile(new URL('../src/views/Montage.vue', import.meta.url), 'utf8');
const listenerStart = mainSource.indexOf("listen('montage-config-changed'");
const listener = mainSource.slice(listenerStart, mainSource.indexOf('\n})', listenerStart) + 3);
const watcherStart = montageSource.indexOf('watch(() => config.montage,');
const watcher = montageSource.slice(watcherStart, montageSource.indexOf('}, { deep: true });', watcherStart) + '}, { deep: true });'.length);

test('main-window writes preserve collage preferences after cross-window sync', async () => {
  const data = new Map();
  const storage = { getItem: key => data.get(key) ?? null, setItem: (key, value) => data.set(key, value) };
  const useConfig = defineStore('configStore', {
    state: () => ({ montage: { mode: 'grid', spacing: 1, format: 0, quality: 0 }, settings: { language: 'en' } }),
    persist: true,
  });
  function windowStore() {
    const pinia = createPinia();
    pinia.use(createPersistedState({ storage }));
    createApp({}).use(pinia);
    return useConfig(pinia);
  }
  const main = windowStore();
  const collage = windowStore();
  const listeners = [];
  for (const [currentWindowLabel, config] of [['main', main], ['montage', collage]]) {
    vm.runInNewContext(listener, { config, currentWindowLabel,
      listen: (_name, callback) => listeners.push(callback),
    });
  }
  let events = 0;
  const stop = vm.runInNewContext(watcher, { config: collage, watch,
    tauriEmit: async (_name, payload) => {
      events++;
      for (const callback of listeners) callback({ payload });
    },
  });
  try {
    Object.assign(collage.montage, { mode: 'pile', spacing: 2.5, format: 2, quality: 1 });
    await nextTick();
    main.settings.language = 'zh';
    await nextTick();
    const reopened = windowStore();
    assert.deepEqual({ ...reopened.montage }, { mode: 'pile', spacing: 2.5, format: 2, quality: 1 });
    assert.equal(events, 1);
    assert.equal(reopened.settings.language, 'zh');
  } finally {
    stop();
  }
});
