// Imported wire validation precedes assimilation into this window's actor.
import {ChatStreamRemoteSnapshotSchema,AppRunRemoteSnapshotSchema,ChatResponseChunkSchema,ChatStreamInvocationRefSchema,QueryInvalidationBatchSchema} from './chat-contracts.mjs';
export function createWindowDelivery({emit,chatMachine,chatActor,appMachine}) {
 const seen=new Map();
 return (name,raw)=>{
  if(name==='window:query-invalidations'){const parsed=QueryInvalidationBatchSchema.safeParse(raw);if(parsed.success)emit(name,parsed.data);return;}
  if(name==='distributed-machine:snapshot'){
   const chat=raw?.machineId==='chat_stream',app=raw?.machineId==='app_run';
   if((!chat&&!app)||raw.protocolVersion!==1||typeof raw.actorInstanceId!=='string'||!raw.actorInstanceId.length||raw.actorInstanceId.length>256||!Number.isSafeInteger(raw.revision)||raw.revision<0)return;
   const parsed=(chat?ChatStreamRemoteSnapshotSchema:AppRunRemoteSnapshotSchema).safeParse(raw.encodedState),id=chat?raw.encodedKey?.chatId:raw.encodedKey?.appId;
   if(!parsed.success||(chat?parsed.data.chatId:parsed.data.appId)!==id||parsed.data.revision!==raw.revision)return;
   const key=raw.machineId+':'+id+':'+raw.actorInstanceId;
   if((seen.get(key)??-1)>=raw.revision)return;
   if(seen.size>=1024&&!seen.has(key))seen.delete(seen.keys().next().value);
   seen.set(key,raw.revision);
   const m=chat?chatMachine(id):appMachine(id);
   if(m.revision>=Number.MAX_SAFE_INTEGER)return;
   m.state={...parsed.data,revision:++m.revision};
   emit(name,{protocolVersion:1,machineId:raw.machineId,encodedKey:chat?{chatId:id}:{appId:id},actorInstanceId:chat?chatActor(id):'web-app_run-'+id,revision:m.revision,encodedState:m.state});return;
  }
  if(!Number.isSafeInteger(raw?.chatId)||raw.chatId<=0)return;
  if(name==='chat:response:chunk'){const parsed=ChatResponseChunkSchema.safeParse(raw);if(parsed.success&&(!parsed.data.invocationRef||parsed.data.invocationRef.entityKey===raw.chatId))emit(name,parsed.data);return;}
  if(name==='chat:stream:end'){emit(name,{chatId:raw.chatId});return;}
  if(name==='chat:stream:start'){
   if(raw.invocationRef!==undefined){const parsed=ChatStreamInvocationRefSchema.safeParse(raw.invocationRef);if(!parsed.success||parsed.data.entityKey!==raw.chatId)return;}
   if(raw.streamId!==undefined&&(!Number.isSafeInteger(raw.streamId)||raw.streamId<0))return;
   emit(name,{chatId:raw.chatId,...(raw.invocationRef?{invocationRef:raw.invocationRef}:{}),...(raw.streamId!==undefined?{streamId:raw.streamId}:{})});
  }
 };
}
