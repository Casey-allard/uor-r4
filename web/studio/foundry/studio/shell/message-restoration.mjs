// FoundrySpecification.messageRestorationBoundary; preparation never mutates storage.
import {readChatQueue} from './chat-queue-store.mjs';
import {storageRequest} from './storage-request.mjs';
import {versionAddress} from './version-history.mjs';
export const messageRestoreFailure=code=>Object.assign(new Error(({invalidMessageRestore:'Invalid message restoration',messageRestoreMissing:'The chat or message no longer exists',messageRestoreBusy:'Stop and clear queued chats before restoring message history',messageRestoreConflict:'Chat history changed; review the restoration again',messageRestoreVersionMissing:'No verified version is available before this message',messageRestoreStorageFailed:'Message restoration could not be saved'})[code]||'Message restoration refused'),{code});
const id=value=>{if(!Number.isSafeInteger(value)||value<1)throw messageRestoreFailure('invalidMessageRestore');return value;};
const bytes=value=>new TextEncoder().encode(JSON.stringify(value)).length;
const copyFields=['role','content','approvalState','sourceCommitHash','commitHash','requestId','maxTokensUsed','model','aiMessagesJson','isCompactionSummary','createdAt'];
export function captureRestoreChat(raw,appId,chatId,messageId){
 let chat;try{chat=structuredClone(raw);}catch{throw messageRestoreFailure('invalidMessageRestore');}
 if(!chat||chat.id!==chatId||chat.appId!==appId)throw messageRestoreFailure('messageRestoreMissing');
 if(!Array.isArray(chat.messages)||chat.messages.length>10000||bytes(chat)>48*1024*1024)throw messageRestoreFailure('invalidMessageRestore');
 let last=0;for(const m of chat.messages){if(!m||!Number.isSafeInteger(m.id)||m.id<=last||!['user','assistant','system'].includes(m.role)||typeof m.content!=='string')throw messageRestoreFailure('invalidMessageRestore');last=m.id;}
 const index=chat.messages.findIndex(m=>m.id===messageId);if(index<0)throw messageRestoreFailure('messageRestoreMissing');return {chat,index};
}
export function messageSourceVersion(chat,index){
 for(let i=index+1;i<chat.messages.length;i++){const m=chat.messages[i];if(m.role==='assistant'){if(m.sourceCommitHash)return versionAddress(m.sourceCommitHash);break;}}
 for(let i=index-1;i>=0;i--){const m=chat.messages[i];if(m.role==='assistant'&&m.commitHash)return versionAddress(m.commitHash);}
 if(chat.initialCommitHash)return versionAddress(chat.initialCommitHash);throw messageRestoreFailure('messageRestoreVersionMissing');
}
export async function prepareMessageRestoration(db,raw,{resolve,verify}){
 let input;try{input=structuredClone(raw);}catch{throw messageRestoreFailure('invalidMessageRestore');}
 if(!input||Object.keys(input).some(k=>!['appId','chatId','messageId','mode','restoreCodebase','versionId'].includes(k))||!['fork','truncate'].includes(input.mode)||typeof input.restoreCodebase!=='boolean')throw messageRestoreFailure('invalidMessageRestore');
 const {appId,chatId,messageId}=input;id(appId);id(chatId);id(messageId);
 let chats;try{chats=await storageRequest(db,'chats','readonly',s=>s.getAll());}catch{throw messageRestoreFailure('messageRestoreStorageFailed');}
 if(!Array.isArray(chats))throw messageRestoreFailure('invalidMessageRestore');const {chat,index}=captureRestoreChat(chats.find(c=>c.id===chatId),appId,chatId,messageId);
 const expectedChats=chats.filter(c=>c.appId===appId).sort((a,b)=>a.id-b.id),expectedQueues=[];if(expectedChats.length>1024||bytes(expectedChats)>64*1024*1024)throw messageRestoreFailure('invalidMessageRestore');
 for(const c of expectedChats){id(c.id);let journal;try{journal=await storageRequest(db,'refs','readonly',s=>s.get(`chat-queue:${c.id}`));}catch{throw messageRestoreFailure('messageRestoreStorageFailed');}
  if(journal!==undefined){try{journal=readChatQueue(journal,c.id,appId);}catch{throw messageRestoreFailure('invalidMessageRestore');}}if(journal?.state.active||journal?.state.items.length)throw messageRestoreFailure('messageRestoreBusy');expectedQueues.push([c.id,journal]);}
 let versionId=null;if(input.restoreCodebase){versionId=input.mode==='truncate'?versionAddress(input.versionId):messageSourceVersion(chat,index);versionId=await resolve(versionId);await verify(versionId);}
 const retained=chat.messages.slice(0,index),next=input.mode==='truncate'?{...chat,messages:retained}:{title:(chat.title||'Chat').slice(0,230)+' (restored)',appId,createdAt:new Date().toISOString(),messages:retained.map((m,i)=>({id:i+1,...Object.fromEntries(copyFields.filter(k=>m[k]!==undefined).map(k=>[k,m[k]])),usingFreeAgentModeQuota:false})),chatMode:chat.chatMode||'build',modelSelection:chat.modelSelection??null,referencedApps:structuredClone(chat.referencedApps||[]),dbTimestamp:null};
 return {versionId,chatUpdate:{profile:'foundry/message-restore/1',appId,chatId,mode:input.mode,expectedChats,expectedQueues,next}};
}
