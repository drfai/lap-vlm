<template>
  <Teleport to="body">
    <div class="print-only">
      <img v-if="src" ref="imageRef" :src="src" alt="" />
    </div>
  </Teleport>
</template>

<script setup lang="ts">
import { ref, nextTick, onUnmounted } from 'vue';

const src = ref('');
const imageRef = ref<HTMLImageElement | null>(null);
let finish: (() => void) | undefined;
let pageStyle: HTMLStyleElement | undefined;

async function print(source: string, orientation?: 'landscape' | 'portrait') {
  src.value = source;
  try {
    if (orientation) {
      pageStyle = document.createElement('style');
      pageStyle.textContent = `@media print { @page { size: A4 ${orientation}; margin: 0; } }`;
      document.head.appendChild(pageStyle);
    }
    await nextTick();
    if (!imageRef.value) throw new Error('Print image element was not rendered');
    await imageRef.value.decode();
    await new Promise(resolve => setTimeout(resolve, 100));
    await new Promise<void>((resolve, reject) => {
      finish = () => {
        window.removeEventListener('afterprint', finish!);
        src.value = '';
        pageStyle?.remove();
        pageStyle = undefined;
        finish = undefined;
        resolve();
      };
      window.addEventListener('afterprint', finish, { once: true });
      try { window.print(); } catch (error) {
        window.removeEventListener('afterprint', finish!);
        finish = undefined;
        reject(error);
      }
    });
  } catch (error) {
    src.value = '';
    pageStyle?.remove();
    pageStyle = undefined;
    throw error;
  }
}

onUnmounted(() => {
  finish?.();
  pageStyle?.remove();
});
defineExpose({ print });
</script>

<style scoped>
.print-only { display: none; }
@media print {
  @page { margin: 0; }
  :global(html), :global(body) {
    margin: 0;
    padding: 0;
    width: 100%;
    height: 100%;
    overflow: hidden;
  }
  :global(body > *:not(.print-only)) { display: none !important; }
  .print-only {
    position: absolute;
    inset: 0;
    box-sizing: border-box;
    display: grid !important;
    place-items: center;
    width: 100%;
    height: 100%;
    overflow: hidden;
    background: #fff;
    break-inside: avoid;
  }
  .print-only img {
    display: block;
    width: 100%;
    height: 100%;
    object-fit: contain;
    object-position: center;
  }
}
</style>
