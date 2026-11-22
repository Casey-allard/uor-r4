import assert from 'node:assert/strict';
import fs from 'node:fs';
import vm from 'node:vm';
import {test} from 'node:test';
import {sameRuleReservation} from '../shell/rules-source.mjs';
import {assertGenerationActive} from '../shell/generation-patch.mjs';
const host=fs.readFileSync(new URL('../shell/host.js',import.meta.url),'utf8');
const functionOf=name=>{const start=host.indexOf('async function '+name+'('),end=host.indexOf('\n}\n',start)+3;assert.ok(start>=0&&end>start);return host.slice(start,end);};
function turnHarness({cancelAt,submitted=false,showFails=false}={}){
 const controller=new AbortController(),machine={queueSignal:controller.signal,abort:null},log={approvals:0,proofs:0,rollbacks:0,ends:0,phases:[],messages:[]};let phase='checked',shows=0;
 const environment={AbortController,Promise,assertGenerationActive,now:()=> 'now',chatMachineOf:()=>machine,
  approve:async()=>{log.approvals++;phase='proving';if(cancelAt==='approve')controller.abort();return{text:'proving',submitted:()=>submitted,cancel:async()=>{if(submitted)return false;log.rollbacks++;phase='checked';return true;},prove:async()=>{log.proofs++;if(cancelAt==='proof'){submitted=true;controller.abort();return new Promise(()=>{});}return{ok:true};}};},
  saveMessages:async(_id,messages)=>{log.messages.push(messages.at(-1).content);if(++shows===1){if(cancelAt==='show')controller.abort();if(showFails)throw Error('storage failed');}},
  emit:type=>{if(type==='chat:stream:end')log.ends++;},publishChat:(_id,state)=>log.phases.push(state),prism:{proofMessage:()=> 'done'},
 };
 return{run:vm.runInNewContext(functionOf('approveTurn')+';approveTurn',environment),log,controller,machine,get phase(){return phase;}};
}
test('cancellation before remote proof restores its reservation and finalizes the chat',{timeout:2000},async()=>{
 for(const cancelAt of ['approve','show']){const h=turnHarness({cancelAt});await h.run(1,7,[],1,{},'intent');assert.equal(h.phase,'checked');assert.equal(h.log.proofs,0);assert.equal(h.log.rollbacks,1);assert.equal(h.log.ends,1);assert.equal(h.machine.abort,null);}
});
test('stop after remote submission only stops observation and never rolls back proving',{timeout:2000},async()=>{
 const h=turnHarness({cancelAt:'proof'});await h.run(1,7,[],1,{},'intent');assert.equal(h.phase,'proving');assert.equal(h.log.proofs,1);assert.equal(h.log.rollbacks,0);assert.equal(h.log.ends,1);assert.equal(h.machine.abort,null);assert.match(h.log.messages.at(-1),/Stopped watching/);
});
function lifecycleHarness({shape='app',refreshFails=false,libraryFails=false,proofFilesFail=false,transientLibrary=false,rollbackRace,completionRace,completionCasRace,proofOk=true,mutate=source=>source}={}){
 const original={phase:'checked',sourceKappa:'source-id',specKappa:'spec-id',logicKappa:'logic-id',spec:{rules:[],properties:[]}},log={commits:0,submits:0,refreshes:0,libraryLoads:0,proofSpecs:[]};
 let state=structuredClone(original),revision=0,controller=new AbortController(),release,entered;const files=new Map([['prism/App.lex.tex','module'],['notes.txt','original note']]);
 const captured=()=>({reference:'revision-'+revision,files:[...files],applicationState:structuredClone(state)});
 const snapshot=()=>({source:files.get('prism/App.lex.tex'),captured:captured(),state:structuredClone(state),expectedState:structuredClone(state)});
 const conflict=()=>Object.assign(Error('changed'),{code:'projectEditConflict'});
 const refresh=async()=>{log.refreshes++;if(refreshFails)throw Error('refresh failed');};
 let waiting=false;const observed=new Promise(resolve=>entered=resolve);
 const prover=async(files,options)=>{if(typeof files==='function')await files({});if(waiting){entered();await new Promise(resolve=>release=resolve);}options.beforeSubmit?.();log.submits++;
  if(completionRace==='replace'){state={...state,approvalId:'newer-approval'};revision++;}
  if(completionRace==='reword'){state={...state,specKappa:'reworded-spec',spec:{...state.spec,title:'new wording'}};revision++;}
  return proofOk?{ok:true,record:{attestation_id:'attestation'},axioms:{}}:{ok:false,error:'Old proof refused'};
 };
 const environment={kernelReady:null,Map,JSON,Promise,AbortController,sameRuleReservation,crypto:{randomUUID:()=> 'unique-attempt'},assertGenerationActive,now:()=> 'now',
  appStateOf:async()=>snapshot(),rulesOf:async()=>structuredClone(state),nextPhase:async(_phase,event)=>event==='approve'?'proving':event==='attest'?'proved':'checked',
  saveAppFiles:async(_app,writes,source,update)=>{
   if(rollbackRace && update.expected.phase==='proving' && update.state.phase==='checked'){
    const race=rollbackRace;rollbackRace=null;revision++;
    if(race==='unrelated')files.set('notes.txt','changed during rollback');
    if(race==='reservation')state={...state,approvalId:'replacement-approval'};
    if(race==='module')files.set('prism/App.lex.tex','replacement module');
   }
   if(completionCasRace && update.expected.phase==='proving' && ['checked','proved'].includes(update.state.phase)){
    const race=Array.isArray(completionCasRace)?completionCasRace.shift():completionCasRace;
    if(!Array.isArray(completionCasRace)||!completionCasRace.length)completionCasRace=null;
    revision++;
    if(race==='unrelated')files.set('notes.txt','changed during completion '+revision);
    if(race==='reservation')state={...state,approvalId:'replacement-approval'};
    if(race==='module')files.set('prism/App.lex.tex','replacement module');
    if(race==='identity')state={...state,sourceKappa:'replacement-identity'};
   }
   if(source.reference!==captured().reference||JSON.stringify(update.expected)!==JSON.stringify(state))throw conflict();
   for(const write of writes)files.set(write.path,write.content);
   state=structuredClone(update.state);revision++;log.commits++;return{source:captured()};
  },
  saveRules:async(_app,next,expected)=>{
   if(rollbackRace && expected.phase==='proving' && next.phase==='checked'){
    const race=rollbackRace;rollbackRace=null;
    if(race==='reword')state={...state,specKappa:'racing-reword',spec:{...state.spec,title:'words changed during rollback'}};
    if(race==='reservation')state={...state,approvalId:'replacement-approval'};
   }
   if(JSON.stringify(state)!==JSON.stringify(expected))return false;state=structuredClone(next);log.commits++;return true;
  },
  refreshApp:refresh,refreshLive:refresh,appModel:async()=>{log.libraryLoads++;if(libraryFails || (transientLibrary && log.libraryLoads===1))throw Error('library failed');return{declarations:[]};},
  appShape:{SOURCE_PATH:'prism/App.lex.tex',STATE_PATH:'prism/app.json',theoremsOf:()=>[{}],promisesOf:()=>[],provingMessage:()=> 'proving',proofMessage:()=> 'done',semanticData:()=>{if(proofFilesFail)throw Error('local render failed');return{};}},
  prism:{proveModule:(_source,_data,_library,options)=>prover(null,options),prove:(spec,options)=>{log.proofSpecs.push(structuredClone(spec));return prover(null,options);},provingMessage:()=> 'proving'},put:async()=>{},enc:new TextEncoder(),kappa:async()=> 'source-id',
 };
 const api=vm.runInNewContext(mutate(['approveApp','proveApp','approveRules','proveRules'].map(functionOf).join('\n'))+';({approveApp,approveRules})',environment);
 return{original,log,controller,observed,wait:()=>{waiting=true;},release:()=>release(),state:()=>structuredClone(state),files:()=>new Map(files),editOther:text=>{files.set('notes.txt',text);revision++;},replace:next=>{state=structuredClone(next);revision++;},approve:()=>shape==='app'?api.approveApp(7,snapshot(),{signal:controller.signal,approvalId:'attempt-one'}):api.approveRules(7,{signal:controller.signal,approvalId:'attempt-one'})};
}
test('stateful and decision cancellation roll back only their exact protected reservation',async()=>{
 for(const shape of ['app','rules']){
  const h=lifecycleHarness({shape}),started=await h.approve();assert.equal(h.state().phase,'proving');h.controller.abort();assert.equal(await started.cancel(),true);assert.deepEqual(h.state(),h.original);assert.equal(h.log.submits,0);assert.equal(await started.cancel(),true);assert.equal(h.log.commits,2);
  const changed=lifecycleHarness({shape}),prior=await changed.approve(),replacement={...changed.state(),approvalId:'other-attempt'};changed.replace(replacement);assert.equal(await prior.cancel(),false);assert.deepEqual(changed.state(),replacement);assert.equal(changed.log.commits,1);
 }
 for(const rollbackRace of [undefined,'unrelated','reservation','module']){
  const h=lifecycleHarness({rollbackRace}),started=await h.approve();h.editOther('changed before rollback');h.controller.abort();
  const restored=await started.cancel();
  if(!rollbackRace || rollbackRace==='unrelated'){assert.equal(restored,true);assert.deepEqual(h.state(),h.original);assert.equal(h.files().get('notes.txt'),rollbackRace?'changed during rollback':'changed before rollback');assert.equal(h.log.commits,2);}
  else {assert.equal(restored,false);assert.equal(h.log.commits,1);if(rollbackRace==='reservation')assert.equal(h.state().approvalId,'replacement-approval');if(rollbackRace==='module')assert.equal(h.files().get('prism/App.lex.tex'),'replacement module');}
  assert.equal(h.log.submits,0);
 }

 for(const rollbackRace of [undefined,'reword','reservation']){
  const h=lifecycleHarness({shape:'rules',rollbackRace}),started=await h.approve();h.replace({...h.state(),specKappa:'reworded-source',spec:{...h.state().spec,title:'new words before rollback'}});h.controller.abort();
  const restored=await started.cancel();
  if(rollbackRace==='reservation'){assert.equal(restored,false);assert.equal(h.state().phase,'proving');assert.equal(h.state().approvalId,'replacement-approval');}
  else {assert.equal(restored,true);assert.equal(h.state().phase,'checked');assert.equal(h.state().spec.title,rollbackRace?'words changed during rollback':'new words before rollback');assert.equal(h.state().approvalId,undefined);}
  assert.equal(h.log.submits,0);
 }

});
test('preview refresh and local library or rendering failures cannot orphan proving state',async()=>{
 for(const shape of ['app','rules']){const h=lifecycleHarness({shape,refreshFails:true});await assert.rejects(h.approve(),/refresh failed/);assert.deepEqual(h.state(),h.original);assert.equal(h.log.submits,0);}
 for(const failure of ['libraryFails','proofFilesFail']){const h=lifecycleHarness({[failure]:true}),started=await h.approve();await assert.rejects(started.prove(),/library failed|local render failed/);assert.deepEqual(h.state(),h.original);assert.equal(h.log.submits,0);}
 const retry=lifecycleHarness({transientLibrary:true}),first=await retry.approve();await assert.rejects(first.prove(),/library failed/);assert.deepEqual(retry.state(),retry.original);const second=await retry.approve();await second.prove();assert.equal(retry.log.libraryLoads,2);assert.equal(retry.log.submits,1);assert.equal(retry.state().phase,'proved');
});
test('cancellation while local proof preparation awaits cannot submit or overwrite a later lifecycle',async()=>{
 for(const shape of ['app','rules']){const h=lifecycleHarness({shape}),started=await h.approve();h.wait();const proof=started.prove();await h.observed;h.controller.abort();await started.cancel();h.release();await assert.rejects(proof,{code:'generationCancelled'});assert.deepEqual(h.state(),h.original);assert.equal(h.log.submits,0);}
});
test('submission consumes rollback authority and proof completion remains source guarded',async()=>{
 for(const shape of ['app','rules']){const h=lifecycleHarness({shape}),started=await h.approve();await started.prove();assert.equal(started.submitted(),true);assert.equal(h.log.submits,1);assert.equal(h.state().phase,'proved');assert.equal(await started.cancel(),false);assert.equal(h.state().phase,'proved');}
 for(const shape of ['app','rules'])for(const proofOk of [true,false]){
  const h=lifecycleHarness({shape,completionRace:'replace',proofOk}),started=await h.approve();
  assert.equal(await started.prove(),null,'old proof must not complete a newer approval');
  assert.equal(h.state().phase,'proving');assert.equal(h.state().approvalId,'newer-approval');assert.equal(h.state().proofError,undefined);assert.equal(h.log.submits,1);assert.equal(h.log.commits,1);assert.equal(await started.cancel(),false);
 }
 for(const proofOk of [true,false])for(const race of ['unrelated','reservation','module','identity']){
  const h=lifecycleHarness({proofOk,completionCasRace:race==='unrelated'?['unrelated','unrelated']:race}),started=await h.approve();
  const result=await started.prove();
  if(race==='unrelated'){
   assert.equal(result?.ok,proofOk,'completed result retries the same reservation after unrelated CAS races');
   assert.equal(h.state().phase,proofOk?'proved':'checked');assert.equal(h.log.commits,2);
   assert.equal(h.files().get('notes.txt'),'changed during completion 3');
   assert.equal(h.files().get('prism/App.lex.tex'),proofOk?'module\n':'module');
  }else{
   assert.equal(result,null);assert.equal(h.state().phase,'proving');assert.equal(h.log.commits,1);
   if(race==='reservation')assert.equal(h.state().approvalId,'replacement-approval');
   if(race==='module')assert.equal(h.files().get('prism/App.lex.tex'),'replacement module');
   if(race==='identity')assert.equal(h.state().sourceKappa,'replacement-identity');
  }
  assert.equal(h.log.submits,1,'retrying local result commit must never resubmit the prover');assert.equal(await started.cancel(),false);
 }
 await assert.rejects(async()=>{
  const h=lifecycleHarness({completionCasRace:'unrelated',mutate:source=>{const marker='// An unrelated edit can race this CAS.';assert.ok(source.includes(marker));return source.replace(marker,'return null; '+marker);}}),started=await h.approve();
  assert.equal((await started.prove())?.ok,true,'a dropped result must fail the completion retry control');
 },{name:'AssertionError'});
 const reword=lifecycleHarness({shape:'rules',completionRace:'reword'}),same=await reword.approve();assert.equal((await same.prove()).ok,true);assert.equal(reword.state().phase,'proved');assert.equal(reword.state().spec.title,'new wording');assert.equal(reword.log.submits,1);assert.equal(reword.log.commits,2);
 const before=lifecycleHarness({shape:'rules'}),approved=await before.approve();before.replace({...before.state(),specKappa:'before-start-reword',spec:{...before.state().spec,title:'new words before start'}});assert.equal((await approved.prove()).ok,true);assert.equal(before.state().phase,'proved');assert.equal(before.state().spec.title,'new words before start');assert.equal(before.log.submits,1);assert.deepEqual(before.log.proofSpecs,[before.original.spec],'submission retains the originally approved payload despite later wording');

});
test('the in-tab prover checks cancellation immediately before it asks the kernel',async()=>{
 const source=fs.readFileSync(new URL('../shell/prism.js',import.meta.url),'utf8'),start=source.indexOf('export async function proveModule('),end=source.indexOf('\n}\n',start)+3;
 const run=source.slice(start,end).replace('export ','');assert.ok(run.includes('beforeSubmit(); r = await'));
 const make=(text)=>{const log={asked:0};return{log,prove:vm.runInNewContext('let lean=null;'+text+';proveModule',{Date,Set,String,moduleTerms:()=>({defs:[],theorems:[{name:'Tab.app.t'}],refused:[]}),openLean:async()=>({module:async()=>{log.asked++;return{refusal:'no kernel here'};}})})};};
 const controller=new AbortController(),cancelled=make(run);
 await assert.rejects(cancelled.prove('m',{},[],{beforeSubmit:()=>{controller.abort();assertGenerationActive(controller.signal);}}),{code:'generationCancelled'});assert.equal(cancelled.log.asked,0);
 let admitted=0;const admittedRun=make(run);assert.equal((await admittedRun.prove('m',{},[],{beforeSubmit:()=>admitted++})).ok,false);assert.equal(admitted,1);assert.equal(admittedRun.log.asked,1);
 const mutant=make(run.replace('beforeSubmit(); r = await','r = await'));
 await assert.rejects(async()=>{await assert.rejects(mutant.prove('m',{},[],{beforeSubmit:()=>{throw Object.assign(Error('cancelled'),{code:'generationCancelled'});}}),{code:'generationCancelled'});},{name:'AssertionError'});
});
