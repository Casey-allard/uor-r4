// The Live gate: the live preview draws an app from its spec alone, deterministically, patching only
// what changed, from the first whole node a model writes, with the one interpreter every app ships.
//
//   node tools/live_preview.mjs --check          every claim below, on the RSVP fixture; exit 1 on a failure
//   node tools/live_preview.mjs --frame <file>   also write the frame as the host assembles it (for a browser run)
//
// Claims (level build: validated here against an oracle, evidence not proof):
//   determinism   the same spec, with its keys in any order, projects to the same root kappa
//   locality      a rule's outcome changes that rule's node and the logic kappa, nothing else; a word
//                 of the screen changes that node and not the logic kappa
//   streaming     every prefix of a streamed answer projects without error; no key drawn mid-stream is
//                 absent from the final screen; the last prefix is the whole spec
//   interpreter   the frame's interpreter is prism.js's, verbatim, and answers every acceptance vector
//                 as the app's rule module does
//   view          a view block the frame cannot draw is named
//   budget        projecting the fixture takes under 4 ms (median), leaving the rest of a 16 ms frame to the DOM
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";
import { performance } from "node:perf_hooks";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const shell = path.join(root, "shell");
const prism = await import(new URL("../shell/prism.js", import.meta.url));
const live = await import(new URL("../shell/live/project.mjs", import.meta.url));
const { frameHtml, FONTS } = await import(new URL("../shell/live/frame-html.mjs", import.meta.url));
const { acceptanceVectors } = await import(new URL("../shell/prism/render.mjs", import.meta.url));

const fixture = JSON.parse(fs.readFileSync(path.join(root, "tools/fixtures/live/rsvp.json"), "utf8"));
let failed = 0;
const claim = (name, ok, detail = "") => { console.log(`${ok ? "ok  " : "FAIL"} ${name}${detail ? " -- " + detail : ""}`); if (!ok) failed++; };

// The fixture is a spec Forge checks: its examples pass and its guarantees have no counterexample.
const checked = prism.checkSpec(fixture);
claim("fixture checks", checked.ok, checked.issues && checked.issues.join("; "));
const spec = checked.spec;

// determinism
const shuffle = (v) => Array.isArray(v) ? v.map(shuffle) : v && typeof v === "object" ? Object.fromEntries(Object.keys(v).reverse().map((k) => [k, shuffle(v[k])])) : v;
const a = live.project(spec), b = live.project(structuredClone(spec)), c = live.project(shuffle(spec));
claim("determinism", a.root === b.root && a.root === c.root, `${a.nodes.length} nodes, root ${a.root.slice(0, 19)}…`);

// locality
const diff = (x, y) => { const m = new Map(x.nodes.map((n) => [n.key, n.kappa])); return y.nodes.filter((n) => m.get(n.key) !== n.kappa).map((n) => n.key).concat(x.nodes.filter((n) => !y.nodes.some((o) => o.key === n.key)).map((n) => n.key)); };
const ruleEdit = structuredClone(spec); ruleEdit.rules[2].then = "none";
const r = live.project(ruleEdit);
claim("locality: a rule", diff(a, r).join() === "rule:2" && r.logicKappa !== a.logicKappa, `changed ${diff(a, r).join() || "nothing"}`);
const wordEdit = structuredClone(spec); wordEdit.view.blocks[0].text = "Join us";
const w = live.project(wordEdit);
claim("locality: a word", diff(a, w).join() === "h:0" && w.logicKappa === a.logicKappa, `changed ${diff(a, w).join() || "nothing"}`);

