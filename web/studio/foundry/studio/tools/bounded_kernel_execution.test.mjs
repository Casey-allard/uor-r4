import assert from 'node:assert/strict';
import fs from 'node:fs';
import vm from 'node:vm';
import {webcrypto} from 'node:crypto';
import {test} from 'node:test';
const source=fs.readFileSync(new URL('../shell/kernel.js',import.meta.url),'utf8'),originalMeta=JSON.parse(fs.readFileSync(new URL('../shell/kernel/kernel.json',import.meta.url))),bytes=fs.readFileSync(new URL('../shell/kernel/guest.wasm',import.meta.url));
function fixture(){const urls=new Map(),timers=new Map(),workers=[];let serial=0,instances=0,hang=false,malformed=false,startup=false;
 const wasm={...WebAssembly,compile:WebAssembly.compile,Module:WebAssembly.Module,instantiate:async(...args)=>{instances++;return WebAssembly.instantiate(...args);}};
 class Blob {constructor(parts){this.text=parts.join('');}}
 class Worker {constructor(url){if(startup)throw Error('worker unavailable');this.terminated=false;this.calls=0;workers.push(this);const self={postMessage:data=>queueMicrotask(()=>{if(!this.terminated)this.onmessage?.({data:malformed?{...data,id:-1}:data});})};vm.runInNewContext(urls.get(url),{self,WebAssembly:wasm,Uint8Array,Error,BigInt});this.scope=self;}
  postMessage(data){this.calls++;if(hang)return;queueMicrotask(()=>{if(!this.terminated)this.scope.onmessage({data});});}
  terminate(){this.terminated=true;this.scope=null;}
 }
 let bridge;const controller={};const self={navigator:{serviceWorker:{controller,addEventListener:(_,fn)=>bridge=fn}}};vm.runInNewContext(source,{self,Worker,Blob,location:{href:'https://example.test/'},URL:{createObjectURL:blob=>{const id='blob:'+ ++serial;urls.set(id,blob.text);return id;},revokeObjectURL:id=>urls.delete(id)},crypto:webcrypto,WebAssembly:wasm,TextEncoder,TextDecoder,Uint8Array,atob,Promise,setTimeout:fn=>{const id=++serial;timers.set(id,fn);return id;},clearTimeout:id=>timers.delete(id)});
 const meta=structuredClone(originalMeta);meta.vectors=[meta.vectors[0]];const get=async path=>path.endsWith('.json')?{ok:true,json:async()=>meta}:{ok:true,arrayBuffer:async()=>bytes.buffer.slice(bytes.byteOffset,bytes.byteOffset+bytes.byteLength)};
 return {load:()=>self.ForgeKernel.load(get),get,controller,bridge:(data,ports,source=controller)=>bridge({data,ports,source}),workers,urls,timers,get instances(){return instances;},set hang(x){hang=x;},set malformed(x){malformed=x;},set startup(x){startup=x;}};
}
const tick=()=>new Promise(resolve=>setImmediate(resolve));
test('acceptance replay terminates its worker and each dispatch retains a fresh instance',async()=>{
 const f=fixture(),kernel=await f.load();assert.equal(f.workers.length,1);assert.ok(f.workers[0].terminated);assert.equal(f.urls.size,0);
 for(let i=0;i<35;i++)assert.equal(await kernel.ask('drafting check'),'checked');
 assert.equal(f.instances,36);assert.equal(f.workers.length,4);assert.ok(f.workers.every(w=>w.calls<=16));assert.equal(f.workers.filter(w=>!w.terminated).length,1);
});
test('queue capacity refuses excess requests before execution and drains admitted requests',async()=>{
 const f=fixture(),kernel=await f.load();const admitted=Array.from({length:64},()=>kernel.ask('drafting check'));await assert.rejects(()=>kernel.ask('drafting check'),{code:'kernelExecutionFailed'});assert.ok((await Promise.all(admitted)).every(x=>x==='checked'));assert.equal(f.instances,65);
});
test('worker timeout terminates its isolate and a subsequent request recovers',async()=>{
 const f=fixture(),kernel=await f.load();f.hang=true;const pending=kernel.ask('drafting check');await tick();assert.equal(f.timers.size,1);for(const fire of [...f.timers.values()])fire();await assert.rejects(()=>pending,{code:'kernelExecutionFailed'});assert.ok(f.workers.at(-1).terminated);f.hang=false;assert.equal(await kernel.ask('drafting check'),'checked');
});
test('malformed worker replies and unavailable startup refuse without poisoning recovery',async()=>{
 const f=fixture(),kernel=await f.load();f.startup=true;await assert.rejects(()=>kernel.ask('drafting check'),{code:'kernelExecutionFailed'});assert.equal(f.urls.size,0);f.startup=false;f.malformed=true;await assert.rejects(()=>kernel.ask('drafting check'),{code:'kernelExecutionFailed'});f.malformed=false;assert.equal(await kernel.ask('drafting check'),'checked');
});
test('queued caller buffers are captured before asynchronous worker execution',async()=>{
 const f=fixture(),kernel=await f.load(),request=new TextEncoder().encode('drafting check'),pending=kernel.dispatch(request);request.fill(0);assert.equal(new TextDecoder().decode(await pending),'checked');
});
test('browser worker absence refuses and canonical compiler acceptance remains mandatory',async()=>{
 const self={};vm.runInNewContext(source,{self,location:{},crypto:webcrypto,WebAssembly,TextEncoder,TextDecoder,Uint8Array,atob});const meta=structuredClone(originalMeta);meta.vectors=[meta.vectors[0]];await assert.rejects(()=>self.ForgeKernel.load(async path=>path.endsWith('.json')?{ok:true,json:async()=>meta}:{ok:true,arrayBuffer:async()=>bytes.buffer.slice(bytes.byteOffset,bytes.byteOffset+bytes.byteLength)}),{code:'kernelExecutionFailed'});
 assert.ok(fs.readFileSync(new URL('../../app/src/FoundrySpecification.lex.tex',import.meta.url),'utf8').includes('name: boundedKernelExecutionBoundary,'));assert.ok(fs.readFileSync(new URL('../../.github/workflows/studio.yml',import.meta.url),'utf8').includes('node tools/compiler_browser.mjs'));
});

