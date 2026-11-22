import assert from 'node:assert/strict';
import fs from 'node:fs';
import {test} from 'node:test';
import {describePortableBuild,withPortableBuildPermission,portablePermissionFailure} from '../shell/portable-build-permission.mjs';
const input={id:'portable-control',source:'blake3:'+'a'.repeat(64)};
function fixture(){let time=1000,record,grants=0,revokes=0,closed=0,comparisons=0;const controller=new AbortController();return {controller,get grants(){return grants;},get revokes(){return revokes;},get closed(){return closed;},get comparisons(){return comparisons;},setTime:t=>time=t,revoke:()=>record={serve:false},changeSource:()=>record.previewConsent.source="blake3:"+"b".repeat(64),boundary:{now:()=>time,online:()=>true,pollMs:10,authorize:async d=>({purpose:"portable-build",disclosure:describePortableBuild(d),expires:time+10000,signal:controller.signal,close:()=>closed++}),assertSource:async()=>comparisons++,grant:async(id,p)=>{grants++;record={serve:true,previewConsent:{profile:'foundry/preview-consent/1',id,source:p.disclosure.source},network:{endpoints:[...p.disclosure.endpoints]}};},readGrant:async()=>record,revoke:async()=>{revokes++;record=undefined;}}};}
test('portable disclosure binds only exact source and fixed destinations',()=>{
 const d=describePortableBuild(input);assert.deepEqual(d.endpoints,['https://esm.sh','https://cdn.tailwindcss.com']);assert.throws(()=>d.endpoints.push('https://other.invalid'),TypeError);
 for(const patch of [{source:'private'}, {id:'../escape'},{extra:true}])assert.throws(()=>describePortableBuild({...input,...patch}),{code:'invalidPortableBuildPermission'});
});
test('denial and stale source prevent effects and temporary grants',async()=>{
 const f=fixture();f.boundary.authorize=async()=>{throw portablePermissionFailure('portableBuildPermissionDenied');};await assert.rejects(withPortableBuildPermission(input,()=>assert.fail('effect before consent'),f.boundary),{code:'portableBuildPermissionDenied'});assert.equal(f.grants,0);
 const stale=fixture();stale.boundary.assertSource=async()=>{throw Object.assign(Error(),{code:'buildSourceChanged'});};await assert.rejects(withPortableBuildPermission(input,()=>assert.fail('stale source executed'),stale.boundary),{code:'buildSourceChanged'});assert.equal(stale.grants,0);assert.equal(stale.closed,1);
});
test('accepted builds check source and grant before effects and clean up after success',async()=>{
 const f=fixture();const result=await withPortableBuildPermission(input,async lease=>{assert.equal(f.grants,1);assert.equal(f.comparisons,1);await lease.check();return 'artifact';},f.boundary);assert.equal(result,'artifact');assert.equal(f.comparisons,2);assert.equal(f.revokes,1);assert.equal(f.closed,1);
});
test('revocation and stop abort active operations and prevent output promotion',async()=>{
 for(const kind of ['stored','source','stop']){const f=fixture();let aborted=false;await assert.rejects(withPortableBuildPermission(input,lease=>new Promise(resolve=>{lease.signal.addEventListener('abort',()=>{aborted=true;resolve('late artifact');});if(kind==='stored')f.revoke();else if(kind==='source')f.changeSource();else f.controller.abort();}),f.boundary),{code:'portableBuildPermissionRevoked'});assert.equal(aborted,true);assert.equal(f.revokes,1);assert.equal(f.closed,1);}
});
test('offline expiry malformed approval and operation failure preserve cleanup',async()=>{
 for(const kind of ['offline','expired']){const f=fixture();if(kind==='offline')f.boundary.online=()=>false;else f.boundary.authorize=async d=>({purpose:"portable-build",disclosure:describePortableBuild(d),expires:0,signal:f.controller.signal,close:()=>{}});await assert.rejects(withPortableBuildPermission(input,()=>assert.fail('unavailable build executed'),f.boundary),{code:kind==='offline'?'portableBuildPermissionOffline':'portableBuildPermissionExpired'});assert.equal(f.grants,0);}
 const f=fixture();await assert.rejects(withPortableBuildPermission(input,()=>{throw Error('compiler failed');},f.boundary),/compiler failed/);assert.equal(f.revokes,1);assert.equal(f.closed,1);
 const invalid=fixture();invalid.boundary.authorize=async()=>({});await assert.rejects(withPortableBuildPermission(input,()=>assert.fail(),invalid.boundary),{code:'invalidPortableBuildPermission'});
});
test('actual portable compiler and styling frame consume the same consent lease',()=>{
 const source=fs.readFileSync(new URL('../shell/host.js',import.meta.url),'utf8'),start=source.indexOf('async function buildPortable('),end=source.indexOf('// Runs inside the compile frame',start),body=source.slice(start,end);
 for(const token of ['withPortableBuildPermission','source:source.reference','signal:lease.signal','await lease.check()','compileTailwind(compilePage, lease)','assertBuildSourceCurrent(await dbReady, appId, source)'])assert.ok(body.includes(token),token);
 const styling=source.slice(source.indexOf('async function compileTailwind('),source.indexOf('window.__portable'));
 assert.ok(styling.includes('const id = lease.id'));assert.ok(styling.includes('lease.signal.addEventListener("abort", removeFrame'));assert.ok(!styling.includes('await grantPreview('));
});
