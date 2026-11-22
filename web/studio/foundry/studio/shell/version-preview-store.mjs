// FoundrySpecification.durableVersionPreviewBoundary. Only guest-admitted transitions reach commit.
import {previewWithGuest,previewAddressWords,previewAddressIdentity,capturePreviewState} from './version-preview-codec.mjs';
import {captureProjectSource,commitPreparedProject} from './project-edit.mjs';
import {prepareSnapshot} from './snapshot-restore.mjs';
import {prepareExistingProject} from './project-creation.mjs';
import {snapshotDependencies} from './project-lifecycle.mjs';
import {resolveStoredVersion} from './version-history.mjs';
import {storageRequest} from './storage-request.mjs';
const messages={invalidStoredPreview:'Stored version preview is invalid',previewUnavailable:'Version preview is unavailable',previewStorageFailed:'Version preview could not be saved',previewSourceChanged:'Project source changed during version preview',previewMissing:'The preview project no longer exists',previewWindowBusy:'Another window owns version preview',previewInterestMissing:'Open version history before selecting a version',previewOperationConflict:'This operation identity has different content',previewUnsupportedBranch:'This project has only the main branch'};
export const previewFailure=code=>Object.assign(new Error(messages[code]||'Version preview operation refused'),{code});
const copy=x=>{try{return structuredClone(x);}catch{throw previewFailure('invalidPreviewCommand');}};
const exact=(x,keys)=>x&&typeof x==='object'&&!Array.isArray(x)&&Object.keys(x).sort().join()===keys.split(',').sort().join();
const identity=x=>typeof x==='string'&&/^blake3:[a-f0-9]{64}$/.test(x);
const id=x=>{if(!Number.isSafeInteger(x)||x<1)throw previewFailure('invalidPreviewCommand');return x;};
const zero=()=>Array(8).fill(0);
const result=(changed,notification=null)=>({repositoryOutcome:changed?'target-applied':'unchanged',notification,runtimeAction:changed?'restart':'none',affectedChatId:null,createdChatId:null});
export function readPreviewJournal(raw,appId){
 id(appId);if(raw===undefined)return {profile:'foundry/version-preview/1',appId,state:{revision:0,active:0,owner:0,origin:zero(),current:zero()},receipts:[]};
 try{const j=copy(raw);if(!exact(j,'profile,appId,state,receipts')||j.profile!=='foundry/version-preview/1'||j.appId!==appId||!Array.isArray(j.receipts)||j.receipts.length>128)throw Error();j.state=capturePreviewState(j.state);const ids=new Set();for(const receipt of j.receipts){if((!exact(receipt,'operationId,identity')&&!exact(receipt,'operationId,identity,result'))||typeof receipt.operationId!=='string'||!receipt.operationId||receipt.operationId.length>256||!identity(receipt.identity)||ids.has(receipt.operationId))throw Error();if(receipt.result!==undefined&&(!exact(receipt.result,'createdChatId,affectedChatId')||[receipt.result.createdChatId,receipt.result.affectedChatId].some(n=>n!==null&&(!Number.isSafeInteger(n)||n<1))))throw Error();ids.add(receipt.operationId);}return j;}catch{throw previewFailure('invalidStoredPreview');}
}
const sameFiles=(left,right)=>JSON.stringify([...left].sort(([a],[b])=>a<b?-1:a>b?1:0))===JSON.stringify([...right].sort(([a],[b])=>a<b?-1:a>b?1:0));
export async function operateVersionPreview(raw,boundary){
 const request=copy(raw);if(!request||Object.keys(request).some(k=>!['appId','owner','kind','versionId','operationId','expectedHeadOid','mode','effectIdentity'].includes(k))||!Number.isInteger(request.owner)||request.owner<1||request.owner>0xffffffff||![1,2,3,4,5].includes(request.kind))throw previewFailure('invalidPreviewCommand');
 if(request.effectIdentity!==undefined&&!identity(request.effectIdentity))throw previewFailure('invalidPreviewCommand');
 id(request.appId);if(request.operationId!==undefined&&(typeof request.operationId!=='string'||!request.operationId||request.operationId.length>256))throw previewFailure('invalidPreviewCommand');
 if(request.mode!==undefined&&!(request.kind===4&&['accept','checkpoint'].includes(request.mode)))throw previewFailure('invalidPreviewCommand');
 if(request.kind===2||request.kind===4&&!request.mode){if(typeof request.versionId!=='string')throw previewFailure('invalidPreviewCommand');}else if(request.versionId!==undefined)throw previewFailure('invalidPreviewCommand');
 if(request.expectedHeadOid!==undefined&&(request.kind!==4||!identity(request.expectedHeadOid)))throw previewFailure('invalidPreviewCommand');
 const captured=await boundary.capture(),journal=readPreviewJournal(captured.expected,request.appId);
 const {owner,...intent}=request;let operationIdentity;
 if(request.operationId!==undefined){try{operationIdentity=await boundary.derive(new TextEncoder().encode(JSON.stringify(Object.fromEntries(Object.entries(intent).sort(([a],[b])=>a<b?-1:a>b?1:0)))));}catch{throw previewFailure('previewUnavailable');}if(!identity(operationIdentity))throw previewFailure('previewUnavailable');const receipt=journal.receipts.find(r=>r.operationId===request.operationId);if(receipt){if(receipt.identity!==operationIdentity)throw previewFailure('previewOperationConflict');return {journal,result:{...result(false),...(receipt.result||{})},replayed:true};}}
 const current=await boundary.prepare(captured.source.reference);
 if(current.name!==captured.source.name||!sameFiles(current.files,captured.source.files)||JSON.stringify(snapshotDependencies(current.files))!==JSON.stringify(captured.source.dependencies))throw previewFailure('previewSourceChanged');
 if(journal.state.active&&request.kind!==5&&!request.mode&&previewAddressIdentity(journal.state.current)!==captured.source.reference)throw previewFailure('previewSourceChanged');
 if(request.expectedHeadOid!==undefined&&request.expectedHeadOid!==(journal.state.active?previewAddressIdentity(journal.state.origin):captured.source.reference))throw previewFailure('previewSourceChanged');
 let prepared={...current,objects:[]},target=zero(),recordVersion=false;
 if(request.kind===1)target=previewAddressWords(captured.source.reference);
 if(request.kind===2){const oid=await boundary.resolve(request.versionId);prepared={...await boundary.prepare(oid),objects:[]};target=previewAddressWords(oid);}
 if(request.kind===3&&journal.state.active){const oid=await boundary.resolve(previewAddressIdentity(journal.state.origin));prepared={...await boundary.prepare(oid),objects:[]};}
 if(request.kind===4){
  if(request.mode==='accept')target=previewAddressWords(captured.source.reference);
  else{const chosen=request.mode==='checkpoint'?current:await boundary.prepare(await boundary.resolve(request.versionId));const parent=request.mode==='checkpoint'?captured.source.reference:journal.state.active?previewAddressIdentity(journal.state.origin):captured.source.reference;prepared=await boundary.prepareChild(chosen,parent);target=previewAddressWords(prepared.oid);recordVersion=true;}
 }
 const answer=await previewWithGuest(boundary.guest,{state:journal.state,expectedRevision:journal.state.revision,command:{kind:request.kind,owner,target}});if(!answer.ok)throw previewFailure(answer.code);
 if(answer.writeRequested&&previewAddressIdentity(answer.target)!==prepared.oid)throw previewFailure('invalidStoredPreview');
 journal.state=answer.state;if(request.operationId!==undefined){journal.receipts.push({operationId:request.operationId,identity:operationIdentity});if(journal.receipts.length>128)journal.receipts.shift();}
 const next=readPreviewJournal(journal,request.appId);const committed=await boundary.commit(captured,prepared,next,{writeRequested:answer.writeRequested,recordVersion});
 return {journal:next,result:{...result(answer.writeRequested&&prepared.oid!==captured.source.reference),...(committed&&typeof committed==="object"?{createdChatId:committed.createdChatId??null,affectedChatId:committed.affectedChatId??null}:{})},replayed:false,createdVersion:recordVersion?prepared.oid:null};
}
export function previewProjectStore(db,appId,model,{chatUpdate}={}){
 id(appId);return {
  guest:model.guest,derive:model.derive,
  read:async()=>{if(!await storageRequest(db,'apps','readonly',s=>s.get(appId)))throw previewFailure('previewMissing');return readPreviewJournal(await storageRequest(db,'refs','readonly',s=>s.get(`preview:${appId}`)),appId);},
  capture:async()=>{if(!await storageRequest(db,'apps','readonly',s=>s.get(appId)))throw previewFailure('previewMissing');const source=await captureProjectSource(db,appId),expected=await storageRequest(db,'refs','readonly',s=>s.get(`preview:${appId}`));return {source,expected};},
  prepare:oid=>prepareSnapshot(oid,model),resolve:reference=>resolveStoredVersion(db,appId,reference),
  prepareChild:(snapshot,parent)=>prepareExistingProject({name:snapshot.name,files:Object.fromEntries(snapshot.files)},model,parent),
  commit:async(captured,prepared,journal,{writeRequested,recordVersion})=>{
   const changedPaths=writeRequested?[...new Set([...captured.source.files.map(([p])=>p),...prepared.files.map(([p])=>p),'prism/spec.json'])]:[];
   try{return await commitPreparedProject(db,appId,changedPaths,captured.source,prepared,{recordVersion:writeRequested&&recordVersion,chatUpdate,previewUpdate:{expected:captured.expected,state:journal},...(writeRequested?{name:prepared.name,dependencies:snapshotDependencies(prepared.files),message:'Restore historical source'}:{})});}catch(error){if(error?.code==='projectEditConflict')throw chatUpdate?Object.assign(new Error('Source or chat history changed; review the restoration again'),{code:'messageRestoreConflict'}):previewFailure('previewSourceChanged');if(error?.code==='projectEditStorageFailed')throw previewFailure('previewStorageFailed');throw error;}
  },
 };
}
