// hologram-forge (ours, not upstream's; see VENDORED.md): the build screen on a phone. One pane at a time over the
// full width (the chat, or the app, or its publishing), chosen from one bar at the bottom: back to the apps,
// Chat / Preview / Publish as the kit's Tabs, and a slot the shell's appearance switch takes. Built only from
// the kit's shadcn/ui components. Both panes stay mounted, so the running app and the chat keep their state
// when the bar switches. Preview and Publish set the preview panel's own mode, the one its toolbar sets.
import { useEffect, useState, type ReactNode } from "react";
import { useSetAtom } from "jotai";
import { useNavigate } from "@tanstack/react-router";
import { ArrowLeft } from "lucide-react";
import { previewModeAtom } from "@/atoms/appAtoms";
import { Button } from "@/components/ui/button";
import { Tabs, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { cn } from "@/lib/utils";

type Pane = "chat" | "preview" | "publish";
const PANES: { value: Pane; label: string }[] = [
  { value: "chat", label: "Chat" },
  { value: "preview", label: "Preview" },
  { value: "publish", label: "Publish" },
];

export function MobileBuild({ chat, preview }: { chat: ReactNode; preview: ReactNode }) {
  const [pane, setPane] = useState<Pane>("chat");
  const setPreviewMode = useSetAtom(previewModeAtom);
  const navigate = useNavigate();

  // The shell's stylesheet reads the pane from <html> (appearance.css: the switch moves into the bar).
  useEffect(() => {
    const root = document.documentElement;
    root.dataset.holoPane = pane;
    return () => {
      delete root.dataset.holoPane;
    };
  }, [pane]);

  const choose = (value: string) => {
    const next = value as Pane;
    if (next !== "chat") setPreviewMode(next === "publish" ? "publish" : "preview");
    setPane(next);
  };

  return (
    <div className="flex h-full min-h-0 w-full flex-col">
      <div className="relative min-h-0 flex-1">
        <div id="chat-panel" className={cn("absolute inset-0", pane !== "chat" && "hidden")}>
          {chat}
        </div>
        <div id="preview-panel" className={cn("absolute inset-0", pane === "chat" && "hidden")}>
          {preview}
        </div>
      </div>
      <nav
        aria-label="Build"
        className="grid shrink-0 grid-cols-[2.75rem_1fr_2.75rem] items-center gap-3 border-t bg-background/85 px-3.5 pt-2.5 pb-[calc(0.625rem+env(safe-area-inset-bottom))] backdrop-blur-xl"
      >
        <Button
          variant="ghost"
          size="icon"
          className="size-11 rounded-full"
          aria-label="Back to apps"
          onClick={() => navigate({ to: "/" })}
        >
          <ArrowLeft className="size-5" />
        </Button>
        <Tabs value={pane} onValueChange={choose} className="mx-auto w-full max-w-xs">
          <TabsList className="grid h-11 w-full grid-cols-3 rounded-full p-1">
            {PANES.map((p) => (
              <TabsTrigger key={p.value} value={p.value} className="rounded-full text-[15px]">
                {p.label}
              </TabsTrigger>
            ))}
          </TabsList>
        </Tabs>
        <span aria-hidden="true" className="size-11" />
      </nav>
    </div>
  );
}
