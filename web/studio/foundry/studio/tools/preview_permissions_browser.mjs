// Actual preview permission, compilation, persistent revocation and source-race acceptance.
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
  const appId = await page.evaluate(async()=>{
    const made=await window.electron.ipcRenderer.invoke("create-app",{name:"preview-permission",initialChatMode:"build"});return made.app.id;
  });
  const start = () => page.evaluate(id=>{window.previewOutcome=null;window.__preview(id).then(value=>window.previewOutcome={value},error=>window.previewOutcome={code:error.code,message:error.message});},appId);
  const outcome = async()=>{await page.waitForFunction(()=>window.previewOutcome!==null);return page.evaluate(()=>window.previewOutcome);};
  await start();
  const dialog=page.getByRole("dialog",{name:"Review preview permissions"});await dialog.waitFor();
  assert.equal(await page.evaluate(()=>document.activeElement.textContent),"Don't allow");
  assert.ok((await dialog.textContent()).includes("https://esm.sh"));
  await dialog.getByRole("button",{name:"Don't allow"}).click();assert.equal((await outcome()).code,"previewPermissionDenied");
  assert.equal(await page.evaluate(async()=> (await window.__host.all("refs")).filter(r=>r?.previewConsent).length),0);
  await start();await page.getByRole("button",{name:"Allow this preview",exact:true}).click();
  const allowed=await outcome();assert.ok(allowed.value?.appUrl,JSON.stringify(allowed));
  const previewId=new URL(allowed.value.appUrl).pathname.split("/").filter(Boolean).pop();
  const initial=await page.evaluate(id=>window.__host.get("refs","capabilities:"+id),previewId);
  assert.deepEqual(initial.network.endpoints,["https://esm.sh","https://cdn.tailwindcss.com"]);assert.equal(initial.previewConsent.id,previewId);
  // The tools are on the Settings page.
  await page.goto(base+"settings");await page.waitForFunction(()=>!!window.__host&&!!window.__preview);
  await page.getByRole("button",{name:"Preview permissions",exact:true}).click();
  const permissions=page.getByRole("dialog",{name:"Stored preview permissions"});await permissions.getByRole("button",{name:"Revoke preview"}).waitFor();
  await page.evaluate(url=>{const frame=document.createElement("iframe");frame.id="permission-test-frame";frame.src=url;document.body.append(frame);},allowed.value.appUrl);
  await permissions.getByRole("button",{name:"Revoke preview"}).click();await permissions.getByText("Preview permission revoked",{exact:true}).waitFor();
  assert.equal(await page.locator("#permission-test-frame").getAttribute("src"),"about:blank");
  const revoked=await page.evaluate(id=>window.__host.get("refs","capabilities:"+id),previewId);assert.equal(revoked.serve,false);assert.deepEqual(revoked.network.endpoints,[]);
  const stale=await page.evaluate(async({id,record})=>{
    const {commitPreviewPermission}=await import(new URL("preview-permissions.mjs",location.href));const {workspace:db}=await import(new URL("inference.js",location.href));
    try {await commitPreviewPermission(await db(),id,{disclosure:{id,source:record.previewConsent.source,endpoints:record.network.endpoints},expires:Date.now()+10000},record);return "accepted";}catch(error){return error.code;}
  },{id:previewId,record:initial});assert.equal(stale,"previewPermissionChanged");
  await permissions.getByRole("button",{name:"Close permissions"}).click();
  await context.setOffline(true);await start();assert.equal((await outcome()).code,"previewPermissionOffline");await context.setOffline(false);
  await start();await dialog.waitFor();
  await page.evaluate(id=>window.electron.ipcRenderer.invoke("edit-app-file",{appId:id,filePath:"notes.txt",content:"changed during approval"}),appId);
  await dialog.getByRole("button",{name:"Allow this preview",exact:true}).click();assert.equal((await outcome()).code,"buildSourceChanged");
  const report={actualPreview:true,deniedNoGrant:true,approvedDestinations:2,reloadRevocation:true,frameUnloaded:true,staleApprovalRefused:true,offlineRefused:true,sourceRaceRefused:true};
  assert.deepEqual(errors, []); console.log(JSON.stringify({...report,pageExceptions:errors.length}));
} finally { if (browser) await browser.close(); await new Promise(resolve => server.close(resolve)); }
