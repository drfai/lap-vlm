<template>
  <Teleport to="body">
    <TransitionGroup
      name="toast"
      tag="div"
      class="pointer-events-none fixed inset-0 z-950 flex items-center justify-center px-4"
    >
      <div
        v-for="item in centerItems"
        :key="item.id"
        :class="toastClass(item.type)"
      >
        <span class="line-clamp-2">{{ item.message }}</span>
      </div>
    </TransitionGroup>

    <TransitionGroup
      name="toast"
      tag="div"
      :class="bottomRightContainerClass"
    >
      <div
        v-for="item in bottomRightItems"
        :key="item.id"
        :class="toastClass(item.type)"
      >
        <span class="line-clamp-2">{{ item.message }}</span>
      </div>
    </TransitionGroup>
  </Teleport>
</template>

<script setup lang="ts">
import { computed, Teleport } from 'vue';
import { config } from '@/common/config';
import { useToast, type ToastType } from '@/common/toast';

const toast = useToast();

const centerItems = computed(() =>
  toast.items.filter((item) => item.placement === 'center')
);

const bottomRightItems = computed(() =>
  toast.items.filter((item) => item.placement === 'bottom-right')
);

const bottomRightContainerClass = computed(() => [
  'pointer-events-none fixed right-2 z-950 flex max-w-sm flex-col items-end gap-2',
  config.settings.showStatusBar ? 'bottom-10' : 'bottom-2',
]);

function toastClass(type: ToastType) {
  const base =
    'pointer-events-auto min-w-[12rem] max-w-full rounded-box px-4 py-2 text-sm shadow-lg backdrop-blur-md bg-base-100/70';

  switch (type) {
    case 'success':
      return `${base} border-success/15 text-success`;
    case 'warning':
      return `${base} border-warning/20 text-warning`;
    case 'error':
      return `${base} border-error/20 text-error`;
    default:
      return `${base} border-base-content/8 text-base-content/70`;
  }
}
</script>

<style scoped>
.toast-enter-active,
.toast-leave-active {
  transition: opacity 0.2s ease, transform 0.2s ease;
}

.toast-enter-from,
.toast-leave-to {
  opacity: 0;
  transform: translateY(8px);
}
</style>
