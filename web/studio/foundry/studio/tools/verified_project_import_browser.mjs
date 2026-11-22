// Real transactions qualify allocation, multi-store rollback and collision refusal.
import assert from "node:assert/strict";
import fs from "node:fs";
import http from "node:http";
import { chromium } from "../vendor/prism/node_modules/playwright-core/index.mjs";
const host = fs.readFileSync(new URL("../shell/host.js", import.meta.url), "utf8");
const importBody = host.slice(host.indexOf('const EXPORT_SCHEMA ='), host.indexOf('// ---- versions:'));
assert.ok(importBody.includes('handlers.set("forge:import"'));
const imports = `export function importHandler(env) { const { handlers, kappa, unb64, all, nextId, nulls, enc, dbReady, prepareProjectImport, coreReady, core } = env; ${importBody}\nreturn handlers.get("forge:import"); }`;
const sources = new Map([
  ...fs.readdirSync(new URL("../shell/", import.meta.url)).filter(x => x.endsWith(".mjs")).map(x => ["/" + x, fs.readFileSync(new URL("../shell/" + x, import.meta.url))]),
  ["/creation.mjs", fs.readFileSync(new URL("../shell/project-creation.mjs", import.meta.url))],
  ["/storage.mjs", fs.readFileSync(new URL("../shell/storage-request.mjs", import.meta.url))],
  ["/blake3.mjs", fs.readFileSync(new URL("../../try/vendor/prism-code/src/blake3.mjs", import.meta.url))],
  ["/import.mjs", imports],
]);
const server = http.createServer((request, response) => {
  const source = sources.get(request.url);
  response.setHeader("content-type", source ? "text/javascript" : "text/html");
  response.end(source || "<!doctype html><title>Atomic creation acceptance</title>");
});
await new Promise(resolve => server.listen(0, "127.0.0.1", resolve));
const executablePath = process.env.CHROME_BIN || ["/usr/bin/chromium", "/usr/bin/google-chrome"].find(p => fs.existsSync(p));
let browser;
try {
  browser = await chromium.launch({ executablePath, headless: true, args: ["--no-sandbox"] });
  const page = await browser.newPage(); await page.goto(`http://127.0.0.1:${server.address().port}/`);
  const report = await page.evaluate(async () => {
    const { prepareNewProject, createStoredProject, createStoredChat } = await import("/creation.mjs");
    const { storageRequest } = await import("/storage.mjs"); const { kappa } = await import("/blake3.mjs");
    const { importHandler } = await import("/import.mjs");
    const { prepareProjectImport } = await import("/project-import.mjs");
    const check = (v, m) => { if (!v) throw Error(m); };
    const stores = ["apps", "chats", "files", "objects", "refs", "versions"];
    const db = await new Promise((resolve, reject) => { const q = indexedDB.open("atomic-creation", 1); q.onupgradeneeded = () => stores.forEach(s => q.result.createObjectStore(s)); q.onsuccess = () => resolve(q.result); q.onerror = () => reject(q.error); });
    const all = store => storageRequest(db, store, "readonly", s => s.getAll());
    const get = (store, key) => storageRequest(db, store, "readonly", s => s.get(key));
    // This independent preimage oracle scopes this test to the adapter/storage,
    // not qualification of the existing generated core's own byte projection.
    const core = { run: ({ project, parent, op }) => op === "restore" ? { decision: "Accept" } : ({ bytes: JSON.stringify({ entries: project.entries.map(e => [e.path, e.kappa, e.bytes]), name: project.label, parent }) }) };
    const prepare = name => prepareNewProject({ name, files: { "src/a.txt": "\uFEFFexact 😀\r\n" }, initialChatMode: "build" }, { core, derive: kappa });
    const encode = bytes => btoa(String.fromCharCode(...bytes));
    const makeExport = async (name = "Example") => {
      const p = await prepare(name);
      return { schema: "hologram-forge/export/1", apps: [{ app: { name, grants: ["https://secret.test"], apiKey: "secret", provider: "secret" }, files: Object.fromEntries(p.files), main: p.oid, versions: [{ oid: p.oid }], rules: { phase: "published" }, chats: [{ title: "Old chat", referencedApps: [99], modelSelection: { provider: "secret" }, messages: [{ id: 1, role: "user", content: "hello" }] }] }], objects: Object.fromEntries(p.objects.map(([key, bytes]) => [key, encode(bytes)])) };
    };
    const nextId = async store => Math.max(0, ...(await all(store)).map(row => row.id)) + 1;
    const importer = database => importHandler({ prepareProjectImport, core, coreReady: Promise.resolve(), handlers: new Map(), kappa, all, nextId, nulls: {}, dbReady: Promise.resolve(database) });
    const run = importer(db), input = await makeExport();
    const result = await run(input), id = result.apps[0].id;
    const app = await get("apps", id), chat = (await all("chats"))[0];
    check(app.grants === undefined && app.apiKey === undefined && app.provider === undefined, "authority imported");
    check(chat.referencedApps.length === 0 && chat.modelSelection === null && !(await get("refs", `prism:${id}`)), "cached authority imported");
    const moved = await run(input), movedId = moved.apps[0].id;
    const oid = await get("refs", `${movedId}:main`), body = await get("objects", oid), state = JSON.parse(new TextDecoder().decode(body));
    check(state.name === "Example (moved)" && (await get("apps", movedId)).name === state.name && state.parent === input.apps[0].main && kappa(body) === oid, "rename snapshot incoherent");
    const snapshot = async () => JSON.stringify(await Promise.all(stores.map(all)));
    const before = await snapshot();
    const bad = [];
    let d = await makeExport("missing"); delete d.objects[d.apps[0].main]; bad.push(d);
    d = await makeExport("corrupt"); d.objects[d.apps[0].main] = encode(new TextEncoder().encode("bad")); bad.push(d);
    d = await makeExport("mismatch"); d.apps[0].files["src/a.txt"] = "wrong"; bad.push(d);
    for (const invalid of bad) { let failed = false; try { await run(invalid); } catch { failed = true; } check(failed && await snapshot() === before, "invalid import changed storage"); }
    const failingDb = { transaction(names, mode) {
      const t = db.transaction(names, mode);
      return new Proxy(t, { get(target, key) {
        if (key === "objectStore") return name => name === "versions" ? { put() { throw Error("injected failure after queued writes"); } } : target.objectStore(name);
        const value = target[key]; return typeof value === "function" ? value.bind(target) : value;
      }, set(target, key, value) { target[key] = value; return true; } });
    } };
    let failed = false; try { await importer(failingDb)(await makeExport("late failure")); } catch (e) { failed = e.code === "projectStorageFailed"; }
    check(failed && await snapshot() === before, "partial import survived abort");
    const recovery = await run(await makeExport("recovery")); check(recovery.apps[0].id === 3, "aborted import consumed ID");
    db.close();
    return { closure: "verified", authority: "discarded", collision: "child-snapshot", rejectedImports: 3, lateFailure: "rolled-back", recovery: "complete" };
  });
  assert.deepEqual(report, { closure: "verified", authority: "discarded", collision: "child-snapshot", rejectedImports: 3, lateFailure: "rolled-back", recovery: "complete" });
  console.log(JSON.stringify(report));
} finally { if (browser) await browser.close(); await new Promise(resolve => server.close(resolve)); }
