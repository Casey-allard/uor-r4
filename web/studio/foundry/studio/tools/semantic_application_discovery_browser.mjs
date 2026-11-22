// Durable guest-admitted delivery through actual host, bridge and IndexedDB.
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
  const context = await browser.newContext({ serviceWorkers: "block" }), page = await context.newPage(), errors = [];
  page.on("pageerror", error => errors.push(error.message));
  await context.route("**/*", route => new URL(route.request().url()).origin === origin ? route.continue() : route.abort());
  await page.goto(base); await page.waitForFunction(() => !!window.__host && !!window.__prism && !!window.electron);
  const invoke=(channel,input)=>page.evaluate(({channel,input})=>window.electron.ipcRenderer.invoke(channel,input),{channel,input});
  const count=()=>page.evaluate(async()=> (await window.__host.all('apps')).length);
  const initial=await count();
  for(const [text,kind] of [['photo gallery','gallery'],['garden with sound and motion','garden'],['audio garden','garden']]){const result=await invoke('forge:discover',{text});assert.equal(result.ok,true,text);assert.equal(result.application.kind,kind);}
  for(const text of ['visual','gallery garden','gallery database','gallery without sound','gallery sound motion']){const result=await invoke('forge:discover',{text});assert.equal(result.ok,false,text);}
  assert.equal(await count(),initial);
  const proposal=await invoke('forge:discover',{text:'gallery'});
  const stale=await page.evaluate(async review=>{try{await window.electron.ipcRenderer.invoke('forge:discovery-create',{text:'photo gallery',name:'stale',review});return 'unexpected';}catch(error){return error.code;}},proposal.review);assert.equal(stale,'discoveryReviewChanged');assert.equal(await count(),initial);
  await page.goto(base+'settings');await page.waitForFunction(()=>!!window.__host&&!!window.electron);
  await page.locator('#foundry-application-discovery').click();let dialog=page.getByRole('dialog');await dialog.getByLabel('What would you like?').fill('gallery database');await dialog.getByRole('button',{name:'Review proposal',exact:true}).click();await page.waitForFunction(()=>document.querySelector('dialog [role=status]')?.textContent.includes('Unsupported words: database'));assert.equal(await dialog.getByRole('button',{name:'Create reviewed application'}).isDisabled(),true);
  await dialog.getByLabel('What would you like?').fill('garden with sound and motion');await dialog.getByRole('button',{name:'Review proposal',exact:true}).click();await dialog.getByRole('heading',{name:'Resonant garden'}).waitFor();await dialog.getByLabel('What would you like?').fill('photo gallery');assert.equal(await dialog.getByRole('button',{name:'Create reviewed application'}).isDisabled(),true);
  await dialog.getByRole('button',{name:'Review proposal',exact:true}).click();await dialog.getByRole('heading',{name:'Quiet gallery'}).waitFor();await dialog.getByLabel('Application name').fill('Discovery browser acceptance');await dialog.getByRole('button',{name:'Create reviewed application'}).click();await page.getByRole('heading',{name:'Open landscape'}).waitFor();assert.equal(await count(),initial+1);
  await page.getByRole('button',{name:'Next room',exact:true}).click();await page.getByRole('heading',{name:'Night sky'}).waitFor();await page.getByRole('button',{name:'Close',exact:true}).click();
  const app=await page.evaluate(async()=> (await window.__host.all('apps')).find(app=>app.name==='Discovery browser acceptance'));await page.reload();await page.waitForFunction(()=>!!window.electron&&!!window.__host);const persisted=await invoke('forge:interaction-run',{appId:app.id,event:0,reset:false,audioGranted:false,reducedMotion:true});assert.equal(persisted.output.screen,1);assert.equal(persisted.output.tone,0);
  assert.deepEqual(errors,[]);console.log(JSON.stringify({actualHost:true,admittedProfiles:3,refusedIntents:5,staleReviewNoWrite:true,nativeReviewInvalidation:true,atomicCreation:true,reloadState:true,pageExceptions:0}));

}finally{if(browser)await browser.close();await new Promise(resolve=>server.close(resolve));}
