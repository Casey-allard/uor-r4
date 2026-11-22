// hologram-forge (ours, not upstream's; see VENDORED.md): what the shell leaves out of the first load. Each entry
// is a patch in hologram.screens.mjs's form (one exact passage of an upstream file, refused when it is gone), so the
// build and the typecheck apply them together. The measurements that chose them are in studio/PERFORMANCE.md.
//
// The first load is the home page and the composer; everything else arrives when it is asked for:
//  - every route but home is its own chunk, fetched on intent (a hover or touch of a link preloads it);
//  - the preview panel's Code, Publish, Configure, Security, Tests, Plan and Problems panels are chunks of
//    their own, behind a named loading state;
//  - the code highlighter's eighteen grammars load with the first highlighted block, not with the page;
//  - a language other than English loads when it is chosen;
//  - Monaco is fetched when an editor is first drawn, not at import (it was fetched on every page load).

// Routes whose page is a chunk of its own: [route file, import line, component name, module].
const ROUTES = [
  ["src/routes/apps.ts", `import AppsPage from "../pages/apps";`, "AppsPage", "../pages/apps"],
  ["src/routes/plugins.ts", `import PluginsPage from "../pages/plugins";`, "PluginsPage", "../pages/plugins"],
  ["src/routes/templates.ts", `import TemplatesPage from "../pages/templates";`, "TemplatesPage", "../pages/templates"],
  ["src/routes/app-details.tsx", `import AppDetailsPage from "../pages/app-details";`, "AppDetailsPage", "../pages/app-details"],
  ["src/routes/chat.tsx", `import ChatPage from "../pages/chat";`, "ChatPage", "../pages/chat"],
  ["src/routes/settings.tsx", `import SettingsPage from "../pages/settings";`, "SettingsPage", "../pages/settings"],
  ["src/routes/library.ts", `import LibraryHomePage from "@/pages/library-home";`, "LibraryHomePage", "@/pages/library-home"],
  ["src/routes/media.ts", `import MediaPage from "@/pages/media";`, "MediaPage", "@/pages/media"],
  ["src/routes/prompts.ts", `import LibraryPage from "@/pages/library";`, "LibraryPage", "@/pages/library"],
  ["src/routes/themes.ts", `import ThemesPage from "@/pages/themes";`, "ThemesPage", "@/pages/themes"],
];
const routePatches = ROUTES.flatMap(([file, line, name, mod]) => [
  { file, from: line, to: `import { lazyRouteComponent } from "@tanstack/react-router";` },
  { file, from: `  component: ${name},`, to: `  component: lazyRouteComponent(() => import(${JSON.stringify(mod)})),` },
]);

// The panels the preview panel switches between, each a named export of its own file.
const PANELS = [
  ["CodeView", "./CodeView", `<CodeView loading={loading} app={app ?? null} />`, "the code"],
  ["ConfigurePanel", "./ConfigurePanel", `<ConfigurePanel />`, "the configuration"],
  ["PublishPanel", "./PublishPanel", `<PublishPanel />`, "publishing"],
  ["SecurityPanel", "./SecurityPanel", `<SecurityPanel />`, "the security review"],
  ["TestsPanel", "./TestsPanel", `<TestsPanel />`, "the tests"],
  ["PlanPanel", "./PlanPanel", `<PlanPanel />`, "the plan"],
  ["Problems", "./Problems", `<Problems />`, "the problems"],
];
const PANEL_FILE = "src/components/preview_panel/PreviewPanel.tsx";
const panelPatches = [
  {
    file: PANEL_FILE,
    from: `import { useCallback, useEffect, useRef, useState } from "react";`,
    to: `import { lazy, Suspense, useCallback, useEffect, useRef, useState } from "react";
import { Skeleton } from "@/components/ui/skeleton";
// A panel is fetched when it is first shown; until then the panel says what it is getting.
function PanelLoading({ label }: { label: string }) {
  return (
    <div role="status" aria-live="polite" className="flex h-full flex-col gap-3 p-4">
      <span className="text-sm text-muted-foreground">Loading {label}…</span>
      <Skeleton className="h-8 w-1/3" />
      <Skeleton className="min-h-40 w-full flex-1" />
    </div>
  );
}
// The panels a visitor opens next are fetched once the page is idle, so the first switch waits on nothing.
(window.requestIdleCallback ?? ((fn: () => void) => setTimeout(fn, 2000)))(() => {
  void import("./CodeView");
  void import("./PublishPanel");
  void import("./ConfigurePanel");
});`,
  },
  ...PANELS.map(([name, mod]) => ({
    file: PANEL_FILE,
    from: `import { ${name} } from "${mod}";`,
    to: `const ${name} = lazy(() => import("${mod}").then((m) => ({ default: m.${name} })));`,
  })),
  ...PANELS.map(([, , el, label]) => ({
    file: PANEL_FILE,
    from: el,
    to: `<Suspense fallback={<PanelLoading label=${JSON.stringify(label)} />}>${el}</Suspense>`,
  })),
];

