import assert from "node:assert/strict";
import fs from "node:fs";
import vm from "node:vm";
import { webcrypto } from "node:crypto";
import { test } from "node:test";
import * as prism from "../shell/prism.js";
import * as live from "../shell/live/project.mjs";
import * as lexlean from "../shell/live/lexlean.mjs";
import { applyOps, createEditor } from "../shell/live/edit.mjs";
import { sameRuleReservation } from "../shell/rules-source.mjs";
import { assertGenerationActive } from "../shell/generation-patch.mjs";
import {applicationShape,retiredApplicationPaths,isStatefulModule} from '../shell/application-shape.mjs';
const host = fs.readFileSync(new URL("../shell/host.js", import.meta.url), "utf8");
const initial = JSON.parse(fs.readFileSync(new URL("./fixtures/live/rsvp.json", import.meta.url)));
const key = spec => { const checked = prism.checkSpec(spec).spec; return live.kappaOfValue({ name: checked.name, logic: live.logicOf(checked) }); };
function fn(name, end, env) {
  const start = host.indexOf(`async function ${name}(`);
  assert.ok(start >= 0 && host.indexOf(end, start) > start);
  return vm.runInNewContext(host.slice(start, host.indexOf(end, start)) + `\n${name}`, {sameRuleReservation, ...env});
}
async function revise(spec, before) {
  let saved, batch;
  const source = { files: [["prism/spec.json", JSON.stringify(initial)], ["src/pages/Index.tsx", "import '@/lib/prism'"]] };
  const run = fn("rulesAfterTurn", "// Lean proves", {
    assertGenerationActive, coreReady: Promise.resolve(), core: {}, dbReady: Promise.resolve({}),
    retiredApplicationPaths, appShape:{SOURCE_PATH:'prism/App.lex.tex',isAppShape:isStatefulModule}, forgetApplicationFrames(){},
    captureProjectSource: async () => source, prism, live, lexlean, rulesSnapshotOf: async () => ({ state: before, expected: before }),
    enc: new TextEncoder(), kappa: async bytes => live.kappaOfValue([...bytes]),
    nextPhase: async (phase, event) => event === "reword" ? phase : event === "check" ? "checked" : "drafting",
    applyStoredProjectPatch: async (_, id, patch, options) => { batch = { patch, options }; saved = options.rulesUpdate?.state; return { changed: patch.writes.length, source: { ...source, reference: "new" } }; },
    saveRules: async () => { assert.fail("status must commit with source"); }, now: () => "now",
  });
  const result = await run(1, `<prism-spec>${JSON.stringify(spec)}</prism-spec>`);
  return { saved, batch, result, source };
}
const proved = () => ({ phase: "proved", spec: prism.checkSpec(initial).spec, specKappa: "prior", logicKappa: key(initial), proof: { id: "proof" } });
test("checked wording changes retain a matching proof but not a current publication claim", async () => {
  const before = { ...proved(), phase: "published", publishedAs: "https://old.example", publicationCurrent: true };
  const spec = { ...initial, title: "New wording" };
  const { saved, batch, source, result } = await revise(spec, before);
  assert.equal(saved.phase, "published"); assert.equal(saved.proof, before.proof);
  assert.equal(saved.publicationCurrent, false); assert.match(prism.card(saved), /wording changed since publication/);
  assert.equal(batch.options.source, source); assert.equal(result.source.reference, "new");
  assert.ok(batch.patch.writes.some(w => w.path === "prism/spec.json"));
  assert.ok(batch.patch.writes.some(w => w.path === "src/lib/prism.ts"));
  for (const stale of [false, true]) {
    let trust;
    const refresh = fn("refreshLive", "// ---- the one write path", {
      applicationShape, dbReady:Promise.resolve({}), captureProjectSource:async()=>({files:[['prism/spec.json',JSON.stringify(spec)]]}),
      get: async () => JSON.stringify(stale ? initial : spec, null, 2) + "\n", rulesOf: async () => saved,
      enc: new TextEncoder(), kappa: async bytes => live.kappaOfValue([...bytes]), pushLive: (_, __, value) => { trust = value; },
    });
    await refresh(1); assert.equal(trust, stale ? "draft" : "proved");
  }
});
test("changed logic drops proof and returns to checked", async () => {
  const spec = structuredClone(initial); spec.rules[0].then = "none"; spec.properties = []; spec.examples = [];
  assert.equal(prism.checkSpec(spec).ok, true);
  for (const candidate of [spec, { ...initial, name: "renamed-module" }]) {
    const { saved } = await revise(candidate, proved());
    assert.equal(saved.phase, "checked"); assert.equal(saved.proof, null); assert.notEqual(saved.logicKappa, key(initial));
  }
});
test("cached identity cannot lend a proof from a different checked spec", async () => {
  const before = proved(); before.spec = { ...before.spec, otherwise: "none" };
  const { saved } = await revise({ ...initial, title: "Reword" }, before);
  assert.equal(saved.phase, "checked"); assert.equal(saved.proof, null);
});
test("proof completion follows matching logic and refuses a competing commit", async () => {
  for (const accepted of [false, true]) {
    const state = { ...proved(), phase: "proving" }, latest = { ...state, specKappa: "reworded" }; let reads = 0, refreshed = 0, audits = 0;
    const prove = fn("proveRules", '// "approve"', {
      rulesOf: async () => ++reads === 1 ? state : latest,
      kernelReady: null, prism: { prove: async () => ({ ok: true, record: state.proof }) }, nextPhase: async () => "proved",
      saveRules: async (_, value, expected) => { assert.equal(expected, latest); assert.equal(value.specKappa, "reworded"); return accepted; },
      put: async (_, __, value) => { assert.equal(value.resource, state.logicKappa); audits++; }, refreshLive: async () => refreshed++, now: () => "now",
    });
    assert.equal((await prove(1, state.logicKappa))?.ok ?? false, accepted);
    assert.equal(refreshed, Number(accepted)); assert.equal(audits, Number(accepted));
  }
});
test("live persistence failure discards optimistic history and reloads committed state", async () => {
  const source = { files: [["prism/spec.json", JSON.stringify(initial)]] }, editors = new Map([[1, createEditor(initial, () => ({ ok: true }))]]);
  let refreshed = 0, refused = 0;
  const patch = fn("patchSpec", "async function stepSpec(", {
    structuredClone, requestAnimationFrame: (f) => setTimeout(f, 0), setTimeout, liveEditors: editors, liveBusy: new Map(), pushLive() {}, applyOps, inOrder: (_, job) => job(),
    assertGenerationActive, captureProjectSource: async () => source, dbReady: Promise.resolve(),
    editorOf: async (_, captured) => { assert.equal(captured, source); return editors.get(1); }, rulesOf: async () => proved(),
    saveSpec: async (_, spec, options) => { assert.equal(spec.title, "Changed"); assert.equal(options.source, source); throw Object.assign(Error("projectEditConflict"), { code: "projectEditConflict" }); },
    refreshLive: async () => refreshed++, refuse: () => refused++,
  });
  await assert.rejects(patch(1, [{ op: "set", path: ["title"], value: "Changed" }]), e => e.code === "projectEditConflict");
  assert.equal(editors.has(1), false); assert.equal(refreshed, 1); assert.equal(refused, 1);
});
test("the signed installed guest supports reword alongside composition and work", async () => {
  const self = {};
  vm.runInNewContext(fs.readFileSync(new URL("../shell/kernel.js", import.meta.url), "utf8"), { self, crypto: webcrypto, WebAssembly, TextEncoder, TextDecoder, Uint8Array, atob });
  const guest = await self.ForgeKernel.load(async path => new Response(fs.readFileSync(new URL("../shell/" + path, import.meta.url))));
  assert.ok(guest.vectors >= 114);
  for (const phase of ["drafting", "checked", "proving", "proved", "published"]) assert.equal(await guest.ask(phase + " reword"), phase);
  assert.equal(await guest.ask("compose "), "malformedComposition");
  assert.equal(await guest.ask("work "), "invalidWorkRequest");
});

