<template>
  <div class="mt-1 w-[52rem]">
    <textarea
      v-model="importText"
      :class='"block w-full h-[36rem] px-2 py-1 rounded-lg textarea text-sm font-mono " + (importTextError ? "textarea-error" : "")'
      @change="onImportTextChanged"
      spellcheck="false"
    />

    <div class="w-full">
      <button class="button-base button-blue mt-3 mr-3" @click="importHand">
        Import
      </button>
      <span v-if="importDoneText" class="mt-1">{{ importDoneText }}</span>
      <span v-if="importTextError" class="mt-1 text-red-500">
        {{ 'Error: ' + importTextError }}
      </span>
    </div>

    <div class="mt-2">
      Note: If the import has missing values, the current value will be used.
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch } from "vue";
import { Config, configKeys, useConfigStore, useStore } from "../store";
import { cardText, parseCardString, Position, Result } from "../utils";
import { setRange, validateRange } from "../range-utils";
import * as invokes from "../invokes";

type NonValidatedHandJSON = Record<string, any>;
type HandJSON = {
  oopRange: string;
  ipRange: string;
  config: Omit<Config, "board"> & { board: string[] };
};

const INVALID_BOARD_ERROR = `Invalid '.config.board'. Set board cards manually for an example.`;

const config = useConfigStore();
const store = useStore();

const importText = ref("{\n  \n}");
const importTextError = ref("");
const importDoneText = ref("");

watch(
  () => store.ranges,
  () => generateImportText(),
  { deep: true }
);
watch(
  () => Object.values(config),
  () => generateImportText(),
  { deep: true }
);

const generateImportText = async () => {
  const configObj = Object.fromEntries(
    Object.entries(config).filter(
      ([key, _value]) => configKeys.indexOf(key) !== -1
    )
  );

  const importObj = {
    oopRange: await invokes.rangeToString(Position.OOP),
    ipRange: await invokes.rangeToString(Position.IP),
    config: {
      ...configObj,
      board: config.board.map((cardId) => {
        const { rank, suitLetter } = cardText(cardId);
        return rank + suitLetter;
      }),
    },
  };

  importText.value = JSON.stringify(importObj, null, 2);
  importTextError.value = '';
  importDoneText.value = '';
};

const onImportTextChanged = () => {
  importDoneText.value = "";
  validateImportTextAndDisplayError();
};

const validateConfigPrimitives = (importConfig: any): Result => {
  if (typeof importConfig !== "object")
    return {
      success: false,
      error: `Expected '.config' to be an object but got ${typeof importConfig}`,
    };

  for (const key of configKeys) {
    const newValue = importConfig[key];
    const existingValue = (config as any)[key];
    if (existingValue === null || existingValue === undefined) continue;
    if (typeof existingValue === typeof newValue) continue;

    if (key === "board") return { success: false, error: INVALID_BOARD_ERROR };
    else
      return {
        success: false,
        error: `Expected '.config.${key}' to be ${typeof importConfig} but got ${typeof newValue}`,
      };
  }

  return { success: true };
};

const validateBoard = (board: any): Result => {
  if (!Array.isArray(board))
    return { success: false, error: INVALID_BOARD_ERROR };

  for (const i in board) {
    const card = board[i];
    if (typeof card !== "string") {
      return { success: false, error: INVALID_BOARD_ERROR };
    }

    const parsedCard = parseCardString(card);
    if (parsedCard === null) {
      return { success: false, error: INVALID_BOARD_ERROR };
    }
  }

  return { success: true };
};

const parseJson = (
  json: string
): Result<{ json?: NonValidatedHandJSON }> => {
  try {
    return { success: true, json: JSON.parse(json) };
  } catch (e) {
    const message = (e as Error).message;
    return { success: false, error: message };
  }
};

const validateImportTextAndDisplayError = (): Result<{
  json?: HandJSON;
}> => {
  importTextError.value = "";

  const validation = validateImportText();
  if (validation.success) return validation;

  importTextError.value = validation.error as string;
  return validation;
};

const validateImportText = (): Result<{ json?: HandJSON }> => {
  const parseValidation = parseJson(importText.value);
  if (!parseValidation.success) return parseValidation;

  const importJson = parseValidation.json;
  if (typeof importJson !== "object")
    return { success: false, error: "Not a valid JSON object" };

  const validateFns: (() => Result)[] = [
    () => validateConfigPrimitives(config),
    () => validateBoard(importJson.config?.board),
    () => validateRange(importJson.oopRange, "oopRange"),
    () => validateRange(importJson.ipRange, "ipRange"),
  ];
  for (const validate of validateFns) {
    const validation = validate();
    if (!validation.success) return validation;
  }

  importJson.config.board = importJson.config.board.map(parseCardString);

  return { success: true, json: importJson as HandJSON };
};

const importHand = async () => {
  const validation = validateImportTextAndDisplayError();
  if (!validation.success) return;
  const importJson = validation.json as HandJSON;

  for (const key in importJson.config) {
    const newValue = (importJson.config as Record<string, any>)[key];
    (config as any)[key] = newValue;
  }

  await setRange(Position.OOP, importJson.oopRange, store);
  await setRange(Position.IP, importJson.ipRange, store);

  importDoneText.value = "Done!";
};

generateImportText();
</script>
