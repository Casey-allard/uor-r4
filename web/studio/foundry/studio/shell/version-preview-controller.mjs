// Session ownership is an origin Web Lock; every source transition is guest admitted.
import {operateVersionPreview,previewFailure} from './version-preview-store.mjs';
import {previewAddressIdentity} from './version-preview-codec.mjs';
import {versionAlias,versionAddress} from './version-history.mjs';
export function createVersionPreviewController({locks,store,mintOwner,validateEvent,publish,restart=async()=>{},created=()=>{},prepareMessageCommand}){
 const sessions=new Map(),snapshots=new Map();let disposed=false;
 const emit=(s,state,settlement=null)=>{s.revision++;s.state=state;s.lastSettlement=settlement;const snapshot={appId:s.appId,revision:s.revision,state,activeInvocationRef:s.invocation,lastSettlement:settlement};snapshots.set(s.appId,snapshot);publish(s.appId,snapshot);};
 const sessionOf=s=>({appId:s.appId,originBranch:'main',targetVersionId:s.journal?.state.active?versionAlias(previewAddressIdentity(s.journal.state.current)):null,checkedOutVersionId:s.journal?.state.active?versionAlias(previewAddressIdentity(s.journal.state.current)):null,exitIntent:{type:'none'},selectedDiffFile:null,isDiffVisible:false});
 const serial=(s,fn)=>{const p=s.pending.then(fn);s.pending=p.catch(()=>{});return p;};
 const transition=async(s,request,boundary=s.boundary)=>{const answer=await operateVersionPreview({appId:s.appId,owner:s.owner,...request},boundary);s.journal=answer.journal;if(answer.createdVersion)created(s.appId,answer.createdVersion);if(answer.result.runtimeAction==='restart'){try{await restart(s.appId);}catch{answer.result.notification={kind:'warning',message:'Source changed, but preview restart failed. Run the project again.'};}}return answer;};
 function acquire(appId){
  if(disposed||!Number.isSafeInteger(appId)||appId<1||!locks?.request)throw previewFailure('previewUnavailable');
  const existing=sessions.get(appId);if(existing){existing.interested=true;return existing.ready;}
  const s={appId,interested:true,owner:mintOwner(),revision:snapshots.get(appId)?.revision||0,state:{type:'closed'},invocation:null,lastSettlement:null,pending:Promise.resolve(),boundary:store(appId)};
  if(!Number.isInteger(s.owner)||s.owner<1||s.owner>0xffffffff)throw previewFailure('previewUnavailable');
  let ready,fail;s.ready=new Promise((resolve,reject)=>{ready=resolve;fail=reject;});sessions.set(appId,s);
  s.lock=locks.request(`foundry:version-preview:${appId}`,{ifAvailable:true},async lock=>{
   if(!lock){sessions.delete(appId);ready({acquired:false});return;}
   const held=new Promise(resolve=>s.unlock=resolve);
   try{
    s.boundary=await s.boundary;s.journal=await s.boundary.read();
    if(s.journal.state.active){if(s.journal.state.owner===s.owner)throw previewFailure('previewUnavailable');await transition(s,{kind:5});try{await transition(s,{kind:3});}catch(error){emit(s,{type:'recovery-required',session:sessionOf(s),error:{message:error.message}});ready({acquired:true});await held;return;}}
    await transition(s,{kind:1});emit(s,{type:'browsing',session:sessionOf(s)});ready({acquired:true});await held;
   }catch(error){sessions.delete(appId);fail(error);}
  }).catch(error=>{sessions.delete(appId);fail(error);});return s.ready;
 }
 async function release(appId,operationId){const s=sessions.get(appId);if(!s)return {cleanupStarted:false};s.interested=false;if(!(await s.ready).acquired)return {cleanupStarted:false};
  return serial(s,async()=>{if(s.interested)return {cleanupStarted:false};if(s.journal?.state.active){try{await transition(s,{kind:3,...(operationId?{operationId}:{})});}catch(error){emit(s,{type:'recovery-required',session:sessionOf(s),error:{message:error.message}});throw error;}}
   emit(s,{type:'closed'});sessions.delete(appId);s.unlock?.();return {cleanupStarted:true};});
 }
 async function dispatch(appId,raw){let event;try{const parsed=validateEvent(structuredClone(raw));if(!parsed.success)throw Error();event=parsed.data;}catch{throw previewFailure('invalidPreviewCommand');}
  if(event.operationId.length>256)throw previewFailure('invalidPreviewCommand');const s=sessions.get(appId);if(!s)throw previewFailure('previewInterestMissing');await s.ready;
  return serial(s,async()=>{
   if(disposed||sessions.get(appId)!==s)throw previewFailure('previewInterestMissing');
   let request,type,commandBoundary=s.boundary;
   if(event.type==='RESTORE_TO_MESSAGE'||event.type==='RESTORE'&&event.currentChatMessageId){if(!prepareMessageCommand)throw previewFailure('invalidPreviewCommand');const command=await prepareMessageCommand(appId,event);request=command.request;commandBoundary=command.boundary;type='restoring';}
   else if(event.type==='SELECT_VERSION'){request={kind:2,versionId:event.versionId};type='checking-out';}
   else if(['CLOSE','APP_CHANGED','RETRY_RETURN'].includes(event.type)){request={kind:3};type='returning';}
   else if(event.type==='SWITCH_BRANCH'){if(event.branch!=='main')throw previewFailure('previewUnsupportedBranch');request={kind:3};type='returning';}
   else if(event.type==='RESTORE'&&!event.currentChatMessageId){request={kind:4,versionId:event.versionId,...(event.expectedHeadOid?{expectedHeadOid:versionAddress(event.expectedHeadOid)}:{})};type='restoring';}
   else if(event.type==='ACCEPT_CURRENT_REPOSITORY'){request={kind:4,mode:'accept'};type='returning';}
   else if(event.type==='CHECKPOINT_AND_ACCEPT_CURRENT_REPOSITORY'){request={kind:4,mode:'checkpoint'};type='returning';}
   else if(event.type==='SHOW_RECOVERY_NOTICE'){emit(s,s.state,{operationId:event.operationId,outcome:'succeeded'});return;}
   else throw previewFailure('invalidPreviewCommand');
   s.invocation={kind:'version-preview-operation',entityKey:appId,operationId:event.operationId};emit(s,{type,session:sessionOf(s),...(type==='restoring'?{fallback:s.state.type==='previewing'?'previewing':'browsing'}:{})});
   try{const answer=await transition(s,{...request,operationId:event.operationId},commandBoundary);s.invocation=null;
    emit(s,s.journal.state.active?{type:'previewing',session:sessionOf(s)}:{type:'closed'},{operationId:event.operationId,outcome:'succeeded'});
    if(!s.journal.state.active){sessions.delete(appId);s.unlock?.();}return answer.result;
   }catch(error){s.invocation=null;emit(s,{type:'recovery-required',session:sessionOf(s),error:{message:error.message}},{operationId:event.operationId,outcome:'failed',error:{message:error.message}});throw error;}
  });
 }
 return {acquire,release,dispatch,snapshot(appId){const s=sessions.get(appId);return s?{appId,revision:s.revision,state:s.state,activeInvocationRef:s.invocation,lastSettlement:s.lastSettlement}:snapshots.get(appId)||{appId,revision:0,state:{type:'closed'},activeInvocationRef:null,lastSettlement:null};},dispose(){disposed=true;for(const s of sessions.values())void s.ready.then(()=>s.pending).catch(()=>{}).finally(()=>s.unlock?.());sessions.clear();}};
}
