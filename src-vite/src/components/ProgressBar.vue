<template>
  <div class="px-1 w-full rounded-box h-0.5">
    <div
      class="h-full rounded-box bg-primary transition-all duration-300 ease-out"
      :style="{ width: progressWidth }"
    ></div>
  </div>
</template>


<script setup>

import { computed } from 'vue';

const props = defineProps({
  percent: {
    type: Number,
    required: true,
  },
});

// Computed property to calculate progress width
const progressWidth = computed(() => {
  // If percent is less than 1, set it to 1
  const adjustedPercent = props.percent < 1 ? 1 : props.percent;
  return `${adjustedPercent}%`;
});

</script>
