// hologram-forge (ours, not upstream's; see VENDORED.md): the shell's sidebar and site header, in place of
// src/components/app-sidebar.tsx (hologram.screens.mjs; it is compiled at that path, so its imports
// resolve as upstream's did). The structure is shadcn's dashboard-01 block: an
// inset sidebar that collapses to icons, a primary action, the main navigation, the user's apps, and
// one account row at the foot (Prism ID and its menu at the left, the gear that opens Settings over the page at
// the right), Search (Ctrl or Cmd K) under
// Home, and More, which holds the items the visitor tucks away (Edit sidebar). Every part is the kit's shadcn/ui component; upstream's hooks and atoms
// supply the data and the navigation, so the screens behind it are unchanged.
import {
  Activity,
  BookOpen,
  Blocks,
  ChevronDown,
  CirclePlus,
  Ellipsis,
  FlaskConical,
  HelpCircle,
  House,
  KeyRound,
  LayoutGrid,
  Menu,
  MessageSquare,
  MonitorPlay,
  Plug,
  Search,
  Settings,
  ShieldCheck,
  SlidersHorizontal,
  Sparkles,
  Star,
  Store,
  TriangleAlert,
  UserRound,
  Workflow,
  Wrench,
  X,
  type LucideIcon,
} from "lucide-react";
import { Link, useLocation, useNavigate, useRouter } from "@tanstack/react-router";
import { useAtomValue, useSetAtom } from "jotai";
import { Fragment, Suspense, lazy, useEffect, useMemo, useRef, useState } from "react";
import { toast } from "sonner";

import { selectedAppIdAtom } from "@/atoms/appAtoms";
import { selectedChatIdAtom } from "@/atoms/chatAtoms";
import { helpDialogAtom } from "@/atoms/helpDialogAtom";
import { HelpDialog } from "@/components/HelpDialog";
import { useChats } from "@/hooks/useChats";
import { CommandDialog, CommandEmpty, CommandGroup, CommandInput, CommandItem, CommandList } from "@/components/ui/command";
import type { AppSearchResult } from "@/lib/schemas";
import { useLoadApps } from "@/hooks/useLoadApps";
import { useOpenApp } from "@/hooks/useOpenApp";
import { useSelectChat } from "@/hooks/useSelectChat";
import {
  Breadcrumb,
  BreadcrumbItem,
  BreadcrumbLink,
  BreadcrumbList,
  BreadcrumbPage,
  BreadcrumbSeparator,
} from "@/components/ui/breadcrumb";
import { Button } from "@/components/ui/button";
import { Checkbox } from "@/components/ui/checkbox";
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from "@/components/ui/dialog";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuGroup,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuShortcut,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { Input } from "@/components/ui/input";
import { Separator } from "@/components/ui/separator";
import { Skeleton } from "@/components/ui/skeleton";
import { Tooltip, TooltipContent, TooltipTrigger } from "@/components/ui/tooltip";
import { SETTINGS_SEARCH_INDEX } from "@/lib/settingsSearchIndex";
import {
  Sidebar,
  SidebarContent,
  SidebarFooter,
  SidebarGroup,
  SidebarGroupContent,
  SidebarGroupLabel,
  SidebarHeader,
  SidebarMenu,
  SidebarMenuBadge,
  SidebarMenuButton,
  SidebarMenuItem,
  SidebarMenuSub,
  SidebarMenuSubButton,
  SidebarMenuSubItem,
  SidebarRail,
  SidebarTrigger,
  useSidebar,
} from "@/components/ui/sidebar";

// The product's name, the same word the build's word transform (vite.shell.config.mts) writes on every
// upstream screen.
const PRODUCT = "Prism";
const TITLE = `${PRODUCT} Foundry`;

// Prism's mark: a triangle split into three lit faces. It is drawn in the text colour at three
// strengths, so it follows every appearance without a colour of its own.
function PrismMark({ className }: { className?: string }) {
  return (
    <svg viewBox="0 0 100 100" aria-hidden="true" className={className} fill="currentColor">
      <path d="M50 8 L50 60 L8 86 Z" />
      <path d="M50 8 L92 86 L50 60 Z" fillOpacity={0.6} />
      <path d="M8 86 L50 60 L92 86 Z" fillOpacity={0.3} />
    </svg>
  );
}

