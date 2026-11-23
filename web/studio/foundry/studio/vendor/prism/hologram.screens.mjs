// hologram-forge (ours, not upstream's; see VENDORED.md): the screens the shell draws its own way, built only
// from the kit's shadcn/ui components after shadcn's blocks (dashboard-01: an inset sidebar, a site header,
// section cards, tables). SCREENS swaps an upstream file for ours whole; PATCHES replace one exact passage of an
// upstream file and refuse the build when the passage is not found, so an upstream change is seen, not missed.
// Upstream's files stay verbatim. The vite config and the typecheck both apply this module.
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import ts from "typescript";
import { privacySource } from "./hologram.privacy.mjs";
import { LEAN } from "./hologram.lean.mjs";
import {PREVIEW_AUTO_START_PATCHES} from "./hologram.preview-auto-start.mjs";
import {CHAT_MODE_PATCHES} from "./hologram.chat-mode-patches.mjs";
import {INITIAL_THEME_PATCHES} from "./hologram.initial-theme.mjs";
import {PLAIN_CHAT_PATCHES} from "./hologram.plain-chat-patches.mjs";
import {PENDING_MESSAGE_PATCHES} from "./hologram.pending-message-patches.mjs";
import {NO_REMOTE_MODEL_PATCHES} from "./hologram.no-remote-model.mjs";

const here = path.dirname(fileURLToPath(import.meta.url));
const slash = (p) => p.replace(/\\/g, "/");

export const SCREENS = {
  "src/components/app-sidebar.tsx": "hologram.screens/AppSidebar.tsx",
  "src/pages/apps.tsx": "hologram.screens/AppsPage.tsx",
  // The site header's breadcrumb leads back; a page draws no back link of its own.
  "src/components/ui/back-button.tsx": "hologram.screens/BackButton.tsx",
  // App covers: the card shows the app's own first screen, its tag and one two row sentence (shell/covers.mjs).
  "src/components/AppShowcaseCard.tsx": "hologram.screens/AppShowcaseCard.tsx",
  "src/hooks/useAppThumbnails.ts": "hologram.screens/useAppThumbnails.ts",
  // The home's floor: the apps already made, or, before the first one, the gallery's templates.
  "src/components/FeaturedAppShowcase.tsx": "hologram.screens/FeaturedAppShowcase.tsx",
};

// A page's body in the inset, as dashboard-01 spaces it: the site header names the page, so the page keeps
// its description and its actions, not a second title.
const PAGE = "w-full overflow-y-auto px-4 py-4 md:py-6 lg:px-6";

