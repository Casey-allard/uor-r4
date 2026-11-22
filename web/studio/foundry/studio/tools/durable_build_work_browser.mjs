// Actual locked browser compiler under the verified shell worker, offline.
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
  const context = await browser.newContext({ serviceWorkers: "allow" }), errors = [];let page = await context.newPage();const workers=[];
  page.on("pageerror", error => errors.push(error.message));
  page.on("worker", worker => workers.push(worker));

  let remoteRequests=0,stylingRequests=0,stall=false;const waiting=[];
  const moduleText='export const title = "Portable accepted";';
  await context.route('**/*',route=>{
    const url=new URL(route.request().url());if(url.origin===origin)return route.continue();
    if(url.origin==='https://esm.sh'){remoteRequests++;if(stall)return new Promise(resolve=>waiting.push(()=>route.abort().catch(()=>{}).finally(resolve)));return route.fulfill({status:200,contentType:'text/javascript',headers:{'access-control-allow-origin':'*'},body:moduleText});}
    if(url.origin==='https://cdn.tailwindcss.com'){stylingRequests++;return route.fulfill({status:200,contentType:'text/javascript',headers:{'access-control-allow-origin':'*'},body:'const sheet=document.createElement("style");sheet.textContent="*:where(*) {--tw-test:0}";document.head.append(sheet);'});}
    return route.abort();
  });
  await page.goto(base);await page.waitForFunction(()=>!!window.__host&&!!window.__portable&&!!window.electron);
  await page.evaluate(async()=>{await navigator.serviceWorker.register('sw.js',{scope:'./'});await navigator.serviceWorker.ready;});await page.waitForFunction(()=>!!navigator.serviceWorker.controller);
  const appId=await page.evaluate(async()=>{
    const call=(c,i)=>window.electron.ipcRenderer.invoke(c,i),made=await call('create-app',{name:'portable-permission',initialChatMode:'build'}),appId=made.app.id;
    await call('edit-app-file',{appId,filePath:'src/main.tsx',content:'import { title } from "https://esm.sh/foundry-portable-acceptance"; document.getElementById("root").textContent=title;'});return appId;
  });
  const start=()=>page.evaluate(id=>{window.workOutcome=null;window.__portable(id).then(value=>window.workOutcome={value},error=>window.workOutcome={code:error.code});},appId);
  const outcome=async()=>{await page.waitForFunction(()=>window.workOutcome!==null,{},{timeout:45000});return page.evaluate(()=>window.workOutcome);};
  const dialog=()=>page.getByRole('dialog',{name:'Review portable build'});
  const journal=()=>page.evaluate(id=>window.__host.get('refs',`work:${id}`),appId);
  await start();await dialog().getByRole('button',{name:"Don't build"}).click();assert.equal((await outcome()).code,'portableBuildPermissionDenied');
  let state=await journal();assert.equal(state.records[0].status,'interrupted');assert.equal(state.records[0].state.reserved,1);assert.equal(state.records[0].state.spent,0);
  const oldOwner=state.records[0].state.owner;
  await start();await dialog().waitFor();
  const other=await context.newPage();await other.goto(base);await other.waitForFunction(()=>!!window.__portable);
  const busy=await other.evaluate(async id=>{try{await window.__portable(id);return null;}catch(error){return error.code;}},appId);assert.equal(busy,'buildWorkBusy');await other.close();
  await dialog().getByRole('button',{name:'Build this source'}).click();const built=await outcome();assert.ok(built.value?.html,JSON.stringify(built));
  state=await journal();assert.equal(state.records[0].status,'complete');assert.equal(state.records[0].state.spent,1);assert.equal(state.records[0].state.remaining,0);assert.equal(state.records[0].state.checkpoint,1);assert.equal(state.records[0].attempts,2);assert.notEqual(state.records[0].state.owner,oldOwner);
  const requests=remoteRequests;await page.reload();await page.waitForFunction(()=>!!window.__portable);await start();assert.equal((await outcome()).value.html,built.value.html);assert.equal(remoteRequests,requests);assert.equal(await dialog().count(),0);
  await page.evaluate(async id=>{const row=await window.__host.get('refs',`work:${id}`);row.records[0].output.html='tampered';await window.__host.put('refs',`work:${id}`,row);},appId);await start();assert.equal((await outcome()).code,'buildWorkArtifactInvalid');assert.equal(remoteRequests,requests);
  await page.evaluate(async({id,state})=>window.__host.put('refs',`work:${id}`,state),{id:appId,state});
  await page.evaluate(id=>window.electron.ipcRenderer.invoke('edit-app-file',{appId:id,filePath:'new-source.txt',content:'new source'}),appId);
  stall=true;await start();await dialog().getByRole('button',{name:'Build this source'}).click();for(let i=0;i<200&&remoteRequests===requests;i++)await page.waitForTimeout(25);assert.ok(remoteRequests>requests);const interrupted=await journal();assert.equal(interrupted.records[1].state.phase,1);
  await page.close();await Promise.all(waiting.splice(0).map(release=>release()));stall=false;
  page=await context.newPage();page.on('pageerror',error=>errors.push(error.message));await page.goto(base);await page.waitForFunction(()=>!!window.__portable);
  assert.equal((await journal()).records[1].state.phase,1);
  await start();await dialog().getByRole('button',{name:'Build this source'}).click();assert.ok((await outcome()).value?.html);state=await journal();assert.equal(state.records.length,2);assert.equal(state.records[1].attempts,2);assert.equal(state.records[1].state.phase,3);assert.notEqual(state.records[1].state.owner,interrupted.records[1].state.owner);
  const transactionReport=await page.evaluate(async appId=>{
    const {workspace:db}=await import(new URL('inference.js',location.href)),{captureProjectSource}=await import(new URL('project-edit.mjs',location.href)),{buildWorkStore}=await import(new URL('build-work.mjs',location.href));const database=await db(),captured=await captureProjectSource(database,appId),store=buildWorkStore(database,appId,captured),before=await store.read();
    const first=structuredClone(before);first.records[0].attempts++;const second=structuredClone(before);second.records[0].attempts+=2;
    const results=await Promise.allSettled([store.save(before,first),store.save(before,second)]);if(results.filter(r=>r.status==='fulfilled').length!==1||!results.some(r=>r.status==='rejected'&&r.reason.code==='buildWorkConflict'))throw Error('work comparison failed');
    const originalTransaction=database.transaction,prior=await store.read();database.transaction=function(...args){const t=originalTransaction.apply(this,args);if(args[1]==='readwrite'){const originalStore=t.objectStore.bind(t);t.objectStore=name=>{const s=originalStore(name);if(name==='refs'){const put=s.put.bind(s);s.put=(value,key)=>{const q=put(value,key);if(key===`work:${appId}`)queueMicrotask(()=>{try{t.abort();}catch{}});return q;};}return s;};}return t;};
    let code;try{await store.save(prior,first);}catch(error){code=error.code;}finally{database.transaction=originalTransaction;}
    if(code!=='projectEditStorageFailed'||JSON.stringify(await store.read())!==JSON.stringify(prior))throw Error('late abort promoted work');
    await window.electron.ipcRenderer.invoke('delete-app',{appId});if(await window.__host.get('refs',`work:${appId}`)!==undefined)throw Error('deleted project retained work');return {storageConflict:true,lateAbortRollback:true,deletion:true};
  },appId);
  assert.deepEqual(errors,[]);console.log(JSON.stringify({guestReservation:true,denialRetainsWork:true,competingTabRefused:true,completedCheckpoint:true,reloadedReuse:true,corruptionRefused:true,closedTabRecovered:true,freshPermission:true,...transactionReport,pageExceptions:0}));
}finally{if(browser)await browser.close();await new Promise(resolve=>server.close(resolve));}
