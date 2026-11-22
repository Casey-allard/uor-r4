import assert from 'node:assert/strict';
import fs from 'node:fs';
import vm from 'node:vm';
import {test} from 'node:test';
import {planCanonicalEdit} from '../shell/canonical-edits.mjs';
import * as lexlean from '../shell/live/lexlean.mjs';
import * as prism from '../shell/prism.js';
import * as appShape from '../shell/live/app.mjs';
import {applicationShape,boundApplicationState,retiredApplicationPaths,activeApplicationModule,isStatefulModule} from '../shell/application-shape.mjs';
import {canonicalEditPath} from '../shell/canonical-edits.mjs';
import {stageGenerationPatch} from '../shell/generation-patch.mjs';
import {commitPreparedProject,captureProjectSource} from '../shell/project-edit.mjs';
import {readBoundRules,readBoundRulesSnapshot,saveBoundRules} from '../shell/rules-source.mjs';
import {assertCanonicalBuild} from '../shell/canonical-build.mjs';
import {requiresRulePublication} from '../shell/build-source.mjs';
import {modulePresentation} from '../shell/module-presentation.mjs';
import * as live from '../shell/live/project.mjs';
import {createEditor} from '../shell/live/edit.mjs';
import {kappa} from '../../try/vendor/prism-code/src/blake3.mjs';
const host=fs.readFileSync(new URL('../shell/host.js',import.meta.url),'utf8');
const module=fs.readFileSync(new URL('../../v2/loop/answers/guests-3.verified.lex.tex',import.meta.url),'utf8');
const spec=JSON.parse(fs.readFileSync(new URL('./fixtures/live/rsvp.json',import.meta.url),'utf8'));
const canonical=lexlean.canonicalSource(spec), decisionPath=`prism/${canonical.name}.lex.tex`;
const oid='blake3:'+'a'.repeat(64), nextOid='blake3:'+'b'.repeat(64), enc=new TextEncoder();
const snapshot=(files,applicationState)=>({name:'app',reference:oid,dependencies:[],files,...(applicationState===undefined?{}:{applicationState})});

test('a real stateful module editor save retains the whole bounded input',()=>{
 const content=module.replace('Guests','Visitors');
 const plan=planCanonicalEdit({filePath:'prism/App.lex.tex',content},{files:[['prism/App.lex.tex',module]]},{lexlean,prism});
 assert.equal(plan.shape,'stateful');assert.equal(plan.moduleText,content);
 assert.throws(()=>planCanonicalEdit({filePath:'prism/App.lex.tex',content:content+' '.repeat(1024*1024)},{files:[]},{lexlean,prism}),{code:'invalidCanonicalEdit'});
});

test('module labels and helper names cannot redirect stateful authority to generic code',()=>{
 assert.equal(isStatefulModule(module.replace('Guests','name: judge')),true);
 assert.equal(isStatefulModule(module+'\ndefinition{ name: judge, body: true }'),true);
 assert.equal(applicationShape(snapshot([['prism/App.lex.tex',module]])),'stateful');
 assert.throws(()=>applicationShape(snapshot([['prism/App.lex.tex','invalid draft']])),{code:'applicationShapeConflict'});
});
test('active module selection refuses mixed authority and ignores inactive alphabetic modules',()=>{
 const files=[['prism/AAA.lex.tex','inactive'],['prism/spec.json',JSON.stringify(spec)],[decisionPath,canonical.source]];
 assert.equal(activeApplicationModule(snapshot(files),lexlean.canonicalSource)[0],decisionPath);
 assert.throws(()=>activeApplicationModule(snapshot([...files,['prism/App.lex.tex',module]]),lexlean.canonicalSource),{code:'applicationShapeConflict'});
});
test('stateful direct editing calls the real shape planner and avoids the decision writer',async()=>{
 const handlers=new Map(),source=snapshot([['prism/App.lex.tex',module]]),calls=[];
 const begin=host.indexOf('handlers.set("edit-app-file",'),end=host.indexOf('handlers.set("delete-app",',begin);
 vm.runInNewContext(host.slice(begin,end),{structuredClone,handlers,coreReady:Promise.resolve(),dbReady:Promise.resolve({}),canonicalEditPath,planCanonicalEdit,lexlean,prism,captureProjectSource:async()=>source,
  appAfterTurn:async(id,text,options)=>{calls.push(options);assert.equal(id,7);assert.equal(text,'');assert.equal(options.source,source);},refreshPreview:async()=>{},
  rulesAfterTurn(){assert.fail('decision-only writer');},editStoredProject(){assert.fail('generic writer');},emit(){},
 });
 const input={appId:7,filePath:'prism/App.lex.tex',content:module};
 await handlers.get('edit-app-file')(input);assert.equal(calls.length,1);assert.equal(calls[0].moduleText,module);
});