const SCREEN_PATCHES = [
  ...INITIAL_THEME_PATCHES,
  ...CHAT_MODE_PATCHES,
  ...PREVIEW_AUTO_START_PATCHES,
  {
    file: "src/pages/settings.tsx",
    from: `    <section
      id={id}
      className={cn(
        "grid grid-cols-1 gap-x-12 gap-y-5 border-t py-8 first:border-t-0 first:pt-0 md:grid-cols-[minmax(150px,190px)_1fr]",
        "scroll-mt-20",
        tone === "danger" ? "border-destructive/20" : "border-border/60",
      )}
    >
      <div className="self-start md:sticky md:top-6">
        <h2
          className={cn(
            "text-[15px] font-semibold tracking-tight",
            tone === "danger" ? "text-destructive" : "text-foreground",
          )}
        >
          {title}
        </h2>
        {description && <p className={cn("mt-1.5", hint)}>{description}</p>}
      </div>
      <div className="min-w-0 space-y-6">{children}</div>
    </section>`,
    to: `    <Card id={id} className={cn("scroll-mt-20 shadow-xs", tone === "danger" && "border-destructive/40")}>
      <CardHeader>
        <CardTitle className={cn(tone === "danger" && "text-destructive")}>{title}</CardTitle>
        {description && <CardDescription>{description}</CardDescription>}
      </CardHeader>
      <CardContent className="min-w-0 space-y-6">{children}</CardContent>
    </Card>`,
  },
  {
    file: "src/pages/settings.tsx",
    from: `import { Button } from "@/components/ui/button";`,
    to: `import { Button } from "@/components/ui/button";\nimport { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";`,
  },
  {
    file: "src/pages/settings.tsx",
    from: `    <div className="min-h-screen bg-background">
      <div className="mx-auto max-w-5xl px-6 py-8 md:px-10">
        <BackButton />

        <header className="mt-1 mb-9">
          <h1 className="text-2xl font-semibold tracking-tight text-foreground">
            Settings
          </h1>
          <p className={cn("mt-1", hint)}>
            Manage your preferences, providers, and integrations.
          </p>
        </header>

        <div>`,
    to: `    <div className="${PAGE}">
      <div className="mx-auto max-w-4xl">
        <p className={cn("mb-6", hint)}>
          Manage your preferences, providers, and integrations.
        </p>

        <div className="flex flex-col gap-4 md:gap-6">`,
  },
  {
    file: "src/pages/plugins.tsx",
    from: `    <div className="w-full min-h-screen px-8 py-4">
      <div className="max-w-5xl pb-12">
        <BackButton />`,
    to: `    <div className="${PAGE}">
      <div className="pb-12">`,
  },
  {
    file: "src/pages/plugins.tsx",
    from: `            <h1 className="text-3xl font-bold text-gray-900 dark:text-white mb-2">
              Plugins
            </h1>
            <p className="text-md text-gray-600 dark:text-gray-400">`,
    to: `            <p className="text-muted-foreground">`,
  },
  {
    file: "src/pages/templates.tsx",
    from: `    <div className="min-h-screen px-8 py-4">
      <div className="max-w-5xl mx-auto pb-12">
        <BackButton />
        <header className="mb-8 text-left">
          <h1 className="text-3xl font-bold text-gray-900 dark:text-white mb-2">
            Pick your default template
          </h1>
          <p className="text-md text-gray-600 dark:text-gray-400">`,
    to: `    <div className="${PAGE}">
      <div className="pb-12">
        <header className="mb-6 text-left">
          <p className="text-muted-foreground">`,
  },
  {
    file: "src/pages/library-home.tsx",
    from: `    <div className="min-h-screen w-full">
      <div className="px-8 py-6">
        <div className="max-w-6xl mx-auto">
          {/* Header */}
          <div className="flex items-center justify-between mb-6">
            <h1 className="text-3xl font-bold">
              <BookOpen className="inline-block h-8 w-8 mr-2" />
              Library
            </h1>`,
    to: `    <div className="${PAGE}">
      <div>
        <div>
          {/* Header */}
          <div className="flex items-center justify-between mb-6">
            <p className="text-muted-foreground">Themes, prompts and media you can use in any app.</p>`,
  },
  {
    // Theater (FA-92) lives in the floor's module, which already holds the catalogue, the covers and the
    // bookmarks it plays from, so the route is declared here rather than in a route file of its own.
    file: "src/router.ts",
    from: `import { mediaRoute } from "./routes/media";`,
    to: `import { mediaRoute } from "./routes/media";
import { Route } from "@tanstack/react-router";
import { TheaterPage } from "./components/FeaturedAppShowcase";
const theaterRoute = new Route({ getParentRoute: () => rootRoute, path: "/theater", component: TheaterPage,
  validateSearch: (search: Record<string, unknown>): { play?: string } => (typeof search.play === "string" ? { play: search.play } : {}) });`,
  },
  { file: "src/router.ts", from: `  mediaRoute,\n  chatRoute,`, to: `  mediaRoute,\n  theaterRoute,\n  chatRoute,` },
  {
    // The page sits in the sidebar's inset, under a site header, as in dashboard-01.
    file: "src/app/layout.tsx",
    from: `import { AppSidebar } from "@/components/app-sidebar";`,
    to: `import { AppSidebar, MobileTabBar, SiteHeader } from "@/components/app-sidebar";\nimport { SidebarInset } from "@/components/ui/sidebar";`,
  },
  {
    file: "src/app/layout.tsx",
    from: `<SidebarProvider defaultOpen={false}>`,
    to: `<SidebarProvider style={{ "--sidebar-width": "calc(var(--spacing) * 64)", "--header-height": "calc(var(--spacing) * 12)" } as React.CSSProperties}>`,
  },
  {
    file: "src/app/layout.tsx",
    from: `                <div className="flex h-screenish min-w-0 flex-1 flex-col overflow-hidden mt-[var(--layout-title-bar-offset)] border-l border-border bg-background">
                  <SubscriptionStatusBanner />
                  <div
                    id="layout-main-content-container"
                    className="flex min-h-0 w-full flex-1 overflow-x-hidden"
                  >
                    {children}
                  </div>
                </div>`,
    to: `                <SidebarInset className="h-[calc(100svh-(--spacing(4)))] min-w-0 overflow-hidden">
                  <SiteHeader />
                  <SubscriptionStatusBanner />
                  <div
                    id="layout-main-content-container"
                    className="flex min-h-0 w-full flex-1 overflow-x-hidden"
                  >
                    {children}
                  </div>
                </SidebarInset>
                <MobileTabBar />`,
  },
  {
    // Its items carry a description under the label; Radix's SelectValue shows the chosen item whole, so the
    // trigger names the label alone, as Base UI's showed one line.
    file: "src/components/ZoomSelector.tsx",
    from: `<SelectValue placeholder={t("general.selectZoom")} />`,
    to: `<SelectValue placeholder={t("general.selectZoom")}>{ZOOM_LEVEL_LABELS[currentZoomLevel as ZoomLevel]}</SelectValue>`,
  },
  {
    // The app's details sit on the kit's Card, not a box styled to look like one.
    file: "src/pages/app-details.tsx",
    retag: `<div className="w-full max-w-2xl mx-auto mt-10 p-4 bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 shadow-sm relative">`,
    to: `<Card className="relative mx-auto mt-6 w-full max-w-2xl gap-0 p-4 shadow-xs">`,
  },
  {
    file: "src/pages/app-details.tsx",
    from: `<span>{selectedApp.createdAt.toString()}</span>`,
    to: `<span>{new Date(selectedApp.createdAt).toLocaleString(undefined, { dateStyle: "medium", timeStyle: "short" })}</span>`,
  },
  {
    file: "src/pages/app-details.tsx",
    from: `<span>{selectedApp.updatedAt.toString()}</span>`,
    to: `<span>{new Date(selectedApp.updatedAt).toLocaleString(undefined, { dateStyle: "medium", timeStyle: "short" })}</span>`,
  },
  // The build screen on a narrow screen: one pane at a time and a bar at the bottom
  // (hologram.screens/MobileBuild.tsx) instead of the two resizable panels. Which applies is the Forge kernel's
  // answer for the window's width (FK-16, FK-17; useNarrow, hologram.lean.mjs).
  {
    file: "src/pages/chat.tsx",
    from: `import { ipc } from "@/ipc/types";`,
    to: `import { ipc } from "@/ipc/types";
import { MobileBuild } from "../../hologram.screens/MobileBuild";`,
  },
  {
    file: "src/pages/chat.tsx",
    from: `  return (
    <PanelGroup autoSaveId="persistence" direction="horizontal">`,
    to: `  const isMobile = useNarrow();
  if (isMobile) {
    return (
      <MobileBuild
        chat={<ChatPanel chatId={chatId} isPreviewOpen={true} onTogglePreview={() => {}} />}
        preview={<PreviewPanel />}
      />
    );
  }
  return (
    <PanelGroup autoSaveId="persistence" direction="horizontal">`,
  },
  // Chat and Plan: the same screen with its preview collapsed, and suggestions above the composer.
  ...PLAIN_CHAT_PATCHES,
  // A sent prompt is on screen at once, before the host has stored it.
  ...PENDING_MESSAGE_PATCHES,
  // Home's one entry point is the composer: importing an app is an item in its + menu, not a button above it.
  {
    file: "src/pages/home.tsx",
    from: `            <div className="mt-4 flex justify-center">
              <ImportAppButton
                className="px-0 pb-0"
                variant="outline"
                size="sm"
              />
            </div>
`,
    to: ``,
  },
  {
    file: "src/pages/home.tsx",
    from: `import { ImportAppButton } from "@/components/ImportAppButton";\n`,
    to: ``,
  },
  // The suggestion chips' row is named, so the shell can give the chips glass that reads over any photo.
  {
    file: "src/pages/home.tsx",
    from: `<div className="mt-4 flex flex-wrap justify-center gap-2">`,
    to: `<div className="mt-4 flex flex-wrap justify-center gap-2" data-slot="home-suggestions">`,
  },
  {
    file: "src/pages/home.tsx",
    from: `    return shuffled.slice(0, 3);`,
    to: `    return shuffled.slice(0, 2);`,
  },
  // A featured app with no link yet puts its idea in the box, where the visitor can send it as their own.
  {
    file: "src/pages/home.tsx",
    from: `  // Initialize random prompts
  useEffect(() => {
    setRandomPrompts(getRandomPrompts());
  }, [getRandomPrompts]);`,
    to: `  // Initialize random prompts
  useEffect(() => {
    setRandomPrompts(getRandomPrompts());
  }, [getRandomPrompts]);

  useEffect(() => {
    const take = (event: Event) => {
      const idea = (event as CustomEvent<string>).detail;
      if (typeof idea === "string" && idea) setInputValue(idea);
    };
    window.addEventListener("foundry:idea", take);
    return () => window.removeEventListener("foundry:idea", take);
  }, [setInputValue]);`,
  },
  // Three chips, always: two ideas and "Surprise me", which writes one more into the box. The chip that
  // asked for another three ideas is gone; a fresh two are offered on every visit.
  {
    file: "src/pages/home.tsx",
    from: `            <button
              type="button"
              disabled={isCheckingProviders}
              onClick={() => setRandomPrompts(getRandomPrompts())}
              className="group flex cursor-pointer items-center gap-2 rounded-full border border-border bg-background px-3.5 py-1.5 text-sm text-muted-foreground transition-colors hover:border-primary/30 hover:bg-accent hover:text-foreground disabled:cursor-default disabled:opacity-50"
            >
              <RefreshCw className="size-4 transition-transform duration-200 group-hover:rotate-[-25deg]" />
              {t("moreIdeas")}
            </button>
`,
    to: `            <Button
              type="button"
              variant="outline"
              size="sm"
              data-slot="home-surprise"
              disabled={isCheckingProviders}
              onClick={() => {
                const ideas = INSPIRATION_PROMPTS.filter((p) => p.prompt !== inputValue);
                setInputValue(ideas[Math.floor(Math.random() * ideas.length)].prompt);
              }}
              className="rounded-full text-muted-foreground"
            >
              <Sparkles aria-hidden="true" />
              Surprise me
            </Button>
`,
  },
  // On a phone the page keeps the inset's 16 px gutter instead of 32 px, so the composer has the room.
  {
    file: "src/pages/home.tsx",
    from: `max-w-3xl w-full m-auto p-8 relative"`,
    to: `max-w-3xl w-full m-auto px-4 py-8 sm:p-8 relative" data-slot="home-stage"`,
  },
  // On a phone home is one screen (appearance.css): a short headline, the composer, and one "Surprise me"
  // that writes an idea into it, in place of the chips; the featured apps live on the Apps page there.
  {
    file: "src/pages/home.tsx",
    from: `<div className="flex min-h-full w-full flex-col pb-28">`,
    to: `<div className="flex min-h-full w-full flex-col pb-28" data-slot="home-page">`,
  },
  {
    file: "src/pages/home.tsx",
    from: `              What do you want to build?
            </h1>
            <p className="mx-auto mt-3 max-w-2xl text-base leading-7 text-muted-foreground">
              Describe your idea. Prism will turn it into a working app.
            </p>`,
    to: `              Build your ideas with <span data-slot="home-name">Prism</span>
            </h1>
            <p className="mx-auto mt-3 max-w-2xl text-base leading-7 text-muted-foreground">
              <span data-slot="home-wide">Describe your idea. Prism will turn it into a working app.</span>
              <span data-slot="home-narrow">Describe it. Prism makes it real.</span>
            </p>`,
  },
  {
    file: "src/pages/home.tsx",
    from: `        <PrivacyBanner />
      </div>
      <FeaturedAppShowcase />`,
    to: `        <PrivacyBanner />
      </div>
      <div data-slot="home-featured">
        <FeaturedAppShowcase />
      </div>`,
  },
  {
    file: "src/pages/home.tsx",
    from: `import { RefreshCw, Zap } from "lucide-react";`,
    to: `import { Sparkles, Zap } from "lucide-react";
import { useQueryClient } from "@tanstack/react-query";
import { useLoadApps } from "@/hooks/useLoadApps";
import { useSelectChat } from "@/hooks/useSelectChat";
import { HomeChatThread, HomeModeToggle, useHomeChat, type HomeMode } from "../../hologram.screens/HomeChatSurface";`,
  },
  {
    file: "src/pages/home.tsx",
    from: `  const navigate = useNavigate();`,
    to: `  const navigate = useNavigate();
  const queryClient = useQueryClient();
  const { refreshApps } = useLoadApps();
  const { selectChat } = useSelectChat();
  const [homeMode, setHomeMode] = useState<HomeMode>("chat");
  const handleHomePromoted = useCallback(async (promotedAppId: number, promotedChatId: number) => {
    await refreshApps();
    await queryClient.invalidateQueries();
    selectChat({ appId: promotedAppId, chatId: promotedChatId });
  }, [queryClient, refreshApps, selectChat]);
  const homeChat = useHomeChat({ onPromoted: handleHomePromoted });`,
  },
  {
    file: "src/pages/home.tsx",
    from: `  const handleSubmit = useCallback(
    (options?: HomeSubmitOptions) => {
      const submittedAttachments = options?.attachments ?? [];
      if (!inputValue.trim() && submittedAttachments.length === 0) return false;
      return sendFirstPrompt({
        type: "SUBMIT",
        payload: {
          prompt: inputValue,
          attachments: submittedAttachments,
          selectedApp: options?.selectedApp,
          chatMode: homeSubmitChatMode,
          isChatModeExplicit: hasManuallySelectedChatMode,
        },
      });
    },
    [
      hasManuallySelectedChatMode,
      homeSubmitChatMode,
      inputValue,
      sendFirstPrompt,
    ],
  );`,
    to: `  const handleSubmit = useCallback(
    (options?: HomeSubmitOptions) => {
      const submittedAttachments = options?.attachments ?? [];
      if (!inputValue.trim() && submittedAttachments.length === 0) return false;
      if (homeMode === "chat" && !options?.selectedApp && submittedAttachments.length === 0) {
        const promptText = inputValue;
        setInputValue("");
        void homeChat.sendMessage(promptText);
        return true;
      }
      return sendFirstPrompt({
        type: "SUBMIT",
        payload: {
          prompt: inputValue,
          attachments: submittedAttachments,
          selectedApp: options?.selectedApp,
          chatMode: homeSubmitChatMode,
          isChatModeExplicit: hasManuallySelectedChatMode,
        },
      });
    },
    [
      hasManuallySelectedChatMode,
      homeChat,
      homeMode,
      homeSubmitChatMode,
      inputValue,
      sendFirstPrompt,
      setInputValue,
    ],
  );`,
  },
  {
    file: "src/pages/home.tsx",
    from: `          <HomeChatInput
            onSubmit={handleSubmit}
            disabled={isCheckingProviders}
          />`,
    to: `          <div className="mb-3 flex justify-center">
            <HomeModeToggle
              mode={homeMode}
              onChange={setHomeMode}
              disabled={isCheckingProviders || homeChat.sending || homeChat.promoting}
            />
          </div>
          {homeMode === "chat" && (
            <HomeChatThread
              messages={homeChat.messages}
              sending={homeChat.sending}
              promoting={homeChat.promoting}
              error={homeChat.error}
              hydration={homeChat.hydration}
              onCancel={homeChat.cancel}
              onClear={homeChat.clear}
              onPromote={homeChat.promoteToStudio}
            />
          )}
          <HomeChatInput
            onSubmit={handleSubmit}
            disabled={isCheckingProviders}
          />`,
  },
  {
    file: "src/components/chat/HomeChatInput.tsx",
    from: `<AuxiliaryActionsMenu onFileSelect={handleFileSelect} />`,
    to: `<AuxiliaryActionsMenu onFileSelect={handleFileSelect} showImport />`,
  },
  // The + menu's submenus open in a portal: inside the menu (which scrolls, and transforms as it animates in)
  // they were clipped to its box, so their items could not be seen. A long list scrolls within the screen.
  {
    file: "src/components/chat/AuxiliaryActionsMenu.tsx",
    from: `  DropdownMenuItem,
} from "@/components/ui/dropdown-menu";`,
    to: `  DropdownMenuItem,
  DropdownMenuPortal,
} from "@/components/ui/dropdown-menu";
import { Upload } from "lucide-react";
import { ImportAppDialog } from "@/components/ImportAppDialog";`,
  },
  {
    file: "src/components/chat/AuxiliaryActionsMenu.tsx",
    from: `  onGenerateImage?: () => void;
}`,
    to: `  onGenerateImage?: () => void;
  showImport?: boolean;
}`,
  },
  {
    file: "src/components/chat/AuxiliaryActionsMenu.tsx",
    from: `  onGenerateImage,
}: AuxiliaryActionsMenuProps) {
  const [isOpen, setIsOpen] = useState(false);`,
    to: `  onGenerateImage,
  showImport,
}: AuxiliaryActionsMenuProps) {
  const [isOpen, setIsOpen] = useState(false);
  const [importOpen, setImportOpen] = useState(false);`,
  },
  // The + sits at the composer's left: the menu opens under the composer, clear of the screen's edge.
  {
    file: "src/components/chat/AuxiliaryActionsMenu.tsx",
    from: `<DropdownMenuContent align="end">`,
    to: `<DropdownMenuContent align="start" collisionPadding={8}>`,
  },
  {
    file: "src/components/chat/AuxiliaryActionsMenu.tsx",
    from: `            <DropdownMenuSubContent>
              <FileAttachmentDropdown
                onFileSelect={onFileSelect}
                closeMenu={() => setIsOpen(false)}
              />
            </DropdownMenuSubContent>`,
    to: `            <DropdownMenuPortal>
              <DropdownMenuSubContent className="max-h-(--radix-dropdown-menu-content-available-height) overflow-y-auto">
                <FileAttachmentDropdown
                  onFileSelect={onFileSelect}
                  closeMenu={() => setIsOpen(false)}
                />
              </DropdownMenuSubContent>
            </DropdownMenuPortal>`,
  },
  {
    file: "src/components/chat/AuxiliaryActionsMenu.tsx",
    from: `            <DropdownMenuSubContent>
              <DropdownMenuItem
                onClick={() => handleThemeSelect(null)}`,
    to: `            <DropdownMenuPortal>
            <DropdownMenuSubContent className="max-h-(--radix-dropdown-menu-content-available-height) overflow-y-auto">
              <DropdownMenuItem
                onClick={() => handleThemeSelect(null)}`,
  },
  {
    file: "src/components/chat/AuxiliaryActionsMenu.tsx",
    from: `              </>
            </DropdownMenuSubContent>
          </DropdownMenuSub>
`,
    to: `              </>
            </DropdownMenuSubContent>
            </DropdownMenuPortal>
          </DropdownMenuSub>

          {showImport && (
            <>
              <DropdownMenuSeparator />
              <DropdownMenuItem
                onClick={() => {
                  setIsOpen(false);
                  setImportOpen(true);
                }}
                className="py-2 px-3"
                data-testid="import-app-menu-item"
              >
                <Upload size={16} className="mr-2" />
                Import app
              </DropdownMenuItem>
            </>
          )}
`,
  },
  {
    file: "src/components/chat/AuxiliaryActionsMenu.tsx",
    from: `      </Dialog>
    </>
  );
}`,
    to: `      </Dialog>

      {showImport && <ImportAppDialog isOpen={importOpen} onClose={() => setImportOpen(false)} />}
    </>
  );
}`,
  },
  // Vercel, Coolify, Supabase and Neon are hosted accounts this product does not use (their screens resolve to
  // hologram.removed-integrations.tsx); the places that named them, and their settings, go with them.
  {
    file: "src/pages/settings.tsx",
    from: `            description="Link Prism to the services you deploy and store data with."
          >
            <div id={SETTING_IDS.github}>
              <GitHubIntegration />
            </div>
            <div id={SETTING_IDS.vercel}>
              <VercelIntegration />
            </div>
            <div id={SETTING_IDS.supabase}>
              <SupabaseIntegration />
            </div>
            <div id={SETTING_IDS.neon}>
              <NeonIntegration />
            </div>`,
    to: `            description="Link Prism to GitHub, where your apps can be published."
          >
            <div id={SETTING_IDS.github}>
              <GitHubIntegration />
            </div>`,
  },
  {
    file: "src/pages/settings.tsx",
    from: `            <div
              id={SETTING_IDS.enableOwnServerDeployment}
              className="space-y-1.5"
            >
              <div className="flex items-center gap-2">
                <Switch
                  id="enable-own-server-deployment"
                  aria-label="Allow deployment to your own server"
                  checked={!!settings?.enableOwnServerDeployment}
                  onCheckedChange={(checked) => {
                    updateSettings({ enableOwnServerDeployment: checked });
                  }}
                />
                <Label htmlFor="enable-own-server-deployment">
                  Allow deployment to your own server
                </Label>
              </div>
              <p className={hint}>
                Integrates with Coolify so you can deploy an app to a server you
                run yourself. This is in early development: it is undocumented,
                unstable, and may change or break without notice.
              </p>
            </div>

`,
    to: ``,
  },
  {
    file: "src/lib/settingsSearchIndex.ts",
    from: `  {
    id: SETTING_IDS.vercel,
    label: "Vercel Integration",
    description: "Connect your Vercel account for deployments",
    keywords: ["vercel", "deploy", "integration", "hosting", "connect"],
    sectionId: SECTION_IDS.integrations,
    sectionLabel: "Integrations",
  },
  {
    id: SETTING_IDS.supabase,
    label: "Supabase Integration",
    description: "Connect your Supabase project",
    keywords: [
      "supabase",
      "database",
      "integration",
      "backend",
      "connect",
      "postgres",
    ],
    sectionId: SECTION_IDS.integrations,
    sectionLabel: "Integrations",
  },
  {
    id: SETTING_IDS.neon,
    label: "Neon Integration",
    description: "Connect your Neon database",
    keywords: [
      "neon",
      "database",
      "integration",
      "postgres",
      "connect",
      "serverless",
    ],
    sectionId: SECTION_IDS.integrations,
    sectionLabel: "Integrations",
  },
`,
    to: ``,
  },
  {
    file: "src/lib/settingsSearchIndex.ts",
    from: `  {
    id: SETTING_IDS.enableOwnServerDeployment,
    label: "Allow deployment to your own server",
    description:
      "Integrates with Coolify so you can deploy an app to a server you run yourself",
    keywords: [
      "deploy",
      "deployment",
      "self-hosted",
      "self hosted",
      "own server",
      "coolify",
      "vps",
      "publish",
      "experiment",
    ],
    sectionId: SECTION_IDS.experiments,
    sectionLabel: "Experiments",
  },
`,
    to: ``,
  },
  {
    file: "src/pages/app-details.tsx",
    from: `          {/* When providerFilter is set, show the selected connector only if the other provider isn't already active */}
          {providerFilter === "supabase" &&
            appId &&
            !selectedApp?.neonProjectId && <SupabaseConnector appId={appId} />}
          {providerFilter === "supabase" &&
            appId &&
            selectedApp?.neonProjectId && (
              <UnavailableIntegrationCard provider="supabase" />
            )}
          {providerFilter === "neon" &&
            appId &&
            !selectedApp?.supabaseProjectId && <NeonConnector appId={appId} />}
          {providerFilter === "neon" &&
            appId &&
            selectedApp?.supabaseProjectId && (
              <UnavailableIntegrationCard provider="neon" />
            )}
          {/* When no providerFilter, show both with existing mutual exclusion */}
          {!providerFilter && (
            <>
              {appId &&
                !selectedApp?.neonProjectId &&
                !selectedApp?.supabaseProjectId && (
                  <div className="flex items-start gap-2 rounded-md border border-muted bg-muted/30 px-3 py-2 text-xs text-muted-foreground">
                    <Info className="h-4 w-4 shrink-0 mt-0.5" />
                    <span>{t("integrations.mutualExclusion.chooseOne")}</span>
                  </div>
                )}
              {appId && !selectedApp?.neonProjectId && (
                <SupabaseConnector appId={appId} />
              )}
              {appId && selectedApp?.neonProjectId && (
                <UnavailableIntegrationCard provider="supabase" />
              )}
              {appId && !selectedApp?.supabaseProjectId && (
                <NeonConnector appId={appId} />
              )}
              {appId && selectedApp?.supabaseProjectId && (
                <UnavailableIntegrationCard provider="neon" />
              )}
            </>
          )}
`,
    to: ``,
  },
  {
    file: "src/components/preview_panel/ConfigurePanel.tsx",
    from: `          To modify environment variables for Supabase or production,`,
    to: `          To modify environment variables for production,`,
  },
  // The chat log upload sent the visitor's chat and code to upstream's own server (upload-logs.dyad.sh). The card
  // that started it is gone, the crash dialog opens Help without it, and the request itself is not in the bundle.
  {
    file: "src/components/HelpDialog.tsx",
    from: `          {/* Upload Chat Session */}
          <div className="border rounded-lg p-4 space-y-3 relative">
            <div className="flex items-center gap-2">
              <MessageSquareIcon className="h-4 w-4 text-primary" />
              <span className="text-sm font-semibold">
                AI / Prism Pro issues
              </span>
            </div>
            <p className="text-sm text-muted-foreground">
              Best for AI quality issues. Uploads your chat session and code for
              the team to reproduce and fix the problem.
            </p>
            <Button
              variant="outline"
              onClick={handleUploadChatSession}
              disabled={isUploading || !selectedChatId}
              className="w-full bg-(--background-lightest)"
            >
              <UploadIcon className="mr-2 h-4 w-4" />{" "}
              {isUploading ? "Preparing Upload..." : "Upload Chat Session"}
            </Button>
            {!selectedChatId && (
              <p className="text-xs text-amber-600 dark:text-amber-400 flex items-center gap-1.5">
                <AlertCircleIcon className="h-3 w-3 shrink-0" />
                Open a chat first to upload a session.
              </p>
            )}
          </div>

`,
    to: ``,
  },
  {
    file: "src/components/HelpDialog.tsx",
    from: `      const response = await fetch(
        "https://upload-logs.dyad.sh/generate-upload-url",
        {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({
            extension: "json",
            contentType: "application/json",
          }),
        },
      );`,
    to: `      const response = { ok: false, statusText: "Uploading chat logs is not offered", json: async () => ({ uploadUrl: "", filename: "" }) };`,
  },
  {
    file: "src/components/ForceCloseDialog.tsx",
    from: `    setHelpDialog({ open: true, uploadChatId: activeChatId });`,
    to: `    setHelpDialog({ open: true });`,
  },
];

