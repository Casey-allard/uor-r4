/**
 * Render a `prism/spec/1` spec into the LexLean model of a text application,
 * its acceptance vectors and SPEC.md.
 *
 * Choices and answers are numbered (`Nat` codes behind named constants), and
 * numbers are `Nat`. Every condition is then a `Nat` comparison (`=?`, `<?`,
 * `<=?`), which `linear_arithmetic` reflects into propositions before it
 * splits the `if` chain and calls `omega`. That is what makes each property
 * provable by one fixed proof form, with no proof written by hand.
 */
import { INVALID, MAX_DIGITS, describeConditions, encodeRequest, holds, judge, requestLimits, respond, sampleRequests } from './spec.mjs';

const LEAN_RESERVED = new Set([
  'let', 'if', 'match', 'fun', 'forall', 'nil', 'cons', 'first', 'second', 'not', 'apply', 'ctor', 'record', 'project', 'prim', 'ref', 'checked', 'then', 'else', 'with', 'true', 'false',
  'at', 'by', 'do', 'end', 'from', 'have', 'in', 'mut', 'open', 'return', 'show', 'try', 'where', 'deriving', 'instance', 'structure', 'inductive', 'theorem', 'def', 'namespace', 'section', 'variable',
  'axiom', 'example', 'abbrev', 'universe', 'local', 'private', 'protected', 'noncomputable', 'partial', 'unsafe', 'macro', 'syntax', 'calc', 'suffices', 'obtain', 'this', 'fields', 'value', 'request', 'text', 'number', 'code', 'type', 'prop', 'sort',
]);

