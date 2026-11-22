import assert from 'node:assert/strict';
import fs from 'node:fs';
import {createHash} from 'node:crypto';
import {test} from 'node:test';
import {operateThemeLibrary,readThemeLibrary,createThemeLibrary} from '../shell/theme-library.mjs';
import {themesData} from '../shell/builtin-themes.mjs';
import {invokeHost} from '../shell/host-boundary.mjs';
const input={name:'Exact theme',prompt:'Instruction\n',description:'Details'};
const create=(state,value=input)=>operateThemeLibrary(state,'create',value,new Date('2026-01-01'));
test('theme CRUD preserves exact records Dates identity omitted fields and monotonic IDs',()=>{
 const a=create();assert.equal(a.result.id,1);assert.ok(a.result.createdAt instanceof Date);a.result.prompt='mutated';assert.equal(a.journal.records[0].prompt,input.prompt);
 const b=operateThemeLibrary(a.journal,'update',{id:1,name:'Changed'},new Date('2025-01-01'));assert.equal(b.result.name,'Changed');assert.equal(b.result.prompt,input.prompt);assert.deepEqual(b.result.createdAt,b.result.updatedAt);
 const c=operateThemeLibrary(b.journal,'delete',{id:1});assert.equal(c.result,undefined);assert.deepEqual(c.journal.records,[]);assert.equal(create(c.journal).result.id,2);
});
test('malformed commands and missing records refuse without changing state',()=>{
 const state=create().journal,original=structuredClone(state);
 for(const [kind,value,code] of [['create',{...input,name:''},'invalidThemeCommand'],['create',{...input,prompt:'é'.repeat(32769)},'invalidThemeCommand'],['update',{id:8},'themeMissing'],['delete',{id:8},'themeMissing'],['delete',1,'invalidThemeCommand'],['list',{},'invalidThemeCommand'],['update',{id:1,createdAt:new Date()},'invalidThemeCommand']])assert.throws(()=>operateThemeLibrary(state,kind,value),{code});
 assert.deepEqual(state,original);
});
test('corrupt journals never become empty results and limits refuse',()=>{
 const state=create().journal;
 for(const mutate of [s=>s.records[0].createdAt='date',s=>s.records[0].id=0,s=>s.nextId=1,s=>s.records.push({...s.records[0]}),s=>s.records[0].prompt=null,s=>s.extra=true,s=>s.records[0].updatedAt=new Date('2020-01-01')]){const bad=structuredClone(state);mutate(bad);assert.throws(()=>readThemeLibrary(bad),{code:'themeLibraryInvalid'});}
 assert.throws(()=>create({...state,nextId:Number.MAX_SAFE_INTEGER}),{code:'themeLibraryLimit'});
 assert.throws(()=>create({...state,nextId:1001,records:Array.from({length:1000},(_,i)=>({...state.records[0],id:i+1}))}),{code:'themeLibraryLimit'});
 let current;for(let i=0;i<63;i++)current=create(current,{name:'large',prompt:'x'.repeat(65536)}).journal;assert.throws(()=>create(current,{name:'large',prompt:'x'.repeat(65536)}),{code:'themeLibraryLimit'});
});
test('catalog returns detached imported data and rejects malformed or duplicate identity',async()=>{
 const read=relative=>fs.readFileSync(new URL(relative,import.meta.url)),hash=bytes=>createHash('sha256').update(bytes).digest('hex'),lock=JSON.parse(read('../BUILTIN_THEMES_LOCK.json'));
 assert.equal(lock.compiler,'0.25.9');assert.equal(hash(read('./builtin_themes_projection.mjs')),lock.generatorSha256);assert.equal(hash(read('../vendor/prism/package-lock.json')),lock.packageLockSha256);assert.equal(hash(read('../shell/builtin-themes.LICENSE.txt')),lock.licenseSha256);assert.equal(hash(read('../vendor/prism/LICENSE')),lock.licenseSha256);for(const input of lock.inputs)assert.equal(hash(read('../vendor/prism/'+input.path)),input.sha256);assert.equal(hash(read('../shell/'+lock.output.path)),lock.output.sha256);
 const call=createThemeLibrary(null,themesData),first=await call('catalog');assert.deepEqual(first,themesData);first[0].prompt='mutated';assert.deepEqual(await call('catalog'),themesData);
 for(const bad of [[],null,[themesData[0],themesData[0]],[{...themesData[0],id:'custom:1'}]])assert.throws(()=>createThemeLibrary(null,bad),{code:'themeCatalogMismatch'});
 await assert.rejects(()=>call('catalog',{}),{code:'invalidThemeCommand'});
});
test('validation precedes database access and storage errors hide private details',async()=>{
 let reads=0;const call=createThemeLibrary({transaction(){reads++;throw Error('private detail');}},themesData);
 await assert.rejects(()=>call('select',{appId:0,themeId:null}),{code:'invalidThemeCommand'});await assert.rejects(()=>call('create',{name:''}),{code:'invalidThemeCommand'});assert.equal(reads,0);
 await assert.rejects(()=>call('list'),{code:'themeStorageFailed',message:'themeStorageFailed'});
});
test('typed failures and actual host integration remain mandatory',async()=>{
 const host=fs.readFileSync(new URL('../shell/host.js',import.meta.url),'utf8');for(const channel of ['get-themes','get-custom-themes','create-custom-theme','update-custom-theme','delete-custom-theme','set-app-theme','get-app-theme'])assert.ok(host.includes('handlers.set("'+channel+'",'));
 assert.ok(host.includes('await themeLibrary("resolve",{appId:chat'));
 const spec=fs.readFileSync(new URL('../../app/src/FoundrySpecification.lex.tex',import.meta.url),'utf8');assert.ok(spec.includes('name: themeLibraryBoundary,'));
 for(const code of ['invalidThemeCommand','themeLibraryInvalid','themeMissing','themeAppMissing','themeLibraryLimit','themeStorageFailed','themeCatalogMismatch']){assert.ok(spec.includes('name: '+code+','));const result=await invokeHost('create-custom-theme',[],()=>{throw Object.assign(Error('private theme'),{code});});assert.equal(result.error.code,code);assert.ok(!JSON.stringify(result).includes('private theme'));}
 assert.ok(fs.readFileSync(new URL('../../.github/workflows/studio.yml',import.meta.url),'utf8').includes('node tools/durable_theme_library_browser.mjs'));
});
