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
  const report=await page.evaluate(async()=>{
    const check=(value,message)=>{if(!value)throw Error(message);},call=(channel,input)=>window.electron.ipcRenderer.invoke(channel,input);
    const made=await call('create-app',{name:'version-history',initialChatMode:'build'}),appId=made.app.id;
    await call('edit-app-file',{appId,filePath:'history-note.txt',content:'First version 😀'});
    await call('edit-app-file',{appId,filePath:'history-note.txt',content:'Second version\r\n'});
    const versions=await call('list-versions',{appId});check(versions.length>=3&&versions.every(v=>/^[0-9a-f]{64}$/.test(v.oid)),'renderer aliases missing');const versionId=versions[0].oid;
    const canonical=await window.__host.get('refs',`${appId}:main`);check(canonical==='blake3:'+versionId,'canonical identity changed');
    await Promise.all([call('set-version-favorite',{appId,versionId,isFavorite:true}),call('set-version-note',{appId,versionId,note:'A retained note 😀'})]);
    const updated=(await call('list-versions',{appId}))[0];check(updated.isFavorite&&updated.note==='A retained note 😀','concurrent metadata lost');
    const delta=await call('get-version-changes',{appId,versionId});const file=delta.find(x=>x.path==='history-note.txt');check(file?.type==='modified'&&file.oldContent==='First version 😀'&&file.newContent==='Second version\r\n','verified diff differs');
    let refused=0;try{await call('set-version-note',{appId,versionId,note:'x'.repeat(10001)});}catch{refused++;}
    try{await call('set-version-favorite',{appId,versionId:'f'.repeat(64),isFavorite:true});}catch{refused++;}
    const before=JSON.stringify(await window.__host.get('versions',appId)),originalPut=IDBObjectStore.prototype.put;
    IDBObjectStore.prototype.put=function(...args){const request=originalPut.apply(this,args);if(this.name==='versions')request.addEventListener('success',()=>this.transaction.abort(),{once:true});return request;};
    try{await call('set-version-note',{appId,versionId,note:'Must abort'});}catch{refused++;}finally{IDBObjectStore.prototype.put=originalPut;}
    check(JSON.stringify(await window.__host.get('versions',appId))===before,'late abort changed durable metadata');
    const body=await window.__host.get('objects',canonical),snapshot=JSON.parse(new TextDecoder().decode(body)),key=snapshot.entries.find(e=>e[0]==='history-note.txt')[1],bytes=await window.__host.get('objects',key);
    await window.__host.put('objects',key,new TextEncoder().encode('corrupted'));
    try{await call('get-version-changes',{appId,versionId});}catch{refused++;}finally{await window.__host.put('objects',key,bytes);}
    check(refused===4,'negative controls did not all refuse');
    return {appId,versionId,canonical,versions:versions.length,refused};
  });
  await page.reload();await page.waitForFunction(()=>!!window.__host&&!!window.electron);
  await page.evaluate(async report=>{
    const call=(channel,input)=>window.electron.ipcRenderer.invoke(channel,input),rows=await call('list-versions',{appId:report.appId});
    if(!rows[0].isFavorite||rows[0].note!=='A retained note 😀')throw Error('metadata did not survive reload');
    const changes=await call('get-version-changes',{appId:report.appId,versionId:report.versionId});if(!changes.some(x=>x.path==='history-note.txt'))throw Error('restored bytes failed verification');
    await call('delete-app',{appId:report.appId});let missing=false;try{await call('set-version-note',{appId:report.appId,versionId:report.versionId,note:'Resurrect'});}catch{missing=true;}
    if(!missing||await window.__host.get('versions',report.appId)!==undefined)throw Error('deleted history resurrected');
  },report);
  assert.deepEqual(errors,[]);console.log(JSON.stringify({actualHost:true,actualCore:true,actualIndexedDB:true,...report,concurrentMetadata:true,lateAbort:true,corruptObjectRefused:true,reload:true,deletedHistoryRefused:true,pageExceptions:0}));

}finally{if(browser)await browser.close();await new Promise(resolve=>server.close(resolve));}
