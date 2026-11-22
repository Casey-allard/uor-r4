import assert from 'node:assert/strict';
import fs from 'node:fs';
import vm from 'node:vm';
import {test} from 'node:test';
import {blake3} from '../../try/vendor/prism-code/src/blake3.mjs';
import {readPreviewJournal,operateVersionPreview} from '../shell/version-preview-store.mjs';
import {encodePreviewRequest,previewAddressIdentity} from '../shell/version-preview-codec.mjs';
import {createVersionPreviewController} from '../shell/version-preview-controller.mjs';
const vectors=JSON.parse(fs.readFileSync(new URL('../../verification/version-preview-vectors.json',import.meta.url))).vectors;
const derive=async bytes=>'blake3:'+Buffer.from(blake3(bytes)).toString('hex'),enc=new TextEncoder(),dec=new TextDecoder();
const named=name=>{const v=vectors.find(x=>x.name===name);assert.ok(v,name);return structuredClone(v);};
function fixture(row=named('acquire')){
 const oid=previewAddressIdentity(row.input.command.target),current={oid,name:'fixture',files:[['a.txt','exact']],objects:[]};let raw=row.input.state.active?{...readPreviewJournal(undefined,1),state:structuredClone(row.input.state)}:undefined,writes=0;
 const source={name:'fixture',reference:row.input.state.active?previewAddressIdentity(row.input.state.current):oid,files:[['a.txt','exact']],dependencies:[]};
 const boundary={read:async()=>readPreviewJournal(raw,1),capture:async()=>({source:structuredClone(source),expected:structuredClone(raw)}),derive,prepare:async id=>({...current,oid:id}),resolve:async id=>id,prepareChild:async(_snapshot,parent)=>({...current,parent}),guest:{dispatch:async bytes=>{assert.equal(dec.decode(bytes),row.request);return enc.encode(row.response);}},commit:async(_captured,_prepared,j)=>{writes++;raw=structuredClone(j);}};
 return {boundary,source,get writes(){return writes;},get raw(){return raw;},set raw(v){raw=v;},request:{appId:1,kind:row.input.command.kind,owner:row.input.command.owner}};
}
test('journal captures exact owner and full identities and rejects corrupt persisted state',()=>{
 const host=fs.readFileSync(new URL('../shell/host.js',import.meta.url),'utf8');assert.ok(!host.includes('return { successMessage: "Reverted" }'));assert.ok(host.includes('purpose!=="preview"'));
 const j=readPreviewJournal(undefined,1);assert.equal(j.state.active,0);
 for(const raw of [{...j,appId:2},{...j,extra:true},{...j,state:{...j.state,owner:7}},{...j,state:{...j.state,current:[1,...Array(7).fill(0)]}},{...j,receipts:[{operationId:'x',identity:'unbound'}]}])assert.throws(()=>readPreviewJournal(raw,1),{code:'invalidStoredPreview'});
});
test('captured acquire commits guest state and exact duplicate cannot perform effects twice',async()=>{
 const f=fixture(),request={...f.request,operationId:'acquire'};const pending=operateVersionPreview(request,f.boundary);request.owner=99;
 const answer=await pending;assert.equal(f.writes,1);assert.equal(answer.journal.state.owner,7);assert.equal(answer.result.repositoryOutcome,'unchanged');
 f.raw.receipts[0].result={createdChatId:2,affectedChatId:null};const replay=await operateVersionPreview({...f.request,operationId:'acquire'},f.boundary);assert.equal(replay.replayed,true);assert.equal(replay.result.createdChatId,2);assert.equal(f.writes,1);
 await assert.rejects(()=>operateVersionPreview({...f.request,kind:3,operationId:'acquire'},f.boundary),{code:'previewOperationConflict'});
});
test('changed files identity dependencies and guest failure refuse before source commit',async()=>{
 for(const mode of ['files','dependencies','name','guest']){const f=fixture();if(mode==='files')f.source.files=[['a.txt','changed']];if(mode==='dependencies')f.source.dependencies=['other'];if(mode==='name')f.source.name='changed';if(mode==='guest')f.boundary.guest.dispatch=async()=>enc.encode('previewWrongOwner');await assert.rejects(()=>operateVersionPreview(f.request,f.boundary),{code:mode==='guest'?'previewWrongOwner':'previewSourceChanged'});assert.equal(f.writes,0);}
});
test('direct command results and late commit refusal preserve real preview acknowledgement',async()=>{
 const host=fs.readFileSync(new URL('../shell/host.js',import.meta.url),'utf8'),handlers=new Map(),events=[],result={repositoryOutcome:'target-applied',notification:null,runtimeAction:'restart',affectedChatId:null,createdChatId:null};
 const versionPreview={acquire:async()=>({acquired:true}),dispatch:async(appId,event)=>{events.push({appId,...event});return result;}};
 for(const channel of ['checkout-version','revert-version']){const start=host.indexOf(`handlers.set("${channel}"`),end=host.indexOf('\n});',start)+4;vm.runInNewContext(host.slice(start,end),{handlers,versionPreview,crypto,previewFailure:code=>Object.assign(Error(code),{code})});}
 assert.equal(await handlers.get('checkout-version')({appId:1,purpose:'preview',versionId:'a'.repeat(64)}),result);assert.equal(events.at(-1).type,'SELECT_VERSION');
 assert.equal(await handlers.get('checkout-version')({appId:1,purpose:'return',branch:'main'}),result);assert.equal(events.at(-1).type,'CLOSE');
 assert.equal(await handlers.get('revert-version')({appId:1,previousVersionId:'b'.repeat(64)}),result);assert.equal(events.at(-1).type,'RESTORE');
 await assert.rejects(()=>handlers.get('checkout-version')({appId:1,purpose:'return',branch:'other'}),{code:'previewUnsupportedBranch'});
 versionPreview.acquire=async()=>({acquired:false});await assert.rejects(()=>handlers.get('checkout-version')({appId:1,purpose:'preview',versionId:'a'.repeat(64)}),{code:'previewWindowBusy'});
 const f=fixture();f.boundary.commit=async()=>{throw Object.assign(Error(),{code:'previewStorageFailed'});};await assert.rejects(()=>operateVersionPreview(f.request,f.boundary),{code:'previewStorageFailed'});
 for(const patch of [{owner:0},{appId:0},{kind:8},{kind:1,versionId:'unused'},{mode:'accept'},{operationId:''},{expectedHeadOid:'wrong'}])await assert.rejects(()=>operateVersionPreview({...f.request,...patch},f.boundary),{code:'invalidPreviewCommand'});
});
test('window lock refuses competing ownership and missing interest cannot acknowledge a selection',async()=>{
 const f=fixture();let held=false;const locks={async request(_name,_opts,fn){if(held)return fn(null);held=true;try{return await fn({name:'lock'});}finally{held=false;}}};
 const make=()=>createVersionPreviewController({locks,store:()=>f.boundary,mintOwner:()=>7,validateEvent:event=>({success:true,data:event}),publish:()=>{}}),a=make(),b=make();
 assert.deepEqual(await a.acquire(1),{acquired:true});assert.deepEqual(await b.acquire(1),{acquired:false});assert.equal(a.snapshot(1).state.type,'browsing');
 await assert.rejects(()=>b.dispatch(1,{type:'SELECT_VERSION',operationId:'select',versionId:'x'}),{code:'previewInterestMissing'});await assert.rejects(()=>a.dispatch(1,{type:'SWITCH_BRANCH',branch:'other',operationId:'branch'}),{code:'previewUnsupportedBranch'});a.dispose();b.dispose();
});
test('selection return restore and abandoned recovery preserve literal guest source targets',async()=>{
 for(const row of vectors.filter(x=>x.input&&typeof x.expected==='object'))assert.equal(dec.decode(encodePreviewRequest(row.input)),row.request);
 for(const name of ['select','selectSame','returnOrigin','restorePrepared','recover']){
  const row=named(name),f=fixture(row),requested=previewAddressIdentity(row.input.command.target);let committed,parent;
  f.boundary.prepareChild=async(snapshot,origin)=>{parent=origin;return {...snapshot,oid:requested,objects:[]};};
  const commit=f.boundary.commit;f.boundary.commit=async(captured,prepared,journal,options)=>{committed={prepared,options};return commit(captured,prepared,journal);};
  const answer=await operateVersionPreview({...f.request,...([2,4].includes(row.input.command.kind)?{versionId:requested}:{})},f.boundary);
  assert.deepEqual(answer.journal.state,row.expected.state,name);assert.equal(committed.options.writeRequested,row.expected.writeRequested,name);
  if(row.expected.writeRequested)assert.equal(committed.prepared.oid,previewAddressIdentity(row.expected.target),name);
  if(name==='selectSame')assert.equal(answer.result.runtimeAction,'none');
  if(name==='restorePrepared')assert.equal(parent,previewAddressIdentity(row.input.state.origin));
 }
 const f=fixture();f.boundary.derive=async()=>{throw Error('private');};await assert.rejects(()=>operateVersionPreview({...f.request,operationId:'identity'},f.boundary),{code:'previewUnavailable'});
});
