import assert from 'node:assert/strict';
import fs from 'node:fs';
import {test} from 'node:test';
import {createWindowCoordinator,readWindowJournal,windowJournalStore} from '../shell/window-coordination.mjs';
import {VisibleEntitySchema,ChatTabOwnershipSchema,WindowInterestSchema,QueryInvalidationScopeSchema,QueryInvalidationBatchSchema} from '../shell/chat-contracts.mjs';
const schemas={visible:VisibleEntitySchema.array().max(100),tabs:ChatTabOwnershipSchema.array().max(100),interest:WindowInterestSchema,scopes:QueryInvalidationScopeSchema.array().min(1).max(128),batch:QueryInvalidationBatchSchema};
let counter=0;const mint=()=>`00000000-0000-4000-8000-${String(++counter).padStart(12,'0')}`;
function fixture(){let value,fail=false;const held=new Map(),channels=new Set(),events=[];
 const locks={async query(){return {held:[...held.keys()].map(name=>({name}))};},async request(name,options,fn){if(typeof options==='function'){fn=options;options={};}if(options.ifAvailable&&held.has(name))return fn(null);while(held.has(name))await held.get(name).done;let release;const done=new Promise(resolve=>release=resolve);held.set(name,{done});try{return await fn({name});}finally{held.delete(name);release();}}};
 const store={read:async()=>structuredClone(value),commit:async(expected,next)=>{assert.deepEqual(expected,value);if(fail)throw Object.assign(Error(),{code:'windowCoordinationStorageFailed'});value=structuredClone(next);return next;}};
 function make(id){const memory=new Map(id?[['holo.window.session',id]]:[]),callbacks=[];const channel={postMessage(data){for(const peer of channels)if(peer!==channel)queueMicrotask(()=>peer.receive(structuredClone(data)));},receive(data){for(const f of callbacks)f({data});},addEventListener(_,f){callbacks.push(f);},close(){channels.delete(channel);}};channels.add(channel);const messages=[];events.push(messages);const api=createWindowCoordinator({store,locks,channel,storage:{getItem:k=>memory.get(k),setItem:(k,v)=>memory.set(k,v)},mintId:mint,validate:(kind,x)=>schemas[kind].safeParse(x),deliver:(name,payload)=>messages.push({name,payload}),exists:async entity=>entity.id<10});return {api,messages,channel};}
 return {make,store,locks,get value(){return value;},set value(next){value=next;},set fail(v){fail=v;}};
}
const tick=()=>new Promise(resolve=>setTimeout(resolve,0));
test('duplicated browser session identities obtain distinct live locks and committed bootstrap state',async()=>{
 const f=fixture(),id=mint(),a=f.make(id),b=f.make(id);const [one,two]=await Promise.all([a.api.bootstrap(),b.api.bootstrap()]);assert.equal(one.windowSessionId,id);assert.notEqual(two.windowSessionId,id);assert.equal(f.value.windows.length,2);assert.equal((await f.locks.query()).held.length,2);a.api.dispose();b.api.dispose();
});
test('focus visibility interests and tab ownership mutate captured bounded records',async()=>{
 const f=fixture(),a=f.make(),b=f.make();await Promise.all([a.api.ready,b.api.ready]);await a.api.setFocused();const entities=[{kind:'app',id:2}],pending=a.api.setVisible(entities);entities[0].id=8;await pending;const tab={tabInstanceId:mint(),chatId:1};await a.api.setTabs([tab]);await assert.rejects(()=>b.api.setTabs([tab]),{code:'windowTabOwnershipConflict'});await a.api.attach({kind:'chat-chunk',chatId:1});const row=f.value.windows.find(w=>w.id===a.api.sessionId);assert.deepEqual(row.visible,[{kind:'app',id:2}]);assert.ok(row.focused>0);assert.equal(row.interests.length,1);await a.api.detach({kind:'chat-chunk',chatId:1});assert.equal(f.value.windows.find(w=>w.id===a.api.sessionId).interests.length,0);a.api.dispose();b.api.dispose();
});
test('durable invalidations reach peers and bootstrap recovers a bounded history gap',async()=>{
 const f=fixture(),a=f.make(),b=f.make();await Promise.all([a.api.ready,b.api.ready]);await a.api.invalidate([{family:'apps'}]);await tick();assert.ok(b.messages.some(x=>x.name==='window:query-invalidations'&&x.payload.invalidations[0].epoch===1));
 for(let i=0;i<257;i++)await a.api.invalidate([{family:'chats'}]);const boot=await b.api.bootstrap({lastSeenQueryInvalidationEpoch:0});assert.equal(f.value.invalidations.length,256);assert.equal(boot.missedInvalidations.length,0);assert.ok(boot.recoveryScopes.some(x=>x.family==='chats'));await assert.rejects(()=>b.api.bootstrap({lastSeenQueryInvalidationEpoch:999}),{code:'invalidWindowCoordination'});a.api.dispose();b.api.dispose();
});
test('only allowlisted stream events reach matching interests without relay loops',async()=>{
 const f=fixture(),a=f.make(),b=f.make();await Promise.all([a.api.ready,b.api.ready]);await b.api.attach({kind:'chat-chunk',chatId:1});a.api.relay('chat:response:chunk',{chatId:1,messages:[]});a.api.relay('chat:response:chunk',{chatId:2,messages:[]});a.api.relay('provider:credentials',{chatId:1,secret:'private'});await tick();assert.equal(b.messages.filter(x=>x.name==='chat:response:chunk').length,1);assert.equal(a.messages.length,0);assert.ok(!b.messages.some(x=>x.name==='provider:credentials'));await b.api.detach({kind:'chat-chunk',chatId:1});a.api.relay('chat:stream:end',{chatId:1});await tick();assert.ok(!b.messages.some(x=>x.name==='chat:stream:end'));a.api.dispose();b.api.dispose();
});
test('malformed journal missing entities duplicate tabs and failed commits never acknowledge',async()=>{
 for(const value of [{}, {...readWindowJournal(undefined),extra:true},{...readWindowJournal(undefined),epoch:1}])assert.throws(()=>readWindowJournal(value),{code:'invalidWindowCoordination'});
 const f=fixture(),a=f.make();await a.api.ready;await assert.rejects(()=>a.api.setVisible([{kind:'app',id:100}]),{code:'invalidWindowCoordination'});const tab={tabInstanceId:mint(),chatId:1};await assert.rejects(()=>a.api.setTabs([tab,tab]),{code:'invalidWindowCoordination'});const old=structuredClone(f.value);f.fail=true;await assert.rejects(()=>a.api.setFocused(),{code:'windowCoordinationStorageFailed'});assert.deepEqual(f.value,old);a.api.dispose();
});
test('real transaction adapter waits for commit and canonical imported ownership remains wired',async()=>{
 function db(late){let tx;return {transaction(){tx={abort(){queueMicrotask(()=>tx.onabort());},objectStore(){return {get(){const q={};queueMicrotask(()=>{q.result=undefined;q.onsuccess();});return q;},put(){queueMicrotask(()=>late?tx.onabort():tx.oncomplete());}};}};return tx;}};}
 const next=readWindowJournal(undefined);assert.deepEqual(await windowJournalStore(db(false)).commit(undefined,next),next);await assert.rejects(()=>windowJournalStore(db(true)).commit(undefined,next),{code:'windowCoordinationStorageFailed'});
 const host=fs.readFileSync(new URL('../shell/host.js',import.meta.url),'utf8');assert.ok(host.includes('windowCoordination.bootstrap(input)'));assert.ok(!host.includes('"window-infrastructure:set-focused", "window-infrastructure:set-visible-entities"'));assert.ok(host.includes('windowCoordination.attach(input)'));assert.ok(fs.readFileSync(new URL('../../app/src/FoundrySpecification.lex.tex',import.meta.url),'utf8').includes('name: windowCoordinationBoundary,'));
});
test('peer snapshots retain local actor identity and reject malformed stale or mismatched state',async()=>{
 const {createWindowDelivery}=await import('../shell/window-delivery.mjs'),emitted=[];
 const m={revision:10,state:{}},deliver=createWindowDelivery({emit:(name,payload)=>emitted.push({name,payload}),chatMachine:()=>m,chatActor:id=>'local-'+id,appMachine:()=>{throw Error('wrong machine');}});
 const state={schemaVersion:1,chatId:1,revision:3,phase:'idle',invocationRef:null,error:null,queueRevision:0,queue:[],queuePaused:false,queuePauseReason:null,stopPolicyVersion:0,capabilities:{canSubmit:true,canCancel:false,canPauseQueue:true,canResumeQueue:false},lastAcceptance:null,lastCompletion:null,lastQueueMutation:null};
 const remote={protocolVersion:1,machineId:'chat_stream',encodedKey:{chatId:1},actorInstanceId:'peer',revision:3,encodedState:state};
 deliver('distributed-machine:snapshot',remote);assert.equal(emitted.length,1);assert.equal(emitted[0].payload.actorInstanceId,'local-1');assert.equal(emitted[0].payload.revision,11);assert.equal(m.state.revision,11);
 deliver('distributed-machine:snapshot',remote);deliver('distributed-machine:snapshot',{...remote,revision:4});deliver('distributed-machine:snapshot',{...remote,encodedKey:{chatId:2}});deliver('distributed-machine:snapshot',{...remote,encodedState:{...state,phase:'invented'}});assert.equal(emitted.length,1);
 deliver('chat:response:chunk',{chatId:1,messages:'wrong'});deliver('chat:stream:start',{chatId:1,invocationRef:{}});assert.equal(emitted.length,1);
 deliver('chat:response:chunk',{chatId:1,messages:[]});assert.equal(emitted.length,2);
 const host=fs.readFileSync(new URL('../shell/host.js',import.meta.url),'utf8');assert.ok(host.includes('deliver:createWindowDelivery('));
});
