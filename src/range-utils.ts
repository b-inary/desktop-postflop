import { Position, Result, trimRegex, comboPat } from "./utils";
import * as invokes from "./invokes";
import { useStore } from "./store";

type Store = ReturnType<typeof useStore>;

export const weightPat = "(?:(?:[01](\\.\\d*)?)|(?:\\.\\d+))";
export const rangeRegex = new RegExp(
  `^(?<range>${comboPat}(?:\\+|(?:-${comboPat}))?)(?::(?<weight>${weightPat}))?$`
);

const trimRange = (rangeString: string) =>
  rangeString.replace(trimRegex, "$1").trim();

export const validateRange = (
  rangeString: any,
  keyForError = "range"
): Result => {
  if (typeof rangeString !== "string")
    return {
      success: false,
      error: `Expected '${keyForError}' to be a string but got '${typeof rangeString}`,
    };

  const trimmed = trimRange(rangeString);
  const ranges = trimmed.split(",");

  if (ranges[ranges.length - 1] === "") ranges.pop();

  for (const range of ranges) {
    if (!rangeRegex.test(range)) {
      return {
        success: false,
        error: `'${range}' is not a valid range. For an example of a valid range, assign a range manually.`,
      };
    }
  }

  return { success: true };
};

export const setRange = async (
  player: Position,
  rangeString: string,
  store: Store
): Promise<Result> => {
  const trimmed = rangeString.replace(trimRegex, "$1").trim();
  const validation = validateRange(rangeString);
  if (!validation.success) return validation;

  const error = await invokes.rangeFromString(player, trimmed);
  if (error) return { success: false, error };

  await loadRangeToStore(player, store);

  return { success: true };
};

const loadRangeToStore = async (player: number, store: Store) => {
  const weights = await invokes.rangeGetWeights(player);
  for (let i = 0; i < 13 * 13; ++i) {
    store.ranges[player][i] = weights[i] * 100;
  }
};
