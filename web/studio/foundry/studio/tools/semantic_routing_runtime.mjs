// Independent qualification of the installed signed guest; this does not compile a substitute.
import assert from 'node:assert/strict';
import fs from 'node:fs';
import vm from 'node:vm';
import {webcrypto} from 'node:crypto';
import {performance} from 'node:perf_hooks';
import {routeWithGuest} from '../shell/routing-codec.mjs';
const self={};vm.runInNewContext(fs.readFileSync(new URL('../shell/kernel.js',import.meta.url),'utf8'),{self,crypto:webcrypto,WebAssembly,TextEncoder,TextDecoder,Uint8Array,atob});
const guest=await self.ForgeKernel.load(async path=>new Response(fs.readFileSync(new URL('../shell/'+path,import.meta.url))));
assert.equal(guest.vectors,JSON.parse(fs.readFileSync(new URL('../shell/kernel/kernel.json',import.meta.url))).vectors.length,'Install the signed routing-capable guest');
const corpus=JSON.parse(fs.readFileSync(new URL('../../verification/routing-vectors.json',import.meta.url))),qualification=JSON.parse(fs.readFileSync(new URL('../../verification/routing-qualification.json',import.meta.url)));
assert.equal(corpus.vectors.length,32);assert.equal(qualification.cases.length,qualification.thresholds.exactJudgments);
let maximumMs=0;
for(const row of corpus.vectors){const started=performance.now(),response=await guest.dispatch(new TextEncoder().encode(row.request));maximumMs=Math.max(maximumMs,performance.now()-started);assert.equal(new TextDecoder().decode(response),row.response,row.name);}
const timings=[];let judgments=0;
for(const row of qualification.cases){const answer=await routeWithGuest(guest,row.input);assert.equal(answer.ok,row.expected.ok,row.name);assert.equal(answer.code||'accepted',row.expected.code,row.name);assert.deepEqual(answer.proposal?.selected.map(r=>r.key)||[],row.expected.selected,row.name);judgments++;}
await routeWithGuest(guest,qualification.cases[0].input);
for(let i=0;i<qualification.thresholds.trials;i++)for(const row of qualification.cases){const started=performance.now();await routeWithGuest(guest,row.input);timings.push(performance.now()-started);}
timings.sort((a,b)=>a-b);const p95=timings[Math.ceil(timings.length*.95)-1];assert.ok(p95<=qualification.thresholds.normalP95Milliseconds,`normal p95 ${p95} ms exceeds fixed threshold`);assert.ok(maximumMs<=qualification.thresholds.maximumCaseMilliseconds,`maximum corpus case ${maximumMs} ms exceeds fixed threshold`);
console.log(JSON.stringify({embedded:guest.vectors,wireVectors:corpus.vectors.length,typedJudgments:judgments,falseAdmissions:0,dependencyMismatches:0,normalP95Milliseconds:p95,maximumCaseMilliseconds:maximumMs,scope:qualification.scope}));
