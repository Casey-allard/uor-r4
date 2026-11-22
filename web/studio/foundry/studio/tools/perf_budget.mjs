// The Performance gate: the renderer as built must stay inside its budgets, and the budgets are the Forge
// kernel's (FK-12 to FK-15, proved by Lean in app/src): every measure is asked as "budget <journey> <value>"
// of the guest KERNEL_REV pins, loaded through shell/kernel.js exactly as the page loads it, and the kernel
// answers within or over. No budget is written anywhere else.
//
//   node tools/perf_budget.mjs             the JS each route needs, gzip bytes, from the build's manifest
//   node tools/perf_budget.mjs --browser   also the mobile journeys in Chromium (the locked Playwright, as CI
//                                          installs it): cold and warm LCP, INP of three presses, and the
//                                          first streamed token against a provider that answers at once
//
// Run from studio/ after the renderer's build and scripts/stamp_shell.py. The browser part serves shell/
// itself under /foundry/ as Pages does, through one throttled link (562 ms RTT, 1474 kbps: Slow 4G as
// Lighthouse applies it), with the CPU slowed 4x, at 390x844. It refuses a route the build no longer has as a
// chunk of its own, a journey the kernel holds no budget for, and any measure the kernel answers over.
import fs from "node:fs";
import http from "node:http";
import path from "node:path";
import zlib from "node:zlib";
import { fileURLToPath, pathToFileURL } from "node:url";

const studio = path.join(path.dirname(fileURLToPath(import.meta.url)), "..");
const shell = path.join(studio, "shell");

// The kernel, as the page runs it: kernel.js checks the guest against its pin and replays every vector first.
globalThis.self = globalThis;
await import(pathToFileURL(path.join(shell, "kernel.js")).href);
const kernel = await self.ForgeKernel.load(async (p) => new Response(fs.readFileSync(path.join(shell, p))));
// The budget itself, for the message only: the largest value the kernel still answers within.
async function budgetOf(journey) {
  let lo = 0, hi = 1 << 20;
  if ((await kernel.ask(`budget ${journey} 0`)) !== "within") return null;
  while (lo < hi) { const mid = Math.ceil((lo + hi) / 2); if ((await kernel.ask(`budget ${journey} ${mid}`)) === "within") lo = mid; else hi = mid - 1; }
  return lo;
}
const viteManifest = path.join(studio, "vendor", "prism", "dist", ".vite", "manifest.json");
const failures = [];
async function report(journey, name, value, unit) {
  const answer = await kernel.ask(`budget ${journey} ${value}`);
  const limit = await budgetOf(journey);
  if (answer !== "within" && answer !== "over") { failures.push(`${name}: the kernel holds no budget for "${journey}" (it answered ${answer})`); return; }
  if (answer === "over") failures.push(`${name}: ${value} ${unit} is over its budget of ${limit} ${unit} (the kernel's answer)`);
  console.log(`${answer === "within" ? "within" : "OVER  "} ${name.padEnd(34)} ${String(value).padStart(7)} ${unit} (kernel budget ${limit})`);
}

// ---- the JS each route needs: the entry's static closure plus the route chunk's, gzip level 9.
if (!fs.existsSync(viteManifest)) { console.error(`no ${path.relative(studio, viteManifest)}: build the renderer first`); process.exit(2); }
const m = JSON.parse(fs.readFileSync(viteManifest, "utf8"));
// The entry the projected page names; the Prism frame is a second entry, which the shell imports itself.
const entryKey = Object.keys(m).find((k) => m[k].isEntry && m[k].file === "assets/index.js");
const gz = new Map();
const gzipOf = (file) => {
  if (!gz.has(file)) {
    const p = path.join(shell, file);
    if (!fs.existsSync(p)) { failures.push(`${file} is in the build's manifest but not under shell/ (run scripts/stamp_shell.py)`); gz.set(file, 0); }
    else gz.set(file, zlib.gzipSync(fs.readFileSync(p), { level: 9 }).length);
  }
  return gz.get(file);
};
function closure(key, into = new Set()) {
  const c = m[key];
  if (!c || into.has(c.file)) return into;
  into.add(c.file);
  for (const i of c.imports ?? []) closure(i, into);
  return into;
}
const entry = closure(entryKey);
// The kernel's journey name for each route, and the chunk the build names it by.
const routes = { home: "home", chat: "chat", settings: "settings", details: "app-details", apps: "apps", code: "CodeView", publish: "PublishPanel" };
// A route or panel is named by its chunk's name (the page or component file's name); a locale file of the
// same name is not it. "home" is the first load: the entry and what it imports statically.
const chunkNamed = (name) => Object.keys(m).filter((k) => m[k].name === name && m[k].isDynamicEntry && !String(m[k].src ?? "").endsWith(".json"));
for (const [journey, route] of Object.entries(routes)) {
  let files = entry;
  if (route !== "home") {
    const keys = chunkNamed(route);
    if (keys.length !== 1) { failures.push(`${route}: the build has ${keys.length} chunks of that name loaded on demand, not one (is it in the first load?)`); continue; }
    files = closure(keys[0], new Set(entry));
  }
  await report(journey, `JS for ${route}`, Math.round([...files].reduce((a, f) => a + gzipOf(f), 0) / 1024), "KB");
}

