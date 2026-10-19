<template>
    
  <div class="sidebar-panel">
    <!-- title bar -->
    <div class="sidebar-panel-header" data-tauri-drag-region>
      <span class="sidebar-panel-header-title flex-1">
        {{ titlebar }}<template v-if="searchHistoryList.length > 0"> ({{ searchHistoryList.length.toLocaleString() }})</template>
      </span>
      <ContextMenu :menuItems="searchPanelMenuItems" :iconMenu="IconMore" :smallIcon="true" />
    </div>

    <div class="mx-1 mb-2 px-1 shrink-0">
      <div
        :class="[
          'h-8 flex items-center rounded-box transition-colors bg-base-100/40',
          isSearchFocused ? 'border-2 border-primary' : 'border border-base-content/10 hover:border-base-content/30',
        ]"
        @click="focusSearchInput"
      >
        <IconSearch
          class="ml-2 w-4 h-4 shrink-0"
          :class="isSearchFocused ? 'text-primary/70' : 'text-base-content/30'"
        />
        <input 
          ref="searchInputRef"
          type="text"
          v-model="searchQuery"
          :placeholder="$t('search.image_search_placeholder')"
          class="w-full min-w-0 bg-transparent border-none focus:ring-0 px-2 text-sm placeholder-base-content/30 focus:outline-none"
          maxlength="255"
          @focus="isSearchFocused = true"
          @blur="isSearchFocused = false"
          @keydown.enter = "handleSearch()"
          @keydown.esc = "handleEscKey()"
        >
        <button
          v-if="searchQuery"
          type="button"
          class="mr-1 p-1 rounded-box text-base-content/30 hover:text-base-content/70"
          @click.stop="searchQuery = ''; focusSearchInput()"
        >
          <IconClose class="w-4 h-4" />
        </button>
      </div>

      <!-- FORK: 精确搜索开关 —— 勾选后本次搜索会调用本地大模型对候选逐一判相关度（慢但准） -->
      <label class="mx-1 mt-1 flex items-center gap-1.5 cursor-pointer select-none">
        <input v-model="preciseSearch" type="checkbox" class="checkbox checkbox-xs checkbox-primary" />
        <span class="text-[11px] text-base-content/50">{{ $t('search.precise_search') }}</span>
        <span class="text-[10px] text-base-content/30">{{ $t('search.precise_search_hint') }}</span>
      </label>
    </div>

    <!-- search history -->
    <div class="overflow-y-auto flex-1" >
        <div v-for="(item, index) in searchHistoryList" :key="index"
          :class="[
            'sidebar-item group',
            libConfig.search.searchHistoryIndex === index ? 'sidebar-item-selected' : 'sidebar-item-hover',
          ]"
          @click="handleSearchHistoryClick(index)"
        >
          <IconSearch class="mx-1 h-5 shrink-0" />
          
          <span class="sidebar-item-label">{{ typeof item === 'string' ? item : item.text }}</span>
          <div class="ml-auto flex flex-row items-center text-base-content/30">
            <span
              v-if="hasSearchHistoryCount(item)"
              class="sidebar-item-count shrink-0"
            >{{ formatSearchResultCount(getSearchHistoryCount(item)) }}</span>
            <div
              :class="[
                libConfig.search.searchHistoryIndex === index ? '' : 'hidden group-hover:flex'
              ]"
            >
              <ContextMenu
                :iconMenu="IconMore"
                :menuItems="() => getSearchHistoryMenuItems(index)"
                :smallIcon="true"
              />
            </div>
          </div>
        </div>  
    </div>

  </div>

  <!-- clear history messagebox -->
  <MessageBox
    v-if="showClearHistoryMsgbox"
    :title="$t('msgbox.clear_search_history.title')"
    :message="`${$t('msgbox.clear_search_history.content')}`"
    :OkText="$t('msgbox.clear_search_history.ok')"
    :cancelText="$t('msgbox.cancel')"
    :warningOk="true"
    @ok="clearHistory()"
    @cancel="showClearHistoryMsgbox = false"
  />

</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch, nextTick } from 'vue';
import { useI18n } from 'vue-i18n';
import { config, libConfig } from '@/common/config';
import { useUIStore } from '@/stores/uiStore';
import MessageBox from '@/components/MessageBox.vue';
import { IconMore, IconTrash, IconSearch, IconClose } from '@/common/icons';
import ContextMenu from '@/components/ContextMenu.vue';

const props = defineProps({
  titlebar: {
    type: String,
    required: true
  }
});

