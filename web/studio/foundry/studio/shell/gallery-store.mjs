// FoundrySpecification.sourceBoundGalleryBoundary. IndexedDB owns publication.
import {checkCard} from './covers.mjs';
const PROFILE='foundry/gallery/1',MAX_PICTURE=8*1024*1024;
const CODES=['invalidGalleryCommand','gallerySourceChanged','galleryStateInvalid','galleryStorageFailed'];
export const galleryFailure=code=>Object.assign(new Error(CODES.includes(code)?code:'galleryStorageFailed'),{code:CODES.includes(code)?code:'galleryStorageFailed'});
const fail=code=>{throw galleryFailure(code);};
const plain=v=>v!==null&&typeof v==='object'&&!Array.isArray(v);
const exact=(v,keys)=>plain(v)&&Object.keys(v).sort().join('|')===[...keys].sort().join('|');
const reference=v=>typeof v==='string'&&/^blake3:[0-9a-f]{64}$/.test(v);
const validSource=s=>exact(s,['name','reference','files','dependencies'])&&typeof s.name==='string'&&s.name.length>0&&reference(s.reference)&&Array.isArray(s.files)&&s.files.every((row,i)=>Array.isArray(row)&&row.length===2&&row.every(v=>typeof v==='string')&&row[0].length>0&&(i===0||s.files[i-1][0]<row[0]))&&Array.isArray(s.dependencies)&&s.dependencies.length<=128&&s.dependencies.every(v=>typeof v==='string'&&v.length<=256);
const picture=v=>v===null||(v instanceof Blob&&v.type==='image/webp'&&v.size>0&&v.size<=MAX_PICTURE);
const card=v=>v===null||(exact(v,['description','tag'])&&typeof v.description==='string'&&typeof v.tag==='string'&&checkCard(v).length===0);
const copy=value=>{try{return structuredClone(value);}catch{fail('invalidGalleryCommand');}};
export function captureGalleryCommand(appId,source,patch){
 const c=copy({appId,source,patch});
 if(!Number.isSafeInteger(c.appId)||c.appId<1||!validSource(c.source)||!plain(c.patch)||!Object.keys(c.patch).length||Object.keys(c.patch).some(k=>!['picture','card','version'].includes(k))||('picture'in c.patch&&!picture(c.patch.picture))||('card'in c.patch&&!card(c.patch.card))||('version'in c.patch&&(typeof c.patch.version!=='string'||c.patch.version.length>4096))||('picture'in c.patch)!==('version'in c.patch))fail('invalidGalleryCommand');
 return {...c,source:{name:c.source.name,reference:c.source.reference,files:c.source.files,dependencies:c.source.dependencies}};
}
async function identity(appId,source){const bytes=new TextEncoder().encode(JSON.stringify({appId,...source}));return [...new Uint8Array(await crypto.subtle.digest('SHA-256',bytes))].map(b=>b.toString(16).padStart(2,'0')).join('');}
function validRecord(r,id){return exact(r,['profile','appId','sourceHash','reference','picture','card','version','revision'])&&Number.isSafeInteger(r.revision)&&r.revision>0&&r.profile===PROFILE&&r.appId===id&&typeof r.sourceHash==='string'&&/^[0-9a-f]{64}$/.test(r.sourceHash)&&reference(r.reference)&&picture(r.picture)&&card(r.card)&&typeof r.version==='string'&&r.version.length<=4096;}
// All owner, head, source bytes and prior-record reads share the transaction.
function transaction(db,appId,write){return new Promise((resolve,reject)=>{
 let tx,error,app,head,record,result;const files=[];let pending=4;
 const refuse=code=>{error??=galleryFailure(code);try{tx?.abort();}catch{}reject(error);};
 try{
  tx=db.transaction(['apps','files','refs'],write?'readwrite':'readonly');
  tx.onabort=()=>reject(error||galleryFailure('galleryStorageFailed'));tx.onerror=()=>refuse('galleryStorageFailed');tx.oncomplete=()=>resolve(result);
  const done=()=>{if(--pending)return;try{
   if(!app){if(write){refuse('gallerySourceChanged');return;}result=null;return;}
   const source={name:app.name,reference:head,files,dependencies:app.extraDeps||[]};
   if(!validSource(source)){refuse('galleryStateInvalid');return;}
   if(record!==undefined&&!validRecord(record,appId)){refuse('galleryStateInvalid');return;}
   if(!write){result={source,record};return;}
   if(JSON.stringify(source)!==JSON.stringify(write.source)){refuse('gallerySourceChanged');return;}
   if(record?.sourceHash===write.hash&&record.reference!==head){refuse('galleryStateInvalid');return;}
   const previous=record?.sourceHash===write.hash?record:{profile:PROFILE,appId,sourceHash:write.hash,reference:head,picture:null,card:null,version:'',revision:0};
   if(previous.revision===Number.MAX_SAFE_INTEGER){refuse('galleryStateInvalid');return;}
   result={...previous,...write.patch,revision:previous.revision+1};tx.objectStore('refs').put(result,`gallery:${appId}`);
  }catch{refuse('galleryStorageFailed');}};
  const a=tx.objectStore('apps').get(appId),h=tx.objectStore('refs').get(`${appId}:main`),r=tx.objectStore('refs').get(`gallery:${appId}`);
  a.onsuccess=()=>{app=a.result;done();};h.onsuccess=()=>{head=h.result;done();};r.onsuccess=()=>{record=r.result;done();};
  const f=tx.objectStore('files').openCursor(IDBKeyRange.bound(`${appId}:`,`${appId};`,false,true));
  f.onsuccess=()=>{try{const cursor=f.result;if(cursor){files.push([cursor.key.slice(String(appId).length+1),cursor.value]);cursor.continue();}else done();}catch{refuse('galleryStorageFailed');}};
 }catch{refuse('galleryStorageFailed');}
});}
export async function publishGallery(database,appId,source,patch){
 const captured=captureGalleryCommand(appId,source,patch);
 try { const [hash,db]=await Promise.all([identity(captured.appId,captured.source),database]);
 const result=await transaction(db,captured.appId,{...captured,hash});
 return structuredClone(result);
 }catch(error){throw galleryFailure(error?.code);}
}
export async function readGallery(database,appId){
 if(!Number.isSafeInteger(appId)||appId<1)fail('invalidGalleryCommand');
 try { const snapshot=await transaction(await database,appId,null);
 if(!snapshot?.record)return null;
 const {source,record}=snapshot;
 if(record.sourceHash!==await identity(appId,source))return null;
 if(record.reference!==source.reference)fail('galleryStateInvalid');
 return structuredClone(record);
 }catch(error){throw galleryFailure(error?.code);}
}
