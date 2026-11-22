// The Obligations proof gate: Lean proves, once for every model, that passing every local obligation means
// every guarantee holds on every request (proof/PrismLive/Regions.lean, regionCheck_sound), and Lean's
// kernel decides every obligation of the Verdant Resonance fixture (proof/PrismLive/VerdantRegions.lean),
// with no compiler trusted. This tool writes VerdantRegions.lean from the fixture's canonical LexLean
// module, through the same reader the live preview uses, so the proof is about the module the LexLean gate
// tests, and refuses a committed file that drifted from it.
//
//   node tools/live_proof.mjs --check    # the committed Lean data is what the fixture reads to
//   node tools/live_proof.mjs --write    # rewrite it
//   node tools/live_proof.mjs --planted  # print the data with rule 2's proximity limit at 60 (plant.sh)
//
// The Lean side runs in CI: `lake build` in proof/ (the kernel decides; a broken model fails the build),
// then the axioms of regionCheck_sound and of Verdant's all_hold must be exactly propext and Quot.sound.
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const { readLexLean } = await import(new URL("../shell/live/lexlean.mjs", import.meta.url));
const source = fs.readFileSync(path.join(root, "tools/fixtures/live/verdant.lex.tex"), "utf8");
const target = path.join(root, "proof/PrismLive/VerdantRegions.lean");

export function leanOf(spec) {
  const OP = { "<": 0, "<=": 1, ">": 2, ">=": 3, is: 4, "=": 4, "is not": 5, "!=": 5 };
  const fi = (n) => spec.fields.findIndex((f) => f.name === n);
  const val = (c) => { const f = spec.fields[fi(c.field)]; if (typeof c.value === "object") throw new Error("a field comparison is not a box"); return f.kind === "choice" ? f.values.indexOf(c.value) : c.value; };
  const cond = (c) => `⟨${fi(c.field)}, ${OP[c.op]}, ${val(c)}⟩`;
  const conds = (cs) => `[${cs.map(cond).join(", ")}]`;
  const out = (o) => spec.outcomes.indexOf(o);
  const box = spec.fields.map((f) => (f.kind === "choice" ? `(0, ${f.values.length - 1})` : `(${f.min ?? 0}, ${f.max})`)).join(", ");
  return `import PrismLive.Regions
open PrismLive.Kernel

/-! Verdant Resonance, read from its canonical LexLean module (tools/fixtures/live/verdant.lex.tex) by
\`tools/live_proof.mjs\`. Do not edit: the gate refuses a file that differs from what the fixture reads to. -/
namespace VerdantR

def rules : List Rule := [
${spec.rules.map((r) => `  ⟨${conds(r.when)}, ${out(r.then)}⟩`).join(",\n")}]

def props : List Property := [
${spec.properties.map((p) => `  ⟨${p.kind === "always"}, ${conds(p.when)}, ${out(p.then)}⟩`).join(",\n")}]

def d : Nat := ${out(spec.otherwise)}

def B : Box := [${box}]

/-- Lean's kernel decides every local obligation of Verdant: no compiler is trusted. -/
theorem checked : regionCheck B rules props d = true := by decide +kernel

/-- Every guarantee holds on every request in the bounds (from \`regionCheck_sound\`). -/
theorem all_hold : ∀ p ∈ props, ∀ x, inBox B x = true → breaks rules d p x = false :=
  fun p hp x hx => regionCheck_sound B rules props d checked p hp x hx

end VerdantR
`;
}

const spec = readLexLean(source);
const mode = process.argv[2];
if (mode === "--write") { fs.writeFileSync(target, leanOf(spec)); console.log(`wrote ${path.relative(root, target)}`); }
else if (mode === "--planted") {
  const broken = structuredClone(spec);
  broken.rules[1].when.find((c) => c.field === "proximity").value = 60;
  process.stdout.write(leanOf(broken));
} else if (mode === "--check") {
  const want = leanOf(spec), have = fs.existsSync(target) ? fs.readFileSync(target, "utf8") : "";
  if (want !== have) { console.log(`FAIL ${path.relative(root, target)} is not what the fixture reads to; run node tools/live_proof.mjs --write`); process.exit(1); }
  console.log(`ok   ${path.relative(root, target)} is what the fixture reads to: ${spec.rules.length} rules, ${spec.properties.length} guarantees`);
} else { console.log("usage: node tools/live_proof.mjs --check | --write | --planted"); process.exit(2); }
