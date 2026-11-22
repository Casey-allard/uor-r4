import assert from 'node:assert/strict';
import fs from 'node:fs';
import vm from 'node:vm';
import {test} from 'node:test';
const read=p=>fs.readFileSync(new URL('../../'+p,import.meta.url),'utf8');
const source=read('studio/core/src/bin/project_shell.rs');
const script=source.match(/<script>\n\/\/ Vite identifies[\s\S]*?<\/script>/)?.[0].replace(/^<script>|<\/script>$/g,'').replace(/\{\{/g,'{').replace(/\}\}/g,'}');
test('stylesheet normalization preserves the resolved resource at root nested and encoded deployment paths',()=>{
 assert.ok(script,'projector must emit normalization');
 for(const base of ['https://example.test/','https://example.test/foundry/','https://example.test/team/app%20space/']){
  const resolved=new URL('assets/index.css',base),attributes={};
  vm.runInNewContext(script,{URL,document:{baseURI:base,getElementById:id=>{assert.equal(id,'renderer-style');return {href:resolved.href,setAttribute:(name,value)=>attributes[name]=value};}}});
  assert.equal(attributes.href,resolved.pathname);assert.equal(new URL(attributes.href,base).href,resolved.href);
 }
});
test('initial stylesheet identity precedes the unchanged host and renderer module boundary',()=>{
 assert.ok(source.includes('<link id="renderer-style" rel="stylesheet" crossorigin>'));
 assert.ok(source.indexOf('const style = document.getElementById("renderer-style")')<source.indexOf('<script type="module" src="host.bundle.js"></script>'));
 assert.ok(source.indexOf('<script type="module" src="host.bundle.js"></script>')<source.indexOf('<script type="module" src="assets/index.js"></script>'));
 assert.ok(!source.includes('async src="assets/index.js"'));
 assert.ok(read('app/src/FoundrySpecification.lex.tex').includes('name: rendererStylesheetBoundary,'));
});
test('mandatory acceptance observes duplicate CSS requests and actual styled renderer navigation',async()=>{
 const workflow=read('.github/workflows/studio.yml'),browser=read('studio/tools/renderer_stylesheet_identity_browser.mjs');
 assert.ok(workflow.includes('node tools/renderer_stylesheet_identity_browser.mjs'));
 for(const required of ['stylesheetRequests,1','getComputedStyle','Build your ideas with Prism','/settings','serviceWorkers: "block"'])assert.ok(browser.includes(required),required);
 assert.equal((browser.match(/assert\.equal\(stylesheetRequests,1,/g)||[]).length,2,'both Home and Settings must refuse duplicate stylesheet loads');
 assert.ok(browser.includes("assert.equal(await tab.locator('body').evaluate(el=>getComputedStyle(el).fontFamily.includes('Geist')),true)"));
 // Execute the browser oracle itself with controlled computed styles. This checks its assertions,
 // not the renderer; only the actual-browser command qualifies rendered CSS and network identity.
 const start=browser.indexOf('  const headlineSizes = [];'),end=browser.indexOf("  assert.equal(await tab.locator('link[rel=stylesheet]",start);
 assert.ok(start>0&&end>start,'the mandatory responsive heading oracle must precede stylesheet and Settings checks');
 const oracle=browser.slice(start,end);
 const expected=[[390,'34px'],[809,'34px'],[810,'34.02px'],[1280,'53.76px'],[1309,'54.978px'],[1310,'55px'],[1440,'55px']];
 const exercise=async computed=>{
  let width;const visited=[];
  const element={};
  const tab={
   async setViewportSize(size){width=size.width;assert.equal(size.height,900);},
   locator(selector){assert.equal(selector,'[data-slot="home-stage"] h1');return {async evaluate(callback){visited.push(width);return callback(element);}};},
  };
  const sizes=await vm.runInNewContext(`(async()=>{${oracle};return headlineSizes;})()`,{assert,tab,getComputedStyle:el=>{assert.equal(el,element);return {fontSize:computed(width)};}});
  return {sizes:JSON.parse(JSON.stringify(sizes)),visited,width};
 };
 const correct=width=>new Map(expected).get(width);
 const actual=await exercise(correct);
 assert.deepEqual(actual.visited,expected.map(([w])=>w),'no width or clamp transition may be omitted');
 assert.deepEqual(actual.sizes,expected.map(([width,fontSize])=>({width,fontSize})));
 assert.equal(actual.width,1280,'restore desktop navigation before Settings');
 // Only the extracted browser assertion may reject a bad style, not the harness's report checks.
 for(const [width] of expected)await assert.rejects(exercise(w=>w===width?'48px':correct(w)),{code:'ERR_ASSERTION'},`the oracle must reject a wrong size at ${width}px`);
});