/// i18n
const { locale, messages } = useI18n();
const localeMsg = computed(() => messages.value[locale.value] as any);
const uiStore = useUIStore();

const showClearHistoryMsgbox = ref(false);

const searchPanelMenuItems = computed(() => [
  {
    label: localeMsg.value.menu.search.clear_history,
    icon: IconTrash,
    action: () => showClearConfirmation(),
  },
]);

function getSearchHistoryMenuItems(index: number) {
  return [
    {
      label: localeMsg.value.menu.home.delete,
      icon: IconTrash,
      action: () => {
        deleteHistoryItem(index);
      }
    },
  ];
}

// search query
const searchInputRef = ref<HTMLInputElement | null>(null);
const searchQuery = ref('');
const isSearchFocused = ref(false);

// FORK: 精确搜索开关。存到 libConfig 而非本地 ref——Content.vue 发起搜索时
// 要读这个值决定是否走后端大模型重排（rerank）。
const preciseSearch = computed({
  get: () => !!(libConfig.search as any).preciseSearch,
  set: (v: boolean) => { (libConfig.search as any).preciseSearch = v; },
});

function syncSearchState() {
  if (libConfig.search.searchHistoryIndex !== -1) {
    const history = libConfig.search.searchHistory as any[];
    const item = history[libConfig.search.searchHistoryIndex];
    if (item) {
      const text = typeof item === 'string' ? item : item.text;
      libConfig.search.searchText = text;
      searchQuery.value = text;
    }
  } else {
    searchQuery.value = libConfig.search.searchText || '';
  }
}

watch(
  () => libConfig.search.searchHistoryIndex,
  () => {
    syncSearchState();
  }
);

function focusSearchInput() {
  searchInputRef.value?.focus();
}

onMounted(() => {
  syncSearchState();
  nextTick(() => focusSearchInput());
});

onUnmounted(() => {
  uiStore.removeInputHandler('ImageSearch');
});

function handleSearchHistoryClick(index: number) {
  const item = libConfig.search.searchHistory[index];
  const searchText = typeof item === 'string' ? item : item?.text;
  if (searchText) {
    libConfig.search.searchText = searchText;
    uiStore.requestCountUpdate({ source: 'search', text: searchText });
  }
  libConfig.search.searchHistoryIndex = index;
}

function clearHistory() {
  libConfig.search.searchText = '';
  libConfig.search.searchHistory = [];
  libConfig.search.searchHistoryIndex = -1;

  showClearHistoryMsgbox.value = false;
}

function deleteHistoryItem(index: number) {
  if (index < 0 || index >= libConfig.search.searchHistory.length) return;
  const selectedIndex = libConfig.search.searchHistoryIndex;
  libConfig.search.searchHistory.splice(index, 1);
  if (selectedIndex === index) {
    libConfig.search.searchText = '';
    searchQuery.value = '';
    libConfig.search.searchHistoryIndex = -1;
  } else if (selectedIndex > index) {
    libConfig.search.searchHistoryIndex = selectedIndex - 1;
  }
}

function handleSearch() {
  if (searchQuery.value.trim().length === 0) return;
  
  const query = searchQuery.value.trim();
  const history = libConfig.search.searchHistory as any[];
  
  // Find existing index considering both string and object formats
  const existingIndex = history.findIndex((item: any) => {
    const text = typeof item === 'string' ? item : item.text;
    return text === query;
  });
  
  if (existingIndex !== -1) {
    libConfig.search.searchHistoryIndex = existingIndex;
  } else {
    // Add new item as object
    history.unshift({ text: query, count: null });
    libConfig.search.searchHistoryIndex = 0;

    // Limit the history size
    if (history.length > config.search.maxSearchHistory) {
      history.pop();
    }
  }

  libConfig.search.searchText = query;
  uiStore.requestCountUpdate({ source: 'search', text: query });
}

function handleEscKey() {
  searchInputRef.value?.blur();
}

const searchHistoryList = computed(() => libConfig.search.searchHistory as any[]);
const getSearchHistoryCount = (item: any) => {
  return Number(typeof item === 'string' ? 0 : item?.count || 0);
};
const hasSearchHistoryCount = (item: any) => getSearchHistoryCount(item) > 0;
const formatSearchResultCount = (count: number) => {
  return count.toLocaleString();
};

function showClearConfirmation() {
  showClearHistoryMsgbox.value = true;
}

defineExpose({
  clearHistory,
  showClearConfirmation,
  focusSearchInput,
});

</script>
