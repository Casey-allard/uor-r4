// Where records are kept: this origin's IndexedDB in the browser, memory in tests. Both have the same
// four operations, and none of them can send anything anywhere. IndexedDB is origin-scoped: no other
// site, and no app a user builds (its preview runs under a separate grant, without this database's
// handle), can open it. It is chosen over OPFS because the shell's main thread writes one record per
// turn, and OPFS's synchronous handles exist only in workers.
//
// A turn is one record: { id, at, model, events: [[kind, at, value, model], ...], dropped }.
// Meta is a small key-value table: model names (the index a record carries) and the loop's state.
// An app is one record: { id, name, installed, net: the model's AppNet, sites: { site: the model's Reach } },
// the totals of the kernel's decisions about it, never the requests themselves.

export function memoryBackend() {
  const turns = new Map(), meta = new Map(), apps = new Map();
  return {
    async putApp(app) { apps.set(app.id, structuredClone(app)); },
    async getApp(id) { return apps.has(id) ? structuredClone(apps.get(id)) : undefined; },
    async allApps() { return [...apps.values()].map((a) => structuredClone(a)); },
    async removeApps(ids) { for (const id of ids) apps.delete(id); },
    async put(turn) { turns.set(turn.id, structuredClone(turn)); },
    async all() { return [...turns.values()].sort((a, b) => a.at - b.at).map((t) => structuredClone(t)); },
    async remove(ids) { for (const id of ids) turns.delete(id); },
    async getMeta(key) { return meta.has(key) ? structuredClone(meta.get(key)) : undefined; },
    async setMeta(key, value) { meta.set(key, structuredClone(value)); },
    async erase() { turns.clear(); meta.clear(); apps.clear(); },
  };
}

export function indexedDbBackend(name = "prism-telemetry") {
  const open = () => new Promise((resolve, reject) => {
    // Version 2 adds the apps; a version 1 database keeps its turns and meta.
    const req = indexedDB.open(name, 2);
    req.onupgradeneeded = () => {
      const d = req.result;
      if (!d.objectStoreNames.contains("turns")) d.createObjectStore("turns", { keyPath: "id" }).createIndex("at", "at");
      if (!d.objectStoreNames.contains("meta")) d.createObjectStore("meta");
      if (!d.objectStoreNames.contains("apps")) d.createObjectStore("apps", { keyPath: "id" });
    };
    req.onsuccess = () => resolve(req.result);
    req.onerror = () => reject(req.error);
  });
  let db = open();
  const tx = async (stores, mode, work) => {
    const d = await db;
    return new Promise((resolve, reject) => {
      const t = d.transaction(stores, mode);
      let result;
      t.oncomplete = () => resolve(result);
      t.onerror = t.onabort = () => reject(t.error);
      result = work(t);
    });
  };
  const request = (r) => new Promise((resolve, reject) => { r.onsuccess = () => resolve(r.result); r.onerror = () => reject(r.error); });
  return {
    put: (turn) => tx(["turns"], "readwrite", (t) => { t.objectStore("turns").put(turn); }),
    async all() { const d = await db; return request(d.transaction("turns").objectStore("turns").index("at").getAll()); },
    remove: (ids) => tx(["turns"], "readwrite", (t) => { for (const id of ids) t.objectStore("turns").delete(id); }),
    putApp: (app) => tx(["apps"], "readwrite", (t) => { t.objectStore("apps").put(app); }),
    async getApp(id) { const d = await db; return request(d.transaction("apps").objectStore("apps").get(id)); },
    async allApps() { const d = await db; return request(d.transaction("apps").objectStore("apps").getAll()); },
    removeApps: (ids) => tx(["apps"], "readwrite", (t) => { for (const id of ids) t.objectStore("apps").delete(id); }),
    async getMeta(key) { const d = await db; return request(d.transaction("meta").objectStore("meta").get(key)); },
    setMeta: (key, value) => tx(["meta"], "readwrite", (t) => { t.objectStore("meta").put(value, key); }),
    async erase() {
      (await db).close();
      await new Promise((resolve, reject) => { const r = indexedDB.deleteDatabase(name); r.onsuccess = resolve; r.onerror = () => reject(r.error); r.onblocked = resolve; });
      db = open();
    },
  };
}
