// FoundrySpecification.versionHistoryBoundary: reversible aliases, verified bytes and atomic metadata.
import {prepareSnapshot} from './snapshot-restore.mjs';
const messages={invalidVersionHistory:'Invalid version request',versionHistoryMissing:'The project or version no longer exists',versionHistoryStorageFailed:'Version history storage failed',versionHistoryStateInvalid:'Stored version history is invalid'};
export const versionHistoryFailure=code=>Object.assign(new Error(messages[code]||messages.versionHistoryStorageFailed),{code});
const id=n=>{if(!Number.isSafeInteger(n)||n<1)throw versionHistoryFailure('invalidVersionHistory');return n;};
export function versionAddress(value){if(typeof value!=='string')throw versionHistoryFailure('invalidVersionHistory');if(/^[0-9a-fA-F]{64}$/.test(value))return 'blake3:'+value.toLowerCase();if(/^blake3:[0-9a-f]{64}$/.test(value))return value;throw versionHistoryFailure('invalidVersionHistory');}
export const versionAlias=value=>versionAddress(value).slice(7);
export function versionRows(raw){
 try{if(!Array.isArray(raw))throw Error();return raw.map(row=>{if(!row||versionAddress(row.oid)!==row.oid||typeof row.message!=='string'||!Number.isFinite(row.timestamp)||row.timestamp<0||typeof row.isFavorite!=='boolean'||!(row.note===null||typeof row.note==='string'&&row.note.length<=10000)||!(row.dbTimestamp===undefined||row.dbTimestamp===null||typeof row.dbTimestamp==='string'))throw Error();return {...structuredClone(row),oid:versionAlias(row.oid)};});}catch{throw versionHistoryFailure('versionHistoryStateInvalid');}
}
export function readVersionHistory(db,appId){
 id(appId);return new Promise((resolve,reject)=>{let tx,app,rows,failure='versionHistoryStorageFailed';const refuse=code=>{failure=code;try{tx?.abort();}catch{}reject(versionHistoryFailure(code));};
  try{tx=db.transaction(['apps','versions'],'readonly');tx.onabort=()=>reject(versionHistoryFailure(failure));tx.onerror=()=>refuse('versionHistoryStorageFailed');tx.oncomplete=()=>{if(!app){reject(versionHistoryFailure('versionHistoryMissing'));return;}try{resolve(versionRows(rows));}catch(error){reject(error);}};
   const aq=tx.objectStore('apps').get(appId),vq=tx.objectStore('versions').get(appId);aq.onsuccess=()=>{app=aq.result;};vq.onsuccess=()=>{rows=vq.result;};
  }catch{refuse('versionHistoryStorageFailed');}
 });
}
export async function listStoredVersions(db,input){
 let request;try{request=structuredClone(input);}catch{throw versionHistoryFailure('invalidVersionHistory');}if(!request||Object.keys(request).some(k=>!['appId','ref'].includes(k)))throw versionHistoryFailure('invalidVersionHistory');
 const rows=await readVersionHistory(db,id(request.appId));if(request.ref===undefined||request.ref==='main')return rows;
 const oid=versionAlias(request.ref),found=rows.find(row=>row.oid===oid);if(!found)throw versionHistoryFailure('versionHistoryMissing');return [found];
}
export async function resolveStoredVersion(db,appId,reference){const oid=versionAlias(reference),rows=await readVersionHistory(db,id(appId));if(!rows.some(row=>row.oid===oid))throw versionHistoryFailure('versionHistoryMissing');return versionAddress(oid);}
export function setStoredVersionMetadata(db,raw,field){
 let input;try{input=structuredClone(raw);}catch{throw versionHistoryFailure('invalidVersionHistory');}
 if(!['isFavorite','note'].includes(field)||!input||Object.keys(input).sort().join()!==['appId','versionId',field].sort().join())throw versionHistoryFailure('invalidVersionHistory');
 const appId=id(input.appId),oid=versionAddress(input.versionId),value=input[field];
 if(field==='isFavorite'?typeof value!=='boolean':!(value===null||typeof value==='string'&&value.length<=10000))throw versionHistoryFailure('invalidVersionHistory');
 return new Promise((resolve,reject)=>{let tx,result,failed=false,failure='versionHistoryStorageFailed';const refuse=code=>{failed=true;failure=code;try{tx?.abort();}catch{}reject(versionHistoryFailure(code));};
  try{tx=db.transaction(['apps','versions'],'readwrite');tx.onabort=()=>reject(versionHistoryFailure(failure));tx.onerror=()=>refuse('versionHistoryStorageFailed');tx.oncomplete=()=>{if(failed||!result)reject(versionHistoryFailure(failure));else resolve(result);};
   const q=tx.objectStore('apps').get(appId);q.onsuccess=()=>{if(!q.result){refuse('versionHistoryMissing');return;}try{const store=tx.objectStore('versions'),v=store.get(appId);v.onsuccess=()=>{try{versionRows(v.result);const rows=structuredClone(v.result),matching=rows.filter(row=>row.oid===oid);if(!matching.length){refuse('versionHistoryMissing');return;}for(const row of matching)row[field]=value;result={oid:versionAlias(oid),isFavorite:matching[0].isFavorite,note:matching[0].note};store.put(rows,appId);}catch(error){refuse(error?.code==='versionHistoryStateInvalid'?error.code:'versionHistoryStorageFailed');}};}catch{refuse('versionHistoryStorageFailed');}};
  }catch{refuse('versionHistoryStorageFailed');}
 });
}
export async function verifiedVersionChanges(db,raw,boundary){
 let input;try{input=structuredClone(raw);}catch{throw versionHistoryFailure('invalidVersionHistory');}if(!input||Object.keys(input).sort().join()!=='appId,versionId')throw versionHistoryFailure('invalidVersionHistory');
 const oid=await resolveStoredVersion(db,input.appId,input.versionId),current=await prepareSnapshot(oid,boundary),prior=current.parent?await prepareSnapshot(current.parent,boundary):{files:[]};
 const before=new Map(prior.files),after=new Map(current.files),changes=[];
 for(const [path,content] of after)if(!before.has(path)||before.get(path)!==content)changes.push({path,type:before.has(path)?'modified':'added',oldContent:before.get(path)??'',newContent:content});
 for(const [path,content] of before)if(!after.has(path))changes.push({path,type:'deleted',oldContent:content,newContent:''});
 return changes.sort((a,b)=>a.path<b.path?-1:a.path>b.path?1:0);
}
