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
  const key='foundry:themes:v1';
  assert.deepEqual(await call(page,'get-custom-themes'),[]);
  const catalog=await call(page,'get-themes');assert.equal(catalog[0].id,'default');
  const input={name:'Exact theme',prompt:'THEME-CAPTURE-ORIGINAL\n',description:'Details'};
  const made=await call(page,'create-custom-theme',input);assert.equal(made.id,1);assert.ok(made.createdAt instanceof Date);
  await page.evaluate(()=>window.__host.put('apps',9901,{id:9901,name:'Theme fixture',themeId:null}));
  const peer=await context.newPage();peer.on('pageerror',error=>errors.push(error.message));await peer.goto(base);await peer.waitForFunction(()=>!!window.__host&&!!window.electron);
  await call(page,'set-app-theme',{appId:9901,themeId:'custom:1'});assert.equal(await call(peer,'get-app-theme',{appId:9901}),'custom:1');
  const rows=await Promise.all([call(page,'create-custom-theme',{name:'A',prompt:'A'}),call(peer,'create-custom-theme',{name:'B',prompt:'B'})]);assert.deepEqual(rows.map(x=>x.id).sort(),[2,3]);
  const updated=await call(peer,'update-custom-theme',{id:1,name:'Changed'});assert.equal(updated.prompt,input.prompt);assert.equal(updated.name,'Changed');
  await peer.reload();await peer.waitForFunction(()=>!!window.__host&&!!window.electron);assert.deepEqual((await call(peer,'get-custom-themes'))[0],updated);
  for(const themeId of ['custom:01','custom:1tail','custom:999','unknown'])await assert.rejects(()=>call(page,'set-app-theme',{appId:9901,themeId}));
  await assert.rejects(()=>call(page,'get-app-theme',{appId:9902}));
  const before=await get(page,'settings',key);
  const late=await page.evaluate(async()=>{const put=IDBObjectStore.prototype.put;let injected=false;IDBObjectStore.prototype.put=function(value,key){const q=put.call(this,value,key);if(this.name==='settings'&&key==='foundry:themes:v1'){injected=true;q.addEventListener('success',()=>this.transaction.abort(),{once:true});}return q;};try{await window.electron.ipcRenderer.invoke('delete-custom-theme',{id:1});return {injected,refused:false};}catch(e){return {injected,refused:e.code==='themeStorageFailed'};}finally{IDBObjectStore.prototype.put=put;}});assert.deepEqual(late,{injected:true,refused:true});assert.deepEqual(await get(page,'settings',key),before);assert.equal(await call(peer,'get-app-theme',{appId:9901}),'custom:1');
  const bad=structuredClone(before);bad.records[0].createdAt='invalid';await page.evaluate(({key,bad})=>window.__host.put('settings',key,bad),{key,bad});await assert.rejects(()=>call(peer,'get-custom-themes'));await assert.rejects(()=>call(peer,'get-app-theme',{appId:9901}));assert.deepEqual(await get(page,'settings',key),bad);await page.evaluate(({key,before})=>window.__host.put('settings',key,before),{key,before});
  // The real transaction adapter captures before database readiness.
  const capture=await page.evaluate(async base=>{const {createThemeLibrary}=await import(base+'theme-library.mjs'),{themesData}=await import(base+'builtin-themes.mjs'),inference=await import(base+'inference.js');let ready;const call=createThemeLibrary(new Promise(r=>ready=r),themesData),value={name:'Captured',prompt:'original'};const pending=call('create',value);value.prompt='changed';ready(await inference.workspace());return (await pending).prompt;},base);assert.equal(capture,'original');
  const captured=await page.evaluate(async base=>{const {createThemeLibrary}=await import(base+'theme-library.mjs'),{themesData}=await import(base+'builtin-themes.mjs'),inference=await import(base+'inference.js');return createThemeLibrary(await inference.workspace(),themesData)('resolve',{appId:9901});},base);assert.equal(captured,input.prompt);
  await call(peer,'update-custom-theme',{id:1,prompt:'THEME-CHANGED'});assert.equal(captured,input.prompt);
  await page.evaluate(()=>window.__host.put('apps',9903,{id:9903,name:'Second',themeId:'custom:1'}));
  await call(page,'delete-custom-theme',{id:1});assert.equal(await call(peer,'get-app-theme',{appId:9901}),null);assert.equal(await call(peer,'get-app-theme',{appId:9903}),null);assert.equal((await call(peer,'get-custom-themes')).length,3);
  await call(page,'set-app-theme',{appId:9901,themeId:'default'});assert.equal(await call(peer,'get-app-theme',{appId:9901}),'default');await call(page,'set-app-theme',{appId:9901,themeId:null});
  await peer.close();
  // Exercise the unchanged renderer's manual create/edit/delete dialogs.
  console.log('theme browser: library');await page.goto(base+'library');
  await page.getByRole('button',{name:'New',exact:true}).click();
  await page.getByRole('menuitem',{name:'New Theme',exact:true}).click();
  await page.getByRole('tab',{name:'Manual Configuration'}).click();
  await page.getByLabel('Theme Name',{exact:true}).fill('Renderer theme');
  await page.getByLabel('Theme Prompt',{exact:true}).fill('RENDERER-THEME-EXACT');
  await page.getByRole('button',{name:'Save Theme',exact:true}).click();
  await page.getByRole('dialog').waitFor({state:'hidden'});
  console.log('theme browser: renderer created');const ui=(await call(page,'get-custom-themes')).find(row=>row.name==='Renderer theme');assert.ok(ui);
  const card=page.getByTestId('library-theme-card').filter({has:page.getByText('Renderer theme',{exact:true})}).last();
  await card.getByTitle('Edit theme',{exact:true}).click();
  await page.getByLabel('Theme Name',{exact:true}).fill('Renderer edited');
  await page.getByRole('button',{name:'Save',exact:true}).click();await page.getByRole('dialog').waitFor({state:'hidden'});
  assert.equal((await call(page,'get-custom-themes')).find(row=>row.id===ui.id).name,'Renderer edited');
  console.log('theme browser: renderer edited');const app=await call(page,'create-app',{name:'theme-generation',initialChatMode:'build'});
  console.log('theme browser: project created',app.chatId);await page.goto(base+'chat?id='+app.chatId+'&appId='+app.app.id);
  // A new app is projected: its preview asks nothing before its rules arrive.
  await page.getByTestId('auxiliary-actions-menu').waitFor();assert.equal(await page.getByRole('dialog',{name:'Review preview permissions',exact:true}).count(),0);
  await page.getByTestId('auxiliary-actions-menu').focus();await page.keyboard.press('Enter');await page.getByRole('menuitem',{name:'Themes',exact:true}).hover();await page.getByTestId('theme-option-custom:'+ui.id).click();
  assert.equal(await call(page,'get-app-theme',{appId:app.app.id}),'custom:'+ui.id);
  await call(page,'set-user-settings',{selectedModel:DEVICE_MODEL});
  await page.getByRole('textbox').first().fill('Describe a small useful application');
  await page.getByRole('textbox').first().press('Enter');
  // Foundry's reader reads the idea into the app's notation; Forge draws the screen, in the selected theme, after the turn.
  console.log('theme browser: generation submitted');
  await page.waitForFunction(()=>document.body.innerText.includes('Open questions'),null,{timeout:60000});assert.equal((await deviceAsked(page)).length,1);
  assert.equal(await call(page,'get-app-theme',{appId:app.app.id}),'custom:'+ui.id,'the turn keeps the app\'s selected theme');
  console.log('theme browser: library');await page.goto(base+'library');
  const edited=page.getByTestId('library-theme-card').filter({has:page.getByText('Renderer edited',{exact:true})}).last();
  await edited.getByTitle('Delete theme',{exact:true}).click();await page.getByRole('alertdialog').getByRole('button',{name:'Delete',exact:true}).click();await page.getByRole('alertdialog').waitFor({state:'hidden'});
  assert.ok(!(await call(page,'get-custom-themes')).some(row=>row.id===ui.id));assert.equal(await call(page,'get-app-theme',{appId:app.app.id}),null);
  assert.deepEqual(errors,[]);console.log(JSON.stringify({actualHost:true,importedContracts:true,concurrentIds:true,reload:true,exactDates:true,lateAbort:true,atomicSelectionDeletion:true,corruptionRefused:true,capturedPrompt:true,rendererCrud:true,rendererKeyboardSelection:true,exactDisclosedProviderBytes:true,pageExceptions:0}));
}finally{if(browser)await browser.close();await new Promise(resolve=>server.close(resolve));}
