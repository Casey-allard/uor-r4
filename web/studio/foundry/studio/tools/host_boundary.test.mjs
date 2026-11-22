import {commandRegistry, invokeBoundCommand} from "../shell/command-registry.mjs";
import assert from "node:assert/strict";
import fs from "node:fs";
import vm from "node:vm";
import { test } from "node:test";
import { invokeHost, unwrapHost } from "../shell/host-boundary.mjs";

const arraySchema = { safeParse: (value) => ({ success: Array.isArray(value) }) };
const channel = "list-apps";
function refused(envelope, code) {
  assert.equal(envelope.__prismIpcEnvelope, "prism-ipc-envelope-v1");
  assert.equal(envelope.ok, false);
  assert.equal(Object.hasOwn(envelope, "value"), false);
  assert.equal(envelope.error.code, code);
  assert.equal(envelope.error.kind, code === "unsupportedChannel" ? "not_found" : "internal");
  assert.throws(() => unwrapHost(envelope), (error) => error.code === code && error.kind === envelope.error.kind);
}

test("unsupported channels refuse even when every empty value satisfies the schema", async () => {
  for (const handler of [undefined, null, false, {}]) {
    let parsed = false;
    refused(await invokeHost(channel, [], handler, { safeParse() { parsed = true; return { success: true }; } }), "unsupportedChannel");
    assert.equal(parsed, false);
  }
});
test("valid async output preserves identity and handler arguments", async () => {
  const value = ["actual result"];
  let calls = 0;
  const result = await invokeHost(channel, [2, 3], async (...args) => { calls++; assert.deepEqual(args, [2, 3]); return value; }, arraySchema);
  assert.equal(calls, 1);
  assert.equal(result.ok, true);
  assert.equal(unwrapHost(result), value);
});
test("a real handler may successfully return an empty list", async () => {
  const value = [];
  assert.equal(unwrapHost(await invokeHost(channel, [], () => value, arraySchema)), value);
});
test("invalid array, scalar, null and undefined outputs refuse without coercion", async () => {
  for (const value of [[], {}, 42, null, undefined]) {
    refused(await invokeHost(channel, [], () => value, { safeParse: () => ({ success: false }) }), "invalidResponse");
  }
});
test("malformed validator results do not admit success", async () => {
  for (const result of [undefined, null, {}, { success: "true" }, { success: 1 }]) {
    refused(await invokeHost(channel, [], () => [], { safeParse: () => result }), "invalidResponse");
  }
});
test("throwing and malformed validators return typed failures", async () => {
  for (const schema of [{}, { safeParse() { throw Error("private response"); } }]) {
    refused(await invokeHost(channel, [], () => [], schema), "validatorFailed");
  }
});
test("sync and async handler failures never reach validation", async () => {
  for (const handler of [() => { throw Error("private credential"); }, async () => { throw Error("private credential"); }]) {
    const result = await invokeHost(channel, ["private argument"], handler, { safeParse() { assert.fail("must not validate a failed operation"); } });
    refused(result, "handlerFailed");
    assert.equal(JSON.stringify(result).includes("private"), false);
  }
});
test("a schema's transformed data cannot replace handler output", async () => {
  const original = { value: "original" };
  const result = await invokeHost(channel, [], () => original, { safeParse: () => ({ success: true, data: "replacement" }) });
  assert.equal(unwrapHost(result), original);
});
test("legacy handlers without a schema preserve output without claiming validation", async () => {
  for (const value of [undefined, null, [], {}]) {
    assert.equal(unwrapHost(await invokeHost(channel, [], () => value, undefined)), value);
  }
});
test("the shipped host dispatch uses the real boundary and both entrypoints preserve failure", async () => {
  const source = fs.readFileSync(new URL("../shell/host.js", import.meta.url), "utf8");
  assert.match(source, /import \{ invokeHost, unwrapHost \} from "\.\/host-boundary\.mjs";/);
  assert.match(source, /invokeEnvelope: \(channel, \.\.\.args\) => dispatch\(channel, args\)/);
  assert.match(source, /invoke: \(channel, \.\.\.args\) => dispatch\(channel, args\)\.then\(unwrapHost\)/);
  assert.doesNotMatch(source, /emptyAnswer|const EMPTY/);
  const begin = source.indexOf("function dispatch(channel, args) {");
  const end = source.indexOf("\n}\n", begin) + 3;
  assert.ok(begin >= 0 && end > begin);
  const handlers = new Map();
  const seen = new Map();
  const dispatch = vm.runInNewContext(source.slice(begin, end) + "\ndispatch", { handlers, seen, invokeHost, commandRegistry, invokeBoundCommand, window: { __contracts: { [channel]: arraySchema }, __inputContracts: { [channel]: { safeParse: data => ({ success: true, data }) } } } });
  refused(await dispatch(channel, []), "commandBindingMismatch");
  handlers.set(channel, () => ({}));
  refused(await dispatch(channel, []), "invalidResponse");
  handlers.set(channel, () => [1]);
  assert.deepEqual(unwrapHost(await dispatch(channel, [])), [1]);
  assert.equal(seen.get(channel), 3);
});
