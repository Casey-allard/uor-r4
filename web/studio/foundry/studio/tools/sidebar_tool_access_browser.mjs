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
  page.on("console",message=>{if(message.type()==='error'&&!message.text().includes('ERR_FAILED'))console.log('browser diagnostic:',message.text());});
  await context.route("**/*",route=>new URL(route.request().url()).origin===origin?route.continue():route.abort());
  await page.goto(base); await page.waitForFunction(() => !!window.__host && !!window.__prism && !!window.electron);
  const dock=page.getByRole('group',{name:'Foundry tools',exact:true});
  const pointer=async (locator,inSidebar=false)=>{
    for(let attempt=0;attempt<8;attempt++){
      const preview=page.getByRole('dialog',{name:'Review preview permissions',exact:true});
      if(await preview.count())await preview.getByRole('button',{name:"Don't allow",exact:true}).click();
      try{await locator.hover({timeout:500});if(await locator.evaluate(node=>{const r=node.getBoundingClientRect(),hit=document.elementFromPoint(r.left+r.width/2,r.top+r.height/2);return hit===node||node.contains(hit);}))break;}catch(error){if(error.name!=='TimeoutError'||attempt===7){console.log(await page.evaluate(()=>({dock:document.querySelector('#foundry-shell-actions')?.outerHTML,sidebars:[...document.querySelectorAll('[data-slot=sidebar-content]')].map(n=>({html:n.outerHTML.slice(-600),rect:n.getBoundingClientRect().toJSON(),ancestors:(()=>{const a=[];for(let p=n;p;p=p.parentElement)a.push([p.tagName,p.getAttribute('aria-hidden'),p.inert,p.dataset.slot]);return a;})()})),dialogs:[...document.querySelectorAll('dialog')].map(n=>n.outerHTML.slice(0,200))})));throw error;}}
    }
    assert.equal(await locator.evaluate(node=>{const r=node.getBoundingClientRect(),hit=document.elementFromPoint(r.left+r.width/2,r.top+r.height/2);return hit===node||node.contains(hit);}),true,'control pointer target is obstructed');
  };
  const activate=async(locator,inSidebar=false)=>{for(let attempt=0;attempt<8;attempt++){await pointer(locator,inSidebar);try{await locator.click({timeout:500});return;}catch(error){if(error.name!=='TimeoutError'||attempt===7)throw error;}}};
  let dialogCount=0;
  const checkTools=async()=>{
    // The tools are the last card of the Settings page, and are not in the sidebar on any page.
    const back=page.url();assert.equal(await page.locator('[data-slot=sidebar-content] #foundry-shell-actions').count(),0,'the tools must not be in the sidebar');
    await page.goto(base+'settings');
    await dock.waitFor();assert.equal(await dock.count(),1);assert.equal(await dock.getByRole('button').count(),3);
    assert.equal(await page.locator('div:has(> #general-settings) > #foundry-shell-actions').count(),1,'the tools must be a card of the Settings page');
    assert.equal(await page.locator('[data-slot=sidebar-content] #foundry-shell-actions').count(),0,'the tools must not be in the sidebar');
    for(const [label,title,close] of [['Preview permissions','Stored preview permissions','Close permissions'],['Interactive apps','Interactive applications','Close'],['Discover an app','Discover an application','Cancel']]){
      console.log('sidebar tool',label);const entry=dock.getByRole('button',{name:label,exact:true});await activate(entry,true);
      const dialog=page.getByRole('dialog',{name:title,exact:true});await dialog.waitFor();await activate(dialog.getByRole('button',{name:close,exact:true}));await dialog.waitFor({state:'hidden'});dialogCount++;
      await page.waitForFunction(()=>{const a=document.activeElement;if(!a?.matches('#foundry-shell-actions button'))return false;const r=a.getBoundingClientRect();return r.width>0&&r.height>0&&r.right>0&&r.left<innerWidth;},null,{timeout:2000});
    }
    if(back!==page.url())await page.goto(back);
  };
  await checkTools();
  const made=await page.evaluate(()=>window.electron.ipcRenderer.invoke('create-app',{name:'sidebar-access',initialChatMode:'build'}));
  await page.goto(base+'chat?id='+made.chatId+'&appId='+made.app.id);
  // An app made in Foundry is projected: its preview is the live frame on this origin, and nothing asks to reach the internet.
  await page.getByTestId('chat-input-container').waitFor();await page.waitForTimeout(1500);assert.equal(await page.getByRole('dialog',{name:'Review preview permissions',exact:true}).count(),0,'a new app asks nothing');
  const auxiliary=page.getByTestId('auxiliary-actions-menu');await activate(auxiliary);await page.getByRole('menuitem',{name:'Themes',exact:true}).hover();await page.getByTestId('theme-option-default').click();
  assert.equal(await page.evaluate(id=>window.electron.ipcRenderer.invoke('get-app-theme',{appId:id}),made.app.id),'default');
  await checkTools();
  for(const width of [390,768,1280,390]){
    console.log("sidebar viewport",width);await page.setViewportSize({width,height:844});
    if(width<768){const toggle=page.locator('[data-slot=sidebar-trigger]:visible');await activate(toggle);await page.locator('[data-mobile=true]').waitFor();}
    await checkTools();
    if(width<768&&await page.locator('[data-mobile=true]').count()){await page.keyboard.press('Escape');await page.locator('[data-mobile=true]').waitFor({state:'hidden'});}
    await activate(auxiliary);await page.getByRole('menuitem',{name:'Themes',exact:true}).waitFor();await page.keyboard.press('Escape');
  }
  assert.equal(dialogCount,18);assert.deepEqual(errors,[]);console.log(JSON.stringify({actualRenderer:true,desktopPointer:true,mobilePointer:true,responsiveLifetimes:4,originalDialogs:dialogCount,duplicateEntries:0,pageExceptions:0}));
}finally{if(browser)await browser.close();await new Promise(resolve=>server.close(resolve));}
