// The page origin's worker is the kernel of the shell. Three duties, no server behind any of them:
//  1. The shell as one versioned closure: manifest.json, written by the projector, lists every shell
//     file with its SHA-256; the cache is named by the closure digest, so a new release is a new
//     cache and a stale worker never serves a mixed shell. Only shell caches are ever dropped; the
//     previews cache and the engine's part cache belong to the page.
//  2. Previews and the capability gate: each project's preview is served under <base>p/<kappa>/
//     from the previews cache the host fills after every in-tab build, in a CSP sandbox with an
//     opaque origin, so model-written code never reaches this origin's keys or stores. Whether it is
//     served at all, and which hosts its policy names, is the kernel's answer: the capability gate
//     Lean proves (kernel.js). Every decision is appended to the audit store, hologram-live's audit
//     row shape.
//  3. Published applications: a .holo the page verified through PrismPM's validator in the core is
//     unpacked into the holo cache under <base>holo/<kappa>/ and served from there on any device;
//     a missing entry hands the URL to the runner page, which finds the bytes in the store or asks
//     for the file; <base>holo/<kappa>.holo downloads the bytes. A published application carries the
//     empty capability baseline: its grant names no host, so the kernel admits no network and its
//     files are served in a CSP sandbox with an opaque origin that reaches no other host and none of
//     this origin's stores.
const CLOSURE = "__CLOSURE__";   // written by the projector; a new shell is a new worker
const BASE = new URL(self.registration.scope).pathname;
const matchShell = async (request, options) => (await caches.open("shell-" + CLOSURE)).match(request, options);

// The kernel decides what an untrusted document may do: its capability gate is proved by Lean
// (FK-05 to FK-09 in UOR-Foundation/foundry). kernel.js checks the guest against its pin and replays
// every acceptance vector before the first question; until then, and if that fails, nothing
// untrusted is served.
importScripts("kernel.js", "shell-cache.js");
let kernelLoad = null;
const kernel = () => (kernelLoad ??= ForgeKernel.load(async (p) => (await matchShell(BASE + p)) || fetch(BASE + p, { cache: "no-store" })).catch((error) => { kernelLoad = null; throw error; }));

async function readManifest() {
  const r = await fetch("manifest.json", { cache: "no-store" });
  return r.json();
}

self.addEventListener("install", (event) => {
  event.waitUntil((async () => {
    const entries = await ShellClosure.prepare(self.registration.scope, CLOSURE);
    try {
      const cache = await caches.open("shell-" + CLOSURE);
      for (const [url, response] of entries) await cache.put(url, response);
    } catch {
      throw Object.assign(new Error("Shell installation refused: shellClosureUnavailable"), { code: "shellClosureUnavailable" });
    }
    await self.skipWaiting();
  })());
});

