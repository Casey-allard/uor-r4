// The Engine gate: the interpreter, the presentation reader and the library model the live preview
// runs for a stateful app are the ones under v2/, byte for byte. v2/ is where they are tested and held
// to compiled guests; the shell serves copies because the deployed site is this directory alone.
//
//   node tools/v2_engine.mjs --check   exit 1 when a copy differs from its source
//   node tools/v2_engine.mjs --write   copy the sources over
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const repo = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const COPIES = {
  "studio/shell/live/v2/interp.mjs": "v2/shell/interp.mjs",
  "studio/shell/live/v2/render.mjs": "v2/shell/render.mjs",
  "studio/shell/live/v2/Model.json": "v2/probe-payment/data/Foundation/View/Browser/V1/Model.json",
  "studio/shell/live/v2/Model.lex.tex": "v2/probe-payment/src/Foundation/View/Browser/V1/Model.lex.tex",
};
// The instructions a model is given for the app shape, as a module the shell can import.
const DERIVED = { "studio/shell/live/v2/rules.mjs": ["v2/loop/RULES.txt", (text) => "// Written by tools/v2_engine.mjs from v2/loop/RULES.txt. Edit that file, not this one.\nexport default " + JSON.stringify(text) + ";\n"] };
const mode = process.argv[2];
if (mode !== "--check" && mode !== "--write") { console.error("usage: node tools/v2_engine.mjs --check | --write"); process.exit(64); }
let failed = 0;
for (const [copy, source] of Object.entries(COPIES)) {
  const want = fs.readFileSync(path.join(repo, source));
  if (mode === "--write") { fs.mkdirSync(path.dirname(path.join(repo, copy)), { recursive: true }); fs.writeFileSync(path.join(repo, copy), want); continue; }
  const have = fs.existsSync(path.join(repo, copy)) ? fs.readFileSync(path.join(repo, copy)) : null;
  const ok = have !== null && have.equals(want);
  console.log(`${ok ? "ok  " : "FAIL"} ${copy} is ${source}`);
  if (!ok) failed++;
}
for (const [copy, [source, derive]] of Object.entries(DERIVED)) {
  const want = Buffer.from(derive(fs.readFileSync(path.join(repo, source), "utf8")));
  if (mode === "--write") { fs.writeFileSync(path.join(repo, copy), want); continue; }
  const have = fs.existsSync(path.join(repo, copy)) ? fs.readFileSync(path.join(repo, copy)) : null;
  const ok = have !== null && have.equals(want);
  console.log(`${ok ? "ok  " : "FAIL"} ${copy} is derived from ${source}`);
  if (!ok) failed++;
}
process.exit(failed ? 1 : 0);
