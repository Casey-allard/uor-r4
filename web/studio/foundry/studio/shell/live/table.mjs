// The decision table: a Prism app decides over a finite domain (each choice, each whole number between a
// field's min and max), so its rules can be materialised once, per logic kappa, into a typed array: one
// cell per request, holding the index of the rule that decides it (the rule count for "otherwise"). Then
// an answer is one array read, every guarantee is checked against every request, and two versions of the
// rules are compared answer by answer.
//
// `tableKernel` is one self-contained function, with no imports and no free names, so the same code runs in
// the gate under Node and in the live frame's worker, built from its source text. The interpreter
// (prism.js's prismAnswer) stays the oracle: tools/live_preview.mjs checks the table against it on every
// request of the fixture.
export function tableKernel() {
  const CAP = 1 << 22; // about 4 million cells; past this, the rules are interpreted directly

  /** Each field's values in table order, or why the domain is not tabulated. */
  function domain(logic) {
    const lists = [];
    let size = 1;
    for (const f of (logic && logic.fields) || []) {
      if (f.kind === "choice") lists.push(f.values.slice());
      else {
        if (f.max === undefined || f.max === null) return { size: 0, reason: `${f.name} has no maximum` };
        const n = f.max - f.min + 1;
        if (!(n > 0)) return { size: 0, reason: `${f.name} is empty` };
        lists.push({ min: f.min, count: n });
      }
      size *= Array.isArray(lists[lists.length - 1]) ? lists[lists.length - 1].length : lists[lists.length - 1].count;
      if (size > CAP) return { size, reason: `${size} requests is more than the ${CAP} the table holds` };
    }
    if (!lists.length || !logic.otherwise) return { size: 0, reason: "the rules are not complete yet" };
    return { size, lists };
  }

  // A condition as a test on the current value of every field: a choice is its index in the field's
  // values, a number is the number itself.
  function compile(logic, when) {
    const at = new Map(logic.fields.map((f, i) => [f.name, i]));
    const tests = when.map((c) => {
      const i = at.get(c.field), f = logic.fields[i];
      if (f.kind === "choice") { const k = f.values.indexOf(c.value); return c.op === "is" ? (v) => v[i] === k : (v) => v[i] !== k; }
      const other = c.value !== null && typeof c.value === "object";
      const j = other ? at.get(c.value.field) : -1, n = other ? 0 : Number(c.value);
      const right = other ? (v) => v[j] : () => n;
      switch (c.op) {
        case "<": return (v) => v[i] < right(v);
        case "<=": return (v) => v[i] <= right(v);
        case ">": return (v) => v[i] > right(v);
        case ">=": return (v) => v[i] >= right(v);
        case "=": return (v) => v[i] === right(v);
        case "!=": return (v) => v[i] !== right(v);
        default: return () => false;
      }
    });
    return (v) => { for (let t = 0; t < tests.length; t++) if (!tests[t](v)) return false; return true; };
  }

  // The fast form of a rule for the table: a condition on one field against a constant is a truth mask
  // over that field's value positions (one array read); a comparison of two number fields stays a
  // comparison. Same meaning as `compile`, which the gate holds it to through the interpreter.
  const OPS = { "<": 0, "<=": 1, ">": 2, ">=": 3, "=": 4, "!=": 5 };
  function fast(logic, d, when) {
    const at = new Map(logic.fields.map((f, i) => [f.name, i]));
    return when.map((c) => {
      const i = at.get(c.field), f = logic.fields[i];
      if (f.kind === "number" && c.value !== null && typeof c.value === "object") return { pair: true, i, j: at.get(c.value.field), op: OPS[c.op] };
      const test = compile(logic, [c]);
      const n = Array.isArray(d.lists[i]) ? d.lists[i].length : d.lists[i].count, base = Array.isArray(d.lists[i]) ? 0 : d.lists[i].min;
      const mask = new Uint8Array(n), probe = logic.fields.map(() => 0);
      for (let k = 0; k < n; k++) { probe[i] = base + k; mask[k] = test(probe) ? 1 : 0; }
      return { pair: false, i, mask };
    });
  }
  function holds(conds, idx, v) {
    for (let t = 0; t < conds.length; t++) {
      const c = conds[t];
      if (!c.pair) { if (!c.mask[idx[c.i]]) return false; continue; }
      const a = v[c.i], b = v[c.j];
      const ok = c.op === 0 ? a < b : c.op === 1 ? a <= b : c.op === 2 ? a > b : c.op === 3 ? a >= b : c.op === 4 ? a === b : a !== b;
      if (!ok) return false;
    }
    return true;
  }

  /** The table: cells[r] is the deciding rule of request r (rules.length for otherwise). */
  function build(logic) {
    const d = domain(logic);
    if (!d.lists) return { size: d.size, reason: d.reason };
    const rules = logic.rules.map((r) => fast(logic, d, r.when));
    const R = rules.length;
    const cells = R < 255 ? new Uint8Array(d.size) : new Uint16Array(d.size);
    const radix = d.lists.map((l) => (Array.isArray(l) ? l.length : l.count));
    const base = d.lists.map((l) => (Array.isArray(l) ? 0 : l.min));
    const v = base.slice(), idx = radix.map(() => 0), last = radix.length - 1;
    for (let r = 0; r < d.size; r++) {
      let k = 0;
      while (k < R && !holds(rules[k], idx, v)) k++;
      cells[r] = k;
      // odometer, last field fastest
      let i = last;
      while (i >= 0) { if (++idx[i] < radix[i]) { v[i] = base[i] + idx[i]; break; } idx[i] = 0; v[i] = base[i]; i--; }
    }
    return { size: d.size, cells, radix, base, lists: d.lists };
  }

  /** The cell of one set of field values (strings, as the frame holds them), or -1 when it is not a request. */
  function cellOf(logic, table, values) {
    let r = 0;
    for (let i = 0; i < logic.fields.length; i++) {
      const f = logic.fields[i], s = values[f.name];
      let k;
      if (f.kind === "choice") { k = f.values.indexOf(s); if (k < 0) return -1; }
      else { if (!/^(0|[1-9][0-9]*)$/.test(String(s))) return -1; k = Number(s) - table.base[i]; if (k < 0 || k >= table.radix[i]) return -1; }
      r = r * table.radix[i] + k;
    }
    return r;
  }

  /** The request line of a cell, as a user would type it. */
  function requestOf(table, r) {
    const parts = new Array(table.radix.length);
    for (let i = table.radix.length - 1; i >= 0; i--) {
      const k = r % table.radix[i];
      r = (r - k) / table.radix[i];
      parts[i] = Array.isArray(table.lists[i]) ? table.lists[i][k] : String(table.base[i] + k);
    }
    return parts.join(" ");
  }

  /** Every guarantee against every request: its first counterexample, or how many requests it covered. */
  function check(logic, table) {
    const R = logic.rules.length;
    const outcome = (k) => (k < R ? logic.rules[k].then : logic.otherwise);
    return (logic.properties || []).map((p) => {
      const when = compile(logic, p.when);
      const v = table.base.slice(), idx = table.radix.map(() => 0), last = table.radix.length - 1;
      let covered = 0;
      for (let r = 0; r < table.size; r++) {
        if (when(v)) {
          covered++;
          const o = outcome(table.cells[r]);
          if (p.kind === "never" ? o === p.then : o !== p.then) return { ok: false, request: requestOf(table, r), outcome: o, rule: table.cells[r], covered, of: table.size };
        }
        let i = last;
        while (i >= 0) { if (++idx[i] < table.radix[i]) { v[i] = table.base[i] + idx[i]; break; } idx[i] = 0; v[i] = table.base[i]; i--; }
      }
      return { ok: true, covered, of: table.size };
    });
  }

  /** Answer by answer, what changed between two versions of the rules over the same requests. */
  function diff(beforeLogic, before, afterLogic, after, samples = 5) {
    if (!before || !after || before.size !== after.size || before.radix.join() !== after.radix.join() || JSON.stringify(before.lists) !== JSON.stringify(after.lists)) return null;
    const ob = (k) => (k < beforeLogic.rules.length ? beforeLogic.rules[k].then : beforeLogic.otherwise);
    const oa = (k) => (k < afterLogic.rules.length ? afterLogic.rules[k].then : afterLogic.otherwise);
    let changed = 0;
    const shown = [];
    for (let r = 0; r < after.size; r++) {
      const b = ob(before.cells[r]), a = oa(after.cells[r]);
      if (b !== a) { changed++; if (shown.length < samples) shown.push({ request: requestOf(after, r), before: b, after: a }); }
    }
    return { changed, of: after.size, samples: shown };
  }

  return { CAP, domain, build, cellOf, requestOf, check, diff };
}
