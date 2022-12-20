<template>
  <button
    :class="
      'relative rounded-lg border select-none enabled:shadow ' +
      (isSelected
        ? 'bg-yellow-300 ring-1 ring-red-600 border-red-600'
        : 'bg-white border-black')
    "
    :style="{
      '--width': width,
      '--font-size': fontSize,
      width: 'var(--width)',
      'padding-top': 'calc(var(--width) * 1.4 - 2px)',
    }"
  >
    <span
      :class="'absolute top-0 font-semibold ' + colorClass"
      :style="{
        left: '15%',
        'font-size': 'calc(var(--font-size) * 1.25)',
      }"
    >
      {{ rank }}
    </span>
    <span
      :class="'absolute ' + colorClass"
      :style="{
        bottom: '5%',
        right: '10%',
        'font-size': 'var(--font-size)',
      }"
    >
      {{ suit }}
    </span>
  </button>
</template>

<script setup lang="ts">
import { cardText } from "../utils";

interface Props {
  cardId: number;
  isSelected?: boolean;
  width?: string;
  fontSize?: string;
}

const props = withDefaults(defineProps<Props>(), {
  isSelected: false,
  width: "40px",
  fontSize: "1rem",
});

const { rank, suit, colorClass } = cardText(props.cardId);
</script>
