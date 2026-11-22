// Proved rules for the apps Forge builds. An app that decides something (a tier, an eligibility, a
// grade) carries a Prism spec, prism/spec/1, beside its code. This module does what prism-code does
// with it, in the tab: checks it (prism/spec.mjs, prism-code's checker), renders its LexLean model
// (prism/render.mjs, prism-code's renderer), and has Lean's kernel prove every guarantee in this tab
// (`prove`, below). It also writes the app's rule module,
// src/lib/prism.ts: the spec as data and one small interpreter, replayed against every acceptance
// vector before it is written. What Lean proves is the rules; the module is tested against them.
import { SpecError, validateSpec, unreachableRules, counterexample, describeConditions, requestLimits, decodeRequest, respond } from "./prism/spec.mjs";
import { names, renderModel, renderProofs, acceptanceVectors, renderSpecDocument } from "./prism/render.mjs";
import { readSpecFile } from "./prism/spec-file.mjs";

/**
 * The seal of a spec: the UOR content address of its canonical bytes, which the app's model
 * carries, so a built app can be held to the JSON it was built from. Taken from the spec itself,
 * never from how prism/spec.json happens to be written, because the address is of the canonical
 * form either way.
 */
const sealOf = (spec) => readSpecFile(spec).specAddress;
import { TEMPLATE_EXAMPLE, appJson, conformanceText, featureText, idsToml, lexleanLockText, rustTestText, suiteOf } from "./prism/template.mjs";
import { toLexLean, draftOf } from "./live/lexlean.mjs";
import { checkObligations } from "./live/obligations.mjs";
import { tableKernel } from "./live/table.mjs";
import { PROMPT_RULES, facetLine, parsePromptTag, facetsOf } from "./prism/builder/route.mjs";
// The app's prompt parts, read and projected where the rules are (host.js rulesAfterTurn).
export { parsePromptTag, facetsOf };
import { verdictWithGuest, verdictAddressWords, verdictAxiomSet } from "./in-tab-verdict-codec.mjs";

// prism-code's checker (prism/spec.mjs, byte for byte) keeps a spec to 24 rules, because its proofs are one
// linear_arithmetic theorem per guarantee over the whole first-match chain. Past that, Forge checks the
// guarantees itself, by local obligations (live/obligations.mjs), or the table when a rule is not a box,
// and never offers the rules for proof until LexLean can prove them locally (UOR-Foundation/PrismPM#605).
export const PROVE_RULES_MAX = 24;

// prism/render.mjs names bytes in hex through Node's Buffer; this is the one use it makes of it.
if (!globalThis.Buffer) globalThis.Buffer = { from: (bytes) => ({ toString: (encoding) => { if (encoding !== "hex") throw new Error("only hex"); return Array.from(bytes, (b) => b.toString(16).padStart(2, "0")).join(""); } }) };

const enc = new TextEncoder();

// ---- what the model is told
// The rules are written as one JSON spec (prism/spec/1), the app's only input. Forge renders the app's
// canonical LexLean module from it (shell/live/lexlean.mjs toLexLean), the same module from the same
// JSON, and Lean proves the guarantees. A model never writes LexLean for a decision app: it is the
// one step that is not deterministic, so it is kept to the smallest thing that says what the app does.
const PARKING = {
  name: "parking-fee", title: "Parking Fee", purpose: "Decide the fee tier from hours parked and vehicle.",
  fields: [{ name: "hours", kind: "number", min: 0, max: 24 }, { name: "vehicle", kind: "choice", values: ["car", "motorcycle", "truck"] }],
  outcomes: ["free", "standard", "premium"],
  rules: [{ when: [{ field: "hours", op: "=", value: 0 }], then: "free" }, { when: [{ field: "vehicle", op: "is", value: "truck" }], then: "premium" }, { when: [{ field: "hours", op: ">", value: 12 }], then: "premium" }],
  otherwise: "standard",
  properties: [{ kind: "never", when: [{ field: "hours", op: "=", value: 0 }], then: "premium" }],
  examples: [{ request: "0 truck", response: "free" }, { request: "13 car", response: "premium" }, { request: "2 car", response: "standard" }],
};
export const SPEC_EXAMPLE = JSON.stringify(PARKING);
/** The example's canonical module in the draft form, without its theorems: what the module reader is held to. */
export const MODULE_EXAMPLE = draftOf(toLexLean(PARKING)).replace(/\n\ntheorem\{[\s\S]*?\n\}(?=\n)/g, "").trim() + "\n";
export const SPEC_RULES = `# You write the app's spec as JSON, and nothing else
Every app here is one JSON spec: its inputs, its answers, its rules and its guarantees. Forge writes the app's LexLean module, its code and its screen from the spec, the same every time, and Lean proves the guarantees. You never write code, files or dependencies: Forge does not apply them. You never write LexLean. To build or change an app, write its spec; to change only one part, write a patch. If a request decides nothing, say in one sentence what the app could decide, and write that spec.
- Write one <prism-spec> block holding the whole spec as one JSON object, and no other text inside it. Keys: "title"; "purpose" (one sentence); "fields" (1 to 8, in the order a request gives them; each {"name": a lower-case word, "kind": "number" with whole "min" and "max", or "choice" with "values", two to 32 lower-case words}); "outcomes" (the answer words, lower case; "invalid" is reserved); "rules" (1 to 24, in order, the first that matches decides; each {"when": [{"field", "op", "value"}], "then": an outcome}); "otherwise" (the outcome when no rule matches); "properties" (what must never or always happen, each a guarantee Lean proves for every request: {"kind": "never" or "always", "when": [...], "then": an outcome}); "examples" (a few requests with their answers: {"request": "the field values separated by spaces", "response"}).
- A condition's "op" is "is" or "is not" for a choice field, with one of its values; and "<", "<=", ">", ">=", "=" or "!=" for a number field, with a whole number or another number field as {"field": "<name>"}. All of a rule's conditions must hold. Numbers are whole, never fractions.
- Think about which rules and guarantees the app needs, not about syntax. State the guarantees the request implies: what must never be answered, and what must always be.
- To change one part (a word, an answer, one rule), or to set the screen's words or order, write a patch on the spec: <prism-spec-patch>[{"op":"set","path":["rules",2,"then"],"value":"none"}]</prism-spec-patch>. Ops are set, insert and remove; \`view\` ({"blocks":[...]}, each one of {"kind":"heading","text"}, {"kind":"text","text"}, {"kind":"input","field","label"}, {"kind":"answer","label"}, {"kind":"guarantees"}, {"kind":"rules"}, {"kind":"examples"}), \`purpose\` (one sentence) and \`meanings\` ({"<answer>": "what it tells the user"}, one key per answer; fields have no meanings) are set this way. When the user points at something on the screen, its paths are given with the request.
- To change the rules, write the whole spec again in a new <prism-spec> block, starting from the current spec when one is given.
- An app that must keep what it is given between requests (a list that grows, entries that are kept) cannot be written yet. Say so in one sentence, and write the spec of what it could decide.
Example: <prism-spec>${SPEC_EXAMPLE}</prism-spec>` + PROMPT_RULES;

// ---- the one thing a model may write
// Every model, on the device or in the cloud, writes the app's JSON spec and nothing else. LexLean a
// model writes is never read: the module is Forge's own rendering of the spec, so the same JSON is the
// same app on every device, and the app's seal is the address of that JSON.
/** Whether a model's turn carries LexLean, closed or still open. */
export const wroteLexLean = (text) => String(text || "").includes("<prism-lexlean>");
/** A model's turn without the LexLean it wrote. */
export const withoutLexLean = (text) => String(text || "").replace(/<prism-lexlean>[\s\S]*?(<\/prism-lexlean>|$)/g, "");

/**
 * What a model is shown of a decision app: its current spec, as the JSON it writes, or that there is
 * none. Never the LexLean module, which Forge renders from the spec.
 */
export function specContext(specText) {
  let spec = null;
  try { spec = specText ? JSON.parse(specText) : null; } catch {}
  if (!spec) return "This app has no spec yet. Write it from my request.";
  return "The app's current spec:\n<prism-spec>" + JSON.stringify(spec).replaceAll("<", "\\u003c") + "</prism-spec>";
}

/** What a model is shown of the app: its current module, in the draft form it writes, or that there is none. */
export function moduleContext(source) {
  if (!source) return "This app has no LexLean module yet. Write it from my request.";
  return "The app's current LexLean module:\n<prism-lexlean>\n" + draftOf(source.replace(/\n\ntheorem\{[\s\S]*?\n\}(?=\n)/g, "")).trim() + "\n</prism-lexlean>";
}
export const CODE_REFUSED = "**Code not applied.** The app's code and screen are drawn from its LexLean module; files a model writes are not used.";

// What the device wrote, without a reasoning block the model may still open.
export const withoutThinking = (text) => (text || "").replace(/<think>[\s\S]*?(<\/think>|$)/g, "").trim();

