// Browser storage qualification; independent preimage fixture scopes this to adapters.
import assert from "node:assert/strict";
import fs from "node:fs";
import http from "node:http";
import { chromium } from "../vendor/prism/node_modules/playwright-core/index.mjs";
const sources = new Map([
  ["/browser-chat-modes.mjs", fs.readFileSync(new URL("../shell/browser-chat-modes.mjs", import.meta.url))],
  ["/chat-mode-catalog.mjs", fs.readFileSync(new URL("../shell/chat-mode-catalog.mjs", import.meta.url))],
  ["/chat-contracts.mjs", fs.readFileSync(new URL("../shell/chat-contracts.mjs", import.meta.url))],

  ["/theme-library.mjs", fs.readFileSync(new URL("../shell/theme-library.mjs", import.meta.url))],
  ["/version-preview-codec.mjs", fs.readFileSync(new URL("../shell/version-preview-codec.mjs", import.meta.url))],["/blake3.mjs", fs.readFileSync(new URL("../../try/vendor/prism-code/src/blake3.mjs", import.meta.url))]]);
for (const name of ["project-lifecycle", "project-edit", "project-creation", "generation-patch", "snapshot-restore", "storage-request"]) sources.set(`/${name}.mjs`, fs.readFileSync(new URL(`../shell/${name}.mjs`, import.meta.url)));
const server = http.createServer((req, res) => { const bytes = sources.get(req.url); res.setHeader("content-type", bytes ? "text/javascript" : "text/html"); res.end(bytes || "<!doctype html><title>Lifecycle acceptance</title>"); });
await new Promise(resolve => server.listen(0, "127.0.0.1", resolve));
let browser;
try {
  browser = await chromium.launch({ executablePath: process.env.CHROME_BIN || ["/usr/bin/chromium", "/usr/bin/google-chrome"].find(fs.existsSync), headless: true, args: ["--no-sandbox"] });
  const page = await browser.newPage(); await page.goto(`http://127.0.0.1:${server.address().port}/`);
  const report = await page.evaluate(async () => {
    const { renameStoredProject, restoreStoredProject, deleteStoredProject, deleteStoredChat, updateStoredChat } = await import("/project-lifecycle.mjs");
    const { prepareNewProject, createStoredProject, createStoredChat } = await import("/project-creation.mjs");
    const { editStoredProject } = await import("/project-edit.mjs");
    const { storageRequest } = await import("/storage-request.mjs"); const { kappa } = await import("/blake3.mjs");
    const check = (ok, why) => { if (!ok) throw Error(why); };
    const stores = ["apps", "chats", "files", "objects", "refs", "versions"];
    const db = await new Promise((resolve, reject) => { const q = indexedDB.open("project-lifecycle", 1); q.onupgradeneeded = () => stores.forEach(s => q.result.createObjectStore(s)); q.onsuccess = () => resolve(q.result); q.onerror = () => reject(q.error); });
    const get = (s, k) => storageRequest(db, s, "readonly", o => o.get(k));
    const put = (s, k, v) => storageRequest(db, s, "readwrite", o => o.put(v, k));
    const snapshot = async () => JSON.stringify(await Promise.all(stores.map(s => storageRequest(db, s, "readonly", o => o.getAll()))));
    const core = { run: ({ project, parent }) => ({ bytes: JSON.stringify({ entries: project.entries.map(e => [e.path, e.kappa, e.bytes]), name: project.label, parent }) }) };
    const boundary = { core, derive: kappa, readObject: k => get("objects", k), admit: async (a, b) => a === b };
    const seed = async () => createStoredProject(db, await prepareNewProject({ name: "Original", files: { "a.txt": "initial", "prism/spec.json": "{}" } }, boundary));
    const p = await seed(), id = p.app.id;
    const renamed = await renameStoredProject(db, { appId: id, appName: "Renamed" }, boundary);
    const renamedSnapshot = JSON.parse(new TextDecoder().decode(await get("objects", renamed)));
    check(renamedSnapshot.parent === p.oid && renamedSnapshot.name === "Renamed" && (await get("apps", id)).name === "Renamed", "rename split name and snapshot");
    const changed = await editStoredProject(db, { appId: id, filePath: "a.txt", content: "changed" }, boundary);
    await put("refs", `prism:${id}`, { proof: "old" });
    await restoreStoredProject(db, id, p.oid, boundary);
    check(await get("refs", `${id}:main`) === p.oid && await get("files", `${id}:a.txt`) === "initial" && (await get("apps", id)).name === "Original", "checkout not exact");
    check((await get("versions", id)).length === 3 && await get("refs", `prism:${id}`) === undefined, "checkout history or proof stale");
    const reverted = await restoreStoredProject(db, id, changed, boundary, true);
    const revertedSnapshot = JSON.parse(new TextDecoder().decode(await get("objects", reverted)));
    check(revertedSnapshot.parent === p.oid && (await get("versions", id)).length === 4 && await get("files", `${id}:a.txt`) === "changed", "revert wrong parent or files");
    const dependencies = JSON.stringify({ profile: "foundry/direct-dependency-requests/1", dependencies: [{ name: "react", version: "19.1.0" }] });
    const depSnapshot = await editStoredProject(db, { appId: id, filePath: "prism/dependencies.json", content: dependencies }, boundary);
    await restoreStoredProject(db, id, depSnapshot, boundary);
    check(JSON.stringify((await get("apps", id)).extraDeps) === '["react@19.1.0"]', "dependency restore not bound");
    await restoreStoredProject(db, id, p.oid, boundary);
    check((await get("apps", id)).extraDeps.length === 0, "dependencies not cleared by absent manifest");
    for (const operation of ["rename", "checkout", "revert", "reference"]) {
      let release, begin, first = true;
      const gate = new Promise(r => { release = r; }), started = new Promise(r => { begin = r; });
      const delayed = { ...boundary, derive: async bytes => { if (first) { first = false; begin(); await gate; } return kappa(bytes); } };
      const pending = operation === "rename" ? renameStoredProject(db, { appId: id, appName: "Late" }, delayed) : restoreStoredProject(db, id, changed, delayed, operation === "revert");
      await started;
      if (operation === "reference") await put("refs", `${id}:main`, "blake3:" + "f".repeat(64));
      else await editStoredProject(db, { appId: id, filePath: "a.txt", content: operation }, boundary);
      const before = await snapshot(); release();
      let code; try { await pending; } catch (e) { code = e.code; }
      check(code === "projectEditConflict" && await snapshot() === before, operation + " overwrote concurrent edit");
    }
    const failing = storeName => ({ transaction(names, mode) {
      const t = db.transaction(names, mode); if (mode !== "readwrite") return t;
      return new Proxy(t, { get(target, key) {
        if (key === "objectStore") return name => {
          const store = target.objectStore(name); if (name !== storeName) return store;
          return new Proxy(store, { get(s, field) { if (field === "delete" || (storeName === "refs" && field === "put")) return () => { throw Error("injected late storage failure"); }; const v = s[field]; return typeof v === "function" ? v.bind(s) : v; } });
        };
        const v = target[key]; return typeof v === "function" ? v.bind(target) : v;
      }, set(target, key, value) { target[key] = value; return true; } });
    } });
    for (const op of [() => renameStoredProject(failing("refs"), { appId: id, appName: "Broken" }, boundary), () => restoreStoredProject(failing("refs"), id, p.oid, boundary), () => deleteStoredProject(failing("versions"), id)]) {
      const before = await snapshot(); let rejected = false; try { await op(); } catch { rejected = true; }
      check(rejected && await snapshot() === before, "failed lifecycle left partial state");
    }
    const other = await seed(), objectsBefore = (await storageRequest(db, "objects", "readonly", s => s.getAllKeys())).length;
    const last = await seed(); await createStoredChat(db, last.app.id); const lastChat = await createStoredChat(db, last.app.id);
    await deleteStoredProject(db, last.app.id);
    check(!await get("apps", last.app.id) && !await get("versions", last.app.id) && !await get("refs", `${last.app.id}:main`) && !await get("files", `${last.app.id}:a.txt`), "delete left project rows");
    check((await storageRequest(db, "chats", "readonly", s => s.getAll())).every(c => c.appId !== last.app.id), "delete orphaned chats");
    check(!!await get("apps", other.app.id) && (await storageRequest(db, "objects", "readonly", s => s.getAllKeys())).length >= objectsBefore, "delete damaged unrelated state");
    check(await updateStoredChat(db, last.chatId, { messages: [{ content: "late" }] }) === false && await get("chats", last.chatId) === undefined, "late chat resurrected deleted app");
    const next = await seed(); check(next.app.id > last.app.id && next.chatId > lastChat, "deleted identifiers reused");
    await deleteStoredChat(db, next.chatId); const newChat = await createStoredChat(db, next.app.id); check(newChat > next.chatId, "chat identity reused");
    let missing; try { await deleteStoredProject(db, last.app.id); } catch (e) { missing = e.code; } check(missing === "projectLifecycleMissing", "missing deletion not typed");
    await put("apps", 100, { name: "Legacy", id: 100 }); await put("chats", 200, { id: 200, appId: 100 });
    await storageRequest(db, "refs", "readwrite", s => s.delete("allocation:apps"));
    await storageRequest(db, "refs", "readwrite", s => s.delete("allocation:chats"));
    await deleteStoredProject(db, 100); const migrated = await seed();
    check(migrated.app.id > 100 && migrated.chatId > 200, "legacy deleted identifiers reused");
    db.close(); return { rename: "atomic", checkout: "exact", revert: "current-parent", races: 4, failures: "rolled-back", dependencies: "snapshot-bound", deletion: "complete", identifiers: "monotonic" };
  });
  assert.deepEqual(report, { rename: "atomic", checkout: "exact", revert: "current-parent", races: 4, failures: "rolled-back", dependencies: "snapshot-bound", deletion: "complete", identifiers: "monotonic" }); console.log(JSON.stringify(report));
} finally { await browser?.close(); await new Promise(r => server.close(r)); }
