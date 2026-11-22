import assert from "node:assert/strict";
import { test } from "node:test";
import fs from "node:fs";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";
import { createCompilerLoader } from "../shell/browser-compiler.mjs";
import { invokeHost } from "../shell/host-boundary.mjs";
const base = "https://foundry.invalid/nested/foundry/";
test("compiler archive locks exact bytes license identity and independent staging", () => {
  const output = execFileSync("python3", [fileURLToPath(new URL("./compiler_assets_cases.py", import.meta.url))], { encoding: "utf8", timeout: 120000 });
  assert.deepEqual(JSON.parse(output), { passed: true, files: 3 });
});
test("compiler imports local script and wasm once under the actual base", async () => {
  const requests = [], starts = [];
  const compiler = { initialize: async options => starts.push(options) };
  const load = createCompilerLoader(async url => { requests.push(url); return compiler; });
  const [a,b] = await Promise.all([load(base),load(base)]);
  assert.equal(a,compiler);assert.equal(a,b);assert.equal(await load(base),compiler);
  assert.deepEqual(requests,[base+'compiler/esm/browser.min.js']);
  assert.deepEqual(starts,[{wasmURL:base+'compiler/esbuild.wasm'}]);
});
test("failed imports are typed private and retryable", async () => {
  let attempts=0;const compiler={initialize:async()=>{}};
  const load=createCompilerLoader(async()=>{if(++attempts===1)throw Error('private source');return compiler;});
  await assert.rejects(load(base), e=>e.code==='browserCompilerUnavailable'&&!e.message.includes('private'));
  assert.equal(await load(base),compiler);assert.equal(attempts,2);
});
test("failed wasm initialization does not poison the shared promise", async () => {
  let attempts=0;const compiler={initialize:async()=>{if(++attempts===1)throw Error('private wasm');}};
  const load=createCompilerLoader(async()=>compiler);
  const settled=await Promise.allSettled([load(base),load(base)]);
  assert.ok(settled.every(r=>r.status==='rejected'&&r.reason.code==='browserCompilerUnavailable'));
  assert.equal(await load(base),compiler);assert.equal(attempts,2);
});
test("actual command boundary preserves the compiler's private typed refusal", async()=>{
  const load=createCompilerLoader(async()=>{throw Error('private URL');});
  const result=await invokeHost('compile',[],()=>load(base),null);
  assert.equal(result.error.code,'browserCompilerUnavailable');assert.ok(!JSON.stringify(result).includes('private URL'));
});
test("actual build publication and CI require the local compiler closure",()=>{
  const read=p=>fs.readFileSync(new URL(p,import.meta.url),'utf8');
  const host=read('../shell/host.js');assert.match(host,/const esbuild = \(\) => loadBrowserCompiler\(BASE\)/);assert.ok(!host.includes('cdn.jsdelivr.net/npm/esbuild-wasm'));
  assert.equal((host.match(/const es = await esbuild\(\)/g)||[]).length,2);
  assert.match(read('../shell/sw.template.js'),/path.startsWith\("compiler\/"\)/);
  for(const path of ['compiler/package.json','compiler/esm/browser.min.js','compiler/esbuild.wasm'])assert.ok(read('../shell/shell-cache.js').includes(JSON.stringify(path)));
  for(const workflow of ['studio.yml','pages.yml']){const s=read('../../.github/workflows/'+workflow);assert.match(s,/npm ci --prefix runtime --ignore-scripts/);assert.match(s,/python3 scripts\/compiler_assets.py --check/);}
  assert.match(read('../../.github/workflows/studio.yml'),/node tools\/compiler_browser.mjs/);
});
