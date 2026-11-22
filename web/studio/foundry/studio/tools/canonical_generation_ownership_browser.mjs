// Generic generation ownership through the actual host and IndexedDB.
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
    const patch = extra => ({writes:[], deletes:[], renames:[], deps:[], ...extra});
    let refusals = 0;
    for (const path of ["prism/spec.json", module, "src/lib/prism.ts"]) {
      for (const candidate of [
        patch({writes:[{path:"mixed.txt",content:"must not commit"},{path,content:"changed"}]}),
        patch({deletes:[{path}]}),
        patch({renames:[{from:path,to:"moved.txt"}]}),
        patch({renames:[{from:"src/App.tsx",to:path}]})
      ]) {
        const before = await snapshot(); let refused = false;
        try { await window.__chat.applyPrismTags(appId,candidate,{allowCanonical:true,rulesUpdate:{state:{phase:"proved"}}}); }
        catch(error) { refused = error.code === "canonicalGenerationRefused"; }
        check(refused && before === await snapshot(), "owned batch changed storage: " + path); refusals++;
      }
    }
    const before = (await versions()).length;
    await window.__chat.applyPrismTags(appId,patch({writes:[{path:"notes.txt",content:"ordinary"}]}));
    check(await get("notes.txt") === "ordinary" && (await versions()).length === before + 1, "ordinary generation failed");
    const cancel = new AbortController(); cancel.abort(); let cancelled = false;
    const saved = await snapshot();
    try { await window.__chat.applyPrismTags(appId,patch({writes:[{path:"notes.txt",content:"cancelled"}]}),{signal:cancel.signal}); }
    catch(error) { cancelled = error.code === "generationCancelled"; }
    check(cancelled && saved === await snapshot(), "cancelled generation changed storage");
    const changed = {...fixture,title:"Canonical command remains available"};
    await call("edit-app-file",{appId,filePath:"prism/spec.json",content:JSON.stringify(changed)});
    check(JSON.parse(await get("prism/spec.json")).title === changed.title, "canonical command refused");
    const plain = await call("create-app",{name:"ordinary-generation",initialChatMode:"build"});
    await window.__chat.applyPrismTags(plain.app.id,patch({writes:[{path:"src/lib/prism.ts",content:"export const ordinary = true;"}]}));
    check(await window.__host.get("files",plain.app.id+":src/lib/prism.ts") === "export const ordinary = true;", "nonmodel generation refused");
    return {actualHost:true,refusals,ordinaryAtomic:true,cancelled:true,canonicalEdit:true,nonmodel:true};
  }, fixture);
  assert.deepEqual(report,{actualHost:true,refusals:12,ordinaryAtomic:true,cancelled:true,canonicalEdit:true,nonmodel:true});
  assert.deepEqual(errors, []); console.log(JSON.stringify({...report,pageExceptions:errors.length}));
} finally { if (browser) await browser.close(); await new Promise(resolve => server.close(resolve)); }
