// FoundrySpecification.browserChatModeControlsBoundary.
import { useEffect, useRef, useState } from "react";
import { ChevronDownIcon } from "lucide-react";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { cn } from "@/lib/utils";
import { useRouter, useRouterState } from "@tanstack/react-router";
import { useChatMode } from "@/hooks/useChatMode";
import { useIsMac } from "@/hooks/useChatModeToggle";
import { useSettings } from "@/hooks/useSettings";
import type { ChatMode } from "@/lib/schemas";
import { modelSelectionScope } from "./hologram.model-selection.mjs";
import { chatModeCatalog } from "./hologram.chat-mode-data.mjs";

// What choosing each way of asking means, in the words a person would use.
const MEANS: Record<string, string> = {
  build: "Makes your app, change by change",
  ask: "Answers questions about it",
  plan: "Maps the steps before any work",
};

export function ChatModeSelector() {
  const router = useRouter(),
    state = useRouterState(),
    isMac = useIsMac();
  const scope = modelSelectionScope(
    state.location.pathname,
    router.options.basepath,
    state.location.search.id,
  );
  const { chat, selectedMode, setChatMode, isLoading, isUpdating } = useChatMode(
    scope.kind === "chat" ? scope.chatId : null,
  );
  const { settings, updateSettings } = useSettings();
  const [open, setOpen] = useState(false);
  const [saving, setSaving] = useState(false),
    [error, setError] = useState<string | null>(null);
  const busy = useRef(false);
  const unavailable =
    scope.kind === "unavailable" ||
    !settings ||
    (scope.kind === "chat" && (isLoading || !chat || isUpdating));
  const chosen = chatModeCatalog.modes.find((choice) => choice.id === selectedMode);
  const supported = !!chosen;
  const choose = async (mode: ChatMode) => {
    if (unavailable || busy.current || !chatModeCatalog.modes.some((choice) => choice.id === mode))
      return;
    busy.current = true;
    setSaving(true);
    setError(null);
    try {
      if (scope.kind === "chat") await setChatMode(mode);
      else if (scope.kind === "default")
        await updateSettings({ selectedChatMode: mode, defaultChatMode: mode });
    } catch {
      setError("The chat mode could not be saved. Try again.");
    } finally {
      busy.current = false;
      setSaving(false);
    }
  };
  useEffect(() => {
    const key = (event: KeyboardEvent) => {
      if (
        event.defaultPrevented ||
        event.key !== "." ||
        event.altKey ||
        event.shiftKey ||
        !(isMac ? event.metaKey && !event.ctrlKey : event.ctrlKey && !event.metaKey)
      )
        return;
      if (unavailable || busy.current) return;
      event.preventDefault();
      const choices = chatModeCatalog.modes,
        index = choices.findIndex((choice) => choice.id === selectedMode);
      void choose(choices[(index + 1) % choices.length].id);
    };
    window.addEventListener("keydown", key);
    return () => window.removeEventListener("keydown", key);
  });
  return (
    // The mode is chosen on a toggle of three; the select behind it stays the one control a reader and the
    // page's own checks see, so the toggle is theirs to read through and never a second voice.
    <div data-slot="chat-mode" className="relative flex flex-col gap-1">
      <select
        aria-label="Chat mode"
        data-testid="chat-mode-selector"
        tabIndex={-1}
        value={supported ? selectedMode : ""}
        disabled={unavailable || saving}
        onChange={(event) => void choose(event.target.value as ChatMode)}
        className="pointer-events-none absolute left-0 top-0 h-px w-px opacity-0"
      >
        {!supported && (
          <option value="" disabled>
            Unavailable mode
          </option>
        )}
        {chatModeCatalog.modes.map((choice) => (
          <option key={choice.id} value={choice.id}>
            {choice.label}
          </option>
        ))}
      </select>
      <DropdownMenu open={open} onOpenChange={setOpen}>
        <DropdownMenuTrigger
          data-slot="chat-mode-toggle"
          aria-hidden="true"
          tabIndex={-1}
          disabled={unavailable || saving}
          className="inline-flex h-8 cursor-pointer items-center gap-2 whitespace-nowrap rounded-full border border-border/70 bg-background/40 pl-3 pr-2 text-xs font-medium text-foreground/90 shadow-none transition-colors hover:border-border hover:bg-muted/60 hover:text-foreground focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring data-[state=open]:bg-muted/60 disabled:cursor-default disabled:opacity-50"
        >
          <span className="truncate">{chosen ? chosen.label : "Mode"}</span>
          <ChevronDownIcon className="size-3.5 shrink-0 text-muted-foreground" aria-hidden="true" />
        </DropdownMenuTrigger>
        <DropdownMenuContent className="w-72 p-1.5" align="end">
          {chatModeCatalog.modes.map((choice) => (
            <DropdownMenuItem
              key={choice.id}
              onSelect={() => void choose(choice.id)}
              className={cn("cursor-pointer gap-3 rounded-lg px-2.5 py-2", selectedMode === choice.id && "bg-accent")}
            >
              <span className="flex min-w-0 flex-1 flex-col gap-0.5">
                <span className={cn("truncate text-[13px] leading-5", selectedMode === choice.id ? "font-medium" : "font-normal")}>
                  {choice.label}
                </span>
                <span className="truncate text-[11px] leading-4 text-muted-foreground">{MEANS[choice.id] ?? ""}</span>
              </span>
            </DropdownMenuItem>
          ))}
        </DropdownMenuContent>
      </DropdownMenu>
      {error && (
        <span role="alert" className="text-xs">
          {error}
        </span>
      )}
    </div>
  );
}
