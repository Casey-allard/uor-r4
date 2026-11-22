/**
 * `prism/spec.json`: the one input of every Prism app. An agent writes it, or
 * edits it, as JSON; everything after that is a function of its bytes:
 *
 *   JSON text  --readSpecFile-->  closed, normalized spec  --canonicalJson-->  spec.json (RFC 8785 bytes)
 *              --kappa-->  specAddress, the UOR content address of those bytes
 *   spec.json  --render.mjs-->  the LexLean model and proofs, the same bytes every time
 *
 * Reading is strict and never alters what it reads: a duplicate key, a number
 * that is not a whole number within plus or minus 2^53, a member the spec does
 * not define, or text the model cannot carry is refused, naming its JSON path
 * (SpecError, with the rule). The normalized spec fills in only what
 * `prism/spec/1` defines by default (ids, the title, the sentence each rule
 * says), and reading it again gives the same bytes.
 */
import { createHash } from './sha256.mjs';
import { kappa } from '../prism-code/src/blake3.mjs';
import { SPEC_SCHEMA, SpecError, validateSpec } from './spec.mjs';

export const SPEC_FILE = 'spec.json';

/** Every rule reading spec.json can refuse by, beyond the spec's own (spec.mjs REFUSALS). */
export const SPEC_FILE_REFUSALS = Object.freeze(['json.syntax', 'json.duplicate', 'json.number', 'json.member', 'json.shape']);

const refuse = (message, where, rule) => {
  throw new SpecError(message, where, rule);
};

/** The members each level of a spec may hold. Anything else is refused. */
const MEMBERS = Object.freeze({
  spec: ['schema', 'name', 'title', 'purpose', 'actors', 'limits', 'fields', 'outcomes', 'meanings', 'rules', 'otherwise', 'otherwiseWhy', 'properties', 'examples'],
  field: ['name', 'kind', 'values', 'min', 'max', 'means'],
  rule: ['id', 'when', 'then', 'says', 'why'],
  property: ['id', 'kind', 'when', 'then', 'says', 'why'],
  condition: ['field', 'op', 'value'],
  operand: ['field'],
  example: ['request', 'response'],
});

const isObject = (v) => v !== null && typeof v === 'object' && !Array.isArray(v);

function closed(value, level, where) {
  if (!isObject(value)) refuse(`\`${where}\` must be an object.`, where, 'json.shape');
  for (const key of Object.keys(value)) {
    if (!MEMBERS[level].includes(key)) refuse(`\`${where}.${key}\` is not a member of ${level === 'spec' ? SPEC_SCHEMA : `a ${level}`}; the members are ${MEMBERS[level].join(', ')}.`, `${where}.${key}`, 'json.member');
  }
}

const list = (value, where) => {
  if (value === undefined) return [];
  if (!Array.isArray(value)) refuse(`\`${where}\` must be a list.`, where, 'json.shape');
  return value;
};

/** Refuse any member `prism/spec/1` does not define, naming its path. */
export function checkMembers(spec) {
  closed(spec, 'spec', '$');
  list(spec.fields, '$.fields').forEach((f, i) => closed(f, 'field', `$.fields[${i}]`));
  const conditions = (conds, where) =>
    list(conds, where).forEach((c, j) => {
      closed(c, 'condition', `${where}[${j}]`);
      if (isObject(c.value)) closed(c.value, 'operand', `${where}[${j}].value`);
    });
  list(spec.rules, '$.rules').forEach((r, i) => {
    closed(r, 'rule', `$.rules[${i}]`);
    conditions(r.when, `$.rules[${i}].when`);
  });
  list(spec.properties, '$.properties').forEach((p, i) => {
    closed(p, 'property', `$.properties[${i}]`);
    conditions(p.when, `$.properties[${i}].when`);
  });
  list(spec.examples, '$.examples').forEach((e, i) => closed(e, 'example', `$.examples[${i}]`));
  if (spec.meanings !== undefined && !isObject(spec.meanings)) refuse('`$.meanings` must be an object.', '$.meanings', 'json.shape');
}

/**
 * JSON text to a value, strictly: a duplicate key, a number that is not a whole
 * number within plus or minus 2^53, or text that is not JSON is refused.
 */
