<template>
  <ModalDialog :title="$t('tag.edit_tag') + (allTags.length ? ` (${allTags.length.toLocaleString()})` : '')" :width="500" :position-key="positionKey" @cancel="clickCancel">
    <template #title-actions>
      <button type="button" class="t-button-default btn-outline gap-1.5" :disabled="!canEditTags || isCreatingTag || newTagGroupId !== null || renamingId !== null || isCreatingGroup" @click="showNewGroup = true">
        <IconAdd class="w-4 h-4" aria-hidden="true" />
        {{ $t('menu.tag.new_group') }}
      </button>
    </template>
    <section class="space-y-3">
      <div class="flex items-center gap-2">
        <div
          :class="[
            'grow h-8 flex items-center rounded-box overflow-hidden transition-colors bg-base-100',
            isSearchFocused
              ? 'border-2 border-primary'
              : 'border border-neutral-content/30 hover:border-neutral-content/70',
          ]"
        >
          <IconSearch class="ml-2 w-4 h-4 text-base-content/70" />
          <input
            ref="tagSearchInputRef"
            type="text"
            v-model="tagSearch"
            :placeholder="$t('menu.tag.search')"
            class="w-full bg-transparent border-none focus:ring-0 px-2 text-sm placeholder-base-content/30 focus:outline-none"
            @focus="onSearchFocus"
            @blur="onSearchBlur"
          />
          <button
            v-if="tagSearch"
            type="button"
            class="mr-1 p-1 rounded-box text-base-content/30 hover:text-base-content/70"
            @click="
              tagSearch = '';
              tagSearchInputRef?.focus();
            "
          >
            <IconClose class="w-4 h-4" />
          </button>
        </div>
      </div>
      <div
        class="min-h-48 max-h-[50vh] overflow-y-auto rounded-box p-2 bg-base-100/30 border border-base-content/10 flex"
        :class="visibleGroups.length === 0 ? 'items-center justify-center' : ''"
      >
        <div v-if="visibleGroups.length > 0" class="w-full">
          <template v-for="group in visibleGroups" :key="group.id">
            <div :id="`dialog-tag-group-${group.id}`" class="group/header flex items-center pr-1 rounded-box hover:bg-base-content/5">
              <button
                type="button"
                class="sidebar-item min-w-0 flex-1 select-none"
                :aria-expanded="expanded(group.id)"
                @keydown.enter.stop.prevent="toggleGroup(group.id)"
                @keydown.space.stop.prevent="toggleGroup(group.id)"
              >
                <IconRight
                  class="p-1 w-6 h-6 shrink-0 transition-transform"
                  :class="expanded(group.id) ? 'rotate-90' : ''"
                  @click="toggleGroup(group.id)"
                />
                <span class="sidebar-item-label text-left font-medium">{{ group.name }}</span>
              </button>
              <div class="relative grid min-w-8 h-8 shrink-0 place-items-center">
                <span class="sidebar-item-count px-1 tabular-nums group-hover/header:invisible group-focus-within/header:invisible">{{ group.tags.length.toLocaleString() }}</span>
                <button
                  type="button"
                  class="absolute inset-0 flex items-center justify-center rounded-box text-base-content/40 hover:text-primary hover:bg-primary/10 opacity-0 group-hover/header:opacity-100 group-focus-within/header:opacity-100 focus:opacity-100 transition-opacity disabled:cursor-not-allowed"
                  :title="$t('msgbox.new_tag.title')"
                  :aria-label="$t('msgbox.new_tag.title')"
                  :disabled="!canEditTags || isCreatingTag"
                  @click.stop="startNewTag(group.id)"
                  @dblclick.stop
                >
                  <IconTagAdd class="w-4 h-4 cursor-pointer" />
                </button>
              </div>
            </div>
            <template v-if="expanded(group.id)">
              <div v-if="newTagGroupId === group.id" class="pl-14 py-1">
                <div class="flex items-center gap-2 text-primary">
                  <IconTag class="w-4 h-4 shrink-0" />
                  <input
                    :ref="(el) => { newTagNameInputRef = el as HTMLInputElement | null; }"
                    v-model="newTagName"
                    type="text"
                    maxlength="255"
                    class="input input-sm px-2 min-w-0 flex-1 text-sm"
                    :aria-label="$t('msgbox.new_tag.title')"
                    :readonly="isCreatingTag"
                    @click.stop
                    @keydown.stop
                    @keydown.enter.prevent="!$event.isComposing && addNewTag()"
                    @keydown.escape.prevent="cancelNewTag"
                    @blur="onNewTagBlur"
                    @input="newTagNameEdited = true"
                  />
                </div>
                <p v-if="duplicateTag" class="mt-2 text-xs text-base-content/60" role="status">
                  {{ $t("menu.tag.exists_in", { group: duplicateTag.group_name }) }}
                  <button type="button" class="text-primary hover:underline" @click="locateDuplicate">
                    {{ $t("menu.tag.locate") }}
                  </button>
                </p>
              </div>
              <div
                v-for="tag in group.tags"
                :id="`dialog-tag-${tag.id}`"
                :key="tag.id"
                :class="[
                  'group w-full pl-8 p-2 flex items-center gap-2 rounded-box text-left cursor-pointer transition-colors',
                  {
                    'text-primary': selectedTags.has(tag.id),
                    'bg-base-content/5':
                      intermediateTags.has(tag.id) && !selectedTags.has(tag.id),
                    'hover:bg-base-content/5':
                      !selectedTags.has(tag.id) &&
                      !intermediateTags.has(tag.id),
                    'ring-2 ring-primary ring-offset-1 ring-offset-base-100':
                      filteredTags[focusedTagIndex]?.id === tag.id,
                  },
                ]"
                @click="toggleTag(tag.id)"
              >
                <label
                  class="flex items-center cursor-pointer shrink-0"
                  @click.stop
                  @dblclick.stop
                >
                  <input
                    type="checkbox"
                    class="checkbox checkbox-xs"
                    :class="
                      selectedTags.has(tag.id)
                        ? 'checkbox-primary opacity-70'
                        : ''
                    "
                    :disabled="!canEditTags"
                    :checked="selectedTags.has(tag.id)"
                    :indeterminate="intermediateTags.has(tag.id)"
                    @change="toggleTag(tag.id)"
                  />
                </label>
                <IconTag class="w-4 h-4 shrink-0" />
                <input
                  v-if="renamingId === Number(tag.id)"
                  ref="renameInput"
                  v-model="renameValue"
                  class="input px-1 min-w-0 flex-1 text-sm"
                  maxlength="255"
                  @click.stop
                  @keydown.stop
                  @keydown.enter.prevent="!$event.isComposing && commitRename(tag)"
                  @keydown.escape.prevent="cancelRename"
                  @blur="commitRename(tag)"
                />
                <span v-else class="min-w-0 flex-1 truncate">{{
                  tag.name
                }}</span>
                <span
                  v-if="
                    renamingId !== Number(tag.id) && Number(tag.count || 0) > 0
                  "
                  class="sidebar-item-count shrink-0 group-hover:hidden"
                  >{{ Number(tag.count || 0).toLocaleString() }}</span
                >
                <div
                  v-if="renamingId !== Number(tag.id)"
                  class="shrink-0 hidden group-hover:flex"
                >
                  <button
                    type="button"
                    class="p-1 text-base-content/40 hover:text-base-content cursor-pointer"
                    :title="$t('menu.tag.rename')"
                    @click.stop="startRename(tag)"
                  >
                    <IconRename class="w-4 h-4" />
                  </button>
                  <button
                    type="button"
                    class="p-1 text-base-content/40 hover:text-error cursor-pointer"
                    :title="$t('menu.tag.delete')"
                    @click.stop="deleteTarget = tag"
                  >
                    <IconTrash class="w-4 h-4" />
                  </button>
                </div>
              </div>
              <div
                v-if="!group.tags.length && newTagGroupId !== group.id"
                class="pl-8 py-2 text-xs text-base-content/40"
              >
                {{ $t("menu.tag.empty") }}
              </div>
            </template>
          </template>
        </div>
        <span v-else class="text-base-content/30">{{
          $t("tag.not_found")
        }}</span>
      </div>
      <div v-if="tagLoadFailed" class="text-sm text-error">
        {{ $t("tag.load_failed") }}
      </div>
    </section>

    <!-- cancel and OK buttons -->
    <div class="mt-4 flex justify-end space-x-4">
      <button class="t-button-default" @click="clickCancel">
        {{ $t("msgbox.cancel") }}
      </button>

      <button
        class="t-button-primary"
        :disabled="
          isLoadingTags || isLoadingCatalog || isApplyingTags || tagLoadFailed || isCreatingTag || newTagGroupId !== null
        "
        @click="clickOk"
      >
        {{ $t("msgbox.ok") }}
      </button>
    </div>
  </ModalDialog>
  <MessageBox
    v-if="showNewGroup"
    :title="$t('menu.tag.new_group')"
    :showInput="true"
    :needValidateInput="true"
    :inputPlaceholder="$t('menu.tag.name')"
    :isLoading="isCreatingGroup"
    :errorMessage="groupError"
    :OkText="$t('msgbox.ok')"
    :cancelText="$t('msgbox.cancel')"
    @ok="addNewGroup"
    @cancel="showNewGroup = false; groupError = ''"
  />
  <MessageBox
    v-if="deleteTarget"
    :title="$t('msgbox.delete_tag.title')"
    :message="$t('msgbox.delete_tag.content', { tag: deleteTarget.name })"
    :OkText="$t('msgbox.delete_tag.ok')"
    :cancelText="$t('msgbox.cancel')"
    :warningOk="true"
    @ok="confirmDelete"
    @cancel="deleteTarget = null"
  />
