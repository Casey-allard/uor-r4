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
  const dispatch=(tab,appId,event)=>call(tab,'distributed-machine:dispatch',{protocolVersion:1,machineId:'version_preview',encodedKey:{appId},messageId:crypto.randomUUID(),encodedEvent:{...event,operationId:crypto.randomUUID()}});
  const made=await call(page,'forge:interaction-create',{name:'Version preview garden',kind:'garden'}),appId=made.app.id;
  const original=await get(page,'refs',`${appId}:main`);
  await call(page,'edit-app-file',{appId,filePath:'history.txt',content:'current source'});
  const current=await get(page,'refs',`${appId}:main`);
  assert.deepEqual(await call(page,'version-preview:acquire-window-interest',{appId}),{acquired:true});
  const peer=await context.newPage();peer.on('pageerror',error=>errors.push(error.message));await peer.goto(base);await peer.waitForFunction(()=>!!window.__host&&!!window.electron);
  assert.deepEqual(await call(peer,'version-preview:acquire-window-interest',{appId}),{acquired:false});
  await dispatch(page,appId,{type:'SELECT_VERSION',versionId:original.slice(7)});
  assert.equal(await get(page,'refs',`${appId}:main`),original);
  let blocked=false;try{await call(peer,'edit-app-file',{appId,filePath:'history.txt',content:'must refuse'});}catch{blocked=true;}assert.equal(blocked,true);
  const active=await get(page,'refs',`preview:${appId}`);assert.equal(active.state.active,1);
  await dispatch(page,appId,{type:'CLOSE'});assert.equal(await get(page,'refs',`${appId}:main`),current);
  const closed=await call(page,'distributed-machine:subscribe',{protocolVersion:1,machineId:'version_preview',encodedKey:{appId}});assert.equal(closed.encodedState.state.type,'closed');assert.ok(closed.revision>0);
  await call(page,'version-preview:acquire-window-interest',{appId});
  const before=await get(page,'refs',`preview:${appId}`);
  const late=await page.evaluate(async({appId,original})=>{
    const put=IDBObjectStore.prototype.put;let injected=false;
    IDBObjectStore.prototype.put=function(value,key){const q=put.call(this,value,key);if(this.name==='refs'&&key===`preview:${appId}`){injected=true;q.addEventListener('success',()=>this.transaction.abort(),{once:true});}return q;};
    try{await window.electron.ipcRenderer.invoke('distributed-machine:dispatch',{protocolVersion:1,machineId:'version_preview',encodedKey:{appId},messageId:'abort',encodedEvent:{type:'SELECT_VERSION',versionId:original.slice(7),operationId:'abort'}});return {injected,refused:false};}catch{return {injected,refused:true};}finally{IDBObjectStore.prototype.put=put;}
  },{appId,original});assert.deepEqual(late,{injected:true,refused:true});assert.deepEqual(await get(page,'refs',`preview:${appId}`),before);assert.equal(await get(page,'refs',`${appId}:main`),current);
  await dispatch(page,appId,{type:'SELECT_VERSION',versionId:original.slice(7)});
  await page.close();
  assert.deepEqual(await call(peer,'version-preview:acquire-window-interest',{appId}),{acquired:true});assert.equal(await get(peer,'refs',`${appId}:main`),current);
  await dispatch(peer,appId,{type:'RESTORE',versionId:original.slice(7),expectedHeadOid:current.slice(7)});
  const restored=await get(peer,'refs',`${appId}:main`);assert.notEqual(restored,original);assert.notEqual(restored,current);
  const snapshot=JSON.parse(new TextDecoder().decode(Uint8Array.from(await peer.evaluate(async oid=>Array.from(await window.__host.get('objects',oid)),restored))));assert.equal(snapshot.parent,current);
  assert.equal(await get(peer,'files',`${appId}:history.txt`),undefined);
  assert.equal((await get(peer,'refs',`preview:${appId}`)).state.active,0);
  // Simulate a separately committed, valid source while the owner previews history.
  // Recovery must preserve those bytes until an explicit accept/checkpoint decision.
  const capture=tab=>tab.evaluate(async appId=>{const {workspace:db}=await import('./inference.js');const {captureProjectSource}=await import('./project-edit.mjs');return captureProjectSource(await db(),appId);},appId);
  const replaceSource=(tab,source)=>tab.evaluate(async({appId,source})=>{const {workspace:db}=await import('./inference.js');const database=await db();await new Promise((resolve,reject)=>{const tx=database.transaction(['files','refs'],'readwrite');tx.oncomplete=resolve;tx.onabort=()=>reject(Error('fixture source transaction aborted'));const files=tx.objectStore('files');files.delete(IDBKeyRange.bound(appId+':',appId+':￿'));for(const [name,text] of source.files)files.put(text,appId+':'+name);tx.objectStore('refs').put(source.reference,appId+':main');});},{appId,source});
  await call(peer,'edit-app-file',{appId,filePath:'recovery.txt',content:'external committed source'});
  const external=await capture(peer);
  await call(peer,'edit-app-file',{appId,filePath:'recovery.txt',content:'owner original source'});
  await call(peer,'version-preview:acquire-window-interest',{appId});
  await dispatch(peer,appId,{type:'SELECT_VERSION',versionId:original.slice(7)});
  await replaceSource(peer,external);
  let sourceChanged=false;try{await dispatch(peer,appId,{type:'CLOSE'});}catch{sourceChanged=true;}assert.equal(sourceChanged,true);assert.equal((await capture(peer)).reference,external.reference);
  let recovery=await call(peer,'distributed-machine:subscribe',{protocolVersion:1,machineId:'version_preview',encodedKey:{appId}});assert.equal(recovery.encodedState.state.type,'recovery-required');
  const externalJournal=await get(peer,'refs',`preview:${appId}`);
  // A current reference alone is not evidence: changed file bytes must refuse acceptance.
  await peer.evaluate(({appId})=>window.__host.put('files',appId+':recovery.txt','corrupt bytes'),{appId});
  let corruptAccept=false;try{await dispatch(peer,appId,{type:'ACCEPT_CURRENT_REPOSITORY'});}catch{corruptAccept=true;}assert.equal(corruptAccept,true);assert.deepEqual(await get(peer,'refs',`preview:${appId}`),externalJournal);
  await replaceSource(peer,external);
  await dispatch(peer,appId,{type:'ACCEPT_CURRENT_REPOSITORY'});assert.equal(await get(peer,'refs',`${appId}:main`),external.reference);assert.equal((await get(peer,'refs',`preview:${appId}`)).state.active,0);
  await call(peer,'version-preview:acquire-window-interest',{appId});await dispatch(peer,appId,{type:'SELECT_VERSION',versionId:original.slice(7)});await replaceSource(peer,external);
  const checkpointBefore=await get(peer,'refs',`preview:${appId}`);
  const abortCheckpoint=await peer.evaluate(async appId=>{const put=IDBObjectStore.prototype.put;let injected=false;IDBObjectStore.prototype.put=function(value,key){const request=put.call(this,value,key);if(this.name==='refs'&&key===`preview:${appId}`){injected=true;request.addEventListener('success',()=>this.transaction.abort(),{once:true});}return request;};try{await window.electron.ipcRenderer.invoke('distributed-machine:dispatch',{protocolVersion:1,machineId:'version_preview',encodedKey:{appId},messageId:'checkpoint-abort',encodedEvent:{type:'CHECKPOINT_AND_ACCEPT_CURRENT_REPOSITORY',operationId:'checkpoint-abort'}});return {injected,refused:false};}catch{return {injected,refused:true};}finally{IDBObjectStore.prototype.put=put;}},appId);assert.deepEqual(abortCheckpoint,{injected:true,refused:true});assert.deepEqual(await get(peer,'refs',`preview:${appId}`),checkpointBefore);assert.equal(await get(peer,'refs',`${appId}:main`),external.reference);
  await dispatch(peer,appId,{type:'CHECKPOINT_AND_ACCEPT_CURRENT_REPOSITORY'});
  const checkpoint=await get(peer,'refs',`${appId}:main`);assert.notEqual(checkpoint,external.reference);const checkpointObject=await get(peer,'objects',checkpoint);const checkpointSnapshot=JSON.parse(new TextDecoder().decode(checkpointObject));assert.equal(checkpointSnapshot.parent,external.reference);assert.deepEqual((await capture(peer)).files,external.files);assert.equal((await get(peer,'refs',`preview:${appId}`)).state.active,0);
  await call(peer,'delete-app',{appId});assert.equal(await get(peer,'refs',`preview:${appId}`),undefined);
  assert.deepEqual(errors,[]);console.log(JSON.stringify({actualHost:true,actualGuest:true,actualCore:true,exclusiveWindows:true,select:true,returnOriginal:true,editGuard:true,lateAbort:true,closedOwnerRecovery:true,restoreChild:true,changedSourceRefused:true,corruptAcceptanceRefused:true,acceptVerifiedCurrent:true,checkpointChild:true,checkpointAbort:true,deletion:true,pageExceptions:0}));
}finally{if(browser)await browser.close();await new Promise(resolve=>server.close(resolve));}
