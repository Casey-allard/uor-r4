// The Table gate: the decision table answers exactly as the interpreter does, on every request; it checks
// every guarantee against every request; it counts what an edit changes, answer by answer; and it is fast
// enough to rebuild on every edit.
//
//   node tools/live_table.mjs --check
//
// Claims (level build: validated here against the interpreter, prism.js's prismAnswer, as oracle):
//   agreement   every cell of the RSVP fixture's table decides as the interpreter does
//   guarantees  the fixture's guarantees hold on every request; a guarantee the rules break is refused with
//               the first request that breaks it, and the interpreter agrees with that counterexample
//   diff        the count of answers an edit changes is the interpreter's count, request by request
//   bounds      a field with no maximum, or more than 4,194,304 requests, is not tabulated, and says why
//   budget      a 1,000,000-request table builds in under 50 ms (median of 5, this machine)
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { performance } from "node:perf_hooks";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const prism = await import(new URL("../shell/prism.js", import.meta.url));
const live = await import(new URL("../shell/live/project.mjs", import.meta.url));
const { tableKernel } = await import(new URL("../shell/live/table.mjs", import.meta.url));
const K = tableKernel();
const { prismAnswer } = new Function(prism.INTERPRETER + "\nreturn { prismAnswer };")();

let failed = 0;
const claim = (name, ok, detail = "") => { console.log(`${ok ? "ok  " : "FAIL"} ${name}${detail ? " -- " + detail : ""}`); if (!ok) failed++; };
const fixture = JSON.parse(fs.readFileSync(path.join(root, "tools/fixtures/live/rsvp.json"), "utf8"));
const logicOf = (spec) => live.project(spec).logic;
const answer = (logic, line) => prismAnswer({ fields: logic.fields, rules: logic.rules, otherwise: logic.otherwise }, line);
const outcome = (logic, k) => (k < logic.rules.length ? logic.rules[k].then : logic.otherwise);

// agreement
const logic = logicOf(fixture);
const table = K.build(logic);
let disagree = 0;
for (let r = 0; r < table.size; r++) if (answer(logic, K.requestOf(table, r)) !== outcome(logic, table.cells[r])) disagree++;
claim("agreement: every request", table.size > 0 && disagree === 0, `${table.size - disagree} of ${table.size} requests`);
const values = { attending: "maybe", party: "3", seats: "2" };
claim("agreement: one read", outcome(logic, table.cells[K.cellOf(logic, table, values)]) === answer(logic, "maybe 3 2") && K.cellOf(logic, table, { ...values, party: "9" }) === -1, "maybe 3 2, and party 9 is not a request");

// guarantees
const checks = K.check(logic, table);
claim("guarantees: hold on every request", checks.length === fixture.properties.length && checks.every((c) => c.ok), checks.map((c) => `${c.covered} covered`).join(", "));
const broken = structuredClone(fixture);
broken.properties.push({ kind: "never", when: [{ field: "attending", op: "is", value: "maybe" }], then: "waitlist", why: "planted" });
const bl = logicOf(broken), bc = K.check(bl, K.build(bl)).at(-1);
claim("guarantees: a broken one is refused", !bc.ok && answer(bl, bc.request) === "waitlist" && bc.request.startsWith("maybe"), bc.ok ? "not refused" : `\`${bc.request}\` answers ${bc.outcome}, by rule ${bc.rule + 1}`);

// diff
const edited = structuredClone(fixture); edited.rules[2].then = "none";
const el = logicOf(edited), et = K.build(el);
const d = K.diff(logic, table, el, et);
let truth = 0;
for (let r = 0; r < table.size; r++) { const line = K.requestOf(table, r); if (answer(logic, line) !== answer(el, line)) truth++; }
claim("diff: counts every changed answer", d && d.changed === truth && d.samples.every((s) => answer(logic, s.request) === s.before && answer(el, s.request) === s.after), d ? `${d.changed} of ${d.of} answers change` : "no diff");

// bounds
const open = structuredClone(fixture); delete open.fields[2].max;
const huge = structuredClone(fixture); huge.fields[2].max = 4000000;
claim("bounds: not tabulated, with a reason", !!K.build(logicOf(open)).reason && !!K.build(logicOf(huge)).reason, `${K.build(logicOf(open)).reason}; ${K.build(logicOf(huge)).reason}`);

// budget: 10 x 1000 x 100 = 1,000,000 requests, four rules
const big = { title: "Budget", purpose: "Timing.", outcomes: ["a", "b", "c"], otherwise: "c",
  fields: [{ name: "kind", kind: "choice", values: ["k0", "k1", "k2", "k3", "k4", "k5", "k6", "k7", "k8", "k9"] }, { name: "x", kind: "number", min: 0, max: 999 }, { name: "y", kind: "number", min: 0, max: 99 }],
  rules: [{ when: [{ field: "kind", op: "is", value: "k3" }], then: "a" }, { when: [{ field: "x", op: ">", value: { field: "y" } }, { field: "y", op: "<", value: 50 }], then: "b" }, { when: [{ field: "x", op: "=", value: 7 }], then: "a" }, { when: [{ field: "y", op: ">=", value: 90 }], then: "b" }] };
const bigLogic = logicOf(big);
const runs = [];
let bigTable;
for (let i = 0; i < 5; i++) { const t = performance.now(); bigTable = K.build(bigLogic); runs.push(performance.now() - t); }
runs.sort((a, b) => a - b);
let spot = 0;
for (let i = 0; i < 2000; i++) { const r = Math.floor((i * 7919 * 104729) % bigTable.size); if (answer(bigLogic, K.requestOf(bigTable, r)) !== outcome(bigLogic, bigTable.cells[r])) spot++; }
claim("budget: 1,000,000 requests under 50 ms", bigTable.size === 1000000 && runs[2] < 50 && spot === 0, `median ${runs[2].toFixed(1)} ms, fastest ${runs[0].toFixed(1)} ms; ${2000 - spot} of 2,000 spread cells agree with the interpreter`);

if (failed) { console.log(`\n${failed} table claim(s) failed`); process.exit(1); }
console.log("\nthe table decides as the rules do, on every request");
