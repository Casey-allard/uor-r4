// FoundrySpecification.browserSettingsTransactionBoundary.
const messages={invalidSettingsCommand:'The settings update is invalid',invalidStoredSettings:'The stored settings record is invalid',settingsStorageFailed:'Browser settings storage failed'};
const failure=code=>Object.assign(new Error(messages[code]),{code});
const record=value=>!!value&&typeof value==='object'&&!Array.isArray(value)&&(Object.getPrototypeOf(value)===Object.prototype||Object.getPrototypeOf(value)===null);
function copy(value,code){try{if(!record(value))throw 0;return structuredClone(value);}catch{throw failure(code);}}
async function transaction(database,mutate){
 let db;try{db=await database;}catch{throw failure('settingsStorageFailed');}
 return new Promise((resolve,reject)=>{
  let tx,result,reason,settled=false;
  const refuse=()=>{if(!settled){settled=true;reject(reason||failure('settingsStorageFailed'));}};
  try{
   tx=db.transaction('settings',mutate?'readwrite':'readonly');
   tx.onabort=refuse;tx.onerror=()=>{};
   tx.oncomplete=()=>{if(reason){refuse();return;}if(!settled){settled=true;resolve(result);}};
   const store=tx.objectStore('settings'),request=store.get('user');
   request.onerror=()=>{reason=failure('settingsStorageFailed');};
   request.onsuccess=()=>{
    try{
     const current=request.result===undefined?null:copy(request.result,'invalidStoredSettings');
     if(mutate){result=copy(mutate(current||{}),'invalidSettingsCommand');const write=store.put(result,'user');write.onerror=()=>{reason=failure('settingsStorageFailed');};}
     else result=current;
    }catch(error){reason=Object.hasOwn(messages,error?.code)?failure(error.code):failure('settingsStorageFailed');try{tx.abort();}catch{refuse();}}
   };
  }catch{reason=failure('settingsStorageFailed');if(tx){try{tx.abort();}catch{refuse();}}else refuse();}
 });
}
export function readUserSettings(database){return transaction(database,null);}
export function changeUserSettings(database,input,apply){
 let captured;try{captured=copy(input,'invalidSettingsCommand');if(typeof apply!=='function')throw failure('invalidSettingsCommand');}catch(error){return Promise.reject(error);}
 return transaction(database,current=>apply(current,captured));
}
export function replaceUserSettings(database,value){return changeUserSettings(database,value,(_current,captured)=>captured);}
