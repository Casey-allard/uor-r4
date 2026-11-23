// Workspace GitHub Repository Connector for PublishPanel, App Details & Import Dialog.
// Replaces src/components/GitHubConnector.tsx via vite.shell.config.mts resolveId.
import { useCallback, useEffect, useState } from "react";
import { useNavigate } from "@tanstack/react-router";
import { useQueryClient } from "@tanstack/react-query";
import {
  ArrowDownToLine,
  ArrowUpFromLine,
  CheckCircle2,
  GitBranch,
  Github,
  Link2,
  Loader2,
  Plus,
  RefreshCw,
  Unlink,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { useLoadApp } from "@/hooks/useLoadApp";
import { showError, showSuccess } from "@/lib/toast";
import { queryKeys } from "@/lib/queryKeys";

interface GitHubConnectorProps {
  appId: number | null;
  folderName: string;
  expanded?: boolean;
}

export interface UnconnectedGitHubConnectorProps {
  appId: number | null;
  folderName: string;
  settings?: any;
  refreshSettings?: () => void;
  expanded?: boolean;
  linkedRepo?: { org: string; repo: string };
}

interface GitHubStatus {
  connected: boolean;
  login: string | null;
}

interface DiffSummary {
  owner: string;
  repo: string;
  branch: string;
  headSha: string | null;
  added: string[];
  modified: string[];
  deleted: string[];
  totalChanged: number;
  clean: boolean;
}

interface RepoRow {
  name: string;
  full_name: string;
  default_branch?: string;
}

const getIpc = () => (window as any).electron?.ipcRenderer;

export function UnconnectedGitHubConnector({
  appId,
  folderName,
  linkedRepo,
}: UnconnectedGitHubConnectorProps) {
  const navigate = useNavigate();
  const queryClient = useQueryClient();
  const [status, setStatus] = useState<GitHubStatus | null>(null);
  const [repos, setRepos] = useState<RepoRow[]>([]);
  const [selectedRepo, setSelectedRepo] = useState(
    linkedRepo ? `${linkedRepo.org}/${linkedRepo.repo}` : "",
  );
  const [newRepoName, setNewRepoName] = useState(() =>
    (folderName || "foundry-app").replace(/[^A-Za-z0-9._-]+/g, "-").slice(0, 64),
  );
  const [busy, setBusy] = useState(false);

  const load = useCallback(async () => {
    const ipc = getIpc();
    if (!ipc) return;
    try {
      const st = await ipc.invoke("github:status");
      setStatus(st);
      if (st?.connected) {
        const list = await ipc.invoke("github:list-repos");
        const arr = Array.isArray(list) ? list : [];
        setRepos(arr);
        if (arr[0]?.full_name) {
          setSelectedRepo((prev) => prev || arr[0].full_name);
        }
      }
    } catch {}
  }, []);

  useEffect(() => {
    void load();
  }, [load]);

  if (!status?.connected) {
    return (
      <div className="space-y-2 rounded-md border border-border/60 bg-muted/20 p-3 text-xs" data-testid="github-connect-prompt">
        <p className="text-muted-foreground">
          Connect your GitHub Personal Access Token in Settings to clone, pull, and push repositories directly from the browser.
        </p>
        <Button
          type="button"
          variant="outline"
          size="sm"
          onClick={() => navigate({ to: "/settings", hash: "setting-github" })}
          data-testid="open-github-settings-button"
        >
          <Github className="mr-1.5 size-3.5" />
          Connect GitHub in Settings
        </Button>
      </div>
    );
  }

  const handleLinkExisting = async () => {
    const ipc = getIpc();
    if (!ipc || !appId || !selectedRepo.includes("/")) return;
    const [owner, repo] = selectedRepo.split("/");
    setBusy(true);
    try {
      await ipc.invoke("github:link-repo", { appId, owner, repo });
      await queryClient.invalidateQueries({ queryKey: queryKeys.apps.detail({ appId }) });
      await queryClient.invalidateQueries({ queryKey: queryKeys.apps.all });
      showSuccess(`Linked app to ${owner}/${repo}`);
    } catch (err: any) {
      showError(err?.message || "Failed to link repository");
    } finally {
      setBusy(false);
    }
  };

  const handleCreateAndLink = async () => {
    const ipc = getIpc();
    if (!ipc || !appId || !newRepoName.trim()) return;
    setBusy(true);
    try {
      const created = await ipc.invoke("github:create-repo", {
        name: newRepoName.trim(),
        isPrivate: true,
      });
      await ipc.invoke("github:link-repo", {
        appId,
        owner: created.owner || status.login,
        repo: created.name,
        branch: created.default_branch || "main",
      });
      await queryClient.invalidateQueries({ queryKey: queryKeys.apps.detail({ appId }) });
      await queryClient.invalidateQueries({ queryKey: queryKeys.apps.all });
      showSuccess(`Created and linked ${created.full_name}`);
    } catch (err: any) {
      showError(err?.message || "Failed to create repository");
    } finally {
      setBusy(false);
    }
  };

  return (
    <div className="space-y-3 rounded-md border border-border/60 bg-muted/20 p-3 text-xs" data-testid="github-unconnected-repo">
      <div className="text-muted-foreground">
        Connected as <span className="font-medium text-foreground">@{status.login}</span>. Link this workspace to a GitHub repository:
      </div>

      {appId !== null && (
        <>
          <div className="flex flex-wrap gap-2">
            <Input
              value={selectedRepo}
              onChange={(e) => setSelectedRepo(e.target.value)}
              placeholder="owner/repo"
              list="uor-github-repos-datalist"
              className="min-w-[180px] flex-1 font-mono text-xs"
            />
            <datalist id="uor-github-repos-datalist">
              {repos.map((r) => (
                <option key={r.full_name} value={r.full_name} />
              ))}
            </datalist>
            <Button
              type="button"
              size="sm"
              variant="secondary"
              disabled={busy || !selectedRepo.includes("/")}
              onClick={handleLinkExisting}
            >
              <Link2 className="mr-1.5 size-3.5" />
              Link Repo
            </Button>
          </div>

          <div className="flex flex-wrap items-center gap-2 border-t border-border/50 pt-2">
            <Input
              value={newRepoName}
              onChange={(e) => setNewRepoName(e.target.value)}
              placeholder="new-repo-name"
              className="min-w-[180px] flex-1 font-mono text-xs"
            />
            <Button
              type="button"
              size="sm"
              disabled={busy || !newRepoName.trim()}
              onClick={handleCreateAndLink}
            >
              {busy ? <Loader2 className="mr-1.5 size-3.5 animate-spin" /> : <Plus className="mr-1.5 size-3.5" />}
              Create &amp; Link
            </Button>
          </div>
        </>
      )}
    </div>
  );
}

export function GitHubConnector({
  appId,
  folderName,
  expanded,
}: GitHubConnectorProps) {
  const { app } = useLoadApp(appId);
  const queryClient = useQueryClient();
  const [diff, setDiff] = useState<DiffSummary | null>(null);
  const [commitMessage, setCommitMessage] = useState("");
  const [busyOp, setBusyOp] = useState<"diff" | "pull" | "push" | "unlink" | null>(null);

  const refreshDiff = useCallback(async () => {
    const ipc = getIpc();
    if (!ipc || !appId || !app?.githubOrg || !app?.githubRepo) return;
    setBusyOp("diff");
    try {
      const d = await ipc.invoke("github:diff", { appId });
      setDiff(d);
    } catch {
      setDiff(null);
    } finally {
      setBusyOp(null);
    }
  }, [appId, app?.githubOrg, app?.githubRepo]);

  useEffect(() => {
    void refreshDiff();
  }, [refreshDiff]);

  if (!appId || !app?.githubOrg || !app?.githubRepo) {
    return (
      <UnconnectedGitHubConnector
        appId={appId}
        folderName={folderName}
        expanded={expanded}
      />
    );
  }

  const branch = app.githubBranch || "main";

  const handlePull = async () => {
    const ipc = getIpc();
    if (!ipc) return;
    setBusyOp("pull");
    try {
      const res = await ipc.invoke("github:pull", { appId, branch });
      await queryClient.invalidateQueries({ queryKey: queryKeys.apps.detail({ appId }) });
      await queryClient.invalidateQueries({ queryKey: queryKeys.versions.list({ appId }) });
      if (res.alreadyUpToDate) {
        showSuccess("Already up to date with GitHub");
      } else {
        showSuccess(`Pulled ${res.updatedCount} file(s) from ${app.githubOrg}/${app.githubRepo}`);
      }
      await refreshDiff();
    } catch (err: any) {
      showError(err?.message || "GitHub pull failed");
    } finally {
      setBusyOp(null);
    }
  };

  const handlePush = async () => {
    const ipc = getIpc();
    if (!ipc) return;
    setBusyOp("push");
    try {
      const res = await ipc.invoke("github:push", {
        appId,
        branch,
        message: commitMessage.trim() || `Update ${app.githubRepo} from Foundry Studio`,
      });
      if (!res.ok) {
        showError(res.error || "GitHub push refused");
        return;
      }
      if (res.nothingToPush) {
        showSuccess("Working tree is clean — nothing new to push");
      } else {
        setCommitMessage("");
        showSuccess(`Pushed ${res.filesPushed} file(s) (${String(res.commitSha || "").slice(0, 7)})`);
      }
      await refreshDiff();
    } catch (err: any) {
      showError(err?.message || "GitHub push failed");
    } finally {
      setBusyOp(null);
    }
  };

  const handleUnlink = async () => {
    const ipc = getIpc();
    if (!ipc) return;
    setBusyOp("unlink");
    try {
      await ipc.invoke("github:unlink-repo", { appId });
      await queryClient.invalidateQueries({ queryKey: queryKeys.apps.detail({ appId }) });
      await queryClient.invalidateQueries({ queryKey: queryKeys.apps.all });
      setDiff(null);
      showSuccess("Unlinked GitHub repository");
    } catch (err: any) {
      showError(err?.message || "Failed to unlink repository");
    } finally {
      setBusyOp(null);
    }
  };

  return (
    <div className="space-y-3 rounded-md border border-border/70 bg-muted/20 p-3 text-xs" data-testid="github-connected-repo">
      <div className="flex flex-wrap items-center justify-between gap-2">
        <div className="space-y-0.5">
          <div className="flex items-center gap-1.5 font-medium text-foreground">
            <CheckCircle2 className="size-3.5 text-emerald-500" />
            <a
              href={`https://github.com/${app.githubOrg}/${app.githubRepo}`}
              target="_blank"
              rel="noopener noreferrer"
              className="font-mono hover:underline"
            >
              {app.githubOrg}/{app.githubRepo}
            </a>
          </div>
          <div className="flex items-center gap-2 font-mono text-[11px] text-muted-foreground">
            <GitBranch className="size-3" />
            <span>{branch}</span>
            {diff?.headSha && <span>· {diff.headSha.slice(0, 7)}</span>}
          </div>
        </div>
        <div className="flex items-center gap-1.5">
          <Button
            type="button"
            variant="ghost"
            size="sm"
            onClick={refreshDiff}
            disabled={busyOp !== null}
            title="Refresh workspace diff"
          >
            <RefreshCw className={`size-3.5 ${busyOp === "diff" ? "animate-spin" : ""}`} />
          </Button>
          <Button
            type="button"
            variant="ghost"
            size="sm"
            onClick={handleUnlink}
            disabled={busyOp !== null}
            data-testid="github-unlink-button"
          >
            <Unlink className="mr-1 size-3.5" />
            Unlink
          </Button>
        </div>
      </div>

      {diff && (
        <div className="rounded border border-border/50 bg-background/60 px-2.5 py-1.5 font-mono text-[11px]" data-testid="github-diff-summary">
          {diff.clean ? (
            <span className="text-muted-foreground">Workspace matches remote snapshot.</span>
          ) : (
            <span>
              Pending changes: +{diff.added.length} added · ~{diff.modified.length} modified · -{diff.deleted.length} deleted
            </span>
          )}
        </div>
      )}

      <div className="flex flex-wrap items-center gap-2">
        <Input
          value={commitMessage}
          onChange={(e) => setCommitMessage(e.target.value)}
          placeholder={`Commit message for ${branch}...`}
          className="min-w-[180px] flex-1 text-xs"
          data-testid="github-commit-message-input"
        />
        <Button
          type="button"
          variant="outline"
          size="sm"
          onClick={handlePull}
          disabled={busyOp !== null}
          data-testid="github-pull-button"
        >
          {busyOp === "pull" ? <Loader2 className="mr-1.5 size-3.5 animate-spin" /> : <ArrowDownToLine className="mr-1.5 size-3.5" />}
          Pull
        </Button>
        <Button
          type="button"
          size="sm"
          onClick={handlePush}
          disabled={busyOp !== null}
          data-testid="github-push-button"
        >
          {busyOp === "push" ? <Loader2 className="mr-1.5 size-3.5 animate-spin" /> : <ArrowUpFromLine className="mr-1.5 size-3.5" />}
          Push to GitHub
        </Button>
      </div>
    </div>
  );
}