// ---- the mobile journeys
if (process.argv.includes("--browser")) {
  process.env.PW_EXPERIMENTAL_SERVICE_WORKER_NETWORK_EVENTS = "1";
  const pw = await import("../vendor/prism/node_modules/playwright-core/index.mjs");
  const { chromium } = pw.default ?? pw;
  const port = await serve();
  const BASE = `http://localhost:${port}/foundry/`;
  // PERF_CHANNEL=chrome runs it in an installed Chrome on a workstation; CI uses the Chromium Playwright installs.
  const browser = await chromium.launch({ headless: true, channel: process.env.PERF_CHANNEL || undefined, executablePath: process.env.CHROME_BIN || undefined, args: ["--no-sandbox"] });
  const median = (xs) => xs.slice().sort((a, b) => a - b)[Math.floor(xs.length / 2)];
  const cold = [], warm = [], inp = [], token = [];
  // On a failed journey, what the page showed is kept (PERF_ARTIFACTS, set by CI), so a timeout says why.
  const keep = async (page, run, error) => {
    const dir = process.env.PERF_ARTIFACTS;
    if (!dir || !page) return;
    fs.mkdirSync(dir, { recursive: true });
    await page.screenshot({ path: path.join(dir, `run-${run}.png`) }).catch(() => {});
    const text = await page.evaluate(() => document.body.innerText).catch(() => "");
    fs.writeFileSync(path.join(dir, `run-${run}.txt`), [error.message, page.url(), text].join("\n\n"));
  };
  for (let run = 0; run < 3; run++) {
    let page = null;
    try {
    const ctx = await browser.newContext({ viewport: { width: 390, height: 844 }, deviceScaleFactor: 3, isMobile: true, hasTouch: true });
    await ctx.addInitScript(() => {
      window.__lcp = 0; window.__inp = 0;
      new PerformanceObserver((l) => { for (const e of l.getEntries()) window.__lcp = e.startTime; }).observe({ type: "largest-contentful-paint", buffered: true });
      new PerformanceObserver((l) => { for (const e of l.getEntries()) if (e.interactionId) window.__inp = Math.max(window.__inp, e.duration); }).observe({ type: "event", buffered: true, durationThreshold: 16 });
    });
    await ctx.route("**/*", route => {
      const url = new URL(route.request().url());
      // Foundry answers with its own model only (Prism R4): nothing off the origin is reached.
      return url.origin === new URL(BASE).origin ? route.continue() : route.abort();
    });
    page = await ctx.newPage();
    const cdp = await ctx.newCDPSession(page);
    await cdp.send("Emulation.setCPUThrottlingRate", { rate: 4 });
    const lcpOf = async () => { await page.waitForSelector("[role=textbox]", { timeout: 120000 }); await page.waitForTimeout(2500); return Math.round(await page.evaluate(() => window.__lcp)); };
    link.on = true;
    await page.goto(BASE, { timeout: 120000 }); cold.push(await lcpOf());
    console.log(JSON.stringify({run,stage:"cold",lcp:cold.at(-1)}));
    // A second visit comes later: the worker has filled the rest of its closure by then (unthrottled here).
    link.on = false;
    await page.waitForFunction(() => !!navigator.serviceWorker.controller, null, {timeout:120000});
    console.log(JSON.stringify({run,stage:"verified-worker-active"}));
    link.on = true;
    await page.reload({ timeout: 120000 }); warm.push(await lcpOf());
    console.log(JSON.stringify({run,stage:"warm",lcp:warm.at(-1)}));
    link.on = false;
    // Three presses: the model picker, the sidebar, a key in the composer.
    await page.evaluate(() => (window.__inp = 0));
    await page.locator("[data-slot=dropdown-menu-trigger]").first().click(); await page.waitForSelector("[role=menu]"); await page.keyboard.press("Escape");
    await page.locator("[data-slot=sidebar-trigger]").first().click(); await page.waitForSelector("[data-mobile=true]"); await page.mouse.click(380, 420);
    await page.waitForTimeout(500);
    await page.click("[role=textbox]"); await page.keyboard.type("A habit tracker");
    await page.waitForTimeout(500);
    inp.push(Math.round(await page.evaluate(() => window.__inp)));
    // Enter, to the reading on screen. Foundry answers with its own reader only (Prism R4): IdeaReader in the
    // kernel guest, so this is the studio's whole work from the key to the first words it shows.
    const t0 = await page.evaluate(() => performance.now());
    await page.keyboard.press("Enter");
    await page.waitForFunction(() => /Open questions/.test(document.body.innerText), null, { timeout: 120000, polling: "raf" });
    token.push(Math.round((await page.evaluate(() => performance.now())) - t0));
    await ctx.close();
    } catch (error) { await keep(page, run, error); throw error; }
  }
  await browser.close();
  console.log("runs", JSON.stringify({ cold, warm, inp, token }));
  // The kernel's journey names: cold and warm LCP, the slowest of the three presses, Enter to the first token.
  await report("cold", "mobile cold LCP", median(cold), "ms");
  await report("warm", "mobile warm LCP", median(warm), "ms");
  await report("inp", "mobile INP", median(inp), "ms");
  await report("token", "Enter to first token", median(token), "ms");
  server.close();
}