// The screens' patches, then the ones that keep the first load lean (hologram.lean.mjs).
export const PATCHES = [...SCREEN_PATCHES, ...LEAN, ...NO_REMOTE_MODEL_PATCHES];

// A retag patch: the one JSX element whose opening tag is exactly `retag` becomes `to`, its closing tag
// renamed with it (found by the parser, so the children are kept whole).
function retag(code, rel, p) {
  const at = code.indexOf(p.retag);
  if (at < 0 || code.indexOf(p.retag, at + 1) >= 0) throw new Error(`${rel}: the element the shell retags was not found once:\n${p.retag}`);
  const sf = ts.createSourceFile(rel, code, ts.ScriptTarget.Latest, true, ts.ScriptKind.TSX);
  let el = null;
  const find = (n) => { if (!el && ts.isJsxElement(n) && n.openingElement.getStart() === at) el = n; else ts.forEachChild(n, find); };
  find(sf);
  if (!el) throw new Error(`${rel}: the element the shell retags is not an element with children`);
  const tag = p.to.match(/^<([A-Za-z.]+)/)[1];
  const close = el.closingElement;
  let out = code.slice(0, close.getStart()) + `</${tag}>` + code.slice(close.getEnd());
  out = out.slice(0, at) + p.to + out.slice(at + p.retag.length);
  return p.imports ? p.imports + "\n" + out : out;
}

