// FoundrySpecification.durableBuildWorkBoundary; semantic transitions belong to WorkProtocol.
import {executeWorkWithGuest,encodeWorkRequest} from './work-codec.mjs';
import {captureProjectSource,commitPreparedProject} from './project-edit.mjs';
import {storageRequest} from './storage-request.mjs';
const messages={invalidBuildWork:'Invalid build work',buildWorkBusy:'A build already owns this project',buildWorkUnavailable:'Durable build work is unavailable',buildWorkConflict:'Build work changed; read its current state again',buildWorkSourceChanged:'The build source changed',buildWorkArtifactInvalid:'The retained build artifact is invalid',buildWorkLimitExceeded:'The retained build journal is full'};
export const buildWorkFailure=code=>Object.assign(new Error(messages[code]||messages.buildWorkUnavailable),{code:Object.hasOwn(messages,code)?code:'buildWorkUnavailable'});
const address=s=>typeof s==='string'&&/^blake3:[a-f0-9]{64}$/.test(s);
const same=(a,b)=>JSON.stringify(a)===JSON.stringify(b);
const uint=n=>Number.isInteger(n)&&n>=0&&n<=0xffffffff;
function copyJournal(input){
 if(input===undefined)return {profile:'foundry/build-work/1',records:[]};
 try{
  if(input.profile!=='foundry/build-work/1'||!Array.isArray(input.records)||input.records.length>32)throw Error();
  const keys=new Set();
  for(const row of input.records){
   if(!address(row.source)||typeof row.environment!=='string'||!/^[a-f0-9]{64}$/.test(row.environment)||!uint(row.attempts)||!row.attempts||!uint(row.state.owner)||!row.state.owner||row.state.capacity!==1||!['running','interrupted','complete'].includes(row.status))throw Error();
   encodeWorkRequest({kind:'transition',state:row.state,owner:row.state.owner,revision:row.state.revision,action:{kind:'pause'}});
   const key=row.source+row.environment;if(keys.has(key))throw Error();keys.add(key);
   if(row.status==='complete'&&(row.state.phase!==3||row.state.remaining!==0||row.state.spent!==1||row.state.reserved!==0||row.state.checkpoint!==1||!address(row.artifact)))throw Error();
  }
  return structuredClone(input);
 }catch{throw buildWorkFailure('invalidBuildWork');}
}
function artifactBytes(value,source){
 try{
  if(!value||typeof value.html!=='string'||value.sourceReference!==source||!Array.isArray(value.moduleFiles)||!Array.isArray(value.moduleClosure?.modules))throw Error();
  for(const file of value.moduleFiles)if(typeof file.path!=='string'||!(file.bytes instanceof Uint8Array))throw Error();
  const bytes=new TextEncoder().encode(JSON.stringify(value));if(bytes.length>16*1024*1024)throw Error();return bytes;
 }catch{throw buildWorkFailure('buildWorkArtifactInvalid');}
}
export async function runBuildWork(input,operation,{guest,locks=globalThis.navigator?.locks,crypto=globalThis.crypto,read,save,derive,assertSource}={}){
 let request;try{request=structuredClone(input);}catch{throw buildWorkFailure('invalidBuildWork');}
 if(!request||Object.keys(request).sort().join()!=='appId,environment,source'||!Number.isSafeInteger(request.appId)||request.appId<1||!address(request.source)||typeof request.environment!=='string'||!/^[a-f0-9]{64}$/.test(request.environment)||typeof operation!=='function')throw buildWorkFailure('invalidBuildWork');
 if(!locks?.request||!crypto?.getRandomValues||![read,save,derive,assertSource].every(f=>typeof f==='function'))throw buildWorkFailure('buildWorkUnavailable');
 let entered=false;try{return await locks.request(`foundry:portable-build:${request.appId}`,{mode:'exclusive',ifAvailable:true},async lock=>{
  entered=true;if(!lock)throw buildWorkFailure('buildWorkBusy');
  await assertSource();let previous=await read(),journal=copyJournal(previous);
  let row=journal.records.find(r=>r.source===request.source&&r.environment===request.environment);
  if(row?.status==='complete'){
   const bytes=artifactBytes(row.output,request.source);if(await derive(bytes)!==row.artifact)throw buildWorkFailure('buildWorkArtifactInvalid');await assertSource();return structuredClone(row.output);
  }
  let owner=0;try{for(let i=0;i<8&&(!owner||owner===row?.state.owner);i++)owner=crypto.getRandomValues(new Uint32Array(1))[0];}catch{throw buildWorkFailure('buildWorkUnavailable');}if(!owner||owner===row?.state.owner)throw buildWorkFailure('buildWorkUnavailable');
  const transition=async action=>{const answer=await executeWorkWithGuest(guest,{kind:'transition',state:row.state,owner:row.state.owner,revision:row.state.revision,action});if(!answer.ok)throw Object.assign(new Error('The work guest refused the transition'),{code:answer.code});row.state=answer.state;};
  const commit=async()=>{await save(previous,journal);previous=structuredClone(journal);};
  if(!row){
   if(journal.records.length>=32)throw buildWorkFailure('buildWorkLimitExceeded');
   const answer=await executeWorkWithGuest(guest,{kind:'initialize',owner,capacity:1,remaining:1});if(!answer.ok)throw Object.assign(new Error('The work guest refused initialization'),{code:answer.code});
   row={source:request.source,environment:request.environment,state:answer.state,attempts:1,status:'running',reason:null,artifact:null,output:null};journal.records.push(row);
   await transition({kind:'reserve',amount:1});await transition({kind:'start'});
  }else{
   if(row.attempts===0xffffffff)throw buildWorkFailure('buildWorkLimitExceeded');
   if(row.state.phase===1)await transition({kind:'pause'});
   await transition({kind:'transfer',owner});await transition({kind:'resume'});row.attempts++;row.status='running';row.reason=null;
  }
  await commit();
  try{
   await assertSource();const output=structuredClone(await operation());const bytes=artifactBytes(output,request.source),artifact=await derive(bytes);if(!address(artifact))throw buildWorkFailure('buildWorkArtifactInvalid');await assertSource();
   await transition({kind:'progress',cost:1,units:1,checkpoint:1});await transition({kind:'finish'});
   row.output=output;row.artifact=artifact;row.status='complete';await commit();return structuredClone(output);
  }catch(error){
   // Keep the last durable state if the completion transaction did not commit.
   journal=copyJournal(previous);row=journal.records.find(r=>r.source===request.source&&r.environment===request.environment);
   if(row.state.phase===1)await transition({kind:'pause'});row.status='interrupted';row.reason='interrupted';
   // Recovery records must survive a source change; storage still compares ownership.
   await save(previous,journal,{recovery:true});throw error;
  }
 });}catch(error){if(!entered)throw buildWorkFailure('buildWorkUnavailable');throw error;}
}
export function buildWorkStore(db,appId,captured){
 const read=()=>storageRequest(db,'refs','readonly',store=>store.get(`work:${appId}`));
 const assertSource=async()=>{const now=await captureProjectSource(db,appId);if(!same(now,captured))throw buildWorkFailure('buildWorkSourceChanged');};
 const save=async(expected,state,{recovery=false}={})=>{
  const source=recovery?await captureProjectSource(db,appId):captured;
  try{await commitPreparedProject(db,appId,[],source,{oid:source.reference,files:source.files,objects:[]},{recordVersion:false,workUpdate:{expected,state}});}catch(error){if(error?.code==='projectEditConflict')throw buildWorkFailure('buildWorkConflict');throw error;}
 };
 return {read,save,assertSource};
}
