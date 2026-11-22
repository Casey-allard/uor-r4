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
  browser=await chromium.launch({executablePath,headless:true,args:['--no-sandbox']});
  const context=await browser.newContext({serviceWorkers:'allow'}),page = await context.addInitScript(deviceSeam).then(() => context.newPage()),errors=[];
  page.on('pageerror',e=>errors.push(e.message));
  page.on('console',m=>{if(m.text().includes('Running app')||m.type()==='error')console.log('browser diagnostic:',m.text());});
  await page.addInitScript(()=>{window.__previewPromptCount=0;const show=HTMLDialogElement.prototype.show;HTMLDialogElement.prototype.show=function(){if(this.getAttribute('aria-label')==='Review preview permissions')window.__previewPromptCount++;return show.call(this);};});
  // Foundry answers with its own model only (Prism R4): nothing off the origin is reached.
  await context.route('**/*',route=>new URL(route.request().url()).origin===origin?route.continue():route.abort());
  await page.goto(base);await page.waitForFunction(()=>!!navigator.serviceWorker.controller&&!!window.__host&&!!window.electron);
  const call=(channel,input)=>page.evaluate(({channel,input})=>window.electron.ipcRenderer.invoke(channel,input),{channel,input});
  const get=(store,key)=>page.evaluate(({store,key})=>window.__host.get(store,key),{store,key});
  const made=await call('create-app',{name:'preview-auto-start',initialChatMode:'ask'}),appId=made.app.id,chatId=made.chatId;
  await call('set-user-settings',{selectedModel:DEVICE_MODEL});
  // Hand-written code is built, and its parts come from other sites, so it is asked about (not modally).
  // An app still made of its scaffold is projected and asks nothing (sidebar_tool_access_browser.mjs).
  await call('edit-app-file',{appId,filePath:'src/pages/Index.tsx',content:'export default function Index() { return <h1>Hand written</h1>; }\n'});
  await page.goto(base+'chat?id='+chatId+'&appId='+appId);
  const mode=page.getByRole('combobox',{name:'Chat mode',exact:true}),preview=page.getByRole('dialog',{name:'Review preview permissions',exact:true});
  const state=async()=>(await call('distributed-machine:subscribe',{protocolVersion:1,machineId:'app_run',encodedKey:{appId}})).encodedState;
  const decline=async(expectedCount)=>{try{await preview.waitFor();}catch(e){console.log(JSON.stringify({state:await state(),errors,body:(await page.locator('body').innerText()).slice(-3500)}));throw e;}await preview.getByRole('button',{name:"Don't allow",exact:true}).click();const deadline=Date.now()+10000;while((await state()).phase!=='errored'){assert.equal(await page.evaluate(()=>window.__previewPromptCount),expectedCount,'decline must not authorize another automatic preview');assert.ok(Date.now()<deadline);await new Promise(r=>setTimeout(r,20));}};
  await decline(1);assert.equal(await page.evaluate(()=>window.__previewPromptCount),1,'decline must not authorize another automatic preview');
  const remount=async expected=>{for(const width of [390,768,1280,390,1280]){await page.setViewportSize({width,height:844});await mode.waitFor();await page.evaluate(()=>new Promise(r=>requestAnimationFrame(()=>requestAnimationFrame(r))));await page.getByTestId('model-picker').hover();assert.equal(await preview.count(),0,'remount must not reopen a declined preview');assert.equal(await page.evaluate(()=>window.__previewPromptCount),expected);assert.equal((await state()).phase,'errored');}};
  await remount(1);
  const before=(await get('chats',chatId)).messages.length,input=page.getByTestId('chat-input-container').getByRole('textbox');await deviceHold(page);await input.fill('Explain this application without changing it');await input.press('Enter');
  for(const deadline=Date.now()+60000;!(await deviceAsked(page)).length;await new Promise(r=>setTimeout(r,50)))assert.ok(Date.now()<deadline,'the reader is asked the turn');
  assert.deepEqual(await deviceAsked(page),['Explain this application without changing it']);await deviceRelease(page);
  const deadline=Date.now()+30000;while(true){const q=await get('refs',`chat-queue:${chatId}`);if(q?.receipts.some(r=>r.outcome==='completed')&&!q.state.active)break;assert.ok(Date.now()<deadline,'Ask must settle after preview decline');await new Promise(r=>setTimeout(r,20));}
  assert.equal((await deviceAsked(page)).length,1);assert.equal((await get('chats',chatId)).messages.length,before+2);assert.ok((await get('chats',chatId)).messages.at(-1).model==='foundry:reader');
  // A chat in Ask mode opens with its preview collapsed (tools/plain_chat_layout_browser.mjs): the header's toggle opens it for the explicit retry.
  await page.getByTestId('toggle-preview-panel-button').click();await page.getByTestId('preview-restart-button').click();await decline(2);assert.equal(await page.evaluate(()=>window.__previewPromptCount),2,'explicit retry must retain permission flow');await remount(2);
  await page.reload();await decline(1);assert.equal(await page.evaluate(()=>window.__previewPromptCount),1,'full navigation creates a new run session');
  assert.deepEqual(errors,[]);console.log(JSON.stringify({profile:'foundry/preview-auto-start/1',initialStart:true,declineStable:true,responsiveRemounts:true,askAfterDecline:true,explicitRetry:true,reloadSession:true,pageErrors:errors}));
} finally {await browser?.close();await new Promise(resolve=>server.close(resolve));}
