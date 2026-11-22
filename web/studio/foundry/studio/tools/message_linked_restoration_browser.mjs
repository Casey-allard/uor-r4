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
  const report=await page.evaluate(async()=>{
    const call=(channel,input)=>window.electron.ipcRenderer.invoke(channel,input),made=await call('forge:interaction-create',{name:'Message restoration',kind:'garden'}),appId=made.app.id,chatId=made.chatId;
    const original=await window.__host.get('refs',`${appId}:main`);
    await call('edit-app-file',{appId,filePath:'later.txt',content:'later source'});const current=await window.__host.get('refs',`${appId}:main`);
    const chat=await window.__host.get('chats',chatId);chat.messages=[{id:1,role:'user',content:'first',createdAt:'2026-01-01'},{id:2,role:'assistant',content:'first answer',sourceCommitHash:original,commitHash:original,usingFreeAgentModeQuota:true,chatTurnIntentId:'private-live',createdAt:'2026-01-01'},{id:3,role:'user',content:'restore here',createdAt:'2026-01-02'},{id:4,role:'assistant',content:'later answer',sourceCommitHash:original,commitHash:current,createdAt:'2026-01-02'}];chat.referencedApps=[];await window.__host.put('chats',chatId,chat);
    const send=event=>call('distributed-machine:dispatch',{protocolVersion:1,machineId:'version_preview',encodedKey:{appId},messageId:crypto.randomUUID(),encodedEvent:event});
    const acquire=()=>call('version-preview:acquire-window-interest',{appId});
    await acquire();await send({type:'RESTORE_TO_MESSAGE',chatId,messageId:3,restoreCodebase:true,operationId:'fork-source'});
    const after=await window.__host.get('refs',`${appId}:main`),chats=(await window.__host.all('chats')).filter(c=>c.appId===appId);if(chats.length!==2||after===current)throw Error('atomic fork/source missing');
    const fork=chats.find(c=>c.id!==chatId);if(fork.messages.length!==2||fork.messages[1].chatTurnIntentId!==undefined||fork.messages[1].usingFreeAgentModeQuota!==false||fork.initialCommitHash!==after)throw Error('fork metadata wrong');if(JSON.stringify(await window.__host.get('chats',chatId))!==JSON.stringify(chat))throw Error('fork changed original chat');
    const beforeCount=chats.length;await acquire();await send({type:'RESTORE_TO_MESSAGE',chatId,messageId:3,restoreCodebase:true,operationId:'fork-source'});if((await window.__host.all('chats')).filter(c=>c.appId===appId).length!==beforeCount)throw Error('replay forked again');
    // Replay keeps the freshly acquired preview open; close it before next operation.
    await send({type:'CLOSE',operationId:'close-replay'});await acquire();
    const beforeChatOnly=await window.__host.get('refs',`${appId}:main`);await send({type:'RESTORE_TO_MESSAGE',chatId,messageId:3,restoreCodebase:false,operationId:'fork-chat'});if(await window.__host.get('refs',`${appId}:main`)!==beforeChatOnly)throw Error('chat-only fork changed source');
    await acquire();const before=JSON.stringify({chat:await window.__host.get('chats',chatId),ref:await window.__host.get('refs',`${appId}:main`),preview:await window.__host.get('refs',`preview:${appId}`)}),put=IDBObjectStore.prototype.put;let refused=false;
    IDBObjectStore.prototype.put=function(value,key){const q=put.call(this,value,key);if(this.name==='chats'&&key===chatId)q.addEventListener('success',()=>this.transaction.abort(),{once:true});return q;};
    try{await send({type:'RESTORE',versionId:original.slice(7),currentChatMessageId:{chatId,messageId:3},operationId:'late-abort'});}catch{refused=true;}finally{IDBObjectStore.prototype.put=put;}
    if(!refused||JSON.stringify({chat:await window.__host.get('chats',chatId),ref:await window.__host.get('refs',`${appId}:main`),preview:await window.__host.get('refs',`preview:${appId}`)})!==before)throw Error('late abort partially committed');
    await send({type:'RESTORE',versionId:original.slice(7),currentChatMessageId:{chatId,messageId:3},operationId:'truncate'});if((await window.__host.get('chats',chatId)).messages.length!==2)throw Error('message truncation missing');
    return {appId,chatId,createdChats:2,sourceFork:true,chatOnlyFork:true,exactReplay:true,truncate:true,actualLateAbort:true};
  });
  assert.deepEqual(errors,[]);console.log(JSON.stringify({actualHost:true,actualGuest:true,actualCore:true,actualIndexedDB:true,...report,pageExceptions:0}));
}finally{if(browser)await browser.close();await new Promise(resolve=>server.close(resolve));}
