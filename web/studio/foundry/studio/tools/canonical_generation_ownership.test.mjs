import assert from "node:assert/strict";
import { test } from "node:test";
import fs from "node:fs";
import vm from "node:vm";
import * as lexlean from "../shell/live/lexlean.mjs";
import { copyGenerationInput, assertCanonicalGeneration } from "../shell/canonical-generation.mjs";
import { assertGenerationActive } from "../shell/generation-patch.mjs";
import { invokeHost } from "../shell/host-boundary.mjs";
const spec = JSON.parse(fs.readFileSync(new URL("./fixtures/live/rsvp.json", import.meta.url)));
const path = "prism/" + lexlean.canonicalSource(spec).name + ".lex.tex";
const source = {name:"app", reference:"snapshot", files:[["prism/spec.json",JSON.stringify(spec)],[path,"module"],["src/lib/prism.ts","runtime"],["other","ordinary"]]};
const patch = extra => ({writes:[],deletes:[],renames:[],deps:[],...extra});
const guard = (s,p) => assertCanonicalGeneration(s,p,lexlean);
const host = fs.readFileSync(new URL("../shell/host.js", import.meta.url), "utf8");
function adapter(env) { const start = host.indexOf("async function applyPrismTags("), end=host.indexOf("window.__chat =",start); return vm.runInNewContext(host.slice(start,end)+"\napplyPrismTags",{structuredClone, copyGenerationInput, assertCanonicalGeneration, assertGenerationActive, lexlean, ...env}); }
test("writes and deletes cannot touch any canonical owned path", () => {
  for (const file of ["prism/spec.json",path,"src/lib/prism.ts"]) for (const p of [patch({writes:[{path:"ordinary",content:"safe"},{path:file,content:"new"}]}),patch({deletes:[{path:file}]})]) assert.throws(()=>guard(source,p),e=>e.code==="canonicalGenerationRefused");
});
test("both rename endpoints and malformed-model modules remain protected", () => {
  for (const file of ["prism/spec.json",path,"src/lib/prism.ts"]) for (const item of [{from:file,to:"ordinary"},{from:"other",to:file}]) assert.throws(()=>guard(source,patch({renames:[item]})),e=>e.code==="canonicalGenerationRefused");
  const broken={...source,files:[["prism/spec.json","invalid"]]};
  assert.throws(()=>guard(broken,patch({writes:[{path:"prism/Unknown.lex.tex",content:"new"}]})),e=>e.code==="canonicalGenerationRefused");
});
test("ordinary batches and non-model projects retain their existing path", () => {
  assert.doesNotThrow(()=>guard(source,patch({writes:[{path:"src/pages/Index.tsx",content:"page"},{path:"prism/Inactive.lex.tex",content:"inactive"}]})));
  assert.doesNotThrow(()=>guard({files:[]},patch({writes:[{path:"src/lib/prism.ts",content:"ordinary code"}]})));
  for (const p of [null,{},patch({writes:[null]}),patch({writes:Array.from({length:1001},()=>({path:"x",content:"x"}))})]) assert.throws(()=>guard(source,p),e=>e.code==="invalidGenerationPatch");
});
test("patch and source snapshots are copied before any await", async () => {
  let release; const pending=new Promise(r=>{release=r}); let called;
  const input=patch({writes:[{path:"other",content:"candidate"}]}), supplied=structuredClone(source);
  const run=adapter({coreReady:pending,dbReady:Promise.resolve({}),core:{},kappa(){},captureProjectSource:()=>assert.fail("supplied source discarded"),applyStoredProjectPatch:async(_,id,p,b)=>{called={p,b};return {changed:1};}});
  const result=run(1,input,{source:supplied}); input.writes[0].content="mutated"; supplied.files.length=0; release(); await result;
  assert.equal(called.p.writes[0].content,"candidate"); assert.equal(called.b.source.files.length,source.files.length);
  assert.throws(()=>copyGenerationInput({fn(){}},source),e=>e.code==="invalidGenerationPatch");
});
test("the actual host refuses before effects and forwards no lifecycle override", async () => {
  let writes=0; const core={real:true}, kappa=()=>{};
  const run=adapter({coreReady:Promise.resolve(),dbReady:Promise.resolve({}),core,kappa,captureProjectSource:async()=>source,applyStoredProjectPatch:async(_,id,p,b)=>{writes++;assert.equal(b.core,core);assert.equal(b.derive,kappa);assert.equal(b.rulesUpdate,undefined);return {changed:1};}});
  await assert.rejects(run(1,patch({writes:[{path,content:"changed"}]}),{allowCanonical:true,rulesUpdate:{state:{phase:"proved"}}}),e=>e.code==="canonicalGenerationRefused"); assert.equal(writes,0);
  await run(1,patch({writes:[{path:"other",content:"normal"}]}),{rulesUpdate:{state:{phase:"proved"}},core:{fake:true},derive:()=>"fake"}); assert.equal(writes,1);
  const cancelled=new AbortController();cancelled.abort();await assert.rejects(run(1,patch({}),{signal:cancelled.signal}),e=>e.code==="generationCancelled");assert.equal(writes,1);
});
test("owned-path refusals remain fixed typed public errors", async () => {
  const result=await invokeHost("generation",[],()=>{throw Object.assign(Error("private module"),{code:"canonicalGenerationRefused"});},null);
  assert.equal(result.error.code,"canonicalGenerationRefused");assert.equal(result.error.kind,"validation");assert.ok(!JSON.stringify(result).includes("private module"));
  const model=fs.readFileSync(new URL("../../app/src/FoundrySpecification.lex.tex",import.meta.url),"utf8");assert.match(model,/name: canonicalGenerationBoundary/);
});
