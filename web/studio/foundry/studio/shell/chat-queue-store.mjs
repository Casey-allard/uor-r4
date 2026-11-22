// FoundrySpecification.durableChatQueueBoundary. State and payloads commit together.
import {queueWithGuest,encodeQueueRequest} from './chat-queue-codec.mjs';
import {storageRequest} from './storage-request.mjs';
const maximumBytes=48*1024*1024;
export const chatQueueFailure=code=>Object.assign(new Error(({invalidChatQueueCommand:'Invalid chat queue command',chatQueueUnavailable:'Chat queues are unavailable',chatQueueJournalInvalid:'Stored chat queue is invalid',chatQueueConflict:'The queue changed; try again',chatQueueMissing:'The chat no longer exists',chatQueueStorageFailed:'The queue could not be saved',chatQueueLimitExceeded:'The chat queue is full',chatQueueIntentConflict:'This submission identity has different content'})[code]||'Chat queue transition refused'),{code});
const id=value=>{if(!Number.isSafeInteger(value)||value<1)throw chatQueueFailure('invalidChatQueueCommand');return value;};
const clone=value=>{try{return structuredClone(value);}catch{throw chatQueueFailure('invalidChatQueueCommand');}};
const bytes=value=>new TextEncoder().encode(JSON.stringify(value));
const exact=(x,keys)=>x&&typeof x==='object'&&!Array.isArray(x)&&Object.keys(x).sort().join()===keys.split(',').sort().join();
export function emptyChatQueue(chatId,appId){return {profile:'foundry/chat-queue/1',chatId:id(chatId),appId:id(appId),nextItem:1,state:{revision:0,paused:0,stopPolicy:0,active:0,owner:0,items:[]},entries:[],receipts:[],pauseReason:null,lastMutation:null};}
export function readChatQueue(raw,chatId,appId){
 try{
  const j=clone(raw);if(!exact(j,'profile,chatId,appId,nextItem,state,entries,receipts,pauseReason,lastMutation')||j.profile!=='foundry/chat-queue/1'||j.chatId!==chatId||j.appId!==appId||!Number.isInteger(j.nextItem)||j.nextItem<1||j.nextItem>0x100000000||![null,'manual','stop','step-limit'].includes(j.pauseReason))throw Error();
  encodeQueueRequest({state:j.state,expectedRevision:j.state.revision,command:{kind:6,item:0,index:0,owner:0,observedStop:0}});
  if(j.state.paused>1||new Set(j.state.items).size!==j.state.items.length||j.state.items.some(n=>!n)||(j.state.active?j.state.items[0]!==j.state.active||!j.state.owner:j.state.owner!==0))throw Error();
  if(!Array.isArray(j.entries)||j.entries.length!==j.state.items.length||!Array.isArray(j.receipts)||j.receipts.length>128)throw Error();
  const identities=new Set();for(let i=0;i<j.entries.length;i++){const e=j.entries[i];if(!exact(e,'id,intent,identity')||e.id!==j.state.items[i]||e.id<1||e.id>=j.nextItem||typeof e.intent?.intentId!=='string'||!e.intent.intentId||e.intent.chatId!==chatId||typeof e.identity!=='string'||!e.identity||identities.has(e.intent.intentId))throw Error();identities.add(e.intent.intentId);}
  for(const e of j.receipts){if(!exact(e,'intentId,identity,outcome')||typeof e.intentId!=='string'||!e.intentId||typeof e.identity!=='string'||!e.identity||!['completed','cancelled','removed'].includes(e.outcome)||identities.has(e.intentId))throw Error();identities.add(e.intentId);}
  if(j.lastMutation!==null&&(!exact(j.lastMutation,'mutationId,outcome')||typeof j.lastMutation.mutationId!=='string'||j.lastMutation.outcome!=='applied'))throw Error();
  if(bytes(j).length>maximumBytes)throw Error();return j;
 }catch{throw chatQueueFailure('chatQueueJournalInvalid');}
}
export async function bindQueuePayload(raw,serialize){
 const value=clone(raw);try{const digest=await crypto.subtle.digest('SHA-256',new TextEncoder().encode(serialize(value)));value.payloadHash=[...new Uint8Array(digest)].map(b=>b.toString(16).padStart(2,'0')).join('');return value;}catch{throw chatQueueFailure('invalidChatQueueCommand');}
}
export async function captureQueueIntent(raw,chatId,appId,{validate,derive,serialize}){
 let value=clone(raw),parsed;try{parsed=validate(value);}catch{throw chatQueueFailure('invalidChatQueueCommand');}
 if(parsed?.success!==true)throw chatQueueFailure('invalidChatQueueCommand');value=clone(parsed.data);
 if(value.chatId!==chatId||(value.appId!==undefined&&value.appId!==appId)||typeof value.intentId!=='string'||!value.intentId||!value.invocationRef||value.invocationRef.entityKey!==chatId||bytes(value).length>maximumBytes)throw chatQueueFailure('invalidChatQueueCommand');
 if((await bindQueuePayload(value,serialize)).payloadHash!==value.payloadHash)throw chatQueueFailure('invalidChatQueueCommand');
 let identity;try{identity=await derive(bytes(value));}catch{throw chatQueueFailure('chatQueueUnavailable');}
 if(typeof identity!=='string'||!identity)throw chatQueueFailure('chatQueueUnavailable');return {intent:value,identity};
}
export async function readStoredChatQueue(db,chatId){
 id(chatId);try{const chat=await storageRequest(db,'chats','readonly',s=>s.get(chatId));if(!chat||!await storageRequest(db,'apps','readonly',s=>s.get(chat.appId)))throw chatQueueFailure('chatQueueMissing');
 const raw=await storageRequest(db,'refs','readonly',s=>s.get(`chat-queue:${chatId}`));return {expected:raw,journal:raw===undefined?emptyChatQueue(chatId,chat.appId):readChatQueue(raw,chatId,chat.appId)};}catch(error){if(['chatQueueMissing','chatQueueJournalInvalid'].includes(error?.code))throw error;throw chatQueueFailure('chatQueueStorageFailed');}
}
export function commitStoredChatQueue(db,expected,journal){
 const next=readChatQueue(journal,journal.chatId,journal.appId),prior=clone(expected);return new Promise((resolve,reject)=>{
  let tx,failure='chatQueueStorageFailed';const refuse=code=>{failure=code;try{tx?.abort();}catch{}reject(chatQueueFailure(code));};
  try{
   tx=db.transaction(['apps','chats','refs'],'readwrite');tx.oncomplete=()=>resolve(clone(next));tx.onabort=()=>reject(chatQueueFailure(failure));tx.onerror=()=>refuse('chatQueueStorageFailed');
   const apps=tx.objectStore('apps'),chats=tx.objectStore('chats'),refs=tx.objectStore('refs'),q=chats.get(next.chatId);
   q.onsuccess=()=>{try{if(!q.result||q.result.appId!==next.appId){refuse('chatQueueMissing');return;}const app=apps.get(next.appId);app.onsuccess=()=>{try{if(!app.result){refuse('chatQueueMissing');return;}const old=refs.get(`chat-queue:${next.chatId}`);old.onsuccess=()=>{try{if(JSON.stringify(old.result)!==JSON.stringify(prior)){refuse('chatQueueConflict');return;}refs.put(next,`chat-queue:${next.chatId}`);}catch{refuse('chatQueueStorageFailed');}};}catch{refuse('chatQueueStorageFailed');}};}catch{refuse('chatQueueStorageFailed');}};
  }catch{refuse('chatQueueStorageFailed');}
 });
}
const command=(kind,fields={})=>({kind,item:0,index:0,owner:0,observedStop:0,...fields});
function retainReceipt(j,entry,outcome){j.receipts.push({intentId:entry.intent.intentId,identity:entry.identity,outcome});if(j.receipts.length>128)j.receipts.shift();}
export async function stageChatQueue(guest,raw,operation,boundary){
 const j=readChatQueue(raw,raw.chatId,raw.appId),op=clone(operation);let change,payload=null;
 for(const entry of j.entries){const captured=await captureQueueIntent(entry.intent,j.chatId,j.appId,boundary);if(captured.identity!==entry.identity)throw chatQueueFailure('chatQueueJournalInvalid');}
 const expectedRevision=op.expectedRevision??j.state.revision;
 switch(op.type){
  case 'enqueue':{
   payload=await captureQueueIntent(op.intent,j.chatId,j.appId,boundary);const existing=j.entries.find(e=>e.intent.intentId===payload.intent.intentId)||j.receipts.find(e=>e.intentId===payload.intent.intentId);
   if(existing){if(existing.identity!==payload.identity)throw chatQueueFailure('chatQueueIntentConflict');return {journal:j,replayed:true,intentId:payload.intent.intentId};}
   if(op.observedStop!==j.state.stopPolicy)throw chatQueueFailure('queueStopConflict');if(j.nextItem>0xffffffff)throw chatQueueFailure('chatQueueLimitExceeded');change=command(1,{item:j.nextItem});break;
  }
  case 'edit':{const entry=j.entries.find(e=>e.id===op.item);if(!entry)throw chatQueueFailure('queueItemMissing');payload=await captureQueueIntent(await bindQueuePayload({...entry.intent,prompt:op.prompt,attachments:op.attachments,selectedComponents:op.selectedComponents},boundary.serialize),j.chatId,j.appId,boundary);change=command(4,{item:op.item});break;}
  case 'remove':change=command(2,{item:op.item});break;
  case 'reorder':change=command(3,{item:op.item,index:op.index});break;
  case 'clear':change=command(5);break;
  case 'pause':change=command(6);break;
  case 'resume':change=command(7,{observedStop:op.observedStop});break;
  case 'stop':if(op.observedStop!==j.state.stopPolicy)throw chatQueueFailure('queueStopConflict');change=command(8);break;
  case 'claim':change=command(9,{owner:op.owner,observedStop:op.observedStop});break;
  case 'complete':change=command(10,{item:op.item,owner:op.owner});break;
  case 'recover':change=command(11,{owner:op.owner});break;
  case 'fail':change=command(12,{owner:op.owner});break;
  default:throw chatQueueFailure('invalidChatQueueCommand');
 }
 const answer=await queueWithGuest(guest,{state:j.state,expectedRevision,command:change});if(!answer.ok)throw chatQueueFailure(answer.code);
 if(op.type==='enqueue'){j.entries.push({id:j.nextItem,...payload});j.nextItem++;}
 if(op.type==='edit'){const entry=j.entries.find(e=>e.id===op.item);entry.intent=payload.intent;entry.identity=payload.identity;}
 for(const entry of j.entries)if(!answer.state.items.includes(entry.id))retainReceipt(j,entry,op.type==='complete'?(op.outcome==='cancelled'?'cancelled':'completed'):'removed');
 j.entries=answer.state.items.map(item=>{const entry=j.entries.find(e=>e.id===item);if(!entry)throw chatQueueFailure('chatQueueJournalInvalid');return entry;});j.state=answer.state;
 if(['stop','recover','fail'].includes(op.type))j.pauseReason='stop';if(op.type==='pause')j.pauseReason='manual';if(op.type==='resume')j.pauseReason=null;
 if(op.mutationId!==undefined){if(typeof op.mutationId!=='string'||!op.mutationId||op.mutationId.length>256)throw chatQueueFailure('invalidChatQueueCommand');j.lastMutation={mutationId:op.mutationId,outcome:'applied'};}
 if(bytes(j).length>maximumBytes)throw chatQueueFailure('chatQueueLimitExceeded');return {journal:readChatQueue(j,j.chatId,j.appId),replayed:false,intentId:payload?.intent.intentId};
}
export function queueSnapshot(journal){
 const j=readChatQueue(journal,journal.chatId,journal.appId);return {queueRevision:j.state.revision,queuePaused:!!j.state.paused,queuePauseReason:j.pauseReason,stopPolicyVersion:j.state.stopPolicy,lastQueueMutation:j.lastMutation,queue:j.entries.filter(e=>e.id!==j.state.active).map(e=>({itemId:`queue-${e.id}`,intentId:e.intent.intentId,prompt:e.intent.prompt,attachments:e.intent.attachments,selectedComponents:e.intent.selectedComponents,redo:e.intent.redo,appId:e.intent.appId,requestedChatMode:e.intent.requestedChatMode,persistence:'durable',editable:true,removable:true}))};
}