function transitionHarness(source) {
 const first=host.indexOf('async function appAfterTurn('),last=host.indexOf('// The kernel\'s answer to a reword.',first);
 const log={checks:0,forgot:0};let patch,options;
 const api=vm.runInNewContext(host.slice(first,last)+';({appAfterTurn,rulesAfterTurn})',{
  Map,JSON,enc,appShape,lexlean,prism,live,modulePresentation,retiredApplicationPaths,boundApplicationState,
  core:{},coreReady:Promise.resolve(),dbReady:Promise.resolve({}),assertGenerationActive(signal){if(signal?.aborted)throw Error('cancelled');},
  captureProjectSource:async()=>source,kappa,nextPhase:async(_phase,event)=>event==='check'?'checked':'drafting',now:()=> 'now',
  rulesSnapshotOf:async()=>({state:null,expected:{phase:'proved',old:true}}),checkAppModule:async()=>{log.checks++;return{ok:true,issues:[]};},forgetApplicationFrames:()=>log.forgot++,
  applyStoredProjectPatch:async(_db,_id,p,o)=>{patch=p;options=o;const staged=stageGenerationPatch(source,p);return{changed:staged.changed,source:{...source,files:[...staged.files],applicationState:o.appUpdate?.state}};},
 });
 return{...api,log,get patch(){return patch;},get options(){return options;}};
}
test('decision to stateful conversion removes obsolete source and proof authority atomically',async()=>{
 const source=snapshot([['prism/spec.json',JSON.stringify(spec)],[decisionPath,canonical.source],['src/lib/prism.ts','runtime'],['prism/Inactive.lex.tex','keep']]);
 const h=transitionHarness(source),result=await h.appAfterTurn(7,`<prism-lexlean>${module}</prism-lexlean>`,{source});
 const files=new Map(result.source.files);
 for(const path of ['prism/spec.json',decisionPath,'src/lib/prism.ts'])assert.equal(files.has(path),false,path);
 assert.equal(files.get('prism/Inactive.lex.tex'),'keep');assert.equal(files.get('prism/App.lex.tex'),module.trim()+'\n');
 assert.equal(h.options.source,source);assert.equal(h.options.rulesUpdate.state,null);assert.equal(h.options.rulesUpdate.expected.old,true);
 assert.equal(h.options.appUpdate.state.phase,'checked');assert.equal(h.log.forgot,1);
});
test('stateful to decision conversion removes stateful source standing and cached live owner',async()=>{
 const source=snapshot([['prism/App.lex.tex',module],['prism/app.json','{"phase":"proved"}'],['prism/Inactive.lex.tex','keep']],{phase:'proved',sourceKappa:oid});
 const h=transitionHarness(source),result=await h.rulesAfterTurn(7,`<prism-spec>${JSON.stringify(spec)}</prism-spec>`,[],{source});
 const files=new Map(result.source.files);
 assert.equal(files.has('prism/App.lex.tex'),false);assert.equal(files.has('prism/app.json'),false);assert.equal(files.has(decisionPath),true);
 assert.equal(files.get('prism/Inactive.lex.tex'),'keep');assert.equal(h.options.appUpdate.state,null);assert.equal(h.options.appUpdate.expected,source.applicationState);assert.equal(h.options.rulesUpdate.state.phase,'checked');assert.equal(h.log.forgot,1);
 const invalid=transitionHarness(source),refused=await invalid.rulesAfterTurn(7,'<prism-spec>{"invalid":true}</prism-spec>',[],{source});
 assert.equal(refused.changed,0);assert.equal(invalid.patch,undefined);assert.equal(refused.source,source);
 const same=transitionHarness(snapshot([['prism/spec.json',JSON.stringify(spec)],[decisionPath,canonical.source]]));
 await same.rulesAfterTurn(7,`<prism-spec>${JSON.stringify({...spec,title:'Updated'})}</prism-spec>`);
 assert.equal(same.log.forgot,0,'ordinary decision editing retains live undo/redo owner');
});
test('direct replacement of a stateful module uses the canonical decision adapter',()=>{
 const source=snapshot([['prism/App.lex.tex',module]]);
 const plan=planCanonicalEdit({filePath:'prism/App.lex.tex',content:canonical.source},source,{lexlean,prism});
 assert.equal(prism.parseSpecTag(plan.text).spec.name,spec.name);
});

