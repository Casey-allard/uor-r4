// Every channel the renderer calls, with the shape its contract declares, and what the host
// answers. The renderer types its calls but validates no answer at runtime, so a handler that returns
// an object where the contract says array crashes a screen ("is not iterable") only when a user opens
// it. This reads the vendored contracts and the host side by side and refuses on a mismatch.
//
//   node tools/contract_shapes.mjs          # the table
//   node tools/contract_shapes.mjs --check  # exit 1 on any array or object mismatch
//
// The contracts are TypeScript with zod, so they are bundled through esbuild (a dependency of the
// vendored renderer) into one module and read with zod's own introspection.
import { build } from "../vendor/prism/node_modules/esbuild/lib/main.js";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const typesDir = path.join(root, "vendor/prism/src/ipc/types");
const entry = path.join(root, "vendor/prism/node_modules/.cache-contract-shapes.mjs");
const modules = fs.readdirSync(typesDir).filter((f) => f.endsWith(".ts") && !f.endsWith(".d.ts") && !f.includes(".test."));
fs.writeFileSync(entry, modules.map((f, i) => `export * as m${i} from ${JSON.stringify(path.join(typesDir, f))};`).join("\n"));

const bundled = path.join(root, "vendor/prism/node_modules/.cache-contract-shapes.bundle.mjs");
await build({
  entryPoints: [entry],
  bundle: true,
  format: "esm",
  platform: "node",
  outfile: bundled,
  logLevel: "silent",
  alias: { "@": path.join(root, "vendor/prism/src") },
  loader: { ".png": "empty", ".svg": "empty", ".css": "empty" },
});
const all = await import("file://" + bundled.replace(/\\/g, "/"));

// zod 4 names the kind in `_def.type`, unwrapped through the wrappers an output may use.
function shapeOf(schema, depth = 0) {
  const def = schema?._def;
  if (!def || depth > 8) return "unknown";
  const kind = def.type ?? String(def.typeName ?? "").replace(/^Zod/, "").toLowerCase();
  if (kind === "array" || kind === "object") return kind;
  if (kind === "nullable" || kind === "optional" || kind === "default" || kind === "readonly" || kind === "catch") return shapeOf(def.innerType, depth + 1);
  if (kind === "pipe") return shapeOf(def.out ?? def.in, depth + 1);
  if (kind === "union") {
    const kinds = new Set((def.options ?? []).map((o) => shapeOf(o, depth + 1)));
    kinds.delete("null");
    return kinds.size === 1 ? [...kinds][0] : "union";
  }
  if (kind === "lazy") return shapeOf(def.getter?.(), depth + 1);
  return kind || "unknown";
}

const contracts = new Map();
const schemas = new Map();
const methods = new Map();   // the client method a screen calls -> its channel
for (const ns of Object.values(all)) {
  for (const group of Object.values(ns ?? {})) {
    if (!group || typeof group !== "object") continue;
    for (const [method, contract] of Object.entries(group)) {
      if (contract && typeof contract === "object" && typeof contract.channel === "string" && contract.output?._def) {
        contracts.set(contract.channel, shapeOf(contract.output));
        schemas.set(contract.channel, contract.output);
        methods.set(method, contract.channel);
      }
    }
  }
}

