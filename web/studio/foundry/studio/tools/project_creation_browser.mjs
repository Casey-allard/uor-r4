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
    const first = await createStoredProject(db, await prepare("first"));
    const projects = await Promise.all(["second", "third", "fourth"].map(async name => createStoredProject(db, await prepare(name))));
    check(new Set([first, ...projects].map(x => x.app.id)).size === 4, "concurrent app IDs collide");
    await Promise.all(Array.from({ length: 8 }, () => createStoredChat(db, first.app.id)).concat([createStoredProject(db, await prepare("fifth"))]));
    const chats = await all("chats"); check(chats.length === 13 && new Set(chats.map(c => c.id)).size === 13, "chat IDs collide");
    const apps = await all("apps"); check(apps.length === 5, "creation lost a project");
    for (const app of apps) {
      const text = await get("files", `${app.id}:src/a.txt`); check(text === "\uFEFFexact 😀\r\n", "file changed");
      const oid = await get("refs", `${app.id}:main`), snapshot = await get("objects", oid);
      check(kappa(snapshot) === oid, "snapshot identity mismatch");
      const entry = JSON.parse(new TextDecoder().decode(snapshot)).entries[0];
      const file = await get("objects", entry[1]); check(kappa(file) === entry[1] && file.length === entry[2], "file object missing");
      check((await get("versions", app.id))[0].oid === oid, "initial version missing");
      check(chats.some(c => c.appId === app.id && c.initialCommitHash === oid), "initial chat missing");
    }
    const snapshot = async () => JSON.stringify(await Promise.all(stores.map(all)));
    const before = await snapshot();
    const failingDb = { transaction(names, mode) {
      const t = db.transaction(names, mode);
      return new Proxy(t, { get(target, key) {
        if (key === "objectStore") return name => name === "versions" ? { add() { throw Error("injected failure after queued writes"); } } : target.objectStore(name);
        const value = target[key]; return typeof value === "function" ? value.bind(target) : value;
      }, set(target, key, value) { target[key] = value; return true; } });
    } };
    let refused = false;
    try { await createStoredProject(failingDb, await prepare("aborted")); } catch (e) { refused = e.code === "projectStorageFailed"; }
    check(refused && await snapshot() === before, "partial project survived aborted transaction");
    refused = false; try { await createStoredChat(db, 999); } catch (e) { refused = e.code === "projectStorageFailed"; }
    check(refused && await snapshot() === before, "missing app acquired chat");
    // Legacy installations have no allocation marks: insert-only writes still matter.
    await storageRequest(db, "refs", "readwrite", s => s.put(0, "allocation:apps"));
    await storageRequest(db, "refs", "readwrite", s => s.put(0, "allocation:chats"));
    const beforeImport = await snapshot();
    const importer = importHandler({ prepareProjectImport, core, coreReady: Promise.resolve(), handlers: new Map(), kappa, unb64: x => Uint8Array.from(atob(x), c => c.charCodeAt(0)), all, nextId: async () => 1, nulls: {}, enc: new TextEncoder(), dbReady: Promise.resolve(db) });
    refused = false; try { await importer({ schema: "hologram-forge/export/1", apps: [{ app: { name: "collision" }, files: { "new.txt": "new" }, chats: [] }] }); } catch (e) { refused = e.code === "projectStorageFailed"; }
    check(refused && await snapshot() === beforeImport, "import replaced an existing project");
    const recovery = await createStoredProject(db, await prepare("recovery")); check(recovery.app.id === 6, "failed transaction consumed ID");
    db.close();
    return { concurrentProjects: 5, uniqueChats: 13, closure: "complete", failedWrite: "rolled-back", missingApp: "refused", importCollision: "refused-without-overwrite", recovery: "complete" };
  });
  assert.deepEqual(report, { concurrentProjects: 5, uniqueChats: 13, closure: "complete", failedWrite: "rolled-back", missingApp: "refused", importCollision: "refused-without-overwrite", recovery: "complete" });
  console.log(JSON.stringify(report));
} finally { if (browser) await browser.close(); await new Promise(resolve => server.close(resolve)); }
