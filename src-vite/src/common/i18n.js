import { createI18n } from 'vue-i18n';

const loaders = import.meta.glob('../locales/*.json');
const pending = new Map();
const fileLocale = language => language === 'zh' ? 'zh-CN' : language;
export const i18n = createI18n({ legacy: false, locale: 'en', fallbackLocale: 'en', messages: {} });

export async function loadLanguage(language) {
  const loader = loaders[`../locales/${fileLocale(language)}.json`];
  if (!loader) throw new Error(`Unsupported language: ${language}`);
  if (i18n.global.availableLocales.includes(language)) return;
  if (!pending.has(language)) {
    pending.set(language, loader().then(module => {
      i18n.global.setLocaleMessage(language, module.default);
    }).finally(() => pending.delete(language)));
  }
  await pending.get(language);
}

export async function initializeLanguage(language) {
  await loadLanguage('en');
  try {
    await loadLanguage(language);
    i18n.global.locale.value = language;
    return language;
  } catch (error) {
    console.error('Failed to load language:', error);
    i18n.global.locale.value = 'en';
    return 'en';
  }
}
