import {readUserSettings,replaceUserSettings} from "./settings-transactions.mjs";
// The page's adapters to the verified core, shared by the builder's host (host.js) and holo.html: core.wasm
// through its JSON ABI, the view, addresses from the holospaces wasm, and the device store. Foundry's
// language model is its reader in the kernel guest (host.js); no engine, weights or server are here.

const BASE = new URL("./", import.meta.url);
const enc = new TextEncoder();
export const short = (k) => (k ? String(k).replace(/^blake3:/, "").replace("did:holo:sha256:", "").slice(0, 12) + "…" : "");

// ---- the verified core, through its JSON ABI
let core = null, coreLoading = null;
export function coreReady() {
  if (core) return Promise.resolve(core);
  if (coreLoading) return coreLoading;
  coreLoading = (async () => {
    const { instance } = await WebAssembly.instantiateStreaming(fetch(new URL("core.wasm", BASE)), {});
    const { memory, holo_alloc, holo_free, holo_run, holo_seal, holo_open } = instance.exports;
    // Bytes in, bytes out through one export: a JSON request, or a key then a record to seal or open (FA-93);
    // null when a record does not open.
    const call = (fn, key, bytes) => {
      const n = key.length + bytes.length, ptr = holo_alloc(n);
      new Uint8Array(memory.buffer, ptr, n).set(key); new Uint8Array(memory.buffer, ptr + key.length, bytes.length).set(bytes);
      const packed = fn(ptr, n), outPtr = Number(packed >> 32n), outLen = Number(packed & 0xffffffffn), out = packed < 0n ? null : new Uint8Array(memory.buffer, outPtr, outLen).slice();
      holo_free(ptr, n); if (out) holo_free(outPtr, outLen);
      return out;
    };
    const none = new Uint8Array(0);
    core = {
      seal: (key, bytes) => call(holo_seal, key, bytes),
      open: (key, bytes) => call(holo_open, key, bytes),
      run(request) {
        const out = JSON.parse(new TextDecoder().decode(call(holo_run, none, enc.encode(JSON.stringify(request)))));
        if (out.error) throw new Error(out.error);
        return out;
      },
    };
    return core;
  })();
  return coreLoading;
}
export let VIEW = null;
export async function viewReady() { if (VIEW) return VIEW; const inline = document.getElementById("view"); VIEW = inline ? JSON.parse(inline.textContent) : (await coreReady()).run({ op: "view" }); return VIEW; }

// ---- BLAKE3 addresses from the holospaces wasm (the same κ every daemon computes)
let kappaFn = null;
export async function kappaReady() {
  if (kappaFn) return kappaFn;
  const mod = await import(new URL("q/pkg/holospaces_web.js", BASE).href);
  if (typeof mod.default === "function") { try { await mod.default(); } catch (e) {} }
  kappaFn = (bytes) => String(mod.kappa(bytes));
  return kappaFn;
}
export async function kappa(bytes) { return (await kappaReady())(bytes); }

// ---- the device store: one database for the builder and the seals. Seals are rows keyed by κ,
// indexed by kind and by memo prompt κ; the builder's own stores sit beside them.
export const DB = "hologram-forge", DB_VERSION = 2;
export const STORES = ["settings", "apps", "chats", "files", "objects", "refs", "versions", "audit"];
let dbHandle = null;
export function db() {
  if (dbHandle) return Promise.resolve(dbHandle);
  return new Promise((resolve, reject) => {
    const req = indexedDB.open(DB, DB_VERSION);
    req.onupgradeneeded = () => {
      const d = req.result;
      for (const s of STORES) if (!d.objectStoreNames.contains(s)) d.createObjectStore(s);
      if (!d.objectStoreNames.contains("seals")) { const s = d.createObjectStore("seals", { keyPath: "id" }); s.createIndex("kind", "kind"); s.createIndex("promptKappa", "promptKappa"); }
    };
    req.onsuccess = () => resolve((dbHandle = req.result)); req.onerror = () => reject(req.error);
  });
}
// ---- settings: the renderer's own record
export function settingsGet() { return readUserSettings(workspace()); }
export function settingsPut(s) { return replaceUserSettings(workspace(),s); }

// ---- Prism ID: the Prism frame (frame/, locked by frame.lock; the renderer's build bundles it as
// assets/prism-frame.js) keeps the visitor's Prism ID. One frame serves the page. Its panel is a centred
// popover, so the element stays off screen, and it is told Studio's theme first, since a frame sets the page's.
export const prismFrame = (globalThis.__prismFrame ??= () => (globalThis.__prismFrameLoad ??= import(new URL("assets/prism-frame.js", BASE).href).then(() => {
  localStorage.setItem("prism-theme", document.documentElement.classList.contains("dark") ? "dark" : "light");
  const frame = Object.assign(document.createElement("prism-frame"), { style: "position:fixed;width:1px;height:1px;overflow:hidden;clip-path:inset(50%)" });
  return document.body.appendChild(frame);
})));