// Prism ID is the Prism frame's own panel, so Foundry keeps the one ID every Prism app keeps. The shell runs
// the frame its lock pins (shell/inference.js, prismFrame); its panel is a centred popover in the top layer.
declare global { interface Window { __prismFrame?: () => Promise<HTMLElement>; __workspaceTold?: string[] } }
function openPrismId() {
  window.__prismFrame?.().then((frame) => frame.shadowRoot?.getElementById("account-panel")?.showPopover(), (error) => console.warn(`Prism ID: ${error}`));
}
// What the visitor is told about their work, by the kernel's answer (shell/inference.js, tell). Each sentence
// is a statement of studio/experience/src/FoundryExperience.lex.tex.
const WORKSPACE_SAID: Record<string, string> = {
  workspaceMoved: "Your work is now sealed to your Prism ID. Only your devices can open it.",
  workspaceArrived: "Your work from your other device is here, and it stays sealed to your Prism ID.",
  workspaceDeviceRemoved: "This device was removed from your Prism ID, so the work it kept stays closed. You start fresh here.",
  workspaceQuotaExceeded: "Your browser's storage is full, so this was not saved. Remove an app you no longer need, then try again.",
  recordTampered: "Some of your work was changed outside Foundry, so it was left closed.",
  recordMisplaced: "Some of your work was changed outside Foundry, so it was left closed.",
  workspaceKeyStale: "This device is waiting for your Prism ID's newest key. Open Foundry on one of your other devices, then try again.",
};
const WORKSPACE_CLOSED = "Foundry could not reach your Prism ID, so your work stays closed for now.";
function sayWorkspace(answer: string) {
  const words = WORKSPACE_SAID[answer];
  if (answer === "workspaceMoved" || answer === "workspaceArrived" || answer === "workspaceDeviceRemoved") toast(words, { id: answer });
  else if (words && answer !== "workspaceKeyStale") toast.error(words, { id: answer });
  else toast.error(words ?? WORKSPACE_CLOSED, { id: answer, duration: Infinity, action: { label: "Try again", onClick: () => location.reload() } });
}

type Nav = { title: string; to: string; icon: LucideIcon; exact?: boolean };
const NAV: Nav[] = [
  { title: "Home", to: "/", icon: House, exact: true },
  { title: "Apps", to: "/apps", icon: LayoutGrid },
  { title: "Templates", to: "/templates", icon: Store },
  { title: "Library", to: "/library", icon: BookOpen },
  { title: "Theater", to: "/theater", icon: MonitorPlay },
  { title: "Plugins", to: "/plugins", icon: Blocks },
];
// What the visitor keeps in the sidebar; the rest waits under More. Home, Search and Apps always stay. The choice is a
// convenience of this browser, so it lives in localStorage and a page without it shows the defaults.
const SIDEBAR_KEY = "holo.sidebar.v1";
const OPTIONAL = ["/templates", "/library", "/plugins"];
function readShown(): string[] {
  try {
    const saved = JSON.parse(localStorage.getItem(SIDEBAR_KEY) ?? "null");
    return Array.isArray(saved) ? saved.filter((to) => OPTIONAL.includes(to)) : ["/library"];
  } catch { return ["/library"]; }
}
const RECENT_APPS = 8;
const RECENT_CHATS = 6;

// The path inside the base the shell is served under.
function usePath() {
  const router = useRouter();
  const pathname = useLocation({ select: (l) => l.pathname });
  const base = (router.options.basepath ?? "").replace(/\/$/, "");
  return (pathname.startsWith(base) ? pathname.slice(base.length) : pathname) || "/";
}

// Matched against the path the visitor is on, segment by segment ("$name" matches any one segment; without
// exact, deeper paths match too); home only on its own path. The router's own match read its settled
// location, which lags a navigation while the next page's code loads, and nothing drew again once it
// settled, so after a click the header and the sidebar kept naming the page before.
function useIsActive() {
  const path = usePath();
  const here = path.split("/").filter(Boolean);
  return (to: string, exact?: boolean) => {
    if (to === "/") return path === "/";
    const want = to.split("/").filter(Boolean);
    if (here.length < want.length || (exact && here.length !== want.length)) return false;
    return want.every((seg, i) => seg.startsWith("$") || seg === here[i]);
  };
}

