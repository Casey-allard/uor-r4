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
  const first=await page.evaluate(async()=>{
    const call=(channel,input)=>window.electron.ipcRenderer.invoke(channel,input);
    const boot=await call('window-infrastructure:bootstrap',{}),made=await call('forge:interaction-create',{name:'Window coordination',kind:'garden'});
    return {...made,sessionId:boot.windowSessionId};
  });
  const peer=await context.newPage();peer.on('pageerror',e=>errors.push(e.message));
  await peer.addInitScript(id=>sessionStorage.setItem('holo.window.session',id),first.sessionId);
  await peer.goto(base);await peer.waitForFunction(()=>!!window.__host&&!!window.electron);
  const second=await peer.evaluate(()=>window.electron.ipcRenderer.invoke('window-infrastructure:bootstrap',{}));
  assert.notEqual(second.windowSessionId,first.sessionId);
  const tab={tabInstanceId:'10000000-0000-4000-8000-000000000001',chatId:first.chatId};
  await page.evaluate(async({first,tab})=>{
    const call=(c,p)=>window.electron.ipcRenderer.invoke(c,p);
    await call('window-infrastructure:set-visible-entities',[{kind:'app',id:first.app.id},{kind:'chat',id:first.chatId}]);
    await call('window-infrastructure:set-chat-tab-ownership',[tab]);await call('window-infrastructure:set-focused');
  },{first,tab});
  await peer.evaluate(async({tab,chatId})=>{
    const call=(c,p)=>window.electron.ipcRenderer.invoke(c,p);let refused=false;
    try{await call('window-infrastructure:set-chat-tab-ownership',[tab]);}catch{refused=true;}if(!refused)throw Error('duplicate tab acknowledged');
    window.received=[];for(const name of ['distributed-machine:snapshot','chat:response:chunk','window:query-invalidations'])window.electron.ipcRenderer.on(name,payload=>window.received.push({name,payload}));
    await call('window-infrastructure:attach-interest',{kind:'chat-chunk',chatId});
    window.localActor=(await call('distributed-machine:subscribe',{protocolVersion:1,machineId:'chat_stream',encodedKey:{chatId}})).actorInstanceId;
  },{tab,chatId:first.chatId});
  await page.evaluate(async chatId=>{
    const call=(c,p)=>window.electron.ipcRenderer.invoke(c,p);
    await call('distributed-machine:dispatch',{protocolVersion:1,machineId:'chat_stream',encodedKey:{chatId},messageId:'window-pause',encodedEvent:{type:'PAUSE_QUEUE',expectedQueueRevision:0,mutationId:'window-pause'}});
    window.__host.emit('chat:response:chunk',{chatId,messages:[]});
  },first.chatId);
  await peer.waitForFunction(()=>window.received.some(x=>x.name==='chat:response:chunk')&&window.received.some(x=>x.name==='distributed-machine:snapshot'&&x.payload.encodedState.queuePaused));
  await peer.evaluate(async chatId=>{
    const event=window.received.filter(x=>x.name==='distributed-machine:snapshot').at(-1).payload;if(event.actorInstanceId!==window.localActor)throw Error('peer replaced actor identity');
    const result=await window.electron.ipcRenderer.invoke('distributed-machine:dispatch',{protocolVersion:1,machineId:'chat_stream',encodedKey:{chatId},expectedActorInstanceId:event.actorInstanceId,messageId:'peer-pause',encodedEvent:{type:'PAUSE_QUEUE',expectedQueueRevision:event.encodedState.queueRevision,mutationId:'peer-pause'}});if(result.kind!=='applied')throw Error('peer actor cannot dispatch');
  },first.chatId);
  await page.evaluate(async()=>{
    const call=(c,p)=>window.electron.ipcRenderer.invoke(c,p),before=JSON.stringify(await window.__host.get('refs','foundry:windows:v1')),put=IDBObjectStore.prototype.put;let refused=false;
    IDBObjectStore.prototype.put=function(value,key){const q=put.call(this,value,key);if(this.name==='refs'&&key==='foundry:windows:v1')q.addEventListener('success',()=>this.transaction.abort(),{once:true});return q;};
    try{await call('window-infrastructure:set-focused');}catch{refused=true;}finally{IDBObjectStore.prototype.put=put;}
    if(!refused||JSON.stringify(await window.__host.get('refs','foundry:windows:v1'))!==before)throw Error('late abort acknowledged');
  });
  const seen=await peer.evaluate(()=>window.electron.ipcRenderer.invoke('window-infrastructure:bootstrap',{}));
  await page.evaluate(async appId=>window.electron.ipcRenderer.invoke('edit-app-file',{appId,filePath:'window.txt',content:'committed invalidation'}),first.app.id);
  await peer.waitForFunction(()=>window.received.some(x=>x.name==='window:query-invalidations'));
  const recovered=await peer.evaluate(epoch=>window.electron.ipcRenderer.invoke('window-infrastructure:bootstrap',{lastSeenQueryInvalidationEpoch:epoch}),seen.currentQueryInvalidationEpoch);
  assert.ok(recovered.currentQueryInvalidationEpoch>seen.currentQueryInvalidationEpoch);assert.ok(recovered.missedInvalidations.length>0);
  assert.deepEqual(errors,[]);console.log(JSON.stringify({actualHost:true,actualIndexedDB:true,twoWindows:true,duplicateSessionReplaced:true,ownershipConflict:true,localActorPreserved:true,peerDispatch:true,missedInvalidations:true,actualLateAbort:true,pageExceptions:0}));
}finally{if(browser)await browser.close();await new Promise(resolve=>server.close(resolve));}
