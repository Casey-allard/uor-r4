// Local obligations: the live mirror of `regionCheck` in the Prism Live spike's Regions.lean, whose
// soundness is proved there (`regionCheck_sound`), and which Lean's kernel decides for Verdant.
//
// For each property p and each rule r whose answer would break p: the part of r's box inside p's
// box must be covered by the boxes of the rules before r. One more obligation covers `otherwise`.
// Each obligation is keyed by meaning: its region (r's box inside p's box), the outcome test, and
// the sorted set of earlier boxes that touch that region (their order, and boxes elsewhere, do not
// matter). Results are kept by key, so an edit re-proves only
// the obligations whose key is new; every other one is a hit. That cache is the delta.

const cache = new Map(); // key -> null (holds) | witness point

// Keys are UOR content addresses: BLAKE3 of the obligation's canonical bytes, as everywhere in Foundry.
import { kappa } from "../prism-code/src/blake3.mjs";
const enc = new TextEncoder();
const kappaOfText = (s) => kappa(enc.encode(s));


// The box of a list of conditions (`boxOf`): exact for <, <=, >, >=, =; `!=` / `is not` does not
// narrow, so `strict` callers refuse it. null when the box is empty.
function boxOf(spec, when, strict) {
  const B = spec.fields.map((f) => (f.kind === "choice" ? [0, f.values.length - 1] : [f.min ?? 0, f.max]));
  for (const c of when) {
    const i = spec.fields.findIndex((f) => f.name === c.field), f = spec.fields[i];
    // A comparison with another field is not a box: a rule with one is refused, a guarantee over-covers.
    if (c.value !== null && typeof c.value === "object") { if (strict) return undefined; continue; }
    const v = f.kind === "choice" ? f.values.indexOf(c.value) : c.value;
    const lo = (x) => (B[i][0] = Math.max(B[i][0], x)), hi = (x) => (B[i][1] = Math.min(B[i][1], x));
    if (c.op === "is" || c.op === "=") { lo(v); hi(v); }
    else if (c.op === "<") { if (v === 0) { lo(1); hi(0); } else hi(v - 1); }
    else if (c.op === "<=") hi(v);
    else if (c.op === ">") lo(v + 1);
    else if (c.op === ">=") lo(v);
    else if (strict) return undefined; // a rule with != : not a box
  }
  return B.every(([l, h]) => l <= h) ? B : null;
}

const meet = (A, B) => { const m = A.map(([al, ah], k) => [Math.max(al, B[k][0]), Math.min(ah, B[k][1])]); return m.every(([l, h]) => l <= h) ? m : null; };
function sub(A, B) {
  if (A.some(([al, ah], k) => ah < B[k][0] || B[k][1] < al)) return [A];
  const out = []; const cur = A.map((p) => p.slice());
  for (let k = 0; k < A.length; k++) {
    const [bl, bh] = B[k];
    if (cur[k][0] < bl) { const p = cur.map((q) => q.slice()); p[k] = [cur[k][0], bl - 1]; out.push(p); cur[k][0] = bl; }
    if (bh < cur[k][1]) { const p = cur.map((q) => q.slice()); p[k] = [bh + 1, cur[k][1]]; out.push(p); cur[k][1] = bh; }
  }
  return out;
}
const subAll = (A, Bs) => Bs.reduce((ps, B) => ps.flatMap((p) => sub(p, B)), [A]);

/**
 * Run every obligation. Returns { ok, holds, witness, broken, total, hits, fresh, manifest, ms }.
 * ok=false when the model is outside the fragment (a rule with != / is not): use the kernel check.
 */
export function checkObligations(spec) {
  const t0 = performance.now();
  const R = spec.rules.map((r) => boxOf(spec, r.when, true));
  if (R.some((b) => b === undefined)) return { ok: false };
  const outcome = (o) => spec.outcomes.indexOf(o);
  const keys = [];
  let hits = 0, fresh = 0, witness = null, broken = null;
  const full = spec.fields.map((f) => (f.kind === "choice" ? [0, f.values.length - 1] : [f.min ?? 0, f.max]));
  for (const p of spec.properties) {
    const W = boxOf(spec, p.when, false);
    if (!W) continue;
    const t = outcome(p.then);
    const isBad = (o) => (p.kind === "always" ? o !== t : o === t);
    const Wk = JSON.stringify(W) + p.kind + t;
    // An obligation depends only on its own region M = A ∩ W and the earlier boxes that touch M:
    // the key names exactly those, so an edit elsewhere leaves it a hit.
    const earlierBoxes = [];
    const run = (A, test) => {
      if (!test) return;
      const M = A && meet(A, W);
      const touching = M ? earlierBoxes.filter((B) => meet(B, M)).map((B) => JSON.stringify(B)).sort() : [];
      const key = kappaOfText(JSON.stringify(M) + "|" + Wk + "|" + touching.join(";"));
      keys.push(key);
      let res;
      if (cache.has(key)) { res = cache.get(key); hits++; }
      else {
        const pieces = M ? subAll(M, earlierBoxes.filter((B) => meet(B, M))) : [];
        res = pieces.length ? pieces[0].map(([l]) => l) : null;
        cache.set(key, res); fresh++;
      }
      if (res && !witness) { witness = res; broken = p.id; }
    };
    for (let i = 0; i < spec.rules.length; i++) {
      run(R[i], R[i] && isBad(outcome(spec.rules[i].then)));
      if (R[i]) earlierBoxes.push(R[i]);
    }
    run(full, isBad(outcome(spec.otherwise)));
  }
  keys.sort();
  return { ok: true, holds: !witness, witness, broken, total: keys.length, hits, fresh, manifest: kappaOfText(keys.join(",")), ms: performance.now() - t0 };
}