</template>

<script setup lang="ts">
import {
  ref,
  computed,
  watch,
  nextTick,
  onMounted,
  onBeforeUnmount,
} from "vue";
import { emit as tauriEmit, listen } from "@tauri-apps/api/event";
import { useI18n } from "vue-i18n";
import { useToast } from "@/common/toast";
import {
  getAllTags,
  getTagGroups,
  saveTagGroup,
  createTag,
  getTagSelectionCounts,
  applyTagsToFiles,
  deleteTag,
  renameTag,
} from "@/common/api";
import {
  IconAdd,
  IconTagAdd,
  IconClose,
  IconRename,
  IconSearch,
  IconTag,
  IconTrash,
  IconRight,
} from "@/common/icons";
import { groupTags, type TagGroup } from "@/common/tagGroups";
import MessageBox from "./MessageBox.vue";
import { libConfig } from "@/common/config";
import { useUIStore } from "@/stores/uiStore";
import ModalDialog from "@/components/ModalDialog.vue";

const props = defineProps({
  fileIds: {
    type: Array as () => number[],
    default: () => [],
  },
  // Optional key for persisting dialog position across sessions.
  positionKey: {
    type: String,
    default: '',
  },
});

const emit = defineEmits(["ok", "cancel", "states-changed"]);
const uiStore = useUIStore();
const toast = useToast();
const { t } = useI18n();

