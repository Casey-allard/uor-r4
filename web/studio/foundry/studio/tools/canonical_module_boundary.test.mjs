import assert from "node:assert/strict";
import fs from "node:fs";
import { test } from "node:test";
import { readLexLean, readLexLeanPartial, specFromTurn, canonicalSource } from "../shell/live/lexlean.mjs";
const source = fs.readFileSync(new URL("./fixtures/live/verdant.lex.tex", import.meta.url), "utf8");
const insert = text => source.replace("\\end{semanticmodule}", text + "\n\\end{semanticmodule}");
const refused = text => assert.throws(() => readLexLean(text), e => e.code === "invalidCanonicalModule" && e.message === "The canonical decision module is invalid or unsupported");
test("the supported complete and proof-free modules reconstruct all declarations", () => {
  const header = readLexLeanPartial("\\begin{lexlean}{VerdantResonance}\n");
  assert.equal(header.complete, false); assert.equal(header.spec.title, "Verdant Resonance");
  assert.deepEqual(header.spec.rules, []); assert.equal(header.spec.otherwise, undefined);
  assert.equal(canonicalSource(readLexLean(source)).source, source);
  const bare = source.replace(/\n\ntheorem\{[\s\S]*?\n\}(?=\n)/g, "");
  assert.equal(canonicalSource(readLexLean(bare)).source, source);
  assert.equal(readLexLean(source.replaceAll("parameters: []", "parameters: [ ]")).rules.length, 35);
});
test("duplicate names and unrepresented definitions cannot disappear", () => {
  refused(insert("definition{name: answerSanctuaryChorus, parameters: [], result: Nat, body: 0}"));
  refused(insert("definition{name: secretDecision, parameters: [], result: Nat, body: 99}"));
  refused(insert("structure{name: Extra, type_parameters: [], fields: []}"));
  refused(source + "private-extra-program");
});
test("unsupported request fields and changed projections refuse", () => {
  assert.ok(source.includes("{name: biome, type: Nat}"));
  refused(source.replace("{name: biome, type: Nat}", "{name: biome, type: Nat}, {name: hidden, type: Bool}"));
  refused(source.replace("{name: biome, type: Nat}", "{name: biome, type: Nat}, {name: biome, type: Nat}"));
  refused(source.replace("project request biome", "project other biome"));
});
test("extra bounds and altered rule arguments cannot simplify into success", () => {
  const start = source.indexOf("name: inBounds,"); const at = source.indexOf("body: ", start) + 6;
  const end = source.indexOf("\n}", at); const body = source.slice(at, end);
  refused(source.slice(0, at) + "(" + body + " && ((project request proximity) =? 0))" + source.slice(end));
  refused(source.replace(/(if \(\w+) request\)/, "$1 other)"));
});
test("input and nesting bounds refuse before unbounded parsing", () => {
  assert.equal(readLexLeanPartial(source + " ".repeat(1024 * 1024)).code, "invalidCanonicalModule");
  for (const value of [null, {}, source + " ".repeat(1024 * 1024 + 1), "(".repeat(257) + source, insert("\n" + Array.from({length:4097}, (_, i) => `definition{name: extra${i}, parameters: [], result: Nat, body: 0}`).join("\n"))]) refused(value);
});
test("turn errors are typed and do not expose supplied source diagnostics", () => {
  const result = specFromTurn("<prism-lexlean>private-source-secret</prism-lexlean>");
  assert.equal(result.code, "invalidCanonicalModule");
  assert.equal(result.error, "The canonical decision module is invalid or unsupported");
  assert.ok(!JSON.stringify(result).includes("private-source-secret"));
  const model = fs.readFileSync(new URL("../../app/src/FoundrySpecification.lex.tex", import.meta.url), "utf8");
  assert.match(model, /name: canonicalModuleBoundary/); assert.match(model, /name: invalidCanonicalModule/);
});
