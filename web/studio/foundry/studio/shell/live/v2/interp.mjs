// The preview projector: runs LexLean semantic modules (lexlean/semantic-module/2, the data form
// the toolchain itself produces) directly, with no compile step.
//
// Trusted, not proved. It is held to the compiled guest by replaying the same vectors through both.
// An expression it does not know is refused when the module loads; a primitive it does not know is
// refused when it is called. Neither is ever answered with a guess.

export class Unsupported extends Error {
  constructor(what, name) { super(`unsupported ${what}: ${name}`); this.code = "interp.unsupported"; this.what = what; this.name_ = name; }
}
export class Malformed extends Error {
  constructor(message) { super(message); this.code = "interp.malformed"; }
}

// Values: Nat and the fixed-width integers are BigInt, Bytes is Uint8Array, String is a string,
// Bool is a boolean, a constructor is {c, a}, a record is a plain object, a list is List.nil/List.cons.
export const NIL = { c: "List.nil", a: [] };
const NONE = { c: "Option.none", a: [] };
const some = (v) => ({ c: "Option.some", a: [v] });
export const cons = (head, tail) => ({ c: "List.cons", a: [head, tail] });
export const toArray = (l) => { const out = []; for (; l.c === "List.cons"; l = l.a[1]) out.push(l.a[0]); return out; };

export const hexToBytes = (hex) => {
  if (hex.length % 2 || /[^0-9a-f]/i.test(hex)) throw new Malformed("not hex");
  const out = new Uint8Array(hex.length / 2);
  for (let i = 0; i < out.length; i++) out[i] = parseInt(hex.substr(2 * i, 2), 16);
  return out;
};

const isBytes = (v) => v instanceof Uint8Array;
const listLength = (l) => { let n = 0n; for (; l.c === "List.cons"; l = l.a[1]) n++; return n; };
const listAppend = (l, r) => { const items = toArray(l); let out = r; for (let i = items.length - 1; i >= 0; i--) out = cons(items[i], out); return out; };
const utf8 = new TextEncoder(), strict = new TextDecoder("utf-8", { fatal: true, ignoreBOM: true });

// The third argument of quotient and remainder is the answer when the divisor is zero.
const PRIMITIVES = {
  append: (l, r) => { if (isBytes(l)) { const out = new Uint8Array(l.length + r.length); out.set(l); out.set(r, l.length); return out; } return listAppend(l, r); },
  length: (v) => (isBytes(v) ? BigInt(v.length) : listLength(v)),
  index: (v, i) => {
    if (isBytes(v)) return i < BigInt(v.length) ? some(BigInt(v[Number(i)])) : NONE;
    for (let n = 0n; v.c === "List.cons"; v = v.a[1], n++) if (n === i) return some(v.a[0]);
    return NONE;
  },
  slice: (v, start, count) => (start + count <= BigInt(v.length) ? some(v.slice(Number(start), Number(start + count))) : NONE),
  equal: (l, r) => (isBytes(l) ? l.length === r.length && l.every((b, i) => b === r[i]) : l === r),
  to_nat: (v) => v,
  subtract: (l, r) => (l > r ? l - r : 0n),
  multiply: (l, r) => l * r,
  quotient: (l, r, zero) => (r === 0n ? zero : l / r),
  remainder: (l, r, zero) => (r === 0n ? zero : l % r),
  bit_and: (l, r) => l & r,
  utf8_encode: (s) => utf8.encode(s),
  utf8_decode: (b) => { try { return some(strict.decode(b)); } catch { return NONE; } },
  compare_bytes: (l, r) => {
    const n = Math.min(l.length, r.length);
    for (let i = 0; i < n; i++) if (l[i] !== r[i]) return { c: l[i] < r[i] ? "Ordering.lt" : "Ordering.gt", a: [] };
    return { c: l.length === r.length ? "Ordering.eq" : l.length < r.length ? "Ordering.lt" : "Ordering.gt", a: [] };
  },
  // All the parts when there are at most `maximum` of them; an empty delimiter splits nothing.
  split_exact: (value, delimiter, maximum) => {
    if (delimiter === "") return NONE;
    const parts = value.split(delimiter);
    if (BigInt(parts.length) > maximum) return NONE;
    let out = NIL; for (let i = parts.length - 1; i >= 0; i--) out = cons(parts[i], out);
    return some(out);
  },
  format_decimal: (v) => v.toString(),
};

