// hologram-forge (ours, not upstream's; see VENDORED.md): the Apps page, in place of src/pages/apps.tsx
// (hologram.screens.mjs; compiled at that path). Laid out as shadcn's dashboard-01 block: section cards with
// trend badges, tabs with counts beside the table's tools, and a data table with row selection and a row
// menu. Every part is the kit's shadcn/ui component. The data and every action are upstream's own (its hooks,
// its ipc calls, its collection views and dialogs), so nothing behind the page changes.
import { useEffect, useMemo, useState } from "react";
import { useNavigate } from "@tanstack/react-router";
import { useAtom } from "jotai";
import {
  CirclePlus,
  EllipsisVertical,
  FolderPlus,
  Loader2,
  Plus,
  Search,
  Star,
  Trash2,
  TrendingUp,
} from "lucide-react";

import { selectedAppIdAtom } from "@/atoms/appAtoms";
import { AddOrEditCollectionDialog } from "@/components/AddOrEditCollectionDialog";
import { AssignAppsToCollectionDialog } from "@/components/AssignAppsToCollectionDialog";
import { CollectionDetailView } from "@/components/CollectionDetailView";
import { CollectionFolderCard } from "@/components/CollectionFolderCard";
import { DeleteCollectionDialog } from "@/components/DeleteCollectionDialog";
import { useAddAppToFavorite } from "@/hooks/useAddAppToFavorite";
import { useAppCollections, type AppCollection } from "@/hooks/useAppCollections";
import { useChats } from "@/hooks/useChats";
import { useLoadApps } from "@/hooks/useLoadApps";
import { useOpenApp } from "@/hooks/useOpenApp";
import { ipc } from "@/ipc/types";
import { sortAppsForShowcase } from "@/lib/sortApps";
import { showError } from "@/lib/toast";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardAction, CardDescription, CardFooter, CardHeader, CardTitle } from "@/components/ui/card";
import { Checkbox } from "@/components/ui/checkbox";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { Input } from "@/components/ui/input";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";

const day = new Intl.DateTimeFormat(undefined, { month: "short", day: "numeric", year: "numeric" });
const when = (d: Date | string) => day.format(new Date(d));

// What counts as this week is the Forge kernel's decision (FK-21, proved by Lean: nothing older than seven
// days): each date is asked as "recent <seconds old>". Until every answer is in, the figures show a dash.
declare global { interface Window { __forgeKernel?: { ask: (line: string) => Promise<string> } } }
function useThisWeek(dates: (Date | string)[]) {
  const key = dates.map((d) => +new Date(d)).join(",");
  const [answers, setAnswers] = useState<Map<number, boolean> | null>(null);
  useEffect(() => {
    let live = true;
    const kernel = window.__forgeKernel;
    if (!kernel) return;
    const times = [...new Set(key ? key.split(",").map(Number) : [])];
    const now = Date.now();
    Promise.all(times.map((t) => kernel.ask("recent " + Math.max(0, Math.floor((now - t) / 1000))).then((a) => [t, a === "yes"] as const)))
      .then((pairs) => { if (live) setAnswers(new Map(pairs)); })
      .catch(() => { if (live) setAnswers(null); });
    return () => { live = false; };
  }, [key]);
  return answers && ((d: Date | string) => answers.get(+new Date(d)) === true);
}