// streaming: the answer a model writes, one character at a time
const ORDER = ["name", "title", "purpose", "view", "fields", "outcomes", "meanings", "rules", "otherwise", "otherwiseWhy", "properties", "examples", "limits"];
const ordered = Object.fromEntries(ORDER.filter((k) => fixture[k] !== undefined).map((k) => [k, fixture[k]]));
const answer = "Here are the rules.\n<prism-spec>" + JSON.stringify(ordered) + "</prism-spec>\nDone.";
const finalKeys = new Set(live.project(fixture).nodes.map((n) => n.key));
let errors = 0, stray = new Set(), first = -1, frames = 0, last = null;
const t0 = performance.now();
for (let i = 1; i <= answer.length; i++) {
  try {
    const raw = live.partialSpec(answer.slice(0, i));
    if (!raw) continue;
    const p = live.project(raw);
    if (p.root !== last) { frames++; last = p.root; }
    if (p.nodes.length && first < 0) first = i;
    for (const n of p.nodes) if (!finalKeys.has(n.key)) stray.add(n.key);
  } catch (e) { errors++; }
}
const streamMs = performance.now() - t0;
claim("streaming: no prefix throws", errors === 0, `${answer.length} prefixes`);
claim("streaming: nothing drawn is taken away", stray.size === 0, stray.size ? [...stray].join(", ") : `${frames} distinct screens, first node at character ${first}`);
claim("streaming: the last prefix is the spec", live.project(live.partialSpec(answer)).root === live.project(fixture).root);

// interpreter
const html = frameHtml({ brandCss: fs.readFileSync(path.join(shell, "brand.css"), "utf8"), componentsCss: fs.readFileSync(path.join(shell, "live/components.css"), "utf8"), componentsJs: fs.readFileSync(path.join(shell, "live/components.js"), "utf8"), interpreter: prism.INTERPRETER, fonts: Object.fromEntries(FONTS.map((p) => [p, fs.readFileSync(path.join(shell, p)).toString("base64")])) });
const module = checked.module;
const inFrame = html.includes(prism.INTERPRETER);
const inModule = prism.INTERPRETER.split("\n").filter(Boolean).every((l) => module.includes(l));
const { prismAnswer } = new Function(prism.INTERPRETER + "\nreturn { prismAnswer, prismDecide };")();
const vectors = acceptanceVectors(spec);
let disagree = 0;
for (const v of vectors) {
  let text; try { text = new TextDecoder("utf-8", { fatal: true }).decode(v.bytes); } catch { if (v.response !== "invalid") disagree++; continue; }
  if (prismAnswer(spec, text) !== v.response) disagree++;
}
claim("interpreter: one copy", inFrame && inModule, inFrame ? (inModule ? "" : "the rule module differs") : "the frame does not carry it");
claim("interpreter: every vector", disagree === 0, `${vectors.length - disagree} of ${vectors.length}`);

// view
const badView = structuredClone(fixture); badView.view.blocks.push({ kind: "chart" }, { kind: "input", field: "guests" });
claim("view: a block the frame cannot draw is named", live.checkView(badView).length === 2 && live.checkView(fixture).length === 0, live.checkView(badView).join(" "));

// budget
const runs = [];
for (let i = 0; i < 300; i++) { const s = performance.now(); live.project(spec); runs.push(performance.now() - s); }
runs.sort((x, y) => x - y);
const median = runs[runs.length >> 1];
claim("budget: projection under 4 ms", median < 4, `median ${median.toFixed(2)} ms, p95 ${runs[Math.floor(runs.length * 0.95)].toFixed(2)} ms; the whole ${answer.length}-character stream ${streamMs.toFixed(0)} ms`);

// the frame's compiled script parses as the classic script the page loads it as
const tmp = fs.mkdtempSync(path.join(os.tmpdir(), "live-"));
try { fs.copyFileSync(path.join(shell, "live/components.js"), path.join(tmp, "frame.cjs")); execFileSync(process.execPath, ["--check", path.join(tmp, "frame.cjs")], { stdio: "pipe" }); claim("frame parses", true); }
catch (e) { claim("frame parses", false, String(e.stderr || e).split("\n").slice(0, 3).join(" ")); }
fs.rmSync(tmp, { recursive: true, force: true });

const out = process.argv.indexOf("--frame");
if (out > 0 && process.argv[out + 1]) { fs.writeFileSync(process.argv[out + 1], html); fs.writeFileSync(process.argv[out + 1] + ".answer.txt", answer); console.log(`frame written: ${process.argv[out + 1]} (${html.length} bytes)`); }
if (failed) { console.log(`\n${failed} live preview claim(s) failed`); process.exit(1); }
console.log("\nthe live preview draws the spec, and only the spec");
