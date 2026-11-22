// Foundry answers with its own model only. In a fresh browser with no WebGPU, the first prompt from the home
// box never meets a "connect a provider" dialog, an API-key field or a provider list: the prompt goes on to a
// new app and its chat, and Settings has no model-provider section. The prompt goes to no other origin.
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import http from "node:http";
import { fileURLToPath } from "node:url";
import { chromium } from "../vendor/prism/node_modules/playwright-core/index.mjs";
const root = fileURLToPath(new URL("../shell/", import.meta.url));
for (const name of ["core.wasm", "assets/index.js", "manifest.json", "sw.js", "provenance.json"]) assert.ok(fs.existsSync(path.join(root, name)), `Build and project the actual shell first: ${name}`);
// The words of the remote-provider screens: none may be in the shipped renderer or on screen.
const REMOTE = ["almost ready to build", "use your own API key", "Watch the walkthrough", "Other providers", "OpenRouter", "Model Providers"];
const bundle = fs.readFileSync(path.join(root, "assets/index.js"), "utf8");
for (const words of ["almost ready to build", "use your own API key", "Watch the walkthrough"]) assert.ok(!bundle.includes(words), `the shipped renderer still carries "${words}"`);
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
await new Promise((resolve) => server.listen(0, "127.0.0.1", resolve));
const base = `http://127.0.0.1:${server.address().port}/foundry/`, origin = new URL(base).origin;
const executablePath = process.env.CHROME_BIN || ["/usr/bin/chromium", "/usr/bin/google-chrome"].find((p) => fs.existsSync(p));
let browser;
try {
  browser = await chromium.launch({ executablePath, headless: true, args: ["--no-sandbox"] });
  const context = await browser.newContext({ viewport: { width: 1280, height: 844 } }), page = await context.newPage(), errors = [], offOrigin = [];
  page.on("pageerror", (error) => errors.push(error.message));
  await context.route("**/*", (route) => { const url = new URL(route.request().url()); if (url.origin === origin) return route.continue(); const body = route.request().postData() || ""; if (/greets people/.test(body + url.href)) offOrigin.push(url.href); return route.abort(); });
  // A browser without WebGPU, from the first script on.
  await context.addInitScript(() => { delete Navigator.prototype.gpu; });
  // Every dialog the page ever shows is recorded with its words.
  await context.addInitScript(() => { window.__dialogs = []; new MutationObserver(() => { for (const d of document.querySelectorAll('[role="dialog"], [role="alertdialog"], dialog[open]')) { const text = d.textContent || ""; if (!window.__dialogs.includes(text)) window.__dialogs.push(text); } }).observe(document, { subtree: true, childList: true, attributes: true }); });
  const remoteWords = () => page.evaluate((REMOTE) => { const seen = [document.body.innerText, ...window.__dialogs].join("\n"); return REMOTE.filter((w) => seen.includes(w)); }, REMOTE);
  await page.goto(base); await page.waitForFunction(() => !!window.__host && !!window.electron);
  assert.equal(await page.evaluate(() => "gpu" in navigator), false, "the browser has no WebGPU");
  const composer = page.locator('[data-testid="home-chat-input-container"] [role="textbox"]'); await composer.waitFor();
  const prompt = "A small page that greets people";
  await composer.fill(prompt);
  await page.waitForFunction(() => { const send = document.querySelector('[data-testid="home-chat-input-container"] button[aria-label="Send message"]'); return !send || !send.disabled; });
  await composer.press("Enter");
  // The prompt goes on: an app and its chat open, carrying the prompt.
  await page.waitForURL(/\/chat\?/, { timeout: 30000 });
  await page.getByTestId("messages-list").getByText(prompt).first().waitFor({ timeout: 30000 });
  await page.waitForTimeout(1500);
  assert.deepEqual(await remoteWords(), [], "no provider or API-key screen after Enter");
  // Settings has no provider section and no key field.
  await page.goto(`${base}settings`); await page.waitForFunction(() => document.body.innerText.includes("Settings"));
  await page.waitForTimeout(800);
  assert.deepEqual(await remoteWords(), [], "no provider or API-key screen in Settings");
  assert.ok(!(await page.evaluate(() => /API key/i.test(document.body.innerText))), "no model API-key field in Settings");
  assert.deepEqual(offOrigin, [], "the prompt was sent to no other origin");
  assert.deepEqual(errors, []);
  console.log("no remote provider: a fresh browser without WebGPU sends from the home box with no setup dialog, and Settings has no provider or key screen");
} finally { await browser?.close(); server.close(); }
