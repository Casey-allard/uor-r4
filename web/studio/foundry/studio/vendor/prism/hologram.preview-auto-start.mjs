// FoundrySpecification.previewAutoStartBoundary: a remount is not a retry request.
export function shouldAutoStartBrowserPreview(snapshot, appId) {
  return (
    Number.isSafeInteger(appId) &&
    appId > 0 &&
    snapshot?.appId === appId &&
    snapshot.phase === "idle" &&
    snapshot.operation === null &&
    snapshot.startedAt === null &&
    snapshot.lastSettlement === null
  );
}
const file = "src/components/preview_panel/PreviewPanel.tsx";
export const PREVIEW_AUTO_START_PATCHES = [
  {
    file,
    from: 'import { usePreviewReloadToken } from "@/hooks/useAppRun";',
    to: 'import { usePreviewReloadToken, useAppRunState } from "@/hooks/useAppRun";\nimport { shouldAutoStartBrowserPreview } from "../../../hologram.preview-auto-start.mjs";',
  },
  {
    file,
    from: "  const { runApp, loading } = useRunApp();",
    to: "  const { runApp, loading } = useRunApp();\n  const browserRunState = useAppRunState(selectedAppId);\n  const autoStartBrowserPreview = shouldAutoStartBrowserPreview(browserRunState, selectedAppId);",
  },
  {
    file,
    from: '        runAppLifecycleInBackground("start", runApp(selectedAppId));',
    to: '        if (autoStartBrowserPreview) {\n          runAppLifecycleInBackground("start", runApp(selectedAppId));\n        }',
  },
  {
    file,
    from: "  }, [selectedAppId, runApp, notifyAppSelected, nodeVersion]);",
    to: "  }, [selectedAppId, runApp, notifyAppSelected, nodeVersion, autoStartBrowserPreview]);",
  },
];