const allTags = ref<any[]>([]);
const showNewGroup = ref(false);
const isCreatingGroup = ref(false);
const groupError = ref('');
const tagsChangeSource = crypto.randomUUID();

async function addNewGroup(value: string) {
  const name = value.trim();
  if (!name || isCreatingGroup.value) return;
  if (groups.value.some(group => group.name.toLocaleLowerCase() === name.toLocaleLowerCase())) {
    groupError.value = t('menu.tag.name_exists');
    return;
  }
  isCreatingGroup.value = true;
  groupError.value = '';
  try {
    const id = await saveTagGroup(null, name);
    if (id == null) throw new Error('save failed');
    tagSearch.value = '';
    collapsed.value = collapsed.value.filter(groupId => groupId !== id);
    await loadAllTags();
    await tauriEmit('tags-changed', { source: tagsChangeSource });
    showNewGroup.value = false;
    await nextTick();
    document.getElementById(`dialog-tag-group-${id}`)?.scrollIntoView({ block: 'nearest' });
    tagSearchInputRef.value?.focus({ preventScroll: true });
  } catch {
    groupError.value = t('tag.name_save_failed');
  } finally {
    isCreatingGroup.value = false;
  }
}
const tagSearchInputRef = ref<HTMLInputElement | null>(null);
const newTagNameInputRef = ref<HTMLInputElement | null>(null);
const tagSearch = ref("");
const newTagName = ref("");
const newTagNameEdited = ref(false);
const isSearchFocused = ref(false);
const isCreatingTag = ref(false);
const focusedTagIndex = ref(-1); // -1 = no tag focused
const isInTagList = ref(false); // true = keyboard focus is in tag list
const isLoadingTags = ref(true);
const isLoadingCatalog = ref(true);
const isApplyingTags = ref(false);
const catalogLoadFailed = ref(false);
const selectionLoadFailed = ref(false);
const tagLoadFailed = computed(() => catalogLoadFailed.value || selectionLoadFailed.value);
const canEditTags = computed(() =>
  !isLoadingCatalog.value && !isLoadingTags.value && !isApplyingTags.value && !tagLoadFailed.value,
);
const renamingId = ref<number | null>(null);
const renameValue = ref("");
const renameInput = ref<HTMLInputElement | HTMLInputElement[] | null>(null);
const deleteTarget = ref<any | null>(null);

