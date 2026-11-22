import assert from 'node:assert/strict';
import fs from 'node:fs';
import {test} from 'node:test';
import {updateChatPreferences} from '../shell/chat-preferences.mjs';
import {invokeHost} from '../shell/host-boundary.mjs';
const model={provider:'featherless',name:'fixture-selected',effortLevel:'medium'};
const original={id:1,appId:2,title:'Before',chatMode:'build',messages:[{role:'user',content:'keep'}],untouched:true};
const turn=()=>new Promise(resolve=>setImmediate(resolve));
function database(initial=original,{hold=false,abort=false,missingApp=false,missingChat=false,badApp=false,putError=false}={}){
 let state=structuredClone(initial),transactions=0,release;const modes=[];
 return {get state(){return state;},get transactions(){return transactions;},get modes(){return modes;},finish(){release?.();},transaction(stores,mode){
  transactions++;modes.push(mode);assert.deepEqual(stores,['apps','chats']);let next;
  const tx={abort(){queueMicrotask(()=>tx.onabort?.());},objectStore(name){return {get(key){const request={};queueMicrotask(()=>{request.result=name==='chats'?(missingChat?undefined:structuredClone(state)):missingApp?undefined:badApp?{id:9}:{id:2};request.onsuccess?.();});return request;},put(value,key){assert.equal(name,'chats');assert.equal(key,1);if(putError)throw Error('private write detail');next=structuredClone(value);const request={};queueMicrotask(()=>{request.onsuccess?.();release=()=>{if(abort)tx.onabort?.();else{state=next;tx.oncomplete?.();}};if(!hold)release();});return request;}};}};
  return tx;
 }};
}
test('imported preferences capture before readiness and preserve unrelated transcript fields',async()=>{
 const db=database();let ready;const input={chatId:1,title:'',chatMode:'ask',modelSelection:structuredClone(model)};
 const change=updateChatPreferences(new Promise(resolve=>ready=resolve),input);input.modelSelection.name='caller-change';input.title='caller-change';assert.equal(db.transactions,0);ready(db);assert.equal(await change,undefined);
 assert.deepEqual(db.state,{...original,title:'',chatMode:'ask',modelSelection:model});assert.deepEqual(db.modes,['readwrite']);
 await updateChatPreferences(db,{chatId:1,chatMode:null});assert.equal(db.state.chatMode,null);assert.deepEqual(db.state.modelSelection,model);assert.deepEqual(db.state.messages,original.messages);
});
test('transaction completion is required and late abort never commits or acknowledges',async()=>{
 for(const abort of [false,true]){
  const db=database(original,{hold:true,abort});let settled=false;
  const result=updateChatPreferences(db,{chatId:1,title:'After'});const observed=result.then(()=>{settled=true;return null;},error=>{settled=true;return error;});
  await turn();assert.equal(settled,false);assert.deepEqual(db.state,original);db.finish();const error=await observed;
  if(abort){assert.equal(error.code,'chatPreferencesStorageFailed');assert.deepEqual(db.state,original);}else{assert.equal(error,null);assert.equal(db.state.title,'After');}
 }
});
test('invalid schema identity missing records and private storage failures stay typed',async()=>{
 for(const input of [null,{}, {chatId:0,title:'x'}, {chatId:1.5,title:'x'}, {chatId:1,chatMode:'unknown'}, {chatId:1,modelSelection:{provider:'x',name:'x'}}, {chatId:1,title:null}]){
  const db=database();await assert.rejects(()=>updateChatPreferences(db,input),{code:'invalidChatPreferences'});assert.equal(db.transactions,0);
 }
 for(const [db,code] of [[database(original,{missingChat:true}),'chatPreferencesMissing'],[database(original,{missingApp:true}),'chatPreferencesMissing'],[database({...original,id:9}),'chatPreferencesStateInvalid'],[database({...original,appId:'2'}),'chatPreferencesStateInvalid'],[database(original,{badApp:true}),'chatPreferencesStateInvalid'],[database(original,{putError:true}),'chatPreferencesStorageFailed'],[{transaction(){throw Error('private database');}},'chatPreferencesStorageFailed']])await assert.rejects(()=>updateChatPreferences(db,{chatId:1,title:'After'}),error=>error.code===code&&!error.message.includes('private'));
 await assert.rejects(()=>updateChatPreferences(Promise.reject(Error('private open')),{chatId:1,title:'After'}),{code:'chatPreferencesStorageFailed'});
});
test('actual host schema projection canonical ownership and browser gates are required',async()=>{
 const read=p=>fs.readFileSync(new URL('../../'+p,import.meta.url),'utf8');
 assert.ok(read('studio/shell/host.js').includes('updateChatPreferences(dbReady,input)'));
 assert.ok(read('studio/tools/chat_contracts_projection.mjs').includes('UpdateChatParamsSchema'));
 const spec=read('app/src/FoundrySpecification.lex.tex');assert.ok(spec.includes('name: chatPreferencesBoundary,'));
 for(const code of ['invalidChatPreferences','chatPreferencesMissing','chatPreferencesStateInvalid','chatPreferencesStorageFailed']){
  assert.ok(spec.includes('name: '+code+','));const response=await invokeHost('update-chat',[],()=>{throw Object.assign(Error('private credential'),{code});});assert.equal(response.error.code,code);assert.ok(!JSON.stringify(response).includes('private credential'));
 }
 const {commandRegistryData}=await import('../shell/command-registry-data.mjs');assert.equal(commandRegistryData.commands.find(x=>x.channel==='update-chat').owner,'FoundrySpecification.chatPreferencesBoundary');
 assert.ok(read('.github/workflows/studio.yml').includes('node tools/atomic_chat_preferences_browser.mjs'));
 assert.ok(read('studio/tools/shell_parse.mjs').includes('chat-preferences.mjs'));
});
