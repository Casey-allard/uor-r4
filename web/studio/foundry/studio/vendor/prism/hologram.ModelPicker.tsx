// UOR-R4 Foundry model picker: offers the built-in pure-integer UOR-R4 Qwen3-0.6B
// Foundry model plus any user-configured OpenAI-compatible Custom AI API endpoints.
// Zero M1 stub models and zero local weight-file upload controls.
//
// FoundrySpecification.chatModelPickerScopeBoundary: a mounted chat owns its selection;
// Home owns the default used by future chats.
import { ChevronDownIcon, CpuIcon, GlobeIcon, PlusCircleIcon } from "lucide-react";
import { useEffect, useRef, useState } from "react";
import { useRouter, useRouterState } from "@tanstack/react-router";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { useChatMode } from "@/hooks/useChatMode";
import { useLanguageModelProviders } from "@/hooks/useLanguageModelProviders";
import { useLanguageModelsByProviders } from "@/hooks/useLanguageModelsByProviders";
import { useSettings } from "@/hooks/useSettings";
import { createModelSelection } from "@/lib/modelEffort";
import type { LanguageModel } from "@/ipc/types";
import type { LargeLanguageModel } from "@/lib/schemas";
import { cn } from "@/lib/utils";

import { modelSelectionScope } from "./hologram.model-selection.mjs";

type ProgressSnapshot = {
  modelId?: string;
  phase?: string;
  percent?: number;
  packsDone?: number;
  packsTotal?: number;
  cached?: boolean;
  cacheBackend?: string | null;
  recoveredPieces?: unknown[];
  error?: string | null;
};

const ORDER = ["uor-r4", "custom-api"];
const WHERE: Record<string, string> = { "uor-r4": "WASM", "custom-api": "API" };
const MEANS: Record<string, string> = {
  "uor-r4": "Pure-integer UOR-R4 WASM on this device",
  "custom-api": "User-configured OpenAI-compatible AI API",
};

const Tag = ({ provider }: { provider: string }) => (
  <span className="inline-flex w-[66px] shrink-0 items-center justify-center gap-1 rounded-full border border-border/60 py-0.5 text-[11px] font-normal leading-4 text-muted-foreground">
    {provider === "custom-api" ? (
      <GlobeIcon className="size-3 shrink-0" aria-hidden="true" />
    ) : (
      <CpuIcon className="size-3 shrink-0" aria-hidden="true" />
    )}
    {WHERE[provider] || "WASM"}
  </span>
);

const Pill = ({ where }: { where: string }) => (
  <span className="shrink-0 rounded-full border border-border/50 px-1.5 py-px text-[10px] font-normal leading-4 text-muted-foreground/70">
    {where}
  </span>
);