// Which channels a screen READS: the generated clients are called from react-query query functions,
// which run when the screen mounts and whose result the screen renders. Those are the calls that can
// crash a screen, so those are the ones the host must answer. A channel called from a mutation runs on
// a click and its failure is a toast, so a plain refusal there is honest and safe.
const read = new Set();
const srcDir = path.join(root, "vendor/prism/src");
const walk = (dir) => fs.readdirSync(dir, { withFileTypes: true }).flatMap((e) => (e.isDirectory() ? walk(path.join(dir, e.name)) : [path.join(dir, e.name)]));
for (const file of walk(srcDir)) {
  if (!/\.(ts|tsx)$/.test(file) || /\.test\./.test(file)) continue;
  const text = fs.readFileSync(file, "utf8");
  for (const m of text.matchAll(/queryFn\s*:/g)) {
    const window = text.slice(m.index, m.index + 300);
    for (const call of window.matchAll(/\.([A-Za-z_$][\w$]*)\s*\(/g)) {
      const channel = methods.get(call[1]);
      if (channel) read.add(channel);
    }
  }
}

// What the host answers. Only a literal answer is classified: `=> ({ ... })` is an object, `=> [...]`
// an array. A block body (`=> { ... }`) is code, never guessed at, because the runtime check in
// `shell/host.js` parses those answers against the same contracts with real data.
const host = fs.readFileSync(path.join(root, "shell/host.js"), "utf8");
const answers = new Map();
// The body is read from the text after the match, not captured by it: a capture of "the next 120
// characters" consumes them, so a handler written within 120 characters of the one before it -- most
// of the one-line handlers here -- was skipped by the scan and never checked at all.
for (const m of host.matchAll(/handlers\.set\("([^"]+)",[ \t]*/g)) {
  const channel = m[1];
  const body = host.slice(m.index + m[0].length, m.index + m[0].length + 160);
  const arrow = body.match(/^(?:async\s*)?(?:\([^)]*\)|[A-Za-z_$][\w$]*)\s*=>\s*([\s\S]*)$/);
  const tail = (arrow ? arrow[1] : "").trimStart();
  let shape = "code";
  if (/^\[/.test(tail)) shape = "array";
  else if (/^\(\s*\{/.test(tail)) shape = "object";
  else if (/^(null|undefined)/.test(tail)) shape = "null";
  else if (/^(true|false)/.test(tail)) shape = "boolean";
  else if (/^["'`]/.test(tail)) shape = "string";
  else if (/^\d/.test(tail)) shape = "number";
  answers.set(channel, shape);
}

const rows = [...answers.keys()].sort().map((channel) => ({ channel, declared: contracts.get(channel) ?? "not in a contract", answered: answers.get(channel) }));
const clash = rows.filter((r) => (r.declared === "array" && ["object", "null", "string", "number", "boolean"].includes(r.answered)) || (r.declared === "object" && r.answered === "array"));
const width = Math.max(...rows.map((r) => r.channel.length));
for (const r of rows) {
  const mark = clash.includes(r) ? "  <-- MISMATCH" : "";
  if (!process.argv.includes("--check") || mark) console.log(`${r.channel.padEnd(width)}  declared ${String(r.declared).padEnd(16)} answered ${r.answered}${mark}`);
}
console.log(`\n${rows.length} channels answered by the host, ${contracts.size} channels under contract, ${clash.length} mismatch(es)`);

// Inventory unsupported reads explicitly. An output schema accepting [] does
// not establish an implemented capability. Runtime refusal is tested separately.
const unsupported = [...read].filter((channel) => !answers.has(channel)).sort();
console.log(`\n${unsupported.length} read channels have no browser handler and return typed unsupportedChannel errors:`);
for (const channel of unsupported) console.log("  " + channel);
console.log("Shape inspection does not establish UI recovery or capability completeness.");
// The declared adapter scope prevents a removed implemented read from silently
// becoming an accepted refusal. Updating this scope is a reviewable model change.
const scope = JSON.parse(fs.readFileSync(path.join(root, "model/host-read-scope.json"), "utf8"));
const scopeErrors = [];
if (scope.schema !== "foundry/host-read-scope/1" || Object.keys(scope).sort().join(",") !== "required,schema,unavailable") throw Error("invalid host read scope");
for (const field of ["required", "unavailable"]) {
  const values = scope[field];
  if (!Array.isArray(values) || values.some(v => typeof v !== "string") || JSON.stringify(values) !== JSON.stringify([...new Set(values)].sort())) throw Error("invalid host read scope: " + field);
}
for (const channel of scope.required) {
  if (!read.has(channel) || !answers.has(channel)) scopeErrors.push("required read is missing: " + channel);
}
for (const channel of scope.unavailable) {
  if (!read.has(channel) || answers.has(channel) || scope.required.includes(channel)) scopeErrors.push("unavailable read changed: " + channel);
}
for (const channel of read) {
  if (!scope.required.includes(channel) && !scope.unavailable.includes(channel)) scopeErrors.push("unclassified read: " + channel);
}
for (const error of scopeErrors) console.error(error);
fs.rmSync(entry, { force: true });
fs.rmSync(bundled, { force: true });
if ((clash.length || scopeErrors.length) && process.argv.includes("--check")) process.exit(1);
