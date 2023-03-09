<template>
  <div v-if="editingPlayer === -1">
    <div class="flex">
      <div
        class="px-3 py-1 text-cyan-600 bg-cyan-50 border-2 border-cyan-600 rounded-md font-semibold"
      >
        Enabling the bunching effect will significantly slow down the solver.<br />
        If you are not sure, we recommend leaving it disabled.
      </div>
    </div>

    <div class="flex mt-4 gap-8 items-center">
      <label class="cursor-pointer">
        <input
          type="checkbox"
          v-model="store.isBunchingEnabled"
          class="mr-1 align-middle rounded"
        />
        Enable bunching effect
      </label>

      <div class="flex">
        <Tippy
          class="inline-block cursor-help text-gray-600"
          max-width="500px"
          trigger="mouseenter click"
          placement="bottom"
          :delay="[200, 0]"
          :interactive="true"
        >
          <QuestionMarkCircleIcon class="inline w-5 h-5" />
          <div class="inline-block ml-0.5 text-sm underline">
            What is the bunching effect?
          </div>
          <template #content>
            <div class="px-1 py-0.5 text-justify">
              <div>
                The bunching effect refers to the card removal effect caused by
                the ranges of folded players. Since most players fold with a
                weak hand, the remaining players will have a higher chance of
                holding a strong hand.
              </div>
              <div class="mt-3">
                This option supports up to 4 folded players (6-max game). By
                entering the ranges of the folded players and performing a
                precomputation, the solver can account for the bunching effect.
              </div>
              <div class="mt-3">
                The simulation is exact and precise. This means that the solver
                does not make any abstractions or approximations. However,
                please note that enabling the bunching effect will significantly
                slow down the solver due to the heavy additional computation
                required.
              </div>
            </div>
          </template>
        </Tippy>
      </div>
    </div>

    <div v-if="store.isBunchingEnabled" class="mt-6">
      <div class="flex gap-8">
        <div v-for="i in 4" :key="i">
          <div class="text-gray-600">Player {{ i }}:</div>
          <RangeMiniViewer
            class="w-44 h-44 mt-2 cursor-pointer"
            :player="i + 1"
            @click="editRange(i - 1)"
          />
          <input
            v-model="rangeTexts[i - 1]"
            type="text"
            :class="
              'flex-grow mt-3 px-2 py-1 rounded-lg text-sm ' +
              (isRangeTextError[i - 1] ? ' input-error' : '')
            "
            @focus="($event.target as HTMLInputElement).select()"
            @change="onRangeTextChange(i - 1)"
          />
          <div class="mt-2 text-center">
            {{ numCombos[i - 1].toFixed(1) }} combos ({{
              ((numCombos[i - 1] * 100) / ((52 * 51) / 2)).toFixed(1)
            }}%)
          </div>
          <div class="flex mt-3 w-full justify-center gap-3">
            <button class="button-base button-blue" @click="invertRange(i - 1)">
              Invert
            </button>
            <button class="button-base button-blue" @click="clearRange(i - 1)">
              Clear
            </button>
          </div>
        </div>
      </div>

      <div class="flex mt-8 gap-3 items-center">
        <button
          class="button-base button-blue"
          :disabled="
            numCombos.every((x) => x === 0) ||
            hasBunchingRun ||
            store.isSolverRunning ||
            numThreads < 1 ||
            numThreads > 64 ||
            numThreads % 1 !== 0
          "
          @click="runPrecomputation"
        >
          Run Precomputation
        </button>
        <button
          class="button-base button-red"
          :disabled="!hasBunchingRun"
          @click="clearPrecomputation"
        >
          Clear
        </button>
        <button
          v-if="!isBunchingPaused"
          class="button-base button-green"
          :disabled="!store.isBunchingRunning"
          @click="() => (pauseFlag = true)"
        >
          Pause
        </button>
        <button
          v-else
          class="button-base button-green"
          :disabled="
            store.isSolverRunning ||
            numThreads < 1 ||
            numThreads > 64 ||
            numThreads % 1 !== 0
          "
          @click="resumePrecomputation"
        >
          Resume
        </button>

        <span class="pl-3">
          Number of threads:
          <input
            v-model="numThreads"
            type="number"
            :class="
              'w-20 ml-1.5 px-2 py-1 rounded-lg text-sm text-center ' +
              (numThreads < 1 || numThreads > 64 || numThreads % 1 !== 0
                ? 'input-error'
                : '')
            "
            min="1"
            max="64"
          />
        </span>
        <span class="pl-3">
          Memory usage:
          {{
            ["-", "60MB", "60MB", "190MB", "3.5GB"][
              numCombos.filter((x) => x > 0).length
            ]
          }}
        </span>
      </div>

      <div class="mt-3">
        <span>
          Status:
          <span
            v-if="store.isBunchingRunning"
            class="spinner inline-block mx-3"
          ></span>
          {{ statusText }}
        </span>
        <span v-if="store.bunchingFlop.length > 0">
          [Flop:
          <span
            v-for="item in store.bunchingFlop.map(cardText)"
            :key="item.rank + item.suit"
            :class="'inline-block ' + item.colorClass"
            >{{ item.rank + item.suit }}</span
          >]
        </span>
      </div>
    </div>
  </div>

  <RangeEditor
    v-else
    :player="editingPlayer + 2"
    :default-text="rangeTextCopy"
    @save="saveEdit"
    @cancel="cancelEdit"
  />