// Transaction scheduler tests committed state, not just captured patch parameters.
function database(source,ruleState) {
 const tables=new Map(Object.entries({apps:new Map([[7,{name:source.name}]]),files:new Map(source.files.map(([p,v])=>['7:'+p,v])),refs:new Map([['7:main',source.reference],...(ruleState===undefined?[]:[['prism:7',ruleState]]),...(source.applicationState===undefined?[]:[['stateful:7',source.applicationState]])]),objects:new Map(),versions:new Map([[7,[]]])}));
 return{tables,transaction(){const copy=structuredClone(tables);let pending=0,aborted=false;const t={abort(){aborted=true;queueMicrotask(()=>t.onabort?.());},objectStore(name){const data=copy.get(name);const request=action=>{const q={};pending++;queueMicrotask(()=>{if(!aborted){q.result=action();q.onsuccess?.();}if(--pending===0&&!aborted){for(const[k,v]of copy)tables.set(k,v);t.oncomplete?.();}});return q;};return{
  get:key=>request(()=>structuredClone(data.get(key))),put:(value,key)=>request(()=>data.set(key,structuredClone(value))),delete:key=>request(()=>data.delete(key)),
  openCursor(){const rows=[...data],q={};let at=0;const advance=()=>{pending++;queueMicrotask(()=>{if(!aborted){q.result=at<rows.length?{key:rows[at][0],value:rows[at++][1],continue:advance}:null;q.onsuccess?.();}if(--pending===0&&!aborted){for(const[k,v]of copy)tables.set(k,v);t.oncomplete?.();}});};advance();return q;},
 };}};return t;}};
}
globalThis.IDBKeyRange={bound(){return{};}};
test('source and stateful lifecycle changes commit together and a lifecycle race rolls back',async()=>{
 const state={phase:'proving',sourceKappa:oid,approvalId:'old'},source=snapshot([['prism/App.lex.tex',module]],state),next={phase:'checked',sourceKappa:nextOid};
 const prepared={oid:nextOid,files:[['prism/App.lex.tex',module+'\n']],objects:[]};
 const db=database(source);
 await commitPreparedProject(db,7,['prism/App.lex.tex'],source,prepared,{appUpdate:{expected:state,state:next}});
 assert.deepEqual(db.tables.get('refs').get('stateful:7'),next);
 const capture=await captureProjectSource(db,7);assert.deepEqual(capture.applicationState,next);assert.equal(capture.files[0][1],module+'\n');
 const raced=database(source);raced.tables.get('refs').set('stateful:7',{...state,approvalId:'new'});const before=structuredClone(raced.tables);
 await assert.rejects(commitPreparedProject(raced,7,['prism/App.lex.tex'],source,prepared,{appUpdate:{expected:state,state:next}}),{code:'projectEditConflict'});assert.deepEqual(raced.tables,before);
});
test('generic source mutation invalidates stateful standing and shape conversion deletes rule standing',async()=>{
 const source=snapshot([['prism/App.lex.tex',module]],{phase:'proved',sourceKappa:oid});const db=database(source,{phase:'proved',old:true});
 const prepared={oid:nextOid,files:[['prism/App.lex.tex',module+'\n']],objects:[]};
 await commitPreparedProject(db,7,['prism/App.lex.tex'],source,prepared,{rulesUpdate:{expected:{phase:'proved',old:true},state:null}});
 assert.equal(db.tables.get('refs').has('stateful:7'),false);assert.equal(db.tables.get('refs').has('prism:7'),false);
});
test('protected state is source bound and imported state files never authenticate it',async()=>{
 const hash=await kappa(enc.encode(module)),state={phase:'proved',sourceKappa:hash,proof:{attestation_id:'verified'}};
 const source=snapshot([['prism/App.lex.tex',module],['prism/app.json',JSON.stringify(state)]],state);
 assert.equal((await boundApplicationState(source,kappa)).phase,'proved');
 assert.equal(await boundApplicationState({...source,applicationState:undefined},kappa),null);
 assert.equal(await boundApplicationState({...source,applicationState:{...state,sourceKappa:oid}},kappa),null);
 assert.equal(await boundApplicationState({...source,applicationState:{...state,proof:null}},kappa),null);
});
test('mixed source cannot expose or renew obsolete decision proof state',async()=>{
 const text=JSON.stringify(spec),state={phase:'proved',specKappa:await kappa(enc.encode(text))};
 const db=database(snapshot([['prism/spec.json',text],['prism/App.lex.tex',module]]),state);
 assert.equal((await readBoundRulesSnapshot(db,7,kappa)).state,null);
 await assert.rejects(readBoundRules(db,7,kappa),{code:'applicationShapeConflict'});
 assert.equal(await saveBoundRules(db,7,state,kappa,state),false);
});
test('committed shape changes forget old stateful runtime and decision live caches',async()=>{
 const start=host.indexOf('function forgetApplicationFrames('),end=host.indexOf('function pushLive(',start);let forgotten=0;
 const maps=Object.fromEntries(['liveLatest','livePending','liveEditors','livePointed','appApps'].map(name=>[name,new Map([[7,'old']]) ]));
 const run=vm.runInNewContext(host.slice(start,end)+';forgetApplicationFrames',{...maps,appFrames:Promise.resolve({forget:id=>{assert.equal(id,7);forgotten++;}})});
 run(7);await new Promise(setImmediate);for(const map of Object.values(maps))assert.equal(map.has(7),false);assert.equal(forgotten,1);
});
test('ordinary decision saves retain real undo redo history and reject unconditional invalidation',async()=>{
 async function verify(mutate=text=>text){
  const source=snapshot([['prism/spec.json',JSON.stringify(spec)],[decisionPath,canonical.source]]),editor=createEditor(spec,()=>({ok:true})),editors=new Map([[7,editor]]);
  const changed=editor.apply([{op:'set',path:['title'],value:'Changed'}]).spec;
  const first=host.indexOf('async function rulesAfterTurn('),last=host.indexOf('// The kernel\'s answer to a reword.',first),save=host.indexOf('async function saveSpec('),saveEnd=host.indexOf('async function patchSpec(',save);
  const run=vm.runInNewContext(mutate(host.slice(first,last))+host.slice(save,saveEnd)+';saveSpec',{
   Map,JSON,enc,appShape,lexlean,prism,live,modulePresentation,retiredApplicationPaths,boundApplicationState,core:{},coreReady:Promise.resolve(),dbReady:Promise.resolve(),assertGenerationActive(){},kappa,
   captureProjectSource:async()=>source,rulesSnapshotOf:async()=>({state:null,expected:undefined}),nextPhase:async()=> 'checked',now:()=> 'now',emit(){},
   applyStoredProjectPatch:async()=>({changed:2,source}),forgetApplicationFrames:id=>editors.delete(id),refreshLive:async id=>{if(editors.has(id))editors.get(id).sync(changed);else editors.set(id,createEditor(changed,()=>({ok:true})));},
  });
  await run(7,changed,{source});assert.equal(editors.get(7),editor);assert.equal(editor.canUndo,true);assert.equal(editor.undo().title,spec.title);assert.equal(editor.redo().title,'Changed');
 }
 await verify();
 await assert.rejects(verify(text=>text.replace("if (applied.changed && appShape.isAppShape(new Map(source.files).get(appShape.SOURCE_PATH)))",'if (applied.changed)')),{name:'AssertionError'});
});
test('actual portable publication and both generic build routes refuse stateful source',async()=>{
 const source=snapshot([['prism/App.lex.tex',module],['prism/app.json','{"phase":"proved"}']]);
 assert.throws(()=>assertCanonicalBuild(source.files),{code:'statefulBuildUnsupported'});
 const start=host.indexOf('handlers.set("holo:publish"'),end=host.indexOf('\nwindow.electron',start),handlers=new Map();
 vm.runInNewContext(host.slice(start,end),{handlers,coreReady:Promise.resolve(),performance:{now:()=>0},URL,BASE:'https://foundry.example/',core:{run:({op})=>op==='view'?{}:{decision:'Accept'}},fetch:async()=>({ok:true,json:async()=>({closure:'closure',attestation_id:'receipt'})}),verifiedBuildSource:async()=>source,requiresRulePublication,put:async()=>{},buildPortable(){assert.fail('generic publication');}});
 await assert.rejects(handlers.get('holo:publish')({appId:7}),{code:'statefulBuildUnsupported'});
 for(const name of ['buildPreview','buildPortable']){
  const first=host.indexOf(`async function ${name}(`),body=host.slice(first,first+450);
  assert.ok(body.includes("applicationShape(source) === 'stateful'"),name+' requires source-based refusal before build effects');
 }
});
test('the stateful lifecycle projection is not directly editable',()=>{
 assert.throws(()=>planCanonicalEdit({filePath:'prism/app.json',content:'{"phase":"proved"}'},{files:[]},{lexlean,prism}),{code:'applicationStateReadOnly'});
});
test('authored app.json cannot supply checked or proved standing',async()=>{
 const start=host.indexOf('async function appStateOf('),end=host.indexOf('// Approval and proof completion',start);
 for(const phase of ['checked','proved']){
  const captured={reference:'snapshot',files:[['prism/App.lex.tex',module],['prism/app.json',JSON.stringify({sourceKappa:'identity',phase})]]};
  const run=vm.runInNewContext(host.slice(start,end)+';appStateOf',{applicationShape,boundApplicationState,captureProjectSource:async()=>captured,dbReady:Promise.resolve({}),appShape,Map,JSON,enc:new TextEncoder(),kappa:async()=> 'identity'});
  const current=await run(7);assert.equal(current.state,null,'ordinary source cannot authenticate proof standing');
 }
});

export {host,module,spec,canonical,decisionPath};
