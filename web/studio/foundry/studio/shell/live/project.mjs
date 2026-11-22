// The live preview's projection: preview = project(spec). Pure and deterministic, so the same spec
// always gives the same screen, and every node carries a kappa (BLAKE3 of its canonical bytes): the
// frame redraws only the nodes whose kappa changed, and leaves every other one exactly as it is.
//
// The spec is the only source. A screen is drawn from a closed set of node kinds the frame knows how
// to draw (heading, text, input, answer, rule, otherwise, example); nothing the model writes is code.
// The spec's optional `view` orders and words them; without one, the default view draws them all.
//
// The input may be a spec still being written (a model streaming it), or rules that never checked,
// so `sanitize` keeps only the parts that are whole and well formed. The checked spec is the one the
// proof is about; what is drawn mid-stream is labelled a draft by the caller.
import { kappa as kappaOf } from "../prism-code/src/blake3.mjs";
import { repairJson } from "../prism.js";

const enc = new TextEncoder();
const OPS = new Set(["is", "is not", "<", "<=", ">", ">=", "=", "!="]);
const OP_WORDS = { is: "is", "is not": "is not", "<": "is under", "<=": "is at most", ">": "is over", ">=": "is at least", "=": "is", "!=": "is not" };
export const KINDS = ["heading", "text", "input", "answer", "guarantees", "rules", "examples"];
const MAX_BLOCKS = 40, MAX_TEXT = 280;

/** JSON with sorted keys: one byte string per value, whatever order the model wrote it in. */
export function canonical(value) {
  if (Array.isArray(value)) return "[" + value.map(canonical).join(",") + "]";
  if (value && typeof value === "object") return "{" + Object.keys(value).filter((k) => value[k] !== undefined).sort().map((k) => JSON.stringify(k) + ":" + canonical(value[k])).join(",") + "}";
  return JSON.stringify(value);
}
export const kappaOfValue = (value) => kappaOf(enc.encode(canonical(value)));

// A field name is one lower-case word, as Forge's check respells it (`partySize` is `partysize`), so a
// node keeps its key when the checked spec replaces the streamed one.
const word = (s) => String(s).toLowerCase().replace(/[^a-z0-9]/g, "");
const prose = (s, max = MAX_TEXT) => (typeof s === "string" ? s.replace(/\p{Cc}/gu, " ").trim().slice(0, max) : "");
const isInt = (n) => Number.isInteger(n) && n >= 0;

/** The whole, well-formed parts of a spec, or of the start of one. Never throws. */
export function sanitize(raw) {
  const s = raw && typeof raw === "object" && !Array.isArray(raw) ? raw : {};
  // `src` records where each kept item sits in the spec as written (its index there), so an edit made on
  // the screen patches that item even when an earlier one was not whole yet and was left out.
  const src = { fields: [], rules: [], properties: [], examples: [] };
  const fields = [];
  (Array.isArray(s.fields) ? s.fields : []).forEach((f, i) => {
    if (!f || typeof f.name !== "string" || !word(f.name) || fields.some((g) => g.name === word(f.name))) return;
    if (f.kind === "choice") {
      const values = Array.isArray(f.values) ? f.values.filter((v) => typeof v === "string" && v) : [];
      if (values.length) { fields.push({ name: word(f.name), kind: "choice", values, means: prose(f.means, 80) }); src.fields.push(i); }
    } else if (f.kind === "number") {
      const min = f.min === undefined ? 0 : f.min;
      if (!isInt(min) || (f.max !== undefined && (!isInt(f.max) || f.max < min))) return;
      fields.push({ name: word(f.name), kind: "number", min, ...(f.max !== undefined ? { max: f.max } : {}), means: prose(f.means, 80) });
      src.fields.push(i);
    }
  });
  const named = new Map(fields.map((f) => [f.name, f]));
  const condition = (c) => {
    if (!c || typeof c.field !== "string" || !OPS.has(c.op)) return null;
    const field = named.get(word(c.field));
    if (!field) return null;
    if (field.kind === "choice") return (c.op === "is" || c.op === "is not") && field.values.includes(c.value) ? { field: field.name, op: c.op, value: c.value } : null;
    if (c.op === "is" || c.op === "is not") return null;
    if (isInt(c.value)) return { field: field.name, op: c.op, value: c.value };
    if (c.value && typeof c.value === "object" && named.get(word(c.value.field))?.kind === "number") return { field: field.name, op: c.op, value: { field: word(c.value.field) } };
    return null;
  };
  const conditions = (list) => { if (!Array.isArray(list) || !list.length) return null; const out = list.map(condition); return out.every(Boolean) ? out : null; };
  const rules = [];
  (Array.isArray(s.rules) ? s.rules : []).forEach((r, i) => {
    const when = r && conditions(r.when);
    if (when && typeof r.then === "string" && r.then) { rules.push({ when, then: r.then, why: prose(r.why) }); src.rules.push(i); }
  });
  // A guarantee (a Prism property) is logic: it is what Lean proves, and what the table checks.
  const properties = [];
  (Array.isArray(s.properties) ? s.properties : []).forEach((p, i) => {
    const when = p && (p.kind === "never" || p.kind === "always") && conditions(p.when);
    if (when && typeof p.then === "string" && p.then) { properties.push({ kind: p.kind, when, then: p.then, why: prose(p.why) }); src.properties.push(i); }
  });
  const examples = [];
  (Array.isArray(s.examples) ? s.examples : []).forEach((e, i) => {
    if (e && typeof e.request === "string" && typeof e.response === "string" && e.request && e.response) { examples.push({ request: e.request, response: e.response }); src.examples.push(i); }
  });
  const meanings = {};
  if (s.meanings && typeof s.meanings === "object") for (const [k, v] of Object.entries(s.meanings)) if (typeof v === "string") meanings[k] = prose(v, 160);
  const out = { title: prose(s.title, 64), purpose: prose(s.purpose), fields, rules, properties, examples, meanings };
  if (typeof s.otherwise === "string" && s.otherwise) { out.otherwise = s.otherwise; out.otherwiseWhy = prose(s.otherwiseWhy); }
  out.outcomes = Array.isArray(s.outcomes) ? s.outcomes.filter((o) => typeof o === "string" && o) : [];
  if (s.view && typeof s.view === "object" && Array.isArray(s.view.blocks)) out.view = { blocks: s.view.blocks.slice(0, MAX_BLOCKS) };
  out.src = src;
  out.has = { otherwiseWhy: typeof s.otherwiseWhy === "string", meanings: !!(s.meanings && typeof s.meanings === "object") };
  return out;
}