</template>

<script setup lang="ts">
import { ref } from "vue";
import { useStore, useConfigStore } from "../store";
import { trimRegex, rangeRegex, cardText } from "../utils";
import * as invokes from "../invokes";

import RangeEditor from "./RangeEditor.vue";
import RangeMiniViewer from "./RangeMiniViewer.vue";
import { Tippy } from "vue-tippy";
import { QuestionMarkCircleIcon } from "@heroicons/vue/20/solid";

const store = useStore();
const configStore = useConfigStore();

const numThreads = ref(navigator.hardwareConcurrency || 1);

const editingPlayer = ref(-1);
const rangeTexts = ref(["", "", "", ""]);
const isRangeTextError = ref([false, false, false, false]);
const numCombos = ref([0, 0, 0, 0]);
const rangeTextCopy = ref("");

const statusText = ref("No bunching data");
const flopCopy = ref<number[]>([]);
const hasBunchingRun = ref(false);
const isBunchingPaused = ref(false);
const terminateFlag = ref(false);
const pauseFlag = ref(false);
const elapsedTimeMs = ref(-1);

let startTime = 0;

const onUpdate = async (player: number) => {
  const weights = await invokes.rangeGetWeights(player + 2);
  for (let i = 0; i < 13 * 13; ++i) {
    store.ranges[player + 2][i] = weights[i] * 100;
  }
  isRangeTextError.value[player] = false;
};

const onUpdateLocal = async (player: number) => {
  rangeTexts.value[player] = await invokes.rangeToString(player + 2);
  numCombos.value[player] = await invokes.rangeNumCombos(player + 2);
};

const editRange = async (player: number) => {
  rangeTextCopy.value = await invokes.rangeToString(player + 2);
  store.headers["bunching"].push(`Player ${player + 1}`);
  editingPlayer.value = player;
};

const onRangeTextChange = async (player: number) => {
  const trimmed = rangeTexts.value[player].replace(trimRegex, "$1").trim();
  const ranges = trimmed.split(",");

  if (ranges[ranges.length - 1] === "") {
    ranges.pop();
  }

  for (const range of ranges) {
    if (!rangeRegex.test(range)) {
      isRangeTextError.value[player] = true;
      return;
    }
  }

  const errorString = await invokes.rangeFromString(player + 2, trimmed);

  if (errorString) {
    isRangeTextError.value[player] = true;
  } else {
    await onUpdate(player);
    await onUpdateLocal(player);
  }
};

const invertRange = async (player: number) => {
  await invokes.rangeInvert(player + 2);
  await onUpdate(player);
  await onUpdateLocal(player);
};

const clearRange = async (player: number) => {
  await invokes.rangeClear(player + 2);
  await onUpdate(player);
  await onUpdateLocal(player);
};

const runPrecomputation = async () => {
  const errorString = await invokes.bunchingInit(configStore.board);
  if (errorString) {
    statusText.value = `Error: ${errorString}`;
  } else {
    store.bunchingFlop = [];
    flopCopy.value = configStore.board.slice(0, 3);
    hasBunchingRun.value = true;
    statusText.value = "Phase 1/3 - Preparing...";
    elapsedTimeMs.value = 0;
    await resumePrecomputation();
  }
};

const clearPrecomputation = async () => {
  if (store.isBunchingRunning) {
    terminateFlag.value = true;
  } else {
    await invokes.bunchingClear();
    store.bunchingFlop = [];
    hasBunchingRun.value = false;
    statusText.value = "No bunching data";
  }
};

const resumePrecomputation = async () => {
  store.isBunchingRunning = true;
  isBunchingPaused.value = false;
  startTime = performance.now();
  await invokes.setNumThreads(numThreads.value);

  while (true) {
    if (terminateFlag.value) {
      await invokes.bunchingClear();
      store.bunchingFlop = [];
      hasBunchingRun.value = false;
      statusText.value = "No bunching data";
      break;
    }

    if (pauseFlag.value) {
      isBunchingPaused.value = true;
      break;
    }

    const [phase, percent] = await invokes.bunchingProgress();

    if (phase === 3 && percent === 100) {
      store.bunchingFlop = flopCopy.value;
      statusText.value = "Bunching data ready!";
      break;
    }

    statusText.value = `Phase ${phase}/3 - ${percent}%`;
  }

  elapsedTimeMs.value += performance.now() - startTime;

  if (!terminateFlag.value && !pauseFlag.value) {
    statusText.value += ` (Time: ${(elapsedTimeMs.value / 1000).toFixed(2)}s)`;
  }

  store.isBunchingRunning = false;
  terminateFlag.value = false;
  pauseFlag.value = false;
};

const saveEdit = async () => {
  isRangeTextError.value[editingPlayer.value] = false;
  await onUpdateLocal(editingPlayer.value);
  store.headers["bunching"].pop();
  editingPlayer.value = -1;
};

const cancelEdit = async () => {
  rangeTexts.value[editingPlayer.value] = rangeTextCopy.value;
  await invokes.rangeFromString(editingPlayer.value + 2, rangeTextCopy.value);
  await onUpdate(editingPlayer.value);
  store.headers["bunching"].pop();
  editingPlayer.value = -1;
};
</script>
