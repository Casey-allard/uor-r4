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
const fixture=JSON.parse(fs.readFileSync(new URL('./fixtures/specs/parking-fee.json',import.meta.url),'utf8')).spec;
const {SPEC_RULES}=await import('../shell/prism.js');
const idea='Build a freelance invoice tracker with client profiles, invoice status, monthly revenue charts, overdue reminders, and a clean dashboard.';
let browser;
try {
  browser = await chromium.launch({ executablePath, headless: true, args: ["--no-sandbox"] });
  const context = await browser.newContext({ serviceWorkers: "allow" }), page = await context.addInitScript(deviceSeam).then(() => context.newPage()), errors = [];
  page.on("pageerror", error => errors.push(error.message));
  page.on("console",message=>{if(message.type()==='error'&&!message.text().includes('ERR_FAILED'))console.log('browser diagnostic:',message.text());});
  // Foundry answers with its own model only (Prism R4): nothing off the origin is reached.
  await context.route("**/*", route => new URL(route.request().url()).origin===origin?route.continue():route.abort());
  await page.goto(base); await page.waitForFunction(() => !!navigator.serviceWorker.controller && !!window.__host && !!window.__prism && !!window.electron);
  const call=(channel,input)=>page.evaluate(({channel,input})=>window.electron.ipcRenderer.invoke(channel,input),{channel,input});
  const made=await call('create-app',{name:'canonical-generation-integration',initialChatMode:'build'}),appId=made.app.id;
  await page.evaluate(({appId,fixture})=>window.__prism.rulesAfterTurn(appId,'<prism-spec>'+JSON.stringify(fixture)+'</prism-spec>'),{appId,fixture});
  await call('edit-app-file',{appId,filePath:'notes.txt',content:'PRIVATE-IMPLEMENTATION-FIXTURE'});
  const theme=await call('create-custom-theme',{name:'Generation fixture',prompt:'THEME-CAPTURED-BEFORE-APPROVAL'});await call('set-app-theme',{appId,themeId:'custom:'+theme.id});
  await call('set-user-settings',{selectedModel:DEVICE_MODEL});
  const stored=path=>page.evaluate(({appId,path})=>window.__host.get('files',appId+':'+path),{appId,path});
  const before=await stored('prism/spec.json');
  await page.goto(base+'chat?id='+made.chatId+'&appId='+appId);await page.waitForFunction(()=>!!window.electron);
  // The preview is read in this tab: no hosted dialog may open and no request may reach the hosted service.
  const hostedAsked=[];page.on('request',r=>{if(r.url().startsWith('https://mcp.uor.foundation/'))hostedAsked.push(r.url());});
  await page.evaluate(()=>{window.__hostedDialogs=[];setInterval(()=>{for(const d of document.querySelectorAll('dialog[aria-label="Review hosted source transfer"],dialog[aria-label="Review preview permissions"]'))window.__hostedDialogs.push(d.getAttribute('aria-label'));},50);});
  await deviceHold(page);
  await page.getByTestId('chat-input-container').getByRole('textbox').fill(idea);await page.getByTestId('chat-input-container').getByRole('textbox').press('Enter');
  for(const deadline=Date.now()+60000;!(await deviceAsked(page)).length;await new Promise(r=>setTimeout(r,50)))assert.ok(Date.now()<deadline,'the reader is asked the turn');
  // The reader is asked the person's words and nothing else: never the theme, the rules, the module or the implementation source.
  const [asked]=await deviceAsked(page);assert.equal(asked,idea);
  for(const hidden of ['THEME-CAPTURED-BEFORE-APPROVAL','\\begin{lexlean}','PRIVATE-IMPLEMENTATION-FIXTURE'])assert.ok(!asked.includes(hidden),hidden);
  await call('update-custom-theme',{id:theme.id,prompt:'THEME-CHANGED-DURING-TURN'});
  await deviceRelease(page);
  for(const deadline=Date.now()+60000;;await new Promise(r=>setTimeout(r,100))){const j=await page.evaluate(id=>window.__host.get('refs','chat-queue:'+id),made.chatId);if(j?.receipts.length&&!j.state.active){assert.equal(j.receipts.at(-1).outcome,'completed');break;}assert.ok(Date.now()<deadline,'the turn completes');}
  const chat=await call('get-chat',made.chatId),last=chat.messages.at(-1);
  // The reading becomes the app's module; the chat shows the person what was read, never the module's text.
  assert.equal(last.model,DEVICE_MODEL.name);assert.ok(last.content.includes('goal: what is the app for'),last.content);assert.ok(!last.content.includes('definition{')&&!last.content.includes('\\begin{lexlean}'));
  assert.ok((await stored('prism/App.lex.tex')).includes('\\begin{lexlean}'));assert.ok((await stored('prism/app.notation')).startsWith('app Freelance invoice tracker'));
  assert.equal(await stored('notes.txt'),'PRIVATE-IMPLEMENTATION-FIXTURE');assert.ok(before);assert.equal(await stored('prism/spec.json'),undefined,'a kept app retires the decision it was');
  assert.deepEqual(await deviceAsked(page),[asked]);
  const meter=await call('chat:count-tokens',{chatId:made.chatId,input:'next'});
  assert.equal(meter.systemPromptTokens,Math.ceil((SPEC_RULES+'\n\nTHEME-CHANGED-DURING-TURN').length/4));assert.equal(meter.messageHistoryTokens,chat.messages.reduce((n,m)=>n+Math.ceil((m.content||'').length/4),0));
  assert.deepEqual(await page.evaluate(()=>window.__hostedDialogs),[],'the turn opens no hosted dialog');assert.deepEqual(hostedAsked,[],'the turn asks no server');
  assert.deepEqual(errors,[]);console.log(JSON.stringify({actualRenderer:true,readerRequest:true,implementationSourceExcluded:true,readingStoredAsModule:true,moduleHiddenFromChat:true,contextMeter:true,pageExceptions:0}));
}finally{if(browser)await browser.close();await new Promise(resolve=>server.close(resolve));}
