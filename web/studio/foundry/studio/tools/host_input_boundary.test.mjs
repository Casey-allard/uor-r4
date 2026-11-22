import {commandRegistry, invokeBoundCommand} from "../shell/command-registry.mjs";
import assert from "node:assert/strict";
import fs from "node:fs";
import vm from "node:vm";
import { test } from "node:test";
import { invokeHost, unwrapHost } from "../shell/host-boundary.mjs";
const channel = "create-app";
const output = { safeParse: () => ({ success: true }) };
const input = { safeParse: data => ({ success: true, data }) };
const invoke = (args, handler, schema = input) => invokeHost(channel, args, handler, output, { schema });
const refused = (envelope, code) => { assert.equal(envelope.ok, false); assert.equal(envelope.error.code, code); assert.equal(envelope.error.kind, code === "invalidInput" ? "validation" : "internal"); assert.equal(JSON.stringify(envelope).includes("private"), false); };
test("input schema refusal prevents all handler effects", async () => {
  let effects = 0;
  for (const result of [undefined, null, {}, { success: false, error: "private" }, { success: "true" }]) refused(await invoke(["private"], () => effects++, { safeParse: () => result }), "invalidInput");
  assert.equal(effects, 0);
});
test("missing or broken input contracts refuse before effects", async () => {
  const handler = () => assert.fail("unexpected effect");
  for (const schema of [undefined, null, {}, { safeParse: true }]) refused(await invokeHost(channel, [], handler, output, { schema }), "inputContractUnavailable");
  refused(await invoke([], handler, { safeParse() { throw Error("private"); } }), "inputValidatorFailed");
  refused(await invoke([], handler, { get safeParse() { throw Error("private"); } }), "inputValidatorFailed");
  refused(await invoke([], handler, { safeParse: () => ({ success: true }) }), "inputValidatorFailed");
  refused(await invoke([], handler, { safeParse: () => ({ success: true, get data() { throw Error("private"); } }) }), "inputValidatorFailed");
});
test("imported commands accept one argument or void and reject extra arguments", async () => {
  for (const args of [null, {}, [1, 2]]) refused(await invoke(args, () => assert.fail("unexpected effect")), "invalidInput");
  assert.equal(unwrapHost(await invoke([], value => value)), undefined);
  assert.deepEqual(unwrapHost(await invoke([{ id: 7 }], value => value)), { id: 7 });
});
test("parsed defaults and transformations are the handler input", async () => {
  const parsed = { id: 3, enabled: true };
  assert.equal(unwrapHost(await invoke([{}], value => { assert.equal(value, parsed); return "done"; }, { safeParse: () => ({ success: true, data: parsed }) })), "done");
});
test("caller mutation cannot change an admitted pending command", async () => {
  const value = { id: 1, nested: { name: "original" } }; let release;
  const pending = invoke([value], async admitted => { await new Promise(resolve => { release = resolve; }); return admitted.nested.name; });
  value.nested.name = "changed"; release(); assert.equal(unwrapHost(await pending), "original");
  refused(await invoke([() => "private"], () => assert.fail("unexpected effect")), "invalidInput");
});
test("unsupported and legacy commands keep their declared boundary semantics", async () => {
  const unknown = await invokeHost(channel, [], undefined, output, { schema: input });
  assert.equal(unknown.error.code, "unsupportedChannel");
  assert.equal(unwrapHost(await invokeHost("legacy", [1, 2], (a, b) => a + b)), 3);
});
test("output validation still runs after admitted input", async () => {
  const result = await invokeHost(channel, [1], () => ({}), { safeParse: () => ({ success: false }) }, { schema: input });
  assert.equal(result.error.code, "invalidResponse");
});
test("the shipped host routes imported commands through input validation", async () => {
  const source = fs.readFileSync(new URL("../shell/host.js", import.meta.url), "utf8");
  const start = source.indexOf("function dispatch(channel, args) {"), end = source.indexOf("\n}\n", start) + 3;
  assert.ok(start >= 0 && end > start);
  let effects = 0;
  const window = { __contracts: { [channel]: output }, __inputContracts: { [channel]: { safeParse: () => ({ success: false }) } } };
  const dispatch = vm.runInNewContext(source.slice(start, end) + "\ndispatch", { window, handlers: new Map([[channel, () => effects++]]), seen: new Map(), invokeHost, commandRegistry, invokeBoundCommand, windowRecoveryScopes:()=>[],windowCoordination:{invalidate:async()=>{}} });
  refused(await dispatch(channel, ["private"]), "invalidInput");
  delete window.__inputContracts[channel]; refused(await dispatch(channel, []), "inputContractUnavailable");
  assert.equal(effects, 0);
  window.__inputContracts[channel] = input;
  assert.equal(unwrapHost(await dispatch(channel, [1])), 0); assert.equal(effects, 1);
});
