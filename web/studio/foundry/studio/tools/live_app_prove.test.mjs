// Run: node --test studio/tools/live_app_prove.test.mjs
// Proving a stateful app's module in the tab (shell/prism.js moduleTerms and the Lean worker's `module`): the
// module's own definitions and theorems are written as Lean kernel terms, each proof applies a lemma studio/proof
// proved once (PrismLive.Tables) or is by evaluation, and the pinned Lean kernel (shell/kernel/lean) judges them
// against the shipped library. A broken app is refused by the kernel, not by this writer. Then what the chat says.
import { test } from "node:test";
import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import zlib from "node:zlib";
import { card, promisesOf, proofMessage, provingMessage, semanticData, theoremsOf } from "../shell/live/app.mjs";
import { expand } from "../shell/live/notation.mjs";
import { moduleTerms } from "../shell/prism.js";

const read = (p) => fs.readFileSync(new URL(p, import.meta.url), "utf8");
const library = JSON.parse(read("../shell/live/v2/Model.json")).declarations;
const moduleOf = (name) => expand(read(`notation/${name}.txt`)).module;
const module = moduleOf("guests");
const termsOf = (source) => moduleTerms(semanticData(source, { theorems: true }), library);

// The page's Lean worker, run here on the pinned glue, kernel and library, as the tab runs it.
const kernel = await (async () => {
  const prism = read("../shell/prism.js"), dir = new URL("../shell/kernel/lean/", import.meta.url);
  const leanWorker = new Function("return " + prism.slice(prism.indexOf("function leanWorker(self) {"), prism.indexOf("// One worker for the page")))();
  const manifest = JSON.parse(fs.readFileSync(new URL("manifest.json", dir)));
  const [glue, wasm, lib] = [manifest.glue, manifest.wasm, manifest.library].map((f) => zlib.gunzipSync(fs.readFileSync(new URL(f.file, dir))));
  const glueFile = path.join(fs.mkdtempSync(path.join(os.tmpdir(), "lean-")), "glue.mjs");
  fs.writeFileSync(glueFile, glue);
  URL.createObjectURL = () => "file://" + glueFile; URL.revokeObjectURL = () => {};
  const replies = []; let wake;
  const self = { postMessage: (m) => { replies.push(m); wake?.(); } };
  leanWorker(self);
  const ask = async (m) => { const p = new Promise((r) => (wake = r)); await self.onmessage({ data: m }); await p; return replies.pop(); };
  await ask({ op: "open", manifest, glue, wasm, library: lib });
  return (terms) => ask({ op: "module", id: 1, defs: terms.defs, theorems: terms.theorems });
})();

test("the theorems and the promises are read from the module, in order, word for word", () => {
  const theorems = theoremsOf(module), promises = promisesOf(module);
  assert.deepEqual(theorems.map((t) => t.name), ["guestsStartsEmpty", "added1NeedsName", "added1NeedsMeal", "added1StopsWhenFull", "cleared2Empties", "staleIgnored"]);
  for (const t of theorems) assert.ok(module.includes("statement: " + t.statement + ","), t.name);
  assert.equal(promises.length, theorems.length);
  assert.equal(promises[0], "Guests starts empty.");
  assert.deepEqual(semanticData(module, { theorems: true }).theorems.map((t) => t.name), theorems.map((t) => t.name));
  assert.equal(semanticData(module).theorems, undefined, "the data form LexLean reads is unchanged");
});

test("Lean's kernel, in the tab, accepts every theorem of every notation app, on the shipped axioms only", { timeout: 60000 }, async () => {
  for (const name of ["guests", "notes", "orders", "model-reading", "model-chores", "model-feedback", "model-guests"]) {
    const terms = termsOf(moduleOf(name));
    assert.deepEqual(terms.refused, [], name);
    assert.ok(terms.theorems.length >= 3 && terms.defs.every((d) => d.name.startsWith("Tab.app.")), name);
    const r = await kernel(terms);
    assert.ok(r.judged, name + ": " + JSON.stringify(r).slice(0, 300));
    for (const [i, j] of r.judged.entries()) {
      assert.ok(j.accepted, name + " " + terms.theorems[i].name + ": " + j.output.slice(-300));
      for (const a of j.axioms) assert.ok(["propext", "Quot.sound"].includes(a), a);
    }
  }
});

test("a broken app is refused by the kernel: a guard that lets an empty name in, a Clear that keeps the rows", { timeout: 60000 }, async () => {
  const judged = async (from, to) => { assert.ok(module.includes(from), from); const t = termsOf(module.replace(from, to)); const r = await kernel(t); return Object.fromEntries(t.theorems.map((x, i) => [x.name.replace("Tab.app.", ""), r.judged[i].accepted])); };
  const weak = await judged("(1 <=? (prim length", "(0 <=? (prim length");
  assert.equal(weak.added1NeedsName, false); assert.equal(weak.added1NeedsMeal, true);
  const kept = await judged("+ 1), guests := (nil Foundation.View.Browser.V1.Model::TableRow)", "+ 1), guests := (project state guests)");
  assert.equal(kept.cleared2Empties, false); assert.equal(kept.guestsStartsEmpty, true);
});

test("what the chat says: each promise beside the exact statement proved, and never more than happened", () => {
  const theorems = theoremsOf(module), promises = promisesOf(module);
  const proved = proofMessage({ ok: true, seconds: 0.2, record: {}, theorems, promises });
  assert.match(proved, /^\*\*Proved by Lean\.\*\* Lean's kernel, in this browser, accepted 6 theorems/);
  for (let i = 0; i < theorems.length; i++) assert.ok(proved.includes("- " + promises[i] + "\n  Lean proved: `" + theorems[i].statement + "`"), theorems[i].name);
  assert.match(proved, /A sentence can say more than its theorem/);
  assert.match(proved, /cannot be published yet/);
  const refused = proofMessage({ ok: false, error: "Lean's kernel refused cleared2Empties", theorems: [], promises: [] });
  assert.match(refused, /^\*\*Not proved\.\*\* Lean's kernel refused cleared2Empties\. The app is back to checked/);
  assert.match(refused, /fix the property/);
  // The kernel not running is not the app failing, and is not blamed on a property.
  const broken = proofMessage({ ok: false, error: "the proof worker did not answer", theorems: [], promises: [] });
  assert.match(broken, /could not run the proof/);
  assert.match(broken, /says nothing about the app/);
  assert.ok(!/fix the property/.test(broken) && !/Proved by Lean/.test(refused));
  assert.match(provingMessage(5), /5 properties/);
  assert.match(provingMessage(1), /1 property /);
  assert.match(card({ phase: "proving" }), /Lean is proving/);
  assert.match(card({ phase: "proved", theorems }), /proved by Lean[\s\S]*6 theorems[\s\S]*cannot be published yet/);
  for (const phase of ["drafting", "checked", "proving"]) assert.ok(!/proved by Lean/i.test(card({ phase, issues: [] })), phase);
  for (const text of [proved, refused, broken, provingMessage(2), card({ phase: "proving" })]) assert.ok(!/hosted|verifier|server/i.test(text), text);
});