// The screen Forge writes when the rules check and no page uses them yet: a form with one input per
// field, answered by decide(). It reads everything from the rules module, so it never goes stale.
export function screenSource() {
  return `// Written by Prism Foundry: a form for the app's rules. The answer comes from decide(), the rules Lean proves.
import { useState } from "react";
import { decide, FIELDS, SPEC } from "@/lib/prism";

export default function Index() {
  const [values, setValues] = useState(() => Object.fromEntries(FIELDS.map((f) => [f.name, f.kind === "choice" ? f.values[0] : String(f.min ?? 0)])));
  const answer = decide(values);
  const set = (name, value) => setValues((v) => ({ ...v, [name]: value }));
  return (
    <main className="min-h-screen flex items-center justify-center bg-background p-6">
      <div className="w-full max-w-md space-y-6">
        <div className="space-y-1">
          <h1 className="text-2xl font-semibold tracking-tight">{SPEC.title}</h1>
          <p className="text-sm text-muted-foreground">{SPEC.purpose}</p>
        </div>
        {FIELDS.map((f) => (
          <label key={f.name} className="block space-y-1.5">
            <span className="text-sm font-medium">{f.means || f.name}</span>
            {f.kind === "choice" ? (
              <select className="w-full rounded-md border bg-background px-3 py-2" value={values[f.name]} onChange={(e) => set(f.name, e.target.value)}>
                {f.values.map((v) => <option key={v} value={v}>{v}</option>)}
              </select>
            ) : (
              <input className="w-full rounded-md border bg-background px-3 py-2" type="number" inputMode="numeric" step={1} min={f.min} max={f.max} value={values[f.name]} onChange={(e) => set(f.name, e.target.value)} />
            )}
          </label>
        ))}
        <div className="rounded-lg border p-4 space-y-1">
          <div className="text-sm text-muted-foreground">Answer</div>
          <div className="text-2xl font-semibold">{answer}</div>
          {SPEC.meanings && SPEC.meanings[answer] ? <div className="text-sm text-muted-foreground">{SPEC.meanings[answer]}</div> : null}
        </div>
      </div>
    </main>
  );
}
`;
}

