// hologram-forge (ours, not upstream's; see VENDORED.md): Chat and Plan show the build screen's own chat
// section alone. Nothing here is a second chat: upstream's ChatPanel, its messages and its composer are the
// ones on screen in every mode, and they stay mounted when the mode changes, so the conversation, the draft
// and the scroll position survive the switch and Build only has to open the preview beside them. What this
// file adds is the decision (usePlainChatLayout), the suggestions above the composer, and the words of an empty
// conversation. Built only from the kit's shadcn/ui components; the decisions are hologram.plain-chat-data.mjs.
import { useEffect, useMemo, useRef, type ReactNode } from "react";
import { useAtomValue, useSetAtom } from "jotai";
import { selectAtom } from "jotai/utils";
import { useQuery } from "@tanstack/react-query";
import { chatInputValuesByIdAtom } from "@/atoms/chatAtoms";
import { selectedAppIdAtom } from "@/atoms/appAtoms";
import { Button } from "@/components/ui/button";
import { useChatMode } from "@/hooks/useChatMode";
import { useChatMessageCount } from "@/hooks/useChatMessages";
import { useChatStreamState } from "@/hooks/useChatStream";
import { isStreamActive } from "@/chat_stream/transition";
import { ipc } from "@/ipc/types";
import { cn } from "@/lib/utils";
import {
  PLAIN_CHAT_WORDS,
  isPlainChatMode,
  plainChatLayout,
  plainChatSuggestions,
  rememberChatMode,
  rememberedChatMode,
  showPlainChatSuggestions,
} from "../hologram.plain-chat-data.mjs";

/** The chat's layout, and its mode when that is a chat-only mode (Chat or Plan), else null. */
function usePlainMode(chatId: number | null | undefined) {
  const { selectedMode, isLoading } = useChatMode(chatId);
  // Read once per chat, before the first paint: the layout the chat was last seen in.
  const remembered = useMemo(() => (chatId == null ? null : rememberedChatMode(localStorage, chatId)), [chatId]);
  useEffect(() => {
    if (chatId != null && !isLoading) rememberChatMode(localStorage, chatId, selectedMode);
  }, [chatId, isLoading, selectedMode]);
  const layout = plainChatLayout({ chatId, isLoading, selectedMode, remembered });
  const mode = isLoading ? remembered : selectedMode;
  return { layout, mode: layout === "plain" && isPlainChatMode(mode) ? mode : null };
}

/** The chat's layout: "plain" in Chat and Plan, "build" in Build, "unknown" until a chat never seen is read. */
export function usePlainChatLayout(chatId: number | null | undefined): "plain" | "build" | "unknown" {
  return usePlainMode(chatId).layout;
}

/** The composer's placeholder: the mode's own words in Chat and Plan, upstream's in Build. */
export function usePlainChatPlaceholder(chatId: number | null | undefined, fallback: string): string {
  const { mode } = usePlainMode(chatId);
  return mode ? PLAIN_CHAT_WORDS[mode].placeholder : fallback;
}

// Whether the chat's app has rules: suggestions about "this app" are offered only when it does.
function useHasSpec(appId: number | null, enabled: boolean): boolean {
  const { data } = useQuery({
    queryKey: ["hologram", "plain-chat-spec", appId],
    queryFn: async () => {
      try {
        return (await ipc.app.readAppFile({ appId: appId!, filePath: "prism/spec.json" })).trim() !== "";
      } catch {
        return false;
      }
    },
    enabled: enabled && appId !== null,
    staleTime: 30_000,
  });
  return data === true;
}

/**
 * The composer with, in Chat and Plan, one row of suggestions directly above it. The row keeps its height
 * whether or not it shows chips, so the conversation never moves when they come and go; in Build the frame
 * draws nothing of its own. A chip writes its prompt into the composer and focuses it, as Home's idea chips
 * do: the words stay the user's to edit before they are sent.
 */
export function ChatComposer({ chatId, children }: { chatId?: number; children: ReactNode }) {
  const { mode } = usePlainMode(chatId);
  const appId = useAtomValue(selectedAppIdAtom);
  const hasSpec = useHasSpec(appId, mode !== null);
  const messageCount = useChatMessageCount(chatId);
  const isStreaming = isStreamActive(useChatStreamState(chatId) ?? { type: "idle" });
  const draft = useAtomValue(
    useMemo(() => selectAtom(chatInputValuesByIdAtom, (values) => (chatId == null ? "" : (values.get(chatId) ?? ""))), [chatId]),
  );
  const setDrafts = useSetAtom(chatInputValuesByIdAtom);
  const frame = useRef<HTMLDivElement>(null);
  const suggestions = plainChatSuggestions(mode, hasSpec);
  const shown = mode !== null && showPlainChatSuggestions({ isStreaming, messageCount, draft });

  const choose = (prompt: string) => {
    if (chatId == null) return;
    setDrafts((values) => new Map(values).set(chatId, prompt));
    requestAnimationFrame(() => frame.current?.querySelector<HTMLElement>('[contenteditable="true"]')?.focus());
  };

  return (
    <div ref={frame} data-slot="chat-composer" className={cn(!mode && "contents")}>
      {mode && (
        <div
          role="group"
          aria-label="Suggestions"
          data-slot="chat-suggestions"
          data-shown={shown ? "true" : "false"}
          className="flex h-12 items-center gap-2 overflow-x-auto px-2 [scrollbar-width:none] [&::-webkit-scrollbar]:hidden"
        >
          {shown &&
            suggestions.map((suggestion) => (
              <Button
                key={suggestion.label}
                type="button"
                variant="outline"
                size="sm"
                className="shrink-0 rounded-full font-normal text-muted-foreground hover:text-foreground"
                onClick={() => choose(suggestion.prompt)}
              >
                {suggestion.label}
              </Button>
            ))}
        </div>
      )}
      {children}
    </div>
  );
}

/** An empty conversation's words: the mode's heading and its read-only note in Chat and Plan, upstream's in Build. */
export function ChatEmptyWords({ chatId, fallback }: { chatId: number | null; fallback: string }) {
  const { mode } = usePlainMode(chatId);
  if (!mode) return <>{fallback}</>;
  const words = PLAIN_CHAT_WORDS[mode];
  return (
    <div data-slot="chat-empty" className="flex flex-col items-center gap-2 px-4 text-center">
      <h2 className="font-display text-2xl font-semibold text-foreground">{words.heading}</h2>
      <p className="text-sm text-muted-foreground">{words.note}</p>
    </div>
  );
}
