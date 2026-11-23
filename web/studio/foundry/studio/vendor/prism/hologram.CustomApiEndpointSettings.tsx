import { useQueryClient } from "@tanstack/react-query";
import {
  AlertCircle,
  CheckCircle2,
  Cpu,
  Globe,
  KeyRound,
  Loader2,
  Pencil,
  Plus,
  Trash2,
} from "lucide-react";
import { useCallback, useEffect, useState } from "react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { useSettings } from "@/hooks/useSettings";
import { queryKeys } from "@/lib/queryKeys";

type CustomEndpoint = {
  id: string;
  name: string;
  baseUrl: string;
  model: string;
  contextWindow: number;
  hasApiKey: boolean;
  createdAt?: string | null;
  updatedAt?: string | null;
  encryption?: string;
};

type TestOutcome = {
  ok: boolean;
  status?: number;
  latencyMs?: number;
  model?: string;
  replyPreview?: string;
  error?: string;
};

function getIpcRenderer() {
  return (
    window as unknown as {
      electron?: {
        ipcRenderer?: {
          invoke: <T = unknown>(channel: string, arg?: unknown) => Promise<T>;
        };
      };
    }
  ).electron?.ipcRenderer;
}

export function ProviderSettingsGrid() {
  const { settings, updateSettings } = useSettings();
  const queryClient = useQueryClient();

  const [endpoints, setEndpoints] = useState<CustomEndpoint[]>([]);
  const [loading, setLoading] = useState(true);
  const [editingId, setEditingId] = useState<string | null>(null);
  const [showForm, setShowForm] = useState(false);

  const [name, setName] = useState("");
  const [baseUrl, setBaseUrl] = useState("");
  const [model, setModel] = useState("");
  const [apiKey, setApiKey] = useState("");
  const [clearApiKey, setClearApiKey] = useState(false);
  const [contextWindow, setContextWindow] = useState("32768");

  const [saving, setSaving] = useState(false);
  const [testingId, setTestingId] = useState<string | null>(null);
  const [deletingId, setDeletingId] = useState<string | null>(null);
  const [formError, setFormError] = useState<string | null>(null);
  const [testResult, setTestResult] = useState<TestOutcome | null>(null);
  const [rowTestResults, setRowTestResults] = useState<Record<string, TestOutcome>>({});

  const refreshEndpoints = useCallback(async () => {
    const ipc = getIpcRenderer();
    if (!ipc) {
      setLoading(false);
      return;
    }
    try {
      const list = await ipc.invoke<CustomEndpoint[]>("custom-api:list");
      setEndpoints(Array.isArray(list) ? list : []);
    } catch {
      setEndpoints([]);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    void refreshEndpoints();
  }, [refreshEndpoints]);

  const invalidateModelQueries = useCallback(async () => {
    await Promise.all([
      queryClient.invalidateQueries({ queryKey: queryKeys.languageModels.providers }),
      queryClient.invalidateQueries({ queryKey: queryKeys.languageModels.byProviders }),
      queryClient.invalidateQueries({
        queryKey: queryKeys.languageModels.forProvider({ providerId: "custom-api" }),
      }),
      queryClient.invalidateQueries({ queryKey: queryKeys.settings.user }),
      queryClient.invalidateQueries({ queryKey: queryKeys.chats.all }),
      queryClient.invalidateQueries({ queryKey: queryKeys.tokenCount.all }),
    ]);
  }, [queryClient]);

  const resetForm = () => {
    setEditingId(null);
    setName("");
    setBaseUrl("");
    setModel("");
    setApiKey("");
    setClearApiKey(false);
    setContextWindow("32768");
    setFormError(null);
    setTestResult(null);
  };

  const openNewForm = () => {
    resetForm();
    setShowForm(true);
  };

  const openEditForm = (ep: CustomEndpoint) => {
    setEditingId(ep.id);
    setName(ep.name);
    setBaseUrl(ep.baseUrl);
    setModel(ep.model);
    setApiKey("");
    setClearApiKey(false);
    setContextWindow(String(ep.contextWindow || 32768));
    setFormError(null);
    setTestResult(null);
    setShowForm(true);
  };

  const handleUseBuiltIn = async () => {
    await updateSettings({
      selectedModel: { provider: "uor-r4", name: "uor-r4:qwen3-0.6b-foundry" },
    });
    await invalidateModelQueries();
  };

  const handleUseEndpoint = async (ep: CustomEndpoint) => {
    await updateSettings({
      selectedModel: { provider: "custom-api", name: ep.id },
    });
    await invalidateModelQueries();
  };

  const handleTestForm = async () => {
    const ipc = getIpcRenderer();
    if (!ipc) return;
    setTestingId("form");
    setFormError(null);
    setTestResult(null);
    try {
      const res = await ipc.invoke<TestOutcome>("custom-api:test", {
        ...(editingId ? { id: editingId } : {}),
        baseUrl,
        model,
        apiKey: apiKey.trim().length > 0 ? apiKey : undefined,
      });
      setTestResult(res);
    } catch (err) {
      setTestResult({ ok: false, error: String((err as Error)?.message || err) });
    } finally {
      setTestingId(null);
    }
  };

  const handleSaveForm = async (e: React.FormEvent) => {
    e.preventDefault();
    const ipc = getIpcRenderer();
    if (!ipc) return;
    setSaving(true);
    setFormError(null);
    try {
      const saved = await ipc.invoke<{ endpoint: CustomEndpoint; endpoints: CustomEndpoint[] }>(
        "custom-api:save",
        {
          ...(editingId ? { id: editingId } : {}),
          name,
          baseUrl,
          model,
          apiKey: apiKey.trim().length > 0 ? apiKey : undefined,
          clearApiKey,
          contextWindow: Number(contextWindow) || 32768,
        },
      );
      setEndpoints(Array.isArray(saved?.endpoints) ? saved.endpoints : []);
      if (saved?.endpoint?.id) {
        await updateSettings({
          selectedModel: { provider: "custom-api", name: saved.endpoint.id },
        });
      }
      await invalidateModelQueries();
      resetForm();
      setShowForm(false);
    } catch (err) {
      setFormError(String((err as Error)?.message || err));
    } finally {
      setSaving(false);
    }
  };

  const handleTestRow = async (ep: CustomEndpoint) => {
    const ipc = getIpcRenderer();
    if (!ipc) return;
    setTestingId(ep.id);
    try {
      const res = await ipc.invoke<TestOutcome>("custom-api:test", { id: ep.id });
      setRowTestResults((prev) => ({ ...prev, [ep.id]: res }));
    } catch (err) {
      setRowTestResults((prev) => ({
        ...prev,
        [ep.id]: { ok: false, error: String((err as Error)?.message || err) },
      }));
    } finally {
      setTestingId(null);
    }
  };

  const handleDelete = async (ep: CustomEndpoint) => {
    const ipc = getIpcRenderer();
    if (!ipc) return;
    setDeletingId(ep.id);
    try {
      const res = await ipc.invoke<{ deleted: boolean; endpoints: CustomEndpoint[] }>(
        "custom-api:delete",
        { id: ep.id },
      );
      setEndpoints(Array.isArray(res?.endpoints) ? res.endpoints : []);
      await invalidateModelQueries();
      if (editingId === ep.id) {
        resetForm();
        setShowForm(false);
      }
    } finally {
      setDeletingId(null);
    }
  };

  const selected = settings?.selectedModel;
  const isBuiltInSelected = !selected || selected.provider === "uor-r4";

  return (
    <div className="space-y-4" data-testid="custom-api-settings">
      <div className="flex flex-col gap-3 rounded-lg border border-border/70 bg-muted/20 p-4 sm:flex-row sm:items-center sm:justify-between">
        <div className="space-y-1">
          <div className="flex items-center gap-2">
            <Cpu className="size-4 text-primary" aria-hidden="true" />
            <span className="text-sm font-medium text-foreground">
              UOR-R4 Qwen3-0.6B Foundry (Built-in Integer WASM)
            </span>
            {isBuiltInSelected && (
              <span className="rounded-full bg-primary/15 px-2 py-0.5 text-[11px] font-medium text-primary">
                Active
              </span>
            )}
          </div>
          <p className="text-xs text-muted-foreground">
            Runs in-browser with OPFS cache and Stitch 2-of-3 parity recovery. No weight upload needed.
          </p>
        </div>
        {!isBuiltInSelected && (
          <Button
            type="button"
            variant="outline"
            size="sm"
            onClick={handleUseBuiltIn}
            data-testid="custom-api-use-builtin-button"
          >
            Use Built-in UOR-R4
          </Button>
        )}
      </div>

      <div className="flex items-center justify-between gap-2">
        <div>
          <h3 className="text-sm font-medium text-foreground">Custom AI API Endpoints</h3>
          <p className="text-xs text-muted-foreground">
            Connect any OpenAI-compatible <code>/v1/chat/completions</code> endpoint. API keys are sealed in IndexedDB via <code>core.wasm</code> ChaCha20-Poly1305.
          </p>
        </div>
        {!showForm && (
          <Button
            type="button"
            variant="outline"
            size="sm"
            onClick={openNewForm}
            data-testid="custom-api-add-button"
          >
            <Plus className="mr-1.5 size-3.5" />
            Add API Endpoint
          </Button>
        )}
      </div>

      {showForm && (
        <form
          onSubmit={handleSaveForm}
          className="space-y-3 rounded-lg border border-border bg-card p-4"
          data-testid="custom-api-form"
        >
          <div className="flex items-center justify-between">
            <span className="text-sm font-medium text-foreground">
              {editingId ? "Edit Custom AI API Endpoint" : "New Custom AI API Endpoint"}
            </span>
          </div>

          <div className="grid grid-cols-1 gap-3 sm:grid-cols-2">
            <div className="space-y-1">
              <Label htmlFor="custom-api-name">Endpoint Name</Label>
              <Input
                id="custom-api-name"
                data-testid="custom-api-name-input"
                placeholder="e.g. OpenRouter Qwen, Local Ollama"
                value={name}
                onChange={(e) => setName(e.target.value)}
                required
              />
            </div>
            <div className="space-y-1">
              <Label htmlFor="custom-api-model">Model ID</Label>
              <Input
                id="custom-api-model"
                data-testid="custom-api-model-input"
                placeholder="e.g. gpt-4o-mini or qwen3:8b"
                value={model}
                onChange={(e) => setModel(e.target.value)}
                required
              />
            </div>
          </div>

          <div className="space-y-1">
            <Label htmlFor="custom-api-base-url">OpenAI-Compatible Base URL</Label>
            <Input
              id="custom-api-base-url"
              data-testid="custom-api-base-url-input"
              placeholder="https://api.openai.com/v1 or http://127.0.0.1:11434/v1"
              value={baseUrl}
              onChange={(e) => setBaseUrl(e.target.value)}
              required
            />
          </div>

          <div className="grid grid-cols-1 gap-3 sm:grid-cols-3">
            <div className="space-y-1 sm:col-span-2">
              <Label htmlFor="custom-api-key">
                API Key {editingId ? "(leave blank to keep existing)" : "(optional)"}
              </Label>
              <Input
                id="custom-api-key"
                type="password"
                autoComplete="off"
                data-testid="custom-api-key-input"
                placeholder="sk-..."
                value={apiKey}
                onChange={(e) => {
                  setApiKey(e.target.value);
                  if (e.target.value) setClearApiKey(false);
                }}
              />
              {editingId && (
                <label className="mt-1 flex items-center gap-1.5 text-xs text-muted-foreground">
                  <input
                    type="checkbox"
                    checked={clearApiKey}
                    onChange={(e) => {
                      setClearApiKey(e.target.checked);
                      if (e.target.checked) setApiKey("");
                    }}
                  />
                  Remove stored API key
                </label>
              )}
            </div>
            <div className="space-y-1">
              <Label htmlFor="custom-api-context-window">Context Window</Label>
              <Input
                id="custom-api-context-window"
                type="number"
                min={1024}
                max={2000000}
                data-testid="custom-api-context-input"
                value={contextWindow}
                onChange={(e) => setContextWindow(e.target.value)}
              />
            </div>
          </div>

          {formError && (
            <div
              role="alert"
              className="flex items-start gap-2 rounded-md border border-destructive/40 bg-destructive/10 px-3 py-2 text-xs text-destructive"
              data-testid="custom-api-form-error"
            >
              <AlertCircle className="mt-0.5 size-3.5 shrink-0" />
              <span>{formError}</span>
            </div>
          )}

          {testResult && (
            <div
              data-testid="custom-api-test-result"
              className={
                testResult.ok
                  ? "flex items-start gap-2 rounded-md border border-emerald-500/30 bg-emerald-500/10 px-3 py-2 text-xs text-emerald-700 dark:text-emerald-300"
                  : "flex items-start gap-2 rounded-md border border-destructive/40 bg-destructive/10 px-3 py-2 text-xs text-destructive"
              }
            >
              {testResult.ok ? (
                <>
                  <CheckCircle2 className="mt-0.5 size-3.5 shrink-0" />
                  <span>
                    Connected ({testResult.latencyMs}ms · {testResult.model}): {testResult.replyPreview}
                  </span>
                </>
              ) : (
                <>
                  <AlertCircle className="mt-0.5 size-3.5 shrink-0" />
                  <span>{testResult.error || "Connection failed"}</span>
                </>
              )}
            </div>
          )}

          <div className="flex flex-wrap items-center gap-2 pt-1">
            <Button
              type="button"
              variant="outline"
              size="sm"
              onClick={handleTestForm}
              disabled={testingId === "form" || !baseUrl.trim() || !model.trim()}
              data-testid="custom-api-test-button"
            >
              {testingId === "form" ? (
                <Loader2 className="mr-1.5 size-3.5 animate-spin" />
              ) : null}
              Test Connection
            </Button>
            <Button
              type="submit"
              size="sm"
              disabled={saving || !name.trim() || !baseUrl.trim() || !model.trim()}
              data-testid="custom-api-save-button"
            >
              {saving ? <Loader2 className="mr-1.5 size-3.5 animate-spin" /> : null}
              Save &amp; Use Endpoint
            </Button>
            <Button
              type="button"
              variant="ghost"
              size="sm"
              onClick={() => {
                resetForm();
                setShowForm(false);
              }}
              data-testid="custom-api-cancel-button"
            >
              Cancel
            </Button>
          </div>
        </form>
      )}

      {loading ? (
        <p className="text-xs text-muted-foreground">Loading configured endpoints…</p>
      ) : endpoints.length === 0 ? (
        !showForm && (
          <p className="rounded-lg border border-dashed border-border/70 px-4 py-3 text-xs text-muted-foreground">
            No custom AI API endpoints configured. Studio uses the built-in UOR-R4 WASM engine until you add an endpoint.
          </p>
        )
      ) : (
        <div className="space-y-2" data-testid="custom-api-endpoint-list">
          {endpoints.map((ep) => {
            const isActive =
              selected?.provider === "custom-api" && selected?.name === ep.id;
            const rowTest = rowTestResults[ep.id];
            return (
              <div
                key={ep.id}
                data-testid={`custom-api-row-${ep.id}`}
                className="flex flex-col gap-2 rounded-lg border border-border/70 bg-card/60 p-3 sm:flex-row sm:items-center sm:justify-between"
              >
                <div className="min-w-0 space-y-0.5">
                  <div className="flex flex-wrap items-center gap-2">
                    <Globe className="size-3.5 shrink-0 text-muted-foreground" />
                    <span className="truncate text-sm font-medium text-foreground">
                      {ep.name}
                    </span>
                    <span className="rounded bg-muted px-1.5 py-0.5 font-mono text-[11px] text-foreground">
                      {ep.model}
                    </span>
                    {ep.hasApiKey && (
                      <span className="inline-flex items-center gap-1 rounded bg-muted/60 px-1.5 py-0.5 text-[10px] text-muted-foreground">
                        <KeyRound className="size-2.5" />
                        Key sealed
                      </span>
                    )}
                    {isActive && (
                      <span className="rounded-full bg-primary/15 px-2 py-0.5 text-[11px] font-medium text-primary">
                        Active
                      </span>
                    )}
                  </div>
                  <p className="truncate text-xs text-muted-foreground">{ep.baseUrl}</p>
                  {rowTest && (
                    <p
                      className={
                        rowTest.ok
                          ? "text-[11px] text-emerald-700 dark:text-emerald-300"
                          : "text-[11px] text-destructive"
                      }
                      data-testid={`custom-api-row-test-${ep.id}`}
                    >
                      {rowTest.ok
                        ? `Connected (${rowTest.latencyMs}ms): ${rowTest.replyPreview}`
                        : rowTest.error}
                    </p>
                  )}
                </div>
                <div className="flex shrink-0 items-center gap-1.5">
                  {!isActive && (
                    <Button
                      type="button"
                      size="sm"
                      variant="outline"
                      onClick={() => handleUseEndpoint(ep)}
                      data-testid={`custom-api-use-${ep.id}`}
                    >
                      Use
                    </Button>
                  )}
                  <Button
                    type="button"
                    size="sm"
                    variant="ghost"
                    onClick={() => handleTestRow(ep)}
                    disabled={testingId === ep.id}
                    data-testid={`custom-api-test-row-${ep.id}`}
                  >
                    {testingId === ep.id ? (
                      <Loader2 className="size-3.5 animate-spin" />
                    ) : (
                      "Test"
                    )}
                  </Button>
                  <Button
                    type="button"
                    size="sm"
                    variant="ghost"
                    onClick={() => openEditForm(ep)}
                    aria-label={`Edit ${ep.name}`}
                    data-testid={`custom-api-edit-${ep.id}`}
                  >
                    <Pencil className="size-3.5" />
                  </Button>
                  <Button
                    type="button"
                    size="sm"
                    variant="ghost"
                    onClick={() => handleDelete(ep)}
                    disabled={deletingId === ep.id}
                    aria-label={`Delete ${ep.name}`}
                    data-testid={`custom-api-delete-${ep.id}`}
                  >
                    {deletingId === ep.id ? (
                      <Loader2 className="size-3.5 animate-spin" />
                    ) : (
                      <Trash2 className="size-3.5 text-destructive" />
                    )}
                  </Button>
                </div>
              </div>
            );
          })}
        </div>
      )}
    </div>
  );
}
