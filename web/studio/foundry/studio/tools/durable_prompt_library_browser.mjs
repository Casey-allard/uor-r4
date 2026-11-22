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
  const call=(tab,channel,input)=>tab.evaluate(({channel,input})=>window.electron.ipcRenderer.invoke(channel,input),{channel,input});
  const get=(tab,store,key)=>tab.evaluate(({store,key})=>window.__host.get(store,key),{store,key});
  const key='foundry:prompts:v1';
  assert.deepEqual(await call(page,'prompts:list'),[]);
  const input={title:'Review source',content:'Exact reusable instruction\n',description:'Local prompt',slug:'review-source'};
  const made=await call(page,'prompts:create',input);assert.equal(made.id,1);assert.ok(made.createdAt instanceof Date);assert.equal(made.content,input.content);
  const peer=await context.newPage();peer.on('pageerror',error=>errors.push(error.message));await peer.goto(base);await peer.waitForFunction(()=>!!window.__host&&!!window.electron);
  assert.deepEqual(await call(peer,'prompts:list'),[made]);
  const races=await Promise.allSettled([call(page,'prompts:create',{title:'A',content:'A',slug:'same'}),call(peer,'prompts:create',{title:'B',content:'B',slug:'same'})]);assert.equal(races.filter(x=>x.status==='fulfilled').length,1);assert.equal(races.filter(x=>x.status==='rejected').length,1);
  const two=await call(page,'prompts:list');assert.equal(two.length,2);assert.equal(two[1].id,2);
  await call(peer,'prompts:update',{id:1,title:'Changed',slug:null});const changed=(await call(page,'prompts:list'))[0];assert.equal(changed.title,'Changed');assert.equal(changed.content,input.content);assert.equal(changed.slug,null);
  await peer.reload();await peer.waitForFunction(()=>!!window.__host&&!!window.electron);assert.deepEqual((await call(peer,'prompts:list'))[0],changed);
  const before=await get(page,'settings',key);
  const late=await page.evaluate(async()=>{const put=IDBObjectStore.prototype.put;let injected=false;IDBObjectStore.prototype.put=function(value,key){const q=put.call(this,value,key);if(this.name==='settings'&&key==='foundry:prompts:v1'){injected=true;q.addEventListener('success',()=>this.transaction.abort(),{once:true});}return q;};try{await window.electron.ipcRenderer.invoke('prompts:create',{title:'abort',content:'must not commit'});return {injected,refused:false};}catch(e){return {injected,refused:e.code==='promptLibraryStorageFailed'};}finally{IDBObjectStore.prototype.put=put;}});assert.deepEqual(late,{injected:true,refused:true});assert.deepEqual(await get(page,'settings',key),before);
  const bad=structuredClone(before);bad.records[0].createdAt='invalid';await page.evaluate(({key,bad})=>window.__host.put('settings',key,bad),{key,bad});
  let corrupt=false;try{await call(peer,'prompts:list');}catch(e){corrupt=e.message.includes('prompts:list');}assert.equal(corrupt,true);
  assert.deepEqual(await get(page,'settings',key),bad);await page.evaluate(({key,before})=>window.__host.put('settings',key,before),{key,before});
  await call(page,'prompts:delete',1);assert.equal((await call(peer,'prompts:list')).length,1);const next=await call(peer,'prompts:create',{title:'Next',content:'Next'});assert.equal(next.id,3);
  const invalid=await page.evaluate(async()=>{try{await window.electron.ipcRenderer.invoke('prompts:create',{title:'',content:'private'});return null;}catch(e){return {code:e.code,message:e.message};}});assert.equal(invalid.code,'invalidPromptCommand');assert.ok(!invalid.message.includes('private'));
  assert.deepEqual(errors,[]);console.log(JSON.stringify({actualHost:true,importedContracts:true,concurrentSlugConflict:true,monotonicIds:true,reload:true,exactDates:true,lateAbort:true,corruptionRefused:true,delete:true,pageExceptions:0}));
}finally{if(browser)await browser.close();await new Promise(resolve=>server.close(resolve));}
