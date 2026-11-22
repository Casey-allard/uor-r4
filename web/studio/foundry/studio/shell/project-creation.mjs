import {browserChatMode} from "./browser-chat-modes.mjs";
// FoundrySpecification.projectCreationBoundary; canonical bytes come from the core.
import {readThemeCatalog, initialProjectTheme} from "./theme-library.mjs";
const MiB = 1024 * 1024;
const encoder = new TextEncoder(), decoder = new TextDecoder("utf-8", { fatal: true, ignoreBOM: true });
const failure = code => Object.assign(new Error(code === "projectScaffoldFailed" ? "Project scaffold unavailable" : code === "projectPreparationFailed" ? "Project preparation failed" : "Project storage failed"), { code });
const portable = path => typeof path === "string" && path.length > 0 && !/[\\\u0000-\u001f:]/.test(path) && path.split("/").every(part => part && part !== "." && part !== "..");
const address = key => typeof key === "string" && /^blake3:[0-9a-f]{64}$/.test(key);
export async function loadProjectScaffold(base, { fetchFn = globalThis.fetch, timeoutMs = 30000 } = {}) {
  const controller = new AbortController(); let timer;
  const deadline = new Promise((_, reject) => { timer = setTimeout(() => { controller.abort(); reject(failure("projectScaffoldFailed")); }, Math.max(1, Math.min(timeoutMs, 30000))); });
  const timed = promise => Promise.race([promise, deadline]);
  let total = 0;
  const read = async (path, maximum) => {
    const url = new URL(path.split("/").map(encodeURIComponent).join("/"), base);
    const response = await timed(fetchFn(url, { signal: controller.signal, redirect: "error", credentials: "omit" }));
    if (!response.ok || !response.body) { void response.body?.cancel().catch(() => {}); throw failure("projectScaffoldFailed"); }
    const reader = response.body.getReader(); const chunks = []; let length = 0;
    try {
      for (;;) {
        const { done, value } = await timed(reader.read()); if (done) break;
        length += value.length; total += value.length;
        if (length > maximum || total > 64 * MiB) throw failure("projectScaffoldFailed");
        chunks.push(value);
      }
      const bytes = new Uint8Array(length); let offset = 0;
      for (const chunk of chunks) { bytes.set(chunk, offset); offset += chunk.length; }
      return decoder.decode(bytes);
    } finally { void reader.cancel().catch(() => {}); reader.releaseLock(); }
  };
  try {
    const list = JSON.parse(await read("files.json", 65536));
    if (!Array.isArray(list) || list.length > 1000 || list.some(path => !portable(path)) || new Set(list).size !== list.length) throw failure("projectScaffoldFailed");
    if (!list.includes("AI_RULES.md")) list.push("AI_RULES.md");
    if (list.length > 1000) throw failure("projectScaffoldFailed");
    const files = [];
    for (const path of list) files.push([path, await read(path, 16 * MiB)]);
    return Object.fromEntries(files);
  } catch { throw failure("projectScaffoldFailed"); }
  finally { clearTimeout(timer); controller.abort(); }
}
async function prepareProject(input, { core, derive }, parent) {
  try {
    const { name, files, initialChatMode } = structuredClone(input);
    if (typeof name !== "string" || !name.trim() || name.length > 255 || !files || typeof files !== "object" || Array.isArray(files)) throw Error();
    const rows = Object.entries(files).sort(([a], [b]) => a < b ? -1 : a > b ? 1 : 0);
    if (rows.length > 1000) throw Error();
    let total = 0; const objects = [], entries = [];
    const copied = rows.map(([path, text]) => {
      if (!portable(path) || typeof text !== "string") throw Error();
      const bytes = encoder.encode(text); total += bytes.length;
      if (bytes.length > 16 * MiB || total > 64 * MiB || decoder.decode(bytes) !== text) throw Error();
      return [path, text, bytes];
    });
    for (const [path, , bytes] of copied) {
      const key = await derive(bytes); if (!address(key)) throw Error();
      entries.push({ path, kappa: key, bytes: bytes.length }); objects.push([key, bytes]);
    }
    const preimage = core.run({ op: "preimage", project: { label: name, entries }, parent }).bytes;
    if (typeof preimage !== "string") throw Error();
    const body = encoder.encode(preimage); if (body.length > MiB) throw Error();
    const oid = await derive(body); if (!address(oid)) throw Error(); objects.push([oid, body]);
    return { name, files: rows, objects, oid, initialChatMode: initialChatMode || "build" };
  } catch { throw failure("projectPreparationFailed"); }
}
export async function prepareNewProject(input, boundary) { return prepareProject(input, boundary, ""); }
export async function prepareExistingProject(input, boundary, parent) {
  if (!address(parent)) throw failure("projectPreparationFailed");
  return prepareProject(input, boundary, parent);
}
const nextIdentifier = keys => {
  let maximum = 0;
  for (const key of keys) { if (!Number.isSafeInteger(key) || key < 1) throw Error(); maximum = Math.max(maximum, key); }
  if (maximum >= Number.MAX_SAFE_INTEGER) throw Error(); return maximum + 1;
};
// A committed high-water mark survives deletion and also accounts for legacy keys.
export function allocateIdentifier(transaction, storeName, accept, refuse) {
  const store = transaction.objectStore(storeName), refs = transaction.objectStore("refs");
  const keys = store.getAllKeys(), mark = refs.get(`allocation:${storeName}`);
  let rows, high, read = false;
  const ready = () => {
    if (!rows || !read) return;
    try {
      if (high !== undefined && (!Number.isSafeInteger(high) || high < 0)) throw Error();
      const id = nextIdentifier(high ? [...rows, high] : rows);
      refs.put(id, `allocation:${storeName}`); accept(id);
    } catch { refuse(); }
  };
  keys.onsuccess = () => { rows = keys.result; ready(); }; mark.onsuccess = () => { high = mark.result; read = true; ready(); };
  keys.onerror = refuse; mark.onerror = refuse;
}
export async function createStoredProject(db, prepared, defaults = {}, initialThemes = null) {
  let data, appDefaults, catalog;
  try { data = structuredClone(prepared); appDefaults = structuredClone(defaults); catalog = initialThemes === null ? null : readThemeCatalog(initialThemes); }
  catch { throw failure("projectStorageFailed"); }
  return new Promise((resolve, reject) => {
    let t, answer;
    const refuse = () => { if (t) { try { t.abort(); } catch {} } reject(failure("projectStorageFailed")); };
    try {
      t = db.transaction(["apps", "chats", "files", "objects", "refs", "versions", ...(catalog ? ["settings"] : [])], "readwrite");
      t.oncomplete = () => answer ? resolve(answer) : reject(failure("projectStorageFailed"));
      t.onabort = () => reject(failure("projectStorageFailed")); t.onerror = refuse;
      const apps = t.objectStore("apps"), chats = t.objectStore("chats");
      let allocatedApp, allocatedChat, themeReady = !catalog, themeId;
      const write = () => {
        if (!allocatedApp || !allocatedChat || !themeReady) return;
        try {
          const id = allocatedApp, chatId = allocatedChat, at = new Date().toISOString();
          const app = { ...appDefaults, ...(catalog ? {themeId} : {}), id, name: data.name, path: data.name, createdAt: at, updatedAt: at };
          apps.add(app, id);
          for (const [path, text] of data.files) t.objectStore("files").add(text, `${id}:${path}`);
          for (const [key, bytes] of data.objects) t.objectStore("objects").put(bytes, key);
          t.objectStore("refs").add(data.oid, `${id}:main`);
          t.objectStore("versions").add([{ oid: data.oid, message: "Init Prism app", timestamp: Math.floor(Date.parse(at) / 1000), dbTimestamp: null, isFavorite: false, note: null }], id);
          chats.add({ id: chatId, appId: id, title: data.name, createdAt: at, messages: [], initialCommitHash: data.oid, dbTimestamp: null, chatMode: data.initialChatMode, modelSelection: null, referencedApps: [] }, chatId);
          answer = { app, chatId, oid: data.oid };
        } catch { refuse(); }
      };
      if (catalog) {
        const settings = t.objectStore("settings"), home = settings.get("user"), journal = settings.get("foundry:themes:v1");
        let homeRead = false, journalRead = false;
        const ready = () => { if (!homeRead || !journalRead) return; try { themeId = initialProjectTheme(home.result, journal.result, catalog); themeReady = true; write(); } catch { refuse(); } };
        home.onsuccess = () => { homeRead = true; ready(); }; journal.onsuccess = () => { journalRead = true; ready(); };
        home.onerror = refuse; journal.onerror = refuse;
      }
      allocateIdentifier(t, "apps", id => { allocatedApp = id; write(); }, refuse);
      allocateIdentifier(t, "chats", id => { allocatedChat = id; write(); }, refuse);
    } catch { refuse(); }
  });
}
export async function createStoredChat(db, appId, at = new Date().toISOString(), initialMode = "build") {
  const capturedMode = browserChatMode(initialMode);
  if (!Number.isSafeInteger(appId) || appId < 1) throw failure("projectStorageFailed");
  return new Promise((resolve, reject) => {
    let t, id;
    const refuse = () => { if (t) { try { t.abort(); } catch {} } reject(failure("projectStorageFailed")); };
    try {
      t = db.transaction(["apps", "chats", "refs"], "readwrite");
      t.oncomplete = () => id ? resolve(id) : reject(failure("projectStorageFailed"));
      t.onabort = () => reject(failure("projectStorageFailed")); t.onerror = refuse;
      const aq = t.objectStore("apps").get(appId);
      aq.onsuccess = () => {
        if (!aq.result) { refuse(); return; }
        allocateIdentifier(t, "chats", allocated => {
          try { id = allocated; t.objectStore("chats").add({ id, appId, title: "New chat", createdAt: at, messages: [], initialCommitHash: null, dbTimestamp: null, chatMode: capturedMode, modelSelection: null, referencedApps: [] }, id); }
          catch { refuse(); }
        }, refuse);
      };
      aq.onerror = refuse;
    } catch { refuse(); }
  });
}
