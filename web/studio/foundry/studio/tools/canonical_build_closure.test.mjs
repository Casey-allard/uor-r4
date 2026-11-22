import assert from "node:assert/strict";
import fs from "node:fs";
import { test } from "node:test";
import { assertCanonicalBuild } from "../shell/canonical-build.mjs";
import { verifyCapturedSource } from "../shell/build-source.mjs";
import { checkSpec } from "../shell/prism.js";
import { canonicalSource } from "../shell/live/lexlean.mjs";
import { invokeHost } from "../shell/host-boundary.mjs";
const spec = JSON.parse(fs.readFileSync(new URL("./fixtures/live/rsvp.json",import.meta.url)));
const model = canonicalSource(spec), modulePath = `prism/${model.name}.lex.tex`;
const files = [["prism/spec.json",JSON.stringify(spec)],[modulePath,model.source],["src/lib/prism.ts",checkSpec(spec).module]];
const changed = (path,text) => files.map(row=>row[0]===path?[path,text]:[...row]);
test("an exact checked canonical build and an ordinary project pass",()=>{
  assert.doesNotThrow(()=>assertCanonicalBuild(files));
  assert.doesNotThrow(()=>assertCanonicalBuild([["src/lib/prism.ts","ordinary code"]]));
  assert.doesNotThrow(()=>assertCanonicalBuild([...files,["notes.txt","ordinary text"]]));
});
test("changed or missing canonical module bytes refuse",()=>{
  for(const candidate of [changed(modulePath,model.source+"\n"), files.filter(([path])=>path!==modulePath), changed(modulePath,model.source.replace("semanticmodule","private-module"))])
    assert.throws(()=>assertCanonicalBuild(candidate),e=>e.code==="canonicalBuildMismatch");
});
test("changed or missing runtime bytes refuse even with coherent content hashes",()=>{
  for(const candidate of [changed("src/lib/prism.ts","private runtime"),files.filter(([path])=>path!=="src/lib/prism.ts")]){
    const source={name:"app",reference:"blake3:"+"a".repeat(64),files:candidate,dependencies:[]};
    assert.throws(()=>verifyCapturedSource(source,{...source,oid:source.reference}),e=>e.code==="canonicalBuildMismatch");
  }
});
test("invalid oversized and contradictory decision specifications refuse",()=>{
  const contradiction=structuredClone(spec);contradiction.rules[0].then="confirmed";
  assert.equal(checkSpec(contradiction).ok,false);
  const invalidModel=canonicalSource(contradiction);
  assert.throws(()=>assertCanonicalBuild(changed("prism/spec.json",JSON.stringify(contradiction)).map(row=>row[0]===modulePath?[modulePath,invalidModel.source]:row)),e=>e.code==="canonicalBuildMismatch");
  for(const text of ["private invalid JSON","null",JSON.stringify({...spec,fields:[]})," ".repeat(1024*1024)+JSON.stringify(spec),JSON.stringify(contradiction)])
    assert.throws(()=>assertCanonicalBuild(changed("prism/spec.json",text)),e=>e.code==="canonicalBuildMismatch");
});
test("the common capture boundary checks closure before either builder effect",()=>{
  const host=fs.readFileSync(new URL("../shell/host.js",import.meta.url),"utf8");
  for(const [start,end,effect] of [["async function buildPreview(","// ---- the live preview","grantPreview("],["async function buildPortable(","// Runs inside the compile frame","esbuild()"]]){
    const body=host.slice(host.indexOf(start),host.indexOf(end,host.indexOf(start)));
    assert.ok(body.includes("verifiedBuildSource(appId)"));
    assert.ok(body.indexOf("verifiedBuildSource(appId)")<body.indexOf(effect));
  }
  const source=fs.readFileSync(new URL("../shell/build-source.mjs",import.meta.url),"utf8");
  assert.match(source,/assertCanonicalBuild\(source.files\)/);
  assert.match(source,/return verifyCapturedSource\(source, snapshot\)/);
});
test("canonical build mismatch has a modeled private public error",async()=>{
  const result=await invokeHost("build",[],()=>{throw Object.assign(Error("private source"),{code:"canonicalBuildMismatch"});},null);
  assert.equal(result.error.code,"canonicalBuildMismatch");assert.equal(result.error.kind,"validation");assert.ok(!JSON.stringify(result).includes("private source"));
  const model=fs.readFileSync(new URL("../../app/src/FoundrySpecification.lex.tex",import.meta.url),"utf8");assert.match(model,/name: canonicalBuildBoundary/);
});
