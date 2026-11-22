import assert from 'node:assert/strict';
import fs from 'node:fs';
import vm from 'node:vm';
import {test} from 'node:test';
import {encodeQueueRequest,decodeQueueResponse,queueWithGuest} from '../shell/chat-queue-codec.mjs';
const vectors=JSON.parse(fs.readFileSync(new URL('../../verification/chat-queue-vectors.json',import.meta.url))).vectors,enc=new TextEncoder(),dec=new TextDecoder();
test('every independent queue representation retains exact state and typed refusal',()=>{
 assert.equal(vectors.length,38);assert.equal(vectors.filter(v=>v.input).length,33);
 for(const row of vectors){if(row.input)assert.equal(dec.decode(encodeQueueRequest(row.input)),row.request,row.name);const answer=decodeQueueResponse(enc.encode(row.response));assert.deepEqual(answer,row.response.startsWith('ok ')?{ok:true,state:row.expected}:{ok:false,code:row.response},row.name);}
});
test('schema list bounds uint fields and unsupported shape refuse before dispatch',async()=>{
 for(const patch of [{unexpected:1},{expectedRevision:-1},{command:{kind:6}},{state:{...vectors[0].input.state,items:Array(33).fill(1)}}]){const input={...structuredClone(vectors[0].input),...patch};assert.deepEqual(await queueWithGuest({dispatch:()=>assert.fail()},input),{ok:false,code:'malformedQueueRequest'});}
 const input=structuredClone(vectors[0].input);input.state.items=[1.5];assert.throws(()=>encodeQueueRequest(input),{code:'malformedQueueRequest'});
});
test('input captures before readiness and runtime failure has no successful state',async()=>{
 let release;const ready=new Promise(resolve=>release=resolve),input=structuredClone(vectors[0].input),pending=queueWithGuest(ready,input);input.command.item=99;release({dispatch:async bytes=>{assert.equal(dec.decode(bytes),vectors[0].request);return enc.encode(vectors[0].response);}});assert.deepEqual(await pending,{ok:true,state:vectors[0].expected});assert.deepEqual(await queueWithGuest(Promise.reject(Error('private')),vectors[0].input),{ok:false,code:'queueRuntimeFailed'});
});
test('malformed output false active ownership and trailing words cannot pass',()=>{
 for(const bytes of [null,new Uint8Array(513),enc.encode('unknown'),enc.encode(vectors[0].response+'00000000'),enc.encode(vectors[0].response.slice(0,-1))])assert.deepEqual(decodeQueueResponse(bytes),{ok:false,code:'invalidQueueResponse'});
 const claim=vectors.find(v=>v.name==='claim').response,ownerOffset=3+5*8;const falseOwner=claim.slice(0,ownerOffset)+'00000000'+claim.slice(ownerOffset+8);assert.deepEqual(decodeQueueResponse(enc.encode(falseOwner)),{ok:false,code:'invalidQueueResponse'});
});
test('actual host adapter invokes only the guest and canonical registry binds it',async()=>{
 const host=fs.readFileSync(new URL('../shell/host.js',import.meta.url),'utf8'),line=host.split('\n').find(s=>s.startsWith('handlers.set("forge:queue"')),handlers=new Map();let count=0;vm.runInNewContext(line,{handlers,queueWithGuest,kernelReady:{dispatch:async()=>{count++;return enc.encode(vectors[0].response);}}});assert.deepEqual(await handlers.get('forge:queue')(vectors[0].input),{ok:true,state:vectors[0].expected});assert.equal(count,1);const {commandRegistry}=await import('../shell/command-registry.mjs');assert.equal(commandRegistry.resolve('forge:queue',handlers.get('forge:queue')).owner,'ChatQueue.boundary');
});
test('finite proof wire corpus and mandatory actual runtime gates remain complete',()=>{
 const read=p=>fs.readFileSync(new URL(p,import.meta.url),'utf8'),model=read('../../app/src/ChatQueue.lex.tex'),protocol=read('../../app/src/ChatQueueProtocol.lex.tex'),kernel=read('../../app/src/ForgeKernel.lex.tex');assert.equal((model.match(/theorem\{/g)||[]).length,24);assert.equal((protocol.match(/theorem\{/g)||[]).length,8);assert.ok(protocol.includes('ChatQueue::transitionQueue'));assert.ok(kernel.includes('ChatQueueProtocol::dispatchQueue'));assert.ok(kernel.includes('acceptanceVectors := (verdictAcceptance)'));for(const row of vectors)assert.ok(kernel.includes('request := 0x'+Buffer.from(row.request).toString('hex')),row.name);assert.ok(read('../../.github/workflows/studio.yml').includes('node tools/chat_queue_runtime.mjs'));assert.ok(read('../../.github/workflows/studio.yml').includes('node tools/chat_queue_browser.mjs'));assert.ok(read('./shell_parse.mjs').includes('chat-queue-codec.mjs'));
});