export function parseStrictJson(text) {
  let at = 0;
  // Not named `fail`: in this package that name means raising a registered diagnostic code.
  const badJson = (message) => refuse(`${message} at character ${at}.`, '$', 'json.syntax');
  const ws = () => {
    while (at < text.length && ' \t\n\r'.includes(text[at])) at++;
  };
  const expect = (ch) => {
    if (text[at] !== ch) badJson(`Expected ${JSON.stringify(ch)}`);
    at++;
  };
  const string = () => {
    const start = at;
    expect('"');
    while (at < text.length && text[at] !== '"') at += text[at] === '\\' ? 2 : 1;
    expect('"');
    try {
      return JSON.parse(text.slice(start, at));
    } catch {
      return badJson('A string is not valid JSON');
    }
  };
  const value = (path) => {
    ws();
    const ch = text[at];
    if (ch === '{') {
      at++;
      const out = {};
      ws();
      if (text[at] === '}') {
        at++;
        return out;
      }
      for (;;) {
        ws();
        const key = string();
        if (Object.hasOwn(out, key)) refuse(`\`${path}.${key}\` appears twice; a key may appear once.`, `${path}.${key}`, 'json.duplicate');
        ws();
        expect(':');
        out[key] = value(`${path}.${key}`);
        ws();
        if (text[at] === ',') {
          at++;
          continue;
        }
        expect('}');
        return out;
      }
    }
    if (ch === '[') {
      at++;
      const out = [];
      ws();
      if (text[at] === ']') {
        at++;
        return out;
      }
      for (;;) {
        out.push(value(`${path}[${out.length}]`));
        ws();
        if (text[at] === ',') {
          at++;
          continue;
        }
        expect(']');
        return out;
      }
    }
    if (ch === '"') return string();
    const m = /^-?(0|[1-9][0-9]*)(\.[0-9]+)?([eE][+-]?[0-9]+)?/.exec(text.slice(at));
    if (m) {
      at += m[0].length;
      if (m[2] || m[3]) refuse(`\`${path}\` is ${m[0]}; numbers in a spec are whole numbers.`, path, 'json.number');
      const n = Number(m[0]);
      if (!Number.isSafeInteger(n)) refuse(`\`${path}\` is ${m[0]}, outside plus or minus 2^53.`, path, 'json.number');
      return n;
    }
    for (const [word, v] of [['true', true], ['false', false], ['null', null]]) {
      if (text.startsWith(word, at)) {
        at += word.length;
        return v;
      }
    }
    return badJson('Not JSON');
  };
  const out = value('$');
  ws();
  if (at !== text.length) badJson('Text after the JSON value');
  return out;
}

/** RFC 8785 (JCS) for a spec's values: keys sorted by UTF-16 code units, no whitespace, ECMAScript strings, whole numbers. */
export function canonicalJson(value) {
  if (Array.isArray(value)) return `[${value.map(canonicalJson).join(',')}]`;
  if (isObject(value)) {
    const keys = Object.keys(value).sort((a, b) => (a < b ? -1 : a > b ? 1 : 0));
    return `{${keys.map((k) => `${JSON.stringify(k)}:${canonicalJson(value[k])}`).join(',')}}`;
  }
  if (typeof value === 'number' && !Number.isSafeInteger(value)) refuse(`${value} is not a whole number within plus or minus 2^53.`, '$', 'json.number');
  return JSON.stringify(value);
}

/**
 * Read a spec: JSON text, or a value already parsed (an agent's tool call).
 * Returns the normalized spec, its canonical text and bytes (what spec.json
 * holds), and two names for those bytes: `specAddress`, the UOR content
 * address (`blake3:<hex>`, the same kappa a holo labels its blobs with, and
 * the seal the app carries), and `specHash`, their `sha256:<hex>`. Throws
 * SpecError naming the rule and the JSON path of the first problem.
 */
export function readSpecFile(input) {
  const raw = typeof input === 'string' ? parseStrictJson(input) : structuredClone(input);
  checkMembers(raw);
  // A parsed value can still hold numbers a text could not: refuse them the same way.
  canonicalJson(raw);
  const spec = validateSpec(raw);
  const text = canonicalJson(spec);
  const bytes = new TextEncoder().encode(text);
  return { spec, text, bytes, specAddress: kappa(bytes), specHash: `sha256:${createHash('sha256').update(bytes).digest('hex')}` };
}
