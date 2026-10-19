<template>
  <div class="absolute inset-0 flex items-center justify-center px-6" data-tauri-drag-region>
    <select
      :value="config.settings.language" @change="config.setLanguage(($event.target as HTMLSelectElement).value)"
      class="select select-bordered select-sm absolute top-4 right-6 w-auto"
      :aria-label="$t('settings.general.select_language')"
    >
      <option v-for="lang in languages" :key="lang.value" :value="lang.value">{{ lang.label }}</option>
    </select>
    <div class="max-w-3xl w-full text-center">
      <div class="mb-8 flex flex-col items-center gap-3">
        <img :src="iconLogo" class="w-32 h-32 select-none" draggable="false" />
        <div>
          <h2 class="text-xl font-semibold text-base-content/70">
            {{ appName }}
          </h2>
          <p class="mt-2 text-sm text-base-content/30">
            {{ $t('settings.about.package.app_description') }}
          </p>
        </div>
      </div>

      <div class="grid grid-cols-1 md:grid-cols-3 gap-3 text-left">
        <div class="rounded-box border border-base-content/5 bg-base-100/60 p-4">
          <IconFolders class="w-5 h-5 mb-3" />
          <h3 class="text-sm font-medium text-base-content/70">{{ $t('welcome.add_album_title') }}</h3>
          <p class="mt-2 text-xs leading-5 text-base-content/30">{{ $t('welcome.add_album_description') }}</p>
          <button class="mt-4 btn btn-primary btn-sm rounded-box" @click="requestAddAlbum">
            <IconAdd class="w-4 h-4" />
            {{ $t('menu.album.add') }}
          </button>
        </div>
        <div class="rounded-box border border-base-content/5 bg-base-100/60 p-4">
          <IconFolderCog class="w-5 h-5 mb-3" />
          <h3 class="text-sm font-medium text-base-content/70">{{ $t('welcome.smart_album_title') }}</h3>
          <p class="mt-2 text-xs leading-5 text-base-content/30">{{ $t('welcome.smart_album_description') }}</p>
        </div>
        <div class="rounded-box border border-base-content/5 bg-base-100/60 p-4">
          <IconBookmark class="w-5 h-5 mb-3" />
          <h3 class="text-sm font-medium text-base-content/70">{{ $t('welcome.collection_title') }}</h3>
          <p class="mt-2 text-xs leading-5 text-base-content/30">{{ $t('welcome.collection_description') }}</p>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { emit as tauriEmit } from '@tauri-apps/api/event';
import { getPackageInfo } from '@/common/api';
import { IconAdd, IconBookmark, IconFolderCog, IconFolders } from '@/common/icons';
import iconLogo from '@/assets/images/icon.png';
import { useConfigStore } from '@/stores/configStore';

const config = useConfigStore();
const appName = ref('');

onMounted(async () => {
  try {
    const packageInfo = await getPackageInfo();
    appName.value = packageInfo.name;
  } catch (error) {
    console.error('Failed to load app name:', error);
  }
});

const languages = [
  { label: 'English', value: 'en' },
  { label: 'Deutsch', value: 'de' },
  { label: 'Español', value: 'es' },
  { label: 'Français', value: 'fr' },
  { label: 'Italiano', value: 'it' },
  { label: 'Magyar', value: 'hu' },
  { label: 'Nederlands', value: 'nl' },
  { label: 'Polski', value: 'pl' },
  { label: 'Português', value: 'pt' },
  { label: 'Русский', value: 'ru' },
  { label: 'Українська', value: 'uk' },
  { label: '中文简体', value: 'zh' },
  { label: '中文繁體', value: 'zh-TW' },
  { label: '日本語', value: 'ja' },
  { label: '한국어', value: 'ko' },
];

const requestAddAlbum = () => {
  tauriEmit('add-album-requested');
};
</script>
