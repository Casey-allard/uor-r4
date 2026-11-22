// Real IndexedDB rollback after a successful request, plus committed recovery. Then the visitor's work sealed to
// their Prism ID (FA-93): the shell itself, with the kernel guest it pins, Hologram's cipher and the locked Prism
// frame its build bundles, served from studio/shell.
import assert from "node:assert/strict";
import fs from "node:fs";
import http from "node:http";
import { chromium } from "../vendor/prism/node_modules/playwright-core/index.mjs";
const source = fs.readFileSync(new URL("../shell/storage-request.mjs", import.meta.url));
const shell = new URL("../shell/", import.meta.url);
const types = { js: "text/javascript", mjs: "text/javascript", json: "application/json", wasm: "application/wasm", html: "text/html", css: "text/css", svg: "image/svg+xml", woff2: "font/woff2" };
const server = http.createServer((request, response) => {
  const path = request.url.split("?")[0];
  // /shell/ serves the shell's files; /foundry/ serves it as the site does, the renderer's routes included.
  const file = path.startsWith("/shell/") ? path.slice(7) : path.startsWith("/foundry/") ? path.slice(9) || "index.html" : null;
  if (file !== null && !path.includes("..")) {
    const found = fs.existsSync(new URL(file, shell)) && !file.endsWith("/") ? file : path.startsWith("/foundry/") && !file.includes(".") ? "index.html" : null;
    if (found) { response.setHeader("content-type", types[found.split(".").pop()] ?? (found.endsWith(".html") ? "text/html" : "application/octet-stream")); return response.end(fs.readFileSync(new URL(found, shell))); }
  }
  response.setHeader("content-type", request.url === "/storage.mjs" ? "text/javascript" : "text/html");
  response.end(request.url === "/storage.mjs" ? source : "<!doctype html><title>Storage commit acceptance</title><body></body>");
});
await new Promise(resolve => server.listen(0, "127.0.0.1", resolve));
const executablePath = process.env.CHROME_BIN || ["/usr/bin/chromium", "/usr/bin/google-chrome"].find(p => fs.existsSync(p));
let browser;
try {
  browser = await chromium.launch({ executablePath, headless: true, args: ["--no-sandbox"] });
  const page = await browser.newPage();
  await page.goto(`http://127.0.0.1:${server.address().port}/`);
  const report = await page.evaluate(async () => {
    const { storageRequest } = await import("/storage.mjs");
    const check = (v, message) => { if (!v) throw Error(message); };
    const open = () => new Promise((resolve, reject) => {
      const q = indexedDB.open("commit-boundary", 1);
      q.onupgradeneeded = () => q.result.createObjectStore("files");
      q.onsuccess = () => resolve(q.result); q.onerror = () => reject(q.error);
    });
    let db = await open();
    const write = v => storageRequest(db, "files", "readwrite", s => s.put(v, "file"));
    const read = () => storageRequest(db, "files", "readonly", s => s.get("file"));
    await write("original");
    let requestSucceeded = false, refused = false;
    try {
      await storageRequest(db, "files", "readwrite", store => {
        const q = store.put("uncommitted", "file");
        q.addEventListener("success", () => { requestSucceeded = true; q.transaction.abort(); });
        return q;
      });
    } catch (e) { check(e.code === "storageTransactionFailed", "wrong abort code"); refused = true; }
    check(requestSucceeded && refused, "request success was reported as commit");
    check(await read() === "original", "aborted write survived");
    refused = false;
    try { await storageRequest(db, "files", "readwrite", s => { s.put("partial", "file"); throw Error("secret"); }); }
    catch (e) { check(e.code === "storageTransactionFailed" && !e.message.includes("secret"), "unsafe storage failure"); refused = true; }
    check(refused && await read() === "original", "synchronous failure committed queued work");
    await write("recovered"); db.close(); db = await open();
    check(await read() === "recovered", "committed write lost after reopening");
    check(await storageRequest(db, "files", "readwrite", s => s.delete("file")) === undefined, "delete result changed");
    check(await read() === undefined, "delete did not commit");
    db.close();
    refused = false;
    try { await read(); } catch (e) { refused = e.code === "storageTransactionFailed"; }
    check(refused, "closed database accepted");
    return { afterRequestAbort: "refused-and-rolled-back", synchronousFailure: "rolled-back", recovery: "committed-and-reopened", deletion: "committed", closedDatabase: "refused" };
  });
  assert.deepEqual(report, { afterRequestAbort: "refused-and-rolled-back", synchronousFailure: "rolled-back", recovery: "committed-and-reopened", deletion: "committed", closedDatabase: "refused" });
  console.log(JSON.stringify(report));

  // FA-93. The page loads the kernel as the shell does, then the shell's own inference.js; each import with its
  // own query is a fresh page load's module.
  const sealed = await page.evaluate(async () => {
    const check = (v, message) => { if (!v) throw Error(message); };
    await new Promise((resolve, reject) => { const s = Object.assign(document.createElement("script"), { src: "/shell/kernel.js", onload: resolve, onerror: reject }); document.head.append(s); });
    const kernel = await self.ForgeKernel.load((path) => fetch("/shell/" + path));
    window.__forgeKernel = { ask: (line) => kernel.ask(line) };
    const load = (n) => import(`/shell/inference.js?load=${n}`);
    const told = () => window.__workspaceTold.splice(0);
    const tx = (db, store, mode, work) => new Promise((resolve, reject) => { const t = db.transaction(store, mode); const r = work(t.objectStore(store)); t.oncomplete = () => resolve(r?.result); t.onerror = () => reject(t.error); });
    const plain = await (await load(0)).db();
    const at = new Date("2026-11-16T00:00:00Z");
    const prompts = { profile: "foundry/prompts/1", nextId: 2, records: [{ id: 1, title: "Kept before", description: null, content: "plain", slug: null, createdAt: at, updatedAt: at }] };
    await tx(plain, "settings", "readwrite", (s) => s.put(prompts, "foundry:prompts:v1"));
    await tx(plain, "files", "readwrite", (s) => s.put("kept before", "7:README.md"));
    await tx(plain, "refs", "readwrite", (s) => s.put({ serve: true }, "capabilities:7"));
    // The one move: sealed in place, opening as it was written, told once.
    let ws = await (await load(1)).workspace();
    check(told().join() === "workspaceMoved", "the move was not told once");
    const opened = await tx(ws, "settings", "readonly", (s) => s.get("foundry:prompts:v1"));
    check(opened.records[0].title === "Kept before" && opened.records[0].createdAt instanceof Date && +opened.records[0].createdAt === +at, "the moved record does not open as written");
    check(await tx(ws, "files", "readonly", (s) => s.get("7:README.md")) === "kept before", "the moved file does not open");
    const rawFile = await tx(plain, "files", "readonly", (s) => s.get("7:README.md"));
    check(rawFile.sealedWork instanceof Uint8Array && !new TextDecoder().decode(rawFile.sealedWork).includes("kept before"), "the file is kept in the clear");
    check((await tx(plain, "refs", "readonly", (s) => s.get("capabilities:7"))).serve === true, "the worker's grant was sealed");
    // A record stored afterwards is sealed; altered, copied and planted records are refused and never returned.
    await tx(ws, "files", "readwrite", (s) => s.put("written now", "7:new.md"));
    check((await tx(plain, "files", "readonly", (s) => s.get("7:new.md"))).sealedWork instanceof Uint8Array, "a new record is kept in the clear");
    const altered = new Uint8Array(rawFile.sealedWork); altered[altered.length - 1] ^= 1;
    await tx(plain, "files", "readwrite", (s) => { s.put({ sealedWork: altered }, "7:altered.md"); s.put(rawFile, "7:copied.md"); return s.put("planted", "7:planted.md"); });
    told();
    for (const key of ["7:altered.md", "7:copied.md", "7:planted.md"]) check(await tx(ws, "files", "readonly", (s) => s.get(key)) === undefined, key + " was returned");
    check(told().join() === "recordTampered,recordMisplaced,recordTampered", "a refused record was not told by the kernel's answer");
    check((await tx(ws, "files", "readonly", (s) => s.getAll())).sort().join() === "kept before,written now", "getAll returned a refused record");
    told();
    // A device removed from the Prism ID: its key is another ID's. Everything kept where work belongs is cleared.
    const id = await new Promise((resolve) => { const q = indexedDB.open("prism-id"); q.onsuccess = () => resolve(q.result); });
    const key = await tx(id, "prism", "readonly", (s) => s.get("workspace"));
    await tx(id, "prism", "readwrite", (s) => s.put({ ...key, prism: "sha256:" + "0".repeat(64) }, "workspace"));
    await (await caches.open("previews")).put("/shell/p/7/index.html", new Response("a built app"));
    ws = await (await load(2)).workspace();
    check(told().join() === "workspaceDeviceRemoved", "the removal was not told, or something left behind moved in");
    check((await tx(plain, "files", "readonly", (s) => s.getAllKeys())).length === 0 && (await tx(plain, "settings", "readonly", (s) => s.getAllKeys())).length === 0, "the removed device's work was kept");
    check((await tx(plain, "refs", "readonly", (s) => s.get("capabilities:7"))).serve === true, "the worker's grant was cleared");
    check(!(await caches.has("previews")), "the removed device's built previews were kept");
    await tx(ws, "files", "readwrite", (s) => s.put("fresh", "8:README.md"));
    check(await tx(ws, "files", "readonly", (s) => s.get("8:README.md")) === "fresh", "the fresh workspace does not open");
    return { moved: "sealed-in-place", stored: "sealed", altered: "recordTampered", copied: "recordMisplaced", planted: "recordTampered", removed: "cleared-and-fresh" };
  });
  assert.deepEqual(sealed, { moved: "sealed-in-place", stored: "sealed", altered: "recordTampered", copied: "recordMisplaced", planted: "recordTampered", removed: "cleared-and-fresh" });
  console.log(JSON.stringify(sealed));

  // FA-95. Two devices of one Prism ID, each its own browser, on the whole shell, with the relay in memory
  // (as identity/test.mjs stands it in): B joins A's ID through the Prism ID panel, then an app made on A
  // arrives on B, sealed there too, and A deleting it deletes it on B.
  const base = `http://127.0.0.1:${server.address().port}/foundry/`, relay = new Map(), sockets = new Set();
  const device = async () => {
    const context = await browser.newContext({ serviceWorkers: "block" });
    await context.route("**/*", (route) => {
      const url = new URL(route.request().url()), topic = url.pathname.split("/")[1], headers = { "access-control-allow-origin": "*", "access-control-allow-headers": "*" };
      if (url.origin === new URL(base).origin) return route.continue();
      if (url.origin !== "https://ntfy.sh") return route.abort();
      if (route.request().method() === "OPTIONS") return route.fulfill({ status: 204, headers });
      if (route.request().method() === "POST") {
        const message = route.request().postData();
        relay.set(topic, [...(relay.get(topic) ?? []), message]);
        for (const socket of sockets) if (socket.topic === topic) socket.ws.send(JSON.stringify({ event: "message", message }));
        return route.fulfill({ status: 200, headers, contentType: "application/json", body: "{}" });
      }
      return route.fulfill({ status: 200, headers, contentType: "application/x-ndjson", body: (relay.get(topic) ?? []).map((message) => JSON.stringify({ event: "message", message })).join("\n") });
    });
    await context.routeWebSocket("wss://ntfy.sh/**", (ws) => { const socket = { topic: new URL(ws.url()).pathname.split("/")[1], ws }; sockets.add(socket); ws.onClose(() => sockets.delete(socket)); });
    const page = await context.newPage();
    await page.goto(base);
    await page.waitForFunction(() => !!window.__host && !!window.electron && !!document.querySelector("prism-frame")?.account, null, { timeout: 60000 });
    return page;
  };
  const a = await device(), b = await device();
  const panel = (p) => p.evaluate(() => document.querySelector("prism-frame").shadowRoot.getElementById("account-panel").showPopover());
  await panel(a); await a.locator("#account-link").click();
  await a.locator("#account-offer").waitFor({ timeout: 15000 });
  const uri = await a.locator("#account-offer").textContent();
  await panel(b); await b.locator("#account-join").click();
  await b.locator("#account-uri").fill(uri); await b.locator("#account-join-go").click();
  await b.locator("#account-join-code").waitFor({ timeout: 15000 });
  await a.locator("#account-code").fill(/(\d{6})/.exec(await b.locator("#account-join-code").textContent())[1]);
  await a.locator("#account-accept").click();
  const id = await a.evaluate(() => document.querySelector("prism-frame").account.id);
  await b.waitForFunction((id) => document.querySelector("prism-frame").account?.id === id, id, { timeout: 15000 });
  const made = await a.evaluate(() => window.electron.ipcRenderer.invoke("create-app", { name: "Made on A", initialChatMode: "build" }));
  // Polled from here: a page predicate that returns a promise reads as true at once.
  const holds = (p, name) => p.evaluate(async (name) => (await window.__host.all("apps")).some((x) => x.name === name), name);
  const until = async (what, done) => { for (const end = Date.now() + 90000; !(await done()); await new Promise((r) => setTimeout(r, 1000))) if (Date.now() > end) throw Error(what); };
  await until("the app made on A never arrived on B", () => holds(b, "Made on A"));
  const kept = await b.evaluate(async () => { const db = await (await import(new URL("inference.js", location.href))).db(); return new Promise((done) => { const q = db.transaction("apps").objectStore("apps").getAll(); q.onsuccess = () => done(q.result.length > 0 && q.result.every((v) => v.sealedWork instanceof Uint8Array)); }); });
  assert.equal(kept, true, "the arrived app is kept in the clear on B");
  await a.evaluate((appId) => window.electron.ipcRenderer.invoke("delete-app", { appId }), made.app.id);
  await until("the app A deleted stayed on B", async () => !(await holds(b, "Made on A")));
  console.log(JSON.stringify({ linked: "one-id", arrived: "sealed", deleted: "followed" }));
} finally {
  if (browser) await browser.close();
  await new Promise(resolve => server.close(resolve));
}
