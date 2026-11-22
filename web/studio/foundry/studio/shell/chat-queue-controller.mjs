// Durable queue orchestration. The guest owns policy; locks/transactions own execution admission.
import {chatQueueFailure,stageChatQueue,queueSnapshot} from './chat-queue-store.mjs';
const sameInvocation=(a,b)=>!!a&&!!b&&a.kind===b.kind&&a.entityKey===b.entityKey&&a.operationId===b.operationId;
const itemId=value=>{if(typeof value!=='string'||!/^queue-[1-9][0-9]*$/.test(value))throw chatQueueFailure('invalidChatQueueCommand');const n=Number(value.slice(6));if(!Number.isInteger(n)||n>0xffffffff)throw chatQueueFailure('invalidChatQueueCommand');return n;};
export function createChatQueueController({store,guest,boundary,locks,publish,run,notify=()=>{},mintOwner}){
 const pumps=new Map(),running=new Map(),lifetime=new AbortController();let disposed=false;
 const requireLocks=()=>{if(disposed||!locks||typeof locks.request!=='function')throw chatQueueFailure('chatQueueUnavailable');};
 const emit=(chatId,journal,extra={})=>publish(chatId,queueSnapshot(journal),{journal,...extra});
 async function mutate(chatId,operation){
  requireLocks();return locks.request(`foundry:chat-queue:${chatId}`,async()=>{
   const current=await store.read(chatId),op=typeof operation==='function'?await operation(current.journal):operation;
   if(op===null)return {journal:current.journal,unchanged:true};
   let staged={journal:current.journal,replayed:false};const b=await boundary();
   for(const next of Array.isArray(op)?op:[op])staged=await stageChatQueue(guest,staged.journal,next,b);
   const journal=staged.replayed?staged.journal:await store.commit(current.expected,staged.journal);
   emit(chatId,journal,{acceptance:staged.intentId?{intentId:staged.intentId,acceptance:staged.replayed?'replayed':'queued'}:null});
   if(!staged.replayed)notify({chatId});return {...staged,journal};
  });
 }
 // FoundrySpecification.chatOwnerRecoveryBoundary: retain interest across owner release.
 function drain(chatId){
  requireLocks();if(pumps.has(chatId))return;
  let recheck=false;
  const promise=locks.request(`foundry:chat-execution:${chatId}`,{signal:lifetime.signal},async lock=>{
   if(!lock||disposed)return;const owner=mintOwner();if(!Number.isInteger(owner)||owner<1||owner>0xffffffff)throw chatQueueFailure('chatQueueUnavailable');
   let recovered=false;
   await mutate(chatId,j=>{if(!j.state.active)return null;recovered=true;if(j.state.owner===owner)throw chatQueueFailure('chatQueueUnavailable');return {type:'recover',owner};});
   if(recovered)return;
   while(!disposed){
    const claimed=await mutate(chatId,j=>j.state.paused||!j.state.items.length?null:{type:'claim',owner,observedStop:j.state.stopPolicy});
    if(claimed.unchanged){recheck=true;return;}const journal=claimed.journal,entry=journal.entries.find(e=>e.id===journal.state.active),controller=new AbortController();
    if(!entry)throw chatQueueFailure('chatQueueJournalInvalid');running.set(chatId,{owner,controller,stopPolicy:journal.state.stopPolicy,invocation:structuredClone(entry.intent.invocationRef)});
    const latest=await store.read(chatId);if(latest.journal.state.owner!==owner||latest.journal.state.stopPolicy!==journal.state.stopPolicy)controller.abort();
    let outcome='errored';try{const result=controller.signal.aborted?{outcome:'cancelled'}:await run(chatId,structuredClone(entry.intent),controller.signal);outcome=controller.signal.aborted?'cancelled':['completed','cancelled'].includes(result?.outcome)?result.outcome:'errored';}catch{outcome=controller.signal.aborted?'cancelled':'errored';}finally{running.delete(chatId);}
    await mutate(chatId,{type:outcome==='errored'?'fail':'complete',owner,item:entry.id,outcome});
    if(outcome==='errored')return;
   }
  }).catch(error=>{if(!disposed)publish(chatId,null,{error:chatQueueFailure(error?.code||'chatQueueUnavailable').message});}).finally(async()=>{
   pumps.delete(chatId);if(disposed||!recheck)return;
   try{const {journal}=await store.read(chatId);if(!journal.state.active&&!journal.state.paused&&journal.state.items.length)drain(chatId);}catch{}
  });pumps.set(chatId,promise);
 }
 async function snapshot(chatId){requireLocks();const {journal}=await store.read(chatId);emit(chatId,journal);drain(chatId);return queueSnapshot(journal);}
 async function dispatch(chatId,raw){
  requireLocks();let copied;try{copied=structuredClone(raw);}catch{throw chatQueueFailure('invalidChatQueueCommand');}const b=await boundary();let parsed;try{parsed=b.validateEvent(copied);}catch{throw chatQueueFailure('invalidChatQueueCommand');}if(parsed?.success!==true)throw chatQueueFailure('invalidChatQueueCommand');const event=structuredClone(parsed.data);
  if(event.type==='REPORT_ERROR'){publish(chatId,null,{reportedError:event.error});return;}
  if(event.type==='CANCEL'){
   await mutate(chatId,j=>{const active=j.entries.find(e=>e.id===j.state.active);if(!active||!sameInvocation(event.invocationRef,active.intent.invocationRef))throw chatQueueFailure('queueActiveConflict');const stop={type:'stop',observedStop:event.observedStopPolicyVersion??0};return event.pauseQueue===false?[stop,{type:'resume',observedStop:j.state.stopPolicy+1}]:stop;});
   const active=running.get(chatId);if(active&&sameInvocation(active.invocation,event.invocationRef))active.controller.abort();notify({chatId,stop:true});return;
  }
  let operation;
  if(event.type==='SUBMIT')operation={type:'enqueue',intent:event.intent,observedStop:event.observedStopPolicyVersion??0};
  else{
   const common={expectedRevision:event.expectedQueueRevision,mutationId:event.mutationId};
   if(event.type==='PAUSE_QUEUE')operation={...common,type:'pause'};
   else if(event.type==='RESUME_QUEUE')operation={...common,type:'resume',observedStop:event.observedStopPolicyVersion??0};
   else if(event.type==='CLEAR_QUEUE')operation={...common,type:'clear'};
   else if(event.type==='REMOVE_QUEUE_ENTRY')operation={...common,type:'remove',item:itemId(event.itemId)};
   else if(event.type==='EDIT_QUEUE_ENTRY')operation={...common,type:'edit',item:itemId(event.itemId),prompt:event.prompt,attachments:event.attachments,selectedComponents:event.selectedComponents};
   else if(event.type==='REORDER_QUEUE_ENTRY')operation=j=>({...common,type:'reorder',item:itemId(event.itemId),index:event.toIndex+(j.state.active?1:0)});
   else throw chatQueueFailure('invalidChatQueueCommand');
  }
  await mutate(chatId,operation);drain(chatId);
 }
 async function refresh(chatId){
  if(disposed)return;const active=running.get(chatId);let journal;try{({journal}=await store.read(chatId));}catch(error){active?.controller.abort();throw error;}emit(chatId,journal);
  if(active&&(journal.state.owner!==active.owner||journal.state.stopPolicy!==active.stopPolicy))active.controller.abort();
  drain(chatId);
 }
 async function cancelCurrent(chatId){const {journal}=await store.read(chatId),active=journal.entries.find(e=>e.id===journal.state.active);if(active)await dispatch(chatId,{type:'CANCEL',invocationRef:active.intent.invocationRef,pauseQueue:true,observedStopPolicyVersion:journal.state.stopPolicy});}
 return {dispatch,snapshot,refresh,cancelCurrent,async idle(chatId){await pumps.get(chatId);},dispose(){disposed=true;lifetime.abort();for(const active of running.values())active.controller.abort();},get activeCount(){return running.size;}};
}