// Shiki's grammars: each becomes a module of its own, loaded by the highlighter when it is first made.
const LANGS = ["astro", "css", "graphql", "html", "java", "javascript", "json", "jsx", "less", "markdown", "python", "sass", "scss", "shell", "sql", "tsx", "typescript", "vue"];
const shikiPatches = [
  {
    file: "src/components/chat/CodeHighlight.tsx",
    from: "// common languages\n" + LANGS.map((l) => `import ${l} from "@shikijs/langs/${l}";`).join("\n") + "\n",
    to: "",
  },
  {
    file: "src/components/chat/CodeHighlight.tsx",
    from: "      langs: [\n" + LANGS.map((l) => `        ${l},`).join("\n") + "\n      ],",
    to: "      langs: [\n" + LANGS.map((l) => `        () => import("@shikijs/langs/${l}"),`).join("\n") + "\n      ],",
  },
];

// Locales: English is in the first load; another language is fetched when it is chosen, before the switch.
const OTHER = [["zhCN", "zh-CN"], ["ptBR", "pt-BR"], ["es", "es"], ["ko", "ko"]];
const NS = ["Common", "Settings", "Chat", "Home", "Errors"];
const NAMES = { zhCN: "Chinese Simplified", ptBR: "Brazilian Portuguese", es: "Spanish", ko: "Korean" };
const i18nPatches = [
  {
    file: "src/i18n/index.ts",
    from: OTHER.map(([id, dir]) => `// ${NAMES[id]}\n` + NS.map((n) => `import ${id}${n} from "./locales/${dir}/${n.toLowerCase()}.json";`).join("\n")).join("\n\n") + "\n",
    to: "",
  },
  {
    file: "src/i18n/index.ts",
    from: OTHER.map(([id, dir]) => `  ${/-/.test(dir) ? JSON.stringify(dir) : dir}: {\n` + NS.map((n) => `    ${n.toLowerCase()}: ${id}${n},`).join("\n") + "\n  },").join("\n") + "\n};",
    to: "};",
  },
  {
    file: "src/i18n/index.ts",
    from: "export default i18n;",
    to: `const NAMESPACES = ["common", "settings", "chat", "home", "errors"] as const;
const LOCALES: Record<string, () => Promise<{ default: Record<string, unknown> }[]>> = {
${OTHER.map(([, dir]) => `  ${JSON.stringify(dir)}: () => Promise.all([${NS.map((n) => `import("./locales/${dir}/${n.toLowerCase()}.json")`).join(", ")}]),`).join("\n")}
};
const loaded = new Set(["en"]);
async function loadLanguage(lng: string) {
  const load = LOCALES[lng];
  if (loaded.has(lng) || !load) return;
  (await load()).forEach((mod, i) => i18n.addResourceBundle(lng, NAMESPACES[i], mod.default, true, true));
  loaded.add(lng);
}
const changeLanguage = i18n.changeLanguage.bind(i18n);
i18n.changeLanguage = (async (lng?: string, ...rest: unknown[]) => {
  if (lng) await loadLanguage(lng);
  return (changeLanguage as (...a: unknown[]) => ReturnType<typeof changeLanguage>)(lng, ...rest);
}) as typeof i18n.changeLanguage;

export default i18n;`,
  },
];

