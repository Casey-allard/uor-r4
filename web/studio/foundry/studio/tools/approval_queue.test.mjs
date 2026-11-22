// One approval at a time (shell/approval-queue.mjs), and what the chat says while a model writes an app in
// the app notation (prism.js shownText). Found on the deployed site on 2026-11-12: the preview's permission
// dialog opened over the cloud request's, covered its Send button, and the request expired unanswered; and
// the chat said "0 declarations" of an app written in eight lines.
//
//   node --test tools/approval_queue.test.mjs
import test from "node:test";
import assert from "node:assert/strict";
import fs from "node:fs";
import { oneAtATime, waitingCount } from "../shell/approval-queue.mjs";
import { shownText, specGrammarCheck } from "../shell/prism.js";

const read = (p) => fs.readFileSync(new URL(p, import.meta.url), "utf8");
const deferred = () => { let resolve, reject; const promise = new Promise((a, b) => { resolve = a; reject = b; }); return { promise, resolve, reject }; };
const settle = () => new Promise((r) => setImmediate(r));

test("a question asked alone is shown at once", async () => {
  let shown = false;
  const answer = oneAtATime(() => { shown = true; return Promise.resolve("yes"); });
  assert.equal(shown, true, "the dialog is opened in the same task as the request");
  assert.equal(await answer, "yes");
});

test("a second question is not shown until the first is answered, whatever the answer", async () => {
  const first = deferred(), second = deferred(), third = deferred(), shown = [];
  const a = oneAtATime(() => { shown.push("preview"); return first.promise; });
  const b = oneAtATime(() => { shown.push("request"); return second.promise; });
  const c = oneAtATime(() => { shown.push("source"); return third.promise; });
  await settle();
  assert.deepEqual(shown, ["preview"]);
  assert.equal(waitingCount(), 2);
  // A refusal frees the next question as an approval does.
  first.reject(new Error("denied"));
  await assert.rejects(a, /denied/);
  await settle();
  assert.deepEqual(shown, ["preview", "request"]);
  second.resolve("sent");
  assert.equal(await b, "sent");
  await settle();
  assert.deepEqual(shown, ["preview", "request", "source"]);
  third.resolve("allowed");
  assert.equal(await c, "allowed");
  assert.equal(waitingCount(), 0);
});

test("a question that cannot be shown rejects and does not hold the others", async () => {
  await assert.rejects(oneAtATime(() => { throw new Error("no document"); }), /no document/);
  assert.equal(await oneAtATime(() => Promise.resolve("next")), "next");
});

test("the three approvals of a build are each asked in their turn", () => {
  for (const [file, name] of [["preview-permissions.mjs", "authorizePreview"]]) {
    const source = read("../shell/" + file);
    assert.ok(source.includes(`export function ${name}(input, options) { return oneAtATime(() => ${name}Now(input, options)); }`), file + " opens its dialog outside the queue");
    assert.equal((source.match(/\.showModal\(\)/g) || []).length >= 1, true);
    // No other export of the file opens the same dialog around the queue.
    assert.equal(new RegExp(`export (async )?function ${name}Now`).test(source), false);
  }
});

test("the chat counts an app's lines while a model writes the app notation, and declarations for a module", () => {
  const lines = 'app Reading list\ntext title "Title" max 120\ntable books "Reading list": title\naction "Add": add to books\n';
  assert.match(shownText("<prism-lexlean>\napp Reading list\ntext title \"Title\" max 120\n"), /Writing the app: 2 lines so far…/);
  assert.match(shownText("<prism-lexlean>\napp Reading list\n"), /Writing the app: 1 line so far…/);
  const done = shownText("Here it is.\n<prism-lexlean>\n" + lines + "</prism-lexlean>\nDone.");
  assert.match(done, /The app, written in 4 lines\. Forge is writing its LexLean module from them\./);
  assert.equal(/0 declarations/.test(done), false);
  assert.equal(done.includes("table books"), false, "the lines themselves are not shown as chat text");
  // A card, once there is one, replaces the line as before.
  assert.match(shownText("<prism-lexlean>\n" + lines + "</prism-lexlean>", "**App** · checked"), /\*\*App\*\* · checked/);
  // A module in full LexLean is still counted in declarations.
  assert.match(shownText("<prism-lexlean>\n\\begin{lexlean}{App}\nstructure{ name: State }\ndefinition{ name: initial }\n"), /Writing the app's LexLean module: 2 declarations so far…/);
});

// The spec grammar a device rules turn is held to (prism.js specGrammarCheck).
const good = '<prism-spec>{"title":"Parking fee tier","purpose":"Decide the fee tier from hours parked and vehicle.","fields":[{"name":"hours","kind":"number","min":0,"max":24},{"name":"vehicle","kind":"choice","values":["car","motorcycle","truck"]}],"outcomes":["free","standard","premium"],"rules":[{"when":[{"field":"hours","op":"=","value":0}],"then":"free"},{"when":[{"field":"vehicle","op":"is not","value":"truck"},{"field":"hours","op":">","value":12}],"then":"premium"}],"otherwise":"standard","properties":[{"kind":"never","when":[{"field":"hours","op":"=","value":0}],"then":"premium"}],"examples":[{"request":"0 truck","response":"free"},{"request":"13 car","response":"premium"}]}</prism-spec>';
test("a whole spec is done and every prefix of it is partial", () => {
assert.equal(specGrammarCheck(good), "done");
for (let k = 0; k < good.length; k++) assert.equal(specGrammarCheck(good.slice(0, k)), "partial", "prefix " + k);
const cut = (s, from, to, by) => s.slice(0, s.indexOf(from)) + by + s.slice(s.indexOf(to));
assert.equal(specGrammarCheck(cut(good, ',"properties":', ',"examples":', ',"properties":[]')), "done");
assert.equal(specGrammarCheck(cut(good, ',"examples":', "}</prism-spec>", ',"examples":[]')), "done");
});
test("form the checker would refuse cannot be written", () => {
const bad = [
  good.replace('"op":"=","value":0}],"then":"free"', '"op":"is","value":0}],"then":"free"'),
  good.replace('"field":"vehicle"', '"field":"car"'),
  good.replace('"value":"truck"}', '"value":"bus"}'),
  good.replace('"then":"free"', '"then":"cheap"'),               // not a declared outcome
  good.replace('"otherwise":"standard"', '"otherwise":"never"'),
  good.replace('"request":"0 truck"', '"request":"0 bus"'),
  good.replace('"request":"0 truck"', '"request":"truck 0"'),
  good.replace('"max":24', '"max":"24"'),
  good.replace('"title":', ' "title":'),
];
for (const b of bad) assert.equal(specGrammarCheck(b), "fail", b.slice(0, 160));
});
