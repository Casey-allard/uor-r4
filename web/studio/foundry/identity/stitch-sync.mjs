#!/usr/bin/env node
// Prism id comes from stitch (afflom/stitch), pinned by commit: the frame runs
// stitch's Prism id and follows it as stitch's Prism model is refined. Stitch
// is not a Foundry component yet, so nothing here edits it: this script copies it.
//
// identity/prism-id.mjs is generated, one self-contained module: stitch's
// src/prism-id.js, the stitch modules it imports (the relay framing its
// substrate bus uses, and what that imports), and the part of stitch's
// generated model they call (every decision they name, what those call, and
// the model's evaluator). The whole model is 1.2 MB; the part used is a few
// KB, so the frame can carry it. Each imported module keeps its own scope.
// identity/stitch.lock names the commit and the hash of every input and of
// the output.
//
//   node identity/stitch-sync.mjs <commit>          fetch that commit, write both
//   node identity/stitch-sync.mjs <commit> <dir>    read a local stitch checkout at it
//   node identity/stitch-sync.mjs --check           the output is the one the lock names
import { createHash } from "node:crypto";
import { execFileSync } from "node:child_process";
import { readFile, writeFile } from "node:fs/promises";
import { join, posix } from "node:path";
import { fileURLToPath } from "node:url";

const root = fileURLToPath(new URL("../", import.meta.url));
const LOCK = join(root, "identity/stitch.lock");
const OUT = join(root, "identity/prism-id.mjs");
const REPOSITORY = "afflom/stitch";
const ENTRY = "src/prism-id.js";
const MODEL = "src/layer1-uor/generated-model.js";
const sha256 = (text) => createHash("sha256").update(text).digest("hex");

const [first, dir] = process.argv.slice(2);
if (first === "--check") {
  const lock = JSON.parse(await readFile(LOCK, "utf8"));
  const actual = sha256(await readFile(OUT));
  if (actual !== lock.output.sha256) {
    console.error(`identity/prism-id.mjs is ${actual}, but identity/stitch.lock names ${lock.output.sha256}. Run node identity/stitch-sync.mjs ${lock.commit}.`);
    process.exit(1);
  }
  console.log(`prism id: stitch ${lock.commit.slice(0, 12)}, output matches the lock`);
  process.exit(0);
}
if (!/^[0-9a-f]{40}$/.test(first ?? "")) {
  console.error("usage: node identity/stitch-sync.mjs <40-hex stitch commit> [<local stitch checkout>] | --check");
  process.exit(2);
}
const commit = first;

async function source(path) {
  if (dir) return execFileSync("git", ["-C", dir, "show", `${commit}:${path}`], { encoding: "utf8", maxBuffer: 64 << 20 });
  const response = await fetch(`https://raw.githubusercontent.com/${REPOSITORY}/${commit}/${path}`);
  if (!response.ok) throw new Error(`${REPOSITORY}@${commit}:${path} answered ${response.status}`);
  return response.text();
}

// Every module prism-id.js reaches through its imports, dependencies first.
const IMPORT = /^import\s*\{([^}]*)\}\s*from\s*'([^']+)';\n/gm;
const sources = {};
const order = [];
async function load(path) {
  if (path in sources) return;
  sources[path] = null;
  const text = await source(path);
  sources[path] = text;
  for (const [, , from] of text.matchAll(IMPORT)) {
    const target = posix.join(posix.dirname(path), from);
    if (target !== MODEL) await load(target);
  }
  order.push(path);
}
await load(ENTRY);
const generated = await source(MODEL);
sources[MODEL] = generated;

// The generated model: `const definitions = {...};`, `const types = {...};`
// (both JSON), then the digests and sources, then the evaluator from
// `const qualify` to `export const model=`, then named bindings.
const lines = generated.split("\n");
const block = (head) => {
  const start = lines.indexOf(`const ${head} = {`);
  const end = lines.indexOf("};", start);
  if (start < 0 || end < 0) throw new Error(`the generated model has no ${head} block`);
  return JSON.parse(lines.slice(start, end + 1).join("\n").replace(`const ${head} = `, "").replace(/;$/, ""));
};
const definitions = block("definitions");
const types = block("types");
const evalStart = lines.findIndex((line) => line.startsWith("const qualify = "));
const evalEnd = lines.findIndex((line) => line.startsWith("export const model="));
if (evalStart < 0 || evalEnd < evalStart) throw new Error("the generated model's evaluator moved");
const evaluator = lines.slice(evalStart, evalEnd + 1).join("\n");
const bindings = Object.fromEntries([...generated.matchAll(/^export const (\w+) = model\["([\w.]+)"\];$/gm)].map(([, name, id]) => [name, id]));