// Sets to track tag states
const selectedTags = ref<Set<number>>(new Set()); // Tags present on ALL selected files
const intermediateTags = ref<Set<number>>(new Set()); // Tags present on SOME selected files
const initialSelectedTags = ref<Set<number>>(new Set());
const initialIntermediateTags = ref<Set<number>>(new Set());
const tagChanges = ref<Map<number, "add" | "remove">>(new Map());

const groups = ref<TagGroup[]>([]);
const newTagGroupId = ref<number | null>(null);
const collapsed = ref<number[]>([]);
const duplicateTag = ref<any>(null);
const visibleGroups = computed(() =>
  groupTags(groups.value, allTags.value, newTagGroupId.value !== null ? "" : tagSearch.value),
);
const expanded = (id: number) =>
  newTagGroupId.value === id || !!tagSearch.value.trim() || !collapsed.value.includes(id);
function toggleGroup(id: number) {
  if (tagSearch.value.trim() || newTagGroupId.value === id) return;
  collapsed.value = expanded(id)
    ? [...collapsed.value, id]
    : collapsed.value.filter((value) => value !== id);
}
const filteredTags = computed(() =>
  visibleGroups.value.flatMap((group) =>
    expanded(group.id) ? group.tags : [],
  ),
);
async function locateDuplicate() {
  const tag = duplicateTag.value;
  if (!tag) return;
  cancelNewTag();
  tagSearch.value = "";
  collapsed.value = collapsed.value.filter((id) => id !== tag.group_id);
  await nextTick();
  document
    .getElementById(`dialog-tag-${tag.id}`)
    ?.scrollIntoView({ block: "nearest" });
  focusedTagIndex.value = filteredTags.value.findIndex(
    (item) => item.id === tag.id,
  );
}
let unlisten: (() => void) | undefined;
let disposed = false;
let loadRequest = 0;

onMounted(async () => {
  window.addEventListener("keydown", handleKeyDown);
  uiStore.pushInputHandler("TaggingDialog");

  await nextTick();
  tagSearchInputRef.value?.focus();

  const stop = await listen<{ source?: string }>("tags-changed", async (event) => {
    if (event.payload?.source === tagsChangeSource) return;
    await loadAllTags();
    if (!disposed) await loadExistingTagsForFiles(true);
  });
  if (disposed) {
    stop();
    return;
  }
  unlisten = stop;
  loadAllTags();
  loadExistingTagsForFiles();
});

onBeforeUnmount(() => {
  disposed = true;
  loadRequest++;
  unlisten?.();
  window.removeEventListener("keydown", handleKeyDown);
  uiStore.removeInputHandler("TaggingDialog");
});

