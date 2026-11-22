import { isDecorativeImage, isPrismIdRelay } from "./decorative_image_fixture.mjs";
// The actual built renderer and host, including legacy local consent and Settings.
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
  const context = await browser.newContext({ serviceWorkers: "block" }), page = await context.newPage(), external = [], errors = [];
  page.on("pageerror", error => errors.push(error.message));
  await context.route("**/*", route => {
    const url = new URL(route.request().url()); if (url.origin === origin) return route.continue(); external.push(url.href);
    return route.abort();
  });
  await context.addInitScript(() => { if (window === top) { localStorage.setItem("prismTelemetryConsent", "opted_in"); localStorage.setItem("prismTelemetryUserId", "legacy-private-fixture"); } });
  await page.goto(base);
  await page.waitForFunction(() => !!window.__host && !!window.electron);
  await page.goto(base + "settings");
  await page.getByText("Browser telemetry is disabled.", { exact: true }).waitFor();
  assert.equal(await page.getByRole("switch", { name: "Telemetry", exact: true }).count(), 0);
  assert.equal(await page.getByText("Telemetry ID", { exact: true }).count(), 0);
  const result = await page.evaluate(async () => {
    const settings = await window.electron.ipcRenderer.invoke("get-user-settings");
    let code; try { await window.electron.ipcRenderer.invoke("set-user-settings", { telemetryConsent: "opted_in" }); } catch (error) { code = error.code; }
    return { consent: settings.telemetryConsent, identifier: settings.telemetryUserId || null, code };
  });
  assert.deepEqual(result, { consent: "opted_out", identifier: null, code: "browserTelemetryUnavailable" });
  await page.waitForTimeout(1000);
  if (errors.length) console.error(JSON.stringify({ errors, external }));
  assert.deepEqual(errors, []);
  // The declared decorative background is a separate product request.
  const unexpected = external.filter(url => !isDecorativeImage(url) && !isPrismIdRelay(url));
  assert.deepEqual(unexpected, []);
  console.log(JSON.stringify({ renderer: "actual-built-shell", legacyConsent: "refused", settings: "privacy-disclosed", enableControl: "absent", typedEnableRefusal: result.code, unexpectedExternalRequests: 0, pageExceptions: 0 }));
} finally { if (browser) await browser.close(); await new Promise(resolve => server.close(resolve)); }
