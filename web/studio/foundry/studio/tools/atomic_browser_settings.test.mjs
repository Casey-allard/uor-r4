import assert from 'node:assert/strict';
import fs from 'node:fs';
import {test} from 'node:test';
import {readUserSettings,changeUserSettings,replaceUserSettings} from '../shell/settings-transactions.mjs';
import {invokeHost} from '../shell/host-boundary.mjs';
function database(initial,{abort=false,readError=false,putError=false}={}){
 let state=structuredClone(initial),active=false;const queue=[],modes=[];
 const next=()=>{if(active||!queue.length)return;active=true;queue.shift()();};
 return {get state(){return structuredClone(state);},modes,transaction(name,mode){assert.equal(name,'settings');modes.push(mode);let request,nextValue,done=false;
  const finish=()=>{if(done)return;done=true;active=false;queueMicrotask(next);};
  const tx={abort(){if(done)return;queueMicrotask(()=>{tx.onabort?.();finish();});},objectStore(){return {get(key){assert.equal(key,'user');request={};queue.push(()=>queueMicrotask(()=>{if(readError){request.onerror?.();tx.abort();return;}request.result=structuredClone(state);request.onsuccess?.();if(mode==='readonly')queueMicrotask(()=>{if(abort)tx.abort();else{tx.oncomplete?.();finish();}});}));queueMicrotask(next);return request;},put(value,key){assert.equal(mode,'readwrite');assert.equal(key,'user');if(putError)throw Error('private storage');nextValue=structuredClone(value);const write={};queueMicrotask(()=>{write.onsuccess?.();if(abort){tx.abort();return;}state=nextValue;tx.oncomplete?.();finish();});return write;}};}};return tx;
 }};
}
test('input is captured before database readiness and returned records are detached',async()=>{
 let ready;const db=database({kept:1}),input={nested:{value:'original'}};const p=changeUserSettings(new Promise(r=>ready=r),input,(current,patch)=>({...current,...patch}));input.nested.value='changed';assert.equal(db.modes.length,0);ready(db);const value=await p;assert.equal(value.nested.value,'original');value.nested.value='caller';assert.equal(db.state.nested.value,'original');assert.equal(value.kept,1);
});
test('concurrent changes serialize their read and write without losing disjoint fields',async()=>{
 const db=database({left:true,right:true});await Promise.all([changeUserSettings(db,{left:false},(s,p)=>({...s,...p})),changeUserSettings(db,{right:false},(s,p)=>({...s,...p}))]);assert.deepEqual(db.state,{left:false,right:false});assert.deepEqual(db.modes,['readwrite','readwrite']);
 await replaceUserSettings(db,{});assert.deepEqual(await readUserSettings(db),{});assert.equal(db.modes.at(-1),'readonly');
});
test('late abort refuses reads and writes and never acknowledges an uncommitted value',async()=>{
 const db=database({kept:true},{abort:true});await assert.rejects(()=>replaceUserSettings(db,{kept:false}),{code:'settingsStorageFailed'});assert.deepEqual(db.state,{kept:true});await assert.rejects(()=>readUserSettings(db),{code:'settingsStorageFailed'});
});
test('invalid input and corrupt roots are distinct from missing settings',async()=>{
 for(const value of [null,[],1,'private',()=>{}])await assert.rejects(()=>replaceUserSettings(database({}),value),{code:'invalidSettingsCommand'});
 for(const value of [null,[],1,'private'])await assert.rejects(()=>readUserSettings(database(value)),{code:'invalidStoredSettings'});
 assert.equal(await readUserSettings(database(undefined)),null);
 const db=database({kept:1});await assert.rejects(()=>changeUserSettings(db,{},async()=>({next:1})),{code:'invalidSettingsCommand'});assert.deepEqual(db.state,{kept:1});
});
test('database and request failures are fixed typed refusals with rollback',async()=>{
 const rejected=Promise.reject(Error('private credential'));await assert.rejects(()=>readUserSettings(rejected),{code:'settingsStorageFailed'});
 for(const db of [{transaction(){throw Error('private database');}},database({kept:1},{readError:true}),database({kept:1},{putError:true})])await assert.rejects(()=>replaceUserSettings(db,{}),e=>e.code==='settingsStorageFailed'&&!e.message.includes('private'));
});
test('all real settings writers share the transaction boundary and public errors are modeled and redacted',async()=>{
 const read=p=>fs.readFileSync(new URL('../../'+p,import.meta.url),'utf8');const host=read('studio/shell/host.js'),inference=read('studio/shell/inference.js');assert.ok(host.includes('return changeUserSettings(dbReady, patch || {}'));
 for(const snippet of ['return readUserSettings(workspace())','return replaceUserSettings(workspace(),s)'])assert.ok(inference.includes(snippet),snippet);
 const spec=read('app/src/FoundrySpecification.lex.tex');assert.ok(spec.includes('name: browserSettingsTransactionBoundary,'));
 for(const code of ['invalidSettingsCommand','invalidStoredSettings','settingsStorageFailed']){assert.ok(spec.includes('name: '+code+','));const response=await invokeHost('set-user-settings',[],()=>{throw Object.assign(Error('private credential'),{code});});assert.equal(response.error.code,code);assert.ok(!JSON.stringify(response).includes('private credential'));}
 assert.ok(read('.github/workflows/studio.yml').includes('node tools/atomic_browser_settings_browser.mjs'));
});