// Fixed-width integers: the result type of the primitive names the width.
const RANGES = { int8: [-(2n ** 7n), 2n ** 7n - 1n], int16: [-(2n ** 15n), 2n ** 15n - 1n], int32: [-(2n ** 31n), 2n ** 31n - 1n], int64: [-(2n ** 63n), 2n ** 63n - 1n], uint8: [0n, 2n ** 8n - 1n], uint16: [0n, 2n ** 16n - 1n], uint32: [0n, 2n ** 32n - 1n], uint64: [0n, 2n ** 64n - 1n] };
const within = (type, what) => { const range = RANGES[type?.kind === "option" ? type.value.kind : type?.kind]; if (!range) throw new Unsupported("width for " + what, JSON.stringify(type)); return (v) => v >= range[0] && v <= range[1]; };
const checked = (operate) => (type) => { const fits = within(type, "checked arithmetic"); return (l, r) => { const v = operate(l, r); return v !== null && fits(v) ? some(v) : NONE; }; };
const TYPED = {
  // Only the one canonical spelling parses: no plus sign, no leading zero, no "-0".
  parse_decimal: (type) => { const fits = within(type, "parse_decimal"); return (text) => { if (!/^(0|-?[1-9][0-9]*)$/.test(text)) return NONE; const v = BigInt(text); return fits(v) ? some(v) : NONE; }; },
  checked_add: checked((l, r) => l + r),
  checked_subtract: checked((l, r) => l - r),
  checked_multiply: checked((l, r) => l * r),
};

const BUILTIN = new Set(["List", "Option", "Result", "Bool", "Nat", "Ordering"]);

