import type { AppRunRemoteSnapshot } from "./src/app_run/transport";
export function shouldAutoStartBrowserPreview(
  snapshot: AppRunRemoteSnapshot | null | undefined,
  appId: number | null,
): boolean;
export const PREVIEW_AUTO_START_PATCHES: readonly { file: string; from: string; to: string }[];
