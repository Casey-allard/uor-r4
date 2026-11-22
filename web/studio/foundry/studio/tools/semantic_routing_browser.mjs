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
  await page.goto(base); await page.waitForFunction(() => !!window.__host && !!window.__prism && !!window.electron);
  const corpus=JSON.parse(fs.readFileSync(new URL('../../verification/routing-vectors.json',import.meta.url))),qualification=JSON.parse(fs.readFileSync(new URL('../../verification/routing-qualification.json',import.meta.url)));
  const results=await page.evaluate(async cases=>{
    const answers=[];for(const row of cases)answers.push(await window.electron.ipcRenderer.invoke('forge:route',row.input));return answers;
  },qualification.cases);
  for(let i=0;i<results.length;i++){const answer=results[i],row=qualification.cases[i];assert.equal(answer.ok,row.expected.ok,row.name);assert.equal(answer.code||'accepted',row.expected.code,row.name);assert.deepEqual(answer.proposal?.selected.map(r=>r.key)||[],row.expected.selected,row.name);}
  const maximum=corpus.vectors.find(row=>row.name==='maximumRanking');const ranked=await page.evaluate(input=>window.electron.ipcRenderer.invoke('forge:route',input),maximum.input);assert.equal(ranked.code,'unresolvedAmbiguity');assert.equal(ranked.ranked.length,32);assert.equal(ranked.proposal.ambiguities.length,32);
  const copied=await page.evaluate(async input=>{const pending=window.electron.ipcRenderer.invoke('forge:route',input);input.intent.concepts=[999];return pending;},corpus.vectors[0].input);assert.equal(copied.ok,true);assert.equal(copied.proposal.selected.at(-1).key,11);
  assert.deepEqual(errors,[]);console.log(JSON.stringify({actualHost:true,typedJudgments:results.length,largestRanking:32,snapshotBeforeAwait:true,pageExceptions:0}));
}finally{if(browser)await browser.close();await new Promise(resolve=>server.close(resolve));}
