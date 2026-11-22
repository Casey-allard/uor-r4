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

function contendedLocks(){
 let busy=true;const waiting=[];let attempts=0;
 const locks={request:async(name,options,callback)=>{
  if(typeof options==='function')return options({});
  attempts++;
  if(options.ifAvailable&&busy)return callback(null);
  return new Promise((resolve,reject)=>{
   const entry={callback,resolve,reject,signal:options.signal};waiting.push(entry);
   entry.abort=()=>{const i=waiting.indexOf(entry);if(i>=0)waiting.splice(i,1);reject(new DOMException('aborted','AbortError'));};
   if(options.signal?.aborted)entry.abort();else options.signal?.addEventListener('abort',entry.abort,{once:true});
   if(!busy)flush();
  });
 }};
 function flush(){if(busy)return;const e=waiting.shift();if(!e)return;busy=true;e.signal?.removeEventListener('abort',e.abort);Promise.resolve().then(()=>e.callback({})).then(e.resolve,e.reject).finally(()=>{busy=false;flush();});}
 return {locks,release(){busy=false;flush();},get waiting(){return waiting.length;},get attempts(){return attempts;}};
}
async function setup(){
 let j=await journal('recover'),commits=0,runs=0;const published=[],lock=contendedLocks();
 const controller=createChatQueueController({store:{read:async()=>({expected:structuredClone(j),journal:structuredClone(j)}),commit:async(expected,next)=>{assert.deepEqual(expected,j);j=structuredClone(next);commits++;return j;}},guest:fixtureGuest,boundary:async()=>boundary,locks:lock.locks,publish:(_id,snapshot,extra)=>published.push({snapshot,extra}),run:async()=>{runs++;},mintOwner:()=>8});
 return {controller,lock,published,get journal(){return j;},get commits(){return commits;},get runs(){return runs;}};
}
const turn=()=>new Promise(resolve=>setImmediate(resolve));
test('contended subscribers retain one exclusive interest and recover exact paused head',async()=>{
 const f=await setup(),before=structuredClone(f.journal);
 try{
  await f.controller.snapshot(1);await turn();
  assert.equal(f.lock.waiting,1,'contention must retain recovery interest');
  await f.controller.snapshot(1);await f.controller.refresh(1);await turn();
  assert.equal(f.lock.waiting,1);assert.equal(f.lock.attempts,1);assert.equal(f.commits,0);assert.equal(f.runs,0);
  f.lock.release();await f.controller.idle(1);
  assert.equal(f.commits,1);assert.equal(f.runs,0);assert.deepEqual(f.journal.state,vector('recover').expected);
  assert.deepEqual(f.journal.entries,before.entries);assert.equal(f.journal.pauseReason,'stop');
 }finally{f.controller.dispose();f.lock.release();}
});
test('disposed waiters cancel lock interest without writes execution or error publication',async()=>{
 const f=await setup();try{await f.controller.snapshot(1);await turn();
 assert.equal(f.lock.waiting,1);const publications=f.published.length;
 f.controller.dispose();assert.equal(f.lock.waiting,0);await f.controller.idle(1);
 f.lock.release();await turn();assert.equal(f.commits,0);assert.equal(f.runs,0);assert.equal(f.published.length,publications);
 await refuses(()=>f.controller.snapshot(1),'chatQueueUnavailable');
 }finally{f.controller.dispose();f.lock.release();}
});
test('live lock failure remains typed and recovery browser is mandatory',async()=>{
 const publications=[];
 const c=createChatQueueController({store:{read:async()=>({journal:await journal('recover')})},guest:fixtureGuest,boundary:async()=>boundary,locks:{request:async()=>{throw Error('private lock detail');}},publish:(_id,_snapshot,extra)=>publications.push(extra),mintOwner:()=>8});
 await c.snapshot(1);await c.idle(1);assert.equal(publications.at(-1).error,'Chat queues are unavailable');c.dispose();
 const root=new URL('../../',import.meta.url);
 assert.match(fs.readFileSync(new URL('app/src/FoundrySpecification.lex.tex',root),'utf8'),/name: chatOwnerRecoveryBoundary/);
 const workflow=fs.readFileSync(new URL('.github/workflows/studio.yml',root),'utf8');assert.ok(workflow.includes('node tools/durable_chat_queue_browser.mjs'));
 const browser=fs.readFileSync(new URL('studio/tools/durable_chat_queue_browser.mjs',root),'utf8');assert.ok(browser.includes('waitingRecovery:true'));assert.ok(browser.includes('navigator.locks.query()'));
});