test("invalid live edits redraw with source-bound publication trust", async () => {
  for (const mismatch of [false, true]) {
    const json = JSON.stringify(initial), state = { ...proved(), phase: "published", publicationCurrent: false, specKappa: mismatch ? "other" : "current" };
    let trust;
    const refresh = fn("refreshLive", "// ---- the one write path", {
      applicationShape, dbReady:Promise.resolve({}), captureProjectSource:async()=>({files:[['prism/spec.json',json]]}),
      get: async () => json, rulesOf: async () => state, enc: new TextEncoder(),
      kappa: async () => "current", pushLive: (_, __, value) => { trust = value; },
    });
    const editor = createEditor(initial, () => ({ ok: false, issues: ["invalid"] }));
    const patch = fn("patchSpec", "async function stepSpec(", {
      structuredClone, requestAnimationFrame: (f) => setTimeout(f, 0), setTimeout, liveEditors: new Map([[1, editor]]), liveBusy: new Map(), applyOps,
      pushLive: (_, __, value) => { trust = value; }, inOrder: (_, job) => job(),
      assertGenerationActive, dbReady: Promise.resolve(), captureProjectSource: async () => ({}),
      editorOf: async () => editor, rulesOf: async () => state, refuse() {}, refreshLive: refresh,
    });
    assert.equal((await patch(1, [{ op: "set", path: ["title"], value: "Rejected" }])).ok, false);
    assert.equal(trust, mismatch ? "draft" : "proved");
  }
});
test("proof completion retries a matching reword CAS without rerunning the prover", async () => {
  const state = { ...proved(), phase: "proving", proof: null }, reword = { ...state, specKappa: "new" };
  let reads = 0, saves = 0, proofs = 0;
  const prove = fn("proveRules", '// "approve"', {
    rulesOf: async () => ++reads <= 2 ? state : reword,
    kernelReady: null, prism: { prove: async () => { proofs++; return { ok: true, record: { id: "settled" } }; } },
    nextPhase: async () => "proved", saveRules: async (_, next, expected) => {
      saves++; if (saves === 1) return false;
      assert.equal(expected, reword); assert.equal(next.specKappa, "new"); assert.equal(next.proof.id, "settled"); return true;
    }, put: async () => {}, refreshLive: async () => {}, now: () => "now",
  });
  assert.equal((await prove(1, state.logicKappa))?.ok, true);
  assert.equal(saves, 2); assert.equal(proofs, 1);
});
test("proof completion stops retrying when the competing edit changes semantics", async () => {
  const state = { ...proved(), phase: "proving" };
  let reads = 0, saves = 0;
  const prove = fn("proveRules", '// "approve"', {
    rulesOf: async () => ++reads <= 2 ? state : { ...state, logicKappa: "different" },
    kernelReady: null, prism: { prove: async () => ({ ok: true, record: { id: "old-proof" } }) },
    nextPhase: async () => "proved", saveRules: async () => { saves++; return false; },
    put() { assert.fail("changed semantics must not accept old proof"); }, now: () => "now",
  });
  assert.equal(await prove(1, state.logicKappa), null); assert.equal(saves, 1);
});

