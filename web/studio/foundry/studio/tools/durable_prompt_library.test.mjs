import assert from 'node:assert/strict';
import fs from 'node:fs';
import {test} from 'node:test';
import {operatePromptLibrary,readPromptLibrary,createPromptLibrary} from '../shell/prompt-library.mjs';
import {invokeHost} from '../shell/host-boundary.mjs';
const input={title:'Reusable prompt',content:'Exact content\n',description:'Details',slug:'reusable'};
const create=(state,value=input)=>operatePromptLibrary(state,'create',value,new Date('2026-01-01'));
test('prompt CRUD preserves detached exact records Dates identity ordering and omitted fields',()=>{
 const a=create(),row=a.result;assert.equal(row.id,1);assert.equal(row.content,input.content);assert.ok(row.createdAt instanceof Date);row.content='caller mutation';assert.equal(a.journal.records[0].content,input.content);
 const b=operatePromptLibrary(a.journal,'update',{id:1,title:'Changed',slug:null},new Date('2025-01-01'));assert.equal(b.result,undefined);assert.equal(b.journal.records[0].content,input.content);assert.equal(b.journal.records[0].slug,null);assert.deepEqual(b.journal.records[0].updatedAt,a.journal.records[0].createdAt);
 const c=operatePromptLibrary(b.journal,'delete',1);assert.deepEqual(c.journal.records,[]);assert.equal(create(c.journal).result.id,2);assert.deepEqual(operatePromptLibrary(undefined,'list').result,[]);
});
test('invalid input missing IDs and duplicate slugs refuse without mutating source',()=>{
 const state=create().journal,original=structuredClone(state);
 for(const [kind,value,code] of [['create',{...input,title:''},'invalidPromptCommand'],['create',{...input,slug:'bad slug'},'invalidPromptCommand'],['create',{...input,content:'é'.repeat(32769)},'invalidPromptCommand'],['create',input,'promptSlugConflict'],['update',{id:8,title:'absent'},'promptMissing'],['delete',8,'promptMissing'],['delete',0,'invalidPromptCommand'],['list',{},'invalidPromptCommand'],['update',{id:1,createdAt:new Date()},'invalidPromptCommand']])assert.throws(()=>operatePromptLibrary(state,kind,value),{code});
 const second=create(state,{...input,slug:'other'});assert.throws(()=>operatePromptLibrary(second.journal,'update',{id:2,slug:'reusable'}),{code:'promptSlugConflict'});assert.deepEqual(state,original);
});
test('corrupt journals cannot become empty results and bounded capacity refuses',()=>{
 const state=create().journal;
 for(const mutate of [s=>s.records[0].createdAt='2026-01-01',s=>s.records[0].id=0,s=>s.nextId=1,s=>s.records.push({...s.records[0],id:2}),s=>s.records[0].content=null,s=>s.extra=true,s=>s.records[0].updatedAt=new Date('2020-01-01')]){const bad=structuredClone(state);mutate(bad);assert.throws(()=>readPromptLibrary(bad),{code:'promptLibraryInvalid'});}
 const exhausted={...state,nextId:Number.MAX_SAFE_INTEGER};assert.throws(()=>create(exhausted,{...input,slug:null}),{code:'promptLibraryLimit'});
 const full={...state,nextId:1001,records:Array.from({length:1000},(_,i)=>({...state.records[0],id:i+1,slug:null}))};assert.throws(()=>create(full),{code:'promptLibraryLimit'});
 let current;for(let i=0;i<63;i++)current=create(current,{title:'large',content:'x'.repeat(65536)}).journal;assert.throws(()=>create(current,{title:'large',content:'x'.repeat(65536)}),{code:'promptLibraryLimit'});
});
function database({abort=false}={}){let state,reads=0;return {get state(){return state;},get reads(){return reads;},transaction(){const tx={objectStore:()=>({get(){reads++;const request={};queueMicrotask(()=>{request.result=structuredClone(state);request.onsuccess();if(!tx.failed&&tx.next===undefined)queueMicrotask(()=>tx.oncomplete());});return request;},put(next){tx.next=next;queueMicrotask(()=>{if(abort){tx.abort();return;}state=structuredClone(next);tx.oncomplete();});}}),abort(){tx.failed=true;queueMicrotask(()=>tx.onabort());}};return tx;}};}
test('transaction completion controls acknowledgement and late abort preserves durable state',async()=>{
 const db=database(),call=createPromptLibrary(Promise.resolve(db));const row=await call('create',input);assert.equal(row.id,1);assert.deepEqual((await call('list'))[0],row);
 const failed=database({abort:true});await assert.rejects(()=>createPromptLibrary(failed)('create',input),{code:'promptLibraryStorageFailed'});assert.equal(failed.state,undefined);
 await assert.rejects(()=>createPromptLibrary({transaction(){throw Error('private detail');}})('list'),{code:'promptLibraryStorageFailed'});
});
test('caller input is captured before database readiness and validation precedes effects',async()=>{
 let ready;const db=database(),call=createPromptLibrary(new Promise(r=>ready=r)),value={...input};const pending=call('create',value);value.content='mutated';ready(db);assert.equal((await pending).content,input.content);
 await assert.rejects(()=>call('delete',-1),{code:'invalidPromptCommand'});assert.equal(db.reads,1);
});
test('typed failures redact content and actual host registration and acceptance cannot disappear',async()=>{
 const host=fs.readFileSync(new URL('../shell/host.js',import.meta.url),'utf8');for(const channel of ['list','create','update','delete'])assert.ok(host.includes('handlers.set("prompts:'+channel+'",'));
 const spec=fs.readFileSync(new URL('../../app/src/FoundrySpecification.lex.tex',import.meta.url),'utf8');assert.ok(spec.includes('name: promptLibraryBoundary,'));
 for(const code of ['invalidPromptCommand','promptLibraryInvalid','promptMissing','promptSlugConflict','promptLibraryLimit','promptLibraryStorageFailed']){assert.ok(spec.includes('name: '+code+','));const result=await invokeHost('prompts:create',[],()=>{throw Object.assign(Error('private prompt'),{code});});assert.equal(result.error.code,code);assert.ok(!JSON.stringify(result).includes('private prompt'));}
 assert.ok(fs.readFileSync(new URL('../../.github/workflows/studio.yml',import.meta.url),'utf8').includes('node tools/durable_prompt_library_browser.mjs'));
});
