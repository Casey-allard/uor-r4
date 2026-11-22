// The app notation (shell/live/notation.mjs): a few lines a model writes for an app that keeps state, and
// the canonical LexLean module they stand for. Claims held here: the same lines are always the same module;
// the module is in the app shape with a theorem for every promise; lines that do not read are refused with
// the line and the reason; and a chat turn written in the notation is the module to everything after it.
// The data form is read, run and checked in the tab.
//
//   node --test tools/live_notation.test.mjs
import test from "node:test";
import assert from "node:assert/strict";
import fs from "node:fs";
import crypto from "node:crypto";
import { expand, parse, isNotation, PROMPT } from "../shell/live/notation.mjs";
import { moduleFromTurn, isAppShape, splitDeclarations, wholePrefix, theoremsOf, promisesOf, moduleContext, notationRepairPrompt, notationCard, checkApp, semanticData, createApp, PROMPT as APP_PROMPT } from "../shell/live/app.mjs";

const read = (p) => fs.readFileSync(new URL(p, import.meta.url), "utf8");
// Each fixture, with the SHA-256 of the module it stands for. The four model-* files are what GLM 5.3 Flash
// wrote through the site's relay on 2026-11-12 from these instructions; every module here passed the hosted
// checker, and Lean verified each one in the locked SDK image (VERIFICATION.md, row Notation).
const FIXTURES = {
  "guests": "24a6e52e2079cd3e", "notes": "d1c16f95a4f2a8ae", "orders": "04d0d19a4d81ae73",
  "model-reading": "039a6a1caa70c20d", "model-chores": "7b9d05f81fb79a15", "model-feedback": "3efd6e69cde2d9a8", "model-guests": "8a87527b097b8003",
};
const lines = (name) => read(`notation/${name}.txt`);

test("the same lines are always the same module, byte for byte", () => {
  for (const [name, digest] of Object.entries(FIXTURES)) {
    const a = expand(lines(name)), b = expand(lines(name));
    assert.equal(a.ok, true, name + ": " + JSON.stringify(a.issues));
    assert.equal(a.module, b.module);
    assert.equal(crypto.createHash("sha256").update(a.module).digest("hex").slice(0, 16), digest, name + " expands to another module than the one that was checked and proved");
    // Line endings and blank lines are not part of what the lines say.
    assert.equal(expand(lines(name).replace(/\n/g, "\r\n\r\n")).module, a.module);
  }
});

test("the module is in the app shape, whole, with one theorem for every promise", () => {
  for (const name of Object.keys(FIXTURES)) {
    const { module, promises } = expand(lines(name));
    assert.equal(isAppShape(module), true);
    const names = splitDeclarations(module).declarations.map((d) => d.name);
    for (const needed of ["State", "catalogue", "initial", "view", "guarded", "step", "promises"]) assert.ok(names.includes(needed), name + " lacks " + needed);
    assert.equal(new Set(names).size, names.length, name + " declares a name twice");
    assert.equal(wholePrefix(module).ready, true);
    assert.equal(theoremsOf(module).length, promises.length);
    assert.deepEqual(promisesOf(module), promises);
    assert.ok(promises.length >= 3 && promises.includes("A press from an out-of-date screen never changes anything."));
    // A linear-arithmetic proof lists its definitions in ascending order, each once: the checker refuses otherwise.
    for (const m of module.matchAll(/linear_arithmetic\{definitions: \[([^\]]*)\]\}/g)) { const listed = m[1].split(", "); assert.deepEqual(listed, [...new Set(listed)].sort((a, b) => (a < b ? -1 : a > b ? 1 : 0))); }
  }
});

test("what the lines say is what the module holds", () => {
  const { module } = expand(lines("orders"));
  // every label, in the catalogue; every text's limit, in its guard; sixteen rows, in every add
  for (const label of ["Orders", "Customer", "Size", "Small", "Large", "Milk", "Oat", "Regulars", "Orders today", "Order", "Remember", "Clear orders"]) assert.ok(module.includes(`"${label}"`), label);
  assert.match(module, /name: customerFits,[\s\S]*?<=\? 30\)\)/);
  assert.equal((module.match(/<\? 16\)/g) || []).length, 2, "each of the two adds stops at sixteen rows");
  assert.match(module, /name: cleared3,[\s\S]*?orders := \(nil Foundation\.View\.Browser\.V1\.Model::TableRow\), ordersCount := 0, regulars := \(project state regulars\)/);
  // an add changes only its own table
  assert.match(module, /name: added2,[\s\S]*?orders := \(project state orders\), ordersCount := \(project state ordersCount\), regulars := \(cons /);
});

