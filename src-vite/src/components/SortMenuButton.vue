<template>
  <DropDownSelect
    iconOnly
    size="small"
    :icon="iconFor(modelValue)"
    :options="options"
    :extendOptions="extendOptions"
    :defaultIndex="dimOf(modelValue)"
    :defaultExtendIndex="dirOf(modelValue)"
    :title="resolvedTitle"
    @select="(dim, dir) => emit('update:modelValue', combine(dim, dir))"
  />
</template>

<script setup lang="ts">
// Compact in-view sort control for the sidebar panels and the calendar.
// Renders a two-tier menu like the main toolbar's sort: a dimension group
// (Name / Count, or Taken / Created / Modified), a separator, then the
// Ascending / Descending group (reusing the toolbar's sort_order_options).
// The stored value encodes both: value = dimension * 2 + direction.
import { computed } from 'vue';
import { useI18n } from 'vue-i18n';
import DropDownSelect from './DropDownSelect.vue';
import { getSelectOptions } from '@/common/utils';
import { IconSortingAsc, IconSortingDesc } from '@/common/icons';

const props = withDefaults(defineProps<{
  modelValue: number;
  kind?: 'category' | 'calendar';
  title?: string;
}>(), { kind: 'category', title: '' });

const emit = defineEmits<{ (e: 'update:modelValue', value: number): void }>();

const { t, locale, messages } = useI18n();
const localeMsg = computed(() => (messages.value as any)[locale.value]);
const isCalendar = computed(() => props.kind === 'calendar');
const resolvedTitle = computed(() => props.title || t('toolbar.tooltip.sort'));

// Dimension labels: category = [Name, Count]; calendar = [Taken, Created, Modified].
const options = computed(() =>
  getSelectOptions(
    localeMsg.value?.settings?.browse?.[isCalendar.value ? 'calendar_sort_options' : 'category_sort_options'] || [],
  ),
);
// Direction labels reuse the toolbar's sort order options: [Ascending, Descending].
const extendOptions = computed(() =>
  getSelectOptions(localeMsg.value?.toolbar?.filter?.sort_order_options || []),
);

const dimOf = (v: number) => Math.floor(Number(v) / 2);
const dirOf = (v: number) => Number(v) % 2;
const combine = (dim: number, dir: number) => Number(dim) * 2 + Number(dir);
// Icon reflects the direction (ascending / descending), matching the main
// toolbar's sort control.
const iconFor = (v: number) => (Number(v) % 2 === 0 ? IconSortingAsc : IconSortingDesc);
</script>
