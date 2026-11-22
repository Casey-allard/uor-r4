// hologram-forge (ours, not upstream's; see VENDORED.md): Foundry answers with its own language and reasoning
// only, decided by Prism R4 (shell/r4.mjs, FA-87). There is no model provider to connect and no key to give,
// so nothing on any screen asks for one. Upstream gates the first prompt on a configured provider: its saga
// shows "You're almost ready to build" (OpenRouter, Google, other providers, your own API key) when none is,
// and its chat shows a setup banner. Here the one model is always set up, and those screens are not drawn.
export const NO_REMOTE_MODEL_PATCHES = [
  // The one model is always there: the first-prompt saga never waits on a provider.
  {
    file: "src/hooks/useLanguageModelProviders.ts",
    from: `  const isAnyProviderSetup = () => {
    if (`,
    to: `  // Foundry's own model (Prism R4) is always set up; a visitor never connects a provider or a key.
  const isAnyProviderSetup = () => true;
  const isAnyRemoteProviderSetup = () => {
    if (`,
  },
  {
    file: "src/hooks/useLanguageModelProviders.ts",
    from: `    isProviderSetup,
    isAnyProviderSetup,
  };`,
    to: `    isProviderSetup,
    isAnyProviderSetup,
    isAnyRemoteProviderSetup,
  };`,
  },
  // The setup dialog is not drawn, and the saga's request for it does nothing.
  {
    file: "src/first_prompt/FirstPromptProvider.tsx",
    from: `    showSetupDialog() {
      posthog.capture("home:ai-setup-dialog-open");
      setIsSetupDialogOpen(true);
    },`,
    to: `    showSetupDialog() {},`,
  },
  {
    file: "src/first_prompt/FirstPromptProvider.tsx",
    from: `      <Dialog
        open={isSetupDialogOpen}
        onOpenChange={(open) => {
          setIsSetupDialogOpen(open);
          if (!open) controller.send({ type: "SETUP_DISMISSED" });
        }}
      >
        <DialogContent className="p-0 sm:max-w-2xl">
          <DialogHeader className="sr-only">
            <DialogTitle>
              {hasConfiguredProvider
                ? "Manage AI setup"
                : "You're almost ready to build"}
            </DialogTitle>
            <DialogDescription>
              {hasConfiguredProvider
                ? "Change how Prism accesses AI."
                : "Choose how Prism should access AI before generating your app."}
            </DialogDescription>
          </DialogHeader>
          <SetupBanner variant="dialog" forceShow />
        </DialogContent>
      </Dialog>
`,
    to: ``,
  },
  {
    file: "src/first_prompt/FirstPromptProvider.tsx",
    from: `import { SetupBanner } from "@/components/SetupBanner";\n`,
    to: ``,
  },
  // No setup banner in a chat, for a provider or for OpenRouter's free model.
  {
    file: "src/components/chat/MessagesList.tsx",
    from: `      if (
        selectedModel?.name === "free" &&
        selectedModel?.provider === "auto" &&
        !isProviderSetup("openrouter")
      ) {
        return <OpenRouterSetupBanner className="w-full" />;
      }
      if (!isAnyProviderSetup()) {
        return <SetupBanner />;
      }
      return null;`,
    to: `      void selectedModel;
      return null;`,
  },
  {
    file: "src/components/chat/MessagesList.tsx",
    from: `import { OpenRouterSetupBanner, SetupBanner } from "../SetupBanner";\n`,
    to: ``,
  },
  // Settings has no Model Providers section, in the page or its search.
  {
    file: "src/pages/settings.tsx",
    from: `          <SettingsSection
            id={SECTION_IDS.providers}
            title="Model Providers"
            description="Connect the AI providers Prism uses to build and run your apps."
          >
            <ProviderSettingsGrid />
          </SettingsSection>
`,
    to: ``,
  },
  {
    file: "src/pages/settings.tsx",
    from: `import { ProviderSettingsGrid } from "@/components/ProviderSettings";\n`,
    to: ``,
  },
  {
    file: "src/pages/settings.tsx",
    from: `Manage your preferences, providers, and integrations.`,
    to: `Manage your preferences and integrations.`,
  },
  {
    file: "src/lib/settingsSearchIndex.ts",
    from: `  // Provider Settings
  {
    id: SECTION_IDS.providers,
    label: "Model Providers",
    description: "Configure AI model providers and API keys",
    keywords: [
      "provider",
      "model",
      "api key",
      "openai",
      "anthropic",
      "claude",
      "gpt",
      "gemini",
      "llm",
    ],
    sectionId: SECTION_IDS.providers,
    sectionLabel: "Model Providers",
  },
`,
    to: ``,
  },
  // The title bar's Pro button opened a provider's key page.
  {
    file: "src/app/TitleBar.tsx",
    from: `          {isPrismPro && <PrismProButton isPrismProEnabled={isPrismProEnabled} />}`,
    to: `          {null}`,
  },
];
