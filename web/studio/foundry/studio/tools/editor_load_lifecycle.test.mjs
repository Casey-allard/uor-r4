import assert from 'node:assert/strict';
import {test} from 'node:test';
import fs from 'node:fs';
import vm from 'node:vm';
import {spawnSync} from 'node:child_process';
import {LEAN} from '../vendor/prism/hologram.lean.mjs';
function projected(){let text=fs.readFileSync(new URL('../vendor/prism/src/components/chat/monaco.ts',import.meta.url),'utf8');for(const patch of LEAN.filter(x=>x.file==='src/components/chat/monaco.ts')){assert.ok(text.includes(patch.from));text=text.replace(patch.from,patch.to);}return text.slice(text.indexOf('if (!IS_VITEST)'));}
function exercise(scenario){
 const script=`const assert=require('node:assert/strict'),vm=require('node:vm');
 const unhandled=[];process.on('unhandledRejection',reason=>unhandled.push(reason));
 const tick=()=>new Promise(resolve=>setTimeout(resolve,10));
 const cancellation={type:'cancelation',msg:'operation is manually canceled'};
 let resolve,reject,calls=0,configured=0,failTheme=false;
 let base=new Promise((yes,no)=>{resolve=yes;reject=no;});
 const monaco={editor:{defineTheme(){if(failTheme)throw Error('private configuration');configured++;}},languages:{typescript:{JsxEmit:{React:2},typescriptDefaults:{setCompilerOptions(){},setDiagnosticsOptions(){}}}}};
 const loader={init(){calls++;let cancelled=false;const ready=base.then(value=>{if(cancelled)throw cancellation;return value;});ready.cancel=()=>{cancelled=true;};return ready;}};
 vm.runInNewContext(${JSON.stringify(projected())},{loader,IS_VITEST:false,customLight:{},customDark:{}});
 (async()=>{${scenario}\nawait tick();assert.deepEqual(unhandled,[]);})().catch(e=>{console.error(e);process.exitCode=1;});`;
 const result=spawnSync(process.execPath,['-e',script],{encoding:'utf8',timeout:10000});assert.equal(result.status,0,result.stdout+result.stderr);
}
test('actual renderer projection is lazy and shares complete configuration across concurrent callers',()=>exercise(`assert.equal(calls,0);const one=loader.init(),two=loader.init();assert.equal(configured,0);resolve(monaco);assert.equal(await one,monaco);assert.equal(await two,monaco);assert.equal(configured,2);assert.equal(await loader.init(),monaco);assert.equal(configured,2);`));
test('cancelling the first mount cannot detach a rejection or strand replacement configuration',()=>exercise(`const one=loader.init();const observed=one.catch(e=>e);one.cancel();const two=loader.init();resolve(monaco);assert.equal(await observed,cancellation);assert.equal(await two,monaco);assert.equal(configured,2);`));
test('cancellation after cached loading affects only that callers owned result',()=>exercise(`resolve(monaco);await tick();const one=loader.init(),two=loader.init();const observed=one.catch(e=>e);one.cancel();assert.equal(await observed,cancellation);assert.equal(await two,monaco);assert.equal(configured,2);`));
test('loader failure is typed private and settles every owner without detached rejection',()=>exercise(`const one=loader.init(),two=loader.init();const a=one.catch(e=>e),b=two.catch(e=>e);reject(Error('private loading'));for(const result of [await a,await b]){assert.equal(result.code,'editorLoadFailed');assert.ok(!result.message.includes('private'));}assert.equal(configured,0);`));
test('configuration failure is typed and does not set successful readiness before retry',()=>exercise(`failTheme=true;const first=loader.init();resolve(monaco);const error=await first.catch(e=>e);assert.equal(error.code,'editorThemeFailed');assert.ok(!error.message.includes('private'));assert.equal(configured,0);failTheme=false;assert.equal(await loader.init(),monaco);assert.equal(configured,2);`));
test('canonical lifecycle and mandatory actual browser gates remain registered',()=>{
 const read=p=>fs.readFileSync(new URL('../../'+p,import.meta.url),'utf8');assert.ok(read('app/src/FoundrySpecification.lex.tex').includes('name: editorLoadLifecycleBoundary,'));assert.ok(read('.github/workflows/studio.yml').includes('node tools/editor_load_lifecycle_browser.mjs'));assert.ok(read('.github/workflows/studio.yml').includes('node tools/editor_browser.mjs'));assert.ok(projected().includes('editorLoadFailed'));assert.ok(projected().includes('editorThemeFailed'));
});

test('locked loader source projection refuses drift and owns underlying promise rejection',async()=>{
 const {editorLoaderSource}=await import('../vendor/prism/hologram.editor-loader.mjs');const id='/locked/node_modules/@monaco-editor/loader/lib/es/utils/makeCancelable.js',source=fs.readFileSync(new URL('./fixtures/editor-loader/makeCancelable.mjs',import.meta.url),'utf8');
 assert.equal(editorLoaderSource('/unrelated.js',source),null);for(const changed of [source+'\n',source.replace('promise.then','promise.finally'),''])assert.throws(()=>editorLoaderSource(id,changed),{code:'editorLoaderProjectionMismatch'});
 const code=editorLoaderSource(id,source);assert.ok(code.includes('}, reject);'));assert.ok(!code.includes('promise["catch"](reject)'));
 const result=spawnSync(process.execPath,['--input-type=module','-e',`import assert from 'node:assert/strict';const errors=[];process.on('unhandledRejection',e=>errors.push(e));const {default:wrap}=await import('data:text/javascript;base64,${Buffer.from(code).toString('base64')}');const failure=Error('private');assert.equal(await wrap(Promise.reject(failure)).catch(e=>e),failure);const pending=wrap(Promise.resolve(3));pending.cancel();assert.equal((await pending.catch(e=>e)).type,'cancelation');await new Promise(r=>setTimeout(r,10));assert.deepEqual(errors,[]);`],{encoding:'utf8',timeout:10000});assert.equal(result.status,0,result.stdout+result.stderr);
 const vite=fs.readFileSync(new URL('../vendor/prism/vite.shell.config.mts',import.meta.url),'utf8');assert.ok(vite.includes('editorLoaderSource(file, code)'));assert.ok(vite.includes('if (!projectedEditorLoaders.size)'));
});