// A small model may stop before the closing tag: the block runs to the end of the answer then.
const TAG = /<prism-spec>([\s\S]*?)(?:<\/prism-spec>|$)/;
export function parseSpecTag(text) {
  const m = TAG.exec(text || "");
  if (!m) return null;
  let body = m[1].trim();
  const fence = body.match(/^```[\w-]*\n([\s\S]*?)\n```$/);
  if (fence) body = fence[1];
  try { return { spec: JSON.parse(body) }; } catch (e) {
    const repaired = repairJson(body);
    if (repaired !== null) {
      // An answer cut short leaves its last example half written: only whole examples are kept.
      if (Array.isArray(repaired.examples)) repaired.examples = repaired.examples.filter((e) => e && typeof e.request === "string" && typeof e.response === "string");
      return { spec: repaired, repaired: true };
    }
    return { error: "The rules are not valid JSON: " + e.message };
  }
}
// The slips a small model makes in JSON: text after the closing brace, closing brackets it never wrote,
// commas before a closer. Repair only restores syntax; the checker still judges every rule, and the
// user still approves the card, so a repaired spec is never proved unseen.
export function repairJson(text) {
  // A quote glued to a closer, `}"]`, as BitNet writes it: the tokenizer's `}"` taken for `}`.
  text = text.replace(/([}\]])"(?=\s*[\]},])/g, "$1");
  // A doubled comma, and an operator glued to its key, `"op">=","value"`, as Bonsai writes them.
  text = text.replace(/,(\s*,)+/g, ",").replace(/"op"\s*(>=|<=|!=|>|<|=)\s*"\s*,/g, '"op":"$1",');
  const start = text.indexOf("{");
  if (start < 0) return null;
  const stack = [];
  const cuts = []; // a comma between complete items, with the closers still open there
  let inString = false, escaped = false, end = -1, out = "";
  for (let i = start; i < text.length; i++) {
    const ch = text[i];
    out += ch;
    if (inString) { if (escaped) escaped = false; else if (ch === "\\") escaped = true; else if (ch === '"') inString = false; continue; }
    if (ch === '"') inString = true;
    else if (ch === "{" || ch === "[") stack.push(ch === "{" ? "}" : "]");
    else if (ch === "}" || ch === "]") { if (stack.pop() !== ch) return null; if (!stack.length) { end = i; break; } }
    else if (ch === ",") cuts.push({ at: out.length - 1, closers: stack.slice().reverse().join("") });
  }
  const attempt = (s) => { try { return JSON.parse(s.replace(/,\s*([}\]])/g, "$1")); } catch { return null; } };
  if (end >= 0) return attempt(out);
  // Cut short (the model's token limit): close what is open, else go back to the last complete item.
  let tail = out + (inString ? '"' : "");
  tail = tail.replace(/[,:\s]+$/, "") + stack.slice().reverse().join("");
  const whole = attempt(tail);
  if (whole) return whole;
  for (let k = cuts.length - 1; k >= 0; k--) { const got = attempt(out.slice(0, cuts[k].at) + cuts[k].closers); if (got) return got; }
  return null;
}
// What the chat shows of an answer. The module is the app's source, not something to read in the chat:
// while it streams it shows as one line counting its declarations, and after the turn the card stands in
// its place (in the last module's place, when a repair wrote it again); a patch shows as nothing, its result
// is on the card. The answer itself (the model's text) is kept whole for reading the module and patches.
const MODULE_ENDS = ["</prism-lexlean>", "\\end{prism-lexlean}", "\\end{lexlean}"];
export function shownText(text, card = null) {
  // The model's reasoning (shown as <think>…) names the tag in passing: it is shown as it is.
  const t = text || "", think = /^<think>[\s\S]*?(<\/think>|$)/.exec(t);
  if (think) return think[0] + (think[1] ? shownText(t.slice(think[0].length), card) : "");
  let out = "", rest = t;
  for (;;) {
    const at = rest.indexOf("<prism-lexlean>");
    if (at < 0) { out += rest; break; }
    out += rest.slice(0, at);
    const body = rest.slice(at);
    let end = -1;
    const close = body.indexOf("</prism-lexlean>");
    if (close >= 0) end = close + 16;
    else for (const m of MODULE_ENDS.slice(1)) { const k = body.lastIndexOf(m); if (k >= 0) end = Math.max(end, k + m.length); }
    const block = end >= 0 ? body.slice(0, end) : body;
    // An app written in the app notation is lines, not declarations: Forge writes the module from them.
    const lines = /^<prism-lexlean>\s*app\s/.test(block) && !block.includes("\\begin{") ? block.replace(/<\/?prism-lexlean>/g, "").split("\n").filter((l) => l.trim()).length : 0;
    const n = lines || (block.match(/\b(definition|structure)\{/g) || []).length;
    out += end >= 0 ? `\u0000${lines ? "L" : ""}${n}\u0000` : lines ? `\n\n_Writing the app: ${n} line${n === 1 ? "" : "s"} so far…_\n\n` : `\n\n_Writing the app's LexLean module: ${n} declaration${n === 1 ? "" : "s"} so far…_\n\n`;
    if (end < 0) break;
    rest = body.slice(end);
  }
  const marks = [...out.matchAll(/\u0000(L?)(\d+)\u0000/g)];
  marks.forEach((m, i) => { out = out.replace(m[0], i === marks.length - 1 ? (card ? "\n\n" + card + "\n\n" : m[1] ? `\n\n_The app, written in ${m[2]} line${m[2] === "1" ? "" : "s"}. Forge is writing its LexLean module from them._\n\n` : `\n\n_The app's LexLean module: ${m[2]} declarations._\n\n`) : ""); });
  // The spec is the app's source, not something to read in the chat: one line while it streams, and the
  // card (or one line) in its place once it is whole.
  out = out.replace(/<prism-spec>[\s\S]*?<\/prism-spec>/g, card ? "\n\n" + card + "\n\n" : "\n\n_The app's spec, written as JSON._\n\n").replace(/<prism-spec>[\s\S]*$/, "\n\n_Writing the app's spec…_\n\n");
  // The app's prompt parts are shown through the card's full-model line, not as JSON.
  out = out.replace(/<prism-spec-patch>[\s\S]*?(<\/prism-spec-patch>|$)/g, "").replace(/<prism-prompt>[\s\S]*?(<\/prism-prompt>|$)/g, "").replace(/\n{3,}/g, "\n\n");
  // A tag still arriving (`<pri…`) is not shown until it is whole.
  const tail = /<[a-z-]*$/.exec(out);
  return tail && ("<prism-lexlean>".startsWith(tail[0]) || "<prism-spec-patch>".startsWith(tail[0]) || "<prism-spec>".startsWith(tail[0]) || "<prism-prompt>".startsWith(tail[0])) ? out.slice(0, tail.index) : out;
}
// The card stands in its own paragraphs, whatever the model wrote around the tag or the module.
export const withCard = (text, card) => {
  const t = text || "";
  return /<prism-lexlean>|<prism-spec>/.test(t) ? shownText(t, card) : t.replace(TAG, card ? "\n\n" + card + "\n\n" : "");
};
export const isApproval = (prompt) => /^\s*\/?(yes,?\s*)?approve(d)?(\s+(the\s+)?(rules|guarantees|spec))?\s*[.!]?\s*$/i.test(prompt || "");
// "publish" builds the proved app with the Prism SDK and puts it live; no model is asked.
export const isPublish = (prompt) => /^\s*\/?(yes,?\s*)?(publish|ship)(\s+(it|the\s+app|now))?\s*[.!]?\s*$/i.test(prompt || "");

// ---- the app's rule module: these two functions are shipped verbatim (their source text) in
// src/lib/prism.ts, and replayed against every acceptance vector before they are.
function prismAnswer(spec, request) {
  const line = String(request);
  if (new TextEncoder().encode(line).length > Math.min(512, Math.max(64, 24 * spec.fields.length + 16))) return "invalid";
  const parts = line.split(" ");
  if (parts.length !== spec.fields.length) return "invalid";
  const values = {};
  for (let i = 0; i < spec.fields.length; i++) {
    const field = spec.fields[i], part = parts[i];
    if (field.kind === "choice") {
      if (!field.values.includes(part)) return "invalid";
      values[field.name] = part;
    } else {
      if (!/^(0|[1-9][0-9]*)$/.test(part) || part.length > 18) return "invalid";
      const n = BigInt(part);
      if (field.min !== undefined && n < BigInt(field.min)) return "invalid";
      if (field.max !== undefined && n > BigInt(field.max)) return "invalid";
      values[field.name] = n;
    }
  }
  const operand = (value) => (typeof value === "object" ? values[value.field] : BigInt(value));
  const holds = (c) => {
    const left = values[c.field];
    switch (c.op) {
      case "is": return left === c.value;
      case "is not": return left !== c.value;
      case "<": return left < operand(c.value);
      case "<=": return left <= operand(c.value);
      case ">": return left > operand(c.value);
      case ">=": return left >= operand(c.value);
      case "=": return left === operand(c.value);
      case "!=": return left !== operand(c.value);
      default: return false;
    }
  };
  for (const rule of spec.rules) if (rule.when.every(holds)) return rule.then;
  return spec.otherwise;
}
function prismDecide(spec, values) {
  return prismAnswer(spec, spec.fields.map((f) => String(values && values[f.name] !== undefined && values[f.name] !== null ? values[f.name] : "")).join(" "));
}

// The same two functions, as the live frame runs them (shell/live/frame.js): one interpreter wherever an
// answer is shown.
export { prismAnswer, prismDecide };
export const INTERPRETER_PARTS = Object.freeze([prismAnswer.toString(), prismDecide.toString()]);
export const INTERPRETER = INTERPRETER_PARTS.join("\n") + "\n";

function moduleSource(spec, vectors) {
  return `// Generated by Prism Foundry from prism/spec.json. Do not edit: change the rules in the chat.
// Lean proves the guarantees about these rules. This module answers with the same rules; Forge
// replayed all ${vectors} acceptance vectors through it before writing it.
export const SPEC = ${JSON.stringify(spec, null, 2)};
${INTERPRETER_PARTS[0]}
${INTERPRETER_PARTS[1]}
export const FIELDS = SPEC.fields;
/** The answer for one set of field values: an outcome word, or "invalid". */
export function decide(values) { return prismDecide(SPEC, values); }
/** The answer for one request line, the field values separated by spaces. */
export function answer(request) { return prismAnswer(SPEC, request); }
`;
}

// ---- the check: what prism_spec runs on a drafted spec, then the module against every vector
// Answer words and field names are lower-case words in a Prism spec. A model that capitalizes an answer
// (grades A to F) or writes debt_ratio or low_fee means the same thing, so each is respelled wherever it is
// named; no rule changes. An answer or a choice is a Prism word (letters, digits, hyphens): low_fee is low-fee.
function lowerAnswers(input) {
  const respelled = new Map();
  const low = (w) => {
    if (typeof w !== "string") return w;
    const word = w.trim().toLowerCase().replace(/[\s_]+/g, "-").replace(/[^a-z0-9-]/g, "");
    if (word !== w.toLowerCase()) respelled.set(w, word);
    return word;
  };
  const spec = structuredClone(input || {});
  if (Array.isArray(spec.outcomes)) spec.outcomes = spec.outcomes.map(low);
  spec.otherwise = low(spec.otherwise);
  for (const r of Array.isArray(spec.rules) ? spec.rules : []) if (r) r.then = low(r.then);
  for (const p of Array.isArray(spec.properties) ? spec.properties : []) if (p) p.then = low(p.then);
  for (const e of Array.isArray(spec.examples) ? spec.examples : []) if (e) e.response = low(e.response);
  // A meaning keyed by the answer's LexLean name (measured: GLM wrote `answerA` for answer `a`) is that
  // answer's meaning; only a key that names no answer either way is left for the checker to refuse.
  const answerWord = (k) => { const m = /^answer([A-Z][\w]*)$/.exec(k); return m ? m[1].replace(/([a-z0-9])([A-Z])/g, "$1-$2").toLowerCase() : null; };
  if (spec.meanings && typeof spec.meanings === "object") spec.meanings = Object.fromEntries(Object.entries(spec.meanings).map(([k, v]) => [Array.isArray(spec.outcomes) && !spec.outcomes.includes(low(k)) && spec.outcomes.includes(answerWord(k)) ? answerWord(k) : low(k), v]));
  // A choice is respelled the same way, in its field, in every condition on that field, and in every
  // example request that names it.
  const choices = new Map();
  for (const f of Array.isArray(spec.fields) ? spec.fields : []) {
    if (!f || f.kind !== "choice" || !Array.isArray(f.values)) continue;
    f.values = f.values.map((v) => { const w = low(v); if (w !== v) choices.set(v, w); return w; });
    for (const r of [...(Array.isArray(spec.rules) ? spec.rules : []), ...(Array.isArray(spec.properties) ? spec.properties : [])]) for (const c of (r && Array.isArray(r.when) ? r.when : [])) if (c && c.field === f.name && typeof c.value === "string") c.value = low(c.value);
  }
  if (choices.size) for (const e of Array.isArray(spec.examples) ? spec.examples : []) if (e && typeof e.request === "string") e.request = e.request.split(" ").map((t) => choices.get(t) ?? t).join(" ");
  // A field name is one lower-case word: `debt_ratio` becomes `debtratio` wherever a condition names it.
  const renamed = new Map();
  for (const f of Array.isArray(spec.fields) ? spec.fields : []) {
    if (!f || typeof f.name !== "string") continue;
    const word = f.name.toLowerCase().replace(/[^a-z0-9]/g, "");
    if (word && word !== f.name) { renamed.set(f.name, word); f.name = word; }
  }
  if (renamed.size) {
    const fix = (c) => { if (!c) return; if (renamed.has(c.field)) c.field = renamed.get(c.field); if (c.value && typeof c.value === "object" && renamed.has(c.value.field)) c.value.field = renamed.get(c.value.field); };
    for (const r of [...(Array.isArray(spec.rules) ? spec.rules : []), ...(Array.isArray(spec.properties) ? spec.properties : [])]) for (const c of (r && Array.isArray(r.when) ? r.when : [])) fix(c);
  }
  return { spec, respelled };
}

/** A spec over prism-code's rule limit, checked by Forge: obligations first, the table for every request. */
function checkLarge(input, notes) {
  const pad = (i) => String(i + 1).padStart(3, "0");
  const spec = { examples: [], limits: [], ...input, rules: input.rules.map((r, i) => ({ id: `REQ-${pad(i)}`, ...r })), properties: (input.properties || []).map((p, i) => ({ id: `INV-${pad(i)}`, ...p })) };
  const issues = [];
  const ob = checkObligations(spec);
  let how;
  if (ob.ok && ob.holds) how = `${ob.total} local obligations, each keyed by what it depends on`;
  else {
    // A break the obligations report, or rules they do not cover, is settled exactly, request by request.
    const K = tableKernel(), table = K.build(spec);
    if (table.reason) return { ok: false, issues: [`More than ${PROVE_RULES_MAX} rules, and ${table.reason}, so Forge cannot check every request.`] };
    how = `every one of its ${table.size.toLocaleString()} requests`;
    K.check(spec, table).forEach((c, i) => { if (!c.ok) issues.push(`${spec.properties[i].id} does not hold: \`${c.request}\` answers ${c.outcome}.`); });
  }
  const vectors = acceptanceVectors(spec);
  let failed = 0;
  for (const v of vectors) {
    let text;
    try { text = new TextDecoder("utf-8", { fatal: true }).decode(v.bytes); } catch { if (v.response !== "invalid") failed++; continue; }
    if (prismAnswer(spec, text) !== v.response) failed++;
  }
  if (failed) issues.push(`The rule module disagrees with ${failed} of ${vectors.length} acceptance vectors. This is a Forge bug.`);
  notes.push(`${spec.rules.length} rules, more than prism-code's ${PROVE_RULES_MAX}: every guarantee was checked by ${how}.`);
  return { ok: issues.length === 0, spec, issues, notes, vectors: vectors.length, module: issues.length ? null : moduleSource(spec, vectors.length) };
}

export function checkSpec(raw) {
  const issues = [], notes = [];
  const lowered = lowerAnswers(raw);
  let input = lowered.spec;
  if (Array.isArray(input.rules) && input.rules.length > PROVE_RULES_MAX) return checkLarge(input, notes);
  const words = [...lowered.respelled].filter(([a, b]) => b && a !== b);
  if (words.length) notes.push("Forge respelled " + words.map(([a, b]) => `\`${a}\` as \`${b}\``).join(", ") + ", the form Prism words take.");
  // `limits` is a list; a model that writes one sentence means a list of one.
  if (typeof input.limits === "string") { input.limits = [input.limits]; notes.push("Forge made `limits` a list."); }
  // A model may write the otherwise answer as the last rule ({"otherwise": "standard"}); it means the otherwise.
  if (Array.isArray(input.rules)) {
    const stray = input.rules.filter((r) => r && !r.when && typeof r.otherwise === "string");
    if (stray.length) { input.rules = input.rules.filter((r) => !stray.includes(r)); if (!input.otherwise) input.otherwise = stray[stray.length - 1].otherwise; notes.push("Forge read the rule `otherwise` as the answer when no rule matches."); }
  }
  // An example is the model's own word for what the rules answer. One that is not a request, or that the
  // rules answer differently, is set aside and named on the card, so the user sees the disagreement.
  try {
    const bare = validateSpec({ ...input, examples: [] });
    const kept = [];
    for (const e of Array.isArray(input.examples) ? input.examples : []) {
      const request = e && typeof e.request === "string" ? e.request : "";
      if (!decodeRequest(bare, request)) { notes.push(`Set aside example \`${request}\`: it is not a request (${bare.fields.map((f) => f.name).join(" ")}).`); continue; }
      const answer = respond(bare, new TextEncoder().encode(request));
      if (answer !== e.response) { notes.push(`Set aside example \`${request}\` → ${e.response}: these rules answer ${answer}.`); continue; }
      kept.push(e);
    }
    input = { ...input, examples: kept };
  } catch (e) { if (!(e instanceof SpecError)) throw e; }
  let spec;
  try { spec = validateSpec(input); } catch (e) { if (e instanceof SpecError) return { ok: false, issues: [e.message] }; throw e; }
  // A rule no request reaches never decides anything, so dropping it changes no answer; prism-code refuses
  // such a spec, so Forge drops them and says so. unreachableRules names one only after trying every request.
  const dead = unreachableRules(validateSpec({ ...spec, properties: [], examples: [] }));
  if (dead.length) {
    const deadSet = new Set(dead);
    try { spec = validateSpec({ ...input, rules: spec.rules.filter((r) => !deadSet.has(r.id)).map(({ id, says, ...r }) => r) }); }
    catch (e) { if (e instanceof SpecError) return { ok: false, issues: [e.message] }; throw e; }
    notes.push(`Dropped ${dead.length} rule${dead.length === 1 ? "" : "s"} no request reaches: an earlier rule always decides first.`);
  }
  for (const p of spec.properties) {
    const found = counterexample(spec, p);
    if (found.request !== null) issues.push(`${p.id} does not hold: \`${found.request}\` answers ${found.outcome}${found.rule ? ` by ${found.rule}` : " by otherwise"}.`);
  }
  const vectors = acceptanceVectors(spec);
  let failed = 0;
  for (const v of vectors) {
    let text;
    try { text = new TextDecoder("utf-8", { fatal: true }).decode(v.bytes); } catch { if (v.response !== "invalid") failed++; continue; }
    if (prismAnswer(spec, text) !== v.response) failed++;
  }
  if (failed) issues.push(`The rule module disagrees with ${failed} of ${vectors.length} acceptance vectors. This is a Forge bug.`);
  return { ok: issues.length === 0, spec, issues, notes, vectors: vectors.length, module: issues.length ? null : moduleSource(spec, vectors.length) };
}

// ---- shell files, as text
async function shellText(path) {
  const r = await fetch(new URL(path, import.meta.url));
  if (!r.ok) throw new Error(path + " answered " + r.status);
  return (await r.text()).replace(/\r\n/g, "\n");
}

// ---- the proof of a rule app: Lean's kernel, in this tab
// A rule app answers through the Forge kernel's RuleJudge, on the words `ruleWordsOf` writes. Lean proved, once for
// every app, that RuleJudge as LexLean compiles it answers no in-bounds request that breaks a guarantee
// `wordsCheck` passes (studio/proof/PrismLive/Regions.lean, `ruleJudge_sound`). An app's proof is then that theorem
// applied to its words: Lean's kernel, compiled to WebAssembly and run in a worker with that library resident,
// evaluates `wordsCheck` and judges the term. Nothing is sent anywhere, and the Forge kernel guest gives the verdict
// (InTabKernel.verdictCode) from what the tab derived.
const OPS = { "<": 0, "<=": 1, ">": 2, ">=": 3, is: 4, "=": 4, "is not": 5, "!=": 5 };
/** The words RuleJudge decides an app's requests with, before the request's own: what a published app sends. */
export function ruleWordsOf(spec) {
  const outcomes = [...new Set([spec.otherwise, ...spec.rules.map((r) => r.then)])], at = (n) => spec.fields.findIndex((f) => f.name === n);
  const cond = (c) => { const f = spec.fields[at(c.field)]; if (!f || typeof c.value === "object" || !(c.op in OPS)) throw new Error("it compares two fields, and the rule judge compares a field with a value"); return [at(c.field), OPS[c.op], f.kind === "choice" ? f.values.indexOf(c.value) : Number(c.value)]; };
  const words = [1, spec.fields.length, ...spec.fields.flatMap((f) => (f.kind === "choice" ? [0, f.values.length - 1] : [f.min ?? 0, f.max ?? 0xffffffff])), outcomes.indexOf(spec.otherwise), spec.rules.length, ...spec.rules.flatMap((r) => [r.when.length, outcomes.indexOf(r.then), ...r.when.flatMap(cond)])];
  // A guarantee about an answer no rule gives names the next number: no rule answers it.
  const guarantees = spec.properties.map((p) => ({ always: p.kind === "always", when: p.when.map(cond), outcome: outcomes.includes(p.then) ? outcomes.indexOf(p.then) : outcomes.length }));
  return { words, outcomes, guarantees, choices: spec.fields.map((f) => (f.kind === "choice" ? f.values : null)) };
}
const AXIOMS = ["propext", "Quot.sound"];
const sha256Hex = async (bytes) => [...new Uint8Array(await crypto.subtle.digest("SHA-256", bytes))].map((b) => b.toString(16).padStart(2, "0")).join("");
function leanWorker(self) {
  const enc = new TextEncoder(), dec = new TextDecoder();
  const sha256 = async (b) => [...new Uint8Array(await crypto.subtle.digest("SHA-256", b))].map((x) => x.toString(16).padStart(2, "0")).join("");
  const said = [];
  let m = null, lib = null, at = null;
  const run = (entry, bytes) => {
    said.length = 0; m.FS.writeFile("/in.ndjson", bytes);
    let exit;
    try { exit = m.ccall(entry, "number", ["string"], ["/in.ndjson"]); } catch (e) { exit = 3; said.push("trap: " + (e && e.message ? e.message : e)); }
    m.FS.unlink("/in.ndjson");
    const axioms = {}, rest = [];
    for (const line of said) { const a = /^axioms ([^:]+): ?(.*)$/.exec(line); if (a) axioms[a[1]] = a[2] ? a[2].split(", ") : []; else rest.push(line); }
    return { exit, axioms: exit === 0 ? axioms : {}, output: rest.join("\n").slice(-400) };
  };
  // The library's names, terms and levels by index (lean4export's format), so a suffix names what it already holds.
  const read = (text) => {
    const t = { names: [""], name: new Map([["", 0]]), expr: [], exprAt: new Map(), level: new Map(), types: new Map(), next: [1, 0, 1] };
    for (const line of text.split("\n")) {
      if (!line) continue;
      const o = JSON.parse(line);
      if ("in" in o) { const pre = t.names[(o.str || o.num).pre]; const full = (pre ? pre + "." : "") + (o.str ? o.str.str : o.num.i); t.names[o.in] = full; t.name.set(full, o.in); t.next[0] = Math.max(t.next[0], o.in + 1); }
      else if ("il" in o) { const { il, ...node } = o; t.level.set(JSON.stringify(node), il); t.next[2] = Math.max(t.next[2], il + 1); }
      else if ("ie" in o) { const { ie, ...node } = o; t.expr[ie] = node; t.exprAt.set(JSON.stringify(node), ie); t.next[1] = Math.max(t.next[1], ie + 1); }
      else if (o.thm || o.def) { const d = o.thm || o.def; t.types.set(d.name, d.type); }
    }
    return t;
  };
  // What a suffix of the library is written with: names, levels and terms by index, new ones after the library's.
  const writer = () => {
    const out = [], own = new Map(), ownExpr = [], ownAt = new Map();
    let [nextName, nextExpr, nextLevel] = lib.next;
    const term = (node) => { const k = JSON.stringify(node), have = lib.exprAt.get(k) ?? ownAt.get(k); if (have !== undefined) return have; const e = nextExpr++; ownExpr[e] = node; ownAt.set(k, e); out.push(JSON.stringify({ ...node, ie: e })); return e; };
    const node = (e) => lib.expr[e] ?? ownExpr[e];
    const name = (full) => { let pre = 0, path = ""; for (const part of full.split(".")) { path = path ? path + "." + part : part; const have = lib.name.get(path) ?? own.get(path); if (have !== undefined) { pre = have; continue; } const n = nextName++; own.set(path, n); out.push(JSON.stringify({ in: n, str: { pre, str: part } })); pre = n; } return pre; };
    const known = (full) => { const n = lib.name.get(full); if (n === undefined) throw new Error("the library does not hold " + full); return n; };
    const C = (full, us = []) => term({ const: { name: known(full), us } });
    const app = (f, ...args) => args.reduce((fn, arg) => term({ app: { arg, fn } }), f);
    const nat = (n) => term({ natVal: String(n) });
    const one = lib.level.get(JSON.stringify({ succ: 0 })) ?? (() => { const l = nextLevel++; out.push(JSON.stringify({ il: l, succ: 0 })); return l; })();
    const list = (T, xs) => xs.reduceRight((tail, x) => app(C("List.cons", [0]), T, x, tail), app(C("List.nil", [0]), T));
    const declare = (kind, full, type, value) => { const n = name(full); out.push(JSON.stringify(kind === "def" ? { def: { all: [n], hints: { regular: 1 }, levelParams: [], name: n, safety: "safe", type, value } } : { thm: { all: [n], levelParams: [], name: n, type, value } })); return term({ const: { name: n, us: [] } }); };
    return { out, own, term, node, name, known, C, app, nat, one, list, declare };
  };
  // A stateful app's declarations, written locally nameless ({c, us} {a} {lam} {pi} {v} {n} {s}; a level is 0 or 1).
  const written = (decls) => {
    const { out, own, term, name, one, declare } = writer();
    const expr = (x, scope) => {
      if (x.v !== undefined) { const i = scope.lastIndexOf(x.v); if (i < 0) throw new Error("unbound " + x.v); return term({ bvar: scope.length - 1 - i }); }
      if (x.c) { const n = lib.name.get(x.c) ?? own.get(x.c); if (n === undefined) throw new Error("the library does not hold " + x.c); return term({ const: { name: n, us: x.us.map((u) => (u ? one : 0)) } }); }
      if (x.a) { const fn = expr(x.a[0], scope); return term({ app: { arg: expr(x.a[1], scope), fn } }); }
      if (x.lam || x.pi) { const [v, ty, body] = x.lam || x.pi, b = { binderInfo: "default", body: expr(body, [...scope, v]), name: name(v), type: expr(ty, scope) }; return term(x.lam ? { lam: b } : { forallE: b }); }
      if (x.n !== undefined) return term({ natVal: String(x.n) });
      if (x.s !== undefined) return term({ strVal: x.s });
      throw new Error("not a term: " + JSON.stringify(x).slice(0, 80));
    };
    for (const d of decls) declare(d.kind, d.name, expr(d.type, []), expr(d.value, []));
    return out.join("\n") + "\n";
  };
  // One guarantee's proof, written over the library: the app's words and the guarantee as data, `wordsCheck` on them
  // judged by evaluation (Eq.refl true), and `ruleJudge_sound` applied to that.
  const suffix = (i, words, g) => {
    const { out, term, node, name, known, C, app, nat, one, list, declare } = writer();
    // A theorem's statement with its first binders taken by closed terms.
    const instantiate = (type, args) => {
      let body = type;
      for (let k = 0; k < args.length; k++) body = node(body).forallE.body;
      const memo = new Map(), go = (e, depth) => {
        const key = e + ":" + depth; if (memo.has(key)) return memo.get(key);
        const x = node(e); let r = e;
        if ("bvar" in x) r = x.bvar < depth ? e : x.bvar - depth < args.length ? args[args.length - 1 - (x.bvar - depth)] : term({ bvar: x.bvar - args.length });
        else if (x.app) r = term({ app: { arg: go(x.app.arg, depth), fn: go(x.app.fn, depth) } });
        else if (x.forallE) r = term({ forallE: { ...x.forallE, type: go(x.forallE.type, depth), body: go(x.forallE.body, depth + 1) } });
        else if (x.lam) r = term({ lam: { ...x.lam, type: go(x.lam.type, depth), body: go(x.lam.body, depth + 1) } });
        else if (x.letE) r = term({ letE: { ...x.letE, type: go(x.letE.type, depth), value: go(x.letE.value, depth), body: go(x.letE.body, depth + 1) } });
        else if (x.proj) r = term({ proj: { ...x.proj, struct: go(x.proj.struct, depth) } });
        memo.set(key, r); return r;
      };
      return go(body, 0);
    };
    const K = "PrismLive.Kernel.", Nat = C("Nat"), Bool = C("Bool"), at = "Tab.g" + i + ".";
    const cond = ([field, op, value]) => app(C(K + "Cond.mk"), nat(field), nat(op), nat(value));
    const P = declare("def", at + "words", app(C("List", [0]), Nat), list(Nat, words.map(nat)));
    const p = declare("def", at + "guarantee", C(K + "Property"), app(C(K + "Property.mk"), C(g.always ? "Bool.true" : "Bool.false"), list(C(K + "Cond"), g.when.map(cond)), nat(g.outcome)));
    const T = C("Bool.true");
    const checked = declare("thm", at + "checked", app(C("Eq", [one]), Bool, app(C(K + "wordsCheck"), P, p), T), app(C("Eq.refl", [one]), Bool, T));
    declare("thm", at + "holds", instantiate(lib.types.get(known(K + "ruleJudge_sound")), [P, p, checked]), app(C(K + "ruleJudge_sound"), P, p, checked));
    return out.join("\n") + "\n";
  };
  async function open({ manifest, glue, wasm, library }) {
    const [g, w, l] = await Promise.all([sha256(glue), sha256(wasm), sha256(library)]), b = manifest.binding;
    if (g !== manifest.glue.sha256 || w !== manifest.wasm.sha256 || l !== manifest.library.sha256) return { ok: false, refusal: "the kernel, its loader or its library is not the pinned one" };
    // The library is entered unchecked: sound only for one this kernel has judged whole (the binding).
    if (!b || b.kernel !== w || b.loader !== g || b.library !== l) return { ok: false, refusal: "no record that this kernel judged this library whole" };
    const url = URL.createObjectURL(new Blob([glue], { type: "text/javascript" }));
    try { m = await (await import(url)).default({ instantiateWasm: (imports, done) => { WebAssembly.instantiate(wasm, imports).then((r) => done(r.instance, r.module)); return {}; }, print: (s) => said.push(s), printErr: (s) => said.push(s) }); } finally { URL.revokeObjectURL(url); }
    if (m.ccall("checker_ready", "number", [], []) !== 0) return { ok: false, refusal: "the kernel did not start" };
    const bytes = new Uint8Array(library), loaded = run("load_library", bytes);
    if (loaded.exit !== 0) return { ok: false, refusal: "the kernel refused the library: " + loaded.output };
    lib = read(dec.decode(bytes)); at = { kernel: w, library: l };
    return { ok: true };
  }
  async function prove({ id, words, guarantees }) {
    const texts = guarantees.map((g, i) => suffix(i, words, g));
    const judged = texts.map((t, i) => { const r = run("check_suffix", enc.encode(t)); return { accepted: r.exit === 0, axioms: r.axioms["Tab.g" + i + ".holds"] || null, output: r.output }; });
    const evidence = { ...at, bundle: await sha256(enc.encode(texts.join(""))), statement: await sha256(enc.encode(JSON.stringify(["prism/in-tab-statement/3", words, guarantees]))) };
    return { op: "prove", id, judged, evidence };
  }
  // A stateful app's theorems, each judged with the app's definitions it reaches.
  async function module({ id, defs, theorems }) {
    const texts = theorems.map((t) => written([...defs, t]));
    const judged = texts.map((text, i) => { const r = run("check_suffix", enc.encode(text)); return { accepted: r.exit === 0, axioms: r.axioms[theorems[i].name] || null, output: r.output }; });
    return { op: "module", id, judged, evidence: { ...at, bundle: await sha256(enc.encode(texts.join(""))) } };
  }
  self.onmessage = async ({ data }) => {
    try { self.postMessage(data.op === "open" ? { op: "open", ...(await open(data)) } : data.op === "module" ? await module(data) : await prove(data)); }
    catch (e) { self.postMessage({ op: data.op, id: data.id, ok: false, refusal: "the proof worker failed: " + (e && e.message ? e.message : e) }); }
  };
}
// One worker for the page, opened on first use from the pinned files (kernel/lean/manifest.json). Each is shipped
// gzipped and named by the SHA-256 of its bytes; the worker checks every address before the kernel runs.
let lean = null;
async function openLean() {
  const LEAN = new URL("./kernel/lean/", import.meta.url);
  const bytes = async (file) => { const r = await fetch(new URL(file, LEAN)); if (!r.ok) throw new Error(file + " answered " + r.status); const b = new Uint8Array(await r.arrayBuffer()); return b[0] === 0x1f && b[1] === 0x8b ? new Response(new Blob([b]).stream().pipeThrough(new DecompressionStream("gzip"))).arrayBuffer() : b.buffer; };
  const manifest = await (await fetch(new URL("manifest.json", LEAN))).json();
  const [glue, wasm, library] = await Promise.all([manifest.glue, manifest.wasm, manifest.library].map((f) => bytes(f.file)));
  const worker = new Worker(URL.createObjectURL(new Blob(["(" + leanWorker + ")(self)"], { type: "text/javascript" })), { type: "module" });
  let id = 0;
  const ask = (message) => new Promise((resolve, reject) => {
    const mine = message.op === "open" ? undefined : ++id;
    const on = ({ data }) => { if (data.op === message.op && data.id === mine) { worker.removeEventListener("message", on); resolve(data); } };
    worker.addEventListener("message", on);
    worker.addEventListener("error", (e) => reject(new Error("the proof worker failed: " + (e.message || "it did not start"))), { once: true });
    worker.postMessage({ ...message, id: mine });
  });
  const opened = await ask({ op: "open", manifest, glue, wasm, library });
  if (!opened.ok) { worker.terminate(); throw new Error(opened.refusal); }
  return { manifest, prove: (words, guarantees) => ask({ op: "prove", words, guarantees }), module: (defs, theorems) => ask({ op: "module", defs, theorems }) };
}
/** Lean's kernel proves the spec's guarantees in this tab. Resolves { ok, record, seconds, axioms } or { ok: false, error }. */
export async function prove(spec, { guest, beforeSubmit = () => {} } = {}) {
  const t0 = Date.now(), no = (error) => ({ ok: false, error });
  let read;
  try { read = ruleWordsOf(spec); } catch (e) { return no("Lean's kernel in this tab cannot read these rules: " + e.message); }
  let k, r;
  try { k = await (lean ||= openLean()); beforeSubmit(); r = await k.prove(read.words, read.guarantees); }
  catch (e) { lean = null; return no(String((e && e.message) || e)); }
  if (!r.judged) return no(r.refusal || "the proof worker did not answer");
  const accepted = r.judged.every((g) => g.accepted), observed = [...new Set(r.judged.flatMap((g) => g.axioms || []))].sort();
  const word = (hex) => verdictAddressWords("sha256:" + hex);
  const statement = await sha256Hex(enc.encode(JSON.stringify(["prism/in-tab-statement/3", read.words, read.guarantees])));
  const verdict = await verdictWithGuest(guest, {
    version: 1, hasBytes: 1, libraryPinned: word(k.manifest.library.sha256), libraryDerived: word(r.evidence.library),
    // The proof is written here, not fetched: its one address is the worker's.
    bundleNamed: word(r.evidence.bundle), bundleDerived: word(r.evidence.bundle), kernelPinned: word(k.manifest.wasm.sha256), kernelDerived: word(r.evidence.kernel),
    specStatement: word(statement), checkedStatement: word(r.evidence.statement), axiomsDeclared: verdictAxiomSet(AXIOMS), axiomsObserved: verdictAxiomSet(observed),
    leanRan: 1, leanAccepted: accepted ? 1 : 0, secondRan: 0, secondAccepted: 0,
  });
  if (!verdict.ok) return no("the kernel guest gave no verdict (" + verdict.code + ")");
  const refused = spec.properties.filter((_, i) => !r.judged[i].accepted).map((p) => p.id);
  if (verdict.verdict !== "proved") return no(refused.length ? "Lean's kernel refused " + refused.join(", ") : "the kernel guest said " + verdict.verdict);
  const seconds = Math.round((Date.now() - t0) / 100) / 10;
  return { ok: true, seconds, axioms: {}, record: { schema: "prism/in-tab-verification/1", verdict: verdict.verdict, verified_at: new Date().toISOString(), lean: k.manifest.lean, kernel: "sha256:" + r.evidence.kernel, library: "sha256:" + r.evidence.library, statement: "sha256:" + statement, bundle: "sha256:" + r.evidence.bundle, axioms: observed, properties: spec.properties.map((p) => p.id), seconds } };
}
// ---- the proof of a stateful app: its own module, as kernel terms, in this tab
// The tab writes the app's module (its data form, live/app.mjs semanticData) as Lean terms with no elaborator: each
// structure is the right-nested product of its fields, `if` is `ite (c = true)`, the operators are Nat's and Bool's
// own functions. Each theorem is stated exactly as the module states it, over the definitions it reaches, and is proved
// by one lemma Lean proved once for every app the notation writes (studio/proof, PrismLive.Tables) applied to this
// app's terms, or by evaluation. Lean's kernel, in the worker, judges each theorem; nothing is sent anywhere.
export function moduleTerms(data, library = []) {
  const fail = (why) => { throw new Error(why); };
  const C = (c, us = []) => ({ c, us }), A = (f, ...xs) => xs.reduce((g, x) => ({ a: [g, x] }), f), V = (v) => ({ v });
  const local = new Map(data.declarations.map((d) => [d.name, d])), lib = new Map(library.map((d) => [d.name, d]));
  const BOOL = { kind: "bool" }, NAT = { kind: "nat" }, ONE = (n) => n.length === 1;
  let fresh = 0; const v = () => "x" + fresh++;
  const struct = (member) => {
    const d = (member.module ? lib.get(member.name) : local.get(member.name) || lib.get(member.name)) || fail("no structure " + member.name);
    if (d.kind !== "structure" || d.type_parameters?.length || !d.fields.length) fail(member.name + " is not a plain structure");
    const types = d.fields.map((f) => ty(f.type)), nest = (i) => (i === types.length - 1 ? types[i] : A(C("Prod", [0, 0]), types[i], nest(i + 1)));
    return { d, types, nest };
  };
  const ty = (t) => ({ nat: () => C("Nat"), bool: () => C("Bool"), string: () => C("String"), bytes: () => C("ByteArray"), list: () => A(C("List", [0]), ty(t.element)), named: () => struct(t.member).nest(0) })[t.kind]?.() || fail("type " + t.kind);
  const field = (t, name) => { const s = struct(t.member), k = s.d.fields.findIndex((f) => f.name === name); if (k < 0) fail("no field " + name); return { s, k, type: s.d.fields[k].type }; };
  const project = (t, name, x) => {
    const { s, k } = field(t, name), n = s.types.length;
    for (let j = 0; j < k; j++) x = A(C("Prod.snd", [0, 0]), s.types[j], s.nest(j + 1), x);
    return k < n - 1 ? A(C("Prod.fst", [0, 0]), s.types[k], s.nest(k + 1), x) : x;
  };
  const infer = (e, env) => {
    switch (e.kind) {
      case "nat": case "add": return NAT;
      case "string": return { kind: "string" };
      case "bool": case "and": case "or": case "beq": case "ble": case "blt": case "eq": return BOOL;
      case "var": return (env.get(e.name) || fail("unbound " + e.name)).type;
      case "record": return { kind: "named", member: e.type };
      case "project": return field(infer(e.value, env), e.field).type;
      case "if": return infer(e.then_value, env);
      case "cons": return { kind: "list", element: infer(e.head, env) };
      case "nil": return { kind: "list", element: e.element };
      case "call": return (local.get(e.function.name) || fail("no definition " + e.function.name)).result;
      case "primitive": return e.result;
    }
    return fail("the tab cannot state " + e.kind);
  };
  const reached = [];
  const term = (e, env) => {
    const t = (x) => term(x, env), op = { add: "Nat.add", beq: "Nat.beq", ble: "Nat.ble", blt: "Nat.blt", and: "Bool.and", or: "Bool.or" }[e.kind];
    if (op) return A(C(op), t(e.left), t(e.right));
    switch (e.kind) {
      case "var": return env.get(e.name).term;
      case "nat": return { n: e.value };
      case "string": return { s: e.value };
      case "bool": return C(e.value ? "Bool.true" : "Bool.false");
      case "eq": return A(C("Eq", [1]), ty(infer(e.left, env)), t(e.left), t(e.right));
      case "if": { const c = t(e.condition); return A(C("ite", [1]), ty(infer(e, env)), A(C("Eq", [1]), C("Bool"), c, C("Bool.true")), A(C("instDecidableEqBool"), c, C("Bool.true")), t(e.then_value), t(e.else_value)); }
      case "record": {
        const s = struct(e.type);
        if (e.fields.length !== s.d.fields.length) fail("record shape");
        const vals = s.d.fields.map((f) => t((e.fields.find((x) => x.field === f.name) || fail("record without " + f.name)).value));
        const mk = (i) => (i === vals.length - 1 ? vals[i] : A(C("Prod.mk", [0, 0]), s.types[i], s.nest(i + 1), vals[i], mk(i + 1)));
        return mk(0);
      }
      case "project": return project(infer(e.value, env), e.field, t(e.value));
      case "cons": return A(C("List.cons", [0]), ty(infer(e.head, env)), t(e.head), t(e.tail));
      case "nil": return A(C("List.nil", [0]), ty(e.element));
      case "call": reach(e.function.name); return A(C("Tab.app." + e.function.name), ...e.arguments.map(t));
      case "primitive":
        if (e.arguments.length === 1 && e.operation === "length" && infer(e.arguments[0], env).kind === "bytes") return A(C("ByteArray.size"), t(e.arguments[0]));
        if (e.arguments.length === 1 && e.operation === "utf8_encode") return A(C("String.toUTF8"), t(e.arguments[0]));
    }
    return fail("the tab cannot state " + e.kind + (e.operation ? " " + e.operation : ""));
  };
  // Binds parameters to terms; with no terms, to fresh variables, returned as binders.
  const bind = (params, values) => {
    const env = new Map(), binders = [];
    params.forEach((p, i) => { if (values) env.set(p.name, { type: p.type, term: values[i] }); else { const x = v(); binders.push([x, ty(p.type)]); env.set(p.name, { type: p.type, term: V(x) }); } });
    return { env, binders };
  };
  const over = (binders, kind, body) => binders.reduceRight((b, [x, T]) => ({ [kind]: [x, T, b] }), body);
  const defs = [], done = new Set();
  function reach(name) {
    if (done.has(name)) return; done.add(name);
    const d = local.get(name) || fail("no definition " + name);
    if (d.kind !== "definition" || d.recursive_argument) fail(name + " is not a plain definition");
    const { env, binders } = bind(d.parameters), value = term(d.body, env);
    defs.push({ kind: "def", name: "Tab.app." + name, type: over(binders, "pi", ty(d.result)), value: over(binders, "lam", value) });
  }
  const callee = (e, env) => { const d = local.get(e.function.name); return { d, env: bind(d.parameters, e.arguments.map((a) => term(a, env))).env }; };
  const rev = (stateType, f) => { const x = v(); return { lam: [x, ty(stateType), project(stateType, f, V(x))] }; };
  const holds = (b) => A(C("Eq", [1]), C("Bool"), b, C("Bool.true"));
  const mentions = (x, names) => JSON.stringify(x).match(/"v":"x\d+"/g)?.some((m) => names.includes(m)) || false;
  // The proof that a guard tree is false, given h : bad = true, down to the leaf the bad case refutes.
  const refute = (g, genv, bad, benv, h) => {
    const badT = term(bad, benv);
    if (g.kind === "and") {
      const inLeft = leafIn(g.left, genv, bad, benv), side = inLeft ? g.left : g.right, h2 = v();
      return A(C(inLeft ? "PrismLive.Tables.left" : "PrismLive.Tables.right"), badT, term(g.left, genv), term(g.right, genv), { lam: [h2, holds(badT), refute(side, genv, bad, benv, V(h2))] }, h);
    }
    if (bad.kind === "ble") return A(C("PrismLive.Tables.full"), term(bad.right, benv), term(bad.left, benv), h);
    if (bad.kind === "or" && bad.right.kind === "ble" && bad.right.left.kind === "nat" && Number(bad.right.left.value) > 0) return A(C("PrismLive.Tables.outside"), term(bad.right.right, benv), { n: String(Number(bad.right.left.value) - 1) }, h);
    return fail("the tab has no lemma for this guard");
  };
  const leafIn = (g, genv, bad, benv) => {
    if (g.kind === "and") return leafIn(g.left, genv, bad, benv) || leafIn(g.right, genv, bad, benv);
    if (bad.kind === "ble") return g.kind === "blt";
    const names = JSON.stringify(term(bad, benv)).match(/"v":"x\d+"/g) || [];
    return g.kind !== "blt" && mentions(term(g, genv), names);
  };
  const proof = (st, env) => {
    if (st.kind === "eq" && st.right.kind === "bool" && st.right.value === false && st.left.kind === "call") {
      const { d: F, env: fenv } = callee(st.left, env), b = F.body;
      if (b.kind === "and" && b.right.kind === "blt" && b.right.right.kind === "project" && b.right.right.value.kind === "call") {
        // bad && (rev s <? rev (act s ...)), where act is `if guards then t else s`
        const s = b.right.left.value, T = infer(s, fenv), { d: act, env: aenv } = callee(b.right.right.value, fenv), h = v();
        if (act.body.kind !== "if") fail(act.name + " is not guarded");
        return A(C("PrismLive.Tables.refused"), ty(T), rev(T, b.right.right.field), term(s, fenv), term(act.body.then_value, aenv), term(b.left, fenv), term(act.body.condition, aenv),
          { lam: [h, holds(term(b.left, fenv)), refute(act.body.condition, aenv, b.left, fenv, V(h))] });
      }
      if (b.kind === "and" && b.left.kind === "or" && b.right.kind === "or" && b.right.left.left.kind === "project" && b.right.left.left.value.kind === "call") {
        // a press from another revision: its guarded step is `if seen =? rev s then x else s`
        const seen = b.left.left.left, s = b.left.left.right.value, T = infer(s, fenv), { d: g, env: genv } = callee(b.right.left.left.value, fenv);
        if (g.body.kind !== "if") fail(g.name + " is not guarded");
        return A(C("PrismLive.Tables.stale"), ty(T), rev(T, b.left.left.right.field), term(s, fenv), term(g.body.then_value, genv), term(seen, fenv));
      }
    }
    if (st.kind !== "eq") fail("the statement is not an equation");
    return A(C("Eq.refl", [1]), ty(infer(st.left, env)), term(st.left, env));
  };
  const theorems = [], refused = [];
  for (const th of data.theorems || []) {
    try {
      const { env, binders } = bind(th.parameters);
      theorems.push({ kind: "thm", name: "Tab.app." + th.name, type: over(binders, "pi", term(th.statement, env)), value: over(binders, "lam", proof(th.statement, env)) });
    } catch (e) { refused.push({ theorem: th.name, reason: e.message }); }
  }
  return { defs, theorems, refused };
}
/** Lean's kernel proves a stateful app's theorems in this tab. Resolves { ok, record, seconds, axioms } or { ok: false, error }. */
export async function proveModule(source, data, library, { guest, beforeSubmit = () => {} } = {}) {
  const t0 = Date.now(), no = (error) => ({ ok: false, error });
  let terms;
  try { terms = moduleTerms(data, library); } catch (e) { return no("Lean's kernel in this tab cannot read this module: " + e.message); }
  if (terms.refused.length) return no("Lean's kernel in this tab cannot state " + terms.refused.map((r) => r.theorem + " (" + r.reason + ")").join(", "));
  if (!terms.theorems.length) return no("the module states no theorems");
  let k, r;
  const failed = (e) => { lean = null; return { refusal: String((e && e.message) || e) }; };
  try { k = await (lean ||= openLean()); } catch (e) { return no(failed(e).refusal); }
  beforeSubmit(); r = await k.module(terms.defs, terms.theorems).catch(failed);
  if (!r.judged) return no(r.refusal || "the proof worker did not answer");
  const accepted = r.judged.every((g) => g.accepted), observed = [...new Set(r.judged.flatMap((g) => g.axioms || []))].sort();
  const word = (hex) => verdictAddressWords("sha256:" + hex), statement = await sha256Hex(enc.encode(source));
  const verdict = await verdictWithGuest(guest, {
    version: 1, hasBytes: 1, libraryPinned: word(k.manifest.library.sha256), libraryDerived: word(r.evidence.library),
    bundleNamed: word(r.evidence.bundle), bundleDerived: word(r.evidence.bundle), kernelPinned: word(k.manifest.wasm.sha256), kernelDerived: word(r.evidence.kernel),
    specStatement: word(statement), checkedStatement: word(statement), axiomsDeclared: verdictAxiomSet(AXIOMS), axiomsObserved: verdictAxiomSet(observed),
    leanRan: 1, leanAccepted: accepted ? 1 : 0, secondRan: 0, secondAccepted: 0,
  });
  if (!verdict.ok) return no("the kernel guest gave no verdict (" + verdict.code + ")");
  const refused = terms.theorems.filter((_, i) => !r.judged[i].accepted).map((t) => t.name.replace("Tab.app.", ""));
  if (verdict.verdict !== "proved") return no(refused.length ? "Lean's kernel refused " + refused.join(", ") : "the kernel guest said " + verdict.verdict);
  const seconds = Math.round((Date.now() - t0) / 100) / 10;
  return { ok: true, seconds, axioms: {}, record: { schema: "prism/in-tab-verification/1", verdict: verdict.verdict, verified_at: new Date().toISOString(), lean: k.manifest.lean, kernel: "sha256:" + r.evidence.kernel, library: "sha256:" + r.evidence.library, statement: "sha256:" + statement, bundle: "sha256:" + r.evidence.bundle, axioms: observed, theorems: terms.theorems.map((t) => t.name.replace("Tab.app.", "")), seconds } };
}
// ---- the build: the commit prism-code makes on a repository created from UOR-Foundation/template
// prism-code builds an app on GitHub Actions by writing it into a template instance (journey/engine.mjs
// writeTemplateApp, after journey/project.mjs writeApp); the template's pages and bootstrap workflows then
// build the holo in the SDK image and Lean checks every theorem. This is the same commit, made in the tab:
// the writers are prism-code's (prism/template.mjs, prism/render.mjs), the steps around them are ported,
// and tools/prism_template.mjs --check holds the result to prism-code's own, byte for byte.
// `tree` lists the instance's paths; `read(path)` gives the text of one. The result is what to write and
// what to delete; nothing is sent from here.
const RENAMED = ["lexlean.toml", "lakefile.toml", "lake-manifest.json", "prismpm.toml"];
export async function templateCommit({ spec, axioms = {}, attestation = null, repository, tree, read, load = shellText }) {
  const n = names(spec);
  // writeApp: the project files, renamed from prism-code's text-request template.
  const project = {};
  for (const file of RENAMED) {
    let text = (await load("./prism/text-request/" + file)).split("prism-text-request").join(`prism-${spec.name}`).split("PrismTextRequest").join(n.prefix).split("src/TextRequest.lex.tex").join(`src/${n.module}.lex.tex`);
    if (file === "prismpm.toml") text = text.replace('project = "TextRequest"', `project = "${n.module}"`);
    project[file] = text;
  }
  const model = renderModel(spec, { axioms, specAddress: sealOf(spec) });
  const proofs = renderProofs(spec, { axioms });
  const sources = [`src/${n.module}.lex.tex`, ...(proofs ? [`src/${n.proofs}.lex.tex`] : [])];
  // writeTemplateApp: the app under app/, and every file the gate and the page read.
  const out = new Map();
  out.set(`app/src/${n.module}.lex.tex`, model);
  if (proofs) out.set(`app/src/${n.proofs}.lex.tex`, proofs);
  for (const file of ["lakefile.toml", "lake-manifest.json", "prismpm.toml"]) out.set(`app/${file}`, project[file]);
  out.set("app/lexlean.toml", project["lexlean.toml"].replace(/^entrypoints = \[[^\]]*\]$/m, `entrypoints = [${sources.map((p) => JSON.stringify(p)).join(", ")}]`));
  const appFile = async (p) => (out.has(`app/${p}`) ? out.get(`app/${p}`) : read(`app/${p}`));
  const files = {};
  for (const p of ["lexlean.toml", "lake-manifest.json", "lakefile.toml", "lean-toolchain"]) files[p] = await appFile(p);
  out.set("app/lexlean.lock", lexleanLockText(await read("app/lexlean.lock"), files));
  out.set("model/ids.toml", idsToml(spec, await read("model/ids.toml")));
  out.set(`features/suites/${suiteOf(spec)}.feature`, featureText(spec));
  out.set(`crates/conformance/tests/${suiteOf(spec)}.rs`, rustTestText(spec));
  out.set("CONFORMANCE.md", conformanceText(spec, await read("CONFORMANCE.md")));
  const status = { properties: Object.fromEntries(spec.properties.map((p) => [p.id, "proved"])), proved: { attestation } };
  out.set(`openspec/specs/${spec.name}/spec.md`, renderSpecDocument(spec, status));
  out.set("site/app.json", appJson(spec, await read("site/app.json"), { repository, attestation }));
  // The template's example goes, and every model directly under app/src that is not this app's.
  const remove = tree.filter((p) => !out.has(p) && (TEMPLATE_EXAMPLE.includes(p) || /^app\/src\/[^/]+\.lex\.tex$/.test(p)));
  return { files: [...out].map(([path, content]) => ({ path, content })), remove };
}

