// FoundrySpecification.initialProjectThemeBoundary: exact imported first-prompt projection.
export const INITIAL_THEME_PATCHES = [
  {
    file: "src/first_prompt/FirstPromptProvider.tsx",
    from: `    async applyTheme(appId) {
      if (settings?.selectedThemeId) {
        await ipc.template.setAppTheme({
          appId,
          themeId: settings.selectedThemeId,
        });
      }
    },`,
    to: `    async applyTheme(appId) {
      // Foundry commits the initial theme with project creation.
      await ipc.template.getAppTheme({ appId });
    },`,
  },
];