// Monaco: its loader fetched the editor (some 750 KB from a CDN) when this module was imported, which the
// chat's message tree does on every page. The themes are now defined on the first init an editor asks for.
const monacoPatches = [
  {
    file: "src/components/chat/monaco.ts",
    from: `if (!IS_VITEST) {
  loader.init().then((monaco) => {`,
    to: `if (!IS_VITEST) {
  const init = loader.init.bind(loader);
  let themed = false;
  loader.init = () => {
    const ready = init();
    const configured = ready.then((monaco) => {
      try {
        if (!themed) {`,
  },
  {
    file: "src/components/chat/monaco.ts",
    from: `      noSemanticValidation: true,
    });
  });
}`,
    to: `      noSemanticValidation: true,
    });
          themed = true;
        }
      } catch {
        throw Object.assign(new Error("The browser editor could not be configured"), { code: "editorThemeFailed" });
      }
      return monaco;
    }, (error) => {
      if (error?.type === "cancelation") throw error;
      throw Object.assign(new Error("The browser editor could not load"), { code: "editorLoadFailed" });
    });
    return Object.assign(configured, { cancel: () => ready.cancel() });
  };
}`,
  },
];

// The router preloads a route's chunk on intent, and names what it is loading when a chunk is slow.
const routerPatches = [
  {
    file: "src/router.ts",
    from: `  defaultErrorComponent: ErrorBoundary,`,
    to: `  defaultErrorComponent: ErrorBoundary,
  defaultPreload: "intent",
  defaultPreloadDelay: 40,
  defaultPendingMs: 150,
  defaultPendingMinMs: 200,
  defaultPendingComponent: () =>
    React.createElement("div", { role: "status", "aria-live": "polite", className: "p-6 text-sm text-muted-foreground" }, "Loading the page…"),`,
  },
];

// The build screen's panels no longer animate their width (a layout property): a collapse or expand is
// immediate. Its narrow layout is upstream's MobileBuild (hologram.screens.mjs), shown when the kernel says so.
const CHAT = "src/pages/chat.tsx";
const chatPatches = [
  {
    file: CHAT,
    from: `const DEFAULT_CHAT_PANEL_SIZE = 50;`,
    to: `const DEFAULT_CHAT_PANEL_SIZE = 50;

// Whether the build screen shows one pane at a time or the two panels side by side is the Forge kernel's
// decision for the window's width (FK-16 and FK-17, proved by Lean: one pane under 768 px). The host asks it
// as soon as the kernel is ready and keeps the answer, so the screen starts in the kernel's layout; it is
// asked again here on load and on every resize.
const LAST_LAYOUT = "holo.layout.narrow";
function useNarrow() {
  const [narrow, setNarrow] = useState(() => { try { return localStorage.getItem(LAST_LAYOUT) === "1"; } catch { return false; } });
  useEffect(() => {
    let live = true;
    const decide = () => {
      const kernel = window.__forgeKernel;
      if (!kernel) return;
      void kernel.ask("layout " + Math.max(0, Math.round(innerWidth))).then((answer) => {
        if (!live || (answer !== "tabs" && answer !== "split")) return;
        setNarrow(answer === "tabs");
        try { localStorage.setItem(LAST_LAYOUT, answer === "tabs" ? "1" : "0"); } catch { /* storage is a convenience */ }
      });
    };
    decide();
    addEventListener("resize", decide);
    return () => { live = false; removeEventListener("resize", decide); };
  }, []);
  return narrow;
}`,
  },
  // Two identical passages, the chat panel's and the preview panel's: each patch takes the first left.
  { file: CHAT, from: `className={cn(!isResizing && "transition-all duration-100 ease-in-out")}`, to: `className={cn("min-w-0", isResizing && "select-none")}` },
  { file: CHAT, from: `className={cn(!isResizing && "transition-all duration-100 ease-in-out")}`, to: `className={cn("min-w-0", isResizing && "select-none")}` },
];

