import assert from "node:assert/strict";
import fs from "node:fs";
import http from "node:http";
import { createHash } from "node:crypto";
import { chromium } from "../vendor/prism/node_modules/playwright-core/index.mjs";
const sha = bytes => createHash("sha256").update(bytes).digest("hex");
const template = fs.readFileSync(new URL("../shell/sw.template.js", import.meta.url), "utf8");
const verifier = fs.readFileSync(new URL("../shell/shell-cache.js", import.meta.url));
const kernel = fs.readFileSync(new URL("../shell/kernel.js", import.meta.url));
let generation = 1, corrupt = false, cookies = 0;
const fixture = () => {
  const files = { "index.html": `<!doctype html><title>Shell generation ${generation}</title><script src="/foundry/assets/index.js"></script>`, "host.js": "// host", "core.wasm": "fixture", "assets/index.js": `window.rendererGeneration = ${generation};`, "assets/index.css": "body{}", "scaffold/files.json": "[]", "editor/vs/loader.js": "// editor", "editor/package.json": "{}", "compiler/package.json": "{}", "compiler/esm/browser.min.js": "// compiler", "compiler/esbuild.wasm": "fixture", "shell-cache.js": verifier, "kernel.js": kernel };
  const rows = Object.entries(files).sort(([a], [b]) => a < b ? -1 : 1).map(([path, body]) => ({ bytes: Buffer.byteLength(body), path, sha256: sha(body) }));
  const closure = sha(JSON.stringify(rows)); return { files, closure, manifest: { spec: "hologram-forge/shell/1", closure, files: rows } };
};
const server = http.createServer((request, response) => {
  const path = new URL(request.url, "http://localhost").pathname.slice("/foundry/".length), f = fixture();
  response.setHeader("cache-control", "no-store");
  response.setHeader("content-type", path.endsWith(".js") ? "text/javascript" : path.endsWith(".json") ? "application/json" : "text/html");
  if (path === "driver") { response.end("<!doctype html><title>Driver</title>"); return; }
  // Imports of the worker bootstrap itself use browser-owned request credentials.
  // Only the verifier's fetches are part of the omit-credentials claim.
  if (["manifest.json", "provenance.json", "assets/index.js", "assets/index.css", "scaffold/files.json", "host.js", "core.wasm", "index.html"].includes(path) && request.headers.cookie) cookies++;
  if (path === "sw.js") response.end(template.replaceAll("__CLOSURE__", f.closure));
  else if (path === "manifest.json") response.end(JSON.stringify(f.manifest));
  else if (path === "provenance.json") response.end(JSON.stringify({ browser_projection_sha256: f.closure }));
  else if (Object.hasOwn(f.files, path)) response.end(corrupt && path === "assets/index.js" ? "// corrupted!" : f.files[path]);
  else if (!path.includes(".") && !/^(app|ui|p|holo|v1|engine)\//.test(path)) { response.end(f.files["index.html"]); }
  else { response.statusCode = 404; response.end("missing"); }
});
await new Promise(resolve => server.listen(0, "127.0.0.1", resolve));
const base = `http://127.0.0.1:${server.address().port}/foundry/`;
const executablePath = process.env.CHROME_BIN || ["/usr/bin/chromium", "/usr/bin/google-chrome"].find(p => fs.existsSync(p));
let browser;
try {
  browser = await chromium.launch({ executablePath, headless: true, args: ["--no-sandbox"] });
  const context = await browser.newContext(); await context.addCookies([{ name: "private", value: "secret", url: base }]);
  const page = await context.newPage(); await page.goto(base + "driver");
  const install = attempt => page.evaluate(async ({ attempt, base }) => {
    const registration = await navigator.serviceWorker.register(`${base}sw.js?attempt=${attempt}`, { scope: base, updateViaCache: "none" });
    const worker = registration.installing || registration.waiting || registration.active;
    return new Promise((resolve, reject) => {
      const timer = setTimeout(() => reject(Error("worker installation did not settle")), 20000);
      const done = () => { if (["activated", "redundant"].includes(worker.state)) { clearTimeout(timer); resolve(worker.state); } };
      worker.addEventListener("statechange", done); done();
    });
  }, { attempt, base });
  assert.equal(await install(1), "activated"); await page.waitForFunction(() => !!navigator.serviceWorker.controller);
  const read = path => page.evaluate(async path => { const r = await fetch(path); return { status: r.status, text: await r.text() }; }, base + path);
  assert.equal((await read("assets/index.js")).text, "window.rendererGeneration = 1;");
  const coherentNavigation = async expected => {
    for (const route of ["", "chat?id=1", "library/media/", "new/version/route"]) {
      const response = await page.goto(base + route);
      assert.equal(response.fromServiceWorker(), true);
      assert.equal(response.headers()["cache-control"], "no-store");
      assert.equal(await page.title(), `Shell generation ${expected}`);
      assert.equal(await page.evaluate(() => window.rendererGeneration), expected);
      await page.reload();
      assert.equal(await page.title(), `Shell generation ${expected}`);
      assert.equal(await page.evaluate(() => window.rendererGeneration), expected);
    }
  };
  await coherentNavigation(1);
  generation = 2; corrupt = true;
  assert.equal(await install(2), "redundant"); assert.equal((await read("assets/index.js")).text, "window.rendererGeneration = 1;");
  await coherentNavigation(1);
  await context.setOffline(true);
  await coherentNavigation(1);
  assert.equal((await read("assets/index.js")).text, "window.rendererGeneration = 1;"); assert.equal((await read("scaffold/files.json")).text, "[]"); assert.equal((await read("assets/missing.js")).status, 404);
  await context.setOffline(false); corrupt = false;
  assert.equal(await install(3), "activated");
  await page.waitForFunction(() => navigator.serviceWorker.controller?.scriptURL.endsWith("attempt=3"));
  assert.equal((await read("assets/index.js")).text, "window.rendererGeneration = 2;");
  await coherentNavigation(2);
  await context.setOffline(true);
  await coherentNavigation(2);
  const caches = await page.evaluate(() => window.caches.keys()); assert.deepEqual(caches.filter(x => x.startsWith("shell-")), ["shell-" + fixture().closure]);
  assert.equal(cookies, 0);
  console.log(JSON.stringify({ verifiedInstall: "activated", corruptUpdate: "refused", previousVersion: "retained", rootAndRendererRefresh: "coherent online and offline after refusal and recovery", offlineRendererAndScaffold: "exact", missingAsset: 404, recovery: "activated", staleCaches: 0, forwardedCookies: cookies }));
} finally { if (browser) await browser.close(); await new Promise(resolve => server.close(resolve)); }
