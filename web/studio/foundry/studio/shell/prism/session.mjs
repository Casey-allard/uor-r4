/**
 * The spec session: six questions, asked one at a time, each answer checked
 * as it arrives. The spec is rebuilt from the raw answers every time, so an
 * earlier answer can be revised and everything after it is checked again.
 *
 * Pure: no files, no processes. mcp.mjs stores the session and runs the SDK.
 */
import {
  INVALID,
  SpecError,
  counterexample,
  describeCondition,
  describeConditions,
  judge,
  decodeRequest,
  nameFromPurpose,
  parseExamples,
  parseFields,
  parseOutcomes,
  parseProperties,
  parseRules,
  respond,
  titleFromName,
  unreachableRules,
  validateSpec,
} from './spec.mjs';

export const STEPS = ['purpose', 'fields', 'outcomes', 'rules', 'properties', 'examples', 'review'];
const NONE = /^\s*(none|no|nothing|skip|n\/a|-)\s*\.?\s*$/i;
const YES = /^\s*(yes|y|ok|okay|go|go ahead|build|build it|ship it|looks good|correct|right|sure|do it|proceed)\b/i;

const pick = (list, fallback) => (list && list.length ? list : fallback);

/** The question for a step, given what is known so far. */
export function question(step, spec = {}) {
  const fields = spec.fields ?? [];
  const outcomes = spec.outcomes ?? [];
  const firstChoice = fields.find((f) => f.kind === 'choice');
  const numbers = fields.filter((f) => f.kind === 'number');
  switch (step) {
    case 'purpose':
      return {
        question: 'What should this app decide? Say it in one sentence.',
        example: 'Payment approval: decide whether a payment request is approved.',
        why: 'The sentence names the app and becomes its description.',
      };
    case 'fields':
      return {
        question: 'What goes into one request? List the fields in order.',
        format: 'name: number   or   name: choice|choice|choice   (bounds: number 1..100, number at least 1)',
        example: 'role: clerk|manager, amount: number at least 1, limit: number',
        why: 'A request is one line of these fields, separated by spaces. Anything else is answered invalid.',
      };
    case 'outcomes':
      return {
        question: 'What can the answer be?',
        format: 'two or more lower-case words',
        example: 'approved, refused',
        why: 'Every request gets exactly one of these answers, or invalid when it is malformed.',
      };
    case 'rules': {
      const a = pick(outcomes, ['yes', 'no']);
      // A number the field can actually hold, so the example reads like a real rule.
      const middle = (f) => (f.max !== undefined ? Math.floor(((f.min ?? 0) + f.max) / 2) : Math.max(100, (f.min ?? 0) * 10));
      const cond = firstChoice ? `${firstChoice.name} is ${firstChoice.values.at(-1)}` : numbers[0] ? `${numbers[0].name} <= ${middle(numbers[0])}` : 'field is value';
      const second = numbers.length >= 2 ? `${numbers[0].name} <= ${numbers[1].name}` : numbers[0] ? `${numbers[0].name} < ${middle(numbers[0])}` : cond;
      return {
        question: 'What are the rules, in order? The first rule that matches decides.',
        format: 'one per line: when <condition> [and <condition>] then <answer>; end with: otherwise <answer>',
        example: `when ${cond} then ${a[0]}\nwhen ${second} then ${a[0]}\notherwise ${a.at(-1)}`,
        why: 'Conditions compare a field with a choice (role is manager), a number (amount > 1000) or another number field (amount <= limit).',
      };
    }
    case 'properties': {
      const a = pick(outcomes, ['yes', 'no']);
      const rule = spec.rules?.[0];
      const suggestion = rule ? `always when ${rule.when.map(describeConditionPlain).join(' and ')} then ${rule.then}` : `never when ... then ${a[0]}`;
      return {
        question: 'What must never happen, or always happen? Each one becomes a theorem that Lean proves for every possible request.',
        format: 'one per line: never when <condition> then <answer>   or   always when <condition> then <answer>',
        example: suggestion,
        why: 'This is the part that is proved, not tested. Think of what would be a serious mistake for this app.',
      };
    }
    case 'examples':
      return {
        question: 'Give one or two real requests and the answer you expect. Say none to skip.',
        format: 'one per line: <request> -> <answer>',
        example: exampleLine(spec),
        why: 'Each example is checked against your rules now, and becomes an acceptance vector the built app must pass.',
      };
    case 'review':
      return {
        question: 'Is this right? Say yes to prove and build it, or tell me what to change.',
        summary: summary(spec),
        why: 'Proving takes a few minutes; the first build takes ten or more.',
      };
    default:
      return { question: 'The spec is complete. Call prism_build to prove and build it.' };
  }
}

