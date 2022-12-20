import { invoke } from "@tauri-apps/api";
import { Results, ChanceReports } from "./result-types";

export const memory = async (): Promise<number[]> => {
  return await invoke("memory");
};

/* Ranges */

export const rangeClear = async (player: number) => {
  await invoke("range_clear", { player });
};

export const rangeUpdate = async (
  player: number,
  row: number,
  col: number,
  weight: number
) => {
  await invoke("range_update", { player, row, col, weight });
};

export const rangeFromString = async (
  player: number,
  str: string
): Promise<string | null> => {
  return await invoke("range_from_string", { player, str });
};

export const rangeToString = async (player: number): Promise<string> => {
  return await invoke("range_to_string", { player });
};

export const rangeGetWeights = async (player: number): Promise<number[]> => {
  return await invoke("range_get_weights", { player });
};

export const rangeRawData = async (player: number): Promise<number[]> => {
  return await invoke("range_raw_data", { player });
};

/* Action Tree */

export const treeNew = async (
  boardLen: number,
  startingPot: number,
  effectiveStack: number,
  donkOption: boolean,
  oopFlopBet: string,
  oopFlopRaise: string,
  oopTurnBet: string,
  oopTurnRaise: string,
  oopTurnDonk: string,
  oopRiverBet: string,
  oopRiverRaise: string,
  oopRiverDonk: string,
  ipFlopBet: string,
  ipFlopRaise: string,
  ipTurnBet: string,
  ipTurnRaise: string,
  ipRiverBet: string,
  ipRiverRaise: string,
  addAllinThreshold: number,
  forceAllinThreshold: number,
  mergingThreshold: number,
  addedLines: string,
  removedLines: string
) => {
  await invoke("tree_new", {
    boardLen,
    startingPot,
    effectiveStack,
    donkOption,
    oopFlopBet,
    oopFlopRaise,
    oopTurnBet,
    oopTurnRaise,
    oopTurnDonk,
    oopRiverBet,
    oopRiverRaise,
    oopRiverDonk,
    ipFlopBet,
    ipFlopRaise,
    ipTurnBet,
    ipTurnRaise,
    ipRiverBet,
    ipRiverRaise,
    addAllinThreshold,
    forceAllinThreshold,
    mergingThreshold,
    addedLines,
    removedLines,
  });
};

export const treeAddedLines = async (): Promise<string> => {
  return await invoke("tree_added_lines");
};

export const treeRemovedLines = async (): Promise<string> => {
  return await invoke("tree_removed_lines");
};

export const treeInvalidTerminals = async (): Promise<string> => {
  return await invoke("tree_invalid_terminals");
};

export const treeActions = async (): Promise<string[]> => {
  return await invoke("tree_actions");
};

export const treeIsTerminalNode = async (): Promise<boolean> => {
  return await invoke("tree_is_terminal_node");
};

export const treeIsChanceNode = async (): Promise<boolean> => {
  return await invoke("tree_is_chance_node");
};

export const treeBackToRoot = async () => {
  await invoke("tree_back_to_root");
};

export const treeApplyHistory = async (line: string[]) => {
  await invoke("tree_apply_history", { line });
};

export const treePlay = async (action: string): Promise<number> => {
  return await invoke("tree_play", { action });
};

export const treeTotalBetAmount = async (): Promise<number[]> => {
  return await invoke("tree_total_bet_amount");
};

export const treeAddBetAction = async (amount: number, isRaise: boolean) => {
  await invoke("tree_add_bet_action", { amount, isRaise });
};

export const treeRemoveCurrentNode = async () => {
  await invoke("tree_remove_current_node");
};

export const treeDeleteAddedLine = async (line: string) => {
  await invoke("tree_delete_added_line", { line });
};

export const treeDeleteRemovedLine = async (line: string) => {
  await invoke("tree_delete_removed_line", { line });
};

/* Game */

