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
  const context = await browser.newContext({ serviceWorkers: "allow" }), page = await context.addInitScript(deviceSeam).then(() => context.newPage()), errors = [];
  page.on("pageerror", error => errors.push(error.message));
  page.on("console",message=>{if(message.type()==='error'&&!message.text().includes('ERR_FAILED'))console.log('browser diagnostic:',message.text());});
  // Foundry answers with its own model only (Prism R4): nothing off the origin is reached.
  await context.route("**/*", route => new URL(route.request().url()).origin===origin?route.continue():route.abort());
  await page.goto(base); await page.waitForFunction(() => !!navigator.serviceWorker.controller && !!window.__host && !!window.__prism && !!window.electron);
  const call=(tab,channel,input)=>tab.evaluate(({channel,input})=>window.electron.ipcRenderer.invoke(channel,input),{channel,input});
  const get=(tab,store,key)=>tab.evaluate(({store,key})=>window.__host.get(store,key),{store,key});
  const mode=page.getByRole('combobox',{name:'Chat mode',exact:true});assert.equal(await mode.count(),1,'the browser must expose its canonical mode control');
  await page.waitForFunction(()=>!document.querySelector('[data-testid="chat-mode-selector"]')?.disabled);
  assert.deepEqual(await mode.locator('option').allTextContents(),['Build','Chat','Plan']);
  const waitValue=async value=>{const deadline=Date.now()+10000;while(await mode.inputValue()!==value||await mode.isDisabled()){assert.ok(Date.now()<deadline,'mode selection did not settle');await new Promise(r=>setTimeout(r,20));}};
  const settings=()=>get(page,'settings','user');
  const waitDefault=async value=>{const deadline=Date.now()+10000;while(true){const s=await settings();if(s.selectedChatMode===value&&s.defaultChatMode===value)return;assert.ok(Date.now()<deadline);await new Promise(r=>setTimeout(r,20));}};
  const abortNext=store=>page.evaluate(store=>{const put=IDBObjectStore.prototype.put;window.__modeAborts=0;IDBObjectStore.prototype.put=function(value,key){const r=put.call(this,value,key);if(this.name===store)r.addEventListener('success',()=>{IDBObjectStore.prototype.put=put;window.__modeAborts++;this.transaction.abort();},{once:true});return r;};},store);
  await mode.selectOption('ask');await waitDefault('ask');await waitValue('ask');
  await abortNext('settings');await mode.selectOption('plan');await page.getByRole('alert').filter({hasText:'The chat mode could not be saved. Try again.'}).waitFor();assert.equal(await page.evaluate(()=>window.__modeAborts),1);await waitDefault('ask');await waitValue('ask');
  for(const value of ['plan','build','ask']){await page.keyboard.press('Control+.');await waitDefault(value);await waitValue(value);}
  await page.reload();await mode.waitFor();await waitValue('ask');
  const made=await call(page,'create-app',{name:'mode-controls'}),chatId=made.chatId,appId=made.app.id;
  assert.equal((await get(page,'chats',chatId)).chatMode,'ask');const extra=await call(page,'create-chat',appId);assert.equal((await get(page,'chats',extra)).chatMode,'ask');
  const explicit=await call(page,'create-app',{name:'explicit-build',initialChatMode:'build'});assert.equal((await get(page,'chats',explicit.chatId)).chatMode,'build');
  const beforeInvalid=await page.evaluate(()=>window.__host.all('apps'));
  const refused=await page.evaluate(async()=>{try{await window.electron.ipcRenderer.invoke('create-app',{name:'unsupported',initialChatMode:'local-agent'});}catch(e){return e.code;}});assert.equal(refused,'invalidChatExecutionMode');assert.deepEqual(await page.evaluate(()=>window.__host.all('apps')),beforeInvalid);
  await call(page,'set-user-settings',{selectedChatMode:'build',defaultChatMode:'build',selectedModel:DEVICE_MODEL});
  await page.evaluate(({appId,spec})=>window.__prism.rulesAfterTurn(appId,'<prism-spec>'+JSON.stringify(spec)+'</prism-spec>'),{appId,spec:JSON.parse(fs.readFileSync(new URL('./fixtures/live/rsvp.json',import.meta.url),'utf8'))});
  await page.goto(base+'chat?id='+chatId+'&appId='+appId);await mode.waitFor();await waitValue('ask');
  const chat=()=>get(page,'chats',chatId),waitChat=async value=>{const deadline=Date.now()+10000;while((await chat()).chatMode!==value){assert.ok(Date.now()<deadline);await new Promise(r=>setTimeout(r,20));}await waitValue(value);};
  await mode.selectOption('plan');await waitChat('plan');assert.equal((await settings()).defaultChatMode,'build');
  for(const value of ['build','ask']){await page.keyboard.press('Control+.');await waitChat(value);}
  await abortNext('chats');await mode.selectOption('plan');await page.getByRole('alert').filter({hasText:'The chat mode could not be saved. Try again.'}).waitFor();assert.equal(await page.evaluate(()=>window.__modeAborts),1);await waitChat('ask');
  await mode.selectOption('plan');await waitChat('plan');await mode.selectOption('ask');await waitChat('ask');
  const snapshot=()=>page.evaluate(async appId=>({files:(await window.__host.all('files')),head:await window.__host.get('refs',`${appId}:main`),rules:await window.__prism.rulesOf(appId),versions:await window.__host.get('versions',appId)}),appId),before=await snapshot();
  await deviceHold(page);
  const idea='Build a freelance invoice tracker with client profiles, invoice status, monthly revenue charts, overdue reminders, and a clean dashboard.';
  const input=page.getByTestId('chat-input-container').getByRole('textbox');await input.fill(idea);await input.press('Enter');
  // The turn is with the reader, held there: it was asked to read the Ask turn's words.
  for(const until=Date.now()+60000;!(await deviceAsked(page)).length;await new Promise(r=>setTimeout(r,20)))assert.ok(Date.now()<until,'the reader is asked');
  assert.deepEqual(await deviceAsked(page),[idea]);
  const peer=await context.newPage();await peer.goto(base);await peer.waitForFunction(()=>!!window.electron&&!!window.__host);await call(peer,'update-chat',{chatId,chatMode:'build'});
  await deviceRelease(page);
  const deadline=Date.now()+30000;let receipt;
  while(Date.now()<deadline){const j=await get(page,'refs',`chat-queue:${chatId}`);if(j?.receipts.length&&!j.state.active){receipt=j.receipts.at(-1);break;}await new Promise(r=>setTimeout(r,20));}
  assert.equal(receipt?.outcome,'completed');assert.equal((await deviceAsked(page)).length,1);assert.deepEqual(await snapshot(),before,'queued Ask must remain read-only after another window selects Build');assert.ok((await chat()).messages.at(-1).content.includes('&lt;prism-lexlean&gt;'));
  await page.reload();await mode.waitFor();await waitValue('build');
  for(const width of [390,768,1280]){
    await page.setViewportSize({width,height:844});await mode.waitFor();await waitValue('build');await page.getByTestId('model-picker').hover();
    for(const control of [mode,page.getByTestId('model-picker')]){const box=await control.boundingBox();assert.ok(box&&box.x>=0&&box.x+box.width<=width+1,'composer controls must fit the viewport');}
    await page.keyboard.press('Control+.');await waitChat('ask');await mode.selectOption('build');await waitChat('build');
  }
  assert.deepEqual(errors,[]);
  console.log(JSON.stringify({profile:'foundry/browser-chat-mode-controls/1',canonicalChoices:true,homePersistence:true,chatPersistence:true,keyboardScope:true,initialDefaults:true,explicitOverride:true,unsupportedRefused:true,lateAbortSettings:true,lateAbortChat:true,queuedModeImmutable:true,responsiveModes:true,readOnlyGeneration:true,reload:true,pageErrors:errors}));
} finally {await browser?.close();await new Promise(resolve=>server.close(resolve));}