function exampleLine(spec) {
  const fields = spec.fields ?? [];
  if (!fields.length) return 'clerk 250 1000 -> approved';
  const parts = fields.map((f) => (f.kind === 'choice' ? f.values[0] : String(f.min ?? 10)));
  try {
    const v = validateSpec({ ...spec, examples: [] });
    const req = decodeRequest(v, parts.join(' '));
    return `${parts.join(' ')} -> ${req ? judge(v, req).outcome : INVALID}`;
  } catch {
    return `${parts.join(' ')} -> ${(spec.outcomes ?? ['yes'])[0]}`;
  }
}

/** A condition written so the parser reads it back. */
export function describeConditionPlain(c) {
  if (c.op === 'is' || c.op === 'is not') return `${c.field} ${c.op} ${c.value}`;
  return `${c.field} ${c.op} ${typeof c.value === 'object' ? c.value.field : c.value}`;
}

/**
 * The spec on one screen. Every property is listed, because the properties are
 * what gets proved; when the whole spec is long, the rules are counted instead.
 */
export function summary(spec) {
  if (!spec?.fields) return '';
  const because = (why) => (why ? `, because ${why}` : '');
  const purpose = [`${spec.title}: ${spec.purpose}`];
  const properties = spec.properties.map((p) => `${p.id}: ${p.kind} ${p.then} when ${describeConditions(p.when)}${because(p.why)} (to be proved)`);
  const rules = [...spec.rules.map((r) => `${r.id}: when ${describeConditions(r.when)}, ${r.then}${because(r.why)}`), `Otherwise: ${spec.otherwise}${because(spec.otherwiseWhy)}`];
  const tail = [
    `Request: ${spec.fields.map((f) => `${f.name} (${f.kind === 'choice' ? f.values.join('|') : `number${f.min !== undefined ? ` from ${f.min}` : ''}${f.max !== undefined ? ` up to ${f.max}` : ''}`}${f.means ? `; ${f.means}` : ''})`).join(', ')}`,
    `Answers: ${spec.outcomes.map((o) => (spec.meanings?.[o] ? `${o} (${spec.meanings[o]})` : o)).join(', ')}, or invalid`,
    ...(spec.limits ? [`Out of scope: ${spec.limits.map((l) => l.replace(/[.]$/, '')).join('; ')}`] : []),
  ];
  const shown = purpose.length + properties.length + rules.length + tail.length <= 16 ? rules : [`${spec.rules.length} rules, then otherwise ${spec.otherwise} (all in SPEC.md)`];
  return [...purpose, ...properties, ...shown, ...tail].join('\n');
}

/** Answers that reproduce a whole spec, so a spec written as data joins the same flow. */
export function answersFromSpec(input) {
  const spec = validateSpec(input);
  const bound = (f) => (f.min !== undefined && f.max !== undefined ? ` ${f.min}..${f.max}` : f.min !== undefined ? ` at least ${f.min}` : f.max !== undefined ? ` at most ${f.max}` : '');
  return {
    purpose: `${spec.title}: ${spec.purpose}`,
    fields: spec.fields.map((f) => `${f.name}: ${f.kind === 'choice' ? f.values.join('|') : `number${bound(f)}`}`).join(', '),
    outcomes: spec.outcomes.join(', '),
    rules: [...spec.rules.map((r) => `when ${r.when.map(describeConditionPlain).join(' and ')} then ${r.then}`), `otherwise ${spec.otherwise}`].join('\n'),
    properties: spec.properties.length ? spec.properties.map((p) => `${p.kind} when ${p.when.map(describeConditionPlain).join(' and ')} then ${p.then}`).join('\n') : 'skip',
    examples: spec.examples.length ? spec.examples.map((e) => `${e.request} -> ${e.response}`).join('\n') : 'none',
    ...(input.name ? { name: input.name } : {}),
    ...aboutOf(spec),
  };
}

