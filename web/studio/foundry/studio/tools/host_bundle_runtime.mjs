// Actual locked compiler acceptance. Studio installs the renderer lock before this gate.
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import {pathToFileURL} from 'node:url';
import {projectHostBundle} from './host_bundle.mjs';
const root=fs.mkdtempSync(path.join(os.tmpdir(),'foundry-host-runtime-'));
try{
 const a=path.join(root,'a'),b=path.join(root,'b');
 for(const dir of [a,b]){
  fs.mkdirSync(path.join(dir,'nested'),{recursive:true});
  fs.writeFileSync(path.join(dir,'package.json'),' {"type":"module"} ');
  fs.writeFileSync(path.join(dir,'host.js'),"export {source,asset,load} from './nested/module.mjs'; export {INTERPRETER_PARTS} from './prism.js';\n");
  fs.writeFileSync(path.join(dir,'prism.js'),"function prismAnswer(value) { return value + 1; }\nfunction prismDecide(value) { return prismAnswer(value); }\nexport const INTERPRETER_PARTS = Object.freeze([prismAnswer.toString(), prismDecide.toString()]);\n");
  fs.writeFileSync(path.join(dir,'nested/module.mjs'),"export const source=import.meta.url;export const asset=new URL('./asset.json',import.meta.url).href;export const load=()=>import('./lazy.mjs');\n");
  fs.writeFileSync(path.join(dir,'nested/lazy.mjs'),"export const source=import.meta.url;\n");
 }
 const first=await projectHostBundle({shellRoot:a}),second=await projectHostBundle({shellRoot:b});
 assert.deepEqual(first,second,'distinct directory reproduction receipt');assert.ok(fs.readFileSync(path.join(a,'host.bundle.js')).equals(fs.readFileSync(path.join(b,'host.bundle.js'))),'distinct directory reproduction bytes');
 const loaded=await import(pathToFileURL(path.join(a,'host.bundle.js')));assert.equal(loaded.source,pathToFileURL(path.join(a,'nested/module.mjs')).href,'module URL identity');assert.equal(loaded.asset,pathToFileURL(path.join(a,'nested/asset.json')).href);assert.equal((await loaded.load()).source,pathToFileURL(path.join(a,'nested/lazy.mjs')).href,'dynamic import identity');
 const original=await import(pathToFileURL(path.join(a,'prism.js')));assert.deepEqual(loaded.INTERPRETER_PARTS,original.INTERPRETER_PARTS,'canonical function source identity');
 await projectHostBundle({shellRoot:a,check:true});
 fs.appendFileSync(path.join(a,'nested/module.mjs'),'export const changed=1;');await assert.rejects(()=>projectHostBundle({shellRoot:a,check:true}),{code:'hostBundleMismatch'});
 fs.writeFileSync(path.join(a,'host.js'),"export const large='"+'x'.repeat(8*1024*1024+1)+"';");await assert.rejects(()=>projectHostBundle({shellRoot:a}),{code:'invalidHostBundleInput'},'oversized output');
 console.log(JSON.stringify({lockedCompiler:true,reproducedBytes:true,reproducedReceipt:true,moduleUrlIdentity:true,canonicalFunctionSourceIdentity:true,dynamicImportIdentity:true,staleSourceRefused:true,oversizedOutputRefused:true}));
}finally{fs.rmSync(root,{recursive:true});}
