// Canonical source closure on actual imported projects and both build entry points.
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
  const fixture = JSON.parse(fs.readFileSync(new URL("./fixtures/live/rsvp.json", import.meta.url), "utf8"));
  const report = await page.evaluate(async fixture => {
    const check = (value, message) => { if (!value) throw Error(message); };
    const call = (channel, input) => window.electron.ipcRenderer.invoke(channel, input);
    const lexlean = await import(new URL("live/lexlean.mjs", location.href));
    const made = await call("create-app", {name:"canonical-edits", initialChatMode:"build"}), appId = made.app.id;
    await window.__prism.rulesAfterTurn(appId, "<prism-spec>" + JSON.stringify(fixture) + "</prism-spec>");
    const get = path => window.__host.get("files", appId + ":" + path);
    const versions = () => window.__host.get("versions", appId);
    const snapshot = async () => JSON.stringify(await Promise.all([get("prism/spec.json"), window.__host.all("files"), window.__host.get("refs", appId + ":main"), window.__host.get("refs", "prism:" + appId), versions()]));
    const module = "prism/" + lexlean.canonicalSource(fixture).name + ".lex.tex";
    const { workspace: db } = await import(new URL("inference.js",location.href));
    const { captureBuildSource } = await import(new URL("build-source.mjs",location.href));
    const boundary = {derive:window.__host.kappa,readObject:key=>window.__host.get("objects",key),admit:(derived,expected)=>window.__host.core().run({op:"restore",derived,expected}).decision === "Accept"};
    const canonical = await captureBuildSource(await db(),appId,boundary);
    check(canonical.files.some(([path])=>path===module),"canonical source rejected");
    const baseFiles = Object.fromEntries(canonical.files);
    let refused = 0;
    for (const [path,text] of [[module,"altered module"],["src/lib/prism.ts","altered runtime"],["prism/spec.json","invalid JSON"]]) {
      const imported = await window.__host.handlers.get("forge:import")({schema:"hologram-forge/export/1",apps:[{app:{name:"imported-"+refused},files:{...baseFiles,[path]:text}}],objects:{}});
      const id = imported.apps[0].id;
      const importedRef = await window.__host.get("refs",id+":main");
      const before = JSON.stringify(await Promise.all([window.__host.all("refs"),window.__host.all("files"),window.__host.all("versions"),caches.keys()]));
      for (const build of [window.__preview,window.__portable]) {
        let code;
        try { await build(id); } catch(error) { code=error.code; }
        check(code==="canonicalBuildMismatch","inconsistent imported build did not refuse: "+code);refused++;
      }
      const after = JSON.stringify(await Promise.all([window.__host.all("refs"),window.__host.all("files"),window.__host.all("versions"),caches.keys()]));
      check(before===after,"refused build changed storage or grants");
      await call("edit-app-file",{appId:id,filePath:"prism/spec.json",content:JSON.stringify(fixture)});
      await captureBuildSource(await db(),id,boundary);
      await call("checkout-version",{purpose:"preview",appId:id,versionId:importedRef});
      for (const build of [window.__preview,window.__portable]) {
        let code;try { await build(id); } catch(error) { code=error.code; }
        check(code === "canonicalBuildMismatch", "restored inconsistent build accepted");refused++;
      }
    }
    const ordinary = await call("create-app",{name:"ordinary-capture",initialChatMode:"build"});
    await captureBuildSource(await db(),ordinary.app.id,boundary);
    return {restored:3,actualImports:3,buildRefusals:refused,unchanged:true,repaired:3,validCanonical:true,ordinary:true};
  }, fixture);
  assert.deepEqual(report,{restored:3,actualImports:3,buildRefusals:12,unchanged:true,repaired:3,validCanonical:true,ordinary:true});
  assert.deepEqual(errors, []); console.log(JSON.stringify({...report,pageExceptions:errors.length}));
} finally { if (browser) await browser.close(); await new Promise(resolve => server.close(resolve)); }