// ---- the visitor's work, sealed to their Prism ID (FA-93, verification/private-workspace.md). Each record
// is sealed by the verified core (ChaCha20-Poly1305 behind hologram-space's PayloadCipher seam) under the
// workspace key the Prism frame keeps for this origin, which only a device of the visitor's Prism ID opens.
// The core seals synchronously, so a record is sealed inside its own transaction and no writer changes. Foundry's Prism kernel (app/src/Workspace.lex.tex) decides whether
// the workspace opens, the one move of the work kept before it, and each read and store: it answers every
// case once, as the workspace opens. Left plain: what the service worker reads (the grants and published
// .holo bytes, public and addressed by their content) and the audit it writes.
const SEALED = new Set(["settings", "apps", "chats", "files", "objects", "refs", "versions"]);
// A gallery card is made to be published, and its picture is a Blob, which cannot be read inside a transaction.
const plainPlace = (store, key) => !SEALED.has(store) || (store === "refs" && /^(capabilities|holo|gallery):/.test(key));
const isSealed = (value) => value?.sealedWork instanceof Uint8Array;
const word = (n) => [0, 8, 16, 24].map((shift) => ((n >>> shift) & 255).toString(16).padStart(2, "0")).join("");
export const workspaceRequest = (...words) => "workspace " + [1, ...words].map(word).join("");
const decide = (...words) => globalThis.__forgeKernel.ask(workspaceRequest(...words));
// The facts the kernel opens on, from the frame's answer for the key: this device was removed, the session is
// live, the ID owns the workspace, the device holds the current key.
export const openFacts = (code) => [+(code === "removed"), +(!code || code === "removed" || code === "stale-key"), 1, +(code !== "stale-key")];
const refusal = (code) => Object.assign(new Error(code), { code });
// What the visitor is told, by the kernel's answer. The renderer says it (hologram.screens/AppSidebar.tsx),
// including what was told before it mounted.
export const tell = (answer) => { (globalThis.__workspaceTold ??= []).push(answer); window.dispatchEvent(new CustomEvent("foundry:workspace", { detail: answer })); };
const base64 = (bytes) => { let text = ""; for (let i = 0; i < bytes.length; i += 32768) text += String.fromCharCode(...bytes.subarray(i, i + 32768)); return btoa(text); };
// Values as JSON, as IndexedDB would keep them: bytes, dates and a key whose value is undefined are tagged, then
// restored by assignment, so the key stays (a reviver returning undefined would drop it).
const encode = (value) => enc.encode(JSON.stringify(value, function (name, v) { const raw = this[name]; return raw instanceof Uint8Array ? { $bytes: base64(raw) } : raw instanceof Date ? { $date: v } : raw === undefined ? { $undefined: 1 } : v; }));
const unbase64 = (text) => Uint8Array.from(atob(text), (c) => c.charCodeAt(0));
const tag = (v) => (v && typeof v === "object" && Object.keys(v).length === 1 ? Object.keys(v)[0] : "");
const restore = (v) => tag(v) === "$bytes" ? unbase64(v.$bytes) : tag(v) === "$date" ? new Date(v.$date) : tag(v) === "$undefined" ? undefined : v && typeof v === "object" ? (Object.keys(v).forEach((k) => (v[k] = restore(v[k]))), v) : v;
const decode = (bytes) => restore(JSON.parse(new TextDecoder().decode(bytes)));
const wrap = (target, own) => new Proxy(target, { get: (t, p) => (p in own ? own[p] : typeof t[p] === "function" ? t[p].bind(t) : t[p]), set: (t, p, v) => ((p in own ? (own[p] = v) : (t[p] = v)), true) });
// What changed in the visitor's work, after its transaction commits: (store, key, value, deleted). Sync keeps
// each app's last change by it (host.js, FA-95).
const written = new Set();
export const onWorkspaceWrite = (fn) => written.add(fn);
let workspaceOpen = null;
export const workspace = () => (workspaceOpen ??= openWorkspace());
async function openWorkspace() {
  // The cipher is the verified core's (core.wasm, hologram-space's PayloadCipher seam): it is loaded at start
  // already, and it seals synchronously, inside a writer's own transaction.
  const [verified, handle, frame] = await Promise.all([coreReady(), db(), prismFrame()]);
  let secret = null, code = null;
  try { secret = await frame.workspaceKey(); } catch (error) { code = error?.code ?? "unavailable"; }
  let opened = await decide(0, ...openFacts(code));
  // A published .holo is public, and holo.js keeps its bytes plain beside its ref: the one plain record a
  // sealed store may hold.
  const holos = new Set((await read(handle, "refs", "getAllKeys")).filter((k) => typeof k === "string" && k.startsWith("holo:")).map((k) => k.slice(5)));
  const published = (store, key, value) => store === "objects" && value instanceof Uint8Array && holos.has(key);
  // A removed device's work is sealed under a key it no longer holds: it is cleared, with anything else kept
  // where work belongs, so nothing left behind enters the new workspace, and the device starts again.
  if (opened === "workspaceDeviceRemoved") {
    await everyRecord(handle, "readwrite", (store, cursor) => plainPlace(store, cursor.key) || published(store, cursor.key, cursor.value) || cursor.delete());
    // The pictures of that work go too: built previews and covers, which the worker serves without the ID.
    await Promise.all(["previews", "covers"].map((name) => caches.delete(name)));
    secret = await frame.workspaceKey({ replace: true });
    tell(opened);
    opened = await decide(0, ...openFacts(null));
  }
  if (opened !== "ok") { tell(opened); throw refusal(opened); }
  const reads = {}, stores = {};
  for (const a of [0, 1]) { stores[a] = await decide(2, 1, 1, a); for (const b of [0, 1]) reads[`${a}${b}`] = await decide(1, 1, a, b); }
  const estimate = await navigator.storage?.estimate?.().catch(() => null);
  let usage = estimate?.usage ?? 0, quota = estimate?.quota ?? Infinity;
  navigator.storage?.persist?.().catch(() => {});
  const place = (store, key) => JSON.stringify([store, key]);
  const seal = (store, key, value) => {
    if (plainPlace(store, key)) return value;
    const bytes = encode([place(store, key), value]);
    const answer = stores[+(usage + bytes.length < quota)];
    if (answer !== "ok") { tell(answer); throw refusal(answer); }
    usage += bytes.length;
    return { sealedWork: verified.seal(secret, bytes) };
  };
  const open = (store, key, value) => {
    if (value === undefined || plainPlace(store, key) || published(store, key, value)) return value;
    let authentic = 0, bound = 0, kept;
    try { const [where, inner] = decode(verified.open(secret, value.sealedWork)); [authentic, bound, kept] = [1, +(where === place(store, key)), inner]; } catch {}
    const answer = reads[`${authentic}${bound}`];
    return answer === "ok" ? kept : void tell(answer);
  };
  // The one move: every record kept before the workspace is sealed in place, in one transaction, so its plain
  // copy is gone exactly when the sealed copy commits. A published .holo's own bytes stay as they are.
  const moved = (await read(handle, "refs", "get", "workspace:moved")) !== undefined;
  if ((await decide(3, 1, +moved)) === "ok") {
    let count = 0;
    const visit = (store, cursor) => isSealed(cursor.value) || plainPlace(store, cursor.key) || (store === "objects" && holos.has(cursor.key)) || (cursor.update(seal(store, cursor.key, cursor.value)), count++);
    await everyRecord(handle, "readwrite", visit, (tx) => tx.objectStore("refs").put(seal("refs", "workspace:moved", true), "workspace:moved"));
    if (count) tell("workspaceMoved");
  }
  const request = (req, read) => wrap(req, { get result() { return read(req.result); } });
  // `note` keeps what a writer changed; the transaction tells onWorkspaceWrite once it commits.
  const objectStore = (name, s, note) => wrap(s, {
    get: (key) => request(s.get(key), (v) => open(name, key, v)),
    put: (v, key) => (note(name, key, v), s.put(seal(name, key, v), key)),
    add: (v, key) => (note(name, key, v), s.add(seal(name, key, v), key)),
    delete: (key) => (note(name, key, undefined, true), s.delete(key)),
    clear: () => (note(name, null, undefined, true), s.clear()),
    getAll: (range, count) => { const keys = s.getAllKeys(range, count); return request(s.getAll(range, count), (vs) => vs.map((v, i) => open(name, keys.result[i], v)).filter((v) => v !== undefined)); },
    openCursor: (range, direction) => request(s.openCursor(range, direction), (c) => c && wrap(c, {
      get value() { return open(name, c.key, c.value); },
      update: (v) => (note(name, c.key, v), c.update(seal(name, c.key, v))),
      delete: () => (note(name, c.key, open(name, c.key, c.value), true), c.delete()),
    })),
  });
  return wrap(handle, { transaction: (...args) => {
    const tx = handle.transaction(...args), notes = [];
    tx.addEventListener("complete", () => { for (const n of notes) for (const fn of written) fn(...n); });
    return wrap(tx, { objectStore: (name) => objectStore(name, tx.objectStore(name), (...n) => notes.push(n)) });
  } });
}
// One transaction over every sealed store, visiting each record; `last` adds a write before it commits.
function everyRecord(handle, mode, visit, last) {
  return new Promise((done, fail) => {
    const tx = handle.transaction([...SEALED], mode);
    let left = SEALED.size;
    for (const store of SEALED) {
      const walk = tx.objectStore(store).openCursor();
      walk.onsuccess = () => { const cursor = walk.result; if (cursor) { visit(store, cursor); cursor.continue(); } else if (!--left && last) last(tx); };
    }
    tx.oncomplete = () => done();
    tx.onerror = tx.onabort = () => fail(tx.error ?? refusal("storageTransactionFailed"));
  });
}
const read = (handle, store, how, key) => new Promise((done, fail) => { const r = handle.transaction(store).objectStore(store)[how](key); r.onsuccess = () => done(r.result); r.onerror = () => fail(r.error); });