self.addEventListener("activate", (event) => {
  event.waitUntil((async () => {
    const keep = "shell-" + CLOSURE;
    for (const name of await caches.keys()) if (name.startsWith("shell-") && name !== keep) await caches.delete(name);
    await self.clients.claim();
  })());
  // A page that loaded under the previous worker may hold files of two closures (the page from the
  // network, a script from the old cache): every open page of the shell is told a newer shell took over,
  // and reloads once, when no turn runs, so what runs is one closure. Preview and application frames are left alone. This is deliberately outside
  // waitUntil and never awaited: a navigation waits for the worker to be active, so waiting on it here
  // would leave this worker activating for ever and every page on the shell it already had.
  (async () => {
    // The page decides when: it reloads at once if no turn runs, else as soon as the running turn ends
    // (host.js), so a build, a proof or a publish is never cut short by a deploy.
    for (const client of await self.clients.matchAll({ type: "window" })) {
      const path = new URL(client.url).pathname.slice(BASE.length);
      if (client.url.startsWith(self.registration.scope) && !/^(p|holo|ui)\//.test(path)) { try { client.postMessage({ type: "shell-updated", closure: CLOSURE }); } catch (e) {} }
    }
  })();
});

// ---- previews and the capability gate
const openDb = () => new Promise((res, rej) => { const r = indexedDB.open("hologram-forge", 2); r.onupgradeneeded = () => { const d = r.result; for (const s of ["settings", "apps", "chats", "files", "objects", "refs", "versions", "audit"]) if (!d.objectStoreNames.contains(s)) d.createObjectStore(s); if (!d.objectStoreNames.contains("seals")) { const st = d.createObjectStore("seals", { keyPath: "id" }); st.createIndex("kind", "kind"); st.createIndex("promptKappa", "promptKappa"); } }; r.onsuccess = () => res(r.result); r.onerror = () => rej(r.error); });
async function audit(record) { const db = await openDb(); await new Promise((res, rej) => { const t = db.transaction("audit", "readwrite"); t.objectStore("audit").put({ timestamp_millis: Date.now(), ...record }, Date.now() + ":" + record.resource); t.oncomplete = res; t.onerror = () => rej(t.error); }); }
async function grants(id) { const db = await openDb(); return new Promise((res, rej) => { const q = db.transaction("refs", "readonly").objectStore("refs").get("capabilities:" + id); q.onsuccess = () => res(q.result || null); q.onerror = () => rej(q.error); }); }
async function principalOf(e) {
  if (!e.clientId) return null;
  const c = await self.clients.get(e.clientId);
  if (!c) return null;
  const m = new URL(c.url).pathname.slice(BASE.length).match(/^(p|holo)\/([^/]+)\//);
  return m ? { kind: m[1], id: m[2] } : null;
}

// ---- untrusted documents
// A preview runs code the model wrote; an installed application runs code someone else wrote. Both
// are served from this origin, where the keys, chats and projects live, so both are served with a
// CSP sandbox: the document gets an opaque origin and cannot open this origin's IndexedDB, Cache API
// or storage, nor reach into the page that framed it. An opaque document is no client of this
// worker, so the worker never sees its requests; its network therefore rides in the same header.
//
// What goes in the header is the kernel's answer, not this worker's. A grant record is
// refs["capabilities:<id>"] = { serve, network: { endpoints, denied } }, written by the page when it
// builds a preview or installs an application. The kernel is asked `gate preview|published serve ...`
// before the document is served at all, and `gate ... fetch ...` once per granted endpoint; only the
// endpoints it allows enter the policy. No record means no grant: the kernel refuses to serve.
const SANDBOX = "sandbox allow-scripts allow-forms allow-popups allow-modals allow-downloads allow-pointer-lock allow-presentation allow-orientation-lock";
const ENDPOINT_FORM = /^https:\/\/[a-z0-9-]+(\.[a-z0-9-]+)+(:[0-9]{1,5})?$/;
function previewGrantValid(record, id) {
  const endpoints = record?.network?.endpoints;
  if (!Array.isArray(endpoints)) return false;
  if (endpoints.length === 0 && record?.previewConsent?.purpose !== "portable-build") return true;
  if (record?.previewConsent?.purpose === "portable-build") {
    const {expires,grantedAt} = record.previewConsent, at = Date.now();
    if (!Number.isFinite(expires) || !Number.isFinite(grantedAt) || at < grantedAt || at >= expires || expires > grantedAt + 120000) return false;
  }
  return record?.previewConsent?.profile === "foundry/preview-consent/1" && record.previewConsent.id === id && /^blake3:[0-9a-f]{64}$/.test(record.previewConsent.source);
}
function policyOf(endpoints) {
  const named = (head) => (endpoints.length ? head + " " + endpoints.join(" ") : head);
  return [SANDBOX, "default-src 'none'", named("script-src 'unsafe-inline' 'wasm-unsafe-eval' blob: data:"), named("style-src 'unsafe-inline' blob: data:"),
    named("img-src blob: data:"), named("font-src blob: data:"), named("media-src blob: data:"), endpoints.length ? "connect-src " + endpoints.join(" ") : "connect-src 'none'",
    "worker-src blob:", "frame-src 'none'", "form-action 'none'", "base-uri 'self'"].join("; ");
}
const flag = (b) => (b ? "yes" : "no");
async function gate(origin, effect, record, granted, denied) {
  return (await (await kernel()).ask(`gate ${origin} ${effect} ${flag(record)} ${flag(granted)} ${flag(denied)}`)) === "allow";
}
// What each app was let reach, for the user's own record on this device (shell/telemetry): the kernel's
// decisions here, as they are made, handed to one open page of the shell. That page asks the kernel before
// it keeps them (`telemetry store`, FK-25); nothing here keeps or sends them anywhere else. With no page
// open there is no record, which loses a count and never a decision.
const nowMicros = () => Date.now() * 1000;
async function noteGate(kind, id, decisions) {
  try {
    const shell = (await self.clients.matchAll({ type: "window" })).filter((c) => c.url.startsWith(self.registration.scope) && !/^(p|holo|app)\//.test(new URL(c.url).pathname.slice(BASE.length)));
    const page = shell.find((c) => !/^ui\//.test(new URL(c.url).pathname.slice(BASE.length))) || shell[0];
    if (page && decisions.length) page.postMessage({ type: "telemetry/gate", id: kind + ":" + id, installed: kind === "holo", decisions });
  } catch (e) {}
}
// The policy for one untrusted document, or null when the kernel refuses to serve it. Each decision is
// added to `decisions` in the words the Telemetry model counts: notOpened, allowed or blocked, with the site.
async function policyFor(kind, id, decisions = []) {
  const origin = kind === "p" ? "preview" : "published";
  const record = await grants(id);
  const network = (record && record.network) || {};
  const denied = new Set(Array.isArray(network.denied) ? network.denied : []);
  const served = await gate(origin, "serve", !!record, !!(record && record.serve === true && (kind !== "p" || previewGrantValid(record, id))), false);
  await audit({ principal: kind + ":" + id, operation: "kernel.gate.serve", resource: id, outcome: served ? "granted" : "refused" });
  if (!served) { decisions.push({ kind: "notOpened", at: Date.now() * 1000 }); return null; }
  const allowed = [];
  for (const endpoint of Array.isArray(network.endpoints) ? network.endpoints : []) {
    if (typeof endpoint !== "string" || !ENDPOINT_FORM.test(endpoint)) continue;
    const yes = await gate(origin, "fetch", true, true, denied.has(endpoint));
    if (yes) allowed.push(endpoint);
    decisions.push({ kind: yes ? "allowed" : "blocked", at: Date.now() * 1000, site: endpoint });
  }
  return policyOf(allowed);
}
function untrusted(response, policy) {
  const headers = new Headers(response.headers);
  headers.set("content-security-policy", policy);
  return new Response(response.body, { status: response.status, statusText: response.statusText, headers });
}
// `opening` is true for the page an app opens with, so an app's own files are not each counted as an opening.
async function serveUntrusted(kind, id, hit, opening = true) {
  let policy;
  const decisions = [];
  try { policy = await policyFor(kind, id, decisions); } catch (error) {
    if (opening) noteGate(kind, id, [{ kind: "notOpened", at: nowMicros() }]);
    return new Response("not served: the kernel did not verify (" + error.message + ")", { status: 503, headers: { "content-type": "text/plain; charset=utf-8" } });
  }
  if (opening) {
    // What the app was served: the bytes of its page on this device.
    let size = 0;
    if (policy) try { size = Number(hit.headers.get("content-length")) || (await hit.clone().blob()).size; } catch (e) {}
    noteGate(kind, id, policy ? [{ kind: "opened", at: nowMicros(), value: size }, ...decisions] : decisions);
  }
  if (!policy) return new Response("not served: no grant to serve " + id, { status: 403, headers: { "content-type": "text/plain; charset=utf-8" } });
  return untrusted(hit, policy);
}

// ---- published applications
async function holoBytes(id) { const db = await openDb(); return new Promise((res, rej) => { const q = db.transaction("objects", "readonly").objectStore("objects").get(id); q.onsuccess = () => res(q.result || null); q.onerror = () => rej(q.error); }); }
async function serveHolo(request, path) {
  const rest = path.slice("holo/".length);
  const download = rest.match(/^([^/]+)\.holo$/);
  if (download) {
    const bytes = await holoBytes(download[1]);
    if (!bytes) return new Response("no application " + download[1], { status: 404 });
    return new Response(bytes, { headers: { "content-type": "application/octet-stream", "content-disposition": "attachment; filename=\"" + download[1].replace(/^blake3:/, "") + ".holo\"" } });
  }
  const slash = rest.indexOf("/");
  if (slash < 0) return Response.redirect(request.url + "/", 302);
  const id = rest.slice(0, slash);
  let file = rest.slice(slash + 1) || "index.html";
  if (file.endsWith("/")) file += "index.html";
  const cache = await caches.open("holo");
  const hit = await cache.match(BASE + "holo/" + id + "/" + file);
  if (hit) return serveUntrusted("holo", id, hit, file === "index.html" || request.mode === "navigate");
  if (file === "index.html" || !file.includes(".")) {
    const runner = (await caches.match("holo.html")) || (await fetch(new Request(BASE + "holo.html", { cache: "no-store" })).catch(() => null));
    if (runner && runner.ok) return new Response(await runner.text(), { status: 200, headers: { "content-type": "text/html; charset=utf-8" } });
  }
  return new Response("no application file " + id + "/" + file, { status: 404 });
}

self.addEventListener("fetch", (event) => {
  const url = new URL(event.request.url);
  if (url.origin === self.location.origin) {
    if (!url.pathname.startsWith(BASE)) return;
    const path = url.pathname.slice(BASE.length);
    if (path.startsWith("holo/")) { event.respondWith(serveHolo(event.request, path)); return; }
    if (path.startsWith("p/")) {
      const id = path.split("/")[1];
      event.respondWith((async () => {
        const cache = await caches.open("previews");
        const hit = await cache.match(BASE + "p/" + id + "/index.html");
        return hit ? serveUntrusted("p", id, hit) : new Response("no preview " + id, { status: 404 });
      })());
      return;
    }
    if (event.request.method !== "GET") return;
    // app/ is the Forge kernel's Prism page, a separate site with its own worker: never answered here.
    if (path === "app" || path.startsWith("app/")) return;
    // Renderer and scaffold bytes come from this worker's verified version, even offline.
    if (path.startsWith("assets/") || path.startsWith("scaffold/") || path.startsWith("editor/") || path.startsWith("compiler/")) {
      event.respondWith((async () => (await (await caches.open("shell-" + CLOSURE)).match(event.request, { ignoreSearch: true })) || new Response("File absent from the active shell", { status: 404 }))()); return;
    }
    // The hash list is the staleness oracle: the page compares the closure it was projected with
    // against the one this origin serves now, so it is never answered from a cache while the network
    // can answer. Offline, the cached copy is the truth.
    if (path === "manifest.json" || path === "provenance.json") { event.respondWith(fetch(event.request, { cache: "no-store" }).catch(() => matchShell(event.request, { ignoreSearch: true }))); return; }
    event.respondWith((async () => {
      const hit = await matchShell(event.request, { ignoreSearch: true });
      if (hit) return hit;
      // Pages maps unknown renderer routes to its HTML fallback. Keep those navigations
      // on this closure too; a route allowlist would drift when a newer renderer adds one.
      // Subapps, engine/API and resource namespaces keep their existing ownership.
      const separate = /^(?:app|ui|p|holo|v1|engine|assets|scaffold|editor|compiler|live|q|kernel|prism|prism-code|fonts|wallpapers)(?:\/|$)/.test(path);
      if (event.request.mode === "navigate" && !separate) {
        return (await matchShell(BASE + "index.html")) || new Response("Document absent from the active shell", { status: 404 });
      }
      try {
        return await fetch(event.request);
      } catch (error) {
        if (event.request.mode === "navigate") return (await matchShell(BASE + "index.html")) || (await matchShell(BASE));
        throw error;
      }
    })());
    return;
  }
  event.respondWith((async () => {
    const who = await principalOf(event);
    if (!who) return fetch(event.request);
    // A frame this worker still controls under p/ or holo/: the kernel's gate decides each request.
    const record = await grants(who.id);
    const network = (record && record.network) || {};
    const granted = Array.isArray(network.endpoints) && network.endpoints.includes(url.origin) && (who.kind !== "p" || previewGrantValid(record, who.id));
    const denied = Array.isArray(network.denied) && network.denied.includes(url.origin);
    let allowed = false;
    try { allowed = await gate(who.kind === "p" ? "preview" : "published", "fetch", !!record, granted, denied); } catch (error) { allowed = false; }
    await audit({ principal: who.kind + ":" + who.id, operation: "kernel.gate.fetch", resource: url.origin, outcome: allowed ? "granted" : "refused" });
    noteGate(who.kind, who.id, [{ kind: allowed ? "allowed" : "blocked", at: nowMicros(), site: url.origin }]);
    if (allowed) return fetch(event.request);
    return new Response("refused by capabilities: " + url.origin, { status: 403, headers: { "content-type": "text/plain" } });
  })());
});
