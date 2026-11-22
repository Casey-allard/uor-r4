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
  const fixture=JSON.parse(fs.readFileSync(new URL("../../verification/event-wire-vectors.json",import.meta.url))).vectors[0].input;
  const first=await page.evaluate(async context=>{
    const call=(channel,input)=>window.electron.ipcRenderer.invoke(channel,input);
    const check=(ok,message)=>{if(!ok)throw Error(message);};
    const made=await call("create-app",{name:"event-delivery",initialChatMode:"build"}),appId=made.app.id;
    const configured=await call("forge:event-configure",{appId,context});check(configured.ok&&configured.state.pending.length===0,"configuration delivered its probe");
    const results=await Promise.allSettled([0,1].map(()=>call("forge:event-deliver",{appId,envelope:context.envelope})));
    check(results.filter(r=>r.status==="fulfilled"&&r.value.ok).length===1,"competing writers both consumed sequence");
    check(results.some(r=>r.status==="rejected"&&r.reason.code==="eventDeliveryConflict")||results.some(r=>r.status==="fulfilled"&&r.value.code==="staleSequence"),"competing writer lacked typed refusal");
    return {appId};
  },fixture);
  await page.reload();await page.waitForFunction(()=>!!window.__host&&!!window.electron);
  const report=await page.evaluate(async({appId,context})=>{
    const call=(channel,input)=>window.electron.ipcRenderer.invoke(channel,input),check=(ok,message)=>{if(!ok)throw Error(message);};
    const pending=await call("forge:event-pending",{appId});check(pending.current&&pending.state.nextSequence===3&&pending.state.pending.length===1,"reload lost delivery");
    let code;
    try{await call("forge:event-acknowledge",{appId,receipt:{generation:3,sequence:2,consumer:{key:1,version:1}}});}catch(error){code=error.code;}
    check(code==="eventDeliveryAcknowledgementRefused","wrong consumer acknowledged delivery");
    await call("edit-app-file",{appId,filePath:"notes.txt",content:"source changed"});
    const stale=await call("forge:event-pending",{appId});check(!stale.current&&stale.state.pending.length===1,"source edit lost pending work");
    const next=structuredClone(context.envelope);next.sequence=3;code=null;
    try{await call("forge:event-deliver",{appId,envelope:next});}catch(error){code=error.code;}
    check(code==="eventDeliverySourceChanged","stale configuration admitted new event");
    const receipt={generation:3,sequence:2,consumer:{key:2,version:1}};
    const ack=await call("forge:event-acknowledge",{appId,receipt});check(ack.state.pending.length===0,"ack did not remove head");
    const repeated=await call("forge:event-acknowledge",{appId,receipt});check(JSON.stringify(ack.state)===JSON.stringify(repeated.state),"duplicate acknowledgement changed journal");
    const updated=structuredClone(context);updated.generation=4;updated.envelope.generation=4;
    check((await call("forge:event-configure",{appId,context:updated})).ok,"new generation cannot configure");
    check((await call("forge:event-deliver",{appId,envelope:updated.envelope})).ok,"new generation cannot deliver");
    const {workspace:db}=await import(new URL("inference.js",location.href));
    const {updateEventJournal}=await import(new URL("event-delivery.mjs",location.href));
    const database=await db(),guest=await window.ForgeKernel.load(path=>fetch(new URL(path,location.href)));
    let arrivals=0,release;const barrier=new Promise(resolve=>release=resolve);
    const delayed={dispatch:async bytes=>{if(++arrivals===2)release();await barrier;return guest.dispatch(bytes);}};
    const raceEnvelope=structuredClone(updated.envelope);raceEnvelope.sequence=3;
    const forced=await Promise.allSettled([0,1].map(()=>updateEventJournal(database,delayed,appId,"deliver",raceEnvelope)));
    check(forced.filter(r=>r.status==="fulfilled"&&r.value.ok).length===1&&forced.some(r=>r.status==="rejected"&&r.reason.code==="eventDeliveryConflict"),"forced overlapping journal writes bypassed comparison");
    const beforeAbort=await window.__host.get("refs",`events:${appId}`),originalTransaction=database.transaction;
    database.transaction=function(...args){const transaction=originalTransaction.apply(this,args);if(args[1]==="readwrite"){const getStore=transaction.objectStore.bind(transaction);transaction.objectStore=name=>{const store=getStore(name);if(name==="refs"){const put=store.put.bind(store);store.put=(value,key)=>{const request=put(value,key);if(key===`events:${appId}`)queueMicrotask(()=>{try{transaction.abort();}catch{}});return request;};}return store;};}return transaction;};
    let abortCode;const abortedEnvelope={...raceEnvelope,sequence:4};
    try{await updateEventJournal(database,guest,appId,"deliver",abortedEnvelope);}catch(error){abortCode=error.code;}finally{database.transaction=originalTransaction;}
    check(abortCode==="projectEditStorageFailed"&&JSON.stringify(await window.__host.get("refs",`events:${appId}`))===JSON.stringify(beforeAbort),"aborted storage consumed event sequence");
    await call("delete-app",{appId});check(await window.__host.get("refs",`events:${appId}`)===undefined,"project deletion retained journal");
    return {guestAdmission:true,concurrentWinner:1,reloadPending:true,wrongConsumerRefused:true,staleSourcePreservesPending:true,acknowledged:true,duplicateAck:true,newGeneration:true,forcedConflict:true,storageRollback:true,deletion:true};
  },{appId:first.appId,context:fixture});
  assert.deepEqual(report,{guestAdmission:true,concurrentWinner:1,reloadPending:true,wrongConsumerRefused:true,staleSourcePreservesPending:true,acknowledged:true,duplicateAck:true,newGeneration:true,forcedConflict:true,storageRollback:true,deletion:true});
  assert.deepEqual(errors, []); console.log(JSON.stringify({...report,pageExceptions:errors.length}));
} finally { if (browser) await browser.close(); await new Promise(resolve => server.close(resolve)); }
