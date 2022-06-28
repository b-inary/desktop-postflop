<template>
  <p class="flex my-1 items-center">
    Number of threads:
    <input
      v-model="numThreads"
      type="number"
      :class="
        'w-20 ml-2 px-2 py-1 rounded-lg text-sm text-center ' +
        (numThreads < 1 || numThreads > 64 || numThreads % 1 !== 0
          ? 'ring-1 ring-red-600 border-red-600 bg-red-50'
          : '')
      "
      min="1"
      max="64"
    />
    <button
      class="ml-3 button button-blue"
      :disabled="isTreeBuilding || store.isSolverRunning || store.isFinalizing"
      @click="buildTree"
    >
      Build tree
    </button>
  </p>

  <p class="my-1">Status: {{ treeStatus }}</p>

  <div v-if="isTreeBuilt" class="mt-4">
    <p>
      <label
        :class="
          memoryUsage > 0.95 * availableMemory
            ? 'text-gray-400'
            : !store.hasSolverRun
            ? 'cursor-pointer'
            : ''
        "
      >
        <input
          v-model="isCompressionEnabled"
          class="mr-2 cursor-pointer disabled:opacity-40 disabled:cursor-default"
          type="radio"
          name="compression"
          :value="false"
          :disabled="store.hasSolverRun || memoryUsage > 0.95 * availableMemory"
        />
        No compression: requires
        {{
          memoryUsage >= 1023.5 * 1024 * 1024
            ? (memoryUsage / (1024 * 1024 * 1024)).toFixed(2) + " GB"
            : (memoryUsage / (1024 * 1024)).toFixed(0) + " MB"
        }}
        of RAM
        {{
          memoryUsage <= 0.85 * availableMemory
            ? "(fast)"
            : memoryUsage <= 0.95 * availableMemory
            ? "(not recommended)"
            : "(out of memory)"
        }}
      </label>
    </p>
    <p>
      <label
        :class="
          memoryUsageCompressed > 0.95 * availableMemory
            ? 'text-gray-400'
            : !store.hasSolverRun
            ? 'cursor-pointer'
            : ''
        "
      >
        <input
          v-model="isCompressionEnabled"
          class="mr-2 cursor-pointer disabled:opacity-40 disabled:cursor-default"
          type="radio"
          name="compression"
          :value="true"
          :disabled="
            store.hasSolverRun || memoryUsageCompressed > 0.95 * availableMemory
          "
        />
        Use compression: requires
        {{
          memoryUsageCompressed >= 1023.5 * 1024 * 1024
            ? (memoryUsageCompressed / (1024 * 1024 * 1024)).toFixed(2) + " GB"
            : (memoryUsageCompressed / (1024 * 1024)).toFixed(0) + " MB"
        }}
        of RAM
        {{
          memoryUsageCompressed <= 0.95 * availableMemory
            ? ""
            : "(out of memory)"
        }}
      </label>
    </p>

    <p class="mt-4">
      Target exploitability:
      <input
        v-model="targetExploitability"
        type="number"
        :class="
          'w-20 ml-2 px-2 py-1 rounded-lg text-sm text-center ' +
          (targetExploitability <= 0
            ? 'ring-1 ring-red-600 border-red-600 bg-red-50'
            : '')
        "
        :disabled="store.hasSolverRun && !store.isSolverPaused"
        min="0"
        step="0.05"
      />
      %
    </p>

    <p class="mt-1">
      Maximum number of iterations:
      <input
        v-model="maxIterations"
        type="number"
        :class="
          'w-[5.5rem] ml-2 px-2 py-1 rounded-lg text-sm text-center ' +
          (maxIterations < 0 ||
          maxIterations % 1 !== 0 ||
          maxIterations > 100000
            ? 'ring-1 ring-red-600 border-red-600 bg-red-50'
            : '')
        "
        :disabled="store.hasSolverRun && !store.isSolverPaused"
        min="0"
        max="100000"
      />
    </p>

    <p class="flex mt-6 gap-3">
      <button
        class="button button-blue"
        :disabled="
          store.hasSolverRun ||
          memoryUsageCompressed > 0.95 * availableMemory ||
          targetExploitability <= 0 ||
          maxIterations < 0 ||
          maxIterations % 1 !== 0 ||
          maxIterations > 100000
        "
        @click="runSolver"
      >
        Run solver
      </button>
      <button
        class="button button-red"
        :disabled="!store.isSolverRunning"
        @click="() => (terminateFlag = true)"
      >
        Stop
      </button>
      <button
        v-if="!store.isSolverPaused"
        class="button button-green"
        :disabled="!store.isSolverRunning"
        @click="() => (pauseFlag = true)"
      >
        Pause
      </button>
      <button
        v-else
        class="button button-green"
        :disabled="
          targetExploitability <= 0 ||
          maxIterations < 0 ||
          maxIterations % 1 !== 0 ||
          maxIterations > 100000
        "
        @click="resumeSolver"
      >
        Resume
      </button>
    </p>

    <div v-if="store.hasSolverRun" class="mt-6">
      <div class="flex items-center">
        <span
          v-if="store.isSolverRunning || store.isFinalizing"
          class="spinner inline-block mr-3"
        ></span>
        {{
          store.isSolverRunning
            ? "Solver running..."
            : store.isFinalizing
            ? "Finalizing..."
            : store.isSolverPaused
            ? "Solver paused."
            : "Solver finished."
        }}
      </div>
      {{ iterationText }}
      <br />
      {{ exploitabilityText }}
      <br />
      {{ timeText }}
    </div>
  </div>
