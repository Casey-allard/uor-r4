import assert from "node:assert/strict";
import fs from "node:fs";
import vm from "node:vm";
import { test } from "node:test";
import { spawnSync } from "node:child_process";
import { verifyCapturedSource, sameBuildSource, requiresRulePublication } from "../shell/build-source.mjs";
const source = { name: "Example", reference: "blake3:" + "a".repeat(64), files: [["b", "\ufeff\u00e9"], ["a", "exact\r\n"]], dependencies: [] };
const snapshot = () => ({ name: source.name, oid: source.reference, files: structuredClone(source.files) });
const host = fs.readFileSync(new URL("../shell/host.js", import.meta.url), "utf8");
test("verified source retains exact text and permits only ordering differences", () => {
  const target = snapshot(); target.files.reverse(); assert.equal(verifyCapturedSource(source, target), source);
  assert.equal(source.files[0][1], "\ufeff\u00e9");
});
test("reference name and file disagreement refuse", () => {
  for (const target of [{ ...snapshot(), oid: "other" }, { ...snapshot(), name: "Other" }, { ...snapshot(), files: [["a", "exact\n"], ["b", "\u00e9"]] }, { ...snapshot(), files: [] }]) assert.throws(() => verifyCapturedSource(source, target), e => e.code === "buildSourceMismatch");
});
test("dependency metadata must be an exact part of snapshot source", () => {
  const declared = ["prism/dependencies.json", JSON.stringify({ profile: "foundry/direct-dependency-requests/1", dependencies: [{ name: "react", version: "19.1.0" }] })];
  const captured = { ...source, files: [...source.files, declared], dependencies: ["react@19.1.0"] }, target = { ...snapshot(), files: captured.files };
  assert.equal(verifyCapturedSource(captured, target), captured);
  assert.throws(() => verifyCapturedSource({ ...captured, dependencies: [] }, target), e => e.code === "buildSourceMismatch");
  assert.throws(() => verifyCapturedSource({ ...source, dependencies: ["react@19.1.0"] }, snapshot()), e => e.code === "buildSourceMismatch");
});
test("malformed sources cannot pass the comparison or checker", () => {
  for (const bad of [null, {}, { ...source, files: null }, { ...source, files: [["a", 1]] }, { ...source, files: [["a", "1"], ["a", "2"]] }, { ...source, dependencies: null }]) {
    assert.equal(sameBuildSource(source, bad), false); assert.throws(() => verifyCapturedSource(bad, snapshot()), e => e.code === "buildSourceMismatch");
  }
  assert.equal(sameBuildSource(source, { ...source, files: [...source.files].reverse() }), true);
  for (const patch of [{ name: "x" }, { reference: "x" }, { files: [] }, { dependencies: ["changed"] }]) assert.equal(sameBuildSource(source, { ...source, ...patch }), false);
});
test("an app whose source has rules is never published as a plain app, even when its proof cache is absent", async () => {
  const start = host.indexOf('handlers.set("holo:publish"'), end = host.indexOf("\nwindow.electron", start), handlers = new Map(); let audits = 0;
  const captured = { ...source, files: [...source.files, ["prism/spec.json", "invalid or unchecked"]] };
  vm.runInNewContext(host.slice(start, end), {
    handlers, coreReady: Promise.resolve(), performance: { now: () => 0 }, URL, BASE: "https://foundry.example/",
    core: { run: ({ op }) => op === "view" ? {} : { decision: "Accept" } },
    fetch: async () => ({ ok: true, json: async () => ({ closure: "closure", attestation_id: "receipt", browser_projection_sha256: "closure" }) }),
    verifiedBuildSource: async () => captured, requiresRulePublication, put: async () => audits++,
    rulesOf: async () => null, buildPortable() { assert.fail("rule publication bypassed"); },
  });
  const result = await handlers.get("holo:publish")({ appId: 1 }); assert.equal(result.decision, "Chat"); assert.equal(audits, 1);
});
test("both builders consume captured files and check source before promotion", () => {
  for (const [name, end] of [["buildPreview", "// ---- the live preview"], ["buildPortable", "// Runs inside the compile frame"]]) {
    const start = host.indexOf(`async function ${name}(`), body = host.slice(start, host.indexOf(end, start));
    assert.ok(body.includes("new Map(source.files)")); assert.ok(body.includes("source.dependencies"));
    assert.ok(body.includes("assertBuildSourceCurrent(await dbReady, appId, source)")); assert.ok(!body.includes("filesOf(appId)"));
  }
  assert.ok(host.includes("buildPortable(appId, captured)")); assert.ok(host.includes("assertBuildSourceCurrent(await dbReady, appId, captured)"));
  assert.ok(fs.readFileSync(new URL("./shell_parse.mjs", import.meta.url), "utf8").includes('"build-source.mjs": "mjs"'));
});


test("build-source browser fixture links the complete production module closure and rejects missing bytes or exports", () => {
  // Let Node's ES-module linker parse actual modules; a text import scan of prism.js
  // would mistake the generated React source inside its template string for imports.
  const link = String.raw`
    import fs from 'node:fs'; import vm from 'node:vm';
    const fixtureURL = new URL(process.argv[1]);
    const fixture = fs.readFileSync(fixtureURL, 'utf8');
    const begin = fixture.indexOf('const sources = new Map('), end = fixture.indexOf('const server =', begin);
    if(begin < 0 || end <= begin) throw Error('missing fixture inventory');
    const sources = vm.runInNewContext(fixture.slice(begin,end).replaceAll('import.meta.url','fixtureURL'), {fs,URL,Map,fixtureURL});
    if(process.argv[2]) sources.delete(process.argv[2]);
    if(process.argv[3]) sources.set(process.argv[3], 'export const wrongExport = true;');
    const entries = [...fixture.matchAll(/\bimport\(["'](\/[^"']+)["']\)/g)].map(match=>match[1]);
    if(entries.length !== 6) throw Error('review changed browser entry imports');
    const context = vm.createContext({TextEncoder,TextDecoder,URL,console}), modules = new Map();
    const moduleOf = name => {
      if(!sources.has(name)) throw Error('fixture does not serve '+name);
      if(!modules.has(name)) modules.set(name,new vm.SourceTextModule(String(sources.get(name)),{context,identifier:name}));
      return modules.get(name);
    };
    const root = new vm.SourceTextModule(entries.map(name=>'import '+JSON.stringify(name)+';').join('\n'),{context,identifier:'/fixture.mjs'});
    await root.link((specifier,parent)=>moduleOf(new URL(specifier,'https://fixture.example'+parent.identifier).pathname));
    await root.evaluate();
    console.log('linked '+modules.size+' exact fixture modules');
  `;
  const run = (missing = '', wrongExport = '') => spawnSync(process.execPath, ['--experimental-vm-modules', '--input-type=module', '-e', link, new URL('./build_source_browser.mjs', import.meta.url).href, missing, wrongExport], {encoding:'utf8',timeout:30000});
  const good = run();assert.equal(good.status,0,good.stdout+'\n'+good.stderr);assert.match(good.stdout,/linked [1-9][0-9]* exact fixture modules/);
  for(const missing of ['/application-shape.mjs','/prism/spec-file.mjs','/live/lexlean.mjs']) {
    const bad = run(missing);assert.notEqual(bad.status,0);assert.ok(bad.stderr.includes('fixture does not serve '+missing),bad.stderr);
  }
  const wrong = run('', '/prism/spec-file.mjs');assert.notEqual(wrong.status,0);assert.match(wrong.stderr,/does not provide an export named 'readSpecFile'/);
});
