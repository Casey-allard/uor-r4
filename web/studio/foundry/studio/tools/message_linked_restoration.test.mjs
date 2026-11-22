import assert from 'node:assert/strict';
import fs from 'node:fs';
import {test} from 'node:test';
import {captureRestoreChat,messageSourceVersion,prepareMessageRestoration} from '../shell/message-restoration.mjs';
import {emptyChatQueue} from '../shell/chat-queue-store.mjs';
import {commitPreparedProject} from '../shell/project-edit.mjs';
const a='blake3:'+'a'.repeat(64),b='blake3:'+'b'.repeat(64),c='blake3:'+'c'.repeat(64);
const initial=()=>({id:1,appId:2,title:'Conversation',initialCommitHash:a,chatMode:'build',modelSelection:{provider:'local',name:'model'},referencedApps:[3],messages:[{id:1,role:'user',content:'first',createdAt:'earlier'},{id:2,role:'assistant',content:'answer',commitHash:b,sourceCommitHash:a,createdAt:'earlier',usingFreeAgentModeQuota:true,chatTurnIntentId:'live-dedupe'},{id:3,role:'user',content:'restore boundary',createdAt:'later'},{id:4,role:'assistant',content:'last answer',sourceCommitHash:c,createdAt:'later'}]});
// Ordering/rollback fixture. Browser acceptance uses actual IndexedDB.
function database(chat=initial()){
 let data={apps:new Map([[2,{id:2,name:'fixture',extraDeps:[]}]]),files:new Map(),objects:new Map(),refs:new Map([['2:main',a]]),versions:new Map([[2,[{oid:a}]]]),chats:new Map([[1,chat]])},writes=0;
 const db={failNext:false,get data(){return data;},get writes(){return writes;},transaction(names,mode){let pending=0,ended=false;const draft=structuredClone(data),late=db.failNext;db.failNext=false;
  const finish=()=>queueMicrotask(()=>{if(ended||pending)return;ended=true;if(late){tx.onabort?.();return;}if(mode==='readwrite'){data=draft;writes++;}tx.oncomplete?.();});
  const req=fn=>{const q={onsuccess:null,onerror:null};pending++;queueMicrotask(()=>{if(ended)return;try{q.result=fn(q);q.onsuccess?.();pending--;finish();}catch{tx.abort();}});return q;};
  const tx={abort(){if(ended)return;ended=true;queueMicrotask(()=>tx.onabort?.());},objectStore(name){const table=draft[name];return {get:key=>req(()=>structuredClone(table.get(key))),getAll:()=>req(()=>structuredClone([...table.values()])),getAllKeys:()=>req(()=>[...table.keys()]),put(value,key){return req(()=>{table.set(key,structuredClone(value));return key;});},add(value,key){return req(()=>{if(table.has(key))throw Error();table.set(key,structuredClone(value));return key;});},delete:key=>req(()=>table.delete(key)),openCursor(range){const rows=[...table].filter(([k])=>k>=range.lower&&k<range.upper);let index=0;const q={};const step=()=>{pending++;queueMicrotask(()=>{if(ended)return;q.result=index<rows.length?{key:rows[index][0],value:rows[index][1],continue(){index++;step();}}:null;q.onsuccess?.();pending--;finish();});};step();return q;}};}};finish();return tx;}};return db;
}
globalThis.IDBKeyRange={bound:(lower,upper)=>({lower,upper})};
const boundary={resolve:async x=>x,verify:async()=>{}};
const request={appId:2,chatId:1,messageId:3,mode:'fork',restoreCodebase:true};
const captured={name:'fixture',reference:a,files:[],dependencies:[]},prepared={name:'fixture',oid:b,files:[['new.txt','new source']],objects:[]};
const preview=()=>({profile:'foundry/version-preview/1',appId:2,state:{revision:4,active:0,owner:0,origin:Array(8).fill(0),current:Array(8).fill(0)},receipts:[{operationId:'restore',identity:c}]});
const commit=(db,change)=>commitPreparedProject(db,2,['new.txt'],captured,prepared,{recordVersion:true,chatUpdate:change,previewUpdate:{expected:undefined,state:preview()}});
test('source selection uses following source then preceding commit then initial identity',()=>{
 const chat=initial();assert.equal(messageSourceVersion(chat,2),c);delete chat.messages[3].sourceCommitHash;assert.equal(messageSourceVersion(chat,2),b);delete chat.messages[1].commitHash;assert.equal(messageSourceVersion(chat,2),a);chat.initialCommitHash=null;assert.throws(()=>messageSourceVersion(chat,2),{code:'messageRestoreVersionMissing'});
 assert.throws(()=>captureRestoreChat(initial(),3,1,3),{code:'messageRestoreMissing'});const duplicate=initial();duplicate.messages[1].id=1;assert.throws(()=>captureRestoreChat(duplicate,2,1,3),{code:'invalidMessageRestore'});
});
test('fork captures earlier messages preserves metadata and excludes live delivery quota identities',async()=>{
 const db=database(),result=await prepareMessageRestoration(db,request,boundary),next=result.chatUpdate.next;
 assert.equal(result.versionId,c);assert.equal(next.messages.length,2);assert.equal(next.messages[1].commitHash,b);assert.equal(next.messages[1].usingFreeAgentModeQuota,false);assert.equal(next.messages[1].chatTurnIntentId,undefined);assert.deepEqual(next.referencedApps,[3]);assert.deepEqual(next.modelSelection,initial().modelSelection);assert.equal(next.chatMode,'build');assert.equal(db.writes,0);
 const raw={...request},pending=prepareMessageRestoration(db,raw,boundary);raw.messageId=1;assert.equal((await pending).chatUpdate.next.messages.length,2);
});
test('truncation and fork-only remain distinct and busy missing or unknown version cannot mutate',async()=>{
 const db=database();const truncated=await prepareMessageRestoration(db,{...request,mode:'truncate',versionId:a},boundary);assert.equal(truncated.versionId,a);assert.equal(truncated.chatUpdate.next.id,1);assert.equal(truncated.chatUpdate.next.messages.length,2);
 const fork=await prepareMessageRestoration(db,{...request,restoreCodebase:false},{resolve:()=>assert.fail(),verify:()=>assert.fail()});assert.equal(fork.versionId,null);
 for(const state of [{active:1,items:[1]},{active:0,items:[1]}]){const j=emptyChatQueue(1,2);j.state={...j.state,...state,owner:state.active?7:0};j.nextItem=2;j.entries=[{id:1,intent:{intentId:'queued',chatId:1},identity:'bound'}];db.data.refs.set('chat-queue:1',j);await assert.rejects(()=>prepareMessageRestoration(db,request,boundary),{code:'messageRestoreBusy'});}db.data.refs.delete('chat-queue:1');
 await assert.rejects(()=>prepareMessageRestoration(db,{...request,messageId:99},boundary),{code:'messageRestoreMissing'});await assert.rejects(()=>prepareMessageRestoration(db,request,{...boundary,verify:()=>{throw Object.assign(Error(),{code:'missingSnapshotObject'});}}),{code:'missingSnapshotObject'});assert.equal(db.writes,0);
});
test('source journal fork allocation and copied messages commit in one transaction',async()=>{
 const db=database(),{chatUpdate}=await prepareMessageRestoration(db,request,boundary),result=await commit(db,chatUpdate);
 assert.deepEqual(result,{oid:b,createdChatId:2,affectedChatId:null});assert.equal(db.writes,1);assert.equal(db.data.refs.get('2:main'),b);assert.equal(db.data.files.get('2:new.txt'),'new source');assert.deepEqual(db.data.chats.get(1),initial());assert.equal(db.data.chats.get(2).initialCommitHash,b);assert.equal(db.data.chats.get(2).messages.length,2);assert.equal(db.data.refs.get('allocation:chats'),2);assert.equal(db.data.refs.get('preview:2').receipts[0].result.createdChatId,2);
});
test('late abort stale history queue admission and deletion refuse all partial source changes',async()=>{
 for(const mode of ['abort','history','queue','delete']){const db=database(),{chatUpdate}=await prepareMessageRestoration(db,request,boundary);if(mode==='abort')db.failNext=true;if(mode==='history')db.data.chats.get(1).title='changed';if(mode==='queue')db.data.refs.set('chat-queue:1',{state:{active:1}});if(mode==='delete')db.data.chats.delete(1);await assert.rejects(()=>commit(db,chatUpdate),{code:mode==='abort'?'projectEditStorageFailed':'projectEditConflict'});assert.equal(db.data.refs.get('2:main'),a);assert.equal(db.data.files.size,0);assert.equal(db.data.chats.has(2),false);assert.equal(db.data.refs.has('allocation:chats'),false);assert.equal(db.writes,0);}
});
test('invalid commands storage failure and canonical transport registration have negative controls',async()=>{
 for(const patch of [{appId:0},{mode:'unknown'},{messageId:0},{restoreCodebase:1},{extra:true}])await assert.rejects(()=>prepareMessageRestoration(database(),{...request,...patch},boundary),{code:'invalidMessageRestore'});
 await assert.rejects(()=>prepareMessageRestoration({transaction(){throw Error('private');}},request,boundary),{code:'messageRestoreStorageFailed'});
 const read=p=>fs.readFileSync(new URL(p,import.meta.url),'utf8');assert.ok(read('../shell/host.js').includes('prepareMessageCommand:async'));assert.ok(read('../../app/src/FoundrySpecification.lex.tex').includes('name: messageRestorationBoundary,'));assert.ok(read('./shell_parse.mjs').includes('message-restoration.mjs'));
});