/** What is wrong with a spec's view, in plain words: a block the frame cannot draw, or a field it names that the spec does not have. */
export function checkView(raw) {
  const issues = [];
  if (!raw || raw.view === undefined) return issues;
  if (!raw.view || !Array.isArray(raw.view.blocks)) return ["`view` is {\"blocks\": [...]}."];
  const fields = new Set((Array.isArray(raw.fields) ? raw.fields : []).map((f) => f && word(f.name)));
  if (raw.view.blocks.length > MAX_BLOCKS) issues.push(`The view has ${raw.view.blocks.length} blocks; the frame draws the first ${MAX_BLOCKS}.`);
  raw.view.blocks.slice(0, MAX_BLOCKS).forEach((b, i) => {
    if (!b || !KINDS.includes(b.kind)) issues.push(`View block ${i + 1} is not one of ${KINDS.join(", ")}.`);
    else if ((b.kind === "heading" || b.kind === "text") && !prose(b.text)) issues.push(`View block ${i + 1} (${b.kind}) has no text.`);
    else if (b.kind === "input" && !fields.has(word(b.field))) issues.push(`View block ${i + 1} shows the field \`${b.field}\`, which the rules do not have.`);
  });
  return issues;
}

const describe = (when) => when.map((c) => `${c.field} ${OP_WORDS[c.op]} ${typeof c.value === "object" ? c.value.field : c.value}`).join(" and ");

// The blocks to draw: the spec's view, or the default one. A field the view forgets still gets its
// input, before the answer, since the answer depends on every field; guarantees the view forgets are
// still shown, before the rules, since they are what the app promises.
function blocksOf(spec) {
  const fallback = [{ kind: "heading", text: spec.title }, { kind: "text", text: spec.purpose }, ...spec.fields.map((f) => ({ kind: "input", field: f.name })), { kind: "answer" }, { kind: "guarantees" }, { kind: "rules" }, { kind: "examples" }];
  let chosen = spec.view ? spec.view.blocks.map((b, i) => (b && KINDS.includes(b.kind) ? { ...b, at: i } : null)).filter(Boolean) : fallback;
  if (spec.view && !chosen.some((b) => b.kind === "guarantees")) {
    const at = chosen.findIndex((b) => b.kind === "rules");
    chosen = at < 0 ? [...chosen, { kind: "guarantees" }] : [...chosen.slice(0, at), { kind: "guarantees" }, ...chosen.slice(at)];
  }
  const shown = new Set(chosen.filter((b) => b.kind === "input").map((b) => word(b.field)));
  const missing = spec.fields.filter((f) => !shown.has(f.name)).map((f) => ({ kind: "input", field: f.name }));
  if (!missing.length) return chosen;
  const at = chosen.findIndex((b) => b.kind === "answer");
  return at < 0 ? [...chosen, ...missing] : [...chosen.slice(0, at), ...missing, ...chosen.slice(at)];
}

