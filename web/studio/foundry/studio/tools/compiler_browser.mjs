import { isDecorativeImage, isPrismIdRelay } from "./decorative_image_fixture.mjs";
// Actual locked browser compiler under the verified shell worker, offline.
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
  page.on("pageerror", error => errors.push(error.message));
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
  const result = await page.evaluate(async base => {
    const kernel = await window.ForgeKernel.load(path => fetch(base + path));
    for (let i=0;i<80;i++) if (await kernel.ask("drafting check")!=="checked") throw Error("Repeated pure execution changed semantics");
    const { createCompilerLoader } = await import(base + "browser-compiler.mjs");
    let first = true;
    const load = createCompilerLoader(url => { const target = first ? base + "compiler/missing.js" : url; first = false; return import(target); });
    let refusal;
    try { await load(base); } catch (error) { refusal = error.code; }
    const compiler = await load(base);
    const transformed = await compiler.transform("const value: number = 6 * 7; globalThis.__compilerAnswer = value;", { loader: "ts", target: "es2020" });
    new Function(transformed.code)();
    const transformedAnswer = globalThis.__compilerAnswer;
    const bundled = await compiler.build({ stdin: { contents: 'import { answer } from "answer"; globalThis.__compilerAnswer = answer;', loader: "ts", resolveDir: "/" }, bundle: true, write: false, format: "iife", plugins: [{ name: "fixture", setup(build) {
      build.onResolve({ filter: /^answer$/ }, () => ({ path: "answer", namespace: "fixture" }));
      build.onLoad({ filter: /.*/, namespace: "fixture" }, () => ({ contents: "export const answer: number = 7 * 6", loader: "ts" }));
    } }] });
    new Function(bundled.outputFiles[0].text)();
    let invalid = false;
    try { await compiler.transform("const broken: =", { loader: "ts", logLevel: "silent" }); } catch (error) { invalid = error.errors.length > 0; }
    const missing = await fetch("compiler/missing.wasm");
    return { refusal, version: compiler.version, transformedAnswer, bundledAnswer: globalThis.__compilerAnswer, invalid, missing: missing.status, controlled: !!navigator.serviceWorker.controller };
  }, base);
  assert.deepEqual(result, { refusal: "browserCompilerUnavailable", version: "0.25.5", transformedAnswer: 42, bundledAnswer: 42, invalid: true, missing: 404, controlled: true });
  await page.waitForTimeout(1000);
  if (errors.length) console.error(JSON.stringify({ errors, external }));
  assert.deepEqual(errors, []);
  const workerResources = (await Promise.all(workers.filter(worker=>!closedWorkers.has(worker)).map(worker => worker.evaluate(() => performance.getEntriesByType("resource").map(entry => entry.name))))).flat();
  assert.ok(workerResources.some(url => url === base + "compiler/esbuild.wasm"), JSON.stringify(workerResources));
  // The declared decorative background is a separate product request.
  const unexpected = external.filter(url => !isDecorativeImage(url) && !isPrismIdRelay(url));
  assert.deepEqual(unexpected, []);
  console.log(JSON.stringify({ compiler: result.version, offlineTransform: result.transformedAnswer, offlineBundle: result.bundledAnswer, invalidSource: "refused", failedImport: "recovered", wasmWorker: true, repeatedKernelRequests:80, pageExceptions: 0 }));
} finally { if (browser) await browser.close(); await new Promise(resolve => server.close(resolve)); }
