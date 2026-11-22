// Actual projected initializer and locked loader/editor in Chromium; no substituted loader.
import { isPrismIdRelay } from "./decorative_image_fixture.mjs";
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import http from 'node:http';
import {fileURLToPath} from 'node:url';
import {build} from '../vendor/prism/node_modules/esbuild/lib/main.js';
import {chromium} from '../vendor/prism/node_modules/playwright-core/index.mjs';
import {screenSource} from '../vendor/prism/hologram.screens.mjs';
import {editorLoaderSource} from '../vendor/prism/hologram.editor-loader.mjs';
const vendor=fileURLToPath(new URL('../vendor/prism/',import.meta.url)),shell=fileURLToPath(new URL('../shell/',import.meta.url));
assert.equal(JSON.parse(fs.readFileSync(path.join(vendor,'node_modules/monaco-editor/package.json'))).version,'0.52.2');
assert.ok(fs.existsSync(path.join(shell,'editor/vs/loader.js')),'Stage the verified editor first');
const file=path.join(vendor,'src/components/chat/monaco.ts');
const projected=screenSource(file,fs.readFileSync(file,'utf8')).replace('if (!IS_VITEST) {','const rawInit = loader.init.bind(loader);\nif (!IS_VITEST) {');
const bundle=await build({stdin:{contents:projected+'\nloader.config({paths:{vs:new URL("/editor/vs",location.origin).href}}); export {loader,rawInit};',resolveDir:path.dirname(file),loader:'ts'},bundle:true,write:false,platform:'browser',format:'esm',target:'es2022',logLevel:'silent',plugins:[{name:'owned-editor-loader',setup(build){build.onLoad({filter:/makeCancelable\.js$/},args=>{const source=fs.readFileSync(args.path,'utf8'),code=editorLoaderSource(args.path,source);return code===null?undefined:{contents:code,loader:'js',resolveDir:path.dirname(args.path)};});}}]});
const server=http.createServer((req,res)=>{
 const pathname=new URL(req.url,'http://localhost').pathname;
 if(pathname==='/'){res.setHeader('content-type','text/html');res.end('<!doctype html><title>Editor lifecycle acceptance</title>');return;}
 if(pathname==='/fixture.mjs'){res.setHeader('content-type','text/javascript');res.end(bundle.outputFiles[0].contents);return;}
 if(!pathname.startsWith('/editor/vs/')||pathname.includes('..')){res.writeHead(404);res.end();return;}
 const file=path.join(shell,pathname.slice(1));if(!fs.existsSync(file)){res.writeHead(404);res.end();return;}
 res.setHeader('content-type',file.endsWith('.css')?'text/css':'text/javascript');res.end(fs.readFileSync(file));
});
await new Promise(resolve=>server.listen(0,'127.0.0.1',resolve));const origin=`http://127.0.0.1:${server.address().port}`;
const executablePath=process.env.CHROME_BIN||['/usr/bin/chromium','/usr/bin/google-chrome'].find(p=>fs.existsSync(p));let browser;
try{
 browser=await chromium.launch({executablePath,headless:true,args:['--no-sandbox']});
 for(const scenario of ['cancel','themeFailure','loadFailure']){
  const context=await browser.newContext(),page=await context.newPage(),errors=[],external=[];page.on('pageerror',error=>errors.push(error.message));
  await context.route('**/*',route=>{const url=new URL(route.request().url());if(url.origin!==origin){external.push(url.href);return route.abort();}if(scenario==='loadFailure'&&url.pathname==='/editor/vs/loader.js')return route.abort();return route.continue();});
  await page.goto(origin);await page.evaluate(async()=>{window.fixture=await import('/fixture.mjs');});
  assert.equal(await page.evaluate(()=>!!window.monaco),false,'initializer must remain lazy');
  const result=await page.evaluate(async scenario=>{
   const {loader,rawInit}=window.fixture;
   if(scenario==='loadFailure'){const error=await loader.init().catch(e=>e);return {code:error.code,message:error.message};}
   if(scenario==='themeFailure'){
    const monaco=await rawInit(),define=monaco.editor.defineTheme;monaco.editor.defineTheme=()=>{throw Error('private theme fixture');};
    const error=await loader.init().catch(e=>e);monaco.editor.defineTheme=define;const restored=await loader.init();
    const node=document.createElement('div');node.style.cssText='width:200px;height:100px';document.body.append(node);window.editor=restored.editor.create(node,{value:'ready',language:'plaintext',theme:'prism-dark'});
    return {code:error.code,message:error.message,background:getComputedStyle(node.querySelector('.monaco-editor')).backgroundColor};
   }
   const first=loader.init(),observed=first.catch(e=>e);first.cancel();const second=loader.init();const third=loader.init();const [cancelled,monaco,other]=await Promise.all([observed,second,third]);
   const cached=loader.init(),cachedResult=cached.catch(e=>e);cached.cancel();const replacement=await loader.init();
   const node=document.createElement('div');node.style.cssText='width:200px;height:100px';document.body.append(node);window.editor=monaco.editor.create(node,{value:'ready',language:'plaintext',theme:'prism-dark'});
   return {cancelled:cancelled.type,cachedCancelled:(await cachedResult).type,same:monaco===other&&monaco===replacement,background:getComputedStyle(node.querySelector('.monaco-editor')).backgroundColor};
  },scenario);
  if(scenario==='cancel')assert.deepEqual(result,{cancelled:'cancelation',cachedCancelled:'cancelation',same:true,background:'rgb(19, 19, 22)'});
  else if(scenario==='themeFailure')assert.deepEqual(result,{code:'editorThemeFailed',message:'The browser editor could not be configured',background:'rgb(19, 19, 22)'});
  else assert.deepEqual(result,{code:'editorLoadFailed',message:'The browser editor could not load'});
  await page.waitForTimeout(100);assert.deepEqual(errors,[]);assert.deepEqual(external.filter(url=>!isPrismIdRelay(url)),[]);console.log(JSON.stringify({scenario,...result,pageExceptions:0,externalRequests:0}));await context.close();
 }
}finally{if(browser)await browser.close();await new Promise(resolve=>server.close(resolve));}