// Every decision the copied modules name, and everything those call:
// model['Id'], call('name') for relay framing's Stitch.Relay, and the model's
// named bindings a module imports.
const qualify = (module, name) => (name.includes("::") ? name.replace("::", ".") : `${module}.${name}`);
const used = new Set();
const visit = (id) => {
  if (used.has(id)) return;
  const definition = definitions[id] ?? (() => { throw new Error(`stitch calls ${id}, which the model lacks`); })();
  used.add(id);
  const walk = (node) => {
    if (!node || typeof node !== "object") return;
    if (node.k === "call") visit(qualify(definition.module, node.name));
    for (const value of Object.values(node)) walk(value);
  };
  walk(definition.body);
};
const named = new Set();
for (const path of order) {
  const text = sources[path];
  for (const [, id] of text.matchAll(/model\['([\w.]+)'\]/g)) visit(id);
  const prefix = /model\[`(Stitch\.\w+)\.\$\{name\}`\]/.exec(text)?.[1];
  if (prefix) for (const [, name] of text.matchAll(/call\('(\w+)'/g)) visit(`${prefix}.${name}`);
  for (const [, names, from] of text.matchAll(IMPORT)) {
    if (posix.join(posix.dirname(path), from) !== MODEL) continue;
    for (const name of names.split(",").map((n) => n.trim()).filter(Boolean)) {
      if (name === "model") continue;
      if (!bindings[name]) throw new Error(`${path} imports ${name}, which the model does not bind`);
      named.add(name);
      visit(bindings[name]);
    }
  }
}
const usedTypes = new Set();
const typeOf = (module, type) => {
  if (!type || ["Nat", "Bool", "Bytes", "String"].includes(type.kind)) return;
  if (type.kind === "List") return typeOf(module, type.element);
  const id = qualify(module, type.kind);
  if (!types[id]) throw new Error(`the model lacks type ${id}`);
  usedTypes.add(id);
};
for (const id of used) {
  definitions[id].params.forEach((p) => typeOf(definitions[id].module, p.type));
  typeOf(definitions[id].module, definitions[id].result);
}
const pick = (all, ids) => Object.fromEntries([...ids].sort().map((id) => [id, all[id]]));
const modelModule = `const definitions = ${JSON.stringify(pick(definitions, used))};
const types = ${JSON.stringify(pick(types, usedTypes))};
${evaluator}
${[...named].sort().map((name) => `export const ${name} = model["${bindings[name]}"];`).join("\n")}
`;

// Each module but prism-id.js keeps its own scope: its imports become reads
// of the modules it names, its exports the object it returns. prism-id.js
// stays the top level, so its exports are this module's.
const scopeName = (path) => `stitch$${path.replace(/^src\//, "").replace(/\.js$/, "").replace(/[^\w]/g, "_")}`;
const EXPORT = /^export (?:async function\*?|function\*?|class|const|let) (\w+)/gm;
function rewrite(path, text) {
  return text.replace(IMPORT, (_, names, from) => {
    const fields = names.split(",").map((n) => n.trim()).filter(Boolean).map((n) => n.replace(/\s+as\s+/, ": "));
    return `const { ${fields.join(", ")} } = ${scopeName(posix.join(posix.dirname(path), from))};\n`;
  });
}
function scoped(path, text) {
  if (/^export (?!(?:async function\*?|function\*?|class|const|let) \w)/m.test(text)) throw new Error(`${path} exports in a form this copy does not handle`);
  const exported = [...text.matchAll(EXPORT)].map(([, name]) => name);
  const body = rewrite(path, text).replace(/^export (?=(?:async function|function|class|const|let) )/gm, "");
  return `const ${scopeName(path)} = (() => {\n${body}\nreturn { ${exported.join(", ")} };\n})();\n`;
}
const parts = [scoped(MODEL, modelModule), ...order.filter((p) => p !== ENTRY).map((p) => scoped(p, sources[p])), rewrite(ENTRY, sources[ENTRY])];
const output = `// GENERATED by identity/stitch-sync.mjs from ${REPOSITORY}@${commit}. DO NOT EDIT.
// src/prism-id.js, the stitch modules it imports (${order.filter((p) => p !== ENTRY).join(", ")}),
// and the ${used.size} model decisions they call (${MODEL}); identity/stitch.lock pins them.
// Change Prism id in stitch, then run node identity/stitch-sync.mjs <commit>.
${parts.join("\n")}`;
await writeFile(OUT, output);
const lock = {
  repository: REPOSITORY,
  commit,
  sources: Object.fromEntries([...order, MODEL].sort().map((path) => [path, sha256(sources[path])])),
  decisions: [...used].sort(),
  output: { path: "identity/prism-id.mjs", sha256: sha256(output) },
};
await writeFile(LOCK, JSON.stringify(lock, null, 2) + "\n");
console.log(`prism id: stitch ${commit.slice(0, 12)}, ${order.length} modules, ${used.size} decisions, ${output.length} bytes -> identity/prism-id.mjs`);
