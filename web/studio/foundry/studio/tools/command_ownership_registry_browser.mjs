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
    const {commandRegistry}=await import(new URL('command-registry.mjs',location.href));
    const inventory={handlers:[...window.__host.handlers.keys()],inputs:Object.keys(window.__inputContracts),outputs:Object.keys(window.__contracts)};
    commandRegistry.assertInventory(inventory);
    const call=(channel,input)=>window.electron.ipcRenderer.invoke(channel,input),check=(ok,message)=>{if(!ok)throw Error(message);};
    let effects=0,missing;window.__host.handlers.set('undeclared-command',()=>{effects++;return true;});
    try{await call('undeclared-command');}catch(error){missing=error.code;}finally{window.__host.handlers.delete('undeclared-command');}
    check(missing==='commandBindingMissing'&&effects===0,'undeclared handler executed');
    const original=window.__host.handlers.get('create-app');window.__host.handlers.delete('create-app');let changed;
    try{await call('create-app',{name:'missing-owner',initialChatMode:'build'});}catch(error){changed=error.code;}finally{window.__host.handlers.set('create-app',original);}
    check(changed==='commandBindingMismatch','removed owned handler became an accepted capability');
    window.__host.handlers.set('agent-tool:get-tools',()=>{effects++;return [];});let added;
    try{await call('agent-tool:get-tools');}catch(error){added=error.code;}finally{window.__host.handlers.delete('agent-tool:get-tools');}
    check(added==='commandBindingMismatch'&&effects===0,'unavailable capability acquired implicit ownership');
    let unsupported;try{await call('agent-tool:get-tools');}catch(error){unsupported=error.code;}
    check(unsupported==='unsupportedChannel','declared unavailable capability lost typed refusal');
    check(await call('get-system-platform')==='web','declared platform adapter failed');
    return {commands:commandRegistry.entries.length,handlers:inventory.handlers.length,inputs:inventory.inputs.length,outputs:inventory.outputs.length,undeclaredRefused:true,missingRefused:true,implicitOwnerRefused:true,unavailableRefused:true};
  });
  assert.deepEqual(errors,[]);console.log(JSON.stringify({...report,pageExceptions:errors.length}));
}finally{if(browser)await browser.close();await new Promise(resolve=>server.close(resolve));}
