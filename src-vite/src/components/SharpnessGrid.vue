<template>
  <svg v-if="enabled && grid && analyzedSource === source" class="absolute inset-0 h-full w-full pointer-events-none" :viewBox="`0 0 ${width} ${height}`" aria-hidden="true">
    <path v-for="(edge, index) in paths" :key="index" :d="edge.d" fill="none" stroke="white" :stroke-opacity="edge.intensity * 0.85" stroke-width="1" />
  </svg>
</template>

<script setup lang="ts">
import { computed, shallowRef, watch } from 'vue';
import { computeSharpnessGrid } from '@/common/sharpnessGrid';

const props = defineProps<{ source: string; enabled: boolean; width: number; height: number; rotate: number }>();
const grid = shallowRef<ReturnType<typeof computeSharpnessGrid> | null>(null);
let analyzedSource = '';

const paths = computed(() => {
  const result: { d: string; intensity: number }[] = [];
  if (!grid.value || !(props.width > 0 && props.height > 0)) return result;
  const rotation = ((props.rotate % 360) + 360) % 360;
  const width = rotation % 180 ? props.height : props.width;
  const height = rotation % 180 ? props.width : props.height;
  const angle = rotation * Math.PI / 180;
  const cos = Math.cos(angle);
  const sin = Math.sin(angle);
  // Draw at display size; half-pixel alignment keeps 1px strokes crisp on both axes.
  const snap = (value: number, size: number) => Math.max(0.5, Math.min(size - 0.5, Math.round(value - 0.5) + 0.5));
  const point = (x: number, y: number) => {
    const dx = x * width / grid.value!.columns - width / 2;
    const dy = y * height / grid.value!.rows - height / 2;
    return `${snap(dx * cos - dy * sin + props.width / 2, props.width)},${snap(dx * sin + dy * cos + props.height / 2, props.height)}`;
  };
  const edges = new Map<string, { d: string; intensity: number }>();
  const edge = (x1: number, y1: number, x2: number, y2: number, intensity: number) => {
    const key = `${Math.min(x1, x2)},${Math.min(y1, y2)},${Math.max(x1, x2)},${Math.max(y1, y2)}`;
    const existing = edges.get(key);
    if (existing) {
      // Shared edges use the stronger cell without stacking translucent strokes.
      existing.intensity = Math.max(existing.intensity, intensity);
      return;
    }
    const segment = { d: `M${point(x1, y1)}L${point(x2, y2)}`, intensity };
    edges.set(key, segment);
    result.push(segment);
  };
  for (const { x, y, intensity } of grid.value.cells) {
    edge(x, y, x + 1, y, intensity);
    edge(x + 1, y, x + 1, y + 1, intensity);
    edge(x, y + 1, x + 1, y + 1, intensity);
    edge(x, y, x, y + 1, intensity);
  }
  return result;
});

watch(() => [props.source, props.enabled] as const, async ([source, enabled], _, onCleanup) => {
  if (source !== analyzedSource) {
    grid.value = null;
    analyzedSource = '';
  }
  if (!enabled || !source || source === analyzedSource) return;
  const controller = new AbortController();
  onCleanup(() => controller.abort());
  let bitmap: ImageBitmap | null = null;
  try {
    const response = await fetch(source, { signal: controller.signal });
    if (!response.ok) return;
    bitmap = await createImageBitmap(await response.blob());
    if (controller.signal.aborted) return;
    // Bound work to the thumbnail, never upscale or decode the original image.
    const scale = Math.min(1, 512 / Math.max(bitmap.width, bitmap.height));
    const canvas = document.createElement('canvas');
    canvas.width = Math.max(1, Math.round(bitmap.width * scale));
    canvas.height = Math.max(1, Math.round(bitmap.height * scale));
    const context = canvas.getContext('2d', { willReadFrequently: true });
    if (!context) return;
    context.drawImage(bitmap, 0, 0, canvas.width, canvas.height);
    const result = computeSharpnessGrid(context.getImageData(0, 0, canvas.width, canvas.height).data, canvas.width, canvas.height);
    analyzedSource = source;
    grid.value = result;
  } catch {
    // Thumbnail loading failures leave the navigator usable without an overlay.
  } finally {
    bitmap?.close();
  }
}, { immediate: true });
</script>