// The first prompt waited 2,000 ms on Home after the stream had started before it opened the chat ("settle"),
// time upstream's desktop backend takes to accept a stream. This host accepts the submit as it is made, and the
// state machine still waits for its own acceptance and the preview decision; upstream's own test mode runs it at 0.
const settlePatches = [
  { file: "src/app/layout.tsx", from: `settleDelayMs={settings?.isTestMode ? 0 : 2_000}`, to: `settleDelayMs={0}` },
];

// The idea chips under the home composer were hand-styled <button>s: they are the kit's outline Button, pill
// shaped, so they press, focus and disable the way every other control does.
const HOME = "src/pages/home.tsx";
const chipPatches = [
  // A prompt on Home opens the chat; the chat page is a chunk of its own, and while it was still arriving
  // the router kept Home on screen at /chat and Home collapsed under the visitor (CLS 0.29 on a phone).
  // Home fetches it once the page is idle, so Enter finds it ready.
  {
    file: HOME,
    from: `export default function HomePage() {`,
    to: `(window.requestIdleCallback ?? ((fn: () => void) => setTimeout(fn, 2000)))(() => { void import("./chat"); });

export default function HomePage() {`,
  },
  {
    file: HOME,
    retag: `<button
                type="button"
                key={item.label}
                disabled={isCheckingProviders}
                onClick={() => setInputValue(item.prompt)}
                className="flex cursor-pointer items-center gap-2 rounded-full border border-border bg-background px-3.5 py-1.5 text-sm text-muted-foreground transition-colors hover:border-primary/30 hover:bg-accent hover:text-foreground disabled:cursor-default disabled:opacity-50"
              >`,
    to: `<Button type="button" variant="outline" size="sm" key={item.label} disabled={isCheckingProviders} onClick={() => setInputValue(item.prompt)} className="rounded-full text-muted-foreground">`,
    imports: `import { Button } from "@/components/ui/button";`,
  },
];

// A screen that asks on mount for a capability the browser host does not have gets a typed refusal
// (unsupportedChannel, FA-15) and draws its own empty state; a toast for it greeted every visitor on Home
// with four red errors (prompts, themes, custom themes, pending user input). Background queries for such a capability are quiet;
// an action the visitor takes (a mutation) still says it is unavailable.
const quietPatches = [
  {
    file: "src/renderer.tsx",
    from: `    onError: (error, query) => {
      if (query.meta?.showErrorToast) {`,
    to: `    onError: (error, query) => {
      if (query.meta?.showErrorToast && (error as { code?: string } | null)?.code !== "unsupportedChannel") {`,
  },
  // The same for the pending user-input requests the renderer hydrates at start: the browser host has none.
  {
    file: "src/user_input/read_model.ts",
    from: `      void hydrate().catch((error) => showErrorToast(error));`,
    to: `      void hydrate().catch((error) => { if ((error as { code?: string } | null)?.code !== "unsupportedChannel") showErrorToast(error); });`,
  },
  // A typed error's class name is a protocol identifier the host and renderer share, not a word for the
  // visitor: toasts and messages built with \`\${error}\` printed "PrismError: ...". The error reads as its message.
  {
    file: "src/errors/prism_error.ts",
    from: `    this.name = "PrismError";
    this.kind = kind;
    this.cause = options?.cause;
  }
}`,
    to: `    this.name = "PrismError";
    this.kind = kind;
    this.cause = options?.cause;
  }

  toString(): string {
    return this.message;
  }
}`,
  },
];

export const LEAN = [...quietPatches, ...settlePatches, ...chipPatches, ...routePatches, ...panelPatches, ...shikiPatches, ...i18nPatches, ...monacoPatches, ...routerPatches, ...chatPatches];
