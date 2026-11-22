// Run: node studio/tools/telemetry_defects.mjs   (writes studio/telemetry/out/defects.log)
// Plants one defect at a time in the bytes that actually run (the model's data form, an adapter, or a
// new file), runs the gates, requires the named gate to fail, restores the file byte for byte and
// requires every gate to pass again. The gates: the test suite (test.mjs) and the egress gate.
import { readFileSync, writeFileSync, rmSync, existsSync, mkdirSync, appendFileSync } from "node:fs";
import { createHash } from "node:crypto";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const here = (p) => fileURLToPath(new URL(p, import.meta.url));
const sha = (b) => createHash("sha256").update(b).digest("hex");
const tests = () => {
  const r = spawnSync(process.execPath, ["--test", here("./telemetry.test.mjs")], { encoding: "utf8" });
  const count = (k) => Number((r.stdout.match(new RegExp(`^# ${k} (\\d+)`, "m")) ?? [0, -1])[1]);
  return { exit: r.status, pass: count("pass"), fail: count("fail"), failing: [...r.stdout.matchAll(/^not ok \d+ - (.*)$/gm)].map((m) => m[1]) };
};
const gate = () => { const r = spawnSync(process.execPath, [here("./telemetry_egress.mjs")], { encoding: "utf8" }); return { exit: r.status, findings: r.stdout.split("\n").filter((l) => l.startsWith("refused:")) }; };
const declaration = (data, name) => data.declarations.find((d) => d.name === name);
const editData = (mutate) => (text) => { const data = JSON.parse(text); mutate(data); return JSON.stringify(data) + "\n"; };
const replace = (from, to) => (text) => { if (!text.includes(from)) throw new Error("anchor not found: " + from); return text.replace(from, to); };

const DEFECTS = [
  {
    name: "egress: the store sends each turn with sendBeacon", file: "../shell/telemetry/store.mjs",
    plant: replace("async put(turn) { turns.set(turn.id, structuredClone(turn)); },", "async put(turn) { navigator.sendBeacon(\"/collect\", JSON.stringify(turn)); turns.set(turn.id, structuredClone(turn)); },"),
  },
  {
    name: "egress: an unlisted file joins the telemetry", file: "../shell/telemetry/beacon.mjs", create: true,
    plant: () => "export const send = (turn) => navigator.sendBeacon(\"/collect\", JSON.stringify(turn));\n",
  },
  {
    name: "adapter: a flow the kernel denies goes ahead", file: "../shell/telemetry/telemetry.mjs",
    plant: replace('if (answer !== "allow") throw', 'if (false) throw'),
  },
  {
    name: "model: overCode is one microsecond late (value > threshold instead of >=)", file: "../shell/telemetry/data/Telemetry.json",
    plant: editData((data) => { const c = declaration(data, "overCode").body.else_value.condition; c.kind = "blt"; }),
  },
  {
    name: "model: persistent drops the majority, so one slow turn among fast ones is suggested", file: "../shell/telemetry/data/Telemetry.json",
    plant: editData((data) => { const d = declaration(data, "persistent"); d.body = d.body.left; }),
  },
  {
    name: "adapter: export ignores whether the user asked", file: "../shell/telemetry/telemetry.mjs",
    plant: replace(`await flow("save", userAct === true);`, `await flow("save", true);`),
  },
  {
    name: "model: standing forgets an app stopped from opening, so it shows as all clear", file: "../shell/telemetry/data/Telemetry.json",
    plant: editData((data) => { const d = declaration(data, "standing"); d.body.condition = d.body.condition.left; }),
  },
  {
    name: "adapter: a refused site is counted as allowed", file: "../shell/telemetry/telemetry.mjs",
    plant: replace("const kind = NET[d && d.kind],", `const kind = NET[d && d.kind === "blocked" ? "allowed" : d && d.kind],`),
  },
  {
    name: "worker: an app's own page is handed what other apps reached", file: "../shell/sw.template.js",
    plant: replace("!/^(p|holo|app)\\//.test(new URL(c.url).pathname.slice(BASE.length))", "true"),
  },
];

mkdirSync(here("../telemetry/out"), { recursive: true });
const logFile = here("../telemetry/out/defects.log");
writeFileSync(logFile, "");
const baseline = { tests: tests(), gate: gate() };
if (baseline.tests.exit !== 0 || baseline.gate.exit !== 0) { console.error("baseline is not green", baseline); process.exit(1); }
let ok = true;
for (const d of DEFECTS) {
  const path = here(d.file), original = d.create ? null : readFileSync(path);
  let planted;
  try {
    writeFileSync(path, d.plant(original ? original.toString("utf8") : ""));
    planted = { tests: tests(), gate: gate() };
  } finally {
    if (d.create) rmSync(path, { force: true }); else writeFileSync(path, original);
  }
  const restored = { tests: tests(), gate: gate() };
  const intact = d.create ? !existsSync(path) : sha(readFileSync(path)) === sha(original);
  const caught = planted.tests.exit !== 0 || planted.gate.exit !== 0;
  const green = restored.tests.exit === 0 && restored.gate.exit === 0;
  const row = { defect: d.name, file: d.file, caught, planted: { tests: `exit ${planted.tests.exit}, ${planted.tests.pass} pass, ${planted.tests.fail} fail`, failing: planted.tests.failing, gate: planted.gate.findings }, restored: `tests ${restored.tests.pass} pass ${restored.tests.fail} fail, gate exit ${restored.gate.exit}`, intact };
  const line = JSON.stringify(row);
  appendFileSync(logFile, line + "\n");
  console.log(line);
  ok &&= caught && green && intact;
}
console.log(ok ? "every defect was caught and every file restored" : "A DEFECT WAS NOT CAUGHT OR A FILE WAS NOT RESTORED");
process.exit(ok ? 0 : 1);
