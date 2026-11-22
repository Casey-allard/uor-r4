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
  const context = await browser.newContext({ serviceWorkers: "allow" }), page = await context.newPage(), external = [], errors = [], workers = [];
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
  const start=()=>page.evaluate(id=>{window.portableOutcome=null;window.__portable(id).then(value=>window.portableOutcome={value},error=>window.portableOutcome={code:error.code,message:error.message});},appId);
  const outcome=async()=>{await page.waitForFunction(()=>window.portableOutcome!==null,{},{timeout:45000});const value=await page.evaluate(()=>window.portableOutcome);return value;};
  const dialog=page.getByRole('dialog',{name:'Review portable build'});
  await start();await dialog.waitFor();assert.equal(await page.evaluate(()=>document.activeElement.textContent),"Don't build");assert.ok((await dialog.textContent()).includes('https://esm.sh'));
  await dialog.getByRole('button',{name:"Don't build"}).click();assert.equal((await outcome()).code,'portableBuildPermissionDenied');assert.equal(remoteRequests,0);assert.equal(stylingRequests,0);
  await start();await dialog.getByRole('button',{name:'Build this source'}).click();
  const built=await outcome();assert.ok(built.value?.html,JSON.stringify(built));assert.equal(built.value.moduleClosure.modules.length,1);assert.equal(remoteRequests,1);assert.equal(stylingRequests,1);
  const independent=await context.newPage();await independent.setContent(built.value.html);assert.equal(await independent.locator('#root').textContent(),'Portable accepted');await independent.close();
  assert.equal(await page.evaluate(async()=> (await window.__host.all('refs')).filter(r=>r?.previewConsent?.id.startsWith('portable-')).length),0);
  assert.equal(await page.locator('iframe[src*="/p/portable-"]').count(),0);
  await page.evaluate(id=>window.electron.ipcRenderer.invoke('edit-app-file',{appId:id,filePath:'new-build.txt',content:'require a new source build'}),appId);
  await context.setOffline(true);await start();assert.equal((await outcome()).code,'portableBuildPermissionOffline');await context.setOffline(false);
  await start();await dialog.waitFor();await page.evaluate(id=>window.electron.ipcRenderer.invoke('edit-app-file',{appId:id,filePath:'notes.txt',content:'changed during approval'}),appId);await dialog.getByRole('button',{name:'Build this source'}).click();assert.equal((await outcome()).code,'buildSourceChanged');assert.equal(remoteRequests,1);
  await page.evaluate(id=>window.electron.ipcRenderer.invoke('edit-app-file',{appId:id,filePath:'src/main.tsx',content:'import { title } from "https://esm.sh/foundry-portable-acceptance";const root=document.getElementById("root");root.textContent=title;root.className="unmatched-fixture";'}),appId);
  await start();await dialog.getByRole('button',{name:'Build this source'}).click();assert.equal((await outcome()).code,'portableBuildStyleIncomplete');
  stall=true;await start();await dialog.getByRole('button',{name:'Build this source'}).click();await page.waitForFunction(async()=> (await window.__host.all('refs')).some(r=>r?.previewConsent?.id.startsWith('portable-')));
  await dialog.getByRole('button',{name:'Stop build'}).click();assert.equal((await outcome()).code,'portableBuildPermissionRevoked');await Promise.all(waiting.splice(0).map(release=>release()));
  assert.equal(await page.evaluate(async()=> (await window.__host.all('refs')).filter(r=>r?.previewConsent?.id.startsWith('portable-')).length),0);assert.equal(await page.locator('iframe[src*="/p/portable-"]').count(),0);
  const beforeRevocation=remoteRequests;await start();await dialog.getByRole('button',{name:'Build this source'}).click();for(let i=0;i<200&&remoteRequests===beforeRevocation;i++)await page.waitForTimeout(25);assert.ok(remoteRequests>beforeRevocation,JSON.stringify(await page.evaluate(()=>window.portableOutcome)));
  await page.evaluate(async()=>{const row=(await window.__host.all('refs')).find(r=>r?.previewConsent?.id.startsWith('portable-'));await window.__host.put('refs','capabilities:'+row.previewConsent.id,{serve:false,network:{endpoints:[]}});});assert.equal((await outcome()).code,'portableBuildPermissionRevoked');await Promise.all(waiting.splice(0).map(release=>release()));
  const expired=await page.evaluate(async()=>{const id='portable-expired',at=Date.now();await window.__host.put('refs','capabilities:'+id,{serve:true,network:{endpoints:['https://esm.sh']},previewConsent:{profile:'foundry/preview-consent/1',purpose:'portable-build',id,source:'blake3:'+'a'.repeat(64),grantedAt:at-2000,expires:at-1000}});await (await caches.open('previews')).put(new URL('p/'+id+'/index.html',location.href).pathname,new Response('<script>parent.postMessage("expired-executed","*")<\/script>',{headers:{'content-type':'text/html'}}));const response=await fetch('p/'+id+'/index.html');return {status:response.status};});assert.equal(expired.status,403);
  await page.reload();await page.waitForFunction(()=>!!window.__host);await page.waitForFunction(async()=>await window.__host.get('refs','capabilities:portable-expired')===undefined);
  assert.deepEqual(errors,[]);console.log(JSON.stringify({actualCompiler:true,actualWorker:true,deniedNoNetwork:true,sourceRaceRefused:true,offlineRefused:true,portableOutputExecuted:true,incompleteStylingRefused:true,sourceBoundStyling:true,stopAborted:true,storedRevocation:true,expiredWorkerRefusal:true,reloadCleanup:true,grantsRemoved:true,framesRemoved:true,pageExceptions:0}));
}finally{if(browser)await browser.close();await new Promise(resolve=>server.close(resolve));}