/** Our file for an upstream file the shell replaces whole (absolute paths), or null. */
export function screenFor(file) {
  const rel = slash(path.relative(here, file));
  return SCREENS[rel] ? path.join(here, SCREENS[rel]) : null;
}

/** An upstream file's source as the shell builds it: ours if replaced, else with its patches applied. */
export function screenSource(file, code) {
  const ours = screenFor(file);
  if (ours) return fs.readFileSync(ours, "utf8");
  const rel = slash(path.relative(here, file));
  let out = code.replace(/\r\n/g, "\n");
  for (const p of PATCHES) {
    if (p.file !== rel) continue;
    if (p.retag) { out = retag(out, rel, p); applied.add(p); continue; }
    if (!out.includes(p.from)) throw new Error(`${rel}: the passage the shell patches was not found:\n${p.from.split("\n")[0]}`);
    out = out.replace(p.from, p.to);
    applied.add(p);
  }
  return privacySource(rel, out);
}

// A patch whose file the build never read did nothing, silently: the build and the typecheck ask for these
// once every file is read, and refuse when any is left.
const applied = new Set();
export function unapplied() {
  return PATCHES.filter((p) => !applied.has(p)).map((p) => `${p.file}: ${(p.from ?? p.retag).split("\n")[0]}`);
}