// load all tags
async function loadAllTags() {
  const request = ++loadRequest;
  isLoadingCatalog.value = true;
  const library = libConfig._libraryId;
  try {
    const [tags, allGroups] = await Promise.all([
      getAllTags(0),
      getTagGroups(),
    ]);
    if (disposed || request !== loadRequest || library !== libConfig._libraryId)
      return;
    if (!tags) throw new Error("load failed");
    catalogLoadFailed.value = false;
    groups.value = allGroups;
    if (newTagGroupId.value !== null && !groups.value.some((group) => group.id === newTagGroupId.value))
      cancelNewTag();
    const sidebarCounts = libConfig.tag.counts || {};
    allTags.value = tags.map((tag: any) => ({
      ...tag,
      count: Number(sidebarCounts[String(tag.id)] || 0),
    }));
    const ids = new Set(tags.map((tag: any) => tag.id));
    for (const id of tagChanges.value.keys())
      if (!ids.has(id)) tagChanges.value.delete(id);
  } catch {
    if (!disposed && request === loadRequest && library === libConfig._libraryId)
      catalogLoadFailed.value = true;
  } finally {
    if (!disposed && request === loadRequest) isLoadingCatalog.value = false;
  }
}

let selectionRequest = 0;
async function loadExistingTagsForFiles(preserveChanges = false) {
  const request = ++selectionRequest;
  const library = libConfig._libraryId;
  isLoadingTags.value = true;
  try {
    const counts = props.fileIds.length
      ? await getTagSelectionCounts(props.fileIds)
      : [];
    if (
      disposed ||
      request !== selectionRequest ||
      library !== libConfig._libraryId
    )
      return;
    if (counts === null) {
      selectionLoadFailed.value = true;
      return;
    }
    selectionLoadFailed.value = false;
    const selected = new Set<number>();
    const intermediate = new Set<number>();
    for (const entry of counts) {
      const tagId = Number(entry.tag_id);
      if (Number(entry.count) === props.fileIds.length) selected.add(tagId);
      else if (Number(entry.count) > 0) intermediate.add(tagId);
    }
    initialSelectedTags.value = new Set(selected);
    initialIntermediateTags.value = new Set(intermediate);
    selectedTags.value = selected;
    intermediateTags.value = intermediate;
    if (!preserveChanges) tagChanges.value.clear();
    for (const [id, change] of tagChanges.value)
      setTagVisualState(id, change === "add" ? "selected" : "none");
  } finally {
    if (!disposed && request === selectionRequest) isLoadingTags.value = false;
  }
}

async function startNewTag(groupId: number) {
  if (!canEditTags.value || isCreatingTag.value) return;
  if (newTagGroupId.value !== null) {
    newTagNameInputRef.value?.focus();
    return;
  }
  cancelRename();
  tagSearch.value = "";
  collapsed.value = collapsed.value.filter((id) => id !== groupId);
  const names = new Set(allTags.value.map((tag) => tag.name.toLocaleLowerCase()));
  const baseName = t("msgbox.new_tag.title");
  let name = baseName;
  let index = 0;
  while (names.has(name.toLocaleLowerCase())) name = `${baseName} ${++index}`;
  newTagName.value = name;
  newTagNameEdited.value = false;
  newTagGroupId.value = groupId;
  duplicateTag.value = null;
  isInTagList.value = false;
  focusedTagIndex.value = -1;
  await nextTick();
  newTagNameInputRef.value?.focus();
  newTagNameInputRef.value?.select();
}

function cancelNewTag() {
  if (isCreatingTag.value) return;
  newTagGroupId.value = null;
  newTagName.value = "";
  newTagNameEdited.value = false;
  duplicateTag.value = null;
}

function onNewTagBlur() {
  if (!newTagNameEdited.value) cancelNewTag();
  else addNewTag();
}

