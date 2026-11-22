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
    const call=(c,p)=>window.electron.ipcRenderer.invoke(c,p),made=await call('forge:interaction-create',{name:'Working tree',kind:'garden'}),appId=made.app.id;
    await call('edit-app-file',{appId,filePath:'working.txt',content:'original\nkeep\nold\n'});
    if((await call('git:get-uncommitted-files',{appId})).length)throw Error('sealed source is dirty');
    await window.__host.put('files',appId+':working.txt','changed\nkeep\nnew\n');await window.__host.put('files',appId+':empty.txt','');
    const changes=await call('git:get-uncommitted-files',{appId});const modified=changes.find(x=>x.path==='working.txt'),added=changes.find(x=>x.path==='empty.txt');
    if(modified?.status!=='modified'||modified.additions!==2||modified.deletions!==2||added?.status!=='added')throw Error('changes lost');
    const diff=await call('git:get-uncommitted-file-diff',{appId,filePath:'working.txt'});if(diff.oldContent!=='original\nkeep\nold\n'||diff.newContent!=='changed\nkeep\nnew\n')throw Error('diff bytes wrong');
    const reference=await window.__host.get('refs',appId+':main'),original=await window.__host.get('objects',reference);await window.__host.put('objects',reference,new TextEncoder().encode('{}'));let refused=0;
    for(const channel of ['git:get-uncommitted-files','git:get-uncommitted-file-diff'])try{await call(channel,{appId,...(channel.endsWith('file-diff')?{filePath:'working.txt'}:{})});}catch{refused++;}
    if(refused!==2)throw Error('corruption became successful diff');await window.__host.put('objects',reference,original);
    if((await call('git:get-uncommitted-files',{appId})).length!==changes.length)throw Error('recovery lost source');
    let invalid=false;try{await call('git:get-uncommitted-file-diff',{appId,filePath:'../escape'});}catch{invalid=true;}if(!invalid)throw Error('invalid path admitted');
    return {sealedClean:true,changedSource:true,emptyFile:true,lineCounts:true,exactText:true,corruptBasisRefusals:refused,recovery:true};
  });
  assert.deepEqual(errors,[]);console.log(JSON.stringify({actualHost:true,actualCore:true,actualIndexedDB:true,...report,pageExceptions:0}));
}finally{if(browser)await browser.close();await new Promise(resolve=>server.close(resolve));}