function liveHelloHarness({fail = false, cached = false, stateful = false, trusted = true} = {}) {
  const begin = host.indexOf('window.addEventListener("message", (e) => {\n  const m = e.data;'), end = host.indexOf('new MutationObserver', begin);
  assert.ok(begin >= 0 && end > begin, 'live hello must use the production message handler');
  let handler;
  const source = {}, other = {}, log = {refresh:[], sent:[], refused:[]}, windows = new Map();
  const refreshPreview = appId => {
    log.refresh.push(appId);
    const result = fail ? Promise.reject(Object.assign(Error('private storage detail'),{code:'projectEditStateInvalid'})) : Promise.resolve();
    // The harness owns the returned rejection even for the original faulty listener.
    // The production handler must independently convert it to a visible refusal.
    void result.catch(()=>{}); return result;
  };
  vm.runInNewContext(host.slice(begin,end), {
    window:{addEventListener:(_name,fn)=>{handler=fn;}},isLiveFrame:win=>trusted&&win===source,
    liveWindows:windows,appApps:new Set(stateful?[1]:[]),liveLatest:new Map(cached?[[1,{}]]:[]),
    appFramesReady:async()=>({has:()=>false}),refreshPreview,sendLive:id=>log.sent.push(id),
    refuse:(...args)=>log.refused.push(args),
  });
  return{log,windows,hello:app=>handler({source,data:{type:'live/hello',app}}),foreign:()=>handler({source:other,data:{type:'live/hello',app:'1'}})};
}
test('warm-up and invalid live hello identities do not read or bind project state',async()=>{
  const h=liveHelloHarness();
  for(const id of ['',0,'0',-1,'-1','01','1.5',1.5,'9007199254740992',Number.MAX_SAFE_INTEGER+1,null,undefined,{}])h.hello(id);
  h.foreign();await new Promise(setImmediate);
  assert.equal(h.windows.size,0);assert.deepEqual(h.log,{refresh:[],sent:[],refused:[]});
});
test('valid live hello keeps trusted frame ownership and refreshes or resends current content',async()=>{
  for(const stateful of [false,true]){
    const h=liveHelloHarness({stateful});h.hello('1');await new Promise(setImmediate);
    assert.deepEqual(h.log.refresh,[1]);assert.equal([...h.windows.values()][0],1);assert.deepEqual(h.log.refused,[]);
  }
  const cached=liveHelloHarness({cached:true});cached.hello(1);await new Promise(setImmediate);
  assert.deepEqual(cached.log.sent,[1]);assert.deepEqual(cached.log.refresh,[]);
  const untrusted=liveHelloHarness({trusted:false});untrusted.hello('1');await new Promise(setImmediate);
  assert.equal(untrusted.windows.size,0);assert.deepEqual(untrusted.log.refresh,[]);
});
test('failed live hello refresh is a private visible refusal, never an unhandled success',async()=>{
  for(const stateful of [false,true]){
    const h=liveHelloHarness({fail:true,stateful});h.hello('1');await new Promise(setImmediate);
    assert.deepEqual(h.log.refresh,[1]);assert.equal(h.log.refused.length,1);
    const [app,key,message]=h.log.refused[0];assert.equal(app,1);assert.equal(key,null);
    assert.match(message,/preview.*could not be refreshed/i);assert.doesNotMatch(message,/private storage detail/);
    assert.deepEqual(h.log.sent,[]);
  }
});
