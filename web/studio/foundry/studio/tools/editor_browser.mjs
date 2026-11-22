import { isDecorativeImage, isPrismIdRelay } from "./decorative_image_fixture.mjs";
// Actual verified shell: offline editor, Settings and language worker.
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import http from "node:http";
import { fileURLToPath } from "node:url";
import { chromium } from "../vendor/prism/node_modules/playwright-core/index.mjs";
assert.equal(JSON.parse(fs.readFileSync(new URL("../vendor/prism/node_modules/monaco-editor/package.json", import.meta.url), "utf8")).version, "0.52.2");
const root = fileURLToPath(new URL("../shell/", import.meta.url));
for (const name of ["core.wasm", "assets/index.js", "manifest.json", "sw.js", "provenance.json"]) assert.ok(fs.existsSync(path.join(root, name)), `Build and project the actual shell first: ${name}`);
const types = { ".js": "text/javascript", ".mjs": "text/javascript", ".css": "text/css", ".json": "application/json", ".wasm": "application/wasm", ".svg": "image/svg+xml", ".woff2": "font/woff2", ".html": "text/html" };
const server = http.createServer((req, res) => {
  const url = new URL(req.url, "http://localhost"), relative = decodeURIComponent(url.pathname).replace(/^\/foundry\//, "");
  let file = path.resolve(root, relative);
  if ((file !== path.resolve(root) && !file.startsWith(root)) || relative.split("/").includes("..")) { res.writeHead(404); res.end(); return; }
  if (fs.existsSync(file) && fs.statSync(file).isDirectory()) file = path.join(file, "index.html");
  if (!fs.existsSync(file) && !path.extname(file) && !relative.startsWith("p/")) file = path.join(root, "index.html");
  if (!fs.existsSync(file)) { res.writeHead(404); res.end(); return; }
  res.setHeader("content-type", types[path.extname(file)] || "application/octet-stream"); res.setHeader("cache-control", "no-store"); res.end(fs.readFileSync(file));
});
await new Promise(resolve => server.listen(0, "127.0.0.1", resolve));
const base = `http://127.0.0.1:${server.address().port}/foundry/`, origin = new URL(base).origin;
const executablePath = process.env.CHROME_BIN || ["/usr/bin/chromium", "/usr/bin/google-chrome"].find(p => fs.existsSync(p));
let browser;
try {
  browser = await chromium.launch({ executablePath, headless: true, args: ["--no-sandbox"] });
  const context = await browser.newContext({ serviceWorkers: "allow" }), page = await context.newPage(), external = [], errors = [], workers = [], closedWorkers = new Set();
  page.on("pageerror", error => { errors.push(error.message); console.error("PAGE ERROR", error.stack); });
  page.on("console", message => { if(message.text().startsWith("REJECTION")) console.error(message.text()); });
  await context.addInitScript(() => addEventListener("unhandledrejection", event => { const reason=event.reason; console.error("REJECTION", JSON.stringify(reason, Object.getOwnPropertyNames(reason || {}))); }));
  page.on("worker", worker => {workers.push(worker);worker.on("close",()=>closedWorkers.add(worker));});
  await context.route("**/*", route => {
    const url = new URL(route.request().url()); if (url.origin === origin) return route.continue(); external.push(url.href);
    return route.abort();
  });
  await page.goto(base);
  await page.waitForFunction(() => !!window.__host && !!window.electron);
  await page.evaluate(async () => {
    await navigator.serviceWorker.register("sw.js", { scope: "./" });
    await navigator.serviceWorker.ready;
  });
  await page.waitForFunction(() => !!navigator.serviceWorker.controller);
  await context.setOffline(true);
  await page.goto(base + "settings");
  await page.getByText("Browser telemetry is disabled.", { exact: true }).waitFor();
  assert.equal(await page.getByRole("switch", { name: "Telemetry", exact: true }).count(), 0);
  assert.equal(await page.getByText("Telemetry ID", { exact: true }).count(), 0);
  // Main loads Monaco when the editor is opened, rather than on the settings page.
  const made=await page.evaluate(()=>window.electron.ipcRenderer.invoke("create-app",{name:"offline-editor",initialChatMode:"build"}));
  await page.goto(base+"chat?id="+made.chatId+"&appId="+made.app.id);
  await page.getByRole("button",{name:"Code",exact:true}).click();
  await page.getByText("AI_RULES.md",{exact:true}).click();
  await page.waitForFunction(() => !!window.monaco?.editor);
  const result = await page.evaluate(async () => {
    const monaco = window.monaco;
    const uri = monaco.Uri.parse("inmemory://foundry/offline.json");
    const model = monaco.editor.createModel('{"value": }', "json", uri);
    const node = document.createElement("div");
    node.style.cssText = "position:fixed;inset:0;z-index:99999;background:white";
    document.body.append(node);
    const editor = monaco.editor.create(node, { model, automaticLayout: true });
    const factory = await monaco.languages.json.getWorker();
    const worker = await factory(uri);
    const invalid = await worker.doValidation(uri.toString());
    editor.setValue('{"value": 31}');
    const valid = await worker.doValidation(uri.toString());
    const text = editor.getValue();
    // Keep the mounted editor alive until the browser closes: asynchronous
    // language features are still attached to this model.
    const missing = await fetch("editor/vs/missing-qualified-test.js");
    return { invalid: invalid.length, valid: valid.length, text, missing: missing.status, controlled: !!navigator.serviceWorker.controller };
  });
  assert.ok(result.invalid > 0);
  assert.deepEqual({ ...result, invalid: undefined }, { invalid: undefined, valid: 0, text: '{"value": 31}', missing: 404, controlled: true });
  await page.waitForTimeout(1000);
  if (errors.length) console.error(JSON.stringify({ errors, external }));
  assert.deepEqual(errors, []);
  const workerResources = (await Promise.all(workers.filter(worker=>!closedWorkers.has(worker)).map(worker => worker.evaluate(() => performance.getEntriesByType("resource").map(entry => entry.name))))).flat();
  assert.ok(workerResources.some(url => url.startsWith(base + "editor/vs/") && url.includes("jsonWorker.js")), JSON.stringify(workerResources));
  // The declared decorative background is a separate product request.
  const unexpected = external.filter(url => !isDecorativeImage(url) && !isPrismIdRelay(url));
  assert.deepEqual(unexpected, []);
  console.log(JSON.stringify({ renderer: "actual-built-shell", offlineSettings: true, offlineEditor: true, offlineJsonWorker: true, missingAsset: result.missing, unexpectedExternalRequests: 0, pageExceptions: 0 }));
} finally { if (browser) await browser.close(); await new Promise(resolve => server.close(resolve)); }
