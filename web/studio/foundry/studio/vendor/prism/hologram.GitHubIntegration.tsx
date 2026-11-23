// Settings -> Integrations -> GitHub & Stitch Mesh + afflom/me bridge card.
// Replaces src/components/GitHubIntegration.tsx via vite.shell.config.mts resolveId.
import { useCallback, useEffect, useState } from "react";
import { useNavigate } from "@tanstack/react-router";
import { useQueryClient } from "@tanstack/react-query";
import {
  CheckCircle2,
  ExternalLink,
  FolderGit2,
  Github,
  KeyRound,
  Link2,
  Loader2,
  Network,
  ShieldCheck,
  Unplug,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { showError, showSuccess } from "@/lib/toast";
import { queryKeys } from "@/lib/queryKeys";

interface GitHubStatus {
  connected: boolean;
  login: string | null;
  name: string | null;
  avatarUrl: string | null;
  apiBase: string;
  oauthRelayUrl?: string | null;
  connectedAt?: string | null;
  encryption?: string;
}

interface GitHubRepoSummary {
  name: string;
  full_name: string;
  private: boolean;
  default_branch?: string;
}

interface StitchStatus {
  profile: string;
  connected: boolean;
  channelKey: string;
  peerId: string | null;
  algorithm: string;
  peers: Array<{ peerId: string }>;
  webrtcPeers: Array<{ peerId: string; state: string }>;
}

const getIpc = () => (window as any).electron?.ipcRenderer;

export function GitHubIntegration() {
  const navigate = useNavigate();
  const queryClient = useQueryClient();

  const [status, setStatus] = useState<GitHubStatus | null>(null);
  const [stitch, setStitch] = useState<StitchStatus | null>(null);
  const [token, setToken] = useState("");
  const [apiBase, setApiBase] = useState(() => {
    try {
      return localStorage.getItem("uor.github.apiBase") || "";
    } catch {
      return "";
    }
  });
  const [showAdvanced, setShowAdvanced] = useState(false);
  const [busy, setBusy] = useState(false);

  const [repos, setRepos] = useState<GitHubRepoSummary[]>([]);
  const [loadingRepos, setLoadingRepos] = useState(false);
  const [cloneUrl, setCloneUrl] = useState("");
  const [cloning, setCloning] = useState(false);

  const [stitchChannel, setStitchChannel] = useState("uor-r4-studio");
  const [stitchBusy, setStitchBusy] = useState(false);

  const refresh = useCallback(async () => {
    const ipc = getIpc();
    if (!ipc) return;
    try {
      const [gh, st] = await Promise.all([
        ipc.invoke("github:status"),
        ipc.invoke("stitch:status"),
      ]);
      setStatus(gh);
      setStitch(st);
      if (gh?.connected) {
        setLoadingRepos(true);
        try {
          const list = await ipc.invoke("github:list-repos");
          setRepos(Array.isArray(list) ? list : []);
        } catch {
          setRepos([]);
        } finally {
          setLoadingRepos(false);
        }
      } else {
        setRepos([]);
      }
    } catch {}
  }, []);

  useEffect(() => {
    void refresh();
    const ipc = getIpc();
    if (!ipc?.on) return;
    const offGh = ipc.on("github:status-changed", (next: GitHubStatus) => setStatus(next));
    const offSt = ipc.on("stitch:status-changed", (next: StitchStatus) => setStitch(next));
    return () => {
      offGh?.();
      offSt?.();
    };
  }, [refresh]);

  const handleConnect = async (e: React.FormEvent) => {
    e.preventDefault();
    const ipc = getIpc();
    if (!ipc) return;
    setBusy(true);
    try {
      const next = await ipc.invoke("github:connect-token", {
        token: token.trim(),
        ...(apiBase.trim() ? { apiBase: apiBase.trim() } : {}),
      });
      setToken("");
      setStatus(next);
      showSuccess(`Connected GitHub as @${next.login}`);
      await refresh();
    } catch (err: any) {
      showError(err?.message || "Failed to connect GitHub token");
    } finally {
      setBusy(false);
    }
  };

  const handleDisconnect = async () => {
    const ipc = getIpc();
    if (!ipc) return;
    setBusy(true);
    try {
      const next = await ipc.invoke("github:disconnect");
      setStatus(next);
      setRepos([]);
      showSuccess("Disconnected GitHub account and wiped browser vault");
    } catch (err: any) {
      showError(err?.message || "Failed to disconnect GitHub");
    } finally {
      setBusy(false);
    }
  };

  const handleClone = async (targetUrl?: string) => {
    const url = String(targetUrl ?? cloneUrl).trim();
    if (!url) return;
    const ipc = getIpc();
    if (!ipc) return;
    setCloning(true);
    try {
      const res = await ipc.invoke("github:clone-repo-from-url", { url });
      if (res?.error) {
        showError(res.error);
        return;
      }
      await queryClient.invalidateQueries({ queryKey: queryKeys.apps.all });
      showSuccess(`Cloned ${res.app?.name || url} into Studio workspace`);
      setCloneUrl("");
      if (res?.app?.id) {
        navigate({ to: "/app-details", search: { appId: res.app.id } });
      }
    } catch (err: any) {
      showError(err?.message || "Clone failed");
    } finally {
      setCloning(false);
    }
  };

  const handleStitchToggle = async () => {
    const ipc = getIpc();
    if (!ipc) return;
    setStitchBusy(true);
    try {
      const next = stitch?.connected
        ? await ipc.invoke("stitch:disconnect")
        : await ipc.invoke("stitch:connect", { channelKey: stitchChannel.trim() || "uor-r4-studio" });
      setStitch(next);
      showSuccess(next?.connected ? `Connected to Stitch channel "${next.channelKey}"` : "Disconnected Stitch peer channel");
    } catch (err: any) {
      showError(err?.message || "Stitch network toggle failed");
    } finally {
      setStitchBusy(false);
    }
  };

  return (
    <div className="space-y-6" data-testid="github-integration-settings">
      <div className="space-y-4 rounded-lg border border-border/70 bg-muted/20 p-4">
        <div className="flex flex-wrap items-start justify-between gap-3">
          <div className="space-y-1">
            <div className="flex items-center gap-2 text-sm font-medium text-foreground">
              <Github className="size-4" />
              <span>GitHub Sync</span>
              {status?.connected ? (
                <span className="inline-flex items-center gap-1 rounded-full bg-emerald-500/15 px-2 py-0.5 text-[11px] font-medium text-emerald-600 dark:text-emerald-400">
                  <CheckCircle2 className="size-3" />@{status.login}
                </span>
              ) : (
                <span className="rounded-full bg-muted px-2 py-0.5 text-[11px] text-muted-foreground">
                  Not connected
                </span>
              )}
            </div>
            <p className="text-xs text-muted-foreground">
              Clone repositories directly into Studio, edit code with UOR-R4 or Linux 6.18.7, pull updates, and push commits via GitHub REST API.
            </p>
          </div>
          {status?.connected && (
            <Button
              type="button"
              variant="outline"
              size="sm"
              onClick={handleDisconnect}
              disabled={busy}
              data-testid="github-disconnect-button"
            >
              <Unplug className="mr-1.5 size-3.5" />
              Disconnect
            </Button>
          )}
        </div>

        <div className="flex items-center gap-1.5 text-[11px] text-muted-foreground">
          <ShieldCheck className="size-3.5 text-emerald-500" />
          <span>
            Credentials are encrypted at rest inside this browser using <code className="font-mono">core.wasm</code> ChaCha20-Poly1305.
          </span>
        </div>

        {!status?.connected ? (
          <form onSubmit={handleConnect} className="space-y-3 pt-1">
            <div className="space-y-1.5">
              <div className="flex items-center justify-between gap-2">
                <Label htmlFor="github-pat-input" className="text-xs font-medium">
                  GitHub Personal Access Token (Fine-Grained or Classic `repo`)
                </Label>
                <a
                  href="https://github.com/settings/personal-access-tokens/new"
                  target="_blank"
                  rel="noopener noreferrer"
                  className="inline-flex items-center gap-1 text-[11px] text-primary hover:underline"
                >
                  Create token <ExternalLink className="size-3" />
                </a>
              </div>
              <div className="flex flex-wrap gap-2">
                <Input
                  id="github-pat-input"
                  data-testid="github-pat-input"
                  type="password"
                  value={token}
                  onChange={(e) => setToken(e.target.value)}
                  placeholder="github_pat_... or ghp_..."
                  autoComplete="off"
                  className="min-w-[240px] flex-1 font-mono text-xs"
                />
                <Button
                  type="submit"
                  size="sm"
                  disabled={busy || !token.trim()}
                  data-testid="github-connect-button"
                >
                  {busy ? <Loader2 className="mr-1.5 size-3.5 animate-spin" /> : <KeyRound className="mr-1.5 size-3.5" />}
                  Connect GitHub
                </Button>
              </div>
            </div>

            <div>
              <button
                type="button"
                onClick={() => setShowAdvanced((v) => !v)}
                className="text-[11px] text-muted-foreground hover:text-foreground"
              >
                {showAdvanced ? "Hide" : "Show"} custom GitHub API base URL (Enterprise / test relay)
              </button>
              {showAdvanced && (
                <div className="mt-2 space-y-1">
                  <Label htmlFor="github-api-base-input" className="text-[11px] text-muted-foreground">
                    API Base URL (optional; defaults to https://api.github.com)
                  </Label>
                  <Input
                    id="github-api-base-input"
                    data-testid="github-api-base-input"
                    type="url"
                    value={apiBase}
                    onChange={(e) => setApiBase(e.target.value)}
                    placeholder="https://api.github.com"
                    className="font-mono text-xs"
                  />
                </div>
              )}
            </div>
          </form>
        ) : (
          <div className="space-y-3 border-t border-border/60 pt-3">
            <div className="space-y-1.5">
              <Label htmlFor="github-clone-url-input" className="text-xs font-medium">
                Clone Repository into Studio Workspace
              </Label>
              <div className="flex flex-wrap gap-2">
                <Input
                  id="github-clone-url-input"
                  data-testid="github-clone-url-input"
                  value={cloneUrl}
                  onChange={(e) => setCloneUrl(e.target.value)}
                  placeholder="owner/repo or https://github.com/owner/repo"
                  className="min-w-[240px] flex-1 font-mono text-xs"
                />
                <Button
                  type="button"
                  size="sm"
                  onClick={() => handleClone()}
                  disabled={cloning || !cloneUrl.trim()}
                  data-testid="github-clone-button"
                >
                  {cloning ? <Loader2 className="mr-1.5 size-3.5 animate-spin" /> : <FolderGit2 className="mr-1.5 size-3.5" />}
                  Clone Repo
                </Button>
              </div>
            </div>

            {loadingRepos ? (
              <p className="text-xs text-muted-foreground">Loading your GitHub repositories...</p>
            ) : repos.length > 0 ? (
              <div className="space-y-1.5">
                <div className="text-[11px] font-medium text-muted-foreground">
                  Recent repositories (click Clone to open in Studio):
                </div>
                <div className="max-h-36 space-y-1 overflow-y-auto pr-1">
                  {repos.slice(0, 12).map((r) => (
                    <div
                      key={r.full_name}
                      className="flex items-center justify-between gap-2 rounded-md border border-border/50 bg-background/60 px-2.5 py-1.5 text-xs"
                    >
                      <span className="truncate font-mono">{r.full_name}</span>
                      <Button
                        type="button"
                        variant="ghost"
                        size="sm"
                        className="h-6 px-2 text-[11px]"
                        disabled={cloning}
                        onClick={() => handleClone(r.full_name)}
                      >
                        Clone
                      </Button>
                    </div>
                  ))}
                </div>
              </div>
            ) : null}
          </div>
        )}
      </div>

      {/* Stitch Mesh & afflom/me card */}
      <div className="space-y-3 rounded-lg border border-border/70 bg-muted/20 p-4" data-testid="stitch-integration-settings">
        <div className="flex flex-wrap items-start justify-between gap-3">
          <div className="space-y-1">
            <div className="flex items-center gap-2 text-sm font-medium text-foreground">
              <Network className="size-4" />
              <span>Stitch Peer Mesh &amp; afflom/me App Store</span>
              <span
                className={
                  stitch?.connected
                    ? "rounded-full bg-emerald-500/15 px-2 py-0.5 text-[11px] font-medium text-emerald-600 dark:text-emerald-400"
                    : "rounded-full bg-muted px-2 py-0.5 text-[11px] text-muted-foreground"
                }
              >
                {stitch?.connected ? "Connected" : "Opt-in standby"}
              </span>
            </div>
            <p className="text-xs text-muted-foreground">
              Ed25519 authenticated <code className="font-mono">stitch.signal/1</code> peer transport and <code className="font-mono">?embed=me</code> workspace bridge for <code className="font-mono">afflom/me</code>.
            </p>
          </div>
          <a
            href="./app.json"
            target="_blank"
            rel="noopener noreferrer"
            className="inline-flex items-center gap-1 text-xs text-primary hover:underline"
          >
            <Link2 className="size-3.5" /> app.json
          </a>
        </div>

        <div className="grid gap-1.5 rounded-md border border-border/50 bg-background/50 px-3 py-2 text-[11px] font-mono text-muted-foreground">
          <div>Peer ID: <span className="text-foreground">{stitch?.peerId || "initializing..."}</span></div>
          <div>
            Signing: <span className="text-foreground">{stitch?.algorithm || "Ed25519"}</span> · Peers:{" "}
            <span className="text-foreground">{stitch?.peers?.length ?? 0}</span> · WebRTC:{" "}
            <span className="text-foreground">{stitch?.webrtcPeers?.length ?? 0}</span>
          </div>
        </div>

        <div className="flex flex-wrap items-center gap-2">
          <Input
            value={stitchChannel}
            onChange={(e) => setStitchChannel(e.target.value)}
            placeholder="uor-r4-studio"
            disabled={stitch?.connected || stitchBusy}
            className="max-w-[220px] font-mono text-xs"
            aria-label="Stitch channel key"
          />
          <Button
            type="button"
            variant={stitch?.connected ? "outline" : "secondary"}
            size="sm"
            disabled={stitchBusy}
            onClick={handleStitchToggle}
            data-testid="stitch-toggle-button"
          >
            {stitchBusy ? <Loader2 className="mr-1.5 size-3.5 animate-spin" /> : null}
            {stitch?.connected ? "Leave Stitch Channel" : "Join Stitch Channel"}
          </Button>
        </div>
      </div>
    </div>
  );
}
