// hologram-forge (ours, replaces src/components/FeaturedAppShowcase.tsx; see VENDORED.md): the floor of the
// home screen. Four ways to look at it, in one row: Search, Featured, My apps, Bookmarked.
//
// Featured is the same four Prism apps for every visitor (shell/featured-apps.mjs), so the screen is whole
// before anything has been made. My apps is what this browser holds. Bookmarked is what the visitor starred;
// a star is one person's convenience on one device, so it is kept in this browser's own storage, never
// travels, and is forgotten by a browser that keeps nothing. Nothing here reaches the network.
//
// A cover is drawn, not fetched: white light entering the card and leaving it as a spectrum, turned to the
// app's own hue. Each is a few hundred bytes.
//
// This file is given to the build as the source of src/components/FeaturedAppShowcase.tsx, so what it
// imports is resolved from there: two levels up is the vendored renderer's own folder.
import { useCallback, useEffect, useMemo, useRef, useState, type ReactNode } from "react";
import { useNavigate, useSearch } from "@tanstack/react-router";
import { useSetAtom } from "jotai";
import { ArrowRight, Maximize, Play, RectangleHorizontal, RotateCw, Search, Share2, SquarePen, Star } from "lucide-react";
import { homeChatInputValueAtom } from "@/atoms/chatAtoms";
import { useLoadApps } from "@/hooks/useLoadApps";
import { useOpenApp } from "@/hooks/useOpenApp";
import { useRunApp } from "@/hooks/useRunApp";
import { useCurrentAppUrl } from "@/hooks/useAppRun";
import { useAppCard, useAppThumbnails } from "@/hooks/useAppThumbnails";
import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";
import { sortAppsForShowcase } from "@/lib/sortApps";
import { featuredApps, featuredOnHome } from "../../hologram.featured-apps.mjs";

type Featured = { id: string; by: string; name: string; sentence: string; tag: string; idea: string; hue: number; url: string };
type View = "featured" | "mine" | "saved";

const BOOKMARKS = "foundry:bookmarks";

function readBookmarks(): string[] {
  try {
    const kept = JSON.parse(window.localStorage.getItem(BOOKMARKS) ?? "[]");
    return Array.isArray(kept) ? kept.filter((key) => typeof key === "string") : [];
  } catch {
    return [];
  }
}

function useBookmarks() {
  const [kept, setKept] = useState<string[]>([]);
  useEffect(() => setKept(readBookmarks()), []);
  const toggle = useCallback((key: string) => {
    setKept((was) => {
      const next = was.includes(key) ? was.filter((one) => one !== key) : [...was, key];
      try {
        window.localStorage.setItem(BOOKMARKS, JSON.stringify(next));
      } catch {
        return was;
      }
      return next;
    });
  }, []);
  return { kept, toggle };
}

function Cover({ hue }: { hue: number }) {
  const band = (index: number) => `hsl(${(hue + index * 26) % 360} 86% 64%)`;
  return (
    <svg viewBox="0 0 288 144" preserveAspectRatio="xMidYMid slice" aria-hidden="true" className="absolute inset-0 h-full w-full">
      <defs>
        <linearGradient id={`g-${hue}-ground`} x1="0" y1="0" x2="1" y2="1">
          <stop offset="0%" stopColor={`hsl(${hue} 34% 13%)`} />
          <stop offset="100%" stopColor={`hsl(${(hue + 40) % 360} 28% 7%)`} />
        </linearGradient>
        <linearGradient id={`g-${hue}-beam`} x1="0" y1="0" x2="1" y2="0">
          <stop offset="0%" stopColor="#ffffff" stopOpacity="0.9" />
          <stop offset="100%" stopColor="#ffffff" stopOpacity="0.06" />
        </linearGradient>
      </defs>
      <rect width="288" height="144" fill={`url(#g-${hue}-ground)`} />
      <polygon points="0,58 96,66 96,78 0,86" fill={`url(#g-${hue}-beam)`} />
      {[0, 1, 2, 3, 4].map((index) => (
        <polygon key={index} points={`96,70 288,${18 + index * 26} 288,${40 + index * 26}`} fill={band(index)} opacity={0.42 - index * 0.045} />
      ))}
      <polygon points="88,52 112,72 88,92" fill="#ffffff" opacity="0.1" />
    </svg>
  );
}

function Keep({ on, label, onClick }: { on: boolean; label: string; onClick: () => void }) {
  return (
    <button
      type="button"
      aria-label={label}
      aria-pressed={on}
      data-slot="home-featured-keep"
      data-chosen={on ? "" : undefined}
      onClick={(event) => {
        event.stopPropagation();
        onClick();
      }}
    >
      <Star className="h-3.5 w-3.5" fill={on ? "currentColor" : "none"} aria-hidden="true" />
    </button>
  );
}