const ruleLine = (r) => `when ${r.when.map(describeConditionPlain).join(' and ')} then ${r.then}`;
const propertyLine = (p) => `${p.kind} ${ruleLine(p)}`;

/**
 * What the six answers cannot hold: each reason, meaning and limit, keyed by
 * what it describes. A rule's reason is keyed by the rule itself, so it stays
 * with that rule through any revision and is dropped when the rule changes.
 */
function aboutOf(spec) {
  const about = {
    fields: Object.fromEntries(spec.fields.filter((f) => f.means).map((f) => [f.name, f.means])),
    meanings: spec.meanings ?? {},
    rules: Object.fromEntries(spec.rules.filter((r) => r.why).map((r) => [ruleLine(r), r.why])),
    properties: Object.fromEntries(spec.properties.filter((p) => p.why).map((p) => [propertyLine(p), p.why])),
    ...(spec.otherwiseWhy ? { otherwise: spec.otherwiseWhy } : {}),
    ...(spec.limits ? { limits: spec.limits } : {}),
  };
  const empty = ['fields', 'meanings', 'rules', 'properties'].every((k) => Object.keys(about[k]).length === 0) && !about.otherwise && !about.limits;
  return empty ? {} : { about };
}

/** The spec with what `about` says of it; what no longer matches is left out. */
function withAbout(spec, about) {
  if (!about || !spec?.fields) return spec;
  return {
    ...spec,
    fields: spec.fields.map((f) => (about.fields?.[f.name] ? { ...f, means: about.fields[f.name] } : f)),
    ...(spec.outcomes && about.meanings ? { meanings: Object.fromEntries(Object.entries(about.meanings).filter(([o]) => spec.outcomes.includes(o))) } : {}),
    ...(spec.rules ? { rules: spec.rules.map((r) => (about.rules?.[ruleLine(r)] ? { ...r, why: about.rules[ruleLine(r)] } : r)) } : {}),
    ...(spec.properties ? { properties: spec.properties.map((p) => (about.properties?.[propertyLine(p)] ? { ...p, why: about.properties[propertyLine(p)] } : p)) } : {}),
    ...(about.otherwise ? { otherwiseWhy: about.otherwise } : {}),
    ...(about.limits ? { limits: about.limits } : {}),
  };
}

/**
 * Rebuild the spec from the raw answers. Returns `{ spec, step, issue, notes }`:
 * `step` is the next question to ask (the first unanswered or failing one),
 * `issue` explains why an answered step must be asked again.
 */
export function compile(answers) {
  const spec = {};
  const notes = [];
  for (const step of STEPS) {
    const raw = answers[step];
    if (raw === undefined || raw === null || String(raw).trim() === '') return { spec, step, issue: null, notes };
    try {
      switch (step) {
        case 'purpose': {
          const text = String(raw).trim();
          spec.name = answers.name ?? nameFromPurpose(text);
          spec.title = text.includes(':') && text.indexOf(':') < 40 ? text.slice(0, text.indexOf(':')).trim() : titleFromName(spec.name);
          spec.purpose = text.includes(':') && text.indexOf(':') < 40 ? text.slice(text.indexOf(':') + 1).trim() : text;
          break;
        }
        case 'fields':
          spec.fields = parseFields(raw);
          break;
        case 'outcomes':
          spec.outcomes = parseOutcomes(raw);
          if (spec.outcomes.includes(INVALID)) throw new SpecError('`invalid` is the answer to a malformed request; pick another word.', 'outcomes', 'outcomes.reserved');
          break;
        case 'rules': {
          Object.assign(spec, parseRules(raw, spec.fields, spec.outcomes));
          const v = validateSpec({ ...spec, properties: [], examples: [] });
          const dead = unreachableRules(v);
          if (dead.length) notes.push(`${dead.join(', ')} never decides anything: an earlier rule always matches first, or its conditions cannot all hold.`);
          break;
        }
        case 'properties': {
          spec.properties = NONE.test(String(raw)) ? [] : parseProperties(raw, spec.fields, spec.outcomes);
          if (spec.properties.length === 0) notes.push('No property is stated, so nothing about the rules will be proved; only the acceptance vectors will test them.');
          const v = validateSpec({ ...spec, examples: [] });
          for (const p of v.properties) {
            const found = counterexample(v, p);
            if (found.request !== null) {
              return {
                spec,
                step,
                issue: `${p.id} does not hold. The request \`${found.request}\` meets "${describeConditions(p.when)}", and the rules answer ${found.outcome} (${found.rule ?? 'otherwise'}). Change a rule (answer the rules step again), or restate the property.`,
                notes,
              };
            }
          }
          break;
        }
        case 'examples': {
          spec.examples = NONE.test(String(raw)) ? [] : parseExamples(raw);
          const v = validateSpec(spec);
          for (const e of v.examples) {
            const got = respond(v, new TextEncoder().encode(e.request));
            if (got !== e.response) {
              const req = decodeRequest(v, e.request);
              const reason = req ? `${judge(v, req).rule ?? 'otherwise'} decides it` : 'it is malformed: it must be the fields in order, separated by single spaces';
              return { spec, step, issue: `Your example \`${e.request}\` expects ${e.response}, but the rules answer ${got}, because ${reason}. Fix the example, or change the rules.`, notes };
            }
          }
          break;
        }
        case 'review':
          if (!YES.test(String(raw))) return { spec, step, issue: 'Tell me what to change: answer that step again (purpose, fields, outcomes, rules, properties or examples).', notes };
          return { spec: validateSpec(spec), step: 'ready', issue: null, notes };
        default:
          break;
      }
    } catch (error) {
      if (!(error instanceof SpecError)) throw error;
      return { spec, step, issue: error.message, refusal: error.rule, notes };
    }
  }
  return { spec, step: 'ready', issue: null, notes };
}