async function addNewTag() {
  if (!canEditTags.value || newTagGroupId.value === null || isCreatingTag.value) return;
  const name = newTagName.value.trim();
  if (!name) {
    cancelNewTag();
    return;
  }
  const existing = allTags.value.find(
    (tag) => tag.name.toLocaleLowerCase() === name.toLocaleLowerCase(),
  );
  if (existing) {
    duplicateTag.value = existing;
    return;
  }
  isCreatingTag.value = true;
  try {
    const newTag = await createTag(name, newTagGroupId.value);
    if (!newTag) {
      toast.error(t("tag.name_save_failed"));
      return;
    }
    allTags.value.push(newTag);
    toggleTag(newTag.id);
    newTagGroupId.value = null;
    newTagName.value = "";
    duplicateTag.value = null;
    await tauriEmit("tags-changed");
    await nextTick();
    document.getElementById(`dialog-tag-${newTag.id}`)?.scrollIntoView({ block: "nearest" });
  } catch {
    toast.error(t("tag.name_save_failed"));
  } finally {
    isCreatingTag.value = false;
  }
}

function toggleTag(tagId: number) {
  if (!canEditTags.value) return;
  const normalized = Number(tagId);
  const next = new Map(tagChanges.value);
  const current = next.get(normalized);
  const initialState = initialSelectedTags.value.has(normalized)
    ? "selected"
    : initialIntermediateTags.value.has(normalized)
      ? "intermediate"
      : "none";

  if (current === "add") {
    if (initialState === "none") {
      next.delete(normalized);
    } else {
      next.set(normalized, "remove");
    }
    setTagVisualState(normalized, "none");
  } else if (current === "remove") {
    next.delete(normalized);
    setTagVisualState(normalized, initialState);
  } else if (selectedTags.value.has(normalized)) {
    next.set(normalized, "remove");
    setTagVisualState(normalized, "none");
  } else {
    next.set(normalized, "add");
    setTagVisualState(normalized, "selected");
  }
  tagChanges.value = next;
}

function setTagVisualState(
  tagId: number,
  state: "selected" | "intermediate" | "none",
) {
  const selected = new Set(selectedTags.value);
  const intermediate = new Set(intermediateTags.value);
  selected.delete(tagId);
  intermediate.delete(tagId);
  if (state === "selected") selected.add(tagId);
  if (state === "intermediate") intermediate.add(tagId);
  selectedTags.value = selected;
  intermediateTags.value = intermediate;
}

async function startRename(tag: any) {
  renamingId.value = Number(tag.id);
  renameValue.value = String(tag.name || "");
  await nextTick();
  const input = Array.isArray(renameInput.value)
    ? renameInput.value[0]
    : renameInput.value;
  input?.focus({ preventScroll: true });
  input?.select();
}

function cancelRename() {
  renamingId.value = null;
  renameValue.value = "";
}

async function commitRename(tag: any) {
  if (renamingId.value !== Number(tag.id)) return;
  const name = renameValue.value.trim();
  if (name && name !== tag.name) {
    const duplicate = allTags.value.some(
      (item) =>
        Number(item.id) !== Number(tag.id) &&
        String(item.name).toLocaleLowerCase() === name.toLocaleLowerCase(),
    );
    if (duplicate) {
      toast.error(t("tag.name_exists"));
      cancelRename();
      return;
    }
    if (await renameTag(Number(tag.id), name)) {
      tag.name = name;
      await tauriEmit("tags-changed");
    } else {
      toast.error(t("tag.name_save_failed"));
    }
  }
  cancelRename();
}

async function confirmDelete() {
  const tag = deleteTarget.value;
  if (!tag) return;
  deleteTarget.value = null;
  if (!(await deleteTag(Number(tag.id)))) return;
  const tagId = Number(tag.id);
  allTags.value = allTags.value.filter((item) => Number(item.id) !== tagId);
  selectedTags.value.delete(tagId);
  intermediateTags.value.delete(tagId);
  initialSelectedTags.value.delete(tagId);
  initialIntermediateTags.value.delete(tagId);
  tagChanges.value.delete(tagId);
  const states = await applyTagsToFiles(props.fileIds, [], []);
  if (states !== null) emit("states-changed", states);
  await tauriEmit("tags-changed");
}