const pascal = (word) => word.split(/[^a-z0-9]+/i).filter(Boolean).map((w) => w[0].toUpperCase() + w.slice(1)).join('');
const camel = (word) => {
  const p = pascal(word);
  return p[0].toLowerCase() + p.slice(1);
};
const hex = (text) => `0x${Buffer.from(typeof text === 'string' ? new TextEncoder().encode(text) : text).toString('hex')}`;
/** A LexLean string literal. Text is refused, never altered: validateSpec admits only printable ASCII without a quote or a backslash here. */
const quote = (text) => {
  const t = String(text);
  if (/[^ -~]|["\\]/.test(t)) throw new Error(`render refuses ${JSON.stringify(t)}: a LexLean string here carries printable ASCII without a quote or a backslash`);
  return `"${t}"`;
};

/** Every identifier the model uses, derived once so the model, the vectors and SPEC.md agree. */
export function names(spec) {
  const used = new Set(['Request', 'byteAt', 'digitsValue', 'parseDigits', 'parseNumber', 'sliceOf', 'fieldEnd', 'decodeField1', 'decodeField2', 'decodeField3', 'decodeField4', 'decodeField5', 'decodeField6', 'decodeField7', 'decodeField8', 'judge', 'encodeAnswer', 'dispatch', 'application']);
  const claim = (base) => {
    let name = base;
    for (let i = 2; used.has(name) || LEAN_RESERVED.has(name); i++) name = `${base}${i}`;
    used.add(name);
    return name;
  };
  const module = pascal(spec.name) || 'App';
  const fields = {};
  for (const f of spec.fields) {
    const id = claim(camel(f.name));
    fields[f.name] = { id, parse: f.kind === 'choice' || f.min !== undefined || f.max !== undefined ? claim(`parse${pascal(f.name)}`) : 'parseNumber', choices: {} };
    if (f.kind === 'choice') for (const v of f.values) fields[f.name].choices[v] = claim(`${id}${pascal(v)}`);
  }
  const answers = {};
  for (const o of spec.outcomes) answers[o] = claim(`answer${pascal(o)}`);
  const rules = Object.fromEntries(spec.rules.map((r, i) => [r.id, claim(`rule${i + 1}`)]));
  const properties = Object.fromEntries(
    spec.properties.map((p) => {
      const stem = p.id.toLowerCase().replace(/[^a-z0-9]/g, '');
      return [p.id, { breaks: claim(`breaks${pascal(stem)}`), theorem: claim(`${stem}Holds`) }];
    }),
  );
  return { module, proofs: `${module}Proofs`, prefix: `Prism${module}`, fields, answers, rules, properties };
}

/** The application's artifact stem: letters, digits, spaces, hyphens, underscores and dots. */
const stem = (title) => title.replace(/[^A-Za-z0-9 _.-]/g, '').replace(/^[^A-Za-z0-9]+/, '').replace(/[ .]+$/, '').slice(0, 128) || 'App';

const condition = (spec, n, c, q = (x) => x) => {
  const field = n.fields[c.field];
  const left = `(project request ${field.id})`;
  if (c.op === 'is') return `(${left} =? (${q(field.choices[c.value])}))`;
  if (c.op === 'is not') return `(not (${left} =? (${q(field.choices[c.value])})))`;
  const right = typeof c.value === 'object' ? `(project request ${n.fields[c.value.field].id})` : String(c.value);
  switch (c.op) {
    case '<': return `(${left} <? ${right})`;
    case '<=': return `(${left} <=? ${right})`;
    case '>': return `(${right} <? ${left})`;
    case '>=': return `(${right} <=? ${left})`;
    case '=': return `(${left} =? ${right})`;
    case '!=': return `(not (${left} =? ${right}))`;
    default: throw new Error(`unknown op ${c.op}`);
  }
};

const conjunction = (terms) => terms.reduce((acc, t) => (acc ? `(${acc} && ${t})` : t), '');

/**
 * The bounds a decoded request satisfies, as conditions, so a property is
 * stated over exactly the requests that decode. A choice is stored as its
 * code, 0 up to one less than the number of choices, and the decoder yields
 * no other code; without that bound an `always` property fails on codes no
 * request can carry, where no rule that names a choice matches.
 */
const bounds = (spec) =>
  spec.fields.flatMap((f) => [
    ...(f.kind === 'choice' ? [{ field: f.name, op: '<', value: f.values.length }] : []),
    ...(f.min !== undefined && f.min > 0 ? [{ field: f.name, op: '>=', value: f.min }] : []),
    ...(f.max !== undefined ? [{ field: f.name, op: '<=', value: f.max }] : []),
  ]);

const ALL = '[Classical.choice, Quot.sound, propext]';
const QP = '[Quot.sound, propext]';

/**
 * The declarations of the model, in order, each `{ name, text, axioms }`.
 * `axioms` is the exact set the declaration is expected to need; a verify that
 * observes another set rewrites it (sdk.mjs).
 */
export function declarations(spec, n = names(spec), { specAddress = null } = {}) {
  const out = [];
  const def = (name, parameters, result, body, axioms = null) => out.push({ name, kind: 'definition', parameters, result, body, axioms });
  const NONE = (t) => `(ctor Option.none [${t}])`;
  const SOME = (t, v) => `(ctor Option.some [${t}] ${v})`;

  for (const f of spec.fields) if (f.kind === 'choice') f.values.forEach((v, i) => def(n.fields[f.name].choices[v], [], 'Nat', String(i)));
  spec.outcomes.forEach((o, i) => def(n.answers[o], [], 'Nat', String(i)));

  out.push({ name: 'Request', kind: 'structure', fields: spec.fields.map((f) => `{name: ${n.fields[f.name].id}, type: Nat}`) });

  // One byte as a Nat, converted in one place: the compiler unrolls UInt8 to Nat
  // bit by bit, and several conversions in one nested expression exceed the
  // exporter's nesting limit.
  def('byteAt', ['s : Bytes', 'i : Nat'], '(Option Nat)', `(match (prim index s i : (Option UInt8))
    | Option.none => ${NONE('Nat')}
    | Option.some b => ${SOME('Nat', '(prim to_nat b : Nat)')})`, QP);
  // Decimal digits one UTF-8 byte at a time, as the chess board is read: the
  // guest then needs only UInt8 to Nat, which the production export supports.
  out.push({
    name: 'digitsValue',
    kind: 'definition',
    parameters: ['s : Bytes', 'i : Nat', 'acc : Nat', 'fuel : Nat'],
    result: '(Option Nat)',
    body: `(match fuel
    | Nat.zero => ${SOME('Nat', 'acc')}
    | Nat.succ rest => (match (byteAt s i)
      | Option.none => ${NONE('Nat')}
      | Option.some d => (if ((d <? 48) || (57 <? d)) then ${NONE('Nat')} else (digitsValue s (i + 1) ((prim multiply acc 10 : Nat) + (prim subtract d 48 : Nat)) rest))))`,
    axioms: QP,
    recursive: 'fuel',
  });
  def(
    'parseDigits',
    ['s : Bytes'],
    '(Option Nat)',
    `(if ((Foundation.Bytes::byteLength s) =? 0) then ${NONE('Nat')} else (if (${MAX_DIGITS} <? (Foundation.Bytes::byteLength s)) then ${NONE('Nat')} else (match (byteAt s 0)
    | Option.none => ${NONE('Nat')}
    | Option.some lead => (if ((lead =? 48) && (1 <? (Foundation.Bytes::byteLength s))) then ${NONE('Nat')} else (digitsValue s 0 0 (Foundation.Bytes::byteLength s))))))`,
    QP,
  );
  def('parseNumber', ['value : Bytes'], '(Option Nat)', '(parseDigits value)', QP);

  for (const f of spec.fields) {
    const field = n.fields[f.name];
    if (f.kind === 'choice') {
      const chain = f.values.reduceRight((rest, v) => `(if (prim equal value ${hex(v)} : Bool) then ${SOME('Nat', `(${field.choices[v]})`)} else ${rest})`, NONE('Nat'));
      def(field.parse, ['value : Bytes'], '(Option Nat)', chain);
    } else if (field.parse !== 'parseNumber') {
      let accept = SOME('Nat', 'parsed');
      if (f.max !== undefined) accept = `(if (${f.max} <? parsed) then ${NONE('Nat')} else ${accept})`;
      if (f.min !== undefined && f.min > 0) accept = `(if (parsed <? ${f.min}) then ${NONE('Nat')} else ${accept})`;
      def(field.parse, ['value : Bytes'], '(Option Nat)', `(match (parseNumber value)\n    | Option.none => ${NONE('Nat')}\n    | Option.some parsed => ${accept})`, QP);
    }
  }

  // The request is read as bytes, never decoded as text: the exporter refuses the
  // compiled form of Lean's UTF-8 decoder as too deeply nested, and every field is
  // ASCII, so a byte that is not ASCII already makes its field fail.
  def('sliceOf', ['s : Bytes', 'start : Nat', 'count : Nat'], 'Bytes', `(match (prim slice s start count : (Option Bytes))\n    | Option.none => 0x\n    | Option.some part => part)`);
  // Where a field ends: the next space, or the end of the request. The guest is
  // built without allocation, so fields are found by position, never as a list.
  out.push({
    name: 'fieldEnd',
    kind: 'definition',
    parameters: ['s : Bytes', 'i : Nat', 'fuel : Nat'],
    result: 'Nat',
    body: `(match fuel
    | Nat.zero => i
    | Nat.succ rest => (match (byteAt s i)
      | Option.none => i
      | Option.some d => (if (d =? 32) then i else (fieldEnd s (i + 1) rest))))`,
    axioms: QP,
    recursive: 'fuel',
  });

  // One decoder per field: read the field that starts at `start`, then hand the
  // values so far to the next one. The last one requires the request to end there.
  const k = spec.fields.length;
  const values = (j) => Array.from({ length: j }, (_, i) => `v${i + 1}`);
  for (let j = k; j >= 1; j--) {
    const f = spec.fields[j - 1];
    const next =
      j === k
        ? `(if (e =? (Foundation.Bytes::byteLength s)) then ${SOME('Request', `(record Request {${spec.fields.map((g, i) => `${n.fields[g.name].id} := v${i + 1}`).join(', ')}})`)} else ${NONE('Request')})`
        : `(if (e <? (Foundation.Bytes::byteLength s)) then (decodeField${j + 1} s (e + 1) ${values(j).join(' ')}) else ${NONE('Request')})`;
    def(
      `decodeField${j}`,
      ['s : Bytes', 'start : Nat', ...values(j - 1).map((v) => `${v} : Nat`)],
      '(Option Request)',
      `(let e : Nat := (fieldEnd s start (Foundation.Bytes::byteLength s)); (match (${n.fields[f.name].parse} (sliceOf s start (prim subtract e start : Nat)))
    | Option.none => ${NONE('Request')}
    | Option.some v${j} => ${next}))`,
      QP,
    );
  }

  for (const r of spec.rules) def(n.rules[r.id], ['request : Request'], 'Bool', conjunction(r.when.map((c) => condition(spec, n, c))));

  const chain = spec.rules.reduceRight((rest, r) => `(if (${n.rules[r.id]} request) then (${n.answers[r.then]}) else ${rest})`, `(${n.answers[spec.otherwise]})`);
  def('judge', ['request : Request'], 'Nat', chain);

  const encode = spec.outcomes.reduceRight((rest, o) => `(if (code =? (${n.answers[o]})) then ${hex(o)} else ${rest})`, hex(INVALID));
  def('encodeAnswer', ['code : Nat'], 'Bytes', encode);

  const limits = requestLimits(spec);
  def(
    'dispatch',
    ['value : Bytes'],
    'Bytes',
    `(if (${limits.requestMaximum} <? (Foundation.Bytes::byteLength value)) then ${hex(INVALID)} else (match (decodeField1 value 0)
    | Option.none => ${hex(INVALID)}
    | Option.some request => (encodeAnswer (judge request))))`,
    QP,
  );

  const vectors = acceptanceVectors(spec);
  const root = `${n.prefix}.${n.module}.dispatch`;
  const AV = 'Foundation.View.Text.V1.Model::AcceptanceVector';
  const vectorList = vectors.reduceRight((rest, v) => `(cons (record ${AV} {request := ${hex(v.bytes)}, response := ${hex(v.response)}}) ${rest})`, `(nil ${AV})`);
  const view = `(record Foundation.View.Text.V1.Model::TextView {title := ${quote(spec.title)}, heading := ${quote(spec.title)}, inputLabel := ${quote(`Request: ${spec.fields.map((f) => f.name).join(' ')}`)}, submitLabel := "Decide", outputLabel := "Answer", inputError := "Invalid request", responseError := "Invalid response"})`;
  def(
    'application',
    [],
    'Foundation.View.Text.V1.Model::TextApplication',
    `(record Foundation.View.Text.V1.Model::TextApplication {profile := "prismpm/text-application/1", name := ${quote(stem(spec.title))}, cargoName := ${quote(`prism-${spec.name}`)}, cargoVersion := "0.1.0", cargoDescription := ${quote(describedBy(spec, specAddress))}, cargoRepository := "https://github.com/UOR-Foundation/PrismPM", cargoHomepage := "https://github.com/UOR-Foundation/PrismPM", libraryRoots := (cons ${quote(root)} (nil String)), acceptanceVectors := ${vectorList}, entryRoot := ${quote(root)}, coreContract := "hologram:guest/core-wasm@1", requestMaximum := (${limits.requestMaximum} : UInt32), responseMaximum := (${limits.responseMaximum} : UInt32), guestAllocationMaximum := (${limits.guestAllocationMaximum} : UInt32), capabilitiesEmpty := true, fatArchive := true, primaryLayer := (0 : UInt8), viewLayer := (1 : UInt8), view := ${view}})`,
  );

  return out;
}

/**
 * The proofs module: one Boolean "breaks" test and one theorem per property,
 * over the definitions of the app module, which it imports. It is not a build
 * entrypoint, so the app's build never compiles the proofs, while
 * `lexlean verify --all` proves them about the very definitions the guest is
 * built from. (The build compiles Lean with two threads under a fixed ten
 * minute limit, which `omega` proofs in the app module can exceed.)
 */
export function proofDeclarations(spec, n = names(spec)) {
  const out = [];
  const q = (name) => `${n.module}::${name}`;
  const inBounds = bounds(spec);
  const constants = [...Object.values(n.answers), ...Object.values(n.fields).flatMap((f) => Object.values(f.choices))];
  for (const p of spec.properties) {
    const answer = `((${q('judge')} request) =? (${q(n.answers[p.then])}))`;
    const body = conjunction([...[...inBounds, ...p.when].map((c) => condition(spec, n, c, q)), p.kind === 'never' ? answer : `(not ${answer})`]);
    out.push({ name: n.properties[p.id].breaks, kind: 'definition', parameters: [`request : ${q('Request')}`], result: 'Bool', body, axioms: null });
    const unfold = [...new Set([n.properties[p.id].breaks, q('judge'), ...Object.values(n.rules).map(q), ...constants.map(q)])].sort();
    out.push({
      name: n.properties[p.id].theorem,
      kind: 'theorem',
      parameters: [`request : ${q('Request')}`],
      statement: `((${n.properties[p.id].breaks} request) = false)`,
      unfold,
      axioms: QP,
    });
  }
  return out;
}

/**
 * The canonical layout `lexlean fmt` gives a declaration, so the generated model
 * passes `lexlean fmt --check` as written (a template instance's gate runs it).
 * A record `kind{...}` or a list `[...]` stays on one line when that line,
 * indentation and trailing comma included, fits in WIDTH columns; otherwise it
 * breaks to one entry per line, two columns deeper, and its entries are laid
 * out the same way. A term is never broken, however long.
 */
export const WIDTH = 100;

const list = (items) => ({ list: items });
const record = (head, fields) => ({ record: head, fields });

/** A value on one line, or null when it cannot be (a term with a line break in it). */
function flat(value) {
  if (typeof value === 'string') return value.includes('\n') ? null : value;
  if (value.list) {
    const items = value.list.map(flat);
    return items.includes(null) ? null : `[${items.join(', ')}]`;
  }
  const fields = value.fields.map(([key, v]) => {
    const f = flat(v);
    return f === null ? null : `${key}: ${f}`;
  });
  return fields.includes(null) ? null : `${value.record}{${fields.join(', ')}}`;
}

/** The lines of `value` written after `lead` at `indent` columns, followed by `tail` (a comma, or nothing). */
function layout(lead, value, indent, tail) {
  const pad = ' '.repeat(indent);
  const one = flat(value);
  if (one !== null && indent + lead.length + one.length + tail.length <= WIDTH) return [`${pad}${lead}${one}${tail}`];
  if (typeof value === 'string') return [`${pad}${lead}${value}${tail}`];
  if (value.list) {
    if (value.list.length === 0) return [`${pad}${lead}[]${tail}`];
    return [`${pad}${lead}[`, ...value.list.flatMap((item, i) => layout('', item, indent + 2, i < value.list.length - 1 ? ',' : '')), `${pad}]${tail}`];
  }
  return [`${pad}${lead}${value.record}{`, ...value.fields.flatMap(([key, v], i) => layout(`${key}: `, v, indent + 2, i < value.fields.length - 1 ? ',' : '')), `${pad}}${tail}`];
}

function declarationValue(d) {
  const axioms = d.axioms && d.axioms !== '[]' ? [['axioms', list(d.axioms.slice(1, -1).split(', '))]] : [];
  if (d.kind === 'structure') return record('structure', [['name', d.name], ['type_parameters', list([])], ['parameters', list([])], ['fields', list(d.fields)]]);
  if (d.kind === 'theorem') return record('theorem', [['name', d.name], ['parameters', list(d.parameters)], ['statement', d.statement], ['proof', record('linear_arithmetic', [['definitions', list(d.unfold)]])], ...axioms]);
  const recursive = d.recursive ? [['recursive_argument', d.recursive]] : [];
  return record('definition', [['name', d.name], ['parameters', list(d.parameters)], ['result', d.result], ['body', d.body], ...axioms, ...recursive]);
}

export const declarationText = (d) => layout('', declarationValue(d), 0, '').join('\n');

const moduleText = (name, imports, decls) =>
  [
    `\\begin{lexlean}{${name}}`,
    '\\useglossary{lexlean.std.bool@1.2.0}',
    '\\useglossary{lexlean.std.nat@1.2.0}',
    '\\useglossary{prism.arch@1.0.0}',
    ...imports.map((m) => `\\importmodule{${m}}`),
    '\\title{Text application model}',
    '\\begin{semanticmodule}',
    decls.map(declarationText).join('\n\n'),
    '\\end{semanticmodule}',
    '\\end{lexlean}',
    '',
  ].join('\n');

const withAxioms = (decls, axioms) => decls.map((d) => (Object.hasOwn(axioms, d.name) ? { ...d, axioms: axioms[d.name] } : d));

/** The app module: everything the guest runs. `axioms` overrides the expected set per declaration name. */
export function renderModel(spec, { axioms = {}, specAddress = null } = {}) {
  const n = names(spec);
  return moduleText(n.module, ['Foundation.Bytes', 'Foundation.View.Text.V1.Model'], withAxioms(declarations(spec, n, { specAddress }), axioms));
}

/**
 * The seal: how an application says which spec it was built from. Its purpose,
 * then the spec's UOR content address in `[prism/spec/1 blake3:<hex>]` — the
 * same kappa the holo labels its own blobs with, so the spec is named the way
 * everything else in a holo is named.
 *
 * It is the model's `cargoDescription`, so it is inside the holo's model blob,
 * whose label is that blob's kappa and whose bytes the archive's footer
 * covers: changing the seal changes the holo, and the integrity check refuses
 * it. `specAddressOf` reads it back, and an app can be held to its spec with
 * nothing but the holo and the JSON. A blob of its own, whose label would be
 * exactly this address, is where the spec itself belongs; that is the Rust
 * builder's to add.
 */
export const SPEC_SEAL_MARK = /\[prism\/spec\/1 (blake3:[0-9a-f]{64})\]/;
export const describedBy = (spec, specAddress) => (specAddress ? `${spec.purpose || spec.title} [prism/spec/1 ${specAddress}]` : spec.purpose || spec.title);

/** The proofs module, or null when the spec states no property. */
export function renderProofs(spec, { axioms = {} } = {}) {
  const n = names(spec);
  if (spec.properties.length === 0) return null;
  return moduleText(n.proofs, [n.module], withAxioms(proofDeclarations(spec, n), axioms));
}

/**
 * The acceptance vectors: the user's examples, one request per distinct way the
 * rules can decide, and the malformed requests every app must refuse, each
 * answered by the evaluator. One vector is exactly the guest allocation bound
 * long, as the text profile requires (SPEC.md section 3, PP2009).
 */
export function acceptanceVectors(spec) {
  const enc = (s) => new TextEncoder().encode(s);
  const out = [];
  const seen = new Set();
  const add = (bytes, why) => {
    const key = Buffer.from(bytes).toString('hex');
    if (seen.has(key)) return;
    seen.add(key);
    out.push({ bytes, request: new TextDecoder().decode(bytes), response: respond(spec, bytes), why });
  };
  for (const e of spec.examples) if (respond(spec, enc(e.request)) === e.response) add(enc(e.request), 'example');

  const all = [...spec.rules.flatMap((r) => r.when), ...spec.properties.flatMap((p) => p.when)];
  const shapes = new Set();
  let decided = 0;
  for (const request of sampleRequests(spec, 50000)) {
    const { rule } = judge(spec, request);
    const shape = `${rule}|${all.map((c) => (holds(c, request) ? 1 : 0)).join('')}`;
    if (shapes.has(shape)) continue;
    shapes.add(shape);
    add(enc(encodeRequest(spec, request)), rule ? `decided by ${rule}` : 'decided by otherwise');
    if (++decided >= 24) break;
  }

  const valid = (() => {
    for (const r of sampleRequests(spec, 1)) return r;
    return null;
  })();
  const { requestMaximum, guestAllocationMaximum } = requestLimits(spec);
  add(enc(''), 'empty request');
  add(Uint8Array.of(0xff), 'not UTF-8');
  if (valid) {
    const parts = encodeRequest(spec, valid).split(' ');
    add(enc(parts.slice(0, -1).join(' ')), 'a field missing');
    add(enc(`${parts.join(' ')} ${parts[parts.length - 1]}`), 'an extra field');
    add(enc(parts.join('  ')), 'two spaces');
    spec.fields.forEach((f, i) => {
      const with_ = (v) => enc(parts.map((p, j) => (j === i ? v : p)).join(' '));
      if (f.kind === 'choice') add(with_('unknown'), `${f.name} not a choice`);
      else {
        add(with_('07'), `${f.name} with a leading zero`);
        add(with_('-1'), `${f.name} negative`);
        add(with_('+1'), `${f.name} with a plus sign`);
        add(with_('1'.padEnd(19, '0')), `${f.name} with 19 digits`);
        add(with_('١'), `${f.name} with a non-ASCII digit`);
        add(with_('x'), `${f.name} not a number`);
        if (f.min !== undefined && f.min > 0) add(with_(String(f.min - 1)), `${f.name} below its minimum`);
        if (f.max !== undefined) add(with_(String(f.max + 1)), `${f.name} above its maximum`);
      }
    });
  }
  add(enc('x'.repeat(requestMaximum)), 'exactly the request bound');
  add(enc('x'.repeat(requestMaximum + 1)), 'one byte over the request bound');
  add(enc('x'.repeat(guestAllocationMaximum)), 'exactly the guest allocation bound');
  return out;
}

/**
 * A request for a scenario: the user's own example when one fits, else the
 * first request the counterexample search would try. Both are answered by the
 * same evaluator the vectors replay, so a scenario never claims what the rules
 * do not do.
 */
function witness(spec, fits) {
  for (const e of spec.examples) {
    const parts = e.request.split(' ');
    if (parts.length !== spec.fields.length) continue;
    const request = Object.fromEntries(spec.fields.map((f, i) => [f.name, f.kind === 'number' && /^\d+$/.test(parts[i]) ? BigInt(parts[i]) : parts[i]]));
    if (respond(spec, new TextEncoder().encode(e.request)) !== INVALID && fits(request)) return { request, example: true };
  }
  let tried = 0;
  for (const request of sampleRequests(spec)) {
    if (fits(request)) return { request, example: false };
    if (++tried > 20000) break;
  }
  return null;
}

/**
 * SPEC.md, in the OpenSpec format (`## Purpose`, `## Requirements`, then a
 * `### Requirement:` with SHALL and a `#### Scenario:` with WHEN and THEN for
 * each claim), so a reader and OpenSpec's own tools see a familiar spec. Each
 * requirement says whether Lean proves it or the vectors test it. It reads
 * why, then how, then what: the purpose and guarantees, the rules, the request.
 */
export function renderSpecDocument(spec, status = {}) {
  const n = names(spec);
  const badge = (id) => ({ proved: 'proved by Lean for every request', failing: 'not proved: see the counterexample', tested: 'tested only' })[status.properties?.[id]] ?? 'to be proved by Lean for every request';
  const because = (why) => (why ? `, because ${why.replace(/[.]$/, '')}` : '');
  const plural = (k, one) => `${k} ${one}${k === 1 ? '' : 's'}`;
  const decide = (request) => judge(spec, request);
  const scenario = (title, found, then) => {
    if (!found) return [`#### Scenario: ${title}`, '', `- **WHEN** a request meets the conditions above`, ...then(null)];
    const text = encodeRequest(spec, found.request);
    return [`#### Scenario: ${title}`, '', `- **WHEN** the request is \`${text}\`${found.example ? ' (an acceptance example)' : ''}`, ...then(decide(found.request))];
  };
  const accepts = (f) => (f.kind === 'choice' ? f.values.map((v) => `\`${v}\``).join(', ') : `whole number${f.min !== undefined ? ` from ${f.min}` : ''}${f.max !== undefined ? ` up to ${f.max}` : ''}`);

  const guarantees = spec.properties.flatMap((p) => {
    const conds = describeConditions(p.when);
    const shall = p.kind === 'never' ? `SHALL NOT answer \`${p.then}\`` : `SHALL answer \`${p.then}\``;
    const found = witness(spec, (r) => p.when.every((c) => holds(c, r)));
    return [
      `### Requirement: ${p.kind === 'never' ? 'Never' : 'Always'} ${p.then} when ${conds}`,
      '',
      `The app ${shall} when ${conds}${because(p.why)}.`,
      '',
      `Verified: ${badge(p.id)} (${p.id}, theorem \`${n.properties[p.id].theorem}\`).`,
      '',
      ...scenario(`${p.id} holds`, found, (d) => [`- **THEN** the answer ${p.kind === 'never' ? `SHALL NOT be \`${p.then}\`` : `SHALL be \`${p.then}\``}`, ...(d ? [`- **AND** the app answers \`${d.outcome}\`, decided by ${d.rule ?? 'the fallback'}`] : [])]),
      '',
    ];
  });

  const rules = spec.rules.flatMap((r, i) => {
    const conds = describeConditions(r.when);
    const found = witness(spec, (q) => decide(q).rule === r.id);
    return [
      `### Requirement: ${r.id} ${r.then} when ${conds}`,
      '',
      `The app SHALL answer \`${r.then}\` when ${conds}${i ? ` and no earlier rule matches` : ''}${because(r.why)}.`,
      '',
      `Verified: tested by the acceptance vectors (\`${n.rules[r.id]}\`).`,
      '',
      ...scenario(`${r.id} decides`, found, () => [`- **THEN** the answer SHALL be \`${r.then}\`, decided by ${r.id}`]),
      '',
    ];
  });
  const overlap = spec.rules.length > 1 ? witness(spec, (q) => spec.rules.filter((r) => r.when.every((c) => holds(c, q))).length > 1) : null;
  const fallback = witness(spec, (q) => decide(q).rule === null);

  const lines = [
    `# ${spec.name} Specification`,
    '',
    '<!-- Generated by Prism from prism/spec.json. Change the app through your agent with /prism, not by editing this file. -->',
    '',
    '## Purpose',
    '',
    `${spec.title}: ${spec.purpose.replace(/[.]?$/, '.')}`,
    '',
    '## Requirements',
    '',
    ...(spec.properties.length ? guarantees : ['No guarantee is stated yet. A guarantee is what must never or always happen, and each one becomes a theorem Lean proves.', '']),
    ...rules,
    `### Requirement: Otherwise ${spec.otherwise}`,
    '',
    `The app SHALL answer \`${spec.otherwise}\` when no rule matches${because(spec.otherwiseWhy)}.`,
    '',
    'Verified: tested by the acceptance vectors.',
    '',
    ...scenario('No rule matches', fallback, () => [`- **THEN** the answer SHALL be \`${spec.otherwise}\``]),
    '',
    ...(overlap
      ? [
          '### Requirement: The first matching rule decides',
          '',
          `The app SHALL try ${spec.rules.map((r) => r.id).join(', ')} in that order, and the first rule that matches SHALL decide.`,
          '',
          'Verified: tested by the acceptance vectors.',
          '',
          ...scenario('Two rules match', overlap, (d) => [`- **THEN** the answer SHALL be \`${d.outcome}\`, decided by ${d.rule}, the earlier rule`]),
          '',
        ]
      : []),
    '### Requirement: Request and answers',
    '',
    `The app SHALL read one request as one line of ${plural(spec.fields.length, 'field')} separated by single spaces, in this order, and SHALL answer exactly one of ${spec.outcomes.map((o) => `\`${o}\``).join(', ')}, or \`invalid\` for anything else.`,
    '',
    '| Field | Accepts | Means |',
    '| --- | --- | --- |',
    ...spec.fields.map((f) => `| \`${f.name}\` | ${accepts(f)} | ${f.means ?? ''} |`),
    '',
    '| Answer | Means |',
    '| --- | --- |',
    ...spec.outcomes.map((o) => `| \`${o}\` | ${spec.meanings?.[o] ?? ''} |`),
    `| \`invalid\` | the request is malformed |`,
    '',
    'Verified: reading a request is tested, not proved.',
    '',
    '#### Scenario: A malformed request is invalid',
    '',
    `- **WHEN** the request is not ${plural(spec.fields.length, 'field')} in this order`,
    '- **THEN** the answer SHALL be `invalid`',
    '',
    ...(spec.limits ? ['## Out of scope', '', ...spec.limits.map((l) => `- ${l}`), ''] : []),
    '## Examples',
    '',
    ...(spec.examples.length ? spec.examples.map((e) => `- \`${e.request}\` gives \`${e.response}\``) : ['No example yet.']),
    '',
    '## What is proved and what is tested',
    '',
    'Lean proves each property marked proved above, for every request that decodes, with no exceptions and no sampling. The proof is about `judge`, the function the WebAssembly guest runs, because both come from the same model.',
    '',
    `The parsing of requests is tested, not proved: ${status.vectors ?? 'the'} acceptance vectors in the app replay through the built guest, including malformed and oversized requests.`,
    '',
    'Nothing here says the rules are the right rules. That is the part only you can judge.',
    '',
    '## Files',
    '',
    `- \`prism/spec.json\` is this spec as data, the app's one input, in canonical JSON (RFC 8785). The model is rendered from it alone, so the same spec.json always gives the same model.`,
    `- \`src/${n.module}.lex.tex\` is the LexLean model generated from it. It is the single source of the app.`,
    '',
  ];
  return lines.join('\n');
}
