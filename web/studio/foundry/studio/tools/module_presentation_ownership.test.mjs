import assert from 'node:assert/strict';
import {test} from 'node:test';
import fs from 'node:fs';
import {modulePresentation} from '../shell/module-presentation.mjs';
import {canonicalSource,readLexLean} from '../shell/live/lexlean.mjs';
const load=p=>{const v=JSON.parse(fs.readFileSync(new URL(p,import.meta.url)));return v.spec??v;};
const prior=load('./fixtures/live/rsvp.json'),parking=load('./fixtures/specs/parking-fee.json');
const moduleOf=s=>readLexLean(canonicalSource(s).source);
test('cross-profile module replaces incompatible presentation without changing admitted logic',()=>{
 const incoming=moduleOf(parking),result=modulePresentation(prior,incoming);
 assert.equal(result.reset,true);assert.deepEqual(result.spec,incoming);assert.notEqual(result.spec.purpose,prior.purpose);assert.equal(canonicalSource(result.spec).source,canonicalSource(incoming).source);incoming.name="changed";assert.equal(result.spec.name,parking.name);
});
test('compatible logic edit preserves detached presentation without copying prior rules',()=>{
 const original=structuredClone(prior),next=moduleOf(prior);next.rules[0].then=next.outcomes.find(x=>x!==next.rules[0].then);
 const expected=structuredClone(next),result=modulePresentation(original,next);assert.equal(result.reset,false);
 for(const key of ['purpose','view','meanings'])assert.deepEqual(result.spec[key],original[key]);
 assert.deepEqual(result.spec.rules,expected.rules);assert.equal(canonicalSource(result.spec).source,canonicalSource(expected).source);
 original.view.blocks[0].text='changed';next.rules[0].then='changed';assert.notEqual(result.spec.view.blocks[0].text,'changed');assert.notEqual(result.spec.rules[0].then,'changed');
});
test('name field bounds choices and outcome identity each prevent accidental retention',()=>{
 const changes=[s=>s.name='renamed',s=>s.fields.find(f=>f.kind==='number').max++,s=>s.fields.find(f=>f.kind==='choice').values.reverse(),s=>s.outcomes.reverse()];
 for(const change of changes){const incoming=moduleOf(prior);change(incoming);assert.equal(modulePresentation(prior,incoming).reset,true);}
 for(const bad of [{}, {...prior,view:{blocks:[{kind:'input',field:'unknown'}]}},{...prior,meanings:{missing:'unbound'}}]){const incoming=moduleOf(prior),result=modulePresentation(bad,incoming);assert.equal(result.reset,true);assert.deepEqual(result.spec,incoming);}
 assert.deepEqual(modulePresentation(null,moduleOf(parking)),{spec:moduleOf(parking),reset:false});
});
test('invalid incoming module refuses with fixed failure and actual host uses the mandatory boundary',()=>{
 for(const value of [{},null,{...parking,outcomes:[]}])assert.deepEqual(modulePresentation(prior,value),{code:'invalidCanonicalModule',error:'The canonical decision module is invalid or unsupported'});
 const read=p=>fs.readFileSync(new URL(p,import.meta.url),'utf8'),host=read('../shell/host.js');assert.ok(host.includes('modulePresentation(previous, lex.spec)'));assert.ok(host.includes('presentationReset ? ["The application interface changed'));assert.ok(!host.includes('JSON.stringify({ ...lex.spec, ...words })'));assert.ok(read('../../.github/workflows/studio.yml').includes('node tools/module_presentation_browser.mjs'));
});