export default function AppsPage() {
  const navigate = useNavigate();
  const { apps, loading, refreshApps } = useLoadApps();
  const { collections, isLoading: collectionsLoading } = useAppCollections();
  const { chats } = useChats(null);
  const openApp = useOpenApp();
  const { toggleFavorite } = useAddAppToFavorite();
  const [selectedAppId, setSelectedAppId] = useAtom(selectedAppIdAtom);
  const [tab, setTab] = useState("apps");
  const [query, setQuery] = useState("");
  const [picked, setPicked] = useState<Set<number>>(new Set());
  const [confirmDelete, setConfirmDelete] = useState(false);
  const [deleting, setDeleting] = useState(false);
  const [assigning, setAssigning] = useState(false);
  const [openCollectionId, setOpenCollectionId] = useState<number | null>(null);
  const [editingCollection, setEditingCollection] = useState<AppCollection | null>(null);
  const [collectionDialog, setCollectionDialog] = useState(false);
  const [deletingCollection, setDeletingCollection] = useState<AppCollection | null>(null);

  const q = query.trim().toLowerCase();
  const rows = useMemo(() => sortAppsForShowcase(apps).filter((a) => !q || a.name.toLowerCase().includes(q)), [apps, q]);
  const shownCollections = useMemo(() => collections.filter((c) => !q || c.name.toLowerCase().includes(q)), [collections, q]);
  const openCollection = collections.find((c) => c.id === openCollectionId) ?? null;
  const collectionName = useMemo(() => new Map(collections.map((c) => [c.id, c.name])), [collections]);
  const pickedApps = apps.filter((a) => picked.has(a.id));
  const allPicked = rows.length > 0 && rows.every((a) => picked.has(a.id));

  const togglePick = (id: number) =>
    setPicked((prev) => {
      const next = new Set(prev);
      if (next.has(id)) next.delete(id);
      else next.add(id);
      return next;
    });

  // Upstream's bulk delete, unchanged: failed ids stay picked so they can be retried.
  const deletePicked = async () => {
    if (!picked.size) return;
    setDeleting(true);
    try {
      const { results } = await ipc.app.deleteApps({ appIds: [...picked] });
      const failed = results.filter((r) => !r.success);
      const gone = new Set(results.filter((r) => r.success).map((r) => r.appId));
      if (selectedAppId != null && gone.has(selectedAppId)) setSelectedAppId(null);
      if (failed.length) {
        showError(`Failed to delete ${failed.length} app${failed.length === 1 ? "" : "s"}: ${failed.map((r) => apps.find((a) => a.id === r.appId)?.name ?? `#${r.appId}`).join(", ")}`);
        setPicked(new Set(failed.map((r) => r.appId)));
      } else setPicked(new Set());
      setConfirmDelete(false);
      await refreshApps();
    } catch (error) {
      showError(error);
      setConfirmDelete(false);
    } finally {
      setDeleting(false);
    }
  };

  const thisWeek = useThisWeek([...apps.map((a) => a.createdAt), ...apps.map((a) => a.updatedAt), ...chats.map((c) => c.createdAt)]);
  const count = <T,>(xs: T[], at: (x: T) => Date | string) => (thisWeek ? xs.filter((x) => thisWeek(at(x))).length : null);
  const stats = [
    { label: "Apps", value: apps.length, trend: count(apps, (a) => a.createdAt), note: "Created this week", foot: "Every app you have built here" },
    { label: "Chats", value: chats.length, trend: count(chats, (c) => c.createdAt), note: "Started this week", foot: "Conversations across all apps" },
    { label: "Updated", value: count(apps, (a) => a.updatedAt) ?? "–", trend: null, note: "Changed in the last 7 days", foot: "Apps you are working on" },
    { label: "Starred", value: apps.filter((a) => a.isFavorite).length, trend: null, note: `${collections.length} collection${collections.length === 1 ? "" : "s"}`, foot: "Pinned to the top of the sidebar" },
  ];

  return (
    <div className="@container/main flex min-h-0 w-full flex-1 flex-col overflow-y-auto">
      <div className="flex flex-col gap-4 py-4 md:gap-6 md:py-6">
        <div className="grid grid-cols-2 gap-3 px-4 @xl/main:gap-4 *:data-[slot=card]:bg-gradient-to-t *:data-[slot=card]:from-primary/5 *:data-[slot=card]:to-card *:data-[slot=card]:shadow-xs lg:px-6 @xl/main:grid-cols-2 @5xl/main:grid-cols-4 dark:*:data-[slot=card]:bg-card">
          {stats.map((s) => (
            <Card key={s.label} className="@container/card">
              <CardHeader>
                <CardDescription>{s.label}</CardDescription>
                <CardTitle className="text-2xl font-semibold tabular-nums @[250px]/card:text-3xl">{loading ? "–" : s.value}</CardTitle>
                {s.trend !== null && (
                  <CardAction>
                    <Badge variant="outline">
                      <TrendingUp />+{s.trend}
                    </Badge>
                  </CardAction>
                )}
              </CardHeader>
              <CardFooter className="hidden flex-col items-start gap-1.5 text-sm @xl/main:flex">
                <div className="line-clamp-1 flex gap-2 font-medium">{s.note}</div>
                <div className="text-muted-foreground">{s.foot}</div>
              </CardFooter>
            </Card>
          ))}
        </div>

        <Tabs
          value={tab}
          onValueChange={(v) => {
            setTab(v);
            setQuery("");
            setPicked(new Set());
            setOpenCollectionId(null);
          }}
          className="w-full flex-col justify-start gap-6"
        >
          <div className="flex flex-wrap items-center justify-between gap-2 px-4 lg:px-6">
            <TabsList className="**:data-[slot=badge]:size-5 **:data-[slot=badge]:rounded-full **:data-[slot=badge]:bg-muted-foreground/30 **:data-[slot=badge]:px-1">
              <TabsTrigger value="apps">
                Apps <Badge variant="secondary">{apps.length}</Badge>
              </TabsTrigger>
              <TabsTrigger value="collections">
                Collections <Badge variant="secondary">{collections.length}</Badge>
              </TabsTrigger>
            </TabsList>
            <div className="flex items-center gap-2">
              <div className="relative">
                <Search className="pointer-events-none absolute top-1/2 left-2.5 size-4 -translate-y-1/2 text-muted-foreground" />
                <Input
                  value={query}
                  onChange={(e) => setQuery(e.target.value)}
                  placeholder={tab === "apps" ? "Search apps" : "Search collections"}
                  aria-label={tab === "apps" ? "Search apps" : "Search collections"}
                  className="h-8 w-48 pl-8 lg:w-64"
                />
              </div>
              {tab === "apps" ? (
                <Button size="sm" onClick={() => navigate({ to: "/" })}>
                  <CirclePlus />
                  <span className="hidden lg:inline">New app</span>
                </Button>
              ) : (
                <Button
                  size="sm"
                  variant="outline"
                  onClick={() => {
                    setEditingCollection(null);
                    setCollectionDialog(true);
                  }}
                >
                  <Plus />
                  <span className="hidden lg:inline">Add collection</span>
                </Button>
              )}
            </div>
          </div>

          <TabsContent value="apps" className="relative flex flex-col gap-4 overflow-auto px-4 lg:px-6">
            {picked.size > 0 && (
              <div className="flex items-center justify-between gap-2 rounded-lg border bg-muted/50 px-3 py-2">
                <span className="text-sm text-muted-foreground">{picked.size} selected</span>
                <div className="flex items-center gap-2">
                  <Button variant="outline" size="sm" onClick={() => setPicked(new Set())}>
                    Clear
                  </Button>
                  <Button variant="outline" size="sm" onClick={() => setAssigning(true)}>
                    <FolderPlus />
                    Add to collection
                  </Button>
                  <Button variant="destructive" size="sm" onClick={() => setConfirmDelete(true)}>
                    <Trash2 />
                    Delete
                  </Button>
                </div>
              </div>
            )}
            <div className="overflow-hidden rounded-lg border">
              <Table>
                <TableHeader className="sticky top-0 z-10 bg-muted">
                  <TableRow>
                    <TableHead className="w-10 pl-4">
                      <Checkbox
                        checked={allPicked || (picked.size > 0 && "indeterminate")}
                        onCheckedChange={(v) => setPicked(v ? new Set(rows.map((a) => a.id)) : new Set())}
                        aria-label="Select all"
                      />
                    </TableHead>
                    <TableHead>Name</TableHead>
                    <TableHead>Collection</TableHead>
                    <TableHead>Created</TableHead>
                    <TableHead>Updated</TableHead>
                    <TableHead className="w-10" />
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {loading ? (
                    <TableRow>
                      <TableCell colSpan={6} className="h-24 text-center text-muted-foreground">
                        Loading apps…
                      </TableCell>
                    </TableRow>
                  ) : rows.length === 0 ? (
                    <TableRow>
                      <TableCell colSpan={6} className="h-24 text-center text-muted-foreground">
                        {q ? "No apps match your search." : "You haven't created any apps yet."}
                      </TableCell>
                    </TableRow>
                  ) : (
                    rows.map((app) => (
                      <TableRow key={app.id} data-state={picked.has(app.id) && "selected"}>
                        <TableCell className="pl-4">
                          <Checkbox checked={picked.has(app.id)} onCheckedChange={() => togglePick(app.id)} aria-label={`Select ${app.name}`} />
                        </TableCell>
                        <TableCell>
                          <Button variant="link" className="h-auto px-0 text-foreground" onClick={() => openApp(app.id)}>
                            {app.name}
                          </Button>
                          {app.isFavorite && <Star className="ml-2 inline size-3.5 fill-current text-muted-foreground" />}
                        </TableCell>
                        <TableCell>
                          {app.collectionId != null && collectionName.has(app.collectionId) ? (
                            <Badge variant="outline" className="px-1.5 text-muted-foreground">
                              {collectionName.get(app.collectionId)}
                            </Badge>
                          ) : (
                            <span className="text-muted-foreground">—</span>
                          )}
                        </TableCell>
                        <TableCell className="text-muted-foreground tabular-nums">{when(app.createdAt)}</TableCell>
                        <TableCell className="text-muted-foreground tabular-nums">{when(app.updatedAt)}</TableCell>
                        <TableCell>
                          <DropdownMenu>
                            <DropdownMenuTrigger asChild>
                              <Button variant="ghost" size="icon" className="size-8 text-muted-foreground data-[state=open]:bg-muted">
                                <EllipsisVertical />
                                <span className="sr-only">Open menu</span>
                              </Button>
                            </DropdownMenuTrigger>
                            <DropdownMenuContent align="end" className="w-40">
                              <DropdownMenuItem onClick={() => openApp(app.id)}>Open</DropdownMenuItem>
                              <DropdownMenuItem onClick={() => toggleFavorite(app.id)}>{app.isFavorite ? "Unstar" : "Star"}</DropdownMenuItem>
                              <DropdownMenuItem
                                onClick={() => {
                                  setPicked(new Set([app.id]));
                                  setAssigning(true);
                                }}
                              >
                                Add to collection
                              </DropdownMenuItem>
                              <DropdownMenuSeparator />
                              <DropdownMenuItem
                                variant="destructive"
                                onClick={() => {
                                  setPicked(new Set([app.id]));
                                  setConfirmDelete(true);
                                }}
                              >
                                Delete
                              </DropdownMenuItem>
                            </DropdownMenuContent>
                          </DropdownMenu>
                        </TableCell>
                      </TableRow>
                    ))
                  )}
                </TableBody>
              </Table>
            </div>
          </TabsContent>

          <TabsContent value="collections" className="flex flex-col gap-4 px-4 lg:px-6">
            {openCollection ? (
              <CollectionDetailView collection={openCollection} apps={apps} collections={collections} onBack={() => setOpenCollectionId(null)} />
            ) : collectionsLoading ? (
              <p className="py-12 text-center text-muted-foreground">Loading collections…</p>
            ) : shownCollections.length === 0 ? (
              <p className="py-12 text-center text-muted-foreground">
                {q ? "No collections match your search." : "No collections yet. Create one to organize your apps."}
              </p>
            ) : (
              <div className="grid grid-cols-[repeat(auto-fill,minmax(260px,1fr))] gap-4">
                {shownCollections.map((c) => (
                  <CollectionFolderCard
                    key={c.id}
                    collection={c}
                    onOpen={(col) => setOpenCollectionId(col.id)}
                    onRename={(col) => {
                      setEditingCollection(col);
                      setCollectionDialog(true);
                    }}
                    onDelete={(col) => setDeletingCollection(col)}
                  />
                ))}
              </div>
            )}
          </TabsContent>
        </Tabs>
      </div>

      <Dialog open={confirmDelete} onOpenChange={(open) => !deleting && setConfirmDelete(open)}>
        <DialogContent className="sm:max-w-sm">
          <DialogHeader>
            <DialogTitle>
              Delete {picked.size} app{picked.size === 1 ? "" : "s"}?
            </DialogTitle>
            <DialogDescription>This cannot be undone. Every file and chat of {picked.size === 1 ? "this app" : "these apps"} is deleted.</DialogDescription>
          </DialogHeader>
          <ul className="max-h-40 overflow-y-auto rounded-md border px-3 py-2 text-sm">
            {pickedApps.map((app) => (
              <li key={app.id} className="truncate py-0.5">
                {app.name}
              </li>
            ))}
          </ul>
          <DialogFooter>
            <Button variant="outline" onClick={() => setConfirmDelete(false)} disabled={deleting}>
              Cancel
            </Button>
            <Button variant="destructive" onClick={deletePicked} disabled={deleting || !picked.size}>
              {deleting && <Loader2 className="animate-spin" />}
              Delete
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
      <AssignAppsToCollectionDialog open={assigning} onOpenChange={setAssigning} apps={pickedApps} collections={collections} onAssigned={() => setPicked(new Set())} />
      <AddOrEditCollectionDialog
        open={collectionDialog}
        onOpenChange={(next) => {
          setCollectionDialog(next);
          if (!next) setEditingCollection(null);
        }}
        collection={editingCollection}
        allApps={apps}
        collections={collections}
      />
      <DeleteCollectionDialog
        open={deletingCollection !== null}
        onOpenChange={(next) => !next && setDeletingCollection(null)}
        collection={deletingCollection}
        onDeleted={() => {
          if (openCollectionId === deletingCollection?.id) setOpenCollectionId(null);
          setDeletingCollection(null);
        }}
      />
    </div>
  );
}
