import assert from 'node:assert/strict';
import fs from 'node:fs';
import vm from 'node:vm';
import {test} from 'node:test';
import {encodeRoutingRequest,decodeRoutingResponse,routeWithGuest} from '../shell/routing-codec.mjs';
const vectors=JSON.parse(fs.readFileSync(new URL('../../verification/routing-vectors.json',import.meta.url))).vectors;
const enc=new TextEncoder(),dec=new TextDecoder();
test('structured routing requests and responses match the independent finite corpus',()=>{
 assert.equal(vectors.length,32);assert.equal(vectors.filter(v=>v.input).length,27);
 for(const row of vectors){if(row.input)assert.equal(dec.decode(encodeRoutingRequest(row.input)),row.request,row.name);const result=decodeRoutingResponse(enc.encode(row.response));assert.notEqual(result.code,'invalidRoutingResponse',row.name);}
 const accepted=decodeRoutingResponse(enc.encode(vectors[0].response));assert.equal(accepted.ok,true);assert.deepEqual(accepted.proposal.selected,[{key:10,version:1},{key:11,version:1}]);assert.equal(accepted.proposal.confidence,0);
 const partial=decodeRoutingResponse(enc.encode(vectors.find(v=>v.name==='crossDomainPartial').response));assert.equal(partial.ok,false);assert.equal(partial.code,'unresolvedConstraint');assert.deepEqual(partial.proposal.unresolved,[102]);
});
test('schema oversized collections and non-uint words refuse before the guest',async()=>{
 for(const patch of [{unexpected:1},{expectedIntent:-1},{intent:{id:1,registryRevision:1,concepts:Array(33).fill(1),context:[]}},{relations:Array(129).fill({fromConcept:1,toConcept:2,weight:1})},{catalog:null}]){
  const value={...structuredClone(vectors[0].input),...patch};const result=await routeWithGuest({dispatch:()=>assert.fail('invalid input reached guest')},value);assert.equal(result.ok,false);assert.equal(result.code,'malformedRoutingRequest');
 }
 const input=structuredClone(vectors[0].input);input.relations[0].weight=1.5;assert.throws(()=>encodeRoutingRequest(input),{code:'malformedRoutingRequest'});
});
test('capture precedes guest readiness and typed semantic errors retain refusal authority',async()=>{
 let release;const ready=new Promise(resolve=>release=resolve),input=structuredClone(vectors[0].input),pending=routeWithGuest(ready,input);input.intent.concepts=[999];release({dispatch:async bytes=>{assert.equal(dec.decode(bytes),vectors[0].request);return enc.encode(vectors[0].response);}});assert.equal((await pending).ok,true);
 for(const row of vectors.filter(v=>v.input)){const answer=await routeWithGuest({dispatch:async()=>enc.encode(row.response)},row.input);if(row.response.startsWith('ok ')&&answer.proposal.ambiguities.length)assert.equal(answer.ok,false,row.name);}
 assert.deepEqual(await routeWithGuest(Promise.reject(Error('private runtime')),vectors[0].input),{ok:false,code:'routingRuntimeFailed'});
});
test('malformed output trailing words oversized counts and false successful ambiguity refuse',()=>{
 const bytes=enc.encode(vectors[0].response);
 for(const value of [null,new Uint8Array(4097),enc.encode('unknown'),enc.encode(vectors[0].response+'00000000'),bytes.slice(0,-1)])assert.deepEqual(decodeRoutingResponse(value),{ok:false,code:'invalidRoutingResponse'});
 const ambiguous=vectors.find(v=>v.name==='ambiguity').response;const changed=ambiguous.slice(0,11)+'00000000'+ambiguous.slice(19);assert.deepEqual(decodeRoutingResponse(enc.encode(changed)),{ok:false,code:'invalidRoutingResponse'});
 const count=vectors[0].response.slice(0,51)+'21000000'+vectors[0].response.slice(59);assert.deepEqual(decodeRoutingResponse(enc.encode(count)),{ok:false,code:'invalidRoutingResponse'});
});
test('actual product route invokes only the guest-backed codec and is canonically bound',async()=>{
 const read=p=>fs.readFileSync(new URL(p,import.meta.url),'utf8'),host=read('../shell/host.js'),handlers=new Map();const line=host.split('\n').find(line=>line.startsWith('handlers.set("forge:route"'));let calls=0;vm.runInNewContext(line,{handlers,kernelReady:Promise.resolve({dispatch:async()=>{calls++;return enc.encode(vectors[0].response);}}),routeWithGuest});assert.equal((await handlers.get('forge:route')(vectors[0].input)).ok,true);assert.equal(calls,1);
 const registry=await import('../shell/command-registry.mjs');assert.equal(registry.commandRegistry.resolve('forge:route',handlers.get('forge:route')).owner,'SemanticRouter.boundary');assert.ok(read('./shell_parse.mjs').includes('routing-codec.mjs'));
});
test('registered model protocol full corpus and mandatory runtime qualification remain present',()=>{
 const read=p=>fs.readFileSync(new URL(p,import.meta.url),'utf8'),model=read('../../app/src/SemanticRouter.lex.tex'),protocol=read('../../app/src/RoutingProtocol.lex.tex'),kernel=read('../../app/src/ForgeKernel.lex.tex');assert.equal((model.match(/theorem\{/g)||[]).length,22);assert.equal((protocol.match(/theorem\{/g)||[]).length,8);assert.ok(model.includes('ProposalAdmission::admitProposal'));assert.ok(model.includes('proposalUnresolved'));assert.ok(kernel.includes('RoutingProtocol::dispatchRouting'));assert.ok(kernel.includes('acceptanceVectors := (verdictAcceptance)'));for(const row of vectors)assert.ok(kernel.includes('request := 0x'+Buffer.from(row.request).toString('hex')),row.name);const workflow=read('../../.github/workflows/studio.yml');assert.ok(workflow.includes('node tools/semantic_routing_runtime.mjs'));assert.ok(workflow.includes('node tools/semantic_routing_browser.mjs'));
});
