import test from 'node:test';
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import vm from 'node:vm';
import ts from 'typescript';

const apiSource = await readFile(new URL('../src/common/api.js', import.meta.url), 'utf8');
const start = apiSource.indexOf('export async function getSystemRegionLocale');
const source = `import { invoke } from '@tauri-apps/api/core';\nimport { locale } from '@tauri-apps/plugin-os';\n${apiSource.slice(start, apiSource.indexOf('// library', start))}`;

async function fixture(region, failure = false) {
  const calls = [];
  const context = vm.createContext({ console: { warn() {} } });
  const module = new vm.SourceTextModule(source, { context });
  await module.link(specifier => {
    const exports = specifier.endsWith('/core') ? {
      invoke: async command => {
        calls.push(command);
        if (failure) throw new Error('OS read failed');
        return region;
      },
    } : { locale: async () => { calls.push('plugin-os'); return 'en-US'; } };
    return new vm.SyntheticModule(Object.keys(exports), function () {
      for (const [key, value] of Object.entries(exports)) this.setExport(key, value);
    }, { context });
  });
  await module.evaluate();
  return { calls, getLocale: module.namespace.getSystemRegionLocale };
}

test('native regional locale takes priority over display language on all platforms', async () => {
  const { calls, getLocale } = await fixture('fr-CH');
  assert.equal(await getLocale(), 'fr-CH');
  assert.deepEqual(calls, ['get_system_region_locale']);
});

test('missing regional locale and native errors fall back to plugin-os', async () => {
  for (const failure of [false, true]) {
    const { getLocale } = await fixture(null, failure);
    assert.equal(await getLocale(), 'en-US');
  }
});

test('dates use Swiss French region; app-language override and formatter cache still work', async () => {
  const utils = await readFile(new URL('../src/common/utils.ts', import.meta.url), 'utf8');
  const start = utils.indexOf('export type DateTimeStyle');
  const end = utils.indexOf('/// format timestamp', start);
  const code = ts.transpile(utils.slice(start, end).replaceAll('export ', ''), {
    target: ts.ScriptTarget.ES2022,
  });
  const config = { systemLocale: 'fr-CH', settings: { language: 'en-US', dateTimeFormat: 0 } };
  const format = new Function('config', 'navigator', code + '; return formatDateTimeWithStyle;')(
    config, { language: 'en-US' },
  );
  const date = new Date(2026, 9, 14, 15, 4, 5);
  assert.equal(format(date, 'date_time'), new Intl.DateTimeFormat('fr-CH', {
    dateStyle: 'short', timeStyle: 'medium',
  }).format(date));
  assert.match(format(date, 'date_time'), /15:04:05/);
  config.settings.dateTimeFormat = 1;
  assert.equal(format(date, 'date_time'), new Intl.DateTimeFormat('en-US', {
    dateStyle: 'short', timeStyle: 'medium',
  }).format(date));
  config.settings.dateTimeFormat = 0;
  assert.match(format(date, 'date_time'), /15:04:05/);
});