export function AppSidebar() {
  const isActive = useIsActive();
  // On a phone the sidebar is the kit's Sheet over the page: a choice in it (a link, an app, a chat) closes
  // it, so the page the visitor asked for is what they see.
  const { isMobile, setOpenMobile } = useSidebar();
  const href = useLocation({ select: (l) => l.href });
  useEffect(() => { if (isMobile) setOpenMobile(false); }, [href, isMobile, setOpenMobile]);
  useEffect(() => {
    (window.__workspaceTold ?? []).forEach(sayWorkspace);
    const said = (event: Event) => sayWorkspace((event as CustomEvent<string>).detail);
    window.addEventListener("foundry:workspace", said);
    return () => window.removeEventListener("foundry:workspace", said);
  }, []);
  // A tooltip names an item of the collapsed rail. In the Sheet it is never shown, but the kit still opens it
  // when the Sheet focuses its first item, and the first Escape then closed that invisible tooltip, not the
  // Sheet. On a phone the items carry none.
  const tip = (label: string) => (isMobile ? undefined : label);
  const navigate = useNavigate();
  const setHelpDialog = useSetAtom(helpDialogAtom);
  const setSelectedAppId = useSetAtom(selectedAppIdAtom);
  const setSelectedChatId = useSetAtom(selectedChatIdAtom);
  const [searching, setSearching] = useState(false);
  const [settingsOpen, setSettingsOpen] = useState(false);
  // Control or Command with comma opens Settings from anywhere, as desktop apps do.
  useEffect(() => {
    const down = (e: KeyboardEvent) => {
      if (e.key === "," && (e.metaKey || e.ctrlKey) && !e.altKey) { e.preventDefault(); setSettingsOpen(true); }
    };
    document.addEventListener("keydown", down);
    return () => document.removeEventListener("keydown", down);
  }, []);
  const { state } = useSidebar();
  const collapsed = state === "collapsed" && !isMobile;
  const [shown, setShown] = useState(readShown);
  const [editing, setEditing] = useState(false);
  const toggle = (to: string) => setShown((was) => {
    const next = was.includes(to) ? was.filter((t) => t !== to) : [...was, to];
    try { localStorage.setItem(SIDEBAR_KEY, JSON.stringify(next)); } catch {}
    return next;
  });
  const tucked = NAV.filter((n) => OPTIONAL.includes(n.to) && !shown.includes(n.to));
  const { apps } = useLoadApps();
  const openApp = useOpenApp();
  const { selectChat } = useSelectChat();
  const byRecency = useMemo(() => [...apps].sort((a, b) => +new Date(b.updatedAt) - +new Date(a.updatedAt))
    .map((a) => ({ id: a.id, name: a.name, createdAt: a.createdAt, matchedChatTitle: null, matchedChatMessage: null })), [apps]);

  const newApp = () => {
    setSelectedAppId(null);
    setSelectedChatId(null);
    navigate({ to: "/" });
  };

  // Settings opens over the page; on a phone the sidebar's Sheet steps aside first.
  const openSettings = () => { if (isMobile) setOpenMobile(false); setSettingsOpen(true); };
  return (
    <>
    <Sidebar collapsible="icon" variant="inset">
      <SidebarHeader>
        <SidebarMenu>
          <SidebarMenuItem>
            {/* Collapsed to icons, the header is the mark alone, centred over the icons below it. */}
            <SidebarMenuButton asChild size="lg" tooltip={tip(TITLE)} className="group-data-[collapsible=icon]:justify-center">
              <Link to="/" onClick={newApp}>
                <PrismMark className="size-6 shrink-0" />
                <span className="font-display text-base font-semibold group-data-[collapsible=icon]:hidden">{TITLE}</span>
              </Link>
            </SidebarMenuButton>
          </SidebarMenuItem>
        </SidebarMenu>
      </SidebarHeader>
      <SidebarContent>
        <SidebarGroup>
          <SidebarGroupContent className="flex flex-col gap-2">
            <SidebarMenu>
              <SidebarMenuItem>
                <SidebarMenuButton
                  tooltip={tip("New app")}
                  onClick={newApp}
                  className="bg-primary text-primary-foreground hover:bg-primary/90 hover:text-primary-foreground active:bg-primary/90 active:text-primary-foreground"
                >
                  <CirclePlus />
                  <span>New app</span>
                </SidebarMenuButton>
              </SidebarMenuItem>
            </SidebarMenu>
            <SidebarMenu>
              {NAV.filter((n) => !tucked.includes(n)).map((item) => (
                <Fragment key={item.to}>
                  <SidebarMenuItem>
                    <SidebarMenuButton asChild tooltip={tip(item.title)} isActive={isActive(item.to, item.exact)}>
                      <Link to={item.to}>
                        <item.icon />
                        <span>{item.title}</span>
                      </Link>
                    </SidebarMenuButton>
                  </SidebarMenuItem>
                  {item.to === "/" && (
                    <SidebarMenuItem>
                      <SidebarMenuButton tooltip={tip("Search")} onClick={() => setSearching(true)}>
                        <Search />
                        <span>Search</span>
                      </SidebarMenuButton>
                    </SidebarMenuItem>
                  )}
                </Fragment>
              ))}
              <SidebarMenuItem>
                <DropdownMenu modal={false}>
                  <DropdownMenuTrigger asChild>
                    <SidebarMenuButton title="More">
                      <Ellipsis />
                      <span>More</span>
                    </SidebarMenuButton>
                  </DropdownMenuTrigger>
                  <DropdownMenuContent side={isMobile ? "bottom" : "right"} align="start" className="min-w-48">
                    {tucked.map((item) => (
                      <DropdownMenuItem key={item.to} asChild>
                        <Link to={item.to}><item.icon />{item.title}</Link>
                      </DropdownMenuItem>
                    ))}
                    {tucked.length > 0 && <DropdownMenuSeparator />}
                    <DropdownMenuItem onSelect={() => setEditing(true)}>Edit sidebar…</DropdownMenuItem>
                  </DropdownMenuContent>
                </DropdownMenu>
              </SidebarMenuItem>
            </SidebarMenu>
          </SidebarGroupContent>
        </SidebarGroup>
        <RecentApps />
      </SidebarContent>
      {/* One row under a hairline: who you are at the left, the gear at the right. Collapsed, the two marks
          stack and no word is shown. */}
      <SidebarFooter data-slot="account-bar" className="border-t border-sidebar-border p-2">
        <div className="flex items-center gap-[5px] group-data-[collapsible=icon]:flex-col group-data-[collapsible=icon]:gap-[8px]">
          <DropdownMenu modal={false}>
            <DropdownMenuTrigger asChild>
              <button type="button" data-slot="account-trigger" aria-label="Prism ID and account menu" title={collapsed ? "Prism ID" : undefined}
                className="flex h-[34px] min-w-0 max-w-full items-center gap-[8px] rounded-md pr-[8px] pl-[5px] text-left text-sm text-sidebar-foreground outline-hidden transition-colors hover:bg-sidebar-accent hover:text-sidebar-accent-foreground focus-visible:ring-2 focus-visible:ring-sidebar-ring data-[state=open]:bg-sidebar-accent group-data-[collapsible=icon]:size-8 group-data-[collapsible=icon]:flex-none group-data-[collapsible=icon]:justify-center group-data-[collapsible=icon]:px-0">
                <span aria-hidden="true" className="flex size-[21px] shrink-0 items-center justify-center rounded-full bg-sidebar-primary text-sidebar-primary-foreground">
                  <UserRound className="size-[13px]" strokeWidth={2.25} />
                </span>
                <span className="truncate font-medium group-data-[collapsible=icon]:hidden">Prism ID</span>
                <ChevronDown aria-hidden="true" className="-ml-[3px] size-[13px] shrink-0 text-muted-foreground group-data-[collapsible=icon]:hidden" strokeWidth={2.5} />
              </button>
            </DropdownMenuTrigger>
            <DropdownMenuContent side={collapsed ? "right" : "top"} align="start" sideOffset={8}
              className="w-[calc(233px-13px)] p-[5px]">
              <DropdownMenuLabel className="flex items-center gap-[8px] px-[8px] py-[8px] font-normal">
                <span aria-hidden="true" className="flex size-[34px] shrink-0 items-center justify-center rounded-full bg-sidebar-primary text-sidebar-primary-foreground">
                  <UserRound className="size-[16px]" strokeWidth={2.25} />
                </span>
                <span className="min-w-0">
                  <span className="block text-sm font-medium text-foreground">Prism ID</span>
                  <span className="block text-xs leading-snug text-muted-foreground">One ID for every Prism app, no password</span>
                </span>
              </DropdownMenuLabel>
              <DropdownMenuSeparator />
              <DropdownMenuGroup>
                <DropdownMenuItem onSelect={openPrismId}>
                  <KeyRound />Devices and recovery
                </DropdownMenuItem>
                <DropdownMenuItem onSelect={openSettings}>
                  <Settings />Settings
                  <DropdownMenuShortcut>{MOD} ,</DropdownMenuShortcut>
                </DropdownMenuItem>
                <DropdownMenuItem onSelect={() => setHelpDialog({ open: true })}>
                  <HelpCircle />Help
                </DropdownMenuItem>
              </DropdownMenuGroup>
            </DropdownMenuContent>
          </DropdownMenu>
          <span aria-hidden="true" className="flex-1 group-data-[collapsible=icon]:hidden" />
          <Tooltip>
            <TooltipTrigger asChild>
              <button type="button" data-slot="settings-trigger" aria-label="Open settings" onClick={openSettings}
                className="flex size-[34px] shrink-0 items-center justify-center rounded-md text-sidebar-foreground/70 outline-hidden transition-colors hover:bg-sidebar-accent hover:text-sidebar-accent-foreground focus-visible:ring-2 focus-visible:ring-sidebar-ring group-data-[collapsible=icon]:size-8">
                <Settings className="size-[18px]" aria-hidden="true" />
              </button>
            </TooltipTrigger>
            <TooltipContent side={collapsed ? "right" : "top"} hidden={isMobile}>Settings</TooltipContent>
          </Tooltip>
        </div>
        <HelpDialog />
      </SidebarFooter>
      <SidebarRail />
      <SearchPalette open={searching} onOpenChange={setSearching} allApps={byRecency} onSelectApp={openApp}
        onSelectChat={(chatId, appId) => selectChat({ chatId, appId })} onNewApp={newApp} />
      <Dialog open={editing} onOpenChange={setEditing}>
        <DialogContent className="sm:max-w-sm">
          <DialogHeader>
            <DialogTitle>Edit sidebar</DialogTitle>
            <DialogDescription>Choose which items appear in your sidebar. The rest stay under More.</DialogDescription>
          </DialogHeader>
          <div className="flex flex-col">
            {NAV.filter((n) => OPTIONAL.includes(n.to)).map((item) => (
              <label key={item.to} className="flex cursor-pointer items-center gap-3 rounded-md px-2 py-2 text-sm hover:bg-accent">
                <Checkbox checked={shown.includes(item.to)} onCheckedChange={() => toggle(item.to)} />
                <item.icon className="size-4 text-muted-foreground" />
                {item.title}
              </label>
            ))}
          </div>
          <DialogFooter>
            <Button onClick={() => setEditing(false)}>Done</Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </Sidebar>
    {/* Mounted only while open: on the build screen, whose page changes on every edit, it costs nothing. */}
    {settingsOpen && <SettingsOverlay open onOpenChange={setSettingsOpen} />}
    </>
  );
}

