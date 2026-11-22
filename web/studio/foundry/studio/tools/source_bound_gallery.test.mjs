import assert from 'node:assert/strict';
import {test} from 'node:test';
import fs from 'node:fs';
import vm from 'node:vm';
import {captureGalleryCommand,publishGallery,readGallery} from '../shell/gallery-store.mjs';
const source=()=>({name:'Gallery',reference:'blake3:'+'a'.repeat(64),files:[['src/a','one']],dependencies:[]});
const card={description:'Make images smaller without losing quality by comparing formats side by side as you go.',tag:'Creative'};
test('gallery commands detach source files dependencies and card before readiness',()=>{
 const s=source(),p={card:{...card}},c=captureGalleryCommand(1,s,p);s.files[0][1]='changed';s.dependencies.push('other');p.card.description='changed';assert.equal(c.source.files[0][1],'one');assert.deepEqual(c.source.dependencies,[]);assert.deepEqual(c.patch.card,card);
});
test('gallery refuses malformed ownership picture bounds and unqualified card values',()=>{
 for(const [id,s,p] of [[0,source(),{card}],[1,{...source(),reference:'private'}, {card}],[1,{...source(),files:[['x','one'],['x','two']]},{card}],[1,source(),{picture:new Blob([],{type:'image/webp'}),version:'v'}],[1,source(),{picture:new Blob(['x'],{type:'image/png'}),version:'v'}],[1,source(),{picture:new Blob([new Uint8Array(8*1024*1024+1)],{type:'image/webp'}),version:'v'}],[1,source(),{card:{description:'unqualified',tag:'Other'}}],[1,source(),{unknown:true}],[1,source(),{version:'v'}]])assert.throws(()=>captureGalleryCommand(id,s,p),{code:'invalidGalleryCommand',message:'invalidGalleryCommand'});
});
test('gallery public storage failures do not reveal database errors or private contents',async()=>{
 const db={transaction(){throw Error('PRIVATE_DATABASE_DETAIL');}};
 for(const operation of [()=>publishGallery(db,1,source(),{card}),()=>readGallery(db,1),()=>publishGallery(Promise.reject(Error('PRIVATE')),1,source(),{card}),()=>readGallery(Promise.reject(Error('PRIVATE')),1)])await assert.rejects(operation,{code:'galleryStorageFailed',message:'galleryStorageFailed'});
});
test('gallery consumers and deletion use the qualified source-bound storage boundary',async()=>{
 const read=p=>fs.readFileSync(new URL(p,import.meta.url),'utf8'),host=read('../shell/host.js'),renderer=read('../vendor/prism/hologram.screens/useAppThumbnails.ts');
 assert.ok(host.includes('publishGallery(dbReady, appId, source, {picture:'));assert.ok(host.includes('coverApp(appId, appUrl, await source)'));assert.ok(!host.includes('covers.saveCover(')&&!host.includes('covers.saveCard('));assert.ok(renderer.includes('window.__gallery.read(appId)'));assert.ok(!renderer.includes('caches.open(')&&!renderer.includes('inference.js'));assert.ok(host.includes('Object.freeze({read: appId => readGallery(dbReady, appId)})'));assert.ok(renderer.includes('URL.revokeObjectURL'));assert.ok(read('../shell/project-lifecycle.mjs').includes('refs.delete(`gallery:${appId}`)'));assert.ok(read('../../.github/workflows/studio.yml').includes('node tools/source_bound_gallery_browser.mjs'));
 const browser=read('./source_bound_gallery_browser.mjs');
 // Exercise the same view selector the actual-browser gate uses; a page-wide tile lookup or an
 // omitted My apps click must fail instead of accidentally exercising Featured or another route.
 const selector=browser.match(/ const selectMyApps=async\(\)=>\{[\s\S]*?\n \};/)?.[0];
 assert.ok(selector,'the browser gate must explicitly select its own-app view');
 const events=[],tile={async waitFor(){events.push('wait');}},floor={
  getByRole(role,options){assert.equal(role,'button');assert.equal(options.name,'My apps');assert.equal(options.exact,true);return {async click(){events.push('mine');}};},
  getByTestId(id){assert.equal(id,'app-showcase-card-source-bound-gallery');events.push('tile');return tile;},
 };
 const page={getByTestId(id){assert.equal(id,'featured-app-showcase');events.push('floor');return floor;}};
 assert.equal(await vm.runInNewContext(`(async()=>{${selector};return await selectMyApps();})()`,{page}),tile);
 assert.deepEqual(events,['floor','mine','tile','wait']);
 assert.ok(browser.includes('const tile=await selectMyApps();await tile.scrollIntoViewIfNeeded();'));
 assert.equal((browser.match(/await page\.reload\(\);await selectMyApps\(\);/g)||[]).length,2,'every renderer reload must reselect My apps');
 const adapter=browser.match(/ const rendererCard=\(\)=>[^\n]+/)?.[0];assert.ok(adapter);
 for(const record of [null,{card},{card:null}]){
  const seen=[],id=19,window={__gallery:{async read(appId){seen.push(appId);return record;}}};
  const page={async evaluate(callback,appId){assert.equal(appId,id);return callback(appId);}};
  assert.deepEqual(await vm.runInNewContext(`(async()=>{${adapter};return await rendererCard();})()`,{page,window,id}),record?.card??null);
  assert.deepEqual(seen,[id],'card assertions must use the real read-only renderer adapter');
 }
 assert.equal((browser.match(/assert\.deepEqual\(await rendererCard\(\),card\);/g)||[]).length,3,'fresh description and tag must survive renderer reload and legacy cache');
 assert.equal((browser.match(/assert\.equal\(await rendererCard\(\),null\);/g)||[]).length,2,'source edits and legacy cache must not revive card text');
 for(const required of ["image.decode()", "return image.naturalWidth;}),20)", "await tile.locator('img').waitFor({state:'hidden'})", "UNBOUND_LEGACY_DESCRIPTION", "assert.deepEqual(abort,{injected:true,code:'galleryStorageFailed'", "assert.equal(captured,'Creative')", "assert.equal(stale,'gallerySourceChanged')", "assert.deepEqual(corruption,{code:'galleryStateInvalid'", "assert.deepEqual(deleted,{code:'gallerySourceChanged',record:undefined,read:null})", "visibleDescriptionChecked:false"])assert.ok(browser.includes(required),required);
});
