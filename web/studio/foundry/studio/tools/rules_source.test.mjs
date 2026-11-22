import assert from "node:assert/strict";
import fs from "node:fs";
import vm from "node:vm";
import { test } from "node:test";
import { saveBoundRules, readBoundRules, sameRuleReservation } from "../shell/rules-source.mjs";
const host = fs.readFileSync(new URL("../shell/host.js", import.meta.url), "utf8").replace(/\r\n/g, "\n");
const rule = { phase: "checked", specKappa: "blake3:" + "1".repeat(64), spec: { title: "rules" } };
function operation(name, end, environment) {
  return vm.runInNewContext(host.slice(host.indexOf(`async function ${name}(`), host.indexOf(end, host.indexOf(`async function ${name}(`))) + `\n${name}`, {sameRuleReservation, ...environment});
}
test("approval completion refuses a competing state instead of starting proof", async () => {
  let expected;
  const approve = operation("approveRules", "// The approve turn", { crypto: {randomUUID: () => "approval-fixture"}, rulesOf: async () => rule, nextPhase: async () => "proving", saveRules: async (_, state, prior) => { expected = prior; assert.equal(state.phase, "proving"); return false; }, now: () => "now", prism: { provingMessage() { assert.fail("must not claim approval"); } } });
  const result = await approve(1);
  assert.equal(expected, rule); assert.match(result.text, /changed/); assert.equal(result.prove, undefined);
});
test("proof completion checks the current source-bound state and its expected predecessor", async () => {
  const proving = { ...rule, phase: "proving" };
  for (const latest of [null, proving]) {
    let calls = 0, checkedPrior = false;
    const prove = operation("proveRules", '// "approve"', { rulesOf: async () => ++calls === 1 ? proving : latest, kernelReady: null, prism: { prove: async () => ({ ok: true, record: { id: "receipt" } }) }, nextPhase: async () => "proved", saveRules: async (_, state, prior) => { checkedPrior = true; assert.equal(prior, proving); assert.equal(state.phase, "proved"); return false; }, now: () => "now", put() { assert.fail("stale proof must not be recorded as accepted"); } });
    assert.equal(await prove(1, proving.specKappa), null);
    assert.equal(checkedPrior, latest !== null);
  }
});
test("stale publication completion does not promote the preview or claim current publication", async () => {
  const messages = [], shown = [], audit = [];
  const proved = { ...rule, phase: "proved" };
  const publish = operation("publishTurn", "window.__prism", {
    chatMachineOf: () => ({}), now: () => "now", saveMessages: async (_, rows) => shown.push(rows.at(-1).content), emit() {}, publishChat() {},
    rulesOf: async () => proved, handlers: new Map([["holo:publish", async () => ({ kappa: "k", url: "u", byteLength: 1024, ms: 1000 })]]), nextPhase: async () => "published",
    saveRules: async (_, state, expected) => { assert.equal(expected, proved); assert.equal(state.phase, "published"); return false; },
    put: async (...args) => audit.push(args), machineOf() { assert.fail("stale completion must not change preview"); }, publish() { assert.fail("stale completion must not change preview"); },
  });
  await publish(1, 1, messages, 1, {}, "intent");
  assert.match(shown.at(-1), /earlier rules/);
  assert.equal(audit.at(-1)[2].outcome, "stale");
});
test("a holo the device refuses publishes nothing and says why", async () => {
  const shown = [], audit = [];
  const publish = operation("publishTurn", "window.__prism", {
    chatMachineOf: () => ({}), now: () => "now", saveMessages: async (_, rows) => shown.push(rows.at(-1).content), emit() {}, publishChat() {},
    rulesOf: async () => ({ ...rule, phase: "proved" }), handlers: new Map([["holo:publish", async () => ({ decision: "Refuse", word: "vector failed" })]]),
    put: async (...args) => audit.push(args), saveRules() { assert.fail("a refused holo must not change the rules"); },
  });
  await publish(1, 1, [], 1, {}, "intent");
  assert.match(shown.at(-1), /Not published: vector failed/);
  assert.equal(audit.at(-1)[2].outcome, "refused");
});
test("imports discard untrusted proof metadata and actual host reads use source binding", () => {
  const imported = host.slice(host.indexOf('handlers.set("forge:import"'), host.indexOf("// ---- versions:"));
  assert.doesNotMatch(imported, /writes\.push\([^\n]*entry\.rules/);
  assert.match(host, /const rulesOf = async \(appId\) => readBoundRules\(await dbReady, appId, kappa\)/);
  assert.match(host, /saveBoundRules\(await dbReady, appId, state, kappa, expected, signal\)/);
  assert.doesNotMatch(host, /Nothing was published:/);
});
test("invalid state and storage errors are typed and private", async () => {
  for (const state of [null, {}, { ...rule, phase: "invented" }, { ...rule, specKappa: "prefix" + rule.specKappa }]) {
    await assert.rejects(saveBoundRules({}, 1, state, () => {}), { code: "invalidRulesState" });
  }
  await assert.rejects(readBoundRules({ transaction() { throw Error("private source"); } }, 1, () => {}), error => error.code === "rulesStorageFailed" && !error.message.includes("private source"));
});