function Card({
  title,
  under,
  cover,
  badge,
  keep,
  onOpen,
}: {
  title: string;
  under: string;
  cover: ReactNode;
  badge: string | null;
  keep: ReactNode;
  onOpen: () => void;
}) {
  return (
    <div className="min-w-0">
      <div
        role="button"
        tabIndex={0}
        onClick={onOpen}
        onKeyDown={(event) => {
          if (event.target !== event.currentTarget) return;
          if (event.key === "Enter" || event.key === " ") {
            event.preventDefault();
            onOpen();
          }
        }}
        title={title}
        data-testid={`app-showcase-card-${title}`}
        className="group flex w-full min-w-0 cursor-pointer flex-col gap-2.5 rounded-xl text-left focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-4 focus-visible:ring-offset-background"
      >
        <div className="relative aspect-[2/1] w-full overflow-hidden rounded-xl border border-border bg-muted transition-colors duration-200 group-hover:border-foreground/20">
          {cover}
          {badge && (
            <span className="absolute bottom-2.5 left-2.5 rounded-md bg-background/90 px-2 py-0.5 text-[11px] font-medium leading-5 text-foreground">
              {badge}
            </span>
          )}
          {keep}
        </div>
        <div className="flex min-w-0 flex-col px-0.5">
          <span className="truncate text-[15px] font-medium leading-snug text-foreground">{title}</span>
          <span className="truncate text-[13px] leading-5 text-muted-foreground">{under}</span>
        </div>
      </div>
    </div>
  );
}

export function FeaturedAppShowcase() {
  const navigate = useNavigate();
  const openApp = useOpenApp();
  const { apps } = useLoadApps();
  const { kept, toggle } = useBookmarks();
  const [view, setView] = useState<View>("featured");
  const [asking, setAsking] = useState(false);
  const [query, setQuery] = useState("");

  const featured = useMemo(() => (featuredApps as Featured[]).slice(0, featuredOnHome), []);
  const mine = useMemo(() => sortAppsForShowcase(apps).slice(0, featuredOnHome), [apps]);
  const thumbnails = useAppThumbnails(useMemo(() => mine.map((app) => app.id), [mine]));

  const asked = query.trim().toLowerCase();
  const matches = (text: string) => !asked || text.toLowerCase().includes(asked);
  const shownFeatured = featured.filter(
    (app) => (view === "saved" ? kept.includes("featured:" + app.id) : view === "featured") && matches(app.name),
  );
  const shownMine = mine.filter(
    (app) => (view === "saved" ? kept.includes("app:" + app.id) : view === "mine") && matches(app.name),
  );
  const nothing = shownFeatured.length === 0 && shownMine.length === 0;

  const take = (app: Featured) => {
    if (app.url) window.open(app.url, "_blank", "noopener,noreferrer");
    else window.dispatchEvent(new CustomEvent("foundry:idea", { detail: app.idea }));
  };

  const pill = (id: View, label: string) => (
    <button key={id} type="button" data-chosen={view === id ? "" : undefined} onClick={() => setView(id)}>
      {label}
    </button>
  );

  return (
    <section data-testid="featured-app-showcase" className="w-full max-w-6xl mx-auto px-8 mt-8 mb-12">
      <div className="mb-4 flex items-center justify-between gap-4">
        <div data-slot="home-featured-views" className="flex min-w-0 items-center">
          <button
            type="button"
            data-slot="home-featured-search"
            aria-label="Search apps"
            aria-expanded={asking}
            data-chosen={asking ? "" : undefined}
            onClick={() => {
              if (asking) setQuery("");
              setAsking((was) => !was);
            }}
          >
            <Search className="h-3.5 w-3.5" aria-hidden="true" />
            {!asking && <span>Search</span>}
          </button>
          {asking && (
            <input
              autoFocus
              value={query}
              onChange={(event) => setQuery(event.target.value)}
              onKeyDown={(event) => {
                if (event.key === "Escape") {
                  setQuery("");
                  setAsking(false);
                }
              }}
              placeholder="Find an app"
              aria-label="Find an app"
              data-slot="home-featured-query"
            />
          )}
          {pill("featured", "Featured")}
          {pill("mine", "My apps")}
          {pill("saved", "Bookmarked")}
        </div>
        <button
          type="button"
          onClick={() => navigate({ to: "/apps" })}
          className="flex shrink-0 items-center gap-1 text-sm font-medium text-primary hover:underline"
        >
          Browse all
          <ArrowRight className="w-4 h-4" />
        </button>
      </div>
      <div className="relative">
        <div
          tabIndex={0}
          role="region"
          aria-label="App showcase"
          className="flex gap-4 overflow-x-auto scrollbar-on-hover pb-3 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary focus-visible:ring-offset-2 rounded-lg"
        >
          {shownFeatured.map((app) => (
            <Card
              key={app.id}
              title={app.name}
              under={app.by}
              badge={app.url ? "Published" : "Coming soon"}
              cover={<Cover hue={app.hue} />}
              keep={<Keep on={kept.includes("featured:" + app.id)} label={`Bookmark ${app.name}`} onClick={() => toggle("featured:" + app.id)} />}
              onOpen={() => take(app)}
            />
          ))}
          {shownMine.map((app) => {
            const picture = thumbnails.get(app.id) ?? null;
            return (
              <Card
                key={app.id}
                title={app.name}
                under="Yours"
                badge={null}
                cover={
                  picture ? (
                    <img src={picture} alt="" loading="lazy" className="absolute inset-0 h-full w-full object-cover object-top" />
                  ) : (
                    <Cover hue={(app.id * 47) % 360} />
                  )
                }
                keep={<Keep on={kept.includes("app:" + app.id)} label={`Bookmark ${app.name}`} onClick={() => toggle("app:" + app.id)} />}
                onOpen={() => openApp(app.id)}
              />
            );
          })}
          {nothing && (
            <p data-slot="home-featured-empty" className="py-6 text-sm text-muted-foreground">
              {view === "saved"
                ? "Nothing bookmarked yet. The star on a card keeps it here."
                : view === "mine"
                  ? "No apps yet. Describe one in the box above."
                  : "Nothing matches that."}
            </p>
          )}
        </div>
      </div>
    </section>
  );
}

