import {commandRegistry, invokeBoundCommand} from "../shell/command-registry.mjs";
import assert from "node:assert/strict";
import { test } from "node:test";
import fs from "node:fs";
import vm from "node:vm";
import { invokeHost, unwrapHost } from "../shell/host-boundary.mjs";
const model = fs.readFileSync(new URL("../../app/src/FoundrySpecification.lex.tex", import.meta.url), "utf8");
const host = fs.readFileSync(new URL("../shell/host.js", import.meta.url), "utf8");
const start = model.indexOf("  name: unavailableBrowserQueries,");
assert.ok(start > 0);
const declaration = model.slice(start, model.indexOf("\n}\n", start));
const channels = [...declaration.matchAll(/\(cons "([^"]+)"/g)].map(match => match[1]);
test("declared unavailable queries cannot install fabricated success handlers", () => {
  assert.equal(channels.length, 29); assert.equal(new Set(channels).size, 29);
  const scope = JSON.parse(fs.readFileSync(new URL("../model/host-read-scope.json", import.meta.url), "utf8"));
  assert.equal(channels.filter(channel => scope.unavailable.includes(channel)).length, 27);
  assert.ok(channels.every(channel => !scope.required.includes(channel)));
  for (const channel of channels) {
    assert.ok(!host.includes(JSON.stringify(channel)) && !host.includes("'" + channel + "'"), `unavailable query has a host registration: ${channel}`);
  }
});
test("actual dispatch preserves unsupported failure for every excluded query", async () => {
  const begin = host.indexOf("function dispatch(channel, args) {");
  const end = host.indexOf("\n}\n", begin) + 3;
  assert.ok(begin > 0 && end > begin);
  const output = { safeParse() { assert.fail("unsupported response cannot validate"); } };
  const input = { safeParse() { assert.fail("unsupported input cannot validate"); } };
  const window = { __contracts: Object.fromEntries(channels.map(channel => [channel, output])), __inputContracts: Object.fromEntries(channels.map(channel => [channel, input])) };
  const dispatch = vm.runInNewContext(host.slice(begin, end) + "\ndispatch", { window, handlers: new Map(), seen: new Map(), invokeHost, commandRegistry, invokeBoundCommand });
  for (const channel of channels) {
    const result = await dispatch(channel, [{ private: "not surfaced" }]);
    assert.equal(result.ok, false); assert.equal(result.error.code, "unsupportedChannel"); assert.equal(result.error.kind, "not_found");
    assert.ok(!JSON.stringify(result).includes("not surfaced"));
    assert.throws(() => unwrapHost(result), error => error.code === "unsupportedChannel");
  }
});
