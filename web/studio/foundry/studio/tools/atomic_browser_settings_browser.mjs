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
  await page.locator('[role=textbox]').waitFor();
  const race=await page.evaluate(async()=>{
    const call=patch=>window.electron.ipcRenderer.invoke('set-user-settings',patch);
    await call({autoExpandPreviewPanel:true,enableContextCompaction:true});
    await Promise.all([call({autoExpandPreviewPanel:false}),call({enableContextCompaction:false})]);
    return window.electron.ipcRenderer.invoke('get-user-settings');
  });
  assert.equal(race.autoExpandPreviewPanel,false,'first concurrent preference lost');
  assert.equal(race.enableContextCompaction,false,'second concurrent preference lost');
  const peer=await context.newPage();peer.on('pageerror',e=>errors.push(e.message));await peer.goto(base);await peer.locator('[role=textbox]').waitFor();
  const call=(tab,patch)=>tab.evaluate(patch=>window.electron.ipcRenderer.invoke('set-user-settings',patch),patch);
  await call(page,{autoExpandPreviewPanel:true,enableContextCompaction:true});
  await Promise.all([call(page,{autoExpandPreviewPanel:false}),call(peer,{enableContextCompaction:false})]);
  const shared=await peer.evaluate(()=>window.electron.ipcRenderer.invoke('get-user-settings'));assert.equal(shared.autoExpandPreviewPanel,false);assert.equal(shared.enableContextCompaction,false);
  const removal=await page.evaluate(async()=>{
    const inference=await import('./inference.js');
    for(let i=0;i<12;i++){
      await inference.settingsPut({providerSettings:{openrouter:{apiKey:{value:'local-fixture'}}}});
      const patch=window.electron.ipcRenderer.invoke('set-user-settings',{autoExpandPreviewPanel:!!(i%2)});
      await inference.settingsPut({});await patch;
      if((await inference.settingsGet())?.providerSettings?.openrouter)return false;
    }return true;
  });assert.equal(removal,true,'removed credential was resurrected');
  await call(page,{enableAppBlueprint:false});
  const before=await page.evaluate(()=>window.__host.get('settings','user'));
  const late=await page.evaluate(async()=>{
    const put=IDBObjectStore.prototype.put;let injected=false;
    IDBObjectStore.prototype.put=function(value,key){const q=put.call(this,value,key);if(this.name==='settings'&&key==='user'&&!injected){injected=true;q.addEventListener('success',()=>this.transaction.abort(),{once:true});}return q;};
    try{await window.electron.ipcRenderer.invoke('set-user-settings',{enableAppBlueprint:true});return {injected,code:null};}catch(e){return {injected,code:e.code};}finally{IDBObjectStore.prototype.put=put;}
  });assert.deepEqual(late,{injected:true,code:'settingsStorageFailed'});assert.deepEqual(await page.evaluate(()=>window.__host.get('settings','user')),before);
  await peer.reload();await peer.locator('[role=textbox]').waitFor();assert.equal((await peer.evaluate(()=>window.electron.ipcRenderer.invoke('get-user-settings'))).enableAppBlueprint,false);
  assert.deepEqual(errors,[]);console.log(JSON.stringify({sameWindowContention:true,crossWindowContention:true,credentialRemoval:true,lateAbort:true,reload:true,pageExceptions:0}));
}finally{if(browser)await browser.close();await new Promise(resolve=>server.close(resolve));}