// Theater (FA-92; FoundryExperience theaterInSidebar to theaterKeys): Prism apps played one after another, as a
// viewer watches a stream. It plays from this module's catalogue, covers and bookmarks, so a star on the floor
// is the same star here. A visitor's app runs exactly as the build preview runs it: started through the run
// state, at the address the run state answers, under the preview's own sandbox.
type Playable = { key: string; name: string; by: string; tag: string; sentence: string; idea: string; hue: number; picture: string | null; url: string | null; appId: number | null };
type Shelf = "all" | View;
// The floor's glass (appearance.css, home-featured): it reads over Immersive's photo and stays quiet on Dark and Light.
const GLASS = "border border-foreground/10 bg-popover/60 backdrop-blur-[21px] backdrop-saturate-150";

const typing = (target: EventTarget | null) =>
  target instanceof HTMLElement && (target.isContentEditable || ["INPUT", "TEXTAREA", "SELECT"].includes(target.tagName));

function Art({ item }: { item: Playable }) {
  return item.picture ? <img src={item.picture} alt="" className="absolute inset-0 h-full w-full object-cover object-top" /> : <Cover hue={item.hue} />;
}

function StageButton({ label, onClick, children }: { label: string; onClick: () => void; children: ReactNode }) {
  return (
    <button type="button" title={label} aria-label={label} onClick={onClick} className="inline-flex h-[34px] w-[34px] items-center justify-center rounded-full text-muted-foreground transition-colors hover:bg-muted hover:text-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring">
      {children}
    </button>
  );
}

