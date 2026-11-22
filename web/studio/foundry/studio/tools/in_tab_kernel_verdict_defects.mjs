// Plants one defect at a time in the verdict codec and reports whether the suite refuses it. Exits 1 if any survives.
//   node studio/tools/in_tab_kernel_verdict_defects.mjs
// The codec is restored after each defect, and at the end whatever happened.
import fs from "node:fs";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const codec = fileURLToPath(new URL("../shell/in-tab-verdict-codec.mjs", import.meta.url));
const suite = fileURLToPath(new URL("./in_tab_kernel_verdict.test.mjs", import.meta.url));
const original = fs.readFileSync(codec, "utf8");
const defects = [
  ["a reply whose word is not its code is believed", "code > 3 || word !== code", "code > 3"],
  ["an extra field, such as a server's verdict, is ignored", "Object.keys(input).sort().join() !== [...order].sort().join()", "false"],
  ["an axiom outside the admitted three is dropped", "allowed.get(n) ?? 8", "allowed.get(n) ?? 0"],
  ["an address is cut to its first 128 bits", "Array.from({ length: 8 }, (_, i) => view.getUint32(i * 4, true));\n}", "Array.from({ length: 8 }, (_, i) => i < 4 ? view.getUint32(i * 4, true) : 0);\n}"],
  ["the request is read after the guest is ready", "let bytes;\n  try { bytes = encodeVerdictRequest(input); } catch { return { ok: false, code: 'malformedVerdictRequest' }; }\n  try { const guest = await ready; return", "let bytes;\n  try { encodeVerdictRequest(input); } catch { return { ok: false, code: 'malformedVerdictRequest' }; }\n  try { const guest = await ready; bytes = encodeVerdictRequest(input); return"],
  ["a flag other than 0 or 1 is sent", "if (n !== 0 && n !== 1) fail();", ""],
  ["a guest fault reads as proved", "catch { return { ok: false, code: 'verdictRuntimeFailed' }; }", "catch { return { ok: true, code: 0, verdict: 'proved' }; }"],
];
const run = () => spawnSync(process.execPath, ["--test", "--test-reporter=tap", suite], { encoding: "utf8" });
let survived = 0;
try {
  for (const [name, find, replace] of defects) {
    if (original.split(find).length !== 2) throw new Error("the defect cannot be planted: " + name);
    fs.writeFileSync(codec, original.replace(find, () => replace));
    const r = run();
    const failing = (r.stdout.match(/^not ok \d+ - .*$/gm) ?? []).map((l) => l.replace(/^not ok \d+ - /, ""));
    const refused = r.status !== 0 && failing.length > 0;
    if (!refused) survived++;
    console.log(`${refused ? "refused " : "SURVIVED"} ${name}${refused ? ": " + failing[0] : ""}`);
  }
} finally {
  fs.writeFileSync(codec, original);
}
const restored = run();
console.log(restored.status === 0 ? "restored: the suite passes" : "restored: THE SUITE FAILS");
process.exit(survived || restored.status !== 0 ? 1 : 0);
