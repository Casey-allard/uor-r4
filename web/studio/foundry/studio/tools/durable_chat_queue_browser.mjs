// Durable guest-admitted delivery through actual host, bridge and IndexedDB.
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import http from "node:http";
import { fileURLToPath } from "node:url";
import { chromium } from "../vendor/prism/node_modules/playwright-core/index.mjs";
import { DEVICE_MODEL, deviceAsked, deviceHold, deviceRelease, deviceSeam } from "./device_fixture.mjs";
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
  // Foundry answers with its own model only (Prism R4): nothing off the origin is reached.
  await context.route("**/*", route => new URL(route.request().url()).origin===origin?route.continue():route.abort());
  await page.goto(base); await page.waitForFunction(() => !!window.__host && !!window.__prism && !!window.electron);
  const made=await page.evaluate(async()=>{
    const call=(channel,input)=>window.electron.ipcRenderer.invoke(channel,input);
    const made=await call('create-app',{name:'durable-queue',initialChatMode:'build'});
    const send=event=>call('distributed-machine:dispatch',{protocolVersion:1,machineId:'chat_stream',encodedKey:{chatId:made.chatId},messageId:crypto.randomUUID(),encodedEvent:event});
    await send({type:'PAUSE_QUEUE',expectedQueueRevision:0,mutationId:'pause'});
    const {bindQueuePayload}=await import('./chat-queue-store.mjs'),{serializeImmutableChatTurnPayload}=await import('./chat-contracts.mjs');
    for(let n=1;n<=3;n++){
      const intent=await bindQueuePayload({schemaVersion:1,intentId:`intent-${n}`,chatId:made.chatId,appId:made.app.id,invocationRef:{kind:'chat-stream',entityKey:made.chatId,operationId:`op-${n}`},prompt:`Queued ${n}`,requestedChatMode:'build'},serializeImmutableChatTurnPayload);
      await send({type:'SUBMIT',intent,observedStopPolicyVersion:0});
    }
    const snapshot=await call('distributed-machine:subscribe',{protocolVersion:1,machineId:'chat_stream',encodedKey:{chatId:made.chatId}});
    if(snapshot.encodedState.queue.length!==3||!snapshot.encodedState.queuePaused)throw Error('durable paused queue missing');
    return {...made,actor:snapshot.actorInstanceId};
  });
  const peer=await context.newPage();peer.on("pageerror",error=>errors.push(error.message));await peer.goto(base);await peer.waitForFunction(()=>!!window.__host&&!!window.electron);
  const first=await peer.evaluate(async made=>window.electron.ipcRenderer.invoke('distributed-machine:subscribe',{protocolVersion:1,machineId:'chat_stream',encodedKey:{chatId:made.chatId}}),made);
  assert.equal(first.encodedState.queue.length,3);assert.equal(first.encodedState.queueRevision,4);
  const edited=await peer.evaluate(async made=>{
    const call=(channel,input)=>window.electron.ipcRenderer.invoke(channel,input),send=event=>call('distributed-machine:dispatch',{protocolVersion:1,machineId:'chat_stream',encodedKey:{chatId:made.chatId},messageId:crypto.randomUUID(),encodedEvent:event});
    await send({type:'EDIT_QUEUE_ENTRY',itemId:'queue-2',prompt:'Edited in peer',expectedQueueRevision:4,mutationId:'edit'});
    await send({type:'REORDER_QUEUE_ENTRY',itemId:'queue-3',toIndex:0,expectedQueueRevision:5,mutationId:'reorder'});
    await send({type:'REMOVE_QUEUE_ENTRY',itemId:'queue-1',expectedQueueRevision:6,mutationId:'remove'});
    let stale=false;try{await send({type:'CLEAR_QUEUE',expectedQueueRevision:4,mutationId:'stale'});}catch{stale=true;}
    if(!stale)throw Error('stale queue mutation acknowledged');
    return await window.__host.get('refs',`chat-queue:${made.chatId}`);
  },made);
  assert.deepEqual(edited.entries.map(e=>e.intent.prompt),['Queued 3','Edited in peer']);assert.equal(edited.state.revision,7);
  await page.reload();await page.waitForFunction(()=>!!window.__host&&!!window.electron);
  const reloaded=await page.evaluate(async made=>window.electron.ipcRenderer.invoke('distributed-machine:subscribe',{protocolVersion:1,machineId:'chat_stream',encodedKey:{chatId:made.chatId}}),made);
  assert.notEqual(reloaded.actorInstanceId,made.actor);assert.deepEqual(reloaded.encodedState.queue.map(e=>e.prompt),['Queued 3','Edited in peer']);assert.equal(reloaded.encodedState.queuePaused,true);
  await page.evaluate(async made=>{
    const call=(channel,input)=>window.electron.ipcRenderer.invoke(channel,input);
    const stale=await call('distributed-machine:dispatch',{protocolVersion:1,machineId:'chat_stream',encodedKey:{chatId:made.chatId},messageId:'old-actor',expectedActorInstanceId:made.actor,encodedEvent:{type:'CLEAR_QUEUE',expectedQueueRevision:7,mutationId:'old'}});
    if(stale.kind!=='rejected'||stale.reason!=='stale-actor')throw Error('obsolete actor changed queue');
    await call('distributed-machine:dispatch',{protocolVersion:1,machineId:'chat_stream',encodedKey:{chatId:made.chatId},messageId:'clear',encodedEvent:{type:'CLEAR_QUEUE',expectedQueueRevision:7,mutationId:'clear'}});
    const j=await window.__host.get('refs',`chat-queue:${made.chatId}`);if(j.entries.length||j.state.revision!==8)throw Error('clear failed');
    await call('delete-chat',made.chatId);if(await window.__host.get('refs',`chat-queue:${made.chatId}`)!==undefined)throw Error('deleted chat retained queue');
  },made);
  // Exercise actual chatTurn and the device model, keeping its answer pending.
  await page.evaluate(async model=>{const inference=await import('./inference.js');await inference.settingsPut({selectedModel:model});},DEVICE_MODEL);
  const tabs=()=>context.pages(),hold=()=>Promise.all(tabs().map(deviceHold)),release=()=>Promise.all(tabs().map(deviceRelease));
  // What the device was asked, per tab: a closed tab keeps its count.
  const asked=new Map(),count=async()=>{for(const tab of tabs())asked.set(tab,(await deviceAsked(tab)).length);return [...asked.values()].reduce((a,b)=>a+b,0);};
  // Either tab may own the execution Web Lock after the broadcast: the owner is the tab whose device is asked.
  const askedTab=async()=>{for(let n=0;n<600;n++){for(const tab of tabs()){const prev=asked.get(tab)??0,now=(await deviceAsked(tab)).length;asked.set(tab,now);if(now>prev)return tab;}await new Promise(r=>setTimeout(r,20));}throw Error('actual chat did not reach the device model');};
  await hold();
  const call=(tab,channel,input)=>tab.evaluate(({channel,input})=>window.electron.ipcRenderer.invoke(channel,input),{channel,input});
  const send=(tab,chatId,event)=>call(tab,'distributed-machine:dispatch',{protocolVersion:1,machineId:'chat_stream',encodedKey:{chatId},messageId:crypto.randomUUID(),encodedEvent:event});
  const submit=async(tab,made,name,stop=0)=>{
    const intent=await tab.evaluate(async({made,name})=>{
      const {bindQueuePayload}=await import('./chat-queue-store.mjs'),{serializeImmutableChatTurnPayload}=await import('./chat-contracts.mjs');
      return bindQueuePayload({schemaVersion:1,intentId:name,chatId:made.chatId,appId:made.app.id,invocationRef:{kind:'chat-stream',entityKey:made.chatId,operationId:name},prompt:`Explain ${name}`,requestedChatMode:'build'},serializeImmutableChatTurnPayload);
    },{made,name});
    await send(tab,made.chatId,{type:'SUBMIT',intent,observedStopPolicyVersion:stop});return intent;
  };
  const journal=(tab,chatId)=>tab.evaluate(chatId=>window.__host.get('refs',`chat-queue:${chatId}`),chatId);
  const waitJournal=async(tab,chatId,condition)=>{for(let n=0;n<500;n++){const j=await journal(tab,chatId);if(j&&condition(j))return j;await new Promise(r=>setTimeout(r,10));}throw Error('queue settlement timed out: '+condition);};
  const active=await call(page,'create-app',{name:'active-queue',initialChatMode:'build'});
  const firstIntent=await submit(page,active,'active-first');
  let owner=await askedTab();assert.equal(await count(),1);
  const other=owner===page?peer:page;
  await submit(other,active,'busy-second');await waitJournal(other,active.chatId,j=>j.entries.length===2&&j.state.active!==0);
  await send(other,active.chatId,{type:'CANCEL',invocationRef:firstIntent.invocationRef,pauseQueue:true,observedStopPolicyVersion:0});
  await waitJournal(other,active.chatId,j=>!j.state.active&&j.state.paused===1&&j.receipts.some(r=>r.intentId==="active-first"&&r.outcome==="cancelled"));
  assert.equal((await journal(other,active.chatId)).entries[0].intent.intentId,'busy-second');assert.equal(await count(),1);
  await release();
  // Request success is not durable success: abort the real IDB transaction after put.
  const before=await journal(other,active.chatId);
  const aborted=await other.evaluate(async({chatId,revision})=>{
    const original=IDBObjectStore.prototype.put;let injected=false;
    IDBObjectStore.prototype.put=function(value,key){const request=original.call(this,value,key);if(this.name==='refs'&&key===`chat-queue:${chatId}`){injected=true;request.addEventListener('success',()=>this.transaction.abort(),{once:true});}return request;};
    try{await window.electron.ipcRenderer.invoke('distributed-machine:dispatch',{protocolVersion:1,machineId:'chat_stream',encodedKey:{chatId},messageId:'late-abort',encodedEvent:{type:'CLEAR_QUEUE',expectedQueueRevision:revision,mutationId:'late-abort'}});return {injected,refused:false};}catch{return {injected,refused:true};}finally{IDBObjectStore.prototype.put=original;}
  },{chatId:active.chatId,revision:before.state.revision});
  assert.deepEqual(aborted,{injected:true,refused:true});assert.deepEqual(await journal(other,active.chatId),before);
  // Close an actual owner while its answer is pending. A fresh tab must recover paused,
  // retaining the prompt without asking it again automatically.
  await hold();await send(other,active.chatId,{type:'RESUME_QUEUE',expectedQueueRevision:before.state.revision,mutationId:'resume',observedStopPolicyVersion:1});
  owner=await askedTab();assert.equal(await count(),2);await waitJournal(owner,active.chatId,j=>j.state.active!==0);
  const survivor=owner===page?peer:page;
  await call(survivor,'distributed-machine:subscribe',{protocolVersion:1,machineId:'chat_stream',encodedKey:{chatId:active.chatId}});
  // Establish interest while the owner still holds its lock. Closing must wake
  // that existing waiter without another command, notification or subscription.
  let interested=false;
  for(let n=0;n<300;n++){
    interested=await survivor.evaluate(async chatId=>(await navigator.locks.query()).pending.some(lock=>lock.name===`foundry:chat-execution:${chatId}`),active.chatId);
    if(interested)break;await new Promise(resolve=>setTimeout(resolve,10));
  }
  assert.equal(interested,true,'surviving subscriber must retain execution-lock interest');
  await owner.close();
  await waitJournal(survivor,active.chatId,j=>!j.state.active&&j.state.paused===1&&j.entries.length===1);
  assert.equal(await count(),2);
  const recovered=await journal(survivor,active.chatId);
  await send(survivor,active.chatId,{type:'RESUME_QUEUE',expectedQueueRevision:recovered.state.revision,mutationId:'resume-recovered',observedStopPolicyVersion:recovered.state.stopPolicy});
  owner=await askedTab();assert.equal(await count(),3);await call(survivor,'delete-chat',active.chatId);
  await release();await new Promise(r=>setTimeout(r,200));
  assert.equal(await journal(survivor,active.chatId),undefined);assert.equal(await count(),3);
  assert.deepEqual(errors,[]);console.log(JSON.stringify({actualHost:true,actualGuest:true,tabs:2,persistedPrompts:3,edit:true,reorder:true,remove:true,staleRevisionRefused:true,reload:true,staleActorRefused:true,clear:true,delete:true,activeBusySubmit:true,crossTabStop:true,ownerCloseRecovery:true,waitingRecovery:true,activeDeletion:true,actualLateAbort:true,pageExceptions:0}));

}catch(error){console.error(error);throw error;}finally{if(browser)await browser.close();await new Promise(resolve=>server.close(resolve));}
