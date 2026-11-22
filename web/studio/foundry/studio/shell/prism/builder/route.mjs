// The chat routes one idea onto every feature of a Prism model. The model writing an app writes its
// behavior as its JSON spec (prism.js SPEC_RULES), and beside it the rest of one prism/prompt/1: who
// the app is for and what they care about, the systems around it, what can go wrong and what answers it,
// how good is good enough, and the journeys through it. The builder projects the spec and these parts
// into every facet, so the full model is defined by the conversation, not left to defaults.
//
// The parts travel in a <prism-prompt> block, the prompt without its app (the spec is the app). They
// are checked with the builder's own rules (readPrompt, deriveModel), so a threat with neither a control
// nor an accepted risk, a journey step that names no rule, or a quality row outside ISO/IEC 25010 is
// refused with the builder's reason, as the spec's own issues are. What the model leaves out the
// builder derives from the spec.
import { ASSET_KEYS, COMPONENT_KINDS, EVIDENCE_KINDS, IMPACT, ISO25010, LIKELIHOOD, MODEL_KINDS, PATTERNS, STRIDE } from "./prompt.mjs";

/** The members of a prism/prompt/1 a model may write; the app is the spec, the rest is the studio's. */
export const PARTS = ["goal", "pattern", "machine", "stakeholders", "concerns", "components", "threats", "quality", "journeys"];

/** What the model is told, written from the builder's closed sets so it can never ask for one the builder refuses. */
export const PROMPT_RULES = `

# Route the idea onto every feature of the app
An app is a whole Prism model, not only its rules. After the <prism-spec> block, write one <prism-prompt> block: JSON holding the rest of the app's prism/prompt/1, everything the spec does not say. Route what the user asked for onto each member; derive the rest from the rules, and leave out a member only when it adds nothing to what the rules imply.
- "goal": the goal in one sentence a developer would say.
- "pattern": ${Object.entries(PATTERNS).map(([k, p]) => `"${k}" (${p.standard})`).join(", ")}. A stateMachine also writes "machine": {"state": <the choice field holding the state>, "initial", "final": [...]}.
- "stakeholders": [{"name", "text", "concerns": [<concern names>]}], who the app is for; a Verifier is always added.
- "concerns": [{"name", "text", "model": one of ${MODEL_KINDS.join(", ")}}], what they care about.
- "components": [{"name", "text", "kind": one of ${COMPONENT_KINDS.join(", ")}}], the systems that send the app requests or read its answers.
- "threats": [{"name", "text", "stride": one of ${STRIDE.join(", ")}, "asset": one of ${ASSET_KEYS.join(", ")}, "likelihood": ${LIKELIHOOD.join("/")}, "impact": ${IMPACT.join("/")}, and either "control": {"name", "text", "evidence": [{"kind": one of ${EVIDENCE_KINDS.join(", ")}, "reference"}]} or "accepted": <why the risk is accepted>}]. Proof evidence names a guarantee by its id (INV-001 is the first). The threats every app has are added for you.
- "quality": [{"characteristic", "subcharacteristic", "requirement", "text", "measure", "measureText", "threshold": <whole number>}], ISO/IEC 25010:2023 pairs only: ${Object.entries(ISO25010).map(([c, s]) => `${c} (${s.join(", ")})`).join("; ")}.
- "journeys": [{"stakeholder", "steps": [<REQ-001 for the first rule, INV-001 for the first guarantee, otherwise, refuse>]}].
Rewrite the whole block when any part of it changes. Example: <prism-prompt>{"goal":"Charge a fair parking fee.","stakeholders":[{"name":"Driver","text":"Parks and pays the fee the app answers.","concerns":["Fair fee"]}],"concerns":[{"name":"Fair fee","text":"A free stay is never charged.","model":"quality"}],"threats":[{"name":"Overcharge","text":"A free stay is charged premium.","stride":"repudiation","asset":"answer","likelihood":"low","impact":"major","control":{"name":"Proved free stay","text":"Lean proves a zero-hour stay is never premium.","evidence":[{"kind":"proof","reference":"INV-001"}]}}],"journeys":[{"stakeholder":"Driver","steps":["REQ-001","otherwise"]}]}</prism-prompt>`;

const TAG = /<prism-prompt>([\s\S]*?)(?:<\/prism-prompt>|$)/;

/** The parts a turn wrote, { parts } or { error }, or null when it wrote none. */
export function parsePromptTag(text) {
  const m = TAG.exec(text || "");
  if (!m) return null;
  let body = m[1].trim();
  const fence = body.match(/^```[\w-]*\n([\s\S]*?)\n```$/);
  if (fence) body = fence[1];
  let value;
  try { value = JSON.parse(body); } catch (e) { return { error: "The app's prompt is not valid JSON: " + e.message }; }
  if (!value || typeof value !== "object" || Array.isArray(value)) return { error: "The app's prompt is a JSON object." };
  return { parts: Object.fromEntries(PARTS.filter((k) => value[k] !== undefined).map((k) => [k, value[k]])) };
}

/**
 * The facets the builder projects from the parts and a checked spec: { ok, summary } with the count of
 * each kind of record, or { ok: false, issue } with the builder's reason. The builder loads only here,
 * when an app's rules are checked, so the shell's first load does not carry it.
 */
export async function facetsOf(parts, spec) {
  try {
    const { deriveModel } = await import("./model.mjs");
    const m = deriveModel({ ...(parts || {}), app: spec });
    const a = m.architecture, s = m.security, q = m.quality;
    return {
      ok: true,
      summary: { pattern: m.pattern.key, stakeholders: a.stakeholders.length, concerns: a.concerns.length, components: a.components.length, features: m.features.features.length, journeys: m.features.journeys.length, threats: s.threats.length, controls: s.controls.length, measures: q.measures.length, obligations: m.compliance.length },
    };
  } catch (e) {
    return { ok: false, issue: String((e && e.message) || e) };
  }
}

/** One line for the rules card: what the full model holds. */
export function facetLine(facets) {
  if (!facets) return null;
  if (!facets.ok) return `Facets not projected: ${facets.issue} Ask me to fix the app's prompt.`;
  const f = facets.summary;
  return `Full model (${PATTERNS[f.pattern].name}): ${f.stakeholders} stakeholders, ${f.concerns} concerns, ${f.components} components, ${f.features} features in ${f.journeys} journeys, ${f.threats} threats with ${f.controls} controls, ${f.measures} quality measures, ${f.obligations} obligations.`;
}