/** The nodes of a spec, in screen order. Keys name what a node is, so a node keeps its key while its words change. */
export function nodesOf(spec) {
  const nodes = [];
  const named = new Map(spec.fields.map((f) => [f.name, f]));
  const count = { heading: 0, text: 0 };
  const push = (key, kind, data) => { if (!nodes.some((n) => n.key === key)) nodes.push({ key, kind, data }); };
  const src = spec.src || { fields: [], rules: [], properties: [], examples: [] };
  const fieldAt = new Map(spec.fields.map((f, i) => [f.name, src.fields[i]]));
  const block = (b, slot) => (b.at !== undefined ? ["view", "blocks", b.at, slot] : null);
  for (const b of blocksOf(spec)) {
    if (b.kind === "heading" || b.kind === "text") {
      const text = prose(b.text);
      const edit = { text: block(b, "text") || [b.kind === "heading" ? "title" : "purpose"] };
      if (text) push(`${b.kind[0]}:${count[b.kind]++}`, b.kind, { text, edit });
    } else if (b.kind === "input") {
      const f = named.get(word(b.field));
      if (f) push("in:" + f.name, "input", { field: f, label: prose(b.label, 80) || f.means || f.name, edit: { label: b.at !== undefined && prose(b.label, 80) ? block(b, "label") : ["fields", fieldAt.get(f.name), "means"] } });
    } else if (b.kind === "answer") {
      if (spec.fields.length && spec.otherwise) push("answer", "answer", { label: prose(b.label, 80) || "Answer", meanings: spec.meanings, edit: { label: b.at !== undefined ? block(b, "label") : null, meaning: spec.has && spec.has.meanings ? ["meanings"] : null } });
    } else if (b.kind === "guarantees") spec.properties.forEach((p, j) => push("pr:" + j, "property", { index: j, kind: p.kind, when: describe(p.when), then: p.then, why: p.why, edit: { then: ["properties", src.properties[j], "then"], why: ["properties", src.properties[j], "why"] } }));
    else if (b.kind === "rules") {
      spec.rules.forEach((r, j) => push("rule:" + j, "rule", { index: j, when: describe(r.when), then: r.then, why: r.why, edit: { then: ["rules", src.rules[j], "then"], why: ["rules", src.rules[j], "why"] } }));
      if (spec.otherwise) push("otherwise", "otherwise", { index: spec.rules.length, then: spec.otherwise, why: spec.otherwiseWhy || "", edit: { then: ["otherwise"], why: ["otherwiseWhy"] } });
    } else if (b.kind === "examples") spec.examples.forEach((e, j) => push("ex:" + j, "example", { request: e.request, response: e.response, edit: { request: ["examples", src.examples[j], "request"], response: ["examples", src.examples[j], "response"] } }));
  }
  return nodes;
}

/** What decides: the part of the spec the interpreter reads, and the only part a proof is about. */
export function logicOf(spec) {
  return { fields: spec.fields.map(({ means, ...f }) => f), rules: spec.rules.map(({ why, ...r }) => r), otherwise: spec.otherwise ?? null, outcomes: spec.outcomes, properties: spec.properties.map(({ why, ...p }) => p) };
}

/** preview = project(spec): every node with its kappa, the logic with its kappa, and the root kappa over both. */
export function project(raw) {
  const spec = sanitize(raw);
  const nodes = nodesOf(spec).map((n) => ({ ...n, kappa: kappaOfValue({ kind: n.kind, data: n.data }) }));
  const logic = logicOf(spec);
  const logicKappa = kappaOfValue(logic);
  const root = kappaOfValue({ nodes: nodes.map((n) => [n.key, n.kappa]), logic: logicKappa });
  return { root, logicKappa, logic, nodes, title: spec.title };
}

// A model writes the spec inside <prism-spec>; mid-answer the block has no end yet.
const TAG = /<prism-spec>([\s\S]*?)(?:<\/prism-spec>|$)/;
/** The spec as far as a streaming answer has written it, or null before the block starts. Only whole
 *  values are read: the text is cut after the last comma or closing bracket outside a string, so a
 *  name or a word is never drawn half written and then redrawn under another key. */
export function partialSpec(text) {
  const m = TAG.exec(text || "");
  if (!m) return null;
  const body = m[1].replace(/^\s*```[\w-]*\s*/, "").replace(/```\s*$/, "");
  const start = body.indexOf("{");
  if (start < 0) return {};
  try { return JSON.parse(body); } catch {}
  let inString = false, escaped = false, safe = -1;
  for (let i = start; i < body.length; i++) {
    const ch = body[i];
    if (inString) { if (escaped) escaped = false; else if (ch === "\\") escaped = true; else if (ch === '"') inString = false; continue; }
    if (ch === '"') inString = true;
    else if (ch === "," || ch === "}" || ch === "]") safe = i;
  }
  if (safe < 0) return {};
  const cut = body.slice(start, body[safe] === "," ? safe : safe + 1);
  return repairJson(cut) || {};
}
