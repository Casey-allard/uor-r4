// The Template gate: the commit Forge makes on a repository created from UOR-Foundation/template is the
// commit prism-code makes there, byte for byte.
//
// prism-code builds an app on GitHub Actions by writing it into a template instance: journey/project.mjs
// writeApp makes the project, journey/engine.mjs writeTemplateApp copies it under app/ and writes the
// claims, tests and page the template's gate reads. Forge makes the same commit in the tab
// (shell/prism.js templateCommit). For each spec under tools/fixtures/specs, both run on the template at
// TEMPLATE (fetched once), and every written file and every removal must agree.
//
//   node tools/prism_template.mjs --check
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const PRISM_CODE = "0.8.0";
const TEMPLATE = "a7a3041cd85d745bb2d6dfc7ee72513185fa48be"; // UOR-Foundation/template main, 2026-11-10
const READ = ["app/lexlean.lock", "app/lean-toolchain", "model/ids.toml", "CONFORMANCE.md", "site/app.json"];
const EXAMPLE = ["app/src/Contract.lex.tex", "crates/conformance/tests/contract.rs", "features/suites/contract.feature"];
const root = path.join(path.dirname(fileURLToPath(import.meta.url)), "..");

const prefix = path.join(os.tmpdir(), "hologram-forge-prism-code-" + PRISM_CODE);
const pkg = path.join(prefix, "node_modules", "prism-code");
if (!fs.existsSync(path.join(pkg, "package.json"))) execFileSync("npm", ["install", "--no-save", "--no-audit", "--no-fund", "--prefix", prefix, "prism-code@" + PRISM_CODE], { stdio: "inherit", shell: process.platform === "win32" });
const up = (p) => import(pathToFileURL(path.join(pkg, p)).href);
const { writeApp } = await up("src/journey/project.mjs");
const { writeTemplateApp } = await up("src/journey/engine.mjs");
const { validateSpec } = await up("src/journey/spec.mjs");
const forge = await import(pathToFileURL(path.join(root, "shell/prism.js")).href);

// The template's files the writers read, at TEMPLATE, cached beside the package.
const cache = path.join(prefix, "template-" + TEMPLATE);
const template = {};
for (const p of READ) {
  const at = path.join(cache, p);
  if (!fs.existsSync(at)) {
    const r = await fetch(`https://raw.githubusercontent.com/UOR-Foundation/template/${TEMPLATE}/${p}`);
    if (!r.ok) throw new Error(`template ${p}: ${r.status}`);
    fs.mkdirSync(path.dirname(at), { recursive: true });
    fs.writeFileSync(at, await r.text());
  }
  template[p] = fs.readFileSync(at, "utf8");
}
const tree = [...READ, ...EXAMPLE, "app/src/Foundation/Bytes.lex.tex"];

const walk = (dir, base = dir) => fs.readdirSync(dir, { withFileTypes: true }).flatMap((e) => (e.isDirectory() ? walk(path.join(dir, e.name), base) : [path.relative(base, path.join(dir, e.name)).split(path.sep).join("/")]));
const specs = fs.readdirSync(path.join(root, "tools/fixtures/specs")).filter((f) => f.endsWith(".json"));
const bad = [];
for (const file of specs) {
  const fixture = JSON.parse(fs.readFileSync(path.join(root, "tools/fixtures/specs", file), "utf8"));
  const spec = validateSpec(fixture.spec);
  const axioms = fixture.axioms || {};
  const repository = `owner/prism-${spec.name}`;
  const attestation = "a".repeat(64);
  // prism-code: a project made by writeApp, written into a clone of the template by writeTemplateApp.
  const work = fs.mkdtempSync(path.join(os.tmpdir(), "forge-template-"));
  const dir = path.join(work, "prism"), clone = path.join(work, "clone");
  for (const p of tree) { fs.mkdirSync(path.dirname(path.join(clone, p)), { recursive: true }); fs.writeFileSync(path.join(clone, p), template[p] ?? "example\n"); }
  const session = { spec, axioms, status: { properties: Object.fromEntries(spec.properties.map((p) => [p.id, "proved"])), proved: { attestation }, repository: { full: repository } } };
  writeApp(dir, session);
  writeTemplateApp(clone, dir, session);
  const theirs = new Map(walk(clone).map((p) => [p, fs.readFileSync(path.join(clone, p), "utf8")]));
  // Forge: the same commit, from the same template files.
  const ours = await forge.templateCommit({ spec, axioms, attestation, repository, tree, read: async (p) => template[p], load: async (p) => fs.readFileSync(new URL(p, pathToFileURL(path.join(root, "shell/prism.js"))), "utf8").replace(/\r\n/g, "\n") });
  const after = new Map(tree.filter((p) => !ours.remove.includes(p)).map((p) => [p, template[p] ?? "example\n"]));
  for (const f of ours.files) after.set(f.path, f.content);
  for (const p of new Set([...theirs.keys(), ...after.keys()])) {
    if (theirs.get(p) !== after.get(p)) bad.push(`${file}: ${p} ${!theirs.has(p) ? "only Forge writes it" : !after.has(p) ? "only prism-code writes it" : "differs"}`);
  }
  fs.rmSync(work, { recursive: true, force: true });
  if (process.argv[2] !== "--check") console.log(`${file}: ${ours.files.length} files written, ${ours.remove.length} removed`);
}
if (bad.length) { console.error("template gate refused:\n  " + bad.join("\n  ")); process.exit(1); }
console.log(`template: ${specs.length} specs, Forge's commit is prism-code ${PRISM_CODE}'s on template ${TEMPLATE.slice(0, 7)}, byte for byte`);
