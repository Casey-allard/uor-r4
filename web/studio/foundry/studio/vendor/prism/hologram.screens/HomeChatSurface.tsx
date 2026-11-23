// UOR-R4 Foundry Home Chat & Build Mode surface.
// Keeps visitors chatting in place on `/` (`Chat` mode) with full read-only
// Foundry context until they switch to `Build App` or click "Build this in Studio".
import { useCallback, useEffect, useRef, useState } from "react";
import {
  ArrowRight,
  Cpu,
  Hammer,
  Loader2,
  MessageSquare,
  Square,
  Trash2,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";

export type HomeMode = "chat" | "build";

export interface HomeChatMessage {
  id: number;
  role: "user" | "assistant";
  content: string;
  model?: string;
  createdAt?: string;
  toolExecutions?: number;
}

export interface HomeHydrationProgress {
  modelId?: string;
  phase?: string;
  percent?: number;
  packsDone?: number;
  packsTotal?: number;
  piecesDone?: number;
  piecesTotal?: number;
  cached?: boolean;
  cacheBackend?: string | null;
  recoveredPieces?: unknown[];
  error?: string | null;
}

type ElectronIpc = {
  invoke: <T = unknown>(channel: string, ...args: unknown[]) => Promise<T>;
  on?: (channel: string, fn: (payload: unknown) => void) => (() => void) | void;
  removeListener?: (channel: string, fn: (payload: unknown) => void) => void;
};

function getIpc(): ElectronIpc | null {
  return (
    (window as unknown as { electron?: { ipcRenderer?: ElectronIpc } }).electron
      ?.ipcRenderer ?? null
  );
}

export function useHomeChat({
  onPromoted,
}: {
  onPromoted?: (appId: number, chatId: number) => Promise<void> | void;
}) {
  const [messages, setMessages] = useState<HomeChatMessage[]>([]);
  const [sending, setSending] = useState(false);
  const [promoting, setPromoting] = useState(false);
  const promotingRef = useRef(false);
  const [error, setError] = useState<string | null>(null);
  const [hydration, setHydration] = useState<HomeHydrationProgress | null>(null);

  useEffect(() => {
    const ipc = getIpc();
    if (!ipc) return;
    let active = true;
    void ipc
      .invoke<{ messages?: HomeChatMessage[] }>("home-chat:history")
      .then((res) => {
        if (active && Array.isArray(res?.messages)) {
          setMessages(res.messages);
        }
      })
      .catch(() => {});

    const onChunk = (payload: unknown) => {
      if (!active || !payload || typeof payload !== "object") return;
      const list = (payload as { messages?: HomeChatMessage[] }).messages;
      if (Array.isArray(list)) setMessages(list);
    };
    const onEnd = (payload: unknown) => {
      if (!active || !payload || typeof payload !== "object") return;
      const data = payload as {
        messages?: HomeChatMessage[];
        error?: string;
      };
      if (Array.isArray(data.messages)) setMessages(data.messages);
      if (data.error) setError(data.error);
      setSending(false);
    };
    const onProgress = (payload: unknown) => {
      if (!active || !payload || typeof payload !== "object") return;
      setHydration(payload as HomeHydrationProgress);
    };

    const offChunk = ipc.on?.("home-chat:chunk", onChunk);
    const offEnd = ipc.on?.("home-chat:end", onEnd);
    const offProg = ipc.on?.("uor-r4:progress", onProgress);
    return () => {
      active = false;
      if (typeof offChunk === "function") offChunk();
      else ipc.removeListener?.("home-chat:chunk", onChunk);
      if (typeof offEnd === "function") offEnd();
      else ipc.removeListener?.("home-chat:end", onEnd);
      if (typeof offProg === "function") offProg();
      else ipc.removeListener?.("uor-r4:progress", onProgress);
    };
  }, []);

  const sendMessage = useCallback(async (prompt: string) => {
    const ipc = getIpc();
    const clean = prompt.trim();
    if (!ipc || !clean) return false;
    setError(null);
    setSending(true);
    try {
      const res = await ipc.invoke<{
        messages?: HomeChatMessage[];
        error?: string;
      }>("home-chat:send", { prompt: clean });
      if (Array.isArray(res?.messages)) setMessages(res.messages);
      if (res?.error) setError(res.error);
      return true;
    } catch (err) {
      setError(String((err as Error)?.message || err));
      return false;
    } finally {
      setSending(false);
    }
  }, []);

  const cancel = useCallback(async () => {
    const ipc = getIpc();
    if (!ipc) return;
    await ipc.invoke("home-chat:cancel").catch(() => {});
  }, []);

  const clear = useCallback(async () => {
    const ipc = getIpc();
    if (!ipc) return;
    setError(null);
    const res = await ipc
      .invoke<{ messages?: HomeChatMessage[] }>("home-chat:clear")
      .catch(() => ({ messages: [] }));
    setMessages(Array.isArray(res?.messages) ? res.messages : []);
  }, []);

  const promoteToStudio = useCallback(async () => {
    const ipc = getIpc();
    if (!ipc || promotingRef.current) return;
    promotingRef.current = true;
    setError(null);
    setPromoting(true);
    try {
      const res = await ipc.invoke<{ appId: number; chatId: number }>(
        "home-chat:promote-to-studio",
        { autoBuild: true },
      );
      await onPromoted?.(res.appId, res.chatId);
    } catch (err) {
      setError(String((err as Error)?.message || err));
    } finally {
      promotingRef.current = false;
      setPromoting(false);
    }
  }, [onPromoted]);

  return {
    messages,
    sending,
    promoting,
    error,
    hydration,
    sendMessage,
    cancel,
    clear,
    promoteToStudio,
  };
}

export function HomeModeToggle({
  mode,
  onChange,
  disabled = false,
}: {
  mode: HomeMode;
  onChange: (next: HomeMode) => void;
  disabled?: boolean;
}) {
  return (
    <div
      className="inline-flex items-center gap-1 rounded-full border border-border/80 bg-background/80 p-1 shadow-xs backdrop-blur"
      role="tablist"
      aria-label="Home interaction mode"
      data-slot="home-mode-toggle"
    >
      <button
        type="button"
        role="tab"
        aria-selected={mode === "chat"}
        disabled={disabled}
        data-testid="home-mode-chat"
        onClick={() => onChange("chat")}
        className={cn(
          "inline-flex cursor-pointer items-center gap-1.5 rounded-full px-3.5 py-1 text-xs font-medium transition-colors disabled:opacity-50",
          mode === "chat"
            ? "bg-foreground text-background"
            : "text-muted-foreground hover:text-foreground",
        )}
      >
        <MessageSquare className="size-3.5" aria-hidden="true" />
        <span>Chat</span>
      </button>
      <button
        type="button"
        role="tab"
        aria-selected={mode === "build"}
        disabled={disabled}
        data-testid="home-mode-build"
        onClick={() => onChange("build")}
        className={cn(
          "inline-flex cursor-pointer items-center gap-1.5 rounded-full px-3.5 py-1 text-xs font-medium transition-colors disabled:opacity-50",
          mode === "build"
            ? "bg-foreground text-background"
            : "text-muted-foreground hover:text-foreground",
        )}
      >
        <Hammer className="size-3.5" aria-hidden="true" />
        <span>Build App</span>
      </button>
    </div>
  );
}

export function HomeChatThread({
  messages,
  sending,
  promoting,
  error,
  hydration,
  onCancel,
  onClear,
  onPromote,
}: {
  messages: HomeChatMessage[];
  sending: boolean;
  promoting: boolean;
  error: string | null;
  hydration: HomeHydrationProgress | null;
  onCancel: () => void;
  onClear: () => void;
  onPromote: () => void;
}) {
  const scrollRef = useRef<HTMLDivElement | null>(null);
  useEffect(() => {
    if (scrollRef.current) {
      scrollRef.current.scrollTop = scrollRef.current.scrollHeight;
    }
  }, [messages]);

  const showHydrating =
    Boolean(hydration?.phase) &&
    hydration?.phase !== "idle" &&
    hydration?.phase !== "ready";

  if (messages.length === 0 && !sending && !showHydrating && !error) {
    return null;
  }

  return (
    <div
      className="mb-3 rounded-2xl border border-border/80 bg-card/85 p-3 shadow-xs backdrop-blur"
      data-testid="home-chat-thread"
    >
      <div className="mb-2 flex flex-wrap items-center justify-between gap-2 border-b border-border/60 pb-2">
        <div className="flex items-center gap-2 text-xs text-muted-foreground">
          <Cpu className="size-3.5 text-primary" aria-hidden="true" />
          <span className="font-medium text-foreground">
            UOR-R4 Home Chat
          </span>
          {showHydrating && (
            <span
              className="inline-flex items-center gap-1 rounded-full bg-muted px-2 py-0.5 text-[11px]"
              data-testid="home-hydration-progress"
            >
              <Loader2 className="size-3 animate-spin" aria-hidden="true" />
              <span>
                Loading weights {hydration?.percent ?? 0}% (
                {hydration?.packsDone ?? 0}/{hydration?.packsTotal ?? 10} packs)
              </span>
            </span>
          )}
        </div>
        <div className="flex items-center gap-1.5">
          {sending && (
            <Button
              type="button"
              variant="outline"
              size="sm"
              data-testid="home-chat-cancel"
              onClick={onCancel}
              className="h-7 gap-1 rounded-full px-2.5 text-xs"
            >
              <Square className="size-3" aria-hidden="true" />
              <span>Stop</span>
            </Button>
          )}
          {messages.length > 0 && !sending && (
            <Button
              type="button"
              variant="ghost"
              size="sm"
              data-testid="home-chat-clear"
              onClick={onClear}
              className="h-7 gap-1 rounded-full px-2.5 text-xs text-muted-foreground"
            >
              <Trash2 className="size-3" aria-hidden="true" />
              <span>Clear</span>
            </Button>
          )}
          {messages.length > 0 && (
            <Button
              type="button"
              size="sm"
              disabled={promoting}
              data-testid="home-promote-to-studio"
              onClick={onPromote}
              className="h-7 gap-1.5 rounded-full px-3 text-xs font-medium"
            >
              {promoting ? (
                <Loader2 className="size-3.5 animate-spin" aria-hidden="true" />
              ) : (
                <ArrowRight className="size-3.5" aria-hidden="true" />
              )}
              <span>Build this in Studio →</span>
            </Button>
          )}
        </div>
      </div>

      {error && (
        <div
          role="alert"
          className="mb-2 rounded-lg border border-destructive/30 bg-destructive/10 px-3 py-1.5 text-xs text-destructive"
        >
          {error}
        </div>
      )}

      <div
        ref={scrollRef}
        className="max-h-72 space-y-2.5 overflow-y-auto pr-1 text-sm"
      >
        {messages.map((m) => (
          <div
            key={m.id}
            data-role={m.role}
            className={cn(
              "rounded-xl px-3 py-2 leading-relaxed",
              m.role === "user"
                ? "ml-8 bg-primary/10 text-foreground"
                : "mr-4 border border-border/60 bg-background/80 text-foreground",
            )}
          >
            <div className="mb-0.5 flex items-center justify-between text-[11px] font-medium text-muted-foreground">
              <span>{m.role === "user" ? "You" : "UOR-R4 Foundry"}</span>
              {m.toolExecutions ? (
                <span>{m.toolExecutions} tool call(s)</span>
              ) : null}
            </div>
            <div className="whitespace-pre-wrap break-words">
              {m.content || (m.role === "assistant" && sending ? "Thinking…" : "")}
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}