export function ModelPicker() {
  const { settings, updateSettings } = useSettings();
  const routerState = useRouterState();
  const router = useRouter();
  const scope = modelSelectionScope(
    routerState.location.pathname,
    router.options.basepath,
    routerState.location.search.id,
  );
  const {
    chat,
    setChatSelection,
    isLoading: chatLoading,
    isUpdating,
  } = useChatMode(scope.kind === "chat" ? scope.chatId : null);
  const { data: providers } = useLanguageModelProviders();
  const { data: modelsByProvider, isLoading } = useLanguageModelsByProviders();
  const [open, setOpen] = useState(false);
  const [selectionError, setSelectionError] = useState<string | null>(null);
  const [selecting, setSelecting] = useState(false);
  const [hydration, setHydration] = useState<ProgressSnapshot | null>(null);
  const selectionInFlight = useRef(false);
  const unavailable =
    scope.kind === "unavailable" || (scope.kind === "chat" && (chatLoading || !chat || isUpdating));

  useEffect(() => {
    const ipc = (window as unknown as {
      electron?: {
        ipcRenderer?: {
          on?: (ch: string, fn: (payload: unknown) => void) => (() => void) | void;
          removeListener?: (ch: string, fn: (payload: unknown) => void) => void;
        };
      };
    }).electron?.ipcRenderer;
    if (!ipc?.on) return;
    const onProgress = (payload: unknown) => {
      if (payload && typeof payload === "object") {
        setHydration(payload as ProgressSnapshot);
      }
    };
    const off = ipc.on("uor-r4:progress", onProgress);
    return () => {
      if (typeof off === "function") off();
      else ipc.removeListener?.("uor-r4:progress", onProgress);
    };
  }, []);

  const listed = (providers ?? [])
    .filter((provider) => ORDER.includes(provider.id))
    .sort((a, b) => ORDER.indexOf(a.id) - ORDER.indexOf(b.id));
  const selected: LargeLanguageModel = chat?.modelSelection ??
    settings?.selectedModel ?? { name: "uor-r4:qwen3-0.6b-foundry", provider: "uor-r4" };
  const modelsOf = (providerId: string) => modelsByProvider?.[providerId] ?? [];
  const isSelected = (providerId: string, model: LanguageModel) =>
    (selected.provider === providerId ||
      (selected.provider === "auto" && providerId === "uor-r4")) &&
    (selected.name === model.apiName || selected.provider === "auto");

  const options = listed.flatMap((provider) =>
    modelsOf(provider.id).map((model) => ({ provider: provider.id, model })),
  );
  const current =
    options.find(({ provider, model }) => isSelected(provider, model)) ??
    options.find(({ provider }) => provider === "uor-r4");
  const label = current ? current.model.displayName : selected.name;
  const where =
    current?.provider === "custom-api"
      ? "API"
      : hydration?.phase === "downloading"
        ? `${hydration.percent ?? 0}%`
        : hydration?.phase === "ready" && hydration?.cached
          ? "Cached"
          : current
            ? WHERE[current.provider]
            : "";

  const choose = async (providerId: string, model: LanguageModel) => {
    if (unavailable || selectionInFlight.current) return;
    selectionInFlight.current = true;
    setSelecting(true);
    setSelectionError(null);
    const next: LargeLanguageModel = { name: model.apiName, provider: providerId };
    try {
      if (scope.kind === "chat") {
        await setChatSelection({
          modelSelection: createModelSelection({ model: next, catalogModel: model }),
        });
      } else if (scope.kind === "default") {
        await updateSettings({ selectedModel: next });
      }
      setOpen(false);
    } catch {
      setSelectionError("The model selection could not be saved. Try again.");
      setOpen(true);
    } finally {
      selectionInFlight.current = false;
      setSelecting(false);
    }
  };

  const openApiSettings = () => {
    setOpen(false);
    void router.navigate({ to: "/settings" });
    setTimeout(() => {
      document.getElementById("provider-settings")?.scrollIntoView?.({ behavior: "smooth" });
    }, 80);
  };

  return (
    <DropdownMenu open={open} onOpenChange={setOpen}>
      <DropdownMenuTrigger
        className="inline-flex h-8 max-w-[240px] cursor-pointer items-center gap-2 whitespace-nowrap rounded-full border border-border/70 bg-background/40 pl-3 pr-2 text-xs font-medium text-foreground/90 shadow-none transition-colors hover:border-border hover:bg-muted/60 hover:text-foreground focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring data-[state=open]:bg-muted/60"
        disabled={unavailable || selecting}
        data-testid="model-picker"
        title={where ? `${label}, ${where}` : label}
      >
        <span className="truncate">{label}</span>
        {where && <Pill where={where} />}
        <ChevronDownIcon className="size-3.5 shrink-0 text-muted-foreground" aria-hidden="true" />
      </DropdownMenuTrigger>
      <DropdownMenuContent className="w-80 p-1.5" align="start">
        {selectionError && (
          <div role="alert" className="px-2 py-1.5 text-sm text-destructive">
            {selectionError}
          </div>
        )}
        {isLoading && (
          <div className="px-2 py-1.5 text-sm text-muted-foreground">Loading models</div>
        )}
        {options.map(({ provider, model }) => (
          <DropdownMenuItem
            key={provider + ":" + model.apiName}
            disabled={unavailable || selecting}
            onClick={() => choose(provider, model)}
            aria-current={isSelected(provider, model) ? "true" : undefined}
            data-testid={`model-option-${provider}-${model.apiName}`}
            className={cn("cursor-pointer gap-3 rounded-lg px-2.5 py-2", isSelected(provider, model) && "bg-accent")}
          >
            <span className="flex min-w-0 flex-1 flex-col gap-0.5">
              <span className={cn("truncate text-[13px] leading-5", isSelected(provider, model) ? "font-medium" : "font-normal")}>
                {model.displayName}
              </span>
              <span className="truncate text-[11px] leading-4 text-muted-foreground">
                {model.description || MEANS[provider]}
              </span>
            </span>
            <Tag provider={provider} />
          </DropdownMenuItem>
        ))}
        <div className="mt-1 border-t border-border/60 pt-1">
          <button
            type="button"
            onClick={openApiSettings}
            data-testid="uor-r4-add-api-endpoint-button"
            className="flex w-full cursor-pointer items-center gap-2 rounded-lg px-2.5 py-1.5 text-left text-[12px] text-foreground/90 hover:bg-muted/60"
          >
            <PlusCircleIcon className="size-3.5 shrink-0 text-muted-foreground" aria-hidden="true" />
            <span>Add / Manage AI API Endpoint…</span>
          </button>
          <div className="px-2.5 py-1 text-[10px] leading-4 text-muted-foreground" data-testid="uor-r4-hydration-status">
            {hydration?.phase === "downloading"
              ? `Downloading packs ${hydration.packsDone ?? 0}/${hydration.packsTotal ?? 10} (${hydration.percent ?? 0}%)…`
              : hydration?.phase === "ready"
                ? `Ready · ${hydration.cacheBackend || "OPFS"} cache${Array.isArray(hydration.recoveredPieces) && hydration.recoveredPieces.length ? ` · Stitch recovered ${hydration.recoveredPieces.length}` : ""}`
                : "Pre-alpha · M1 complete (1 of 8 milestones) · UORR4V13 Integer WASM"}
          </div>
        </div>
      </DropdownMenuContent>
    </DropdownMenu>
  );
}
