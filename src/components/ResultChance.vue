<template>
  <div class="flex w-full h-full">
    <div class="flex flex-col h-full pt-[2%] gap-[1%]" style="flex: 4">
      <div
        v-for="suit in 4"
        :key="suit"
        class="flex shrink-0 w-full justify-center gap-[1%]"
      >
        <!-- <div class="w-[5.5%]"></div> -->
        <BoardSelectorCard
          v-for="rank in 13"
          :key="rank"
          class="disabled:opacity-75 disabled:brightness-75"
          font-size="max(1.2vw, 13px)"
          width="5.5%"
          :card-id="56 - 4 * rank - suit"
          :disabled="selectedChance.cards[56 - 4 * rank - suit].isDead"
          :is-selected="selectedChance.selectedIndex === 56 - 4 * rank - suit"
          @click="deal(56 - 4 * rank - suit)"
        />
      </div>

      <div
        ref="chartParentDiv"
        class="flex flex-grow w-full max-h-[50%] mt-3 justify-center"
      >
        <div
          v-if="chartParentDivHeight >= 180"
          class="flex w-[90%] h-full items-center justify-center"
        >
          <!-- Chart here -->
        </div>
      </div>
    </div>

    <ResultTable
      style="flex: 3"
      table-mode="chance"
      :chance-type="selectedChance.player"
      :cards="null"
      :selected-spot="selectedSpot"
      :results="null"
      :chance-reports="chanceReports"
      :display-player="displayPlayer"
      :hover-content="null"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, watch } from "vue";
import { ChanceReports, Spot, SpotChance } from "../result-types";

import BoardSelectorCard from "./BoardSelectorCard.vue";
import ResultTable from "./ResultTable.vue";

defineProps<{
  selectedSpot: Spot;
  selectedChance: SpotChance;
  chanceReports: ChanceReports | null;
  displayPlayer: "oop" | "ip";
}>();

const emit = defineEmits<{
  (event: "deal-card", card: number): void;
}>();

const chartParentDiv = ref<HTMLDivElement | null>(null);
const chartParentDivHeight = ref(0);

const assignChartParentDivHeight = () => {
  if (chartParentDiv.value) {
    chartParentDivHeight.value = chartParentDiv.value.clientHeight;
  }
};

watch(chartParentDiv, assignChartParentDivHeight);
window.addEventListener("resize", assignChartParentDivHeight);

const deal = (card: number) => {
  emit("deal-card", card);
};
</script>
