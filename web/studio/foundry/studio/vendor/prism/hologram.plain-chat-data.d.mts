export type PlainChatMode = "ask" | "plan";
export type PlainChatSuggestion = { readonly label: string; readonly prompt: string };
export type PlainChatWords = { readonly heading: string; readonly note: string; readonly placeholder: string };
export const PLAIN_CHAT_MODES: readonly PlainChatMode[];
export function isPlainChatMode(mode: unknown): mode is PlainChatMode;
export const PLAIN_CHAT_WORDS: Readonly<Record<PlainChatMode, PlainChatWords>>;
export const PLAIN_CHAT_SUGGESTIONS: Readonly<
  Record<PlainChatMode, { readonly app: readonly PlainChatSuggestion[]; readonly general: readonly PlainChatSuggestion[] }>
>;
export function plainChatSuggestions(mode: unknown, hasSpec: boolean): readonly PlainChatSuggestion[];
export function showPlainChatSuggestions(state: { isStreaming: boolean; messageCount: number; draft: string }): boolean;
export function rememberedChatMode(storage: Pick<Storage, "getItem">, chatId: number): string | null;
export function rememberChatMode(storage: Pick<Storage, "setItem">, chatId: number, mode: unknown): void;
export function plainChatLayout(state: {
  chatId: number | null | undefined;
  isLoading: boolean;
  selectedMode: unknown;
  remembered: string | null;
}): "plain" | "build" | "unknown";
