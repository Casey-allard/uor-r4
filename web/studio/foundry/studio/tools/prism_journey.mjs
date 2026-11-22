// The Prism journey, vendored into the shell.
//
// Every app that decides something carries a prism/spec/1 spec; the shell checks it, renders its
// LexLean model and asks Lean to prove it, with prism-code's own code, so a spec means here what it
// means to `npx prism-code` and to the prism MCP server. These files are prism-code's, byte for
// byte, at the version below:
//
//   shell/prism/spec.mjs                               src/journey/spec.mjs   (the checker)
//   shell/prism/spec-file.mjs                          src/journey/spec-file.mjs (the strict reader: the
//                                                      canonical bytes of a spec, and its UOR address)
//   shell/prism/render.mjs                             src/journey/render.mjs (the model renderer)
//   shell/prism/session.mjs                            src/journey/session.mjs (the spec compiler the
//                                                      builder, shell/prism/builder, checks a spec with)
//   shell/prism/lex/Foundation/Bytes.lex.tex           templates/text-request/src/Foundation/Bytes.lex.tex
//   shell/prism/lex/Foundation/View/Text/V1/Model.lex.tex  templates/text-request/src/Foundation/View/Text/V1/Model.lex.tex
//   shell/prism/template.mjs                           src/journey/template.mjs (the GitHub template writer)
//   shell/prism/text-request/*                         templates/text-request/{lexlean.toml, lakefile.toml, lake-manifest.json, prismpm.toml}
//
//   shell/prism-code/src/*                             src/{browser,index,archive,...}.mjs: prism-code's browser
//                                                      graph, so Forge opens, checks and runs a built holo with
//                                                      prism-code's own code (loadArchive, acceptance, dispatch)
//
// A few imports differ, on purpose: node:crypto, which no browser has, becomes ./sha256.mjs, and
// spec-file.mjs reads blake3 from the shell's prism-code graph. SUBSTITUTE names each; nothing else
// may differ.
//
//   node tools/prism_journey.mjs            copy them from the package
//   node tools/prism_journey.mjs --check    the Journey gate: refuse any byte that differs
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

const PRISM_CODE = "0.8.0";
const root = path.join(path.dirname(fileURLToPath(import.meta.url)), "..");
const FILES = [
  ["src/journey/spec.mjs", "shell/prism/spec.mjs"],
  ["src/journey/spec-file.mjs", "shell/prism/spec-file.mjs"],
  ["src/journey/render.mjs", "shell/prism/render.mjs"],
  ["src/journey/session.mjs", "shell/prism/session.mjs"],
  ["templates/text-request/src/Foundation/Bytes.lex.tex", "shell/prism/lex/Foundation/Bytes.lex.tex"],
  ["templates/text-request/src/Foundation/View/Text/V1/Model.lex.tex", "shell/prism/lex/Foundation/View/Text/V1/Model.lex.tex"],
  ["src/journey/template.mjs", "shell/prism/template.mjs"],
  ["templates/text-request/lexlean.toml", "shell/prism/text-request/lexlean.toml"],
  ["templates/text-request/lakefile.toml", "shell/prism/text-request/lakefile.toml"],
  ["templates/text-request/lake-manifest.json", "shell/prism/text-request/lake-manifest.json"],
  ["templates/text-request/prismpm.toml", "shell/prism/text-request/prismpm.toml"],
];
// Everything src/browser.mjs reaches, statically or through index.mjs's browser branch.
for (const file of ["browser.mjs", "index.mjs", "archive.mjs", "errors.mjs", "blake3.mjs", "attestations.mjs", "view.mjs", "core.mjs", "capabilities.mjs", "closure.mjs", "sha256.mjs", "inspect.mjs", "vendor/verify.mjs", "verify.mjs", "replay.mjs", "version.mjs", "middleware.mjs", "containers.mjs", "canonical/index.js", "tracing.mjs", "manifest.mjs", "toml.mjs", "provenance.mjs"]) FILES.push(["src/" + file, "shell/prism-code/src/" + file]);
// Node imports the browser has not: each file's substitutions, applied in order. template.mjs and
// spec-file.mjs hash through node:crypto, and spec-file.mjs reads blake3 from prism-code's own graph,
// which the shell keeps under prism-code/src.
const SUBSTITUTE = {
  "shell/prism/template.mjs": [["import { createHash } from 'node:crypto';", "import { createHash } from './sha256.mjs';"]],
  "shell/prism/spec-file.mjs": [
    ["import { createHash } from 'node:crypto';", "import { createHash } from './sha256.mjs';"],
    ["import { kappa } from '../blake3.mjs';", "import { kappa } from '../prism-code/src/blake3.mjs';"],
  ],
};

const prefix = path.join(os.tmpdir(), "hologram-forge-prism-code-" + PRISM_CODE);
const pkg = path.join(prefix, "node_modules", "prism-code");
if (!fs.existsSync(path.join(pkg, "package.json"))) {
  execFileSync("npm", ["install", "--no-save", "--no-audit", "--no-fund", "--prefix", prefix, "prism-code@" + PRISM_CODE], { stdio: "inherit", shell: process.platform === "win32" });
}
const version = JSON.parse(fs.readFileSync(path.join(pkg, "package.json"), "utf8")).version;
if (version !== PRISM_CODE) throw new Error(`prism-code ${version} installed, ${PRISM_CODE} pinned`);
// Text is compared with one line ending, so a Windows checkout and the Linux lane agree.
const text = (p) => fs.readFileSync(p, "utf8").replace(/\r\n/g, "\n");
const bad = [];
for (const [from, to] of FILES) {
  let upstream = text(path.join(pkg, from));
  for (const [was, is] of SUBSTITUTE[to] ?? []) {
    if (!upstream.includes(was)) throw new Error(`${from} no longer has ${was}`);
    upstream = upstream.replace(was, is);
  }
  const target = path.join(root, to);
  if (process.argv[2] === "--check") {
    if (!fs.existsSync(target) || text(target) !== upstream) bad.push(to);
  } else {
    fs.mkdirSync(path.dirname(target), { recursive: true });
    fs.writeFileSync(target, upstream);
  }
}
if (bad.length) { console.error(`journey gate refused: ${bad.join(", ")} differ from prism-code ${PRISM_CODE}; run node tools/prism_journey.mjs`); process.exit(1); }
console.log(`journey: ${FILES.length} files are prism-code ${PRISM_CODE}'s${process.argv[2] === "--check" ? ", byte for byte" : ""}`);
