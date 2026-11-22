import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import {test} from 'node:test';
import {hostBundleInputs,checkHostBundleBytes,checkHostBundleCompiler} from './host_bundle.mjs';
const fixture=()=>fs.mkdtempSync(path.join(os.tmpdir(),'foundry-host-policy-'));
test('source capture binds actual bytes and returns only root-relative regular file identities',()=>{
 const root=fixture();try{fs.mkdirSync(path.join(root,'nested'));fs.writeFileSync(path.join(root,'nested/a.mjs'),'export const a=1;');const inputs=hostBundleInputs(root);const bytes=inputs.capture(path.join(root,'nested/a.mjs'));assert.equal(bytes.toString(),'export const a=1;');assert.equal(inputs.captured.get('nested/a.mjs').sha256,'6b76a8cb884637895f9aef9a5776db8f459b49cb2a4f6bd3a963ac5d8d60a85a');}finally{fs.rmSync(root,{recursive:true});}
});
test('escaping paths and symbolic links cannot enter the source graph',()=>{
 const root=fixture();try{fs.mkdirSync(path.join(root,'shell'));fs.writeFileSync(path.join(root,'outside.mjs'),'export const x=1;');const inputs=hostBundleInputs(path.join(root,'shell'));assert.throws(()=>inputs.capture(path.join(root,'outside.mjs')),{code:'invalidHostBundleInput'});fs.symlinkSync('../outside.mjs',path.join(root,'shell/link.mjs'));assert.throws(()=>inputs.capture(path.join(root,'shell/link.mjs')),{code:'invalidHostBundleInput'});}finally{fs.rmSync(root,{recursive:true});}
});
test('source file count and byte capacity reject excess inputs',()=>{
 const root=fixture();try{const inputs=hostBundleInputs(root);for(let i=0;i<256;i++){const p=path.join(root,i+'.mjs');fs.writeFileSync(p,'');inputs.capture(p);}fs.writeFileSync(path.join(root,'overflow.mjs'),'');assert.throws(()=>inputs.capture(path.join(root,'overflow.mjs')),{code:'invalidHostBundleInput'});fs.writeFileSync(path.join(root,'large.mjs'),Buffer.alloc(16*1024*1024+1));assert.throws(()=>hostBundleInputs(root).capture(path.join(root,'large.mjs')),{code:'invalidHostBundleInput'});}finally{fs.rmSync(root,{recursive:true});}
});
test('locked compiler identity cannot silently change',()=>{
 const lock=JSON.stringify({packages:{'node_modules/esbuild':{version:'0.25.9'}}});checkHostBundleCompiler(lock,'0.25.9');for(const [value,version] of [[lock,'0.26.0'],['{}','0.25.9'],['invalid','0.25.9']])assert.throws(()=>checkHostBundleCompiler(value,version),{code:'hostBundleCompilerMismatch'});
});
test('missing changed or stale artifacts and receipts refuse without rewriting them',()=>{
 const root=fixture(),bytes=Buffer.from('exact bundle'),receipt='exact receipt\n';try{assert.throws(()=>checkHostBundleBytes(root,bytes,receipt),{code:'hostBundleMismatch'});fs.writeFileSync(path.join(root,'host.bundle.js'),bytes);fs.writeFileSync(path.join(root,'host-bundle.json'),receipt);checkHostBundleBytes(root,bytes,receipt);fs.writeFileSync(path.join(root,'host.bundle.js'),'changed');assert.throws(()=>checkHostBundleBytes(root,bytes,receipt),{code:'hostBundleMismatch'});assert.equal(fs.readFileSync(path.join(root,'host.bundle.js'),'utf8'),'changed');fs.writeFileSync(path.join(root,'host.bundle.js'),bytes);fs.writeFileSync(path.join(root,'host-bundle.json'),'stale');assert.throws(()=>checkHostBundleBytes(root,bytes,receipt),{code:'hostBundleMismatch'});}finally{fs.rmSync(root,{recursive:true});}
});
test('real staging comparison and runtime preservation remain mandatory with modeled build failures',()=>{
 const read=p=>fs.readFileSync(new URL('../../'+p,import.meta.url),'utf8'),builder=read('studio/tools/host_bundle.mjs'),stage=read('studio/scripts/stamp_shell.py'),workflow=read('.github/workflows/studio.yml'),projector=read('studio/core/src/bin/project_shell.rs'),runtime=read('studio/tools/host_bundle_runtime.mjs');
 for(const text of ['define:{\'import.meta.url\':\'__foundrySourceURL\'}','args.kind!==\'dynamic-import\'','result.outputFiles[0].contents.length>8*1024*1024','checkHostBundleBytes(shellRoot,output,receiptBytes)'])assert.ok(builder.includes(text),text);
 assert.ok(stage.includes('"host_bundle.mjs"), "--check"'));assert.ok(workflow.includes('node tools/host_bundle.mjs --check'));assert.ok(workflow.includes('node tools/host_bundle_runtime.mjs'));assert.ok(projector.includes('<script type="module" src="host.bundle.js"></script>'));
 for(const text of ['module URL identity','distinct directory reproduction','oversized output','dynamic import identity','canonical function source identity'])assert.ok(runtime.includes(text),text);
 const spec=read('app/src/FoundrySpecification.lex.tex');for(const code of ['invalidHostBundleInput','hostBundleMismatch','hostBundleCompilerMismatch'])assert.ok(spec.includes('name: '+code+','));
});