export const gameInit = async (
  numThreads: number,
  board: number[],
  startingPot: number,
  effectiveStack: number,
  rakeRate: number,
  rakeCap: number,
  donkOption: boolean,
  oopFlopBet: string,
  oopFlopRaise: string,
  oopTurnBet: string,
  oopTurnRaise: string,
  oopTurnDonk: string,
  oopRiverBet: string,
  oopRiverRaise: string,
  oopRiverDonk: string,
  ipFlopBet: string,
  ipFlopRaise: string,
  ipTurnBet: string,
  ipTurnRaise: string,
  ipRiverBet: string,
  ipRiverRaise: string,
  addAllinThreshold: number,
  forceAllinThreshold: number,
  mergingThreshold: number,
  addedLines: string,
  removedLines: string
): Promise<string | null> => {
  return await invoke("game_init", {
    numThreads,
    board,
    startingPot,
    effectiveStack,
    rakeRate,
    rakeCap,
    donkOption,
    oopFlopBet,
    oopFlopRaise,
    oopTurnBet,
    oopTurnRaise,
    oopTurnDonk,
    oopRiverBet,
    oopRiverRaise,
    oopRiverDonk,
    ipFlopBet,
    ipFlopRaise,
    ipTurnBet,
    ipTurnRaise,
    ipRiverBet,
    ipRiverRaise,
    addAllinThreshold,
    forceAllinThreshold,
    mergingThreshold,
    addedLines,
    removedLines,
  });
};

export const gamePrivateCards = async (): Promise<number[][]> => {
  return await invoke("game_private_cards");
};

export const gameMemoryUsage = async (): Promise<number[]> => {
  return await invoke("game_memory_usage");
};

export const gameAllocateMemory = async (enableCompression: boolean) => {
  await invoke("game_allocate_memory", { enableCompression });
};

export const gameSolveStep = async (currentIteration: number) => {
  await invoke("game_solve_step", { currentIteration });
};

export const gameExploitability = async (): Promise<number> => {
  return await invoke("game_exploitability");
};

export const gameFinalize = async () => {
  await invoke("game_finalize");
};

export const gameApplyHistory = async (history: number[]) => {
  await invoke("game_apply_history", { history });
};

export const gameTotalBetAmount = async (
  append: number[]
): Promise<number[]> => {
  return await invoke("game_total_bet_amount", { append });
};

export const gameActionsAfter = async (append: number[]): Promise<string[]> => {
  return await invoke("game_actions_after", { append });
};

export const gamePossibleCards = async (append: number[]): Promise<bigint> => {
  return BigInt(await invoke("game_possible_cards", { append }));
};

type ResultsResponse = {
  current_player: "oop" | "ip" | "chance" | "terminal";
  num_actions: number;
  is_empty: number;
  eqr_base: number[];
  weights: number[][];
  normalizer: number[][];
  equity: number[][];
  ev: number[][];
  eqr: number[][];
  strategy: number[];
  action_ev: number[];
};

export const gameGetResults = async (): Promise<Results> => {
  const results: ResultsResponse = await invoke("game_get_results");
  return {
    currentPlayer: results.current_player,
    numActions: results.num_actions,
    isEmpty: results.is_empty,
    eqrBase: results.eqr_base,
    weights: results.weights,
    normalizer: results.normalizer,
    equity: results.equity,
    ev: results.ev,
    eqr: results.eqr,
    strategy: results.strategy,
    actionEv: results.action_ev,
  };
};

type ChanceReportsResponse = {
  num_actions: number;
  status: number[];
  combos: number[][];
  equity: number[][];
  ev: number[][];
  eqr: number[][];
  strategy: number[];
};

export const gameGetChanceReports = async (
  append: number[],
  currentPlayer: "oop" | "ip" | "terminal",
  numActions: number
): Promise<ChanceReports> => {
  const reports: ChanceReportsResponse = await invoke(
    "game_get_chance_reports",
    { append, numActions }
  );
  return {
    currentPlayer,
    numActions,
    status: reports.status,
    combos: reports.combos,
    equity: reports.equity,
    ev: reports.ev,
    eqr: reports.eqr,
    strategy: reports.strategy,
  };
};
