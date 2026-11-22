// FoundrySpecification.chatPreferencesBoundary: imported schema, atomic storage.
import {UpdateChatParamsSchema} from './chat-contracts.mjs';
const fail=code=>Object.assign(new Error('Chat preference operation refused'),{code});
const identity=value=>Number.isSafeInteger(value)&&value>0;
export async function updateChatPreferences(database,input){
 let captured;
 try{captured=UpdateChatParamsSchema.parse(structuredClone(input));if(!identity(captured.chatId))throw Error();}
 catch{throw fail('invalidChatPreferences');}
 let db;try{db=await database;}catch{throw fail('chatPreferencesStorageFailed');}
 return new Promise((resolve,reject)=>{
  let transaction,settled=false;
  const refuse=(code='chatPreferencesStorageFailed')=>{
   if(settled)return;settled=true;
   try{transaction?.abort();}catch{}
   reject(fail(code));
  };
  try{
   transaction=db.transaction(['apps','chats'],'readwrite');
   transaction.oncomplete=()=>{if(!settled){settled=true;resolve();}};
   transaction.onabort=()=>refuse();transaction.onerror=()=>refuse();
   const chats=transaction.objectStore('chats'),request=chats.get(captured.chatId);
   request.onsuccess=()=>{
    if(settled)return;
    try{
     const chat=request.result;
     if(chat===undefined){refuse('chatPreferencesMissing');return;}
     if(!chat||chat.id!==captured.chatId||!identity(chat.appId)){refuse('chatPreferencesStateInvalid');return;}
     const owner=transaction.objectStore('apps').get(chat.appId);
     owner.onsuccess=()=>{
      if(settled)return;
      try{
       if(owner.result===undefined){refuse('chatPreferencesMissing');return;}
       if(!owner.result||owner.result.id!==chat.appId){refuse('chatPreferencesStateInvalid');return;}
       const next={...chat};
       for(const key of ['title','chatMode','modelSelection'])if(captured[key]!==undefined)next[key]=captured[key];
       chats.put(next,captured.chatId);
      }catch{refuse();}
     };
    }catch{refuse();}
   };
  }catch{refuse();}
 });
}
