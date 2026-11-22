import assert from 'node:assert/strict';
import fs from 'node:fs';
import {test} from 'node:test';
import {readLexLean,readDraftLexLean,draftOf,toLexLean,specFromTurn} from '../shell/live/lexlean.mjs';
import {SPEC_RULES,MODULE_EXAMPLE,moduleContext,shownText,withCard} from '../shell/prism.js';
import {assertGenerationActive} from '../shell/generation-patch.mjs';
const source=fs.readFileSync(new URL('./fixtures/live/verdant.lex.tex',import.meta.url),'utf8');
const host=()=>fs.readFileSync(new URL('../shell/host.js',import.meta.url),'utf8');
const refused=text=>assert.throws(()=>readDraftLexLean(text),{code:'invalidCanonicalModule',message:'The canonical decision module is invalid or unsupported'});
test('draft expression formatting reconstructs exactly through the strict canonical reader',()=>{
 const draft=draftOf(source);assert.notEqual(draft,source);assert.throws(()=>readLexLean(draft),{code:'invalidCanonicalModule'});assert.equal(toLexLean(readDraftLexLean(draft)),source);assert.equal(toLexLean(readDraftLexLean(source)),source);
 const example=MODULE_EXAMPLE;const result=specFromTurn('<prism-lexlean>'+example+'</prism-lexlean>');assert.equal(result.name,'ParkingFee');assert.deepEqual(readLexLean(result.source),result.spec);assert.ok(moduleContext(result.source).includes(draftOf(result.source).split('theorem{')[0].trim().slice(0,80)));
});
test('draft reconstruction cannot erase unknown declarations constraints fields or resource limits',()=>{
 const draft=draftOf(source);
 for(const text of [draft.replace('\\end{semanticmodule}','definition{name: hiddenDecision, parameters: [], result: Nat, body: 99}\n\\end{semanticmodule}'),draft.replace('{name: biome, type: Nat}','{name: biome, type: Nat}, {name: hidden, type: Bool}'),draft+'extra-program',draft.replace('name: inBounds,','name: unsupportedBounds,'),draft.replace('parameters: []','parameters: [], axioms: [private]'),'x'.repeat(1024*1024+1),'('.repeat(257)+draft+')'.repeat(257)])refused(text);
});
test('whole draft framing is tolerated but incomplete modules and extra declarations never admit',()=>{
 const draft=draftOf(source),end=draft.indexOf('\\end{semanticmodule}');assert.equal(specFromTurn('<prism-lexlean>'+draft.slice(0,end)),null);
 for(const tail of ['', '\\end{lexlean}','\\end{prism-lexlean}'])assert.equal(specFromTurn('<prism-lexlean>'+draft.slice(0,end)+'\\end{semanticmodule}'+tail).source,source);
 assert.equal(specFromTurn('<prism-lexlean>'+draft+'definition{name: hidden, parameters: [], result: Nat, body: 7}</prism-lexlean>').code,'invalidCanonicalModule');
});
test('chat projection hides complete and streaming module text while retaining surrounding prose',async()=>{
 const text='Before\n<prism-lexlean>'+source+'</prism-lexlean>\n<prism-spec-patch>[]</prism-spec-patch>\nAfter';const result=withCard(text,'Accepted module');assert.ok(result.includes('Before')&&result.includes('After')&&result.includes('Accepted module'));assert.ok(!result.includes('definition{')&&!result.includes('prism-spec-patch'));
 for(let i=text.indexOf('<prism-lexlean>')+15;i<text.indexOf('</prism-lexlean>');i+=37)assert.ok(!shownText(text.slice(0,i)).includes('definition{'));
 assert.equal(shownText('<think>Use <prism-lexlean> eventually'),'<think>Use <prism-lexlean> eventually');
 const h=host(),a=h.indexOf('    let textWork = Promise.resolve();'),b=h.indexOf("    // Foundry's own language model reads the idea",a);assert.ok(a>0&&b>a);
 const make=new Function('onText',h.slice(a,b)+'return {enqueueText};');
 let release;const order=[],held=new Promise(resolve=>release=resolve);
 const stream=make(async text=>{order.push('start:'+text);if(text==='partial')await held;order.push('saved:'+text);});
 stream.enqueueText('partial');const final=stream.enqueueText('final');await Promise.resolve();assert.deepEqual(order,['start:partial']);release();await final;assert.deepEqual(order,['start:partial','saved:partial','start:final','saved:final']);

});
function patchRunner(){const text=host(),a=text.indexOf('    const patchRefusals = [];'),b=text.indexOf('    if (refusedCode)',a);assert.ok(a>0&&b>a);return new (Object.getPrototypeOf(async function(){}).constructor)('full','spec','patchSpec','abort','assertGenerationActive','appId','onCommit',text.slice(a,b)+'return spec;');}
test('actual per-operation retry retains source transitions and rejects malformed path reporting safely',async()=>{
 const run=patchRunner(),calls=[],spec={source:'s0',changed:0,card:''},ops=[null,{op:'set',path:[{toString:null}]},{op:'set',path:['title'],value:'accepted'}];
 const apply=async(id,list,options)=>{calls.push(options.source);return list.length===1&&list[0]?.value==='accepted'?{ok:true,changed:1,source:'s1',card:'accepted'}:{ok:false,issues:['invalid operation']};};
 await run('<prism-spec-patch>'+JSON.stringify(ops)+'</prism-spec-patch><prism-spec-patch>'+JSON.stringify([ops[2]])+'</prism-spec-patch>',spec,apply,new AbortController(),assertGenerationActive,1,()=>{});assert.deepEqual(calls,['s0','s0','s0','s0','s1']);assert.equal(spec.source,'s1');assert.equal(spec.changed,2);assert.ok(spec.card.includes('Patch refused')&&spec.card.includes('invalid operation'));
});
test('cancellation between accepted live operations stops retries without erasing the accepted source',async()=>{
 const run=patchRunner(),controller=new AbortController(),spec={source:'s0',changed:0,card:''};let calls=0;
 const apply=async()=>{calls++;if(calls===1)return {ok:false,issues:['batch refused']};controller.abort();return {ok:true,changed:1,source:'s1',card:'accepted'};};
 await assert.rejects(()=>run('<prism-spec-patch>[{"op":"set"},{"op":"set"}]</prism-spec-patch>',spec,apply,controller,assertGenerationActive,1,()=>{}),{code:'generationCancelled'});assert.equal(calls,2);assert.equal(spec.source,'s1');assert.equal(spec.changed,1);
});
test('a chat turn cannot apply model-written files',()=>{
 const text=host();assert.ok(text.includes('writes: [], deletes: [], renames: [], deps: []'));assert.ok(!text.includes('buildSystemPrompt('));assert.ok(text.lastIndexOf('if (refusedCode)')>text.indexOf('    for (const match of full.matchAll(/<prism-spec-patch>'));assert.ok(text.includes('await enqueueText((notationLines.length'));assert.ok(text.includes('const history = chat ? await messagesOf(chatId) : [];'));
 const workflow=fs.readFileSync(new URL('../../.github/workflows/forge-perf.yml',import.meta.url),'utf8');assert.ok(workflow.includes('${GITHUB_WORKSPACE}/studio/vendor/prism/node_modules/playwright-core/index.mjs'));assert.ok(!workflow.includes('npm root --global'));
 assert.ok(fs.statSync(new URL('../scripts/live_e2e_all.sh',import.meta.url)).mode & 0o111,'mandatory every-app replay must be executable');
 assert.ok(fs.readFileSync(new URL('../../app/src/FoundrySpecification.lex.tex',import.meta.url),'utf8').includes('name: liveMainIntegrationBoundary,'));assert.ok(fs.readFileSync(new URL('../../.github/workflows/studio.yml',import.meta.url),'utf8').includes('node tools/live_main_integration_browser.mjs'));
});
