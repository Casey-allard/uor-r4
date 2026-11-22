/**
 * LexLean text for the facet modules the builder writes.
 *
 * The layout is the one `lexlean fmt` gives a declaration, the same rule
 * prism-code's journey renderer follows (vendor/prism-code/src/journey/render.mjs,
 * `layout`): a record `kind{...}` or a list `[...]` stays on one line when the
 * line, indentation and trailing comma included, fits in `WIDTH` columns, and
 * otherwise breaks to one entry per line two columns deeper. A term is never
 * broken. The journey keeps `layout` private and lays out only its own
 * declaration kinds (its theorems are all `linear_arithmetic`), so the facet
 * modules, whose theorems are `decide`, carry the same rule here.
 */
import { WIDTH } from '../render.mjs';

const list = (items) => ({ list: items });
const record = (head, fields) => ({ record: head, fields });

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

/** `text` split at its top-level `sep`, outside brackets and string literals. */
function splitTop(text, sep) {
  const out = [];
  let depth = 0;
  let quoted = false;
  let start = 0;
  for (let i = 0; i < text.length; i++) {
    const ch = text[i];
    if (ch === '"') quoted = !quoted;
    else if (quoted) continue;
    else if ('([{'.includes(ch)) depth++;
    else if (')]}'.includes(ch)) depth--;
    else if (depth === 0 && text.startsWith(sep, i)) {
      out.push(text.slice(start, i));
      start = i + sep.length;
      i += sep.length - 1;
    }
  }
  out.push(text.slice(start));
  return out;
}

/** The value a one-line `kind{key: v, ...}`, `[a, b]` or term denotes, for `layout`. */
function parseValue(text) {
  const head = /^([A-Za-z_][A-Za-z0-9_.]*)\{([\s\S]*)\}$/.exec(text);
  if (head) return record(head[1], head[2] === '' ? [] : splitTop(head[2], ', ').map((f) => {
    const at = f.indexOf(': ');
    return [f.slice(0, at), parseValue(f.slice(at + 2))];
  }));
  if (text.startsWith('[') && text.endsWith(']')) return list(text === '[]' ? [] : splitTop(text.slice(1, -1), ', ').map(parseValue));
  return text;
}

/**
 * Text laid out again where a line passes `WIDTH`, as `lexlean fmt` lays it
 * out: a renamed declaration is longer than the one that was laid out.
 */
export function reflow(text) {
  return text.split('\n').flatMap((line) => {
    if (line.length <= WIDTH) return [line];
    const m = /^( *)((?:[A-Za-z_][A-Za-z0-9_]*: )?)(.*?)(,?)$/.exec(line);
    return layout(m[2], parseValue(m[3]), m[1].length, m[4]);
  }).join('\n');
}

/** A LexLean string literal: printable ASCII only, without quotes or backslashes (the journey's rule). */
export const str = (text) => `"${String(text).replace(/[^\x20-\x7e]/g, '').replace(/["\\]/g, '')}"`;

/** `(cons a (cons b (nil T)))`. */
export const cons = (type, items) => items.reduceRight((rest, item) => `(cons ${item} ${rest})`, `(nil ${type})`);

/**
 * The most items one definition's list literal holds. LexLean links a module
 * through JSON whose nesting depth is bounded, and each cons nests one level,
 * so a longer list is written in parts, each part's tail a reference to the
 * next. The list stays a closed term, with no call, as the SDK reads an
 * application's metadata.
 */
export const LIST_PART = 64;

/** A list as one definition `name`, continued by `name1`, `name2`, ... when it is longer than LIST_PART. */
export function listDefinitions(name, type, items) {
  const parts = [];
  for (let i = 0; i < items.length; i += LIST_PART) parts.push(items.slice(i, i + LIST_PART));
  if (parts.length <= 1) return [definition(name, `(List ${type})`, cons(type, items))];
  const part = (i) => (i === 0 ? name : `${name}${i}`);
  // A definition is declared before it is referenced, so the last part comes first.
  return parts.map((items_, i) => definition(part(i), `(List ${type})`, items_.reduceRight((rest, item) => `(cons ${item} ${rest})`, i + 1 < parts.length ? `(${part(i + 1)})` : `(nil ${type})`))).reverse();
}

/** `(record T {k := v, ...})`. */
export const rec = (type, fields) => `(record ${type} {${Object.entries(fields).map(([k, v]) => `${k} := ${v}`).join(', ')}})`;

/** `(ctor T.c)`. */
export const ctor = (type, name) => `(ctor ${type}.${name})`;

/** A top-level definition. */
export const definition = (name, result, body, { parameters = [], recursive } = {}) => ({ kind: 'definition', name, parameters, result, body, recursive });

/** A top-level theorem proved by `decide`. */
export const theorem = (name, statement) => ({ kind: 'theorem', name, statement });

/** A structure. */
export const structure = (name, fields) => ({ kind: 'structure', name, fields });

function value(d) {
  if (d.kind === 'structure') return record('structure', [['name', d.name], ['type_parameters', list([])], ['parameters', list([])], ['fields', list(d.fields)]]);
  if (d.kind === 'theorem') return record('theorem', [['name', d.name], ['parameters', list([])], ['statement', d.statement], ['proof', 'decide{}']]);
  const recursive = d.recursive ? [['recursive_argument', d.recursive]] : [];
  return record('definition', [['name', d.name], ['parameters', list(d.parameters)], ['result', d.result], ['body', d.body], ...recursive]);
}

export const declarationText = (d) => layout('', value(d), 0, '').join('\n');

/** A whole module: its glossaries, imports, title and declarations, as `lexlean fmt` writes it. */
export function moduleText({ name, glossaries, imports, title, declarations }) {
  return [
    `\\begin{lexlean}{${name}}`,
    ...glossaries.map((g) => `\\useglossary{${g}}`),
    ...imports.map((m) => `\\importmodule{${m}}`),
    `\\title{${title}}`,
    '\\begin{semanticmodule}',
    // An append is declared once however many lists of its type a module holds.
    declarations.filter((d, i) => declarations.findIndex((e) => e.name === d.name) === i).map(declarationText).join('\n\n'),
    '\\end{semanticmodule}',
    '\\end{lexlean}',
    '',
  ].join('\n');
}
