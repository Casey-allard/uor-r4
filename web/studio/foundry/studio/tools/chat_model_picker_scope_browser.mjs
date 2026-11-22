// Durable guest-admitted delivery through actual host, bridge and IndexedDB.
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import http from "node:http";
import { fileURLToPath } from "node:url";
import { chromium } from "../vendor/prism/node_modules/playwright-core/index.mjs";
import { DEVICE_MODEL, deviceSeam } from "./device_fixture.mjs";
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
  const context = await browser.newContext({ serviceWorkers: "block" }), page = await context.addInitScript(deviceSeam).then(() => context.newPage()), errors = [], sent = [];
  page.on("pageerror", error => errors.push(error.message));
  page.on("console",message=>{if(message.type()==='error'&&!message.text().includes('ERR_FAILED'))console.log('browser diagnostic:',message.text());});
  // Foundry offers its own model only (Prism R4): nothing off the origin is reached.
  await context.route("**/*", route => {const url=new URL(route.request().url());if(/openrouter\.ai|featherless/.test(url.host))sent.push(url.href);return url.origin===origin?route.continue():route.abort();});
  await page.goto(base); await page.waitForFunction(() => !!window.__host && !!window.__prism && !!window.electron);
  const call=(tab,channel,input)=>tab.evaluate(({channel,input})=>window.electron.ipcRenderer.invoke(channel,input),{channel,input});
  const get=(tab,store,key)=>tab.evaluate(({store,key})=>window.__host.get(store,key),{store,key});
  const made=await call(page,'create-app',{name:'picker-scope',initialChatMode:'build'}),other=await call(page,'create-app',{name:'other-picker-chat',initialChatMode:'build'});
  // A chat can still hold a selection stored before Foundry offered only its own model. The picker
  // offers the device model alone; choosing it is the chat's own change, never Home's or another chat's.
  const globalModel=DEVICE_MODEL,cloud={provider:'featherless',name:'zai-org/GLM-5.3-Flash',effortLevel:'medium'};
  await call(page,'set-user-settings',{selectedModel:globalModel});await call(page,'update-chat',{chatId:made.chatId,modelSelection:cloud});await call(page,'update-chat',{chatId:other.chatId,modelSelection:cloud});
  const openChat=async()=>{
    await page.goto(base+'chat?id='+made.chatId+'&appId='+made.app.id);await page.waitForFunction(()=>!!window.__host&&!!window.electron);
    const preview=page.getByRole('dialog',{name:'Review preview permissions',exact:true});try{await preview.waitFor({timeout:3000});await preview.getByRole('button',{name:"Don't allow",exact:true}).click();}catch(error){if(error.name!=='TimeoutError')throw error;}
    await page.getByTestId('model-picker').waitFor();await page.waitForFunction(()=>!document.querySelector('[data-testid="model-picker"]')?.disabled);
  };
  const choose=async label=>{const item=page.getByRole('menuitem',{name:new RegExp(label)});if(!await item.isVisible())await page.getByTestId('model-picker').click();await item.click();};
  const waitModel=async(chatId,provider)=>{const deadline=Date.now()+10000;while(Date.now()<deadline){const row=await get(page,'chats',chatId);if(row.modelSelection?.provider===provider)return row;await new Promise(r=>setTimeout(r,20));}throw Error('chat model did not commit');};
  await openChat();await page.getByTestId('model-picker').click();assert.deepEqual((await page.getByRole('menuitem').allTextContents()).filter(t=>/Foundry reader|Bonsai|GLM|Kimi|OpenRouter|Featherless/.test(t)).map(t=>/Foundry reader/.test(t)),[true],'the picker offers the device model alone');
  await page.evaluate(()=>{const put=IDBObjectStore.prototype.put;window.__pickerAbort=0;IDBObjectStore.prototype.put=function(value,key){const request=put.call(this,value,key);if(this.name==='chats'){request.addEventListener('success',()=>{IDBObjectStore.prototype.put=put;window.__pickerAbort++;this.transaction.abort();},{once:true});}return request;};});
  await choose('Foundry reader');await page.getByRole('alert').filter({hasText:'The model selection could not be saved. Try again.'}).waitFor();
  assert.equal(await page.evaluate(()=>window.__pickerAbort),1);assert.deepEqual((await get(page,'chats',made.chatId)).modelSelection,cloud);assert.deepEqual((await get(page,'settings','user')).selectedModel,globalModel);
  await choose('Foundry reader');const changed=await waitModel(made.chatId,'local');assert.equal(changed.messages.length,0,'empty chat must persist selection');assert.deepEqual((await get(page,'settings','user')).selectedModel,globalModel);assert.deepEqual((await get(page,'chats',other.chatId)).modelSelection,cloud);
  await page.goto(base);await page.getByTestId('model-picker').waitFor();await choose('Foundry reader');
  const deadline=Date.now()+10000;while((await get(page,'settings','user')).selectedModel.provider!=='local'){assert.ok(Date.now()<deadline);await new Promise(r=>setTimeout(r,20));}
  assert.deepEqual((await get(page,'chats',other.chatId)).modelSelection,cloud,'Home changes the default only');
  await openChat();assert.ok((await page.getByTestId('model-picker').textContent()).includes('Foundry reader'));assert.equal((await get(page,'chats',made.chatId)).modelSelection.provider,'local');
  await page.reload();await page.getByTestId('model-picker').waitFor();await page.waitForFunction(()=>!document.querySelector('[data-testid="model-picker"]')?.disabled);assert.ok((await page.getByTestId('model-picker').textContent()).includes('Foundry reader'));
  assert.equal(sent.length,0);assert.deepEqual(errors,[]);console.log(JSON.stringify({profile:'foundry/chat-model-picker-scope/1',mountedChatLabel:true,emptyChatPersistence:true,lateAbortVisible:true,homeDefaultIndependent:true,otherChatUnchanged:true,reload:true,pageErrors:errors}));
} finally {await browser?.close();await new Promise(resolve=>server.close(resolve));}
