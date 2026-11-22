// prism-prism: Publish as a Hologram application. Ours, not Prism's (see VENDORED.md). The words come
// from the verified View through the host; the host composes the .holo through PrismPM's own archive
// code in the core, keeps it at its address on this device, and serves it under that address.
// An app with rules is built with the Prism SDK instead (shell/host.js publishTurn): the button says
// "publish" in the app's chat, as a person would, so the build streams there.
import { useCallback, useEffect, useState } from "react";
import { useAtomValue } from "jotai";
import { selectedChatIdAtom } from "@/atoms/chatAtoms";
import { useStreamChat } from "@/hooks/useStreamChat";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";

type Words = Record<string, string>;
type Published = { decision: string; word?: string; kappa?: string; url?: string; byteLength?: number; ms?: number; viewBytes?: number };
type Installed = { kappa: string; url?: string; application?: string; installed: number; byteLength: number; appId?: number };

const ipc = () => (window as any).electron.ipcRenderer;
const short = (k?: string) => (k ? k.replace(/^blake3:/, "").slice(0, 12) + "…" : "");

export function HologramSection({ appId }: { appId: number }) {
  const [words, setWords] = useState<Words | null>(null);
  const [apps, setApps] = useState<Installed[]>([]);
  const [busy, setBusy] = useState(false);
  const [result, setResult] = useState<Published | null>(null);
  const chatId = useAtomValue(selectedChatIdAtom);
  const { streamMessage } = useStreamChat();
  const refresh = useCallback(() => {
    ipc().invoke("holo:list", appId).then(setApps).catch(() => setApps([]));
  }, [appId]);
  useEffect(() => {
    ipc().invoke("holo:view").then(setWords).catch(() => setWords(null));
    refresh();
  }, [refresh]);
  if (!words) return null;
  const publish = async () => {
    setBusy(true);
    setResult(null);
    try {
      const rules = await ipc().invoke("prism:status", { appId }).catch(() => null);
      if (rules && rules.spec && chatId) {
        await streamMessage({ prompt: "publish", chatId, redo: false });
        setResult({ decision: "Chat", word: "Publishing in the chat: the Prism SDK builds the app on GitHub Actions." });
        return;
      }
      setResult(await ipc().invoke("holo:publish", { appId }));
      refresh();
    } catch (e: any) {
      setResult({ decision: "Refuse", word: String(e?.message || e) });
    } finally {
      setBusy(false);
    }
  };
  return (
    <Card data-testid="hologram-section">
      <CardHeader className="pb-3">
        <CardTitle>{words.holoLabel}</CardTitle>
      </CardHeader>
      <CardContent className="space-y-3">
        <p className="text-sm text-gray-600 dark:text-gray-400">{words.runsLabel}</p>
        <Button onClick={publish} disabled={busy} data-testid="hologram-publish">
          {busy ? words.publishingLabel : words.publishLabel}
        </Button>
        {result && result.decision === "Accept" && (
          <p className="text-sm" data-testid="hologram-published">
            {words.publishedLabel} · <span className="font-mono">{short(result.kappa)}</span> · {result.byteLength} bytes · {result.ms} ms ·{" "}
            <a className="underline" href={result.url} target="_blank" rel="noreferrer">
              {words.openLabel}
            </a>
          </p>
        )}
        {result && result.decision === "Chat" && (
          <p className="text-sm text-muted-foreground" data-testid="hologram-chat">
            {result.word}
          </p>
        )}
        {result && result.decision !== "Accept" && result.decision !== "Chat" && (
          <p className="text-sm text-red-600 dark:text-red-400" data-testid="hologram-refused">
            {result.word}
          </p>
        )}
        {apps.length > 0 && (
          <ul className="space-y-1 text-sm">
            {apps.map((a) => (
              <li key={a.kappa} className="flex flex-wrap items-center gap-2">
                <span className="font-mono">{short(a.kappa)}</span>
                <span className="text-gray-500">{a.byteLength} bytes</span>
                <a className="underline" href={a.url} target="_blank" rel="noreferrer">
                  {words.openLabel}
                </a>
                <a className="underline" href={a.url ? a.url.replace(/\/$/, "") + ".holo" : "#"} download>
                  {words.downloadLabel}
                </a>
              </li>
            ))}
          </ul>
        )}
      </CardContent>
    </Card>
  );
}
