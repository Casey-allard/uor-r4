import assert from 'node:assert/strict';
import fs from 'node:fs';
import vm from 'node:vm';
import {test} from 'node:test';
import {encodePreviewRequest,decodePreviewResponse,previewWithGuest,previewAddressWords,previewAddressIdentity} from '../shell/version-preview-codec.mjs';
const vectors=JSON.parse(fs.readFileSync(new URL('../../verification/version-preview-vectors.json',import.meta.url))).vectors,enc=new TextEncoder(),dec=new TextDecoder();
test('independent preview corpus preserves exact original current and target identities',()=>{
 assert.equal(vectors.length,30);assert.equal(vectors.filter(x=>x.input).length,24);
 for(const row of vectors){if(row.input)assert.equal(dec.decode(encodePreviewRequest(row.input)),row.request,row.name);assert.deepEqual(decodePreviewResponse(enc.encode(row.response)),typeof row.expected==='string'?{ok:false,code:row.expected}:{ok:true,...row.expected},row.name);}
 const id='blake3:'+Array.from({length:32},(_,i)=>i.toString(16).padStart(2,'0')).join('');assert.equal(previewAddressIdentity(previewAddressWords(id)),id);assert.equal(previewAddressIdentity(Array(8).fill(0xffffffff)),'blake3:'+'ff'.repeat(32));assert.equal(previewAddressIdentity(Array(8).fill(0)),'blake3:'+'00'.repeat(32));
});
test('exact shapes address widths and unsigned words refuse before guest dispatch',async()=>{
 for(const patch of [{extra:true},{expectedRevision:-1},{command:{kind:1}},{state:{...vectors[0].input.state,origin:Array(7).fill(0)}},{state:{...vectors[0].input.state,current:[4294967296,...Array(7).fill(0)]}}])assert.deepEqual(await previewWithGuest({dispatch:()=>assert.fail()},{...structuredClone(vectors[0].input),...patch}),{ok:false,code:'malformedPreviewRequest'});
 for(const identity of ['blake3:'+'a'.repeat(63),'sha256:'+'a'.repeat(64),'blake3:'+'A'.repeat(64)])assert.throws(()=>previewAddressWords(identity),{code:'malformedPreviewRequest'});
});
test('request capture precedes readiness and guest faults cannot invent transitions',async()=>{
 let release;const ready=new Promise(resolve=>release=resolve),input=structuredClone(vectors[0].input),pending=previewWithGuest(ready,input);input.command.target[0]=99;release({dispatch:async bytes=>{assert.equal(dec.decode(bytes),vectors[0].request);return enc.encode(vectors[0].response);}});assert.deepEqual(await pending,{ok:true,...vectors[0].expected});assert.deepEqual(await previewWithGuest(Promise.reject(Error('private')),vectors[0].input),{ok:false,code:'previewRuntimeFailed'});
});
test('malformed replies false owner state and non-Boolean write markers refuse',()=>{
 const good=vectors[0].response;
 for(const value of [null,new Uint8Array(236),enc.encode('unknown'),enc.encode(good+'00000000'),enc.encode(good.slice(0,-1)),enc.encode(good.slice(0,3+3*8)+'00000000'+good.slice(3+4*8)),enc.encode(good.slice(0,3+20*8)+'02000000'+good.slice(3+21*8))])assert.deepEqual(decodePreviewResponse(value),{ok:false,code:'invalidPreviewResponse'});
});
test('actual host representation boundary delegates to the guest and canonical registry',async()=>{
 const host=fs.readFileSync(new URL('../shell/host.js',import.meta.url),'utf8'),handlers=new Map();let calls=0;vm.runInNewContext(host.split('\n').find(s=>s.startsWith('handlers.set("forge:version-preview"')),{handlers,previewWithGuest,kernelReady:{dispatch:async()=>{calls++;return enc.encode(vectors[0].response);}}});assert.deepEqual(await handlers.get('forge:version-preview')(vectors[0].input),{ok:true,...vectors[0].expected});assert.equal(calls,1);const {commandRegistry}=await import('../shell/command-registry.mjs');assert.equal(commandRegistry.resolve('forge:version-preview',handlers.get('forge:version-preview')).owner,'VersionPreview.boundary');
});
test('model proofs exact corpus and mandatory signed runtime gates remain complete',()=>{
 const read=p=>fs.readFileSync(new URL(p,import.meta.url),'utf8'),model=read('../../app/src/VersionPreview.lex.tex'),protocol=read('../../app/src/VersionPreviewProtocol.lex.tex'),kernel=read('../../app/src/ForgeKernel.lex.tex');assert.equal((model.match(/theorem\{/g)||[]).length,24);assert.equal((protocol.match(/theorem\{/g)||[]).length,8);assert.ok(protocol.includes('VersionPreview::transitionPreview'));assert.ok(kernel.includes('VersionPreviewProtocol::dispatchPreview'));assert.ok(kernel.includes('acceptanceVectors := (verdictAcceptance)'));for(const row of vectors)assert.ok(kernel.includes('request := 0x'+Buffer.from(row.request).toString('hex')),row.name);for(const name of ['version_preview_runtime.mjs','version_preview_browser.mjs'])assert.ok(read('../../.github/workflows/studio.yml').includes('node tools/'+name));assert.ok(read('./shell_parse.mjs').includes('version-preview-codec.mjs'));
});