</template>

<script lang="ts">
import { computed, defineComponent, ref } from "vue";
import {
  useStore,
  useConfigStore,
  useTmpConfigStore,
  saveConfig,
  saveConfigTmp,
} from "../store";
import { invoke } from "@tauri-apps/api";

function checkConfig(config: ReturnType<typeof useConfigStore>): string | null {
  if (config.board.length < 3) {
    return "Board must consist of at least three cards";
  }

  if (config.startingPot <= 0) {
    return "Starting pot must be positive";
  }

  if (config.startingPot > 100000) {
    return "Starting pot is too large";
  }

  if (config.startingPot % 1 !== 0) {
    return "Starting pot must be an integer";
  }

  if (config.effectiveStack <= 0) {
    return "Effective stack must be positive";
  }

  if (config.effectiveStack > 100000) {
    return "Effective stack is too large";
  }

  if (config.effectiveStack % 1 !== 0) {
    return "Effective stack is be an integer";
  }

  if (config.oopFlopBet === null) {
    return "Invalid tree configuration (OOP flop bet)";
  }

  if (config.oopFlopRaise === null) {
    return "Invalid tree configuration (OOP flop raise)";
  }

  if (config.oopTurnBet === null) {
    return "Invalid tree configuration (OOP turn bet)";
  }

  if (config.oopTurnRaise === null) {
    return "Invalid tree configuration (OOP turn raise)";
  }

  if (config.oopRiverBet === null) {
    return "Invalid tree configuration (OOP river bet)";
  }

  if (config.oopRiverRaise === null) {
    return "Invalid tree configuration (OOP river raise)";
  }

  if (config.ipFlopBet === null) {
    return "Invalid tree configuration (IP flop bet)";
  }

  if (config.ipFlopRaise === null) {
    return "Invalid tree configuration (IP flop raise)";
  }

  if (config.ipTurnBet === null) {
    return "Invalid tree configuration (IP turn bet)";
  }

  if (config.ipTurnRaise === null) {
    return "Invalid tree configuration (IP turn raise)";
  }

  if (config.ipRiverBet === null) {
    return "Invalid tree configuration (IP river bet)";
  }

  if (config.ipRiverRaise === null) {
    return "Invalid tree configuration (IP river raise)";
  }

  if (config.addAllInThreshold < 0) {
    return "Invalid add all-in threshold";
  }

  if (config.addAllInThreshold > 100000) {
    return "Add all-in threshold is too large";
  }

  if (config.forceAllInThreshold < 0) {
    return "Invalid force all-in threshold";
  }

  if (config.forceAllInThreshold > 100000) {
    return "Force all-in threshold is too large";
  }

  return null;
}