export function TheaterPage() {
  const navigate = useNavigate();
  const openApp = useOpenApp();
  const setIdea = useSetAtom(homeChatInputValueAtom);
  const { apps } = useLoadApps();
  const { kept, toggle } = useBookmarks();
  const { runApp, restartApp } = useRunApp();
  const { play } = useSearch({ strict: false }) as { play?: string };
  const [shelf, setShelf] = useState<Shelf>("all");
  const [wide, setWide] = useState(false);
  const [more, setMore] = useState(false);
  const [copied, setCopied] = useState(false);
  const [reload, setReload] = useState(0);
  const stage = useRef<HTMLDivElement>(null);

  const mine = useMemo(() => sortAppsForShowcase(apps), [apps]);
  const thumbnails = useAppThumbnails(useMemo(() => mine.map((app) => app.id), [mine]));
  const all = useMemo<Playable[]>(
    () => [
      ...(featuredApps as Featured[]).map((app) => ({ key: "featured:" + app.id, name: app.name, by: app.by, tag: app.tag, sentence: app.sentence, idea: app.idea, hue: app.hue, picture: null, url: app.url || null, appId: null })),
      ...mine.map((app) => ({ key: "app:" + app.id, name: app.name, by: "Yours", tag: "My app", sentence: "", idea: "", hue: (app.id * 47) % 360, picture: thumbnails.get(app.id) ?? null, url: null, appId: app.id })),
    ],
    [mine, thumbnails],
  );
  const current = all.find((item) => item.key === play) ?? all[0];
  const card = useAppCard(current.appId ?? 0);
  const { appUrl } = useCurrentAppUrl(current.appId);
  const source = current.url ?? appUrl;
  const sentence = current.sentence || card.description || "An app you made with Prism, kept in this browser.";
  const tag = current.appId === null ? current.tag : (card.tag ?? current.tag);
  const shown = all.filter((item) =>
    shelf === "all" ? true : shelf === "featured" ? item.appId === null : shelf === "mine" ? item.appId !== null : kept.includes(item.key),
  );

  const choose = useCallback((key: string) => {
    setMore(false);
    navigate({ to: "/theater", search: { play: key } });
  }, [navigate]);
  const next = useCallback(() => choose(all[(all.indexOf(current) + 1) % all.length].key), [all, current, choose]);
  const full = useCallback(() => {
    if (document.fullscreenElement) void document.exitFullscreen();
    else void stage.current?.requestFullscreen();
  }, []);

  useEffect(() => {
    if (current.appId !== null) void runApp(current.appId);
  }, [current.appId, runApp]);
  useEffect(() => {
    const onKey = (event: KeyboardEvent) => {
      if (typing(event.target) || event.ctrlKey || event.metaKey || event.altKey) return;
      if (event.key === "f") full();
      else if (event.key === "t") setWide((was) => !was);
      else if (event.key === "N" && event.shiftKey) next();
      else return;
      event.preventDefault();
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [full, next]);

  const share = () => {
    void navigator.clipboard?.writeText(window.location.href).then(() => {
      setCopied(true);
      setTimeout(() => setCopied(false), 1597);
    });
  };
  const build = () => {
    setIdea(current.idea);
    navigate({ to: "/" });
  };
  const builder = () => {
    if (current.appId !== null) openApp(current.appId);
    else if (current.url) window.open(current.url, "_blank", "noopener,noreferrer");
    else build();
  };
  const restart = () => {
    if (current.appId !== null) void restartApp({ appId: current.appId });
    else setReload((was) => was + 1);
  };
  const live = Boolean(source);
  const unpublished = current.appId === null && !current.url;
  const saved = kept.includes(current.key);

  return (
    <div className="w-full overflow-y-auto px-[21px] py-[21px]">
      <div className={cn("mx-auto flex max-w-[1597px] flex-col gap-[21px]", !wide && "xl:flex-row xl:items-start")}>
        <main className="min-w-0 flex-1">
          <div ref={stage} className="relative mx-auto aspect-video max-h-[61.8vh] w-full max-w-[calc(61.8vh*16/9)] overflow-hidden rounded-[13px] bg-muted">
            {source ? (
              <iframe
                key={source + reload}
                src={source}
                title={current.name}
                className="absolute inset-0 h-full w-full border-0 bg-background"
                sandbox="allow-same-origin allow-scripts allow-forms allow-popups allow-modals allow-orientation-lock allow-pointer-lock allow-presentation allow-downloads"
                allow="clipboard-read; clipboard-write; fullscreen; autoplay; picture-in-picture"
              />
            ) : (
              <>
                <Art item={current} />
                <div className="absolute inset-0 flex flex-col items-center justify-center gap-[13px] bg-background/40 text-center backdrop-blur-[2px]">
                  {unpublished ? (
                    <>
                      <p className="text-[13px] text-foreground/80">Not published yet</p>
                      <Button onClick={build} className="h-[34px] rounded-full px-[21px]">
                        <Play className="h-3.5 w-3.5" fill="currentColor" aria-hidden="true" /> Build it now
                      </Button>
                    </>
                  ) : (
                    <p className="text-[13px] text-foreground/80">Starting</p>
                  )}
                </div>
              </>
            )}
          </div>
          <div className={cn("mx-auto mt-[13px] max-w-[calc(61.8vh*16/9)] rounded-[21px] p-[21px] pt-[13px]", GLASS)}>
            <div className="flex h-[34px] items-center justify-between">
              <span className="flex items-center gap-[8px] text-[13px] text-muted-foreground">
                <span className={cn("h-[8px] w-[8px] rounded-full", live ? "bg-primary" : "bg-muted-foreground/40")} />
                {live ? "Live" : unpublished ? "Not published yet" : "Starting"}
              </span>
              <span className="flex items-center gap-[5px]">
                <StageButton label="Restart" onClick={restart}><RotateCw className="h-4 w-4" /></StageButton>
                <StageButton label={wide ? "Default view (t)" : "Wide view (t)"} onClick={() => setWide((was) => !was)}><RectangleHorizontal className="h-4 w-4" /></StageButton>
                <StageButton label="Full screen (f)" onClick={full}><Maximize className="h-4 w-4" /></StageButton>
              </span>
            </div>
            <h1 className="mt-[13px] text-[21px] font-semibold leading-tight tracking-tight text-foreground">{current.name}</h1>
            <div className="mt-[13px] flex flex-wrap items-center justify-between gap-[13px]">
              <span className="flex items-center gap-[8px]">
                <span className="flex h-[34px] w-[34px] items-center justify-center rounded-full bg-muted text-[13px] font-semibold text-foreground">{current.by.charAt(0)}</span>
                <span className="text-[13px] font-medium text-foreground">{current.by}</span>
              </span>
              <div data-slot="home-featured-views" className="flex shrink-0 items-center whitespace-nowrap">
                <button type="button" aria-pressed={saved} data-chosen={saved ? "" : undefined} onClick={() => toggle(current.key)}>
                  <Star className="h-3.5 w-3.5" fill={saved ? "currentColor" : "none"} aria-hidden="true" /> Bookmark
                </button>
                <button type="button" onClick={share}><Share2 className="h-3.5 w-3.5" aria-hidden="true" /> {copied ? "Copied" : "Share"}</button>
                <button type="button" onClick={builder}><SquarePen className="h-3.5 w-3.5" aria-hidden="true" /> Open<span className="max-sm:hidden"> in builder</span></button>
              </div>
            </div>
            <div className="mt-[13px] rounded-[13px] bg-foreground/5 p-[13px] text-[13px] leading-5">
              <span className="font-medium text-foreground">{tag}</span>
              <p className="mt-[5px] text-foreground">{sentence}</p>
              {current.idea && (more ? (
                <p className="mt-[5px] text-muted-foreground">Built from the idea: {current.idea}</p>
              ) : (
                <button type="button" onClick={() => setMore(true)} className="mt-[5px] font-medium text-foreground hover:underline">more</button>
              ))}
            </div>
          </div>
        </main>
        {!wide && (
          <aside aria-label="Up next" className={cn("w-full shrink-0 rounded-[21px] p-[13px] xl:w-[377px]", GLASS)}>
            <div data-slot="home-featured-views" className="mb-[13px] inline-flex items-center">
              {([["all", "All"], ["featured", "Featured"], ["mine", "My apps"], ["saved", "Bookmarked"]] as [Shelf, string][]).map(([id, label]) => (
                <button key={id} type="button" data-chosen={shelf === id ? "" : undefined} onClick={() => setShelf(id)}>{label}</button>
              ))}
            </div>
            <ol className="flex flex-col gap-[5px]">
              {shown.map((item) => {
                const playing = item.key === current.key;
                return (
                  <li key={item.key}>
                    <button
                      type="button"
                      aria-current={playing ? "true" : undefined}
                      onClick={() => choose(item.key)}
                      className={cn("flex w-full gap-[8px] rounded-[13px] p-[5px] text-left transition-colors hover:bg-foreground/5 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring", playing && "bg-foreground/10")}
                    >
                      <span className="relative h-[89px] w-[144px] shrink-0 overflow-hidden rounded-[8px] border border-border">
                        <Art item={item} />
                        {playing && (
                          <span className="absolute bottom-[5px] left-[5px] inline-flex items-center gap-[5px] rounded-md bg-background/90 px-[5px] text-[11px] font-medium leading-5 text-foreground">
                            <Play className="h-3 w-3" fill="currentColor" aria-hidden="true" /> Playing
                          </span>
                        )}
                      </span>
                      <span className="flex min-w-0 flex-col gap-[5px] py-[2px]">
                        <span className="line-clamp-2 text-[13px] font-medium leading-5 text-foreground">{item.name}</span>
                        <span className="truncate text-[12px] text-muted-foreground">{item.by} · {item.tag}</span>
                      </span>
                    </button>
                  </li>
                );
              })}
              {shown.length === 0 && <li className="py-[13px] text-[13px] text-muted-foreground">Nothing here yet.</li>}
            </ol>
          </aside>
        )}
      </div>
    </div>
  );
}
