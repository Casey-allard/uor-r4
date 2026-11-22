// prism-prism (ours, not upstream's; see VENDORED.md): the home page's Import App dialog, replacing upstream's at build
// time through `resolveId` in vite.shell.config.mts. Upstream's imports a folder from disk or a GitHub repository
// into its apps folder; a web page has neither. What a visitor brings here is a Forge export file: the site
// Forge was served from before keeps each visitor's apps in its own browser storage, and its moved page writes
// them to one file (schema hologram-forge/export/1). The host's `forge:import` (shell/host.js) writes the file's
// apps, chats, files and versions into this site's store under new ids. Keys never travel in the file.
import { useRef, useState } from "react";
import { useQueryClient } from "@tanstack/react-query";
import { Loader2, Upload } from "lucide-react";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { queryKeys } from "@/lib/queryKeys";
import { showError, showSuccess } from "@/lib/toast";

// Upstream's dialog exports this prompt and other modules may import it; it is kept so they still resolve.
export const AI_RULES_PROMPT =
  "Generate an AI_RULES.md file for this app. Describe the tech stack in 5-10 bullet points and describe clear rules about what libraries to use for what.";

type Imported = { apps: { id: number; name: string; from: string; chats: number[]; files: number }[]; objects: number; refused: number };

const host = () => (window as any).electron.ipcRenderer;

export function ImportAppDialog({ isOpen, onClose }: { isOpen: boolean; onClose: () => void }) {
  const queryClient = useQueryClient();
  const input = useRef<HTMLInputElement>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const take = async (file: File | undefined) => {
    if (!file) return;
    setBusy(true);
    setError(null);
    try {
      const result: Imported = await host().invoke("forge:import", await file.text());
      await queryClient.invalidateQueries({ queryKey: queryKeys.apps.all });
      await queryClient.invalidateQueries({ queryKey: queryKeys.chats.all });
      const n = result.apps.length;
      const renamed = result.apps.filter((a) => a.name !== a.from).map((a) => a.name);
      showSuccess(
        `Imported ${n} app${n === 1 ? "" : "s"}.` + (renamed.length ? ` Renamed: ${renamed.join(", ")}.` : ""),
      );
      onClose();
    } catch (e: any) {
      const message = e instanceof SyntaxError ? "This file is not JSON." : e?.message || String(e);
      setError(message);
      showError(message);
    } finally {
      setBusy(false);
      if (input.current) input.current.value = "";
    }
  };

  return (
    <Dialog open={isOpen} onOpenChange={(open) => !open && !busy && onClose()}>
      <DialogContent className="sm:max-w-md">
        <DialogHeader>
          <DialogTitle>Import apps</DialogTitle>
          <DialogDescription>
            Bring the apps you made on the earlier Forge site. Open that site, choose Export my apps, then
            choose the file it saved here. Your apps, chats and versions come with it; model keys do not, so
            enter them again in Settings.
          </DialogDescription>
        </DialogHeader>
        <input
          ref={input}
          type="file"
          accept=".json,application/json"
          className="hidden"
          data-testid="forge-import-file"
          onChange={(e) => take(e.target.files?.[0])}
        />
        {error && <p className="text-sm text-destructive">{error}</p>}
        <DialogFooter>
          <Button variant="outline" onClick={onClose} disabled={busy}>
            Cancel
          </Button>
          <Button onClick={() => input.current?.click()} disabled={busy}>
            {busy ? <Loader2 className="mr-2 h-4 w-4 animate-spin" /> : <Upload className="mr-2 h-4 w-4" />}
            {busy ? "Importing" : "Choose export file"}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
