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
 browser=await chromium.launch({executablePath,headless:true,args:['--no-sandbox']});
 const context=await browser.newContext({serviceWorkers:'block'}),page=await context.newPage(),peer=await context.newPage(),errors=[];
 for(const p of [page,peer])p.on('pageerror',e=>errors.push(e.message));
 await context.route('**/*',r=>new URL(r.request().url()).origin===origin?r.continue():r.abort());
 const initialize=async p=>{await p.goto(base);await p.waitForFunction(()=>!!window.__host&&!!window.electron);await p.evaluate(async base=>{window.gallery=await import(base+'gallery-store.mjs');window.sourceModule=await import(base+'project-edit.mjs');window.database=await (await import(base+'inference.js')).workspace();},base);};
 await initialize(page);await initialize(peer);
 const call=(channel,input)=>page.evaluate(({channel,input})=>window.electron.ipcRenderer.invoke(channel,input),{channel,input});
 const make=async name=>(await call('create-app',{name,initialChatMode:'build'})).app.id;
 const id=await make('source-bound-gallery');
 const card={description:'Make images smaller without losing quality by comparing formats side by side as you go.',tag:'Creative'};
 const prepare=async(p,id)=>p.evaluate(async({id,card})=>{window.galleryId=id;window.gallerySource=await window.sourceModule.captureProjectSource(window.database,id);const c=document.createElement('canvas');c.width=20;c.height=10;const g=c.getContext('2d');g.fillStyle='#236abc';g.fillRect(0,0,20,10);window.galleryPicture=await new Promise(r=>c.toBlob(r,'image/webp'));window.galleryCard=card;},{id,card});
 await prepare(page,id);await prepare(peer,id);
 await Promise.all([page.evaluate(()=>gallery.publishGallery(database,galleryId,gallerySource,{picture:galleryPicture,version:'first'})),peer.evaluate(()=>gallery.publishGallery(database,galleryId,gallerySource,{card:galleryCard}))]);
 const both=await page.evaluate(async()=>{const r=await gallery.readGallery(database,galleryId);return {picture:r.picture instanceof Blob,card:r.card,revision:r.revision};});assert.equal(both.picture,true);assert.equal(both.revision,2);assert.equal(both.card.tag,'Creative');
 const abort=await page.evaluate(async()=>{const put=IDBObjectStore.prototype.put;let injected=false;IDBObjectStore.prototype.put=function(value,key){const r=put.call(this,value,key);if(this.name==='refs'&&key===`gallery:${galleryId}`){injected=true;r.addEventListener('success',()=>this.transaction.abort(),{once:true});}return r;};try{await gallery.publishGallery(database,galleryId,gallerySource,{card:null});return {injected,code:'accepted'};}catch(e){return {injected,code:e.code,message:e.message};}finally{IDBObjectStore.prototype.put=put;}});assert.deepEqual(abort,{injected:true,code:'galleryStorageFailed',message:'galleryStorageFailed'});assert.equal(await page.evaluate(async()=>(await gallery.readGallery(database,galleryId)).card.tag),'Creative');
 const captured=await page.evaluate(async()=>{let ready;const pending=new Promise(r=>ready=r),source=structuredClone(gallerySource),card=structuredClone(galleryCard);const publication=gallery.publishGallery(pending,galleryId,source,{card});source.files[0][1]='MUTATED_AFTER_CALL';card.tag='Tools';ready(database);await publication;return (await gallery.readGallery(database,galleryId)).card.tag;});assert.equal(captured,'Creative');
 await call('edit-app-file',{appId:id,filePath:'notes.txt',content:'new source'});
 assert.equal(await page.evaluate(()=>gallery.readGallery(database,galleryId)),null);
 const stale=await page.evaluate(async()=>{try{await gallery.publishGallery(database,galleryId,gallerySource,{card:galleryCard});return 'accepted';}catch(e){return e.code;}});assert.equal(stale,'gallerySourceChanged');
 await prepare(page,id);await page.evaluate(()=>gallery.publishGallery(database,galleryId,gallerySource,{card:galleryCard}));
 assert.equal(await page.evaluate(async()=>(await gallery.readGallery(database,galleryId)).picture),null);
 await page.evaluate(()=>gallery.publishGallery(database,galleryId,gallerySource,{picture:galleryPicture,version:'new'}));
 // Home defaults to Featured on every mount; own media belongs to the explicit My apps view.
 const selectMyApps=async()=>{
  const floor=page.getByTestId('featured-app-showcase');
  await floor.getByRole('button',{name:'My apps',exact:true}).click();
  const tile=floor.getByTestId('app-showcase-card-source-bound-gallery');
  await tile.waitFor();return tile;
 };
 // Home shows only a picture and owner, and the supported Apps route is a table. Card text is
 // checked through the real renderer adapter here, not claimed as visible description coverage.
 const rendererCard=()=>page.evaluate(async id=>(await window.__gallery.read(id))?.card??null,id);
 await page.goto(base);await page.waitForFunction(()=>!!window.electron&&!!window.__gallery);
 const tile=await selectMyApps();await tile.scrollIntoViewIfNeeded();await tile.locator('img').waitFor();
 assert.equal(await tile.locator('img').evaluate(async image=>{await image.decode();return image.naturalWidth;}),20);
 assert.deepEqual(await rendererCard(),card);
 await call('edit-app-file',{appId:id,filePath:'notes.txt',content:'another source'});await tile.locator('img').waitFor({state:'hidden'});
 assert.equal(await rendererCard(),null);
 // A forged legacy cache entry never restores a current gallery image or card.
 await page.evaluate(async({base,id})=>{const cache=await caches.open('covers');await cache.put(new URL('covers/'+id+'.json',base),new Response(JSON.stringify({description:'UNBOUND_LEGACY_DESCRIPTION',tag:'Tools'})));}, {base,id});
 await page.reload();await selectMyApps();assert.ok(!(await tile.innerText()).includes('UNBOUND_LEGACY_DESCRIPTION'));assert.equal(await tile.locator('img').count(),0);
 assert.equal(await rendererCard(),null);
 await initialize(page);await prepare(page,id);
 const corruption=await page.evaluate(async()=>{await gallery.publishGallery(database,galleryId,gallerySource,{card:galleryCard});const before=await window.__host.get('refs',`gallery:${galleryId}`);await window.__host.put('refs',`gallery:${galleryId}`,{...before,picture:'PRIVATE_CORRUPTION'});let code;try{await gallery.readGallery(database,galleryId);}catch(e){code={code:e.code,message:e.message};}await window.__host.put('refs',`gallery:${galleryId}`,before);return code;});assert.deepEqual(corruption,{code:'galleryStateInvalid',message:'galleryStateInvalid'});
 assert.deepEqual(await rendererCard(),card);
 await page.reload();await selectMyApps();assert.equal(await tile.locator('img').count(),0);
 assert.deepEqual(await rendererCard(),card);
 // Restore the module bindings for the storage/deletion checks after the renderer reload.
 await initialize(page);await prepare(page,id);
 await call('delete-app',{appId:id});
 const deleted=await page.evaluate(async()=>{let code;try{await gallery.publishGallery(database,galleryId,gallerySource,{picture:galleryPicture,version:'late'});}catch(e){code=e.code;}return {code,record:await window.__host.get('refs',`gallery:${galleryId}`),read:await gallery.readGallery(database,galleryId)};});assert.deepEqual(deleted,{code:'gallerySourceChanged',record:undefined,read:null});
 assert.deepEqual(errors,[]);console.log(JSON.stringify({actualHost:true,actualIndexedDB:true,twoWindows:true,atomicFieldMerge:true,lateAbort:true,sourceChangeRefused:true,inputCaptured:true,staleFieldsDiscarded:true,rendererCheckedMedia:true,rendererCardAdapterFreshness:true,visibleDescriptionChecked:false,legacyCacheIgnored:true,corruptionRefused:true,deletionPreventsResurrection:true,pageExceptions:0}));
}finally{if(browser)await browser.close();await new Promise(r=>server.close(r));}
