// Durable guest-admitted delivery through actual host, bridge and IndexedDB.
import assert from "node:assert/strict";
import {observeReducedMotion} from "./reduced_motion_oracle.mjs";
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

  const started=Date.now();
  await page.goto(base+'settings');await page.waitForFunction(()=>!!window.__host&&!!window.electron);await page.getByRole('button',{name:'Interactive apps',exact:true}).click();
  await page.getByLabel('Application name').fill('Journey gallery');
  await page.getByRole('button',{name:'Create gallery',exact:true}).click();
  await page.getByRole('heading',{name:'Open landscape',exact:true}).waitFor();
  await page.getByRole('button',{name:'Next room',exact:true}).focus();await page.keyboard.press('Enter');
  await page.getByRole('heading',{name:'Night sky',exact:true}).waitFor();
  assert.equal(await page.getByRole('button',{name:'Next room',exact:true}).isDisabled(),true);
  const firstPixelMs=Date.now()-started;
  await page.getByRole('button',{name:'Close',exact:true}).click();
  await page.reload();await page.waitForFunction(()=>!!window.__host&&!!window.electron);
  await page.goto(base+'settings');await page.waitForFunction(()=>!!window.__host&&!!window.electron);await page.getByRole('button',{name:'Interactive apps',exact:true}).click();
  await page.getByRole('dialog').getByRole('button',{name:'Journey gallery',exact:true}).click();
  await page.getByRole('heading',{name:'Night sky',exact:true}).waitFor();
  await page.getByRole('button',{name:'Close',exact:true}).click();
  const report=await page.evaluate(async()=>{
    const call=(channel,input)=>window.electron.ipcRenderer.invoke(channel,input),check=(ok,message)=>{if(!ok)throw Error(message);};
    const made=await call('forge:interaction-create',{name:'Journey garden',kind:'garden'}),appId=made.app.id;
    const run=action=>call('forge:interaction-run',{appId,event:0,reset:false,audioGranted:false,reducedMotion:true,...action});
    const first=await run({event:1});check(first.output.state===1&&first.output.tone===0&&first.output.motion===0,'garden ignored consent or motion');
    const audible=await run({audioGranted:true,reducedMotion:false});check(audible.output.tone===440&&audible.output.motion===20,'garden domains did not coordinate');
    await call('edit-app-file',{appId,filePath:'notes.txt',content:'retained semantic state'});
    check((await run({})).output.state===1,'unrelated edit reset the state');
    const d=JSON.parse(await call('read-app-file',{appId,filePath:'prism/interaction.json'}));d.graph.frames[1].tone=660;
    await call('edit-app-file',{appId,filePath:'prism/interaction.json',content:JSON.stringify(d)});
    let code;try{await run({});}catch(error){code=error.code;}check(code==='interactionStateStale','changed graph reused stale state');
    check((await run({reset:true})).output.state===0,'explicit reset failed');
    try{await call('edit-app-file',{appId,filePath:'prism/Application.lex.tex',content:'tampered'});throw Error('canonical write succeeded');}catch(error){check(error.code==='interactionApplicationMismatch','canonical authority was writable');}
    const {workspace:db,kappa}=await import(new URL('inference.js',location.href)),database=await db();
    const {runStoredInteraction}=await import(new URL('interaction-state.mjs',location.href));
    const guest=await window.ForgeKernel.load(path=>fetch(new URL(path,location.href))),model=await (await fetch(new URL('interaction-model.lex.tex',location.href))).text();
    let arrivals=0,release;const barrier=new Promise(resolve=>release=resolve),delayed={dispatch:async bytes=>{if(++arrivals===2)release();await barrier;return guest.dispatch(bytes);}};
    const command={appId,event:1,reset:false,audioGranted:false,reducedMotion:true};
    const race=await Promise.allSettled([0,1].map(()=>runStoredInteraction(database,delayed,kappa,command,model)));
    check(race.filter(r=>r.status==='fulfilled').length===1&&race.some(r=>r.status==='rejected'&&r.reason.code==='interactionStateChanged'),'competing transitions both committed');
    const before=await window.__host.get('refs',`interaction:${appId}`),original=database.transaction;
    database.transaction=function(...args){const tx=original.apply(this,args);if(args[1]==='readwrite'){const get=tx.objectStore.bind(tx);tx.objectStore=name=>{const store=get(name);if(name==='refs'){const put=store.put.bind(store);store.put=(value,key)=>{const request=put(value,key);if(key===`interaction:${appId}`)queueMicrotask(()=>{try{tx.abort();}catch{}});return request;};}return store;};}return tx;};
    let abort;try{await runStoredInteraction(database,guest,kappa,{...command,event:2},model);}catch(error){abort=error.code;}finally{database.transaction=original;}
    check(abort==='projectEditStorageFailed'&&JSON.stringify(await window.__host.get('refs',`interaction:${appId}`))===JSON.stringify(before),'transaction abort lost application state');
    const {createInteractionAudio}=await import(new URL('interaction-ui.mjs',location.href));
    async function render(enabled){const offline=new OfflineAudioContext(1,4800,48000),adapter=createInteractionAudio(()=>({state:'running',resume:async()=>{},close:async()=>{},createOscillator:()=>offline.createOscillator(),createGain:()=>offline.createGain(),destination:offline.destination}));if(enabled)await adapter.enable();adapter.render(audible.output.tone);const buffer=await offline.startRendering();adapter.revoke();return buffer.getChannelData(0);}
    const silence=await render(false);check(silence.every(x=>x===0),'unconsented audio produced samples');const samples=await render(true);let peak=0,frequency=0;for(let hz=20;hz<=1000;hz+=10){let re=0,im=0;for(let i=0;i<samples.length;i++){re+=samples[i]*Math.cos(2*Math.PI*hz*i/48000);im+=samples[i]*Math.sin(2*Math.PI*hz*i/48000);}const power=re*re+im*im;if(power>peak){peak=power;frequency=hz;}}check(frequency===440&&peak>1,'actual audio spectrum does not match guest tone');
    await call('delete-app',{appId});check(await window.__host.get('refs',`interaction:${appId}`)===undefined,'deletion retained state');
    return {guestDomains:true,unrelatedEdit:true,staleModel:true,explicitReset:true,canonicalReadOnly:true,concurrentWinner:1,rollback:true,audioPeakHz:frequency,unconsentedSamplesSilent:true,deletion:true};
  });
  await page.emulateMedia({reducedMotion:'reduce'});
  await page.goto(base+'settings');await page.waitForFunction(()=>!!window.__host&&!!window.electron);await page.getByRole('button',{name:'Interactive apps',exact:true}).click();
  await page.getByLabel('Application name').fill('Motion garden');await page.getByRole('button',{name:'Create garden',exact:true}).click();
  await page.getByRole('heading',{name:'Seed',exact:true}).waitFor();
  const reducedMotionObservation=await page.evaluate(observeReducedMotion);
  // Exercise the same oracle against real JS and CSS movement, then restore.
  for(const kind of ['waapi','css']){
    await page.evaluate(kind=>{
      const section=document.querySelector('dialog section');
      if(kind==='waapi')window.__reducedMotionMutant=section.animate([{transform:'translateY(0)'},{transform:'translateY(12px)'}],{duration:3000,iterations:Infinity});
      else {const style=document.createElement('style');style.textContent='@keyframes foundry-motion-mutant { from { transform: translateY(0); } to { transform: translateY(12px); } } dialog section { animation: foundry-motion-mutant 3s linear infinite !important; }';document.head.append(style);window.__reducedMotionMutant=style;}
    },kind);
    try {await assert.rejects(page.evaluate(observeReducedMotion),/Reduced motion: active effect/);}
    finally {await page.evaluate(()=>{const mutant=window.__reducedMotionMutant;if(mutant instanceof Animation)mutant.cancel();else mutant.remove();delete window.__reducedMotionMutant;});}
    await page.evaluate(observeReducedMotion);
  }
  await page.getByRole('button',{name:'Enable sound',exact:true}).click();await page.getByRole('button',{name:'Mute sound',exact:true}).waitFor();
  await page.getByRole('button',{name:'Mute sound',exact:true}).click();await page.getByRole('button',{name:'Enable sound',exact:true}).waitFor();
  await page.getByRole('button',{name:'Close',exact:true}).click();
  assert.deepEqual(errors,[]);console.log(JSON.stringify({...report,keyboardGallery:true,reloadRecovery:true,reducedMotion:true,reducedMotionObservation,reducedMotionNegativeControls:2,explicitAudio:true,firstPixelMs,pageExceptions:errors.length}));
} finally { if(browser)await browser.close();await new Promise(resolve=>server.close(resolve)); }
