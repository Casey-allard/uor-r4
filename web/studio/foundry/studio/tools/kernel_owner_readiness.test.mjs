import assert from 'node:assert/strict';
import fs from 'node:fs';
import vm from 'node:vm';
import {webcrypto} from 'node:crypto';
import {test} from 'node:test';
const source=fs.readFileSync(new URL('../shell/kernel.js',import.meta.url),'utf8'),originalMeta=JSON.parse(fs.readFileSync(new URL('../shell/kernel/kernel.json',import.meta.url))),bytes=fs.readFileSync(new URL('../shell/kernel/guest.wasm',import.meta.url));
function fixture(){const urls=new Map(),timers=new Map(),durations=new Map(),workers=[];let serial=0,instances=0,hang=false,malformed=false,startup=false;
 const wasm={...WebAssembly,compile:WebAssembly.compile,Module:WebAssembly.Module,instantiate:async(...args)=>{instances++;return WebAssembly.instantiate(...args);}};
 class Blob {constructor(parts){this.text=parts.join('');}}
 class Worker {constructor(url){if(startup)throw Error('worker unavailable');this.terminated=false;this.calls=0;workers.push(this);const self={postMessage:data=>queueMicrotask(()=>{if(!this.terminated)this.onmessage?.({data:malformed?{...data,id:-1}:data});})};vm.runInNewContext(urls.get(url),{self,WebAssembly:wasm,Uint8Array,Error,BigInt});this.scope=self;}
  postMessage(data){this.calls++;if(hang)return;queueMicrotask(()=>{if(!this.terminated)this.scope.onmessage({data});});}
  terminate(){this.terminated=true;this.scope=null;}
 }
 let bridge;const controller={};const self={navigator:{serviceWorker:{controller,addEventListener:(_,fn)=>bridge=fn}}};vm.runInNewContext(source,{self,Worker,Blob,location:{href:'https://example.test/'},URL:{createObjectURL:blob=>{const id='blob:'+ ++serial;urls.set(id,blob.text);return id;},revokeObjectURL:id=>urls.delete(id)},crypto:webcrypto,WebAssembly:wasm,TextEncoder,TextDecoder,Uint8Array,atob,Promise,setTimeout:(fn,ms)=>{const id=++serial;timers.set(id,fn);durations.set(id,ms);return id;},clearTimeout:id=>{timers.delete(id);durations.delete(id);}});
 const meta=structuredClone(originalMeta);meta.vectors=[meta.vectors[0]];const get=async path=>path.endsWith('.json')?{ok:true,json:async()=>meta}:{ok:true,arrayBuffer:async()=>bytes.buffer.slice(bytes.byteOffset,bytes.byteOffset+bytes.byteLength)};
 return {load:(fetch=get)=>self.ForgeKernel.load(fetch),get,controller,bridge:(data,ports,source=controller)=>bridge({data,ports,source}),workers,urls,timers,durations,get instances(){return instances;},set hang(x){hang=x;},set malformed(x){malformed=x;},set startup(x){startup=x;}};
}
const tick=()=>new Promise(resolve=>setImmediate(resolve));
const key=originalMeta.digest+':'+originalMeta.guest.sha256;
const message=(identity=key)=>({kind:'foundry:kernel-execute/1',key:identity,request:new TextEncoder().encode('drafting check')});
function port(){const replies=[];return {replies,closed:false,postMessage(value){replies.push(value);},close(){this.closed=true;}};}
function delayed(f,bad=false){let release;const held=new Promise(resolve=>release=resolve);let fetched=0;const get=async path=>{fetched++;await held;if(bad&&!path.endsWith('.json'))return {ok:true,arrayBuffer:async()=>new Uint8Array(bytes.length).buffer};return f.get(path);};return {get,release:()=>release(),get fetched(){return fetched;}};}
test('initializing document waits before serving a matching verified guest',async()=>{
 const f=fixture(),gate=delayed(f),owner=f.load(gate.get),p=port(),pending=f.bridge(message(),[p]);
 try{await tick();assert.equal(p.replies.length,0);assert.equal(f.instances,0);}finally{gate.release();}
 await owner;await pending;assert.equal(p.replies.length,1);assert.equal(p.replies[0].ok,true);assert.equal(new TextDecoder().decode(p.replies[0].result),'checked');assert.equal(p.closed,true);assert.equal(f.instances,2);
});
test('failed integrity and readiness deadlines close ports without unverified execution',async()=>{
 const f=fixture(),gate=delayed(f,true),owner=f.load(gate.get);const rejection=assert.rejects(()=>owner,{code:'kernelIntegrityMismatch'}),p=port(),pending=f.bridge(message(),[p]);
 await tick();assert.equal(p.replies.length,0);gate.release();await rejection;await pending;assert.equal(p.replies[0].ok,false);assert.equal(p.replies[0].code,'kernelExecutionFailed');assert.equal(p.closed,true);assert.equal(f.instances,0);
 const good=await f.load();assert.equal(await good.ask('drafting check'),'checked');
 const rejected=fixture(),delayedReplay=delayed(rejected),replayLoad=rejected.load(async path=>{const response=await delayedReplay.get(path);if(path.endsWith('.json')){const meta=structuredClone(await response.json());meta.vectors[0][1]='d3Jvbmc=';return {ok:true,json:async()=>meta};}return response;});
 const badReplay=assert.rejects(()=>replayLoad,{code:'kernelAcceptanceFailed'}),replayPort=port(),replayRequest=rejected.bridge(message(),[replayPort]);delayedReplay.release();await badReplay;await replayRequest;assert.equal(replayPort.replies[0].ok,false);assert.equal(replayPort.closed,true);

 const slow=fixture(),held=delayed(slow),loading=slow.load(held.get),timed=port(),waiting=slow.bridge(message(),[timed]);
 try{await tick();assert.equal(slow.timers.size,1);assert.deepEqual([...slow.durations.values()],[30000]);for(const fire of [...slow.timers.values()])fire();await waiting;assert.equal(timed.closed,true);assert.equal(timed.replies[0].ok,false);assert.equal(slow.instances,0);}finally{held.release();}
 await loading;const recovered=port();await slow.bridge(message(),[recovered]);assert.equal(recovered.replies[0].ok,true);assert.equal(slow.timers.size,0);

});
test('foreign controllers never wait or execute and unmatched identity still refuses after readiness',async()=>{
 const f=fixture(),gate=delayed(f),owner=f.load(gate.get),foreign=port();const foreignTask=f.bridge(message(),[foreign],{});
 try{
  await tick();assert.equal(foreign.closed,true);assert.equal(foreign.replies.length,0);await foreignTask;
  const mismatch=port(),pending=f.bridge(message('unmatched-guest'),[mismatch]);await tick();assert.equal(mismatch.replies.length,0);gate.release();await owner;await pending;assert.equal(mismatch.replies[0].ok,false);assert.equal(f.instances,1);
 }finally{gate.release();await owner;await foreignTask;}

});
test('initialization capacity refuses before fetch and frees every settled slot',async()=>{
 const f=fixture(),gate=delayed(f),owners=Array.from({length:8},()=>f.load(gate.get));
 const excess=f.load(gate.get).then(()=> 'accepted',error=>error.code);
 try{await tick();assert.equal(gate.fetched,8);}finally{gate.release();}
 assert.equal(await excess,'kernelExecutionFailed');await Promise.all(owners);const next=await f.load();assert.equal(await next.ask('drafting check'),'checked');
 const read=p=>fs.readFileSync(new URL(p,import.meta.url),'utf8');assert.ok(read('../../app/src/FoundrySpecification.lex.tex').includes('name: kernelOwnerReadinessBoundary,'));assert.ok(read('../../.github/workflows/studio.yml').includes('node tools/kernel_owner_readiness_browser.mjs'));
});
