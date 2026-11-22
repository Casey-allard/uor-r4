import assert from 'node:assert/strict';
import {test} from 'node:test';
import fs from 'node:fs';
import {modulePresentation} from '../shell/module-presentation.mjs';
import {canonicalSource,readLexLean} from '../shell/live/lexlean.mjs';
import {validateSpec} from '../shell/prism/spec.mjs';
const original=fs.readFileSync(new URL('./fixtures/live/verdant.lex.tex',import.meta.url),'utf8');
const large=()=>readLexLean(original),failure={code:'invalidCanonicalModule',error:'The canonical decision module is invalid or unsupported'};
test('large canonical modules preserve every rule and byte without changing the proof profile',()=>{
 const spec=large(),before=structuredClone(spec);assert.ok(spec.rules.length>24);
 const result=modulePresentation(null,spec);assert.equal(result.reset,false);assert.deepEqual(result.spec,before);assert.equal(canonicalSource(result.spec).source,canonicalSource(before).source);
 assert.throws(()=>validateSpec(spec),/at most 24 rules/);spec.rules.at(-1).then='caller-change';assert.deepEqual(result.spec,before);
});
test('compatible large edits keep only detached presentation and exact incoming logic',()=>{
 const previous=large();previous.purpose='My garden';previous.view={blocks:[{kind:'heading',text:'My garden'}]};previous.meanings={[previous.outcomes[0]]:'A quiet garden'};
 const next=large();next.rules.at(-1).then=next.outcomes.find(x=>x!==next.rules.at(-1).then);const expected=structuredClone(next);
 const result=modulePresentation(previous,next);assert.equal(result.reset,false);
 for(const key of ['purpose','view','meanings'])assert.deepEqual(result.spec[key],previous[key]);
 assert.deepEqual(result.spec.rules,expected.rules);assert.equal(canonicalSource(result.spec).source,canonicalSource(expected).source);
 previous.view.blocks[0].text='caller-change';next.rules.at(-1).then='caller-change';assert.equal(result.spec.view.blocks[0].text,'My garden');assert.deepEqual(result.spec.rules,expected.rules);
});
test('invalid tail rules properties examples and presentation refuse without partial admission',()=>{
 const invalid=[s=>s.rules.at(-1).then='unknown',s=>s.rules[24].when[0].field='unknown',s=>s.rules.at(-1).when=[],s=>s.properties.push({kind:'never',when:[{field:'unknown',op:'=',value:0}],then:s.outcomes[0]}),s=>s.examples.push({request:'example',response:'unknown'}),s=>s.meanings={unknown:'bad'},s=>s.view={blocks:[{kind:'input',field:'unknown'}]}];
 for(const change of invalid){const s=large();change(s);const before=structuredClone(s);assert.deepEqual(modulePresentation(null,s),failure);assert.deepEqual(s,before);}
 for(const change of invalid){const prior=large(),incoming=large();change(prior);const result=modulePresentation(prior,incoming);assert.equal(result.reset,true);assert.deepEqual(result.spec,incoming);}
});
test('schema checks span multiple batches and canonical source browser remains mandatory',()=>{
 const s=large();while(s.rules.length<73)s.rules.push(structuredClone(s.rules[0]));
 assert.deepEqual(modulePresentation(null,s).spec,s);for(const index of [0,23,24,47,48,71,72]){const bad=structuredClone(s);bad.rules[index].then='unknown';assert.deepEqual(modulePresentation(null,bad),failure,'invalid rule '+index);}
 const root=new URL('../../',import.meta.url);assert.match(fs.readFileSync(new URL('app/src/FoundrySpecification.lex.tex',root),'utf8'),/name: largeModulePresentationBoundary/);
 assert.ok(fs.readFileSync(new URL('.github/workflows/studio.yml',root),'utf8').includes('node tools/large_module_presentation_browser.mjs'));
});
