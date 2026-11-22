import { createHash } from "node:crypto";
import assert from "node:assert/strict";
import fs from "node:fs";
import { test } from "node:test";
import { encodeComposition, admitWithGuest } from "../shell/composition-codec.mjs";
const vectors = JSON.parse(fs.readFileSync(new URL("../../verification/composition-wire-vectors.json", import.meta.url)));
const policy = { cpu: 4, memory: 6, grants: [5], targets: [1], evidence: [10, 11] };
const feature = { key: 1, version: 1, cpu: 2, memory: 3, vocabulary: [1], outputs: [7], state: [], events: [], errors: [], effects: [], conflicts: [], targets: [1], accessibility: [10], quality: [11], needs: [], inputs: [] };
const hex = bytes => Buffer.from(bytes).toString("hex");
test("codec matches independent generated-guest acceptance vector bytes", () => {
  assert.equal(hex(encodeComposition(policy, [feature])), vectors.find(v => v.name === "single").request);
  assert.equal(hex(encodeComposition(policy, [])), vectors.find(v => v.name === "empty").request);
  assert.equal(hex(encodeComposition({ ...policy, cpu: 1 }, [feature])), vectors.find(v => v.name === "cpu").request);
  for (const vector of vectors.filter(v => v.policy)) {
    const request = encodeComposition(vector.policy, vector.features);
    assert.equal(hex(request), vector.request, vector.name);
    assert.match(new TextDecoder().decode(request), /^compose [0-9a-f]*$/);
  }
});
test("scalar encoding refuses coercion, fractions, negatives and overflow", () => {
  for (const cpu of [undefined, null, "1", -1, 1.5, Infinity, NaN, 2 ** 32]) assert.throws(() => encodeComposition({ ...policy, cpu }, [feature]), { code: "malformedComposition" });
});
test("wire shape and list bounds refuse before invoking any guest", async () => {
  for (const [p, f] of [[{ ...policy, extra: 1 }, [feature]], [policy, [{ ...feature, extra: 1 }]], [policy, Array(33).fill(feature)], [{ ...policy, grants: Array(65).fill(1) }, [feature]], [policy, [{ ...feature, needs: [{}] }]], [policy, [{ ...feature, inputs: [{ producer: {}, schema: 7 }] }]]]) {
    const result = await admitWithGuest({ dispatch() { assert.fail("malformed request invoked guest"); } }, p, f);
    assert.equal(result.code, "malformedComposition");
  }
});
test("large aggregate payload refuses with the modeled size error", () => {
  const large = { ...feature, ...Object.fromEntries(["vocabulary", "outputs", "state", "events", "errors", "effects", "conflicts", "targets", "accessibility", "quality"].map(key => [key, Array(64).fill(1)])) };
  assert.throws(() => encodeComposition(policy, Array(32).fill(large)), { code: "compositionTooLarge" });
});
test("adapter never decides admission and preserves every model refusal", async () => {
  for (const code of ["accepted", "emptyComposition", "invalidDeclaration", "duplicateFeature", "missingDependency", "incompatibleInput", "featureConflict", "permissionDenied", "unsupportedTarget", "missingEvidence", "budgetExceeded", "malformedComposition", "compositionTooLarge"]) {
    let calls = 0;
    const result = await admitWithGuest({ dispatch(bytes) { calls++; assert.equal(hex(bytes), vectors[0].request); return new TextEncoder().encode(code); } }, policy, [feature]);
    assert.equal(calls, 1);
    assert.equal(result.ok, code === "accepted");
    if (!result.ok) assert.equal(result.code, code);
  }
});
test("invalid guest responses and runtime failures refuse", async () => {
  for (const response of ["accepted", new Uint8Array([255]), new Uint8Array(65), new TextEncoder().encode("accepted\n"), new TextEncoder().encode("unknown")]) {
    assert.equal((await admitWithGuest({ dispatch: () => response }, policy, [feature])).code, "invalidCompositionResponse");
  }
  assert.equal((await admitWithGuest({ dispatch() { throw Error("private"); } }, policy, [feature])).code, "compositionRuntimeFailed");
});

// Closed byte aliases keep metadata within the SDK's admitted grammar and
// avoid repeatedly elaborating large Lean byte-array literals.
const fixtureChunks = new Map();
function requestTerm(hex) {
  if (hex.length <= 128) {
    const name = "wire" + createHash("sha256").update(Buffer.from(hex, "hex")).digest("hex").slice(0, 16);
    if (fixtureChunks.has(name)) assert.equal(fixtureChunks.get(name), hex);
    fixtureChunks.set(name, hex);
    return `(CompositionFixtures::${name})`;
  }
  const cut = Math.floor(Math.ceil(hex.length / 128) / 2) * 128;
  return `(prim append ${requestTerm(hex.slice(0, cut))} ${requestTerm(hex.slice(cut))} : Bytes)`;
}
test("the actual guest model carries the full wire corpus and routes through semantic admission", () => {
  const kernel = fs.readFileSync(new URL("../../app/src/ForgeKernel.lex.tex", import.meta.url), "utf8");
  const protocol = fs.readFileSync(new URL("../../app/src/CompositionProtocol.lex.tex", import.meta.url), "utf8");
  assert.equal(vectors.length, 26);
  assert.match(kernel, /CompositionProtocol::dispatchComposition/);
  assert.match(protocol, /FeatureComposition::admitComposition/);
  assert.match(kernel, /requestMaximum := \(32776 : UInt32\)/);
  const fixtures = fs.readFileSync(new URL("../../app/src/CompositionFixtures.lex.tex", import.meta.url), "utf8");
  for (const vector of vectors) {
    const expected = Buffer.from(vector.response).toString("hex");
    assert.ok(kernel.includes(`request := ${requestTerm(vector.request)}, response := 0x${expected}`), `missing runtime vector ${vector.name}`);
  }
  for (const [name, hex] of fixtureChunks) assert.ok(fixtures.replace(/\s+/g, "").includes(`definition{name:${name},parameters:[],result:Bytes,body:0x${hex}}`), `fixture alias changed: ${name}`);
});
