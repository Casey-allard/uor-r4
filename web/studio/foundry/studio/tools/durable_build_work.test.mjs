import assert from 'node:assert/strict';
import fs from 'node:fs';
import vm from 'node:vm';
import {createHash,webcrypto} from 'node:crypto';
import {test} from 'node:test';
import {invokeHost} from '../shell/host-boundary.mjs';
import {runBuildWork} from '../shell/build-work.mjs';
const self={};vm.runInNewContext(fs.readFileSync(new URL('../shell/kernel.js',import.meta.url),'utf8'),{self,crypto:webcrypto,WebAssembly,TextEncoder,TextDecoder,Uint8Array,atob});
const guest=await self.ForgeKernel.load(async path=>new Response(fs.readFileSync(new URL('../shell/'+path,import.meta.url))));
const source='blake3:'+'a'.repeat(64),request={appId:1,source,environment:'b'.repeat(64)},output=()=>({html:'<p>actual artifact</p>',sourceReference:source,moduleFiles:[{path:'x.js',bytes:new Uint8Array([1,2])}],moduleClosure:{modules:[]}});
function harness(){let journal,held=false,stale=false;let saves=0;const boundary={guest,crypto:webcrypto,derive:async bytes=>'blake3:'+createHash('sha256').update(bytes).digest('hex'),locks:{request:async(_key,_options,run)=>{if(held)return run(null);held=true;try{return await run({});}finally{held=false;}}},read:async()=>structuredClone(journal),save:async(expected,next)=>{assert.deepEqual(journal,expected);journal=structuredClone(next);saves++;},assertSource:async()=>{if(stale)throw Object.assign(Error('stale'),{code:'buildWorkSourceChanged'});}};return {boundary,get journal(){return journal;},set journal(value){journal=value;},get saves(){return saves;},set stale(value){stale=value;}};}
test('actual verified guest reserves before effects and atomically checkpoints reusable output',async()=>{
 const h=harness();let effects=0;const result=await runBuildWork(request,async()=>{effects++;const row=h.journal.records[0];assert.equal(row.state.phase,1);assert.equal(row.state.reserved,1);assert.equal(row.state.spent,0);return output();},h.boundary);
 assert.deepEqual(result,output());const row=h.journal.records[0];assert.equal(row.status,'complete');assert.equal(row.state.phase,3);assert.equal(row.state.spent,1);assert.equal(row.state.remaining,0);assert.equal(row.state.reserved,0);assert.equal(row.state.checkpoint,1);assert.equal(h.saves,2);
 result.moduleFiles[0].bytes[0]=99;assert.deepEqual(await runBuildWork(request,()=>assert.fail('cached work repeated'),h.boundary),output());assert.equal(effects,1);
});
test('failure retains reservations and a new owner resumes through the guest',async()=>{
 const h=harness();await assert.rejects(runBuildWork(request,()=>{throw Error('private failure');},h.boundary),/private failure/);const prior=structuredClone(h.journal.records[0]);assert.equal(prior.status,'interrupted');assert.equal(prior.state.phase,2);assert.equal(prior.state.spent,0);assert.equal(prior.state.reserved,1);assert.equal(prior.reason,'interrupted');
 await runBuildWork(request,output,h.boundary);const next=h.journal.records[0];assert.notEqual(next.state.owner,prior.state.owner);assert.equal(next.attempts,2);assert.equal(next.state.spent,1);
 const interrupted=harness();let release;const pending=runBuildWork(request,()=>new Promise(resolve=>release=resolve),interrupted.boundary);pending.catch(()=>{});for(let i=0;i<100&&!release;i++)await new Promise(r=>setTimeout(r,1));assert.equal(typeof release,'function','reserved work did not enter its operation');const crash=structuredClone(interrupted.journal);release(output());await pending;
 const reloaded=harness();reloaded.journal=crash;await runBuildWork(request,output,reloaded.boundary);assert.equal(reloaded.journal.records[0].state.phase,3);assert.equal(reloaded.journal.records[0].attempts,2);
});
test('concurrent owner missing locks source changes and guest refusal cannot execute',async()=>{
 const h=harness();let release;const pending=runBuildWork(request,()=>new Promise(resolve=>release=resolve),h.boundary);pending.catch(()=>{});for(let i=0;i<100&&!release;i++)await new Promise(r=>setTimeout(r,1));assert.equal(typeof release,'function','reserved work did not enter its operation');await assert.rejects(runBuildWork(request,()=>assert.fail('duplicate effect'),h.boundary),{code:'buildWorkBusy'});release(output());await pending;
 await assert.rejects(runBuildWork(request,()=>assert.fail(),{...h.boundary,locks:null}),{code:'buildWorkUnavailable'});await assert.rejects(runBuildWork(request,()=>assert.fail(),{...h.boundary,locks:{request(){throw Error('private');}}}),{code:'buildWorkUnavailable'});h.stale=true;await assert.rejects(runBuildWork(request,()=>assert.fail(),h.boundary),{code:'buildWorkSourceChanged'});
 const refused=harness();await assert.rejects(runBuildWork(request,()=>assert.fail(),{...refused.boundary,guest:{dispatch:async()=>new TextEncoder().encode('wrongOwner')}}),{code:'wrongOwner'});assert.equal(refused.saves,0);const envelope=await invokeHost('forge:publish',[],()=>{throw Object.assign(Error('private guest data'),{code:'wrongOwner'});});assert.equal(envelope.error.code,'wrongOwner');assert.ok(!envelope.error.message.includes('private')); 
});
test('corrupt retained output and invalid descriptors never become successful reuse',async()=>{
 const h=harness();await runBuildWork(request,output,h.boundary);h.journal.records[0].output.html='tampered';await assert.rejects(runBuildWork(request,()=>assert.fail(),h.boundary),{code:'buildWorkArtifactInvalid'});
 for(const patch of [{appId:0},{source:'unbound'},{environment:'unbound'},{owner:7}])await assert.rejects(runBuildWork({...request,...patch},output,h.boundary),{code:'invalidBuildWork'});
 const invalid=harness();await assert.rejects(runBuildWork(request,()=>({html:'no source'}),invalid.boundary),{code:'buildWorkArtifactInvalid'});assert.equal(invalid.journal.records[0].status,'interrupted');
 invalid.journal.records[0].state.owner=0;await assert.rejects(runBuildWork(request,output,invalid.boundary),{code:'invalidBuildWork'});
});
test('failed completion preserves last durable work and journal bounds refuse new sources',async()=>{
 const h=harness(),save=h.boundary.save;let fail=true;h.boundary.save=async(expected,next,options)=>{if(next.records[0].status==='complete'&&fail){fail=false;throw Object.assign(Error('conflict'),{code:'buildWorkConflict'});}return save(expected,next,options);};await assert.rejects(runBuildWork(request,output,h.boundary),{code:'buildWorkConflict'});assert.equal(h.journal.records[0].state.spent,0);assert.equal(h.journal.records[0].state.reserved,1);assert.equal(h.journal.records[0].output,null);await runBuildWork(request,output,h.boundary);
 const row=h.journal.records[0];h.journal.records=Array.from({length:32},(_,i)=>({...structuredClone(row),source:'blake3:'+i.toString(16).padStart(64,'0')}));await assert.rejects(runBuildWork(request,()=>assert.fail(),h.boundary),{code:'buildWorkLimitExceeded'});
});
test('actual host transaction deletion parser and CI retain the work adapter',()=>{
 const read=p=>fs.readFileSync(new URL(p,import.meta.url),'utf8'),host=read('../shell/host.js');assert.ok(host.includes('return runBuildWork({appId,source:source.reference,environment}'));assert.ok(host.includes('guest:await kernelReady,derive:kappa,...buildWorkStore'));assert.ok(read('../shell/project-edit.mjs').includes('JSON.stringify(currentWork) !== JSON.stringify(work.expected)'));assert.ok(read('../shell/project-lifecycle.mjs').includes('refs.delete(`work:${appId}`)'));assert.ok(read('./shell_parse.mjs').includes('build-work.mjs'));assert.ok(read('../../.github/workflows/studio.yml').includes('node tools/durable_build_work_browser.mjs'));
});
