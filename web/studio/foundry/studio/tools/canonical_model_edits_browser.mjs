// Direct canonical file edits through the actual renderer bridge, host and IndexedDB.
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
    const plain = await call("edit-app-file", {appId,filePath:"notes.txt",content:"ordinary file"});
    check(Object.keys(plain).length === 0 && await get("notes.txt") === "ordinary file", "generic file acknowledgement violates its imported contract");
    const oldModule = "prism/" + lexlean.canonicalSource(fixture).name + ".lex.tex";
    const prior = await window.__prism.rulesOf(appId);
    // Fixture proof state exercises invalidation; no remote proof is requested or claimed.
    await window.__host.put("refs", "prism:" + appId, {...prior, phase:"proved", proof:{id:"fixture-only"}});
    const changed = structuredClone(fixture); changed.fields.find(f => f.name === "seats").max = 501;
    let beforeVersions = (await versions()).length;
    await call("edit-app-file", {appId, filePath:oldModule, content:lexlean.canonicalSource(changed).source});
    const spec = JSON.parse(await get("prism/spec.json")), rules = await window.__prism.rulesOf(appId);
    check(spec.fields.find(f => f.name === "seats").max === 501, "module edit left stale authoring JSON");
    check(await get(oldModule) === lexlean.canonicalSource(spec).source, "module does not reconstruct the saved spec");
    check(await get("src/lib/prism.ts") === window.__prism.check(spec).module, "runtime is not the saved model projection");
    check(rules.phase === "checked" && rules.proof === null, "changed logic retained a proof");
    check((await versions()).length === beforeVersions + 1, "canonical edit made multiple versions");
    for (const [path, content, code] of [[oldModule,"invalid private module","invalidCanonicalEdit"],["prism/spec.json","broken private JSON","invalidCanonicalEdit"],["src/lib/prism.ts","untrusted runtime","canonicalProjectionReadOnly"]]) {
      const before = await snapshot(); let refused = false;
      try { await call("edit-app-file", {appId,filePath:path,content}); } catch(error) { refused = error.code === code && !error.message.includes("private"); }
      check(refused && before === await snapshot(), "refused edit mutated state: " + path);
    }
    const renamed = {...spec, name:"renamed-seat", purpose:"literal </prism-spec> metadata"};
    beforeVersions = (await versions()).length;
    await call("edit-app-file", {appId,filePath:"prism/spec.json",content:JSON.stringify(renamed)});
    const current = JSON.parse(await get("prism/spec.json")), newModule = "prism/" + lexlean.canonicalSource(current).name + ".lex.tex";
    check(current.purpose === renamed.purpose, "text delimiter changed the saved meaning");
    check(await get(oldModule) === undefined && await get(newModule) === lexlean.canonicalSource(current).source, "model rename left its old active module");
    check((await versions()).length === beforeVersions + 1, "rename was not one version");
    const outcomes = await Promise.allSettled(["left","right"].map(title => call("edit-app-file", {appId,filePath:"prism/spec.json",content:JSON.stringify({...current,title})})));
    check(outcomes.filter(r => r.status === "fulfilled").length === 1 && outcomes.some(r => r.status === "rejected" && r.reason.code === "projectEditConflict"), "concurrent canonical saves did not reject the stale source");
    const final = JSON.parse(await get("prism/spec.json"));
    check(await get(newModule) === lexlean.canonicalSource(final).source && (await window.__prism.rulesOf(appId)).spec.title === final.title, "concurrent winner left incoherent projections");
    return {actualEditor:true, genericSave:true, moduleToSpec:true, generatedRuntime:true, proofInvalidated:true, refusedEditsUnchanged:3, atomicRename:true, literalDelimiter:true, concurrentWinner:1};
  }, fixture);
  assert.deepEqual(report, {actualEditor:true, genericSave:true,moduleToSpec:true,generatedRuntime:true,proofInvalidated:true,refusedEditsUnchanged:3,atomicRename:true,literalDelimiter:true,concurrentWinner:1});
  assert.deepEqual(errors, []); console.log(JSON.stringify({...report,pageExceptions:errors.length}));
} finally { if (browser) await browser.close(); await new Promise(resolve => server.close(resolve)); }