export default defineComponent({
  setup() {
    const store = useStore();
    const config = useConfigStore();
    const tmpConfig = useTmpConfigStore();

    const numThreads = ref(navigator.hardwareConcurrency || 1);
    const targetExploitability = ref(0.3);
    const maxIterations = ref(1000);

    const isTreeBuilding = ref(false);
    const isTreeBuilt = ref(false);
    const treeStatus = ref("Module not loaded");
    const availableMemory = ref(0);
    const memoryUsage = ref(0);
    const memoryUsageCompressed = ref(0);
    const isCompressionEnabled = ref(false);
    const terminateFlag = ref(false);
    const pauseFlag = ref(false);
    const currentIteration = ref(-1);
    const exploitability = ref(Number.POSITIVE_INFINITY);
    const elapsedTimeMs = ref(-1);

    let startTime = 0;
    let exploitabilityUpdated = false;

    const iterationText = computed(() => {
      if (currentIteration.value === -1) {
        return "Allocating memory...";
      } else {
        return `Iteration: ${currentIteration.value}`;
      }
    });

    const exploitabilityText = computed(() => {
      if (!Number.isFinite(exploitability.value)) {
        return "";
      } else {
        const valueText = exploitability.value.toFixed(2);
        const percent = (exploitability.value * 100) / config.startingPot;
        const percentText = `${percent.toFixed(2)}%`;
        return `Exploitability: ${valueText} (${percentText})`;
      }
    });

    const timeText = computed(() => {
      if (elapsedTimeMs.value === -1 || !store.isSolverFinished) {
        return "";
      } else {
        return `Time: ${(elapsedTimeMs.value / 1000).toFixed(2)}s`;
      }
    });

    const buildTree = async () => {
      isTreeBuilt.value = false;

      if (numThreads.value < 1 || numThreads.value % 1 !== 0) {
        treeStatus.value = "Error: Invalid number of threads";
        return;
      }

      if (numThreads.value > 64) {
        treeStatus.value = "Error: Too many threads";
        return;
      }

      const configError = checkConfig(config);
      if (configError !== null) {
        treeStatus.value = `Error: ${configError}`;
        return;
      }

      // needed for type inference
      if (
        config.oopFlopBet === null ||
        config.oopFlopRaise === null ||
        config.oopTurnBet === null ||
        config.oopTurnRaise === null ||
        config.oopRiverBet === null ||
        config.oopRiverRaise === null ||
        config.ipFlopBet === null ||
        config.ipFlopRaise === null ||
        config.ipTurnBet === null ||
        config.ipTurnRaise === null ||
        config.ipRiverBet === null ||
        config.ipRiverRaise === null
      ) {
        return;
      }

      saveConfigTmp();
      isTreeBuilding.value = true;
      treeStatus.value = "Building tree...";

      const errorString = await invoke("game_init", {
        payload: {
          num_threads: numThreads.value,
          board: tmpConfig.board,
          starting_pot: tmpConfig.startingPot,
          effective_stack: tmpConfig.effectiveStack,
          oop_flop_bet: tmpConfig.oopFlopBet,
          oop_flop_raise: tmpConfig.oopFlopRaise,
          oop_turn_bet: tmpConfig.oopTurnBet,
          oop_turn_raise: tmpConfig.oopTurnRaise,
          oop_river_bet: tmpConfig.oopRiverBet,
          oop_river_raise: tmpConfig.oopRiverRaise,
          ip_flop_bet: tmpConfig.ipFlopBet,
          ip_flop_raise: tmpConfig.ipFlopRaise,
          ip_turn_bet: tmpConfig.ipTurnBet,
          ip_turn_raise: tmpConfig.ipTurnRaise,
          ip_river_bet: tmpConfig.ipRiverBet,
          ip_river_raise: tmpConfig.ipRiverRaise,
          add_all_in_threshold: tmpConfig.addAllInThreshold / 100,
          force_all_in_threshold: tmpConfig.forceAllInThreshold / 100,
          adjust_last_two_bet_sizes: tmpConfig.adjustLastTwoBetSizes,
        },
      });

      if (errorString) {
        isTreeBuilding.value = false;
        treeStatus.value = "Error: " + errorString;
        return;
      }

      saveConfig();

      availableMemory.value = await invoke("available_memory");

      [memoryUsage.value, memoryUsageCompressed.value] = await invoke(
        "game_memory_usage"
      );

      if (memoryUsage.value <= 0.85 * availableMemory.value) {
        isCompressionEnabled.value = false;
      } else if (memoryUsageCompressed.value <= 0.95 * availableMemory.value) {
        isCompressionEnabled.value = true;
      }

      const threadText = `${numThreads.value} thread${
        numThreads.value === 1 ? "" : "s"
      }`;

      isTreeBuilding.value = false;
      isTreeBuilt.value = true;
      treeStatus.value = `Successfully built tree (${threadText})`;

      store.isSolverRunning = false;
      store.isSolverPaused = false;
      store.isSolverFinished = false;
    };

    const runSolver = async () => {
      terminateFlag.value = false;
      pauseFlag.value = false;
      currentIteration.value = -1;
      exploitability.value = Number.POSITIVE_INFINITY;
      elapsedTimeMs.value = -1;

      store.isSolverRunning = true;

      startTime = performance.now();

      await invoke("game_allocate_memory", {
        enableCompression: isCompressionEnabled.value,
      });

      currentIteration.value = 0;
      exploitabilityUpdated = false;
      await resumeSolver();
    };

    const resumeSolver = async () => {
      store.isSolverRunning = true;
      store.isSolverPaused = false;

      if (startTime === 0) {
        startTime = performance.now();
      }

      const target = (config.startingPot * targetExploitability.value) / 100;

      while (
        !terminateFlag.value &&
        currentIteration.value < maxIterations.value &&
        exploitability.value > target
      ) {
        if (pauseFlag.value) {
          const end = performance.now();
          elapsedTimeMs.value += end - startTime;
          startTime = 0;
          pauseFlag.value = false;
          store.isSolverRunning = false;
          store.isSolverPaused = true;
          return;
        }

        const payload = { currentIteration: currentIteration.value };
        await invoke("game_solve_step", payload);
        ++currentIteration.value;
        exploitabilityUpdated = false;

        if (currentIteration.value % 10 === 0) {
          exploitability.value = await invoke("game_exploitability");
          exploitability.value = Math.max(exploitability.value, 0);
          exploitabilityUpdated = true;
        }
      }

      if (!exploitabilityUpdated) {
        exploitability.value = Math.max(await invoke("game_exploitability"), 0);
      }

      store.isSolverRunning = false;
      store.isFinalizing = true;

      await invoke("game_finalize");

      store.isFinalizing = false;
      store.isSolverFinished = true;

      const end = performance.now();
      elapsedTimeMs.value += end - startTime;
    };

    return {
      store,
      numThreads,
      targetExploitability,
      maxIterations,
      isTreeBuilding,
      isTreeBuilt,
      treeStatus,
      availableMemory,
      memoryUsage,
      memoryUsageCompressed,
      isCompressionEnabled,
      terminateFlag,
      pauseFlag,
      iterationText,
      exploitabilityText,
      timeText,
      buildTree,
      runSolver,
      resumeSolver,
    };
  },
});
</script>

<style scoped>
.button {
  @apply rounded-lg shadow-sm px-3.5 py-1.5 text-white text-sm font-medium;
  @apply focus:outline-none focus:ring-4 disabled:opacity-40;
}

.button-blue {
  @apply bg-blue-600 hover:bg-blue-700 focus:ring-blue-300 disabled:bg-blue-600;
}

.button-red {
  @apply bg-red-600 hover:bg-red-700 focus:ring-red-300 disabled:bg-red-600;
}

.button-green {
  @apply bg-green-600 hover:bg-green-700 focus:ring-green-300 disabled:bg-green-600;
}
</style>
