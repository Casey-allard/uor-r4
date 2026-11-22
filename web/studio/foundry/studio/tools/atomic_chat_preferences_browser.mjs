// Durable guest-admitted delivery through actual host, bridge and IndexedDB.
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import http from "node:http";
import { fileURLToPath } from "node:url";
import { chromium } from "../vendor/prism/node_modules/playwright-core/index.mjs";
import { DEVICE_MODEL, deviceAsked, deviceSeam } from "./device_fixture.mjs";
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
  const context = await browser.newContext({ serviceWorkers: "block" }), page = await context.addInitScript(deviceSeam).then(() => context.newPage()), errors = [];
  page.on("pageerror", error => errors.push(error.message));
  page.on("console",message=>{if(message.type()==='error'&&!message.text().includes('ERR_FAILED'))console.log('browser diagnostic:',message.text());});
  // Foundry answers with its own model only (Prism R4): nothing off the origin is reached.
  await context.route("**/*", route => new URL(route.request().url()).origin===origin?route.continue():route.abort());
  await page.goto(base); await page.waitForFunction(() => !!window.__host && !!window.__prism && !!window.electron);
  const call=(tab,channel,input)=>tab.evaluate(({channel,input})=>window.electron.ipcRenderer.invoke(channel,input),{channel,input});
  const get=(tab,store,key)=>tab.evaluate(({store,key})=>window.__host.get(store,key),{store,key});
  const created=await call(page,'create-app',{name:'preferences',initialChatMode:'build'}),chatId=created.chatId;
  const peer=await context.newPage();await peer.goto(base);await peer.waitForFunction(()=>!!window.__host&&!!window.electron);
  const modelSelection={...DEVICE_MODEL,effortLevel:'medium'};
  await Promise.all([call(page,'update-chat',{chatId,title:'',chatMode:'ask'}),call(peer,'update-chat',{chatId,modelSelection})]);
  const row=await get(page,'chats',chatId);assert.equal(row.title,'');assert.equal(row.chatMode,'ask');assert.deepEqual(row.modelSelection,modelSelection);
  await Promise.all([call(page,'update-chat',{chatId,title:'Concurrent'}),peer.evaluate(async chatId=>{
    const {updateStoredChat}=await import('./project-lifecycle.mjs');const {workspace:open}=await import('./inference.js');
    const db=await open();
    await updateStoredChat(db,chatId,{messages:[{id:1,role:'user',content:'preserved transcript',approvalState:null,commitHash:null,createdAt:new Date().toISOString()}]});
  },chatId)]);
  await page.reload();await page.waitForFunction(()=>!!window.__host&&!!window.electron);
  const reloaded=await get(page,'chats',chatId);assert.equal(reloaded.title,'Concurrent');assert.equal(reloaded.messages[0].content,'preserved transcript');assert.deepEqual(reloaded.modelSelection,modelSelection);
  assert.equal((await call(page,'chat:count-tokens',{chatId,input:'hello'})).contextWindow,3000);
  const late=await page.evaluate(async chatId=>{
    const put=IDBObjectStore.prototype.put;let injected=false;
    IDBObjectStore.prototype.put=function(value,key){const request=put.call(this,value,key);if(this.name==='chats'){injected=true;request.addEventListener('success',()=>this.transaction.abort(),{once:true});}return request;};
    try{await window.electron.ipcRenderer.invoke('update-chat',{chatId,title:'Must not commit'});return {injected,code:null};}catch(e){return {injected,code:e.code};}finally{IDBObjectStore.prototype.put=put;}
  },chatId);assert.deepEqual(late,{injected:true,code:'chatPreferencesStorageFailed'});assert.deepEqual(await get(peer,'chats',chatId),reloaded);
  await call(page,'update-chat',{chatId,chatMode:'build'});
  await page.evaluate(async({chatId,appId})=>{
    const {bindQueuePayload}=await import('./chat-queue-store.mjs'),{serializeImmutableChatTurnPayload}=await import('./chat-contracts.mjs');
    const intent=await bindQueuePayload({schemaVersion:1,intentId:'preferences-request',chatId,appId,invocationRef:{kind:'chat-stream',entityKey:chatId,operationId:'preferences-request'},prompt:'A todo app so that I finish my chores, within 30 days',requestedChatMode:'build'},serializeImmutableChatTurnPayload);
    await window.electron.ipcRenderer.invoke('distributed-machine:dispatch',{protocolVersion:1,machineId:'chat_stream',encodedKey:{chatId},messageId:crypto.randomUUID(),encodedEvent:{type:'SUBMIT',intent,observedStopPolicyVersion:0}});
  },{chatId,appId:created.app.id});
  for(const deadline=Date.now()+60000;!(await get(page,'chats',chatId)).messages.some(m=>m.role==='assistant'&&m.content.includes('goal: i finish my chores'));await new Promise(r=>setTimeout(r,50)))assert.ok(Date.now()<deadline,'the reader answers the turn');
  assert.equal((await deviceAsked(page)).length,1);assert.equal((await get(page,'chats',chatId)).messages.at(-1).model,modelSelection.name);
  await call(peer,'delete-chat',chatId);
  const missing=await page.evaluate(async chatId=>{try{await window.electron.ipcRenderer.invoke('update-chat',{chatId,title:'revived'});}catch(e){return e.code;}},chatId);
  assert.equal(missing,'chatPreferencesMissing');assert.equal(await get(page,'chats',chatId),undefined);
  assert.equal((await deviceAsked(page)).length,1);assert.deepEqual(errors,[]);
  console.log(JSON.stringify({profile:'foundry/atomic-chat-preferences/1',concurrentPreferences:true,concurrentTranscript:true,reload:true,selectedModelMeter:true,selectedModelRequest:true,lateAbort:true,noRevival:true,pageErrors:errors}));
} finally {await browser?.close();await new Promise(resolve=>server.close(resolve));}
