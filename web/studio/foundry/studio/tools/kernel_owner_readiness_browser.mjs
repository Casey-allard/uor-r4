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
const rsvp=readFixture('./fixtures/live/rsvp.json'),parking=readFixture('./fixtures/specs/parking-fee.json');
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
  await context.addInitScript(() => {
    if(window!==window.top)return;
    const nativeFetch=window.fetch.bind(window);let release;
    const ready=new Promise(resolve=>release=resolve);
    window.__releaseOwnerVerification=()=>{window.__ownerReleased=true;release();};
    window.__ownerRepliesBeforeReady=0;
    window.fetch=(input,...args)=>{
      const url=new URL(typeof input==='string'?input:input.url??String(input),location.href);
      if(location.pathname.endsWith('/chat')&&url.pathname.endsWith('/kernel/kernel.json')){
        window.__ownerVerificationHeld=true;return ready.then(()=>nativeFetch(input,...args));
      }
      return nativeFetch(input,...args);
    };
    navigator.serviceWorker?.addEventListener('message',event=>{
      if(event.data?.kind!=='foundry:kernel-execute/1'||!event.ports[0])return;
      window.__ownerBridgeRequested=true;
      const port=event.ports[0],post=port.postMessage.bind(port);
      port.postMessage=(...args)=>{if(window.__ownerVerificationHeld&&!window.__ownerReleased)window.__ownerRepliesBeforeReady++;return post(...args);};
    });
  });
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
  await apply('<prism-spec>'+JSON.stringify(rsvp)+'</prism-spec>');
  const oldPath='prism/'+canonicalSource(rsvp).name+'.lex.tex';assert.ok(await get(page,'files',appId+':'+oldPath));
  const incoming=canonicalSource(parking),result=await apply('<prism-lexlean>'+incoming.source+'</prism-lexlean>');
  assert.ok(result.changed>0);assert.match(result.card,/view and descriptions were reset/);
  const current=JSON.parse(await get(page,'files',appId+':prism/spec.json'));
  assert.equal(current.name,parking.name);assert.equal(current.view,undefined);assert.equal(current.meanings,undefined);assert.notEqual(current.purpose,rsvp.purpose);
  assert.equal(await get(page,'files',appId+':'+oldPath),undefined);
  assert.equal(await get(page,'files',appId+':prism/'+incoming.name+'.lex.tex'),incoming.source);
  assert.equal((await page.evaluate(appId=>window.__prism.rulesOf(appId),appId)).issues.length,0);
  // Compatible logic replacement keeps the actual checked presentation.
  const custom={...current,purpose:'A custom parking explanation',view:{blocks:[{kind:'heading',text:'Your parking quote'},{kind:'input',field:'hours'},{kind:'input',field:'vehicle'},{kind:'answer'}]},meanings:Object.fromEntries(current.outcomes.map(x=>[x,'Meaning of '+x]))};
  await apply('<prism-spec>'+JSON.stringify(custom)+'</prism-spec>');
  const edited=structuredClone(parking);edited.rules[0].then=edited.outcomes.find(x=>x!==edited.rules[0].then);
  const editModule=canonicalSource(edited),same=await apply('<prism-lexlean>'+editModule.source+'</prism-lexlean>');
  assert.ok(!same.card.includes('view and descriptions were reset'));
  const after=JSON.parse(await get(page,'files',appId+':prism/spec.json'));
  for(const key of ['purpose','view','meanings'])assert.deepEqual(after[key],custom[key]);
  assert.equal(canonicalSource(after).source,editModule.source);
  const head=await get(page,'refs',appId+':main');const bad=await apply('<prism-lexlean>invalid</prism-lexlean>');assert.equal(bad.changed,0);assert.equal(await get(page,'refs',appId+':main'),head);
  await page.goto(base+'chat?id='+made.chatId+'&appId='+appId);
  // The page opens the sealed workspace (FA-93) before the bridge is asked, which takes longer on a CI runner.
  await page.waitForFunction(()=>window.__ownerVerificationHeld&&window.__ownerBridgeRequested,null,{timeout:120000});
  assert.equal(await page.evaluate(()=>window.__ownerRepliesBeforeReady),0);
  await page.evaluate(()=>window.__releaseOwnerVerification());
  await page.evaluate(()=>window.__forgeKernel.ask('drafting check')); 
  await page.evaluate(async appId=>{await window.__live.showLive(appId);await window.__live.refreshLive(appId);},appId);
  await page.getByTestId('preview-iframe-element').waitFor();
  const frame=page.frameLocator('[data-testid="preview-iframe-element"]');
  try{await frame.getByText('Your parking quote',{exact:true}).waitFor({timeout:20000});}catch(error){console.log('frames',page.frames().map(f=>f.url()));console.log('body',await page.locator('body').innerText());console.log('preview',await frame.locator('body').innerText().catch(()=>''));console.log('exceptions',errors);throw error;}
  assert.ok(!(await frame.locator('body').innerText()).includes("You're invited"));
  assert.deepEqual(errors,[]);
  console.log(JSON.stringify({profile:'foundry/kernel-owner-readiness-acceptance/1',delayedVerification:true,prematureReplies:0,fullNavigation:true,crossProfile:true,oldModuleRemoved:true,canonicalBytes:true,compatiblePresentation:true,invalidModuleNoWrite:true,actualPreview:true,pageErrors:errors}));
} finally {await browser?.close();await new Promise(resolve=>server.close(resolve));}
