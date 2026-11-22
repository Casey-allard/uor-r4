// Durable guest-admitted delivery through actual host, bridge and IndexedDB.
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
  const context = await browser.newContext({ serviceWorkers: "block" }), page = await context.newPage(), errors = [];
  page.on("pageerror", error => errors.push(error.message));
  await context.route("**/*", route => new URL(route.request().url()).origin === origin ? route.continue() : route.abort());

  let stylesheetRequests=0;
  // A fresh context with the worker blocked exposes every actual network load.
  await context.close();
  const fresh=await browser.newContext({serviceWorkers: "block"});
  await fresh.route("**/*",route=>new URL(route.request().url()).origin===origin?route.continue():route.abort());
  const tab=await fresh.newPage();tab.on('pageerror',e=>errors.push(e.message));
  tab.on('request',request=>{if(new URL(request.url()).pathname==='/foundry/assets/index.css')stylesheetRequests++;});
  await tab.goto(base);
  await tab.getByRole('heading',{name:'Build your ideas with Prism',exact:true}).waitFor();
  await tab.locator('[role=textbox]').waitFor();
  // The Home heading uses clamp(34px, 4.2vw, 55px), including both clamp transitions.
  const headlineSizes = [];
  for (const [width, fontSize] of [[390, '34px'], [809, '34px'], [810, '34.02px'], [1280, '53.76px'], [1309, '54.978px'], [1310, '55px'], [1440, '55px']]) {
    await tab.setViewportSize({ width, height: 900 });
    const actual = await tab.locator('[data-slot="home-stage"] h1').evaluate(el=>getComputedStyle(el).fontSize);
    assert.equal(actual, fontSize, `Home heading at ${width}px must follow clamp(34px, 4.2vw, 55px)`);
    headlineSizes.push({ width, fontSize: actual });
  }
  await tab.setViewportSize({ width: 1280, height: 900 });
  assert.equal(await tab.locator('link[rel=stylesheet][href="/foundry/assets/index.css"]').count(),1);
  assert.equal(stylesheetRequests,1,'home must reuse initial renderer stylesheet');
  // Settings is no longer a sidebar row (the gear opens it over the page), so the route is reached the way the
  // renderer moves between screens: a history entry its router follows, with no page load.
  await tab.evaluate(()=>{history.pushState({},'',location.pathname.replace(/\/?$/,'/')+'settings');dispatchEvent(new PopStateEvent('popstate',{state:{}}));});
  await tab.waitForURL('**/settings');
  await tab.locator('#general-settings').waitFor();
  assert.equal(await tab.locator('body').evaluate(el=>getComputedStyle(el).fontFamily.includes('Geist')),true);
  assert.equal(stylesheetRequests,1,'settings must retain the initial stylesheet');
  assert.deepEqual(errors,[]);console.log(JSON.stringify({actualRenderer:true,home:true,headlineSizes,settings:true,stylesheetRequests,pageExceptions:0}));
}finally{if(browser)await browser.close();await new Promise(resolve=>server.close(resolve));}