/** The parts of a spec a round trip must give back; ids and the sentences describing rules are derived. */
export const ROUND_TRIP_PARTS = Object.freeze(['name', 'title', 'purpose', 'fields', 'outcomes', 'rules', 'otherwise', 'properties', 'examples']);
const part = (spec, key) => {
  const value = spec[key];
  if (key === 'rules') return JSON.stringify(value?.map((r) => ({ when: r.when, then: r.then })));
  if (key === 'properties') return JSON.stringify(value?.map((p) => ({ kind: p.kind, when: p.when, then: p.then })));
  return JSON.stringify(value);
};

/**
 * Prism's own check that a spec survives its answers: `answersFromSpec`, then
 * `compile`, gives back the same spec. Null when it does; else the part that
 * came back different, with the rule that refused it when one did. A
 * difference is a Prism bug, never the user's: the spec was valid. A property
 * or example the rules break stops `compile` early, so only the parts it read
 * are compared.
 */
export function roundTrip(input) {
  const spec = validateSpec(input);
  const answers = { ...answersFromSpec(spec), review: 'yes' };
  const back = compile(answers);
  if (back.refusal) return { part: back.step, refusal: back.refusal };
  const about = withAbout(back.spec, answers.about);
  for (const key of ROUND_TRIP_PARTS) if (key in back.spec && part(about, key) !== part(spec, key)) return { part: key, refusal: null };
  return null;
}

/** What the agent shows next: the question, any issue, notes, and progress. */
export function nextTurn(answers) {
  const { spec: bare, step, issue, refusal, notes } = compile(answers);
  const spec = withAbout(bare, answers.about);
  const done = STEPS.indexOf(step) === -1 ? STEPS.length : STEPS.indexOf(step);
  const full = (() => {
    try {
      return validateSpec(spec);
    } catch {
      return null;
    }
  })();
  if (full && step === 'review') {
    const unexplained = full.properties.filter((p) => !p.why).map((p) => p.id);
    if (unexplained.length) notes.push(`${unexplained.join(', ')} ${unexplained.length === 1 ? 'has' : 'have'} no reason. Lean proves ${unexplained.length === 1 ? 'it' : 'them'}, so say why each matters, or ask the user.`);
  }
  return {
    step,
    ...(issue ? { issue } : {}),
    ...(notes.length ? { notes } : {}),
    ...question(step, full ?? spec),
    ...(step === 'fields' || step === 'outcomes' ? { inferred: `The app is called ${spec.name}.` } : {}),
    progress: `${Math.min(done, 6)} of 6 answered`,
    spec: full,
    refusal: refusal ?? null,
  };
}

export { describeCondition };