const MOD = typeof navigator !== "undefined" && /Mac|iPhone|iPad/.test(navigator.platform) ? "⌘" : "Ctrl";

// Settings over the page, as a desktop app keeps it: centred, the sections at the left and one at a time at the
// right. The page itself is upstream's settings screen as the build patches it (loaded with its own chunk, as
// its route is), so every control and its storage are unchanged; the sections are read from what it draws,
// never listed twice, and the search runs over upstream's own index of settings.
const SettingsPage = lazy(() => import("@/pages/settings"));
const SECTION_MARKS: Record<string, LucideIcon> = {
  "general-settings": SlidersHorizontal,
  "workflow-settings": Workflow,
  "ai-settings": Sparkles,
  telemetry: Activity,
  integrations: Plug,
  "agent-permissions": ShieldCheck,
  advanced: Wrench,
  experiments: FlaskConical,
  "danger-zone": TriangleAlert,
};
type Section = { id: string; title: string };

function SettingsOverlay({ open, onOpenChange }: { open: boolean; onOpenChange: (open: boolean) => void }) {
  const [pane, setPane] = useState<HTMLDivElement | null>(null);
  const [sections, setSections] = useState<Section[]>([]);
  const [active, setActive] = useState<string | null>(null);
  const [query, setQuery] = useState("");
  const flash = useRef<HTMLElement | null>(null);
  // A link inside Settings leads to another page: the overlay steps aside for it.
  const href = useLocation({ select: (l) => l.href });
  const openedAt = useRef(href);
  useEffect(() => { if (href !== openedAt.current) onOpenChange(false); }, [href]); // eslint-disable-line react-hooks/exhaustive-deps
  // The sections are the cards the settings page draws, each with its id and its title, in order.
  useEffect(() => {
    if (!pane) return;
    const read = () => {
      const found = [...pane.querySelectorAll<HTMLElement>('[data-slot="card"][id]')].map((card) => ({
        id: card.id,
        title: card.querySelector('[data-slot="card-title"]')?.textContent?.trim() || card.id,
      }));
      setSections((was) => (was.map((s) => s.id).join() === found.map((s) => s.id).join() ? was : found));
    };
    read();
    const watch = new MutationObserver(read);
    watch.observe(pane, { childList: true, subtree: true });
    return () => watch.disconnect();
  }, [pane]);
  const shown = active && sections.some((s) => s.id === active) ? active : sections[0]?.id ?? null;
  const q = query.trim().toLowerCase();
  const matches = q
    ? SETTINGS_SEARCH_INDEX.filter((item) =>
        sections.some((s) => s.id === item.sectionId) &&
        [item.label, item.description, item.sectionLabel, ...item.keywords].some((w) => w.toLowerCase().includes(q)))
    : [];
  const go = (sectionId: string, settingId?: string) => {
    setActive(sectionId);
    setQuery("");
    requestAnimationFrame(() => {
      const target = settingId ? pane?.querySelector<HTMLElement>(`[id="${settingId}"]`) : null;
      if (!target) { pane?.scrollTo({ top: 0 }); return; }
      target.scrollIntoView({ block: "center" });
      flash.current?.removeAttribute("data-found");
      target.setAttribute("data-found", "");
      flash.current = target;
      setTimeout(() => target.removeAttribute("data-found"), 1600);
    });
  };
  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent showCloseButton={false} data-slot="settings-overlay" aria-describedby={undefined}
        onOpenAutoFocus={(e) => e.preventDefault()}
        className="flex h-[min(550px,85vh)] w-[min(890px,90vw)] max-w-none gap-0 overflow-hidden border-[color-mix(in_srgb,var(--popover-foreground)_12%,transparent)] bg-[color-mix(in_srgb,var(--popover)_97%,transparent)]! p-0 text-popover-foreground shadow-[0_34px_89px_-21px_rgb(0_0_0/0.6)] backdrop-blur-[28px] sm:max-w-none max-md:h-dvh max-md:w-screen max-md:flex-col max-md:rounded-none max-md:border-0">
        <nav aria-label="Settings sections" className="flex w-[233px] shrink-0 flex-col gap-[8px] border-r bg-muted/40 p-[13px] max-md:w-full max-md:border-r-0 max-md:border-b">
          {/* On a phone the close sits before the title: the page's own switches hold the top right corner. */}
          <div className="flex items-center gap-[5px] px-[5px] pt-[5px] pb-[5px] max-md:-ml-[8px]">
            <button type="button" aria-label="Close settings" onClick={() => onOpenChange(false)}
              className="flex size-[34px] items-center justify-center rounded-md text-muted-foreground hover:bg-accent hover:text-foreground md:hidden">
              <X className="size-4" />
            </button>
            <DialogTitle className="text-base font-semibold">Settings</DialogTitle>
          </div>
          <div className="relative">
            <Search aria-hidden="true" className="pointer-events-none absolute top-1/2 left-[8px] size-4 -translate-y-1/2 text-muted-foreground" />
            <Input value={query} onChange={(e) => setQuery(e.target.value)} placeholder="Search settings" aria-label="Search settings"
              className="h-[34px] pl-[29px] text-sm" />
          </div>
          {q ? (
            <div className="flex min-h-0 flex-col gap-px overflow-y-auto max-md:max-h-[34vh]">
              {matches.length === 0 && <p className="px-[8px] py-[8px] text-sm text-muted-foreground">No setting matches.</p>}
              {matches.map((item) => (
                <button key={item.id} type="button" onClick={() => go(item.sectionId, item.id)}
                  className="flex flex-col rounded-md px-[8px] py-[5px] text-left hover:bg-accent">
                  <span className="truncate text-sm">{item.label}</span>
                  <span className="truncate text-xs text-muted-foreground">{item.sectionLabel}</span>
                </button>
              ))}
            </div>
          ) : (
            <div className="flex min-h-0 flex-col gap-px overflow-y-auto max-md:flex-row max-md:overflow-x-auto max-md:overflow-y-hidden">
              {sections.length === 0 && [0, 1, 2, 3].map((i) => <Skeleton key={i} className="h-[34px] w-full" />)}
              {sections.map((s) => {
                const Mark = SECTION_MARKS[s.id] ?? SlidersHorizontal;
                const on = s.id === shown;
                return (
                  <button key={s.id} type="button" title={s.title} aria-current={on ? "page" : undefined} onClick={() => go(s.id)}
                    className={`flex h-[34px] shrink-0 items-center gap-[8px] rounded-md px-[8px] text-left text-sm transition-colors hover:bg-accent ${on ? "bg-accent font-medium" : ""} ${s.id === "danger-zone" ? "text-destructive" : on ? "text-foreground" : "text-muted-foreground"}`}>
                    <Mark aria-hidden="true" className="size-4 shrink-0" />
                    <span className="truncate">{s.title}</span>
                  </button>
                );
              })}
            </div>
          )}
        </nav>
        <div className="relative flex min-w-0 flex-1 flex-col">
          <button type="button" aria-label="Close settings" onClick={() => onOpenChange(false)}
            className="absolute top-[13px] right-[13px] z-10 flex size-[34px] items-center justify-center rounded-md text-muted-foreground transition-colors hover:bg-accent hover:text-foreground max-md:hidden">
            <X className="size-4" />
          </button>
          {/* Only the chosen section is drawn; the tools Foundry docks under General stay with General. */}
          {shown && (
            <style>{`[data-settings-pane] [data-slot="card"][id]:not([id="${shown}"]){display:none}${shown === "general-settings" ? "" : `[data-settings-pane] div:has(> #general-settings) > :not([data-slot="card"]){display:none!important}`}`}</style>
          )}
          <div ref={setPane} data-settings-pane=""
            className="min-h-0 flex-1 overflow-y-auto px-[34px] py-[21px] max-md:px-[13px] [&>div]:overflow-visible [&>div]:p-0 [&>div>div>p:first-child]:hidden [&_[data-slot=card]]:gap-[21px] [&_[data-slot=card]]:rounded-none [&_[data-slot=card]]:border-0 [&_[data-slot=card]]:bg-transparent [&_[data-slot=card]]:py-0 [&_[data-slot=card]]:shadow-none [&_[data-slot=card-content]]:px-0 [&_[data-slot=card-header]]:border-b [&_[data-slot=card-header]]:px-0 [&_[data-slot=card-header]]:pr-[34px] [&_[data-slot=card-header]]:pb-[13px] [&_[data-slot=card-title]]:text-base [&_[data-found]]:rounded-md [&_[data-found]]:ring-2 [&_[data-found]]:ring-ring [&_[data-found]]:ring-offset-[5px] [&_[data-found]]:ring-offset-background">
            <Suspense fallback={<div className="flex flex-col gap-[13px]"><Skeleton className="h-[21px] w-[144px]" /><Skeleton className="h-[55px] w-full" /><Skeleton className="h-[55px] w-full" /></div>}>
              <SettingsPage />
            </Suspense>
          </div>
        </div>
      </DialogContent>
    </Dialog>
  );
}

// The user's apps, starred first, then the most recently changed; the open app shows its recent chats.
function RecentApps() {
  const { apps } = useLoadApps();
  const openApp = useOpenApp();
  const selectedAppId = useAtomValue(selectedAppIdAtom);
  const selectedChatId = useAtomValue(selectedChatIdAtom);
  const { chats } = useChats(selectedAppId);
  const { selectChat } = useSelectChat();
  const shown = useMemo(
    () =>
      [...apps]
        .sort((a, b) => Number(b.isFavorite) - Number(a.isFavorite) || +new Date(b.updatedAt) - +new Date(a.updatedAt))
        .slice(0, RECENT_APPS),
    [apps],
  );
  if (!shown.length) return null;
  return (
    <SidebarGroup className="group-data-[collapsible=icon]:hidden">
      <SidebarGroupLabel>Your apps</SidebarGroupLabel>
      <SidebarMenu>
        {shown.map((app) => (
          <SidebarMenuItem key={app.id}>
            <SidebarMenuButton isActive={app.id === selectedAppId} onClick={() => openApp(app.id)}>
              <span className="truncate">{app.name}</span>
            </SidebarMenuButton>
            {app.isFavorite && (
              <SidebarMenuBadge>
                <Star className="size-3" />
              </SidebarMenuBadge>
            )}
            {app.id === selectedAppId && chats.length > 0 && (
              <SidebarMenuSub>
                {chats.slice(0, RECENT_CHATS).map((chat) => (
                  <SidebarMenuSubItem key={chat.id}>
                    <SidebarMenuSubButton
                      asChild
                      isActive={chat.id === selectedChatId}
                    >
                      <button type="button" className="w-full" onClick={() => selectChat({ chatId: chat.id, appId: app.id })}>
                        <span className="truncate">{chat.title || "New chat"}</span>
                      </button>
                    </SidebarMenuSubButton>
                  </SidebarMenuSubItem>
                ))}
              </SidebarMenuSub>
            )}
          </SidebarMenuItem>
        ))}
        {apps.length > shown.length && (
          <SidebarMenuItem>
            <SidebarMenuButton asChild className="text-sidebar-foreground/70">
              <Link to="/apps">
                <span>All apps ({apps.length})</span>
              </Link>
            </SidebarMenuButton>
          </SidebarMenuItem>
        )}
      </SidebarMenu>
    </SidebarGroup>
  );
}

const TITLES: [string, string][] = [
  ["/apps", "Apps"],
  ["/templates", "Templates"],
  ["/library", "Library"],
  ["/theater", "Theater"],
  ["/plugins", "Plugins"],
  ["/settings", "Settings"],
];
const LIBRARY: Record<string, string> = { themes: "Themes", prompts: "Prompts", media: "Media" };

// Where you are, as a trail: the section, then the page inside it (an app, a plugin, a provider, a part of
// the library). The section links back, so no page needs a back link of its own.
function useTrail(): { title: string; to?: string }[] {
  const isActive = useIsActive();
  const selectedAppId = useAtomValue(selectedAppIdAtom);
  const { apps } = useLoadApps();
  const app = apps.find((a) => a.id === selectedAppId);
  const rest = usePath().split("/").filter(Boolean);
  if (isActive("/chat", true) || isActive("/app-details", true)) return app ? [{ title: "Apps", to: "/apps" }, { title: app.name }] : [{ title: "Apps", to: "/apps" }];
  if (isActive("/providers/$provider", true)) return [{ title: "Settings", to: "/settings" }, { title: decodeURIComponent(rest[1] ?? "") }];
  const section = TITLES.find(([p]) => isActive(p));
  if (!section) return [{ title: "Home" }];
  const sub = rest[1] ? (section[0] === "/library" ? LIBRARY[rest[1]] : decodeURIComponent(rest[1])) : null;
  return sub ? [{ title: section[1], to: section[0] }, { title: sub }] : [{ title: section[1] }];
}

// The site header: the sidebar's trigger and the trail, as in dashboard-01. Its right end is left clear for
// the appearance switch the page shell keeps at the top right.
export function SiteHeader() {
  const trail = useTrail();
  const { toggleSidebar } = useSidebar();
  // On a phone the menu is the familiar three lines at the top left, and home shows no title: the page's own
  // headline says where you are. From the tablet up, the kit's panel trigger and the trail.
  const home = usePath() === "/";
  return (
    <header className="flex h-(--header-height) shrink-0 items-center gap-2 border-b pr-14 transition-[width,height] ease-linear">
      <div className="flex w-full min-w-0 items-center gap-1 px-4 lg:gap-2 lg:px-6">
        <button type="button" data-slot="sidebar-trigger" data-mobile-menu="" aria-label="Menu" onClick={toggleSidebar}
          className="-ml-1 inline-flex size-[34px] items-center justify-center rounded-full md:hidden">
          <Menu className="size-[21px]" aria-hidden="true" />
        </button>
        <SidebarTrigger className="-ml-1 hidden md:flex" />
        <Separator orientation="vertical" className={`mx-2 data-[orientation=vertical]:h-4 ${home ? "max-md:hidden" : ""}`} />
        <Breadcrumb className={`min-w-0 ${home ? "max-md:hidden" : ""}`}>
          <BreadcrumbList className="flex-nowrap">
            {trail.map((step, i) => (
              <Fragment key={i}>
                {i > 0 && <BreadcrumbSeparator />}
                <BreadcrumbItem className="min-w-0">
                  {step.to ? (
                    <BreadcrumbLink asChild>
                      <Link to={step.to}>{step.title}</Link>
                    </BreadcrumbLink>
                  ) : (
                    <BreadcrumbPage className="truncate text-base font-medium">{step.title}</BreadcrumbPage>
                  )}
                </BreadcrumbItem>
              </Fragment>
            ))}
          </BreadcrumbList>
        </Breadcrumb>
      </div>
    </header>
  );
}

// A phone's way around: Home, Apps and Library in a floating bar at the foot of the page, as the sidebar's
// own links (the menu at the top left keeps everything else). The build screen has its own bar, so this one
// is drawn on every other screen, and only below the tablet width.
const TABS = NAV.filter((n) => n.to === "/" || n.to === "/apps" || n.to === "/library");
export function MobileTabBar() {
  const isActive = useIsActive();
  return (
    <nav data-slot="mobile-tabs" aria-label="Main" className="md:hidden">
      {TABS.map((tab) => {
        const on = isActive(tab.to, tab.exact);
        return (
          <Link key={tab.to} to={tab.to} aria-label={tab.title} title={tab.title} aria-current={on ? "page" : undefined} data-active={on ? "" : undefined}>
            <tab.icon aria-hidden="true" />
          </Link>
        );
      })}
    </nav>
  );
}

// The shell's one search (Ctrl or Cmd K, and Search under Home): a centred palette, as the kit's command dialog
// draws it. The apps and chats are already in memory, so it filters as you type, with no debounce and no round
// trip. Recent ones show before a query; New app is always one choice away.
const SEARCH_TABS = ["All", "Apps", "Chats"] as const;
const RECENT_FOUND = 6;

function SearchPalette({ open, onOpenChange, onSelectApp, allApps, disableShortcut, onSelectChat, onNewApp }: {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onSelectApp: (appId: number) => void;
  allApps: AppSearchResult[];
  disableShortcut?: boolean;
  onSelectChat?: (chatId: number, appId: number) => void;
  onNewApp?: () => void;
}) {
  const [tab, setTab] = useState<(typeof SEARCH_TABS)[number]>("All");
  const [query, setQuery] = useState("");
  const { chats } = useChats(null);
  useEffect(() => {
    if (disableShortcut) return;
    const down = (e: KeyboardEvent) => {
      if (e.key === "k" && (e.metaKey || e.ctrlKey)) { e.preventDefault(); onOpenChange(!open); }
    };
    document.addEventListener("keydown", down);
    return () => document.removeEventListener("keydown", down);
  }, [open, onOpenChange, disableShortcut]);
  // Opening starts clean: an empty query shows the recent ones.
  useEffect(() => { if (open) { setQuery(""); setTab("All"); } }, [open]);
  const searching = query.trim() !== "";
  const names = new Map(allApps.map((a) => [a.id, a.name]));
  const apps = tab === "Chats" ? [] : searching ? allApps : allApps.slice(0, RECENT_FOUND);
  const recent = [...chats].sort((a, b) => +new Date(b.createdAt) - +new Date(a.createdAt));
  const found = !onSelectChat || tab === "Apps" ? [] : searching ? recent : recent.slice(0, RECENT_FOUND);
  const then = (act: () => void) => () => { onOpenChange(false); act(); };
  return (
    <CommandDialog open={open} onOpenChange={onOpenChange} title="Search" description="Find an app or a chat" className="sm:max-w-xl" data-testid="app-search-dialog">
      <CommandInput placeholder={onSelectChat ? "Search apps and chats" : "Search apps"} value={query} onValueChange={setQuery} data-testid="app-search-input" />
      {onSelectChat && (
        <div className="flex gap-1 border-b px-3 py-2" role="tablist" aria-label="Search in">
          {SEARCH_TABS.map((t) => (
            <button key={t} type="button" role="tab" aria-selected={tab === t} onClick={() => setTab(t)}
              className="rounded-md px-2.5 py-1 text-xs font-medium text-muted-foreground hover:text-foreground aria-selected:bg-accent aria-selected:text-foreground">
              {t}
            </button>
          ))}
        </div>
      )}
      {/* A fixed height, so the box never jumps while the results change under the cursor. */}
      <CommandList className="h-[min(55vh,24rem)] max-h-none" data-testid="app-search-list">
        <CommandEmpty>No results</CommandEmpty>
        {apps.length > 0 && (
          <CommandGroup heading={searching ? "Apps" : "Recent apps"}>
            {apps.map((app) => (
              <CommandItem key={`a${app.id}`} value={`${app.name} #a${app.id}`} onSelect={then(() => onSelectApp(app.id))} data-testid={`app-search-item-${app.id}`}>
                <LayoutGrid />
                <span className="truncate">{app.name}</span>
              </CommandItem>
            ))}
          </CommandGroup>
        )}
        {found.length > 0 && (
          <CommandGroup heading={searching ? "Chats" : "Recent chats"}>
            {found.map((chat) => (
              <CommandItem key={`c${chat.id}`} value={`${chat.title || "New chat"} ${names.get(chat.appId) ?? ""} #c${chat.id}`} onSelect={then(() => onSelectChat?.(chat.id, chat.appId))}>
                <MessageSquare />
                <span className="truncate">{chat.title || "New chat"}</span>
                {names.get(chat.appId) !== chat.title && <span className="ml-auto truncate text-xs text-muted-foreground">{names.get(chat.appId)}</span>}
              </CommandItem>
            ))}
          </CommandGroup>
        )}
        {onNewApp && tab === "All" && (
          <CommandGroup heading="Actions">
            <CommandItem value="New app #new" onSelect={then(onNewApp)}>
              <CirclePlus />
              <span>New app</span>
            </CommandItem>
          </CommandGroup>
        )}
      </CommandList>
    </CommandDialog>
  );
}
