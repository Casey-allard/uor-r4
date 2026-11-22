import {canonicalSource,readLexLean} from '../shell/live/lexlean.mjs';
// Durable guest-admitted delivery through actual host, bridge and IndexedDB.
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import http from "node:http";
import { fileURLToPath } from "node:url";
import { chromium } from "../vendor/prism/node_modules/playwright-core/index.mjs";
assert.equal(JSON.parse(fs.readFileSync(new URL("../vendor/prism/node_modules/monaco-editor/package.json", import.meta.url), "utf8")).version, "0.52.2");
const readFixture=p=>{const v=JSON.parse(fs.readFileSync(new URL(p,import.meta.url)));return v.spec??v;};
const verdant=readLexLean(fs.readFileSync(new URL('./fixtures/live/verdant.lex.tex',import.meta.url),'utf8'));
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
  const context = await browser.newContext({ serviceWorkers: "allow" }), page = await context.newPage(), errors = [], sent = [];
  page.on("pageerror", error => errors.push(error.message));
  page.on("console",message=>{if(message.type()==='error'&&!message.text().includes('ERR_FAILED'))console.log('browser diagnostic:',message.text());});
  await context.route("**/*", route => {
    const request=route.request(),url=new URL(request.url());
    return url.origin===origin?route.continue():route.abort();
  });
  await page.goto(base); await page.waitForFunction(() => !!navigator.serviceWorker.controller && !!window.__host && !!window.__prism && !!window.electron);
  const call=(tab,channel,input)=>tab.evaluate(({channel,input})=>window.electron.ipcRenderer.invoke(channel,input),{channel,input});
  const get=(tab,store,key)=>tab.evaluate(({store,key})=>window.__host.get(store,key),{store,key});
  const made=await call(page,'create-app',{name:'module-presentation',initialChatMode:'build'}),appId=made.app.id;
  const apply=text=>page.evaluate(({appId,text})=>window.__prism.rulesAfterTurn(appId,text),{appId,text});
  const initial=canonicalSource(verdant),result=await apply('<prism-lexlean>'+initial.source+'</prism-lexlean>');
  assert.ok(result.changed>0,'complete large module must be stored');
  assert.equal(await get(page,'files',appId+':prism/'+initial.name+'.lex.tex'),initial.source);
  const status=await page.evaluate(appId=>window.__prism.rulesOf(appId),appId);
  assert.equal(status.phase,'checked');assert.equal(status.proof,null);assert.equal(status.spec.rules.length,verdant.rules.length);
  assert.ok(!status.notes.some(note=>note.includes('Lean proves at most')),'the tab proves every rule; no rule cap remains');
  const custom={...verdant,purpose:'My complete garden',view:{blocks:[{kind:'heading',text:'My complete garden'},{kind:'answer'}]},meanings:{[verdant.outcomes[0]]:'Garden response'}};
  await apply('<prism-spec>'+JSON.stringify(custom)+'</prism-spec>');
  const edited=structuredClone(verdant);edited.rules.push({...structuredClone(edited.rules[0]),id:'rule'+(edited.rules.length+1)});
  const incoming=canonicalSource(edited),replacement=await apply('<prism-lexlean>'+incoming.source+'</prism-lexlean>');
  assert.ok(replacement.changed>0);assert.ok(!replacement.card.includes('view and descriptions were reset'));
  const stored=JSON.parse(await get(page,'files',appId+':prism/spec.json'));
  assert.equal(stored.rules.length,edited.rules.length);for(const key of ['purpose','view','meanings'])assert.deepEqual(stored[key],custom[key]);
  assert.equal(await get(page,'files',appId+':prism/'+incoming.name+'.lex.tex'),incoming.source);
  const before=await get(page,'refs',appId+':main'),invalid=structuredClone(edited);invalid.rules.at(-1).then='unknown';
  const rejected=await apply('<prism-lexlean>'+canonicalSource(invalid).source+'</prism-lexlean>');assert.equal(rejected.changed,0);assert.equal(await get(page,'refs',appId+':main'),before);
  await page.goto(base+'chat?id='+made.chatId+'&appId='+appId);await page.waitForFunction(()=>!!window.__live&&!!window.__forgeKernel);
  await page.evaluate(()=>window.__forgeKernel.ask('drafting check'));
  await page.evaluate(async appId=>{await window.__live.showLive(appId);await window.__live.refreshLive(appId);},appId);
  const frame=page.frameLocator('[data-testid="preview-iframe-element"]');await frame.getByText('My complete garden',{exact:true}).waitFor({timeout:20000});
  assert.deepEqual(errors,[]);
  console.log(JSON.stringify({profile:'foundry/large-module-presentation-acceptance/1',rules:verdant.rules.length,completeCanonicalSource:true,checkedLocally:true,proofRefused:true,compatiblePresentation:true,invalidTailNoWrite:true,actualPreview:true,pageErrors:errors}));
} finally {await browser?.close();await new Promise(resolve=>server.close(resolve));}