test("lines that do not read are refused, each with its line and the reason", () => {
  const refuse = (text, pattern) => { const e = expand(text); assert.equal(e.ok, false, text); assert.equal(e.module, ""); assert.ok(e.issues.some((i) => pattern.test(i.message) && i.line >= 1), JSON.stringify(e.issues)); };
  refuse('text name "Name"\n', /first line names the app/);
  refuse('app A\ntext Name "Name"\ntable t "T": Name\naction "Add": add to t\n', /is not a name/);
  refuse('app A\ntext state "Name"\ntable t "T": state\naction "Add": add to t\n', /reserved word/);
  refuse('app A\ntext a "A"\ntext a "B"\ntable t "T": a\naction "Add": add to t\n', /already used/);
  refuse('app A\ntext a "A" max 0\ntable t "T": a\naction "Add": add to t\n', /between 1 and 200/);
  refuse('app A\nchoice c "C": "One"\ntable t "T": c\naction "Add": add to t\n', /between 2 and 8 options/);
  refuse('app A\ntext a "A"\ntable t "T": a, b\naction "Add": add to t\n', /"b" is not a text or a choice/);
  refuse('app A\ntext a "A"\ntable t "T": a\naction "Add": add to u\n', /"u" is not a table/);
  refuse('app A\ntext a "A"\ntable t "T": a\n', /nothing adds to the table "t"/);
  refuse('app A\ntext a "A"\n', /no table/);
  refuse('app A\ntext a "A"\ntable t "T": a\naction "Add": add to t\nmake it nice\n', /is not one of/);
  refuse('app A\ntext a "A \\" B"\ntable t "T": a\naction "Add": add to t\n', /is not one of/);
  refuse('app A\ntext a "A"\ntable rows "Rows": a\ntable rowsCount "More": a\naction "Add": add to rows\naction "More": add to rowsCount\n', /generated table count/);
  refuse('app A\ntext a "A"\ntable rowsCount "More": a\ntable rows "Rows": a\naction "More": add to rowsCount\naction "Add": add to rows\n', /generated table count/);
  refuse('app A\ntext a "A"\ntable rows "Rows": ,\naction "Add": add to rows\n', /between 1 and 6 columns/);
  refuse('app A\ntext a "A"\ntable rows "Rows": a,\naction "Add": add to rows\n', /between 1 and 6 columns/);
  // No label can leave its string: a quote or a backslash never reaches the module.
  assert.equal(parse('app A "quoted"\n').issues.length > 0, true);
});

test("a chat turn written in the notation is its module, while it streams and when it ends", () => {
  const text = lines("guests");
  const turn = "Here is your app.\n<prism-lexlean>\n" + text + "</prism-lexlean>\nDone.";
  const whole = moduleFromTurn(turn);
  assert.equal(whole.closed, true);
  assert.equal(whole.text, expand(text).module);
  assert.equal(whole.notation, text);
  assert.deepEqual(whole.issues, []);
  // Every prefix of a streaming turn is a module in the app shape or nothing yet, and never throws.
  const open = "<prism-lexlean>\n" + text;
  let shown = 0;
  for (let cut = 16; cut <= open.length; cut += 7) {
    const part = moduleFromTurn(open.slice(0, cut));
    assert.equal(part.closed, false);
    // Until the first line names the app, the block is not yet known to be in the notation.
    if (part.notation && part.text) { assert.equal(isAppShape(part.text), true); assert.equal(wholePrefix(part.text).ready, true); shown++; }
  }
  assert.ok(shown > 20, "the preview has a module to run from the first whole line on");
  // A turn that ends with a line that does not read is no module, and says why.
  const bad = moduleFromTurn("<prism-lexlean>\napp A\ntext a \"A\"\ntable t \"T\": a\nplease add more\n</prism-lexlean>");
  assert.equal(bad.text, "");
  assert.equal(bad.issues[0].line, 4);
  assert.match(notationRepairPrompt(bad.issues), /Line 4: this line is not one of/);
  assert.match(notationCard(bad.issues), /nothing was changed/);
  // A module written in full LexLean is still read as before.
  const full = expand(text).module;
  assert.deepEqual(moduleFromTurn("<prism-lexlean>\n" + full + "</prism-lexlean>"), { text: "\n" + full, closed: true });
  assert.equal(isNotation(full), false);
});