if (failures.length) { console.log("\nPerformance gate refused:\n  " + failures.join("\n  ")); process.exit(1); }
console.log(process.argv.includes("--browser") ? "\nPerformance gate: every route and journey is within its budget" : "\nPerformance gate: every route is within its budget (the journeys run with --browser)");

// ---- a stand-in for Pages: the shell under /foundry/, 404.html for any other path, gzip, one throttled link
var server, link;
function serve() {
  link = { on: false, rtt: 562, Bps: (1474 * 1000) / 8, free: 0 };
  const TYPES = { ".html": "text/html; charset=utf-8", ".js": "text/javascript", ".mjs": "text/javascript", ".css": "text/css", ".json": "application/json", ".wasm": "application/wasm", ".svg": "image/svg+xml", ".png": "image/png", ".jpg": "image/jpeg", ".woff2": "font/woff2", ".webmanifest": "application/manifest+json" };
  const wait = (ms) => new Promise((r) => setTimeout(r, ms));
  server = http.createServer(async (req, res) => {
    const url = new URL(req.url, "http://x");
    if (!url.pathname.startsWith("/foundry/")) { res.writeHead(404); return res.end(); }
    const rel = decodeURIComponent(url.pathname.slice("/foundry/".length));
    let body, status = 200, type;
    let file = path.join(shell, rel);
    if (fs.existsSync(file) && fs.statSync(file).isDirectory()) file = path.join(file, "index.html");
    if (!fs.existsSync(file)) { file = path.join(shell, "404.html"); status = 404; }
    body = fs.readFileSync(file); type = TYPES[path.extname(file)] || "application/octet-stream";
    const headers = { "content-type": type, "cache-control": "max-age=600" };
    if (/text|json|javascript|wasm|svg|manifest/.test(type)) { body = zlib.gzipSync(body); headers["content-encoding"] = "gzip"; }
    if (link.on) await wait(link.rtt);
    res.writeHead(status, headers);
    for (let i = 0; i < body.length; i += 8192) {
      const part = body.subarray(i, i + 8192);
      if (link.on) { const now = Date.now(); const start = Math.max(now, link.free); link.free = start + (part.length / link.Bps) * 1000; if (link.free - now > 1) await wait(link.free - now); }
      res.write(part);
    }
    res.end();
  });
  return new Promise((r) => server.listen(0, () => r(server.address().port)));
}
