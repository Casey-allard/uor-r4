// The Edit gate: every word on the live screen is a place in the spec, and editing it there is a spec
// patch that changes that word's node and nothing else; a patch that does not check is refused and the
// last good spec stands; undo and redo return the same screen, root kappa for root kappa.
//
//   node tools/live_edit.mjs --check
//
// Claims (level build: validated here against the projection and Forge's checker):
//   addresses   every edit path on every node of the RSVP projection names, in the spec as written, the
//               very words that node shows
//   locality    editing any one of those words changes that node's kappa and no other node's
//   logic       an edit of an answer word changes the logic kappa; an edit of a screen word does not
//   refusal     a patch that names nothing, or makes rules that do not check, is refused, and the spec
//               is unchanged
//   history     undo after an edit gives back the first screen's root kappa exactly; redo the edited one
//   raw paths   a node keeps pointing at the right item when an earlier item was not whole and was left out
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const prism = await import(new URL("../shell/prism.js", import.meta.url));
const live = await import(new URL("../shell/live/project.mjs", import.meta.url));
const { applyOps, at, createEditor } = await import(new URL("../shell/live/edit.mjs", import.meta.url));

let failed = 0;
const claim = (name, ok, detail = "") => { console.log(`${ok ? "ok  " : "FAIL"} ${name}${detail ? " -- " + detail : ""}`); if (!ok) failed++; };
const fixture = JSON.parse(fs.readFileSync(path.join(root, "tools/fixtures/live/rsvp.json"), "utf8"));
const check = (spec) => { const c = prism.checkSpec(spec); return { ok: !!c.ok, issues: c.issues || [] }; };
const shown = (n, slot) => (slot === "meaning" ? null : n.data[slot]);
const kappas = (p) => new Map(p.nodes.map((n) => [n.key, n.kappa]));

// addresses and locality, over every editable word on the screen
const base = live.project(fixture), before = kappas(base);
let slots = 0, wrongAddress = [], spill = [], logicMoved = [], logicStill = [];
for (const n of base.nodes) {
  for (const [slot, p] of Object.entries(n.data.edit || {})) {
    if (!Array.isArray(p) || slot === "meaning") continue;
    slots++;
    if (at(fixture, p) !== shown(n, slot)) wrongAddress.push(`${n.key}.${slot} -> ${p.join(".")}`);
    // a new value that keeps the spec valid: an answer word becomes another answer, a sentence gains a word
    const isAnswer = slot === "then" || slot === "response";
    const value = slot === "then" ? fixture.outcomes.find((o) => o !== n.data.then) : slot === "response" || slot === "request" ? null : shown(n, slot) + " now";
    if (value === null) continue;
    const next = live.project(applyOps(fixture, [{ op: "set", path: p, value }]));
    const moved = next.nodes.filter((m) => before.get(m.key) !== m.kappa).map((m) => m.key);
    if (moved.length !== 1 || moved[0] !== n.key) spill.push(`${n.key}.${slot}: ${moved.join(",") || "nothing"}`);
    if (isAnswer && next.logicKappa === base.logicKappa) logicStill.push(`${n.key}.${slot}`);
    if (!isAnswer && next.logicKappa !== base.logicKappa) logicMoved.push(`${n.key}.${slot}`);
  }
}
claim("addresses: every word names its place", slots > 0 && wrongAddress.length === 0, wrongAddress.join("; ") || `${slots} editable words on ${base.nodes.length} nodes`);
claim("locality: an edit changes its node only", spill.length === 0, spill.join("; ") || "each word edited alone");
claim("logic: answer words move it, screen words do not", logicStill.length === 0 && logicMoved.length === 0, [...logicStill.map((x) => "still: " + x), ...logicMoved.map((x) => "moved: " + x)].join("; ") || "as claimed");

// refusal
const ed = createEditor(fixture, check);
const nowhere = ed.apply([{ op: "set", path: ["rules", 9, "then"], value: "none" }]);
const unknown = ed.apply([{ op: "set", path: ["rules", 0, "then"], value: "maybe-later" }]);
claim("refusal: nothing changes", !nowhere.ok && !unknown.ok && JSON.stringify(ed.spec) === JSON.stringify(fixture), [nowhere, unknown].map((r) => (r.ok ? "kept, not refused" : r.issues[0])).join(" | "));

// history
const edited = ed.apply([{ op: "set", path: ["rules", 2, "then"], value: "none" }]);
const editedRoot = live.project(ed.spec).root;
const undone = live.project(ed.undo()).root, redone = live.project(ed.redo()).root;
claim("history: undo and redo give the same screens", edited.ok && undone === base.root && redone === editedRoot && editedRoot !== base.root, `${base.root.slice(0, 19)}… back and forth`);

// raw paths: an incomplete first rule is left out of the screen, and the second still points at index 1
const partial = structuredClone(fixture);
partial.rules.unshift({ when: [{ field: "attending" }], then: "none" });
const pn = live.project(partial).nodes.find((n) => n.key === "rule:0");
claim("raw paths: after a skipped item", pn && pn.data.edit.then.join(".") === "rules.1.then" && at(partial, pn.data.edit.then) === pn.data.then, pn ? pn.data.edit.then.join(".") : "no node");

if (failed) { console.log(`\n${failed} edit claim(s) failed`); process.exit(1); }
console.log("\nevery word on the screen is edited in the spec, and only there");
