<template>
  <aside class="flex flex-col shrink-0 w-56 my-4 overflow-y-auto border-r-2">
    <button
      :class="itemStyle('hand-importer')"
      @click="store.sideView = 'hand-importer'"
    >
      Import / Export
    </button>

    <button
      :class="itemStyle('oop-range')"
      @click="store.sideView = 'oop-range'"
    >
      OOP Range
      <span class="flex my-2 justify-center">
        <RangeMiniViewer :player="Position.OOP" />
      </span>
    </button>

    <button :class="itemStyle('ip-range')" @click="store.sideView = 'ip-range'">
      IP Range
      <span class="flex my-2 justify-center">
        <RangeMiniViewer :player="Position.IP" />
      </span>
    </button>

    <button :class="itemStyle('board')" @click="store.sideView = 'board'">
      Board
      <span class="flex mt-1 justify-center font-semibold">
        <span
          v-for="(item, i) in boardTexts"
          :key="i"
          :class="
            'inline-block ' + (i === 3 ? 'mx-1 ' : 'mx-0.5 ') + item.colorClass
          "
        >
          {{ item.rank + item.suit }}
        </span>
      </span>
    </button>

    <button
      :class="itemStyle('tree-config')"
      @click="store.sideView = 'tree-config'"
    >
      Tree Configuration
    </button>

    <button :class="itemStyle('bunching')" @click="store.sideView = 'bunching'">
      Bunching Effect
    </button>

    <button
      :class="itemStyle('run-solver')"
      @click="store.sideView = 'run-solver'"
    >
      Run Solver
    </button>

    <button :class="itemStyle('about')" @click="store.sideView = 'about'">
      About
    </button>
  </aside>
</template>

<script setup lang="ts">
import { computed } from "vue";
import { SideView, useStore, useConfigStore } from "../store";
import { Position, cardText } from "../utils";

import RangeMiniViewer from "./RangeMiniViewer.vue";

const store = useStore();
const config = useConfigStore();

const itemStyle = (view: SideView) => {
  return (
    "side-bar-item " + (view === store.sideView ? "font-bold bg-blue-100" : "")
  );
};

const boardTexts = computed(() => {
  if (config.board.length === 0) {
    return [{ rank: "-", suit: "", colorClass: "text-black" }];
  } else {
    return config.board.map(cardText);
  }
});
</script>

<style scoped>
.side-bar-item {
  @apply block mx-2 my-[0.1875rem] px-4 py-2.5 rounded-3xl;
  @apply text-left text-[1.0625rem] select-none;
  @apply transition-colors hover:bg-blue-100;
}
</style>
