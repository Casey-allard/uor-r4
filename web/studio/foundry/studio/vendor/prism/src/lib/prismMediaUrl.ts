/**
 * Builds a prism-media:// protocol URL for serving media files in Electron.
 */
export function buildPrismMediaUrl(appPath: string, fileName: string): string {
  return `prism-media://media/${encodeURIComponent(appPath)}/.prism/media/${encodeURIComponent(fileName)}`;
}

/**
 * Builds a renderer-safe media URL whose filesystem path is resolved in main.
 */
export function buildPrismMediaUrlForApp(
  appId: number,
  fileName: string,
): string {
  return `prism-media://media/app-id/${appId}/.prism/media/${encodeURIComponent(fileName)}`;
}

/**
 * Builds a versioned URL for a bounded media-library thumbnail derivative.
 * The source version lets Chromium cache the derivative without showing stale
 * content after an image is replaced in place.
 */
export function buildPrismMediaThumbnailUrl(
  appPath: string,
  fileName: string,
  modifiedAtMs: number,
  sizeBytes: number,
): string {
  const url = new URL(buildPrismMediaUrl(appPath, fileName));
  url.searchParams.set("thumbnail", "1");
  url.searchParams.set("v", `${modifiedAtMs}:${sizeBytes}`);
  return url.toString();
}