function compile(node, ctx) {
  const sub = (n) => compile(n, ctx);
  // A constructor of a type this project declares is named with its module, so two modules agree on it.
  const tag = (ref) => {
    if (ref.module) return ref.module + "::" + ref.name;
    const type = ref.name.slice(0, ref.name.lastIndexOf("."));
    return BUILTIN.has(type) ? ref.name : ctx.module + "::" + ref.name;
  };
  switch (node.kind) {
    case "var": { const name = node.name; return (env) => env[name]; }
    case "nat": case "integer": { const value = BigInt(node.value); return () => value; }
    case "bool": { const value = node.value; return () => value; }
    case "string": { const value = node.value; return () => value; }
    case "bytes": { const value = hexToBytes(node.hex); return () => value; }
    case "add": { const l = sub(node.left), r = sub(node.right); return (env) => l(env) + r(env); }
    case "beq": { const l = sub(node.left), r = sub(node.right); return (env) => l(env) === r(env); }
    case "ble": { const l = sub(node.left), r = sub(node.right); return (env) => l(env) <= r(env); }
    case "blt": { const l = sub(node.left), r = sub(node.right); return (env) => l(env) < r(env); }
    case "and": { const l = sub(node.left), r = sub(node.right); return (env) => l(env) && r(env); }
    case "or": { const l = sub(node.left), r = sub(node.right); return (env) => l(env) || r(env); }
    case "not": { const v = sub(node.value); return (env) => !v(env); }
    case "if": { const c = sub(node.condition), t = sub(node.then_value), e = sub(node.else_value); return (env) => (c(env) ? t(env) : e(env)); }
    case "nil": return () => NIL;
    case "cons": { const h = sub(node.head), t = sub(node.tail); return (env) => cons(h(env), t(env)); }
    case "constructor": { const name = tag(node.constructor), args = node.arguments.map(sub); return (env) => ({ c: name, a: args.map((f) => f(env)) }); }
    case "record": { const fields = node.fields.map((f) => [f.field, sub(f.value)]); return (env) => { const out = {}; for (const [k, f] of fields) out[k] = f(env); return out; }; }
    case "project": { const v = sub(node.value), field = node.field; return (env) => v(env)[field]; }
    case "call": {
      const name = (node.function.module ?? ctx.module) + "::" + node.function.name, args = node.arguments.map(sub);
      return (env) => { const fn = ctx.functions[name]; if (!fn) throw new Unsupported("function", name); return fn(args.map((f) => f(env))); };
    }
    case "primitive": {
      const operation = node.operation, args = node.arguments.map(sub);
      const op = TYPED[operation] ? TYPED[operation](node.result) : PRIMITIVES[operation];
      if (!op) { ctx.missing.add(operation); return () => { throw new Unsupported("primitive", operation); }; }
      return (env) => op(...args.map((f) => f(env)));
    }
    case "match": {
      const scrutinee = sub(node.scrutinee);
      const branches = node.branches.map((b) => ({ name: tag(b.constructor), binders: b.binders, body: sub(b.body) }));
      return (env) => {
        const value = scrutinee(env);
        if (typeof value === "boolean") {
          const want = value ? "Bool.true" : "Bool.false";
          for (const b of branches) if (b.name === want) return b.body(env);
        } else if (typeof value === "bigint") {
          for (const b of branches) {
            if (b.name === "Nat.zero" && value === 0n) return b.body(env);
            if (b.name === "Nat.succ" && value > 0n) { const inner = Object.create(env); inner[b.binders[0]] = value - 1n; return b.body(inner); }
          }
        } else if (value.c === undefined) {
          // A structure, matched by its one constructor: the binders are its fields in declared order.
          for (const b of branches) {
            const fields = ctx.structures[b.name];
            if (!fields) continue;
            const inner = Object.create(env);
            b.binders.forEach((name, i) => { inner[name] = value[fields[i]]; });
            return b.body(inner);
          }
        } else {
          for (const b of branches) if (b.name === value.c) {
            if (!b.binders.length) return b.body(env);
            const inner = Object.create(env);
            b.binders.forEach((name, i) => { inner[name] = value.a[i]; });
            return b.body(inner);
          }
        }
        throw new Malformed("no branch matched: " + (typeof value === "object" ? value.c ?? JSON.stringify(Object.keys(value)) : typeof value + " " + String(value)) + " against " + branches.map((b) => b.name).join(", "));
      };
    }
    default: throw new Unsupported("expression", node.kind);
  }
}

// Compiles every definition of every module once. `modules` maps a module name, as a source file
// declares it, to its data form. Theorems are not run: proving is Lean's job, not the tab's.
export function loadProject(modules) {
  const functions = {}, theorems = [], missing = new Set(), structures = {};
  for (const [module, data] of Object.entries(modules)) {
    if (data.spec !== "lexlean/semantic-module/2") throw new Unsupported("module form", String(data.spec));
    const ctx = { module, functions, missing, structures };
    for (const d of data.declarations) {
      if (d.kind === "definition") {
        const names = d.parameters.map((p) => p.name), body = compile(d.body, ctx);
        functions[module + "::" + d.name] = (args) => { const env = {}; names.forEach((n, i) => { env[n] = args[i]; }); return body(env); };
      } else if (d.kind === "theorem") theorems.push(module + "::" + d.name);
      else if (d.kind === "structure") structures[module + "::" + d.name + ".mk"] = d.fields.map((f) => f.name);
      else if (d.kind !== "inductive") throw new Unsupported("declaration", d.kind);
    }
  }
  return {
    call: (name, ...args) => { const fn = functions[name]; if (!fn) throw new Unsupported("function", name); return fn(args); },
    theorems, definitions: Object.keys(functions), missingPrimitives: [...missing].sort(),
  };
}

// One module on its own; names need no module.
export function load(data) {
  const project = loadProject({ Main: data });
  return { call: (name, ...args) => project.call("Main::" + name, ...args), theorems: project.theorems.map((t) => t.slice(6)), definitions: project.definitions.map((t) => t.slice(6)), missingPrimitives: project.missingPrimitives };
}
