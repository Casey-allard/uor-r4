// hologram-forge (ours, replaces src/hooks/useAppThumbnails.ts; see VENDORED.md): an app's cover and card
// words come only from the source-checked IndexedDB gallery boundary. The host announces each new cover with a "hologram:covers" event; the query refetches.
import { useEffect, useMemo } from "react";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { queryKeys } from "@/lib/queryKeys";

export interface AppCard {
  thumbnailUrl: string | null;
  description: string | null;
  tag: string | null;
}

const EVENT = "hologram:covers";
interface GalleryRecord {
  sourceHash: string;
  revision: number;
  picture: Blob | null;
  card: { description: string; tag: string } | null;
}
declare global {
  interface Window {
    __gallery: { read: (appId: number) => Promise<GalleryRecord | null> };
  }
}
const urls = new Map<number, { stamp: string; url: string }>();
const emptyCard = (): AppCard => ({
  thumbnailUrl: null,
  description: null,
  tag: null,
});
const release = (appId: number) => {
  const held = urls.get(appId);
  if (held) URL.revokeObjectURL(held.url);
  urls.delete(appId);
};
async function readCard(appId: number): Promise<AppCard> {
  try {
    const record = await window.__gallery.read(appId);
    if (!record) {
      release(appId);
      return emptyCard();
    }
    let thumbnailUrl: string | null = null;
    if (record.picture) {
      const stamp = record.sourceHash + ":" + record.revision;
      const held = urls.get(appId);
      if (held?.stamp === stamp) thumbnailUrl = held.url;
      else {
        release(appId);
        thumbnailUrl = URL.createObjectURL(record.picture);
        urls.set(appId, { stamp, url: thumbnailUrl });
      }
    } else release(appId);
    return {
      thumbnailUrl,
      description: record.card?.description ?? null,
      tag: record.card?.tag ?? null,
    };
  } catch (error) {
    release(appId);
    throw error;
  }
}

function useCards(appIds: number[]): Map<number, AppCard> {
  const sortedIds = useMemo(() => [...appIds].sort((a, b) => a - b), [appIds]);
  const client = useQueryClient();
  useEffect(() => {
    const refresh = () =>
      client.invalidateQueries({ queryKey: queryKeys.apps.thumbnails });
    addEventListener(EVENT, refresh);
    return () => removeEventListener(EVENT, refresh);
  }, [client]);
  const { data, isError } = useQuery({
    queryKey: [...queryKeys.apps.thumbnails, "cards", sortedIds],
    queryFn: async () =>
      Promise.all(
        sortedIds.map(async (appId) => [appId, await readCard(appId)] as const),
      ),
    enabled: sortedIds.length > 0 && typeof indexedDB !== "undefined",
  });
  return useMemo(() => new Map(isError ? [] : (data ?? [])), [data, isError]);
}

/** Upstream's signature: `Map<appId, thumbnailUrl>`, for every screen that only shows the picture. */
export function useAppThumbnails(appIds: number[]): Map<number, string | null> {
  const cards = useCards(appIds);
  return useMemo(
    () => new Map([...cards].map(([id, c]) => [id, c.thumbnailUrl])),
    [cards],
  );
}

/** One app's card: its picture, its sentence and its tag. */
export function useAppCard(appId: number): AppCard {
  const ids = useMemo(() => [appId], [appId]);
  return (
    useCards(ids).get(appId) ?? {
      thumbnailUrl: null,
      description: null,
      tag: null,
    }
  );
}
