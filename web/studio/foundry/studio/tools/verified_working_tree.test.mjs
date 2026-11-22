import assert from 'node:assert/strict';
import fs from 'node:fs';
import {test} from 'node:test';
import {compareWorkingFiles,lineChanges,readWorkingTree} from '../shell/working-tree.mjs';
import {blake3} from '../../try/vendor/prism-code/src/blake3.mjs';
const enc=new TextEncoder(),derive=async bytes=>'blake3:'+Buffer.from(blake3(bytes)).toString('hex');
async function fixture(){const text=enc.encode('before\n'),file=await derive(text),bytes=enc.encode(JSON.stringify({name:'app',parent:'',entries:[['a.txt',file,text.length]]})),reference=await derive(bytes),objects=new Map([[file,text],[reference,bytes]]);return {reference,file,objects,capture:async()=>({reference,files:[['a.txt','after\n']]}),boundary:{derive,readObject:async key=>objects.get(key),admit:async()=>true}};}
test('working tree distinguishes additions deletions modifications and empty file presence',()=>{
 assert.deepEqual(compareWorkingFiles([['modify','a\nb\n'],['empty','']],[['modify','a\nx\n'],['gone','old']]),[{path:'empty',status:'added',oldContent:'',newContent:'',additions:0,deletions:0},{path:'gone',status:'deleted',oldContent:'old',newContent:'',additions:0,deletions:1},{path:'modify',status:'modified',oldContent:'a\nx\n',newContent:'a\nb\n',additions:1,deletions:1}]);assert.deepEqual(compareWorkingFiles([['same','unchanged']],[['same','unchanged']]),[]);
});
test('line counts retain separated common lines and reject exhausted comparison budgets',()=>{
 assert.deepEqual(lineChanges('x\nkeep\ny\n','z\nkeep\nw\n'),{additions:2,deletions:2});assert.deepEqual(lineChanges('','x\n'),{additions:1,deletions:0});assert.deepEqual(lineChanges('x','x\n'),{additions:1,deletions:1});
 assert.throws(()=>lineChanges('a\nb\n','c\nd\n',{remaining:3}),{code:'workingTreeComparisonLimit'});
 assert.throws(()=>lineChanges('', 'x\n'.repeat(100001)),{code:'workingTreeComparisonLimit'});
 assert.throws(()=>compareWorkingFiles([['huge','x'.repeat(48*1024*1024+1)]],[]),{code:'workingTreeComparisonLimit'});
});
test('invalid paths duplicate files malformed text and resource overflow cannot become clean state',()=>{
 for(const files of [[['../escape','x']],[['a','x'],['a','y']],[['a',null]],[['a\\b','x']],Array.from({length:1001},(_,i)=>[String(i),''])])assert.throws(()=>compareWorkingFiles(files,[]),{code:'workingTreeStateInvalid'});
});
test('working queries capture caller input and verify actual snapshot and file identities',async()=>{
 const f=await fixture(),input={appId:1,filePath:'a.txt'},pending=readWorkingTree(null,input,f.boundary,{capture:f.capture});input.filePath='../changed';assert.deepEqual(await pending,{path:'a.txt',oldContent:'before\n',newContent:'after\n'});
 assert.deepEqual(await readWorkingTree(null,{appId:1},f.boundary,{capture:f.capture}),[{path:'a.txt',status:'modified',additions:1,deletions:1}]);
 for(const input of [{appId:0},{appId:1,filePath:'../a'},{appId:1,extra:1}])await assert.rejects(()=>readWorkingTree(null,input,f.boundary,{capture:()=>assert.fail('invalid input captured')}),{code:'invalidWorkingTreeRequest'});
});
test('missing corrupt and core-refused basis never becomes empty successful diff',async()=>{
 for(const mode of ['missing','snapshot','file','admission']){const f=await fixture();let code;
  if(mode==='missing'){f.objects.delete(f.reference);code='missingSnapshotObject';}if(mode==='snapshot'){f.objects.set(f.reference,enc.encode('{}'));code='snapshotIdentityMismatch';}if(mode==='file'){f.objects.set(f.file,enc.encode('bad'));code='fileIdentityMismatch';}if(mode==='admission'){f.boundary.admit=async()=>false;code='snapshotCoreRefused';}
  await assert.rejects(()=>readWorkingTree(null,{appId:1},f.boundary,{capture:f.capture}),{code});
 }
 await assert.rejects(()=>readWorkingTree(null,{appId:1},{},{capture:async()=>{throw Object.assign(Error(),{code:'projectEditStorageFailed'});}}),{code:'projectEditStorageFailed'});
});
test('actual host adapters and mandatory browser acceptance bind both queries to verified capture',()=>{
 const host=fs.readFileSync(new URL('../shell/host.js',import.meta.url),'utf8');for(const channel of ['git:get-uncommitted-files','git:get-uncommitted-file-diff'])assert.ok(host.includes('handlers.set("'+channel+'", async input => readWorkingTree('));
 assert.ok(fs.readFileSync(new URL('../../app/src/FoundrySpecification.lex.tex',import.meta.url),'utf8').includes('name: workingTreeBoundary,'));assert.ok(fs.readFileSync(new URL('../../.github/workflows/studio.yml',import.meta.url),'utf8').includes('node tools/verified_working_tree_browser.mjs'));
});
