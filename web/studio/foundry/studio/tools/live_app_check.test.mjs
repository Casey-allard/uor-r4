// Run: node --test studio/tools/live_app_check.test.mjs
// A stateful app's module is checked at the end of its turn (shell/live/app.mjs, checkApp), in the tab: it is read
// as LexLean reads it, its first frame draws, and every theorem it states is put as a term Lean's kernel can judge.
// What the chat says about it claims no more than the check found. Nothing is sent anywhere.
import { test } from "node:test";
import assert from "node:assert/strict";
import fs from "node:fs";
import { appLive, card, checkApp, issueLine, repairPrompt, CARD } from "../shell/live/app.mjs";
import { expand } from "../shell/live/notation.mjs";

const read = (path) => fs.readFileSync(new URL(path, import.meta.url), "utf8");
const module = expand(read("notation/guests.txt")).module;
const library = { "Foundation.View.Browser.V1.Model": JSON.parse(read("../shell/live/v2/Model.json")) };
const offline = async (fn) => { const f = globalThis.fetch; let asked = 0; globalThis.fetch = async () => { asked++; throw new Error("offline"); }; try { return { result: await fn(), asked }; } finally { globalThis.fetch = f; } };

test("every notation module is checked in the tab, with no request", async () => {
  for (const name of ["guests", "notes", "orders", "model-reading", "model-chores", "model-feedback", "model-guests"]) {
    const { result, asked } = await offline(() => checkApp(expand(read(`notation/${name}.txt`)).module, library));
    assert.deepEqual({ ok: result.ok, issues: result.issues }, { ok: true, issues: [] }, name);
    assert.equal(asked, 0, name);
  }
});

test("a planted fault is refused by name, and the refusal becomes a repair request", async () => {
  const source = module.replace(/parameters: \[state : State\]/, "parameters: [state : Stat]");
  assert.notEqual(source, module, "the fault was planted");
  const bad = await checkApp(source, library);
  assert.equal(bad.ok, false);
  assert.ok(bad.issues.some((i) => /Stat/.test(i)), bad.issues.join("; "));
  assert.deepEqual((await checkApp(module.replace("statement: (", "statement: (nosuch "), library)).issues, ["guestsStartsEmpty: no definition nosuch"]);
  assert.equal((await checkApp(module.replace("statement: (", "statement: (("), library)).ok, false, "a module that does not read is refused");
  const prompt = repairPrompt(["Line 41: unknown name `gests` (LLT4001)"]);
  assert.match(prompt, /Line 41: unknown name `gests`/);
  assert.match(prompt, /whole module again, in one <prism-lexlean> block/);
  assert.equal(issueLine({ message: "no position" }), "no position");
  assert.equal(issueLine({ line: 3, message: "x", code: "C" }), "Line 3: x (C)");
});

test("the card says checked only for a checked state, lists what was refused, and never says proved", () => {
  const checked = card({ phase: "checked", issues: [] });
  assert.match(checked, /^\*\*App\*\* · checked/);
  assert.match(checked, /not proved yet/);
  const refused = card({ phase: "drafting", issues: ["Line 41: unknown name `gests` (LLT4001)"] });
  assert.match(refused, /^\*\*App\*\* · not checked/);
  assert.match(refused, /- Line 41: unknown name/);
  const unreachable = card({ phase: "drafting", issues: [], unchecked: "offline" });
  assert.ok(unreachable.startsWith(CARD) && /could not be checked just now: offline/.test(unreachable));
  for (const text of [checked, refused, unreachable]) assert.ok(!/\bproved by\b|is proved|are proved/i.test(text));
  assert.ok(!/hosted|server/i.test(card({ phase: "proving" }) + card({ phase: "proved", theorems: [1] })));
});

test("the same module under a new standing is drawn again with the new badge", async () => {
  const posted = [];
  const live = appLive({ post: (appId, m) => posted.push(m), library });
  live.push(4, module, "draft"); await live.settled(4);
  live.push(4, module, "checked"); await live.settled(4);
  assert.deepEqual(posted.map((m) => m.trust), ["draft", "checked"]);
  assert.deepEqual(posted[1].nodes, posted[0].nodes);
});
