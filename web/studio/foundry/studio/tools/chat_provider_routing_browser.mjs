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
let reply="";
const fixture=name=>{const value=JSON.parse(fs.readFileSync(new URL(`./fixtures/${name==='rsvp'?'live':'specs'}/${name}.json`,import.meta.url),"utf8"));return value.spec??value;};
try {
  browser = await chromium.launch({ executablePath, headless: true, args: ["--no-sandbox"] });
  const context = await browser.newContext({ serviceWorkers: "block" }), errors = [], sent = [];
  await context.addInitScript(deviceSeam); const page = await context.newPage();
  page.on("pageerror", error => errors.push(error.message));
  await context.route("**/*", route => {
    const request=route.request(),url=new URL(request.url());
    if(['https://openrouter.ai/api/v1/chat/completions','https://api.featherless.ai/v1/chat/completions'].includes(url.href)){
      sent.push({url:url.href,body:JSON.parse(request.postData()),authorization:request.headers().authorization});
      return route.fulfill({status:200,contentType:'text/event-stream',body:'data: '+JSON.stringify({id:'route-fixture',choices:[{index:0,delta:{content:'ROUTE ANSWER'},finish_reason:'stop'}]})+'\n\ndata: [DONE]\n\n'});
    }
    return url.origin===origin?route.continue():route.abort();
  });
  await page.goto(base);await page.waitForFunction(()=>!!window.__host&&!!window.__prism&&!!window.electron);
  const call=(channel,input)=>page.evaluate(({channel,input})=>window.electron.ipcRenderer.invoke(channel,input),{channel,input});
  const get=(store,key)=>page.evaluate(({store,key})=>window.__host.get(store,key),{store,key});
  const made=await call('create-app',{name:'provider-routing',initialChatMode:'ask'}),appId=made.app.id;let chatId=made.chatId;
  await call('set-user-settings',{selectedModel:DEVICE_MODEL});
  const submit=id=>page.evaluate(async({id,chatId,appId})=>{
    const {bindQueuePayload}=await import('./chat-queue-store.mjs'),{serializeImmutableChatTurnPayload}=await import('./chat-contracts.mjs');
    const intent=await bindQueuePayload({schemaVersion:1,intentId:id,chatId,appId,invocationRef:{kind:'chat-stream',entityKey:chatId,operationId:id},prompt:'Explain this application',requestedChatMode:'ask'},serializeImmutableChatTurnPayload);
    return window.electron.ipcRenderer.invoke('distributed-machine:dispatch',{protocolVersion:1,machineId:'chat_stream',encodedKey:{chatId},messageId:crypto.randomUUID(),encodedEvent:{type:'SUBMIT',intent,observedStopPolicyVersion:0}});
  },{id,chatId,appId});
  // A chat that still holds a cloud selection is not answered by another engine in its place, and no
  // request is made for it: Prism R4 refuses every route to a model outside Foundry before the turn
  // looks up or sends anything (FA-87). The captured selection is kept as it was chosen.
  // No cloud window is known any more: the meter shows the device model's.
  for(const [provider,name,windowSize] of [['openrouter','openrouter/free',3000],['featherless','zai-org/GLM-5.3-Flash',3000]]){
    chatId=await call('create-chat',appId);
    await call('update-chat',{chatId,modelSelection:{provider,name,effortLevel:'medium'}});
    assert.equal((await call('chat:count-tokens',{chatId,input:'Explain this application'})).contextWindow,windowSize);
    const count=sent.length;await submit(provider);
    const deadline=Date.now()+30000;while(true){const q=await get('refs',`chat-queue:${chatId}`);if(q?.state.paused===1&&!q.state.active&&q.entries.some(e=>e.intent.intentId===provider))break;assert.ok(Date.now()<deadline,'the refused turn must remain durably paused');await new Promise(r=>setTimeout(r,20));}
    const state=await call('distributed-machine:subscribe',{protocolVersion:1,machineId:'chat_stream',encodedKey:{chatId}});assert.equal(state.encodedState.lastCompletion.outcome,'errored');assert.match(state.encodedState.error,/only with its own language and reasoning/);
    assert.equal(sent.length,count,'no request is made for a cloud selection');assert.equal((await deviceAsked(page)).length,0,'the device model does not answer in its place');
    assert.equal(await page.getByRole('dialog',{name:'Review cloud request',exact:true}).count(),0);
    assert.deepEqual((await get('chats',chatId)).modelSelection,{provider,name,effortLevel:'medium'});
  }
  for(const [id,selection,code] of [['unsupported',{provider:'unknown',name:'x',effortLevel:'medium'},'unsupportedChatProvider'],['invalid',{provider:'local',name:'not-bitnet',effortLevel:'medium'},'invalidChatModelSelection']]){
    chatId=await call('create-chat',appId);
    await call('update-chat',{chatId,modelSelection:selection});const before=(await get('chats',chatId)).messages,count=sent.length;
    const refusal=await page.evaluate(async({chatId})=>{try{await window.electron.ipcRenderer.invoke('chat:count-tokens',{chatId,input:'x'});}catch(e){return e.code;}},{chatId});assert.equal(refusal,code);
    await submit(id);const deadline=Date.now()+10000;while(true){const q=await get('refs',`chat-queue:${chatId}`);if(q?.state.paused===1&&!q.state.active&&q.entries.some(e=>e.intent.intentId===id))break;assert.ok(Date.now()<deadline,'failed turn must remain durably paused');await new Promise(r=>setTimeout(r,20));}
    const state=await call('distributed-machine:subscribe',{protocolVersion:1,machineId:'chat_stream',encodedKey:{chatId}});assert.equal(state.encodedState.lastCompletion.outcome,'errored');assert.equal(state.encodedState.error,code==='unsupportedChatProvider'?'The chat provider is unsupported':'The chat model selection is invalid');assert.equal(sent.length,count);assert.deepEqual((await get('chats',chatId)).messages,before);assert.equal(await page.getByRole('dialog',{name:'Review cloud request',exact:true}).count(),0);
  }
  // The reader needs no GPU: a browser without WebGPU reads the idea like any other.
  {
    chatId=await call('create-chat',appId);await call('update-chat',{chatId,modelSelection:{...DEVICE_MODEL,effortLevel:'medium'}});
    const count=sent.length;await submit('reader');
    let state;for(const deadline=Date.now()+30000;;await new Promise(r=>setTimeout(r,20))){state=await call('distributed-machine:subscribe',{protocolVersion:1,machineId:'chat_stream',encodedKey:{chatId}});if(state.encodedState.lastCompletion?.outcome)break;assert.ok(Date.now()<deadline,'the reader turn settles');}
    assert.equal(state.encodedState.lastCompletion.outcome,'completed');assert.deepEqual(await deviceAsked(page),['Explain this application']);assert.equal(sent.length,count);
  }
  assert.deepEqual(errors,[]);console.log(JSON.stringify({profile:'foundry/chat-provider-routing/1',cloudRefusedByR4:true,readerAnswers:true,noRequest:true,noSubstitution:true,captureKept:true,accounting:true,unsupportedRefused:true,invalidRefused:true,noPartialTranscript:true,pageErrors:errors}));
} finally {await browser?.close();await new Promise(resolve=>server.close(resolve));}
