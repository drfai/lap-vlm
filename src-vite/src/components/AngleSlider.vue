<template>
  <div
    ref="rootRef"
    class="angle-slider relative w-full select-none touch-none outline-none"
    :class="[
      disabled ? 'opacity-40 pointer-events-none' : 'cursor-ew-resize',
      dragging ? 'is-dragging' : '',
    ]"
    :style="{ height: HEIGHT + 'px' }"
    role="slider"
    tabindex="0"
    :aria-valuemin="min"
    :aria-valuemax="max"
    :aria-valuenow="modelValue"
    :aria-valuetext="`${modelValue}°`"
    :aria-label="ariaLabel"
    @pointerdown="onPointerDown"
    @dblclick="onDoubleClick"
    @keydown="onKeyDown"
  >
    <span
      class="absolute left-1/2 top-0 -translate-x-1/2 text-xs font-medium tabular-nums cursor-pointer"
      :class="dragging ? 'text-primary' : 'text-base-content/70 hover:text-primary'"
      :title="ariaLabel"
      @pointerdown.stop
      @click.stop="onDoubleClick"
    >{{ modelValue }}°</span>
    <div class="angle-ruler absolute inset-x-0 bottom-0 h-5 overflow-hidden" aria-hidden="true">
      <div
        class="absolute bottom-1 h-px bg-base-content/20"
        :style="{ left: (50 + (min - modelValue) / (max - min) * 100) + '%', width: '100%' }"
      ></div>
      <div
        v-for="t in ticks"
        :key="t.value"
        class="absolute bottom-1 w-px rounded-sm"
        :class="t.isZero ? 'bg-primary' : t.isMajor ? 'bg-base-content/45' : 'bg-base-content/20'"
        :style="{
          left: (50 + (t.value - modelValue) / (max - min) * 100) + '%',
          height: (t.isZero ? 13 : t.isMajor ? 9 : 5) + 'px',
        }"
      ></div>
    </div>
    <div class="absolute bottom-0 left-1/2 h-5 w-0.5 -translate-x-1/2 rounded-full bg-primary" aria-hidden="true"></div>
  </div>
</template>

<script setup lang="ts">
import { computed, onBeforeUnmount, ref } from 'vue';

const props = withDefaults(
  defineProps<{
    modelValue: number;
    min?: number;
    max?: number;
    step?: number;
    /** interval (deg) between small ticks */
    minorStep?: number;
    /** interval (deg) between labeled ticks */
    majorStep?: number;
    /** |angle| within this many degrees snaps to 0 (center detent) */
    snapZero?: number;
    disabled?: boolean;
    ariaLabel?: string;
  }>(),
  {
    min: -45,
    max: 45,
    step: 1,
    minorStep: 5,
    majorStep: 15,
    snapZero: 1,
    disabled: false,
    ariaLabel: 'Angle',
  },
);

const emit = defineEmits<{
  (e: 'update:modelValue', value: number): void;
}>();

const HEIGHT = 40;

const rootRef = ref<HTMLElement | null>(null);
const dragging = ref(false);
let dragStartX = 0;
let dragStartValue = 0;
let dragWidth = 0;
onBeforeUnmount(() => {
  window.removeEventListener('pointermove', onPointerMove);
  window.removeEventListener('pointerup', onPointerUp);
  window.removeEventListener('pointercancel', onPointerUp);
});

interface Tick {
  value: number;
  isMajor: boolean;
  isZero: boolean;
}

const ticks = computed<Tick[]>(() => {
  const out: Tick[] = [];
  for (let v = props.min; v <= props.max + 1e-9; v += props.minorStep) {
    const value = Math.round(v);
    out.push({
      value,
      isMajor: Math.abs(v % props.majorStep) < 1e-6,
      isZero: Math.abs(v) < 1e-6,
    });
  }
  return out;
});

const quantize = (raw: number): number => {
  const v = Math.abs(raw) < props.snapZero ? 0 : Math.round(raw / props.step) * props.step;
  return Math.min(props.max, Math.max(props.min, Number(v.toFixed(2))));
};

const setValue = (v: number) => {
  if (v !== props.modelValue) emit('update:modelValue', v);
};

const onPointerMove = (e: PointerEvent) => {
  if (dragging.value && dragWidth > 0) {
    setValue(quantize(dragStartValue - (e.clientX - dragStartX) / dragWidth * (props.max - props.min)));
  }
};

const onPointerUp = () => {
  dragging.value = false;
  window.removeEventListener('pointermove', onPointerMove);
  window.removeEventListener('pointerup', onPointerUp);
  window.removeEventListener('pointercancel', onPointerUp);
};

const onPointerDown = (e: PointerEvent) => {
  if (props.disabled || e.button !== 0) return;
  e.preventDefault();
  rootRef.value?.focus();
  dragStartX = e.clientX;
  dragStartValue = props.modelValue;
  dragWidth = rootRef.value?.getBoundingClientRect().width ?? 0;
  dragging.value = true;
  rootRef.value?.setPointerCapture?.(e.pointerId);
  window.addEventListener('pointermove', onPointerMove);
  window.addEventListener('pointerup', onPointerUp);
  window.addEventListener('pointercancel', onPointerUp);
};

const onDoubleClick = () => {
  if (props.disabled) return;
  setValue(0);
};

const onKeyDown = (e: KeyboardEvent) => {
  if (props.disabled) return;
  const s = props.step;
  let handled = true;
  switch (e.key) {
    case 'ArrowLeft':
    case 'ArrowDown':
      setValue(quantize(props.modelValue - s));
      break;
    case 'ArrowRight':
    case 'ArrowUp':
      setValue(quantize(props.modelValue + s));
      break;
    case 'Home':
      setValue(props.min);
      break;
    case 'End':
      setValue(props.max);
      break;
    case '0':
      setValue(0);
      break;
    default:
      handled = false;
  }
  if (handled) {
    e.preventDefault();
    e.stopPropagation();
  }
};
</script>

<style scoped>
.angle-ruler {
  mask-image: linear-gradient(to right, transparent, black 15%, black 85%, transparent);
}

</style>
