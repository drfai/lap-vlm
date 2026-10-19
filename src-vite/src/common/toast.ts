import { MAX_TOASTS_PER_PLACEMENT } from '@/common/constants';
import { reactive } from 'vue';

export type ToastType = 'info' | 'success' | 'warning' | 'error';
export type ToastPlacement = 'bottom-right' | 'center';

export interface ToastOptions {
  message: string;
  type?: ToastType;
  placement?: ToastPlacement;
  duration?: number;
}

export interface ToastItem {
  id: number;
  message: string;
  type: ToastType;
  placement: ToastPlacement;
  duration: number;
}

const DEFAULT_DURATION: Record<ToastType, number> = {
  info: 1200,
  success: 1600,
  warning: 2400,
  error: 3000,
};

const items = reactive<ToastItem[]>([]);
let nextToastId = 1;

function removeToast(id: number) {
  const index = items.findIndex((item) => item.id === id);
  if (index !== -1) {
    items.splice(index, 1);
  }
}

function showToast(options: ToastOptions) {
  const message = options.message?.trim();
  if (!message) return 0;

  const type = options.type ?? 'info';
  const placement = options.placement ?? 'bottom-right';
  const duration = options.duration ?? DEFAULT_DURATION[type];
  const id = nextToastId++;

  const samePlacementItems = items.filter((item) => item.placement === placement);
  if (samePlacementItems.length >= MAX_TOASTS_PER_PLACEMENT) {
    removeToast(samePlacementItems[0].id);
  }

  items.push({
    id,
    message,
    type,
    placement,
    duration,
  });

  window.setTimeout(() => {
    removeToast(id);
  }, duration);

  return id;
}

export function useToast() {
  return {
    items,
    showToast,
    removeToast,
    info(message: string, options: Omit<ToastOptions, 'message' | 'type'> = {}) {
      return showToast({ ...options, message, type: 'info' });
    },
    success(message: string, options: Omit<ToastOptions, 'message' | 'type'> = {}) {
      return showToast({ ...options, message, type: 'success' });
    },
    warning(message: string, options: Omit<ToastOptions, 'message' | 'type'> = {}) {
      return showToast({ ...options, message, type: 'warning' });
    },
    error(message: string, options: Omit<ToastOptions, 'message' | 'type'> = {}) {
      return showToast({ ...options, message, type: 'error' });
    },
  };
}
