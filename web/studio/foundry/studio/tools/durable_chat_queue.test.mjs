import assert from 'node:assert/strict';
import fs from 'node:fs';
import {createHash} from 'node:crypto';
import {test} from 'node:test';
import {emptyChatQueue,readChatQueue,captureQueueIntent,bindQueuePayload,stageChatQueue,queueSnapshot,commitStoredChatQueue,readStoredChatQueue} from '../shell/chat-queue-store.mjs';
import {encodeQueueRequest} from '../shell/chat-queue-codec.mjs';
import {createChatQueueController} from '../shell/chat-queue-controller.mjs';
import {SerializableChatTurnIntentSchema,ChatStreamIntentEventSchema,serializeImmutableChatTurnPayload} from '../shell/chat-contracts.mjs';
const hash=b=>createHash('sha256').update(b).digest('hex');
const boundary={validate:x=>SerializableChatTurnIntentSchema.safeParse(x),validateEvent:x=>ChatStreamIntentEventSchema.safeParse(x),serialize:serializeImmutableChatTurnPayload,derive:async b=>'test-sha256:'+hash(b)};
const vectors=JSON.parse(fs.readFileSync(new URL('../../verification/chat-queue-vectors.json',import.meta.url))).vectors;
const vector=name=>vectors.find(v=>v.name===name),enc=new TextEncoder(),dec=new TextDecoder();
// Literal wire corpus is an adapter fixture, never a transition implementation or guest acceptance.
const fixtureGuest={dispatch:async bytes=>{const row=vectors.find(v=>v.request===dec.decode(bytes));assert.ok(row,'unregistered fixture request');return enc.encode(row.response);}};
async function intent(n=11){return bindQueuePayload({schemaVersion:1,intentId:`intent-${n}`,chatId:1,appId:2,invocationRef:{kind:'chat-stream',entityKey:1,operationId:`op-${n}`},prompt:`Prompt ${n}`,redo:false,selectedComponents:[],requestedChatMode:'build'},boundary.serialize);}
async function journal(name='enqueue'){const j=emptyChatQueue(1,2);j.state=structuredClone(vector(name).input.state);j.nextItem=Math.max(0,...j.state.items)+1;for(const n of j.state.items)j.entries.push({id:n,...await captureQueueIntent(await intent(n),1,2,boundary)});return j;}
const refuses=(fn,code)=>assert.rejects(fn,{code});
test('imported schemas and immutable payload serialization are byte-bound to locked sources',()=>{
 const root=new URL('../',import.meta.url),lock=JSON.parse(fs.readFileSync(new URL('CHAT_CONTRACTS_LOCK.json',root)));
 assert.equal(hash(fs.readFileSync(new URL('vendor/prism/package-lock.json',root))),lock.packageLockSha256);
 assert.equal(hash(fs.readFileSync(new URL('tools/chat_contracts_projection.mjs',root))),lock.generatorSha256);
 assert.equal(hash(fs.readFileSync(new URL('shell/chat-contracts.mjs',root))),lock.output.sha256);
 assert.equal(hash(fs.readFileSync(new URL('shell/chat-contracts.LICENSE.txt',root))),lock.licenseSha256);
 for(const row of lock.inputs.filter(r=>!r.path.startsWith('node_modules/')))assert.equal(hash(fs.readFileSync(new URL('vendor/prism/'+row.path,root))),row.sha256,row.path);
 assert.ok(fs.readFileSync(new URL('../.github/workflows/studio.yml',root),'utf8').includes('node tools/durable_chat_queue_browser.mjs'));
 assert.equal(ChatStreamIntentEventSchema.safeParse({type:'PAUSE_QUEUE'}).success,false);
 assert.ok(fs.readFileSync(new URL('../.github/workflows/studio.yml',root),'utf8').includes('node tools/chat_contracts_projection.mjs --check'));
});
test('payload capture rejects tampering cross-chat identities and private identity failures',async()=>{
 const raw=await intent(),pending=captureQueueIntent(raw,1,2,boundary);raw.prompt='changed';assert.equal((await pending).intent.prompt,'Prompt 11');
 await refuses(()=>captureQueueIntent(raw,1,2,boundary),'invalidChatQueueCommand');
 for(const patch of [{chatId:3},{appId:3},{invocationRef:{kind:'chat-stream',entityKey:3,operationId:'x'}},{schemaVersion:2}])await refuses(async()=>captureQueueIntent(await bindQueuePayload({...await intent(),...patch},boundary.serialize),1,2,boundary),'invalidChatQueueCommand');
 await refuses(async()=>captureQueueIntent(await intent(),1,2,{...boundary,derive:()=>{throw Error('private')}}),'chatQueueUnavailable');
});
test('guest-approved edits reorder clear receipts and replay bind exact durable payloads',async()=>{
 const operations={enqueue:{type:'enqueue',intent:await intent(14),observedStop:2},remove:{type:'remove',item:12},reorder:{type:'reorder',item:13,index:0},edit:{type:'edit',item:12,prompt:'Edited',selectedComponents:[]},clear:{type:'clear'},clearRetainsActive:{type:'clear'},pause:{type:'pause'},resume:{type:'resume',observedStop:2},stop:{type:'stop',observedStop:2},claim:{type:'claim',owner:7,observedStop:2},complete:{type:'complete',owner:7,item:11,outcome:'completed'},recover:{type:'recover',owner:8},fail:{type:'fail',owner:7}};
 for(const [name,op] of Object.entries(operations)){const original=await journal(name),before=structuredClone(original),result=await stageChatQueue(fixtureGuest,original,op,boundary);assert.deepEqual(original,before);assert.deepEqual(result.journal.state,vector(name).expected,name);assert.deepEqual(result.journal.entries.map(e=>e.id),vector(name).expected.items);if(name==='edit')assert.equal(result.journal.entries[1].intent.prompt,'Edited');if(name==='complete')assert.equal(result.journal.receipts[0].outcome,'completed');if(name==='recover'||name==='fail')assert.equal(result.journal.pauseReason,'stop');}
 const j=await journal();assert.equal((await stageChatQueue({dispatch:()=>assert.fail()},j,{type:'enqueue',intent:await intent(11),observedStop:2},boundary)).replayed,true);
 await refuses(async()=>stageChatQueue(fixtureGuest,j,{type:'enqueue',intent:await bindQueuePayload({...await intent(11),prompt:'conflict'},boundary.serialize),observedStop:2},boundary),'chatQueueIntentConflict');
 assert.deepEqual(queueSnapshot(await journal('complete')).queue.map(e=>e.itemId),['queue-12','queue-13']);
});
test('stale revisions Stop ownership and corrupt journals refuse without mutation',async()=>{
 for(const [name,op,code] of [['staleRevision',{type:'pause',expectedRevision:2},'queueRevisionConflict'],['wrongOwner',{type:'complete',item:11,owner:8},'queueOwnerConflict'],['activeRemove',{type:'remove',item:11},'queueActiveConflict'],['staleResume',{type:'resume',observedStop:1},'queueStopConflict']]){const j=await journal(name),before=structuredClone(j);await refuses(()=>stageChatQueue(fixtureGuest,j,op,boundary),code);assert.deepEqual(j,before);}
 const exhausted=emptyChatQueue(1,2);exhausted.nextItem=0x100000000;await refuses(async()=>stageChatQueue(fixtureGuest,exhausted,{type:'enqueue',intent:await intent(),observedStop:0},boundary),'chatQueueLimitExceeded');
 const j=await journal();await refuses(()=>stageChatQueue(fixtureGuest,j,{type:'stop',observedStop:1},boundary),'queueStopConflict');
 for(const patch of [{nextItem:1},{entries:[]},{receipts:[{}]},{state:{...j.state,owner:3}}])assert.throws(()=>readChatQueue({...j,...patch},1,2),{code:'chatQueueJournalInvalid'});
 j.entries[0].identity='tampered';await refuses(()=>stageChatQueue(fixtureGuest,j,{type:'pause'},boundary),'chatQueueJournalInvalid');
});
test('transaction commit is the only success and late abort CAS and deletion stay typed',async()=>{
 const j=emptyChatQueue(1,2);
 await refuses(()=>readStoredChatQueue({transaction(){throw Error('private');}},1),'chatQueueStorageFailed');
 function database({old,missing=false,late=false}={}){let puts=0;const tx={abort(){queueMicrotask(()=>tx.onabort());},objectStore(name){return {get(){const req={};queueMicrotask(()=>{req.result=name==='chats'?(missing?undefined:{appId:2}):name==='apps'?{}:old;req.onsuccess();});return req;},put(){puts++;queueMicrotask(()=>late?tx.onabort():tx.oncomplete());}};}};return {transaction:()=>tx,get puts(){return puts;}};}
 const db=database();assert.deepEqual(await commitStoredChatQueue(db,undefined,j),j);assert.equal(db.puts,1);
 await refuses(()=>commitStoredChatQueue(database({late:true}),undefined,j),'chatQueueStorageFailed');
 const stale=database({old:j});await refuses(()=>commitStoredChatQueue(stale,undefined,j),'chatQueueConflict');assert.equal(stale.puts,0);
 const missing=database({missing:true});await refuses(()=>commitStoredChatQueue(missing,undefined,j),'chatQueueMissing');assert.equal(missing.puts,0);
});
test('controller recovers abandoned ownership paused and refuses unavailable execution',async()=>{
 let j=await journal('recover'),commits=0,runs=0;const published=[];
 const locks={request:async(_name,options,cb)=>typeof options==='function'?options():cb({})};
 const controller=createChatQueueController({store:{read:async()=>({expected:structuredClone(j),journal:structuredClone(j)}),commit:async(expected,next)=>{assert.deepEqual(expected,j);j=structuredClone(next);commits++;return j;}},guest:fixtureGuest,boundary:async()=>boundary,locks,publish:(_id,snapshot)=>published.push(snapshot),run:async()=>{runs++;},mintOwner:()=>8});
 await controller.snapshot(1);await controller.idle(1);assert.equal(commits,1);assert.equal(runs,0);assert.equal(j.state.paused,1);assert.equal(j.entries.length,3);assert.equal(published.at(-1).queuePauseReason,'stop');
 await refuses(()=>controller.dispatch(1,{type:'PAUSE_QUEUE'}),'invalidChatQueueCommand');controller.dispose();await refuses(()=>controller.snapshot(1),'chatQueueUnavailable');
 // Literal expected states cover concurrent submission and Stop during an owned turn.
 j=await journal('claim');let admitted;const started=new Promise(resolve=>admitted=resolve);let executions=0;
 const states=[{revision:3,paused:0,stopPolicy:2,active:0,owner:0,items:[11,12,13]},{revision:4,paused:0,stopPolicy:2,active:11,owner:7,items:[11,12,13]},{revision:5,paused:0,stopPolicy:2,active:11,owner:7,items:[11,12,13,14]},{revision:6,paused:1,stopPolicy:3,active:11,owner:7,items:[11,12,13,14]},{revision:7,paused:1,stopPolicy:3,active:0,owner:0,items:[12,13,14]}];
 const commands=[{kind:9,owner:7,observedStop:2},{kind:1,item:14},{kind:8},{kind:10,item:11,owner:7}];let step=0;
 const encodeState=x=>'ok '+[1,x.revision,x.paused,x.stopPolicy,x.active,x.owner,x.items.length,...x.items].map(n=>{const b=Buffer.alloc(4);b.writeUInt32LE(n);return b.toString('hex');}).join('');
 const traceGuest={dispatch:async bytes=>{assert.ok(step<commands.length);assert.equal(dec.decode(bytes),dec.decode(encodeQueueRequest({state:states[step],expectedRevision:states[step].revision,command:{item:0,index:0,owner:0,observedStop:0,...commands[step]}})));return enc.encode(encodeState(states[++step]));}};
 const active=createChatQueueController({store:{read:async()=>({expected:structuredClone(j),journal:structuredClone(j)}),commit:async(expected,next)=>{assert.deepEqual(expected,j);j=structuredClone(next);return j;}},guest:traceGuest,boundary:async()=>boundary,locks,publish:()=>{},mintOwner:()=>7,run:async(_id,_intent,signal)=>{executions++;admitted();await new Promise(resolve=>signal.addEventListener('abort',resolve,{once:true}));return {outcome:'cancelled'};}});
 await active.snapshot(1);await started;
 await active.dispatch(1,{type:'SUBMIT',intent:await intent(14),observedStopPolicyVersion:2});assert.equal(j.entries.length,4);assert.equal(executions,1);
 await refuses(()=>active.dispatch(1,{type:'CANCEL',invocationRef:{kind:'chat-stream',entityKey:1,operationId:'obsolete'},observedStopPolicyVersion:2}),'queueActiveConflict');
 await active.dispatch(1,{type:'CANCEL',invocationRef:(await intent(11)).invocationRef,observedStopPolicyVersion:2});await active.idle(1);
 assert.equal(step,4);assert.equal(executions,1);assert.deepEqual(j.state,states[4]);assert.equal(j.receipts[0].outcome,'cancelled');active.dispose();

});
