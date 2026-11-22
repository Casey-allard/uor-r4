import assert from 'node:assert/strict';
import fs from 'node:fs';
import vm from 'node:vm';
import {test} from 'node:test';
import {blake3} from '../../try/vendor/prism-code/src/blake3.mjs';
import {versionAddress,versionAlias,versionRows,readVersionHistory,listStoredVersions,resolveStoredVersion,setStoredVersionMetadata,verifiedVersionChanges} from '../shell/version-history.mjs';
const enc=new TextEncoder(),derive=async bytes=>'blake3:'+Buffer.from(blake3(bytes)).toString('hex'),a='blake3:'+'a'.repeat(64),b='blake3:'+'b'.repeat(64);
const row=oid=>({oid,message:'version',timestamp:1,dbTimestamp:null,isFavorite:false,note:null});
// Request/commit ordering fixture; actual IndexedDB is exercised by the browser gate.
function database(initial=[row(a)],{missing=false,late=false}={}){
 let rows=structuredClone(initial),writes=0;const waiting=[];let busy=false;
 const next=()=>{if(busy||!waiting.length)return;busy=true;waiting.shift()();};
 return {get rows(){return rows;},get writes(){return writes;},transaction(){let pending=0,started=false,ended=false,draft;const requests=[];
  const finish=()=>{if(ended||pending||!started)return;ended=true;queueMicrotask(()=>{if(late)tx.onabort?.();else{if(draft){rows=draft;writes++;}tx.oncomplete?.();}busy=false;next();});};
  const schedule=f=>{pending++;const execute=()=>queueMicrotask(()=>{if(ended)return;f();pending--;finish();});if(started)execute();else requests.push(execute);};
  const tx={abort(){if(ended)return;ended=true;queueMicrotask(()=>{tx.onabort?.();busy=false;next();});},objectStore(name){return {get(){const q={};schedule(()=>{q.result=structuredClone(name==='apps'?(missing?undefined:{id:1}):rows);q.onsuccess?.();});return q;},put(value){draft=structuredClone(value);}};}};
  waiting.push(()=>{started=true;for(const request of requests)request();finish();});queueMicrotask(next);return tx;
 }};
}
async function snapshots(){const objects=new Map();async function make(files,parent=''){const entries=[];for(const [path,text] of files){const bytes=enc.encode(text),key=await derive(bytes);objects.set(key,bytes);entries.push([path,key,bytes.length]);}const bytes=enc.encode(JSON.stringify({name:'app',parent,entries})),oid=await derive(bytes);objects.set(oid,bytes);return oid;}const parent=await make([['change.txt','old'],['delete.txt','gone']]),oid=await make([['change.txt','new'],['add.txt','😀\r\n']],parent);return {oid,parent,objects,boundary:{readObject:async key=>objects.get(key),derive,admit:async()=>true},db:database([row(oid),row(parent)])};}
test('hexadecimal aliases are reversible exact canonical identities and invalid requests refuse',async()=>{
 assert.equal(versionAlias(a),'a'.repeat(64));assert.equal(versionAddress('A'.repeat(64)),a);assert.equal(versionAddress(a),a);
 for(const value of ['a'.repeat(40),'main','blake3:'+ 'G'.repeat(64),null,1])assert.throws(()=>versionAddress(value),{code:'invalidVersionHistory'});
 const db=database();assert.equal((await listStoredVersions(db,{appId:1}))[0].oid,'a'.repeat(64));assert.equal(await resolveStoredVersion(db,1,'A'.repeat(64)),a);
 await assert.rejects(()=>resolveStoredVersion(db,1,b),{code:'versionHistoryMissing'});await assert.rejects(()=>listStoredVersions(db,{appId:0}),{code:'invalidVersionHistory'});
 assert.equal((await listStoredVersions(db,{appId:1,ref:a})).length,1);
});
test('atomic metadata returns imported shape and concurrent note and favorite preserve each other',async()=>{
 const db=database();await Promise.all([setStoredVersionMetadata(db,{appId:1,versionId:'a'.repeat(64),isFavorite:true},'isFavorite'),setStoredVersionMetadata(db,{appId:1,versionId:a,note:'Exact note 😀'},'note')]);
 assert.equal(db.rows[0].isFavorite,true);assert.equal(db.rows[0].note,'Exact note 😀');assert.equal(db.writes,2);
 assert.deepEqual(await setStoredVersionMetadata(db,{appId:1,versionId:a,note:null},'note'),{oid:'a'.repeat(64),isFavorite:true,note:null});
 assert.throws(()=>setStoredVersionMetadata(db,{appId:1,versionId:a,note:'x'.repeat(10001)},'note'),{code:'invalidVersionHistory'});
 assert.throws(()=>setStoredVersionMetadata(db,{appId:1,oid:a,note:'legacy wrong field'},'note'),{code:'invalidVersionHistory'});
});
test('late abort missing projects unknown versions and corrupt rows never acknowledge writes',async()=>{
 for(const [db,code] of [[database([row(a)],{late:true}),'versionHistoryStorageFailed'],[database([row(a)],{missing:true}),'versionHistoryMissing'],[database([row(b)]),'versionHistoryMissing'],[database([{}]),'versionHistoryStateInvalid']]){await assert.rejects(()=>setStoredVersionMetadata(db,{appId:1,versionId:a,isFavorite:true},'isFavorite'),{code});assert.equal(db.writes,0);}
 await assert.rejects(()=>readVersionHistory({transaction(){throw Error('private');}},1),{code:'versionHistoryStorageFailed'});
 await assert.rejects(()=>readVersionHistory(database([],{missing:true}),1),{code:'versionHistoryMissing'});
 for(const rows of [null,[{...row(a),timestamp:NaN}],[{...row(a),oid:'a'.repeat(64)}],[{...row(a),note:1}]])assert.throws(()=>versionRows(rows),{code:'versionHistoryStateInvalid'});
});
test('differences use captured verified parent and file bytes with complete added changed deleted text',async()=>{
 const f=await snapshots();assert.deepEqual(await verifiedVersionChanges(f.db,{appId:1,versionId:versionAlias(f.oid)},f.boundary),[{path:'add.txt',type:'added',oldContent:'',newContent:'😀\r\n'},{path:'change.txt',type:'modified',oldContent:'old',newContent:'new'},{path:'delete.txt',type:'deleted',oldContent:'gone',newContent:''}]);
 const root=await verifiedVersionChanges(f.db,{appId:1,versionId:f.parent},f.boundary);assert.ok(root.every(x=>x.type==='added'));
});
test('missing corrupt and core-refused objects cannot become empty successful differences',async()=>{
 for(const mode of ['missing','snapshot','parent','file','admission']){const f=await snapshots();let code;
  if(mode==='missing'){f.objects.delete(f.oid);code='missingSnapshotObject';}
  if(mode==='snapshot'){f.objects.set(f.oid,enc.encode('{}'));code='snapshotIdentityMismatch';}
  if(mode==='parent'){f.objects.set(f.parent,enc.encode('{}'));code='snapshotIdentityMismatch';}
  if(mode==='file'){const state=JSON.parse(new TextDecoder().decode(f.objects.get(f.oid)));const key=state.entries[0][1];f.objects.set(key,enc.encode('bad'));code='fileIdentityMismatch';}
  if(mode==='admission'){f.boundary.admit=async()=>false;code='snapshotCoreRefused';}
  await assert.rejects(()=>verifiedVersionChanges(f.db,{appId:1,versionId:f.oid},f.boundary),{code});
 }
});
test('actual host metadata adapters and mandatory browser gate bind the canonical ownership',async()=>{
 const host=fs.readFileSync(new URL('../shell/host.js',import.meta.url),'utf8'),handlers=new Map(),db=database();
 for(const channel of ['set-version-favorite','set-version-note'])vm.runInNewContext(host.split('\n').find(s=>s.startsWith(`handlers.set("${channel}"`)),{handlers,dbReady:Promise.resolve(db),setStoredVersionMetadata});
 const result=await handlers.get('set-version-favorite')({appId:1,versionId:a,isFavorite:true});assert.equal(result.isFavorite,true);assert.equal(result.oid,'a'.repeat(64));
 const {commandRegistry}=await import('../shell/command-registry.mjs');assert.equal(commandRegistry.resolve('set-version-note',handlers.get('set-version-note')).owner,'FoundrySpecification.versionHistoryBoundary');
 assert.ok(fs.readFileSync(new URL('../../.github/workflows/studio.yml',import.meta.url),'utf8').includes('node tools/canonical_version_history_browser.mjs'));
 assert.ok(fs.readFileSync(new URL('./shell_parse.mjs',import.meta.url),'utf8').includes('version-history.mjs'));
});