// ---- what the chat shows
const said = (p) => `It ${p.kind === "never" ? "never" : "always"} answers ${p.then} when ${describeConditions(p.when)}.`;
const PHASE_WORD = { drafting: "not checked", checked: "checked, not yet proved", proving: "Lean is proving", proved: "proved by Lean", published: "proved by Lean, published" };
export function card(state) {
  const spec = state.spec;
  if (!spec) return `**Rules** · not checked\n\n${state.issues.map((i) => "- " + i).join("\n")}\n\nAsk me to fix the rules.`;
  const lines = [`**Rules: ${spec.title}** · ${state.phase === "published" && state.publicationCurrent === false ? "proved by Lean; wording changed since publication" : PHASE_WORD[state.phase] || state.phase}`, "", spec.purpose, ""];
  if (spec.properties.length) {
    lines.push(state.phase === "proved" || state.phase === "published" ? "Guarantees, proved by Lean for every request:" : "Guarantees Lean will prove for every request:");
    spec.properties.forEach((p, i) => lines.push(`${i + 1}. ${said(p)}${p.why ? ` *${p.why}*` : ""}`));
    lines.push("");
  } else lines.push("No guarantees are stated, so Lean has nothing to prove about these rules.", "");
  lines.push("Rules, the first that matches decides:");
  for (const r of spec.rules) lines.push(`- When ${describeConditions(r.when)}: **${r.then}**${r.why ? ` *${r.why}*` : ""}`);
  lines.push(`- Otherwise: **${spec.otherwise}**${spec.otherwiseWhy ? ` *${spec.otherwiseWhy}*` : ""}`, "");
  lines.push(`Request: \`${spec.fields.map((f) => f.name).join(" ")}\` · answers: ${spec.outcomes.join(", ")}, or invalid`);
  if (spec.examples.length) lines.push("", `Examples: ${spec.examples.map((e) => `\`${e.request}\` → ${e.response}`).join(" · ")}`);
  const facets = facetLine(state.facets);
  if (facets) lines.push("", facets);
  if (state.notes && state.notes.length) lines.push("", ...state.notes.map((n) => "*" + n + "*"));
  if (state.issues && state.issues.length) lines.push("", "Not checked:", ...state.issues.map((i) => "- " + i), "", "Ask me to fix the rules.");
  else if (state.phase === "checked") lines.push("", `Checked: consistent, ${spec.examples.length} examples pass, ${state.vectors} acceptance vectors replayed through the app's rule module.`, "", "Reply **approve** to have Lean's kernel prove the guarantees in this tab. Nothing is sent anywhere.");
  return lines.join("\n");
}
export function provingMessage(spec) {
  return `Proving ${spec.properties.length} guarantee${spec.properties.length === 1 ? "" : "s"} in this tab with Lean's kernel. Nothing is sent anywhere.`;
}
export function proofMessage(spec, result) {
  if (result.ok) return `**Proved in this tab by Lean's kernel.** ${spec.properties.length} of ${spec.properties.length} guarantee${spec.properties.length === 1 ? " holds" : "s hold"} for every request the published app answers, in ${result.seconds} s. Lean ${result.record.lean.version}, kernel \`${result.record.kernel.slice(7, 19)}…\`, library \`${result.record.library.slice(7, 19)}…\`. No server took part.`;
  return `**Not proved.** ${result.error}. The rules are back to checked: ask me to change them, or reply **approve** to try again.`;
}
export { requestLimits };