async function clickOk() {
  if (
    isLoadingTags.value ||
    isLoadingCatalog.value ||
    isApplyingTags.value ||
    isCreatingTag.value ||
    newTagGroupId.value !== null ||
    tagLoadFailed.value
  )
    return;
  isApplyingTags.value = true;
  const addTagIds = Array.from(tagChanges.value)
    .filter(([, change]) => change === "add")
    .map(([tagId]) => tagId);
  const removeTagIds = Array.from(tagChanges.value)
    .filter(([, change]) => change === "remove")
    .map(([tagId]) => tagId);
  const result = await applyTagsToFiles(props.fileIds, addTagIds, removeTagIds);
  if (result !== null) {
    await tauriEmit("tags-changed");
    emit("ok", result);
  } else {
    isApplyingTags.value = false;
  }
}

function clickCancel() {
  emit("cancel");
}

watch(newTagName, () => { duplicateTag.value = null; });

// Reset tag focus when search results change
watch(filteredTags, () => {
  focusedTagIndex.value = -1;
  isInTagList.value = false;
});

function onSearchFocus() {
  isSearchFocused.value = true;
  isInTagList.value = false;
  focusedTagIndex.value = -1;
}

function onSearchBlur() {
  isSearchFocused.value = false;
}

function enterTagList() {
  if (filteredTags.value.length > 0) {
    isInTagList.value = true;
    focusedTagIndex.value = 0;
    tagSearchInputRef.value?.blur();
    newTagNameInputRef.value?.blur();
  }
}

function exitTagList() {
  isInTagList.value = false;
  focusedTagIndex.value = -1;
  tagSearchInputRef.value?.focus();
}

// Keyboard: ArrowDown→tag list, Space→toggle, Enter→OK, Escape→back/close
const handleKeyDown = (e: KeyboardEvent) => {
  if (!uiStore.isInputActive("TaggingDialog")) return;

  const { key } = e;
  const active = document.activeElement;
  if (
    active instanceof HTMLSelectElement ||
    (active instanceof HTMLButtonElement && !isInTagList.value)
  )
    return;
  const isInAnyInput =
    active === tagSearchInputRef.value || active === newTagNameInputRef.value;

  // Escape: tag list → search input → close dialog
  if (key === "Escape") {
    if (isInTagList.value) {
      exitTagList();
    } else {
      clickCancel();
    }
    return;
  }

  // ArrowDown → enter tag list (from any state, unless already navigating)
  if (key === "ArrowDown" && !isInTagList.value) {
    e.preventDefault();
    enterTagList();
    return;
  }

  // Tag list keyboard navigation
  if (isInTagList.value && filteredTags.value.length > 0) {
    const lastIndex = filteredTags.value.length - 1;

    if (key === "ArrowRight") {
      e.preventDefault();
      focusedTagIndex.value =
        focusedTagIndex.value >= lastIndex ? 0 : focusedTagIndex.value + 1;
    } else if (key === "ArrowLeft") {
      e.preventDefault();
      focusedTagIndex.value =
        focusedTagIndex.value <= 0 ? lastIndex : focusedTagIndex.value - 1;
    } else if (key === "ArrowUp") {
      e.preventDefault();
      exitTagList();
    } else if (key === " ") {
      e.preventDefault();
      const tag = filteredTags.value[focusedTagIndex.value];
      if (tag) toggleTag(tag.id);
    } else if (key === "Enter") {
      e.preventDefault();
      clickOk();
    } else if (key === "Tab") {
      e.preventDefault();
      if (e.shiftKey) {
        isInTagList.value = false;
        focusedTagIndex.value = -1;
        tagSearchInputRef.value?.focus();
      } else {
        exitTagList(); // Tab → back to search input (keep focus inside dialog)
      }
    }
    return;
  }

  // Enter → confirm dialog (when not typing in any input)
  if (key === "Enter" && !isInAnyInput) {
    e.preventDefault();
    clickOk();
  }
};
</script>
