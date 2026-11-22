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
  const context = await browser.newContext({ serviceWorkers: "block" }), page = await context.newPage(), errors = [], sent = [];
  page.on("pageerror", error => errors.push(error.message));
  page.on("console",message=>{if(message.type()==='error'&&!message.text().includes('ERR_FAILED'))console.log('browser diagnostic:',message.text());});
  await context.route("**/*", route => {
    const request=route.request(),url=new URL(request.url());
    return url.origin===origin?route.continue():route.abort();
  });
  await page.goto(base); await page.waitForFunction(() => !!window.__host && !!window.__prism && !!window.electron);
  const call=(tab,channel,input)=>tab.evaluate(({channel,input})=>window.electron.ipcRenderer.invoke(channel,input),{channel,input});
  const get=(tab,store,key)=>tab.evaluate(({store,key})=>window.__host.get(store,key),{store,key});
  const key='foundry:themes:v1';
  const set=async selectedThemeId=>call(page,'set-user-settings',{selectedThemeId});
  const create=(tab,name)=>call(tab,'create-app',{name,initialChatMode:'build'});
  const refuse=async name=>{const result=await page.evaluate(async name=>{try{await window.electron.ipcRenderer.invoke('create-app',{name});return null;}catch(e){return e.code;}},name);assert.equal(result,'projectStorageFailed');};
  // Project storage only: the window journal and sync's change marks (FA-95) are the tab's own bookkeeping, which runs beside every step.
  const snapshot=()=>page.evaluate(async()=>Object.fromEntries(await Promise.all(['apps','chats','files','objects','refs','versions'].map(async s=>[s,(await window.__host.all(s)).filter(v=>v?.profile!=='foundry/windows/1'&&!(typeof v?.at==='number'&&'gone' in v&&'appId' in v))]))));
  await set('default');const builtin=await create(page,'initial-builtin');assert.equal(builtin.app.themeId,'default');
  await set('');const empty=await create(page,'initial-empty');assert.equal(empty.app.themeId,null);
  const theme=await call(page,'create-custom-theme',{name:'Initial',prompt:'INITIAL-THEME'});
  await set('custom:'+theme.id);const custom=await create(page,'initial-custom');assert.equal(custom.app.themeId,'custom:'+theme.id);
  await page.reload();await page.waitForFunction(()=>!!window.__host&&!!window.electron);assert.equal(await call(page,'get-app-theme',{appId:custom.app.id}),'custom:'+theme.id);
  const good=await get(page,'settings',key);
  for(const selectedThemeId of ['custom:999','unknown','custom:01']){
    await set(selectedThemeId);const before=await snapshot();await refuse('refused');assert.deepEqual(await snapshot(),before);
  }
  await set('default');await page.evaluate(key=>window.__host.put('settings',key,{}),key);const before=await snapshot();await refuse('corrupt');assert.deepEqual(await snapshot(),before);await page.evaluate(({key,good})=>window.__host.put('settings',key,good),{key,good});
  const beforeAbort=await snapshot();
  const late=await page.evaluate(async()=>{
    const add=IDBObjectStore.prototype.add;let injected=false;
    IDBObjectStore.prototype.add=function(value,key){const request=add.call(this,value,key);if(this.name==='apps'){injected=true;request.addEventListener('success',()=>this.transaction.abort(),{once:true});}return request;};
    try{await window.electron.ipcRenderer.invoke('create-app',{name:'aborted'});return {injected,refused:false};}catch(e){return {injected,refused:e.code==='projectStorageFailed'};}finally{IDBObjectStore.prototype.add=add;}
  });assert.deepEqual(late,{injected:true,refused:true});assert.deepEqual(await snapshot(),beforeAbort);
  const peer=await context.newPage();await peer.goto(base);await peer.waitForFunction(()=>!!window.__host&&!!window.electron);
  await set('custom:'+theme.id);
  const settings=await get(page,'settings','user');
  const [created]=await Promise.all([create(page,'concurrent-delete'),call(peer,'delete-custom-theme',{id:theme.id})]);
  assert.equal(await call(page,'get-app-theme',{appId:created.app.id}),null);assert.equal(await call(page,'get-app-theme',{appId:custom.app.id}),null);
  assert.deepEqual(await get(page,'settings','user'),{...settings,selectedThemeId:''});
  const after=await create(peer,'after-delete');assert.equal(after.app.themeId,null);
  await set('default');assert.equal(await call(peer,'get-app-theme',{appId:after.app.id}),null);
  assert.equal(sent.length,0);assert.deepEqual(errors,[]);
  console.log(JSON.stringify({profile:'foundry/initial-project-theme-acceptance/1',builtin:true,custom:true,empty:true,invalidRollback:true,lateAbort:true,concurrentDeletion:true,reload:true,pageErrors:errors}));
} finally {await browser?.close();await new Promise(resolve=>server.close(resolve));}