// The prism/spec/1 grammar a device model is held to while it writes an app's rules: one compact spec in a
// <prism-spec> block, keys in a fixed order (title, purpose, fields, outcomes, rules, otherwise, properties,
// examples). A condition names a declared field, with an operator of that field's kind and, for a choice,
// one of its values; every answer is one of the declared outcomes; an example's request is the fields in
// order. The grammar fixes form, never meaning: which fields, rules and answers the request needs is the
// model's, and the checker and the approval card still judge them.
//   specGrammarCheck(text) -> "fail" (no spec starts with text), "partial" (some does), "done" (text is a whole spec)
const END = { end: true }, FAIL = { fail: true };
const NUM_OPS = ["<", "<=", ">", ">=", "=", "!="], CHOICE_OPS = ["is", "is not"];
const RESERVED = new Set(["invalid", "never", "always"]);
const isLow = (c) => c >= "a" && c <= "z", isDig = (c) => c >= "0" && c <= "9";

export function specGrammarCheck(text) {
  let i = 0;
  const n = text.length;
  const peek = () => { if (i >= n) throw END; return text[i]; };
  const lit = (s) => { for (let k = 0; k < s.length; k++) { if (i >= n) throw END; if (text[i] !== s[k]) throw FAIL; i++; } };
  const str = (ok, max, done) => {
    lit('"'); let s = "";
    for (;;) {
      const c = peek();
      if (c === '"') { if (!done(s)) throw FAIL; i++; return s; }
      if (s.length >= max || !ok(c, s)) throw FAIL;
      s += c; i++;
    }
  };
  const oneOf = (list) => str((c, s) => list.some((w) => w.startsWith(s + c)), 64, (s) => list.includes(s));
  const word = () => str((c, s) => (s ? isLow(c) || isDig(c) || c === "-" : isLow(c)), 24, (s) => s.length > 0 && !s.endsWith("-") && !RESERVED.has(s));
  const name = () => str((c, s) => (s ? isLow(c) || isDig(c) : isLow(c)), 24, (s) => s.length > 0);
  const prose = (max) => str((c) => c >= " " && c <= "~" && c !== "\\", max, (s) => s.trim().length > 0);
  const int = () => {
    let s = "";
    for (;;) {
      if (i >= n) throw END;
      const c = text[i];
      if (!isDig(c)) break;
      if (s === "0" || s.length >= 6) throw FAIL;
      s += c; i++;
    }
    if (!s) throw FAIL;
    return +s;
  };
  const list = (item, min, max, empty = false) => {
    if (empty && peek() === "]") { i++; return 0; }
    let k = 0;
    for (;;) {
      item(); k++;
      const c = peek();
      if (c === ",") { if (k >= max) throw FAIL; i++; continue; }
      if (c === "]") { if (k < min) throw FAIL; i++; return k; }
      throw FAIL;
    }
  };
  const fields = [], outcomes = [];
  const field = () => {
    lit('{"name":');
    const nm = name();
    if (fields.some((f) => f.name === nm)) throw FAIL;
    lit(',"kind":');
    const f = { name: nm, kind: oneOf(["number", "choice"]), values: [] };
    if (f.kind === "number") { lit(',"min":'); f.min = int(); lit(',"max":'); f.max = int(); if (f.max < f.min) throw FAIL; }
    else { lit(',"values":['); list(() => { const v = word(); if (f.values.includes(v)) throw FAIL; f.values.push(v); }, 2, 32); }
    lit("}");
    fields.push(f);
  };
  const cond = () => {
    lit('{"field":');
    const nm = oneOf(fields.map((g) => g.name));
    const f = fields.find((x) => x.name === nm);
    lit(',"op":');
    oneOf(f.kind === "number" ? NUM_OPS : CHOICE_OPS);
    lit(',"value":');
    if (f.kind === "number") int(); else oneOf(f.values);
    lit("}");
  };
  const answer = () => oneOf(outcomes);
  // An example's request: each field's value in order, separated by single spaces.
  const request = () => {
    lit('"');
    for (let k = 0; k < fields.length; k++) {
      if (k) lit(" ");
      const f = fields[k];
      if (f.kind === "number") int();
      else { let s = ""; for (;;) { const c = peek(); if (c === " " || c === '"') break; if (!f.values.some((v) => v.startsWith(s + c))) throw FAIL; s += c; i++; } if (!f.values.includes(s)) throw FAIL; }
    }
    lit('"');
  };
  try {
    lit('<prism-spec>{"title":'); prose(60);
    lit(',"purpose":'); prose(200);
    lit(',"fields":['); list(field, 1, 8);
    lit(',"outcomes":['); list(() => { const w = word(); if (outcomes.includes(w)) throw FAIL; outcomes.push(w); }, 2, 16);
    lit(',"rules":['); list(() => { lit('{"when":['); list(cond, 1, 8); lit(',"then":'); answer(); lit("}"); }, 1, 24);
    lit(',"otherwise":'); answer();
    lit(',"properties":['); list(() => { lit('{"kind":'); oneOf(["never", "always"]); lit(',"when":['); list(cond, 1, 8); lit(',"then":'); answer(); lit("}"); }, 1, 6, true);
    lit(',"examples":['); list(() => { lit('{"request":'); request(); lit(',"response":'); answer(); lit("}"); }, 1, 4, true);
    lit("}</prism-spec>");
    return i === n ? "done" : "fail";
  } catch (e) {
    if (e === END) return "partial";
    if (e === FAIL) return "fail";
    throw e;
  }
}