test('service-worker requests use only a matching verified document guest and never instantiate locally',async()=>{
 const document=fixture();await document.load();const self={clients:{matchAll:async()=>[{postMessage:(data,ports)=>void document.bridge(data,ports)}]}};
 vm.runInNewContext(source,{self,location:{},MessageChannel,crypto:webcrypto,WebAssembly:{...WebAssembly,Module:WebAssembly.Module,compile:WebAssembly.compile,instantiate:()=>assert.fail('service worker instantiated')},TextEncoder,TextDecoder,Uint8Array,atob,setTimeout,clearTimeout});
 const kernel=await self.ForgeKernel.load(document.get);assert.equal(await kernel.ask('drafting check'),'checked');assert.equal(document.instances,3);
});
test('service-worker absence of a verified matching document refuses acceptance',async()=>{
 const owner=fixture();await owner.load();let closed=false;const foreignPort={close(){closed=true;},postMessage(){assert.fail('foreign source received execution');}};
 await owner.bridge({kind:'foundry:kernel-execute/1',key:originalMeta.digest+':'+originalMeta.guest.sha256,request:new TextEncoder().encode('drafting check')},[foreignPort],{});
 assert.ok(closed);assert.equal(owner.instances,1);
 for(const mismatch of [false,true]){
  const document=fixture();await document.load();const self={clients:{matchAll:async()=>mismatch?[{postMessage:(data,ports)=>void document.bridge({...data,key:'unknown-guest'},ports)}]:[]}};
  vm.runInNewContext(source,{self,location:{},MessageChannel,crypto:webcrypto,WebAssembly,TextEncoder,TextDecoder,Uint8Array,atob,setTimeout,clearTimeout});
  await assert.rejects(()=>self.ForgeKernel.load(document.get),{code:'kernelAcceptanceFailed'});
 }
});
