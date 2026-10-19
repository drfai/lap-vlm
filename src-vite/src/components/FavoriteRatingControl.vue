<template>
  <div class="h-6 flex items-center gap-0.5">
    <button
      class="btn btn-ghost btn-xs min-h-0 h-6 w-6 p-0 mr-1 relative"
      :title="favoriteTitle"
      :aria-label="favoriteTitle"
      :disabled="disabled"
      @click.stop="emit('favorite', favorite === true ? false : true)"
    >
      <component
        :is="favorite === true || favorite === null ? IconHeartFilled : IconHeart"
        class="w-3.5 h-3.5"
        :class="favoriteIconClass"
      />
    </button>
    <div class="w-px h-4 shrink-0 bg-base-content/10 mx-1"></div>
    <!-- <span class="mr-1 text-[11px] font-medium" :class="disabled ? 'text-base-content/30' : labelClass">
      {{ $t('rating.title') }}
    </span> -->
    <span
      v-if="rating === null && !disabled"
      class="mr-1 rounded border border-base-content/5 px-1 py-0.5 text-[9px] font-bold uppercase tracking-wide text-warning/70"
      :title="`${$t('rating.title')}: ${$t('rating.mixed')}`"
    >
      {{ $t('rating.mixed') }}
    </span>
    <button
      v-for="value in [1, 2, 3, 4, 5]"
      :key="value"
      class="btn btn-ghost btn-xs min-h-0 h-6 w-6 p-0"
      :title="getRatingLabel(value)"
      :aria-label="getRatingLabel(value)"
      :disabled="disabled"
      @click.stop="emit('rating', rating === value ? 0 : value)"
    >
      <component
        :is="Number(rating || 0) >= value ? IconStarFilled : IconStar"
        class="w-3.5 h-3.5"
        :class="getRatingIconClass(value)"
      />
    </button>
    <div class="w-px h-4 shrink-0 bg-base-content/10 mx-1"></div>
    <span
      v-if="culling === null && !disabled"
      class="mr-1 rounded border border-base-content/5 px-1 py-0.5 text-[9px] font-bold uppercase tracking-wide text-primary/70"
      :title="`${$t('culling.title')}: ${$t('rating.mixed')}`"
    >
      {{ $t('rating.mixed') }}
    </span>
    <button
      class="btn btn-ghost btn-xs min-h-0 h-6 w-6 p-0"
      :title="$t('culling.picks')"
      :aria-label="$t('culling.picks')"
      :disabled="disabled"
      @click.stop="emit('culling', 1)"
    >
      <component :is="IconFlagFilled" class="w-3.5 h-3.5" :class="getCullingIconClass(1)" />
    </button>
    <button
      class="btn btn-ghost btn-xs min-h-0 h-6 w-6 p-0"
      :title="$t('culling.rejected')"
      :aria-label="$t('culling.rejected')"
      :disabled="disabled"
      @click.stop="emit('culling', 2)"
    >
      <component :is="IconFlagOff" class="w-3.5 h-3.5" :class="getCullingIconClass(2)" />
    </button>
    <button
      class="btn btn-ghost btn-xs min-h-0 h-6 w-6 p-0"
      :title="$t('culling.unreviewed')"
      :aria-label="$t('culling.unreviewed')"
      :disabled="disabled"
      @click.stop="emit('culling', 0)"
    >
      <component :is="IconFlag" class="w-3.5 h-3.5" :class="getCullingIconClass(0)" />
    </button>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { useI18n } from 'vue-i18n';
import { IconFlag, IconFlagFilled, IconFlagOff, IconHeart, IconHeartFilled, IconStar, IconStarFilled } from '@/common/icons';

const props = withDefaults(defineProps<{
  favorite: boolean | null;
  rating: number | null;
  culling: number | null;
  disabled?: boolean;
  favoriteLabel?: string;
  unfavoriteLabel?: string;
  labelClass?: string;
  inactiveRatingClass?: string;
}>(), {
  disabled: false,
  favoriteLabel: '',
  unfavoriteLabel: '',
  labelClass: 'text-base-content/50',
  inactiveRatingClass: 'text-base-content/30',
});

const emit = defineEmits<{
  favorite: [value: boolean];
  rating: [value: number];
  culling: [value: number];
}>();

const { locale, messages, t } = useI18n();
const localeMsg = computed(() => messages.value[locale.value] as any);

const favoriteTitle = computed(() => {
  if (props.favorite === null) {
    const label = props.favoriteLabel || t('menu.meta.favorite');
    return `${label} (${t('rating.mixed')})`;
  }
  if (props.favorite) {
    return props.unfavoriteLabel || t('menu.meta.unfavorite');
  }
  return props.favoriteLabel || t('menu.meta.favorite');
});

const favoriteIconClass = computed(() => {
  if (props.disabled) return 'text-base-content/30';
  if (props.favorite === true) return 'text-error';
  return 'text-base-content/70 hover:text-error/70';
});

function getRatingLabel(rating: number) {
  const keys: Record<number, string> = {
    5: 'five_stars',
    4: 'four_stars',
    3: 'three_stars',
    2: 'two_stars',
    1: 'one_star',
  };
  const key = keys[rating];
  return localeMsg.value.favorite?.[key] || `${rating}★`;
}

function getRatingIconClass(value: number) {
  if (props.disabled) return 'text-base-content/30';
  if (props.rating === null) return 'text-base-content/70 hover:text-warning/70';
  if (Number(props.rating || 0) >= value) return 'text-warning';
  return `${props.inactiveRatingClass} hover:text-warning/70`;
}

function getCullingIconClass(value: number) {
  if (props.disabled) return 'text-base-content/30';
  if (props.culling === null) return 'text-base-content/70 hover:text-primary/70';
  if (Number(props.culling) === value) {
    return value === 2 ? 'text-error' : 'text-primary';
  }
  return value === 2 ? 'text-base-content/30 hover:text-error/70' : 'text-base-content/30 hover:text-primary/70';
}
</script>
