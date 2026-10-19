<template>

  <div class="w-full min-w-60 flex flex-col items-center select-none my-1 rounded-box border border-base-content/5">

    <!-- title -->
    <div 
      :class="[
        'mt-2 p-1 rounded-box text-nowrap cursor-pointer',
        isSelected(year, -1) ? 'text-primary bg-base-100 hover:bg-base-100 selected-item' : 'hover:text-base-content hover:bg-base-100/30'
      ]"
      @click="clickDate(year, -1)"
    >
      {{ yearTitle }}
    </div>

    <!-- month list -->
    <div class="p-2 grid grid-cols-6 gap-x-2 gap-y-2 text-center">
      <div v-for="m in 12" 
        :key="m" 
        class="size-7 text-xs flex items-center justify-center rounded-box"
        :class="{
          'bg-base-content/5 cursor-default scale-80': sumMonthCount(m) === 0,
          'text-base-100/70 hover:text-base-100 hover:bg-base-content/80 hover:scale-110 transition-all cursor-pointer': sumMonthCount(m) > 0,
          'bg-base-content/30': heatLevel(sumMonthCount(m)) === 1,
          'bg-base-content/40': heatLevel(sumMonthCount(m)) === 2,
          'bg-base-content/50': heatLevel(sumMonthCount(m)) === 3,
          'bg-base-content/60': heatLevel(sumMonthCount(m)) === 4 && !isSelected(year, m),
          'text-base-100! bg-primary hover:bg-primary scale-110': isSelected(year, m),
          'border border-base-content/20': isThisMonth(year, m),
        }"
        @click="sumMonthCount(m) > 0 ? clickDate(year, m) : null" 
      >
        {{ sumMonthCount(m) > 0 ? (sumMonthCount(m) < 10000 ? sumMonthCount(m) : '9k+') : '' }}
      </div>
    </div>

  </div>

</template>

<script setup lang="ts">

import { computed, PropType } from 'vue';
import { useI18n } from 'vue-i18n';
import { libConfig } from '@/common/config';
import { formatDate } from '@/common/utils';

const props = defineProps({
  year: {
    type: Number,
    required: true,
  },
  months: {
    type: Object,
    required: true,
  },
  heatmapThresholds: {
    type: Array as PropType<number[] | null>,
    default: null,
  }
});

/// i18n
const { locale, messages } = useI18n();
const localeMsg = computed(() => messages.value[locale.value] as any);

// Title for the year
const yearTitle = computed(() => formatDate(props.year, 1, 1, 'year'));

// Sum the count values for the given month
function sumMonthCount(month: number) {
  let sum = 0;
  if (props.months[month]) {
    props.months[month].forEach((entry: any) => {
      sum += Number(entry.count) || 0; // Sum the count values, defaulting to 0 if missing
    });
  }
  return sum;
}

function heatLevel(count: number) {
  if (count === 0) return 0;

  const thresholds = props.heatmapThresholds;
  if (thresholds) {
    if (count < thresholds[0]) return 1;
    if (count < thresholds[1]) return 2;
    if (count < thresholds[2]) return 3;
    return 4;
  }

  if (count < 100) return 1;
  if (count < 1000) return 2;
  return 4;
}

// Check if the given month is this month
function isThisMonth(year: number, month: number) {
  const now = new Date();
  // Check if the given year and month match the current year and month
  return year === now.getFullYear() && (month - 1) === now.getMonth();
}

// Check if the year or month is selected
const isSelected = (year: number, month: number) => libConfig.calendar.year === year && libConfig.calendar.month === month;

// click a year or a month to select it
const clickDate = (year: number, month: number) => {
  libConfig.calendar.year = year;
  libConfig.calendar.month = month; // -1 means selecting a year
  libConfig.calendar.date = -1;   // -1 means selecting a month

  console.log('clickDate:', libConfig.calendar.year, libConfig.calendar.month, libConfig.calendar.date);
};

</script>
