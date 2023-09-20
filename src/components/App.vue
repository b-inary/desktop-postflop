<template>
  <div class="min-w-[1080px]" :style="{ height: clientHeight + 'px' }">
    <NavBar />

    <div
      v-show="store.navView === 'solver'"
      class="flex w-full mx-auto max-w-screen-xl"
      style="height: calc(100% - 2.5rem)"
    >
      <SideBar style="height: calc(100% - 2rem)" />

      <div
        class="flex-grow my-4 px-6 pt-2 overflow-y-auto"
        style="height: calc(100% - 2rem)"
      >
        <div class="flex">
          <div
            :class="
              'mb-5 pl-2 pr-3 pb-0.5 text-lg font-bold border-l-8 border-b-2 ' +
              'border-blue-600 rounded rounded-br-none'
            "
          >
            {{ header }}
          </div>
        </div>

        <div v-show="store.sideView === 'hand-importer'">
          <HandImporter />
        </div>
        <div v-show="store.sideView === 'oop-range'">
          <RangeEditor :player="0" />
        </div>
        <div v-show="store.sideView === 'ip-range'">
          <RangeEditor :player="1" />
        </div>
        <div v-show="store.sideView === 'board'">
          <BoardSelector />
        </div>
        <div v-show="store.sideView === 'tree-config'">
          <TreeConfig />
        </div>
        <div v-show="store.sideView === 'bunching'">
          <BunchingEffect />
        </div>
        <div v-show="store.sideView === 'run-solver'">
          <RunSolver />
        </div>
        <div v-if="store.sideView === 'about'">
          <AboutPage />
        </div>
      </div>
    </div>

    <div
      v-show="store.navView === 'results'"
      style="height: calc(max(100%, 720px) - 2.5rem)"
    >
      <ResultViewer />
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from "vue";
import { useStore } from "../store";

import NavBar from "./NavBar.vue";
import SideBar from "./SideBar.vue";
import RangeEditor from "./RangeEditor.vue";
import BoardSelector from "./BoardSelector.vue";
import TreeConfig from "./TreeConfig.vue";
import BunchingEffect from "./BunchingEffect.vue";
import RunSolver from "./RunSolver.vue";
import AboutPage from "./AboutPage.vue";
import ResultViewer from "./ResultViewer.vue";
import HandImporter from "./HandImporter.vue";

const store = useStore();
const header = computed(() => store.headers[store.sideView].join(" > "));

const clientHeight = ref(0);
const updateClientHeight = () => {
  clientHeight.value = document.documentElement.clientHeight - 0.01;
};

updateClientHeight();
window.addEventListener("resize", updateClientHeight);
</script>
