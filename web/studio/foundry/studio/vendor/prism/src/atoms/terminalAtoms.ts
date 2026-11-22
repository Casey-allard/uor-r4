import { atom } from "jotai";
import { atomWithStorage } from "jotai/utils";

export const terminalOpenByChatIdAtom = atom<Map<number, boolean>>(new Map());

export const terminalFontSizeAtom = atomWithStorage<number>(
  "prism:terminal-font-size",
  14,
);