test("a model is shown the lines only while they stand for the stored module", () => {
  const text = lines("notes"), module = expand(text).module;
  assert.match(moduleContext(module, text), /current lines, in the app notation:\n<prism-lexlean>\napp Notes\n/);
  // The module was edited since, or the lines are another app's: the module itself is shown.
  assert.match(moduleContext(module.replace('"Notes"', '"My notes"'), text), /current LexLean module/);
  assert.match(moduleContext(module, lines("guests")), /current LexLean module/);
  assert.match(moduleContext(module), /current LexLean module/);
});

test("the instructions teach the notation, and their example reads", () => {
  assert.ok(APP_PROMPT.includes(PROMPT));
  const example = /Example: <prism-lexlean>\n([\s\S]*?)<\/prism-lexlean>/.exec(PROMPT)[1];
  assert.equal(expand(example).ok, true);
  assert.ok(APP_PROMPT.length < 2500, "the instructions for a stateful app stay short: " + APP_PROMPT.length);
});

// Each fixture's data form as LexLean 0.4.0 itself reads it (PrismPM f4656641, the semantic snapshot's App module
// without its theorems), as a SHA-256 of its keys in sorted order: the tab reads the same data form, no server.
const DATA = { "guests": "d2e7fb0a560bdc05", "notes": "22f327367fa623d4", "orders": "a77dca84258cec2a", "model-reading": "b749c65d4cf62408", "model-chores": "27fe555f5991c062", "model-feedback": "b6421110ec202324", "model-guests": "731af3b4c7cc2dbc" };
const canon = (x) => JSON.stringify(x, (k, v) => (v && typeof v === "object" && !Array.isArray(v) ? Object.fromEntries(Object.keys(v).sort().map((key) => [key, v[key]])) : v));
test("every module is read in the tab as LexLean reads it, and runs: a row is added, a bad one is refused, a clear empties", async () => {
  const library = { "Foundation.View.Browser.V1.Model": JSON.parse(read("../shell/live/v2/Model.json")) };
  for (const name of Object.keys(FIXTURES)) {
    const { module } = expand(lines(name));
    assert.deepEqual((({ ok, issues }) => ({ ok, issues }))(await checkApp(module, library)), { ok: true, issues: [] }, name);
    const data = semanticData(module);
    assert.equal(crypto.createHash("sha256").update(canon(data)).digest("hex").slice(0, 16), DATA[name], name + " is read otherwise than LexLean reads it");
    const app = createApp(data, library);
    const numbered = app.frame().nodes.map((n, i) => ({ ...n, node: i + 1 }));
    const fields = numbered.filter((n) => n.kind === "Input" || n.kind === "Select");
    const actions = numbered.filter((n) => n.kind === "Action");
    const rows = (frame) => frame.nodes.filter((n) => n.kind === "Table").map((t) => t.rows.length);
    const filled = fields.map((n) => (n.kind === "Input" ? { node: n.node, text: "Ada" } : { node: n.node, choice: 1 }));
    const one = app.act(actions[0].action, filled);
    assert.equal(one.revision, "1", name);
    assert.equal(rows(one)[0], 1, name);
    // An empty text, or no choice, changes nothing.
    const refused = app.act(actions[0].action, fields.map((n) => (n.kind === "Input" ? { node: n.node, text: "" } : { node: n.node, choice: 0 })));
    assert.equal(refused.revision, "1", name);
    // Sixteen rows is the most a table holds.
    let last = one; for (let i = 0; i < 20; i++) last = app.act(actions[0].action, filled);
    assert.equal(rows(last)[0], 16, name);
    const clear = actions.find((a) => (a.binds || []).length === 0);
    if (clear) assert.equal(rows(app.act(clear.action, []))[0], 0, name);
  }
});
