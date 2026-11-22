import assert from "node:assert/strict";
import { test } from "node:test";
import fs from "node:fs";
import { encodeWorkRequest, decodeWorkResponse, executeWorkWithGuest } from "../shell/work-codec.mjs";
const corpus = JSON.parse(fs.readFileSync(new URL("../../verification/work-wire-vectors.json", import.meta.url), "utf8"));
const state = { owner: 7, revision: 3, capacity: 20, spent: 4, reserved: 8, checkpoint: 2, remaining: 5, phase: 1 };
const transition = action => ({ kind: "transition", state, owner: 7, revision: 3, action });
const text = bytes => new TextDecoder().decode(bytes);
const bytes = text => new TextEncoder().encode(text);
const vector = name => corpus.vectors.find(value => value.name === name);
test("initialization and every action encode exact independent wire words", () => {
  assert.equal(text(encodeWorkRequest({ kind: "initialize", owner: 7, capacity: 20, remaining: 5 })), vector("initialize").request);
  for (const [name, action, changes] of [["progress", { kind: "progress", cost: 3, units: 2, checkpoint: 9 }, {}], ["pause", { kind: "pause" }, {}], ["reserve", { kind: "reserve", amount: 8 }, { phase: 2 }], ["resume", { kind: "resume" }, { phase: 2 }], ["transfer", { kind: "transfer", owner: 9 }, { phase: 2 }], ["finish", { kind: "finish" }, { phase: 2, remaining: 0 }], ["start", { kind: "start" }, { phase: 0, spent: 0, checkpoint: 0 }]]) {
    const input = transition(action); input.state = { ...state, ...changes };
    assert.equal(text(encodeWorkRequest(input)), vector(name).request);
  }
});
test("the wire adapter rejects invalid shapes and unrepresentable scalar values", () => {
  for (const input of [null, {}, { kind: "initialize", owner: 7, capacity: 20, remaining: 5, extra: true }, transition({ kind: "pause", amount: 3 }), transition({ kind: "toString" }), transition({ kind: "unknown" })]) assert.throws(() => encodeWorkRequest(input));
  for (const owner of [-1, 0x100000000, 0.1, NaN, Infinity, "7"]) assert.throws(() => encodeWorkRequest({ kind: "initialize", owner, capacity: 20, remaining: 5 }));
});
test("wire encoding preserves zero and maximum declarations for guest admission", () => {
  const input = { kind: "initialize", owner: 0, capacity: 0xffffffff, remaining: 0 };
  assert.equal(text(encodeWorkRequest(input)), "work 0000000000000000ffffffff00000000");
});
test("successful responses preserve every returned state field", () => {
  assert.deepEqual(decodeWorkResponse(bytes(vector("progress").response)), { ok: true, state: { owner: 7, revision: 4, capacity: 20, spent: 7, reserved: 5, checkpoint: 9, remaining: 3, phase: 1 } });
  assert.deepEqual(decodeWorkResponse(bytes(vector("transfer").response)).state, { ...state, owner: 9, revision: 4, phase: 2 });
});
test("typed guest refusals survive while malformed replies fail closed", () => {
  for (const value of corpus.vectors.filter(value => !value.response.startsWith("ok "))) assert.deepEqual(decodeWorkResponse(bytes(value.response)), { ok: false, code: value.response });
  for (const value of ["", "ok ", "invalid", vector("progress").response + "0", "ok " + "f".repeat(64)]) assert.deepEqual(decodeWorkResponse(bytes(value)), { ok: false, code: "workRuntimeFailed" });
});
test("execution invokes the guest once and never supplies a semantic fallback", async () => {
  let calls = 0; const kernel = { async dispatch(request) { calls++; assert.equal(text(request), vector("pause").request); return bytes(vector("pause").response); } };
  assert.equal((await executeWorkWithGuest(kernel, transition({ kind: "pause" }))).state.phase, 2); assert.equal(calls, 1);
  assert.deepEqual(await executeWorkWithGuest(kernel, {}), { ok: false, code: "invalidWorkRequest" }); assert.equal(calls, 1);
  assert.deepEqual(await executeWorkWithGuest({ dispatch() { throw Error("private"); } }, transition({ kind: "pause" })), { ok: false, code: "workRuntimeFailed" });
});
test("every independent corpus vector is embedded in the generated guest acceptance", () => {
  const kernel = fs.readFileSync(new URL("../../app/src/ForgeKernel.lex.tex", import.meta.url), "utf8");
  assert.equal(corpus.spec, "foundry/work-wire-vectors/1"); assert.equal(corpus.vectors.length, 30);
  for (const value of corpus.vectors) {
    const request = Buffer.from(value.request).toString("hex"), response = Buffer.from(value.response).toString("hex");
    assert.ok(kernel.includes(`request := 0x${request}, response := 0x${response}`), value.name);
  }
  assert.ok(kernel.includes('\\importmodule{WorkProtocol}'));
  assert.ok(kernel.includes('WorkProtocol::dispatchWork'));
  assert.ok(kernel.includes('responseMaximum := (4096 : UInt32)'));
  const model = fs.readFileSync(new URL("../../app/src/WorkProtocol.lex.tex", import.meta.url), "utf8");
  assert.equal(model.match(/theorem\{/g).length, 3);
  for (const [name, label] of [["wrong-owner", "wireRejectsWrongOwner"], ["progress", "wireProgressKeepsAccounting"], ["counter-exhausted", "wireRefusesCounterOverflow"]]) {
    const start = model.indexOf(`theorem{\n  name: ${label},`);
    assert.ok(start >= 0); const proof = model.slice(start, model.indexOf("\n}\n", start));
    const value = vector(name);
    assert.ok(proof.includes(`statement: ((dispatchWork 0x${Buffer.from(value.request.slice(5)).toString("hex")}) = 0x${Buffer.from(value.response).toString("hex")})`));
    assert.ok(proof.includes("proof: reflexivity{}")); assert.ok(proof.includes("axioms: [Quot.sound, propext]"));
  }
});
