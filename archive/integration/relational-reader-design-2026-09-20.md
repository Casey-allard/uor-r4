# Design note — learned contextual geometric relational reader

**Execution qualification:** this prospective note planned soft/straight-through descriptors and natural-text fit/regression; the delivered run used discrete coordinate search and no natural-text fit/CE panel. The [principal review](relational-reader-review-2026-09-20.md) identifies further train/evaluation and task-design defects. Preserve this original plan; use the [current successor](deepseek-competitive-reader-step-2026-09-20.md) for execution.

Written **before** any fitting or scoring, per the execution prompt. Records the mechanism choice,
the shared pool, the loss-aligned objective, the comparators and the falsifier.

## Mechanism

```
q_t  = Q(x_{i-1})                   learned query descriptor  (the ordered role root)
k_j  = Q(x_{j-1})                   learned source descriptor (the same learned map)
r_tj = inverse(q_t) * k_j           directed relative element, common-LEFT invariant
s_tj = bias + rank[r_tj] + sum_e w_e * phi_e(j) + sb[a]
a_t  = NoRead, or (exact occurrence reference j, bounded strength A[a])
```

`Q : V -> 120` is a **learned** soft map (V x 120 logits) hardened to `argmax`; it is initialised
from the frozen S write-slot assignment so that `r` starts as the old eight-class equality and the
learned part is a measurable movement away from it. `rank : 120 -> i32` is a learned scalar table on
the relative element. `phi` are exact-evidence indicators that every comparator also receives, so
broadening the candidate pool does not blind anyone.

**Why this representation.** The eight-class slot equality of PR #1314 is `Q` fixed by the frozen
palette. Nothing in that construction can express "these two role tokens are the same scope" unless
they collide in the palette. A *learned* `Q` can place a same-scope role pair `(u, u')` in a common
left coset, `Q(u) = g_f u`, `Q(u') = g_f u'`, so that
`r = inverse(g_f u) * (g_f u') = inverse(u) * inverse(g_f) * g_f * u' = inverse(u) u'`
becomes **family-independent**, and a single learned `rank` entry generalises across every family.
That is a genuine algebraic generalisation the exact comparator cannot express, and it is the
concrete question this experiment asks. It is a finite quaternion relation, i.e. the reduced
Hopf/spin special case, not a general S7 state.

Order convention is declared: `inverse(q) * k` is invariant under a common **left** change of frame,
`q * inverse(k)` under a common right change. Only the first is used. The relative element is kept
for a later state update rather than collapsed to a scalar.

Exact token, occurrence and payload identity stay outside the descriptor: the payload is always an
observed token read through a validated `(seq, abs)` reference.

## Shared causal candidate pool

Both fitted selectors use **one** pool, so conditional ranking is attributable. Admission keeps a
candidate `j` (bounded to the newest 24 admitted, from a 128-token ring) if it shares **any** exact
context token with the query: `x_j == x_i` or `x_{j-1} == x_{i-1}` or `x_{j-2} == x_{i-2}`, and its
successor `x_{j+1}` is observed (`j + 1 <= i - 1`). Admission never inspects the target. The old
exact-key-only pool is kept as a **support comparison**, and every comparator also receives the exact
indicators (`same_cur`, `same_prev`, `same_prev2`), which the old reader obtained implicitly from
admission. No gold source is inserted.

Scanned records, descriptor updates and candidate visits are counted, not inferred from the final
candidate count.

## Loss-aligned objective

For payload `y`, local probability `p_y`, true next token `x` and boost `a` nats, the exact one-step
change in negative log likelihood is `delta(a) = log(1 + p_y (exp(a) - 1)) - a * 1[y == x]`. Every
admitted `(candidate, strength)` action and the NoRead action therefore has an **observable** cost
from the observed next token alone. The policy is a softmax over
`{NoRead} u {(c, a)}`, and the objective is the expected actual action loss `sum_a pi(a) delta(a)`.

`delta` is computed offline and is a constant per action, so this is a plain differentiable objective
whose gradient is `pi_i (delta_i - sum_j pi_j delta_j)`. No reward model and no reinforcement
learning are needed. Multiple occurrences with the same payload share lexical credit; the label never
identifies a unique source.

**Learned strength, including zero.** The strengths are three declared powers of two
(`amp_shift 6, 10, 13`, i.e. about 0.0625, 1 and 8 nats). Which one to use, if any, is learned
through `sb[a]`; NoRead means strength zero. This replaces the single destructive 16-nat constant and
follows the audit's `D(a) = log(1 + p(exp(a)-1)) - r a` with `a_opt = clip(logit r - logit p, 0, A)`.

**Descriptor learning.** `Q` is trained by enumerating, for each position, the current `argmax` root
plus the top-`K` softmax alternatives (`K = 6`) for the query-role and source-role tokens, and
back-propagating through the softmax over those alternatives with the exact `rank` differences. That
is the standard bounded straight-through estimator for a discrete map; the enumeration is declared
and its coverage reported.

## Local baseline

Frozen E plus S's query row only, with S's absence behaviour and **no history fold**, implemented
independently of the occurrence reader and cross-checked against the retained CPX3 scorer.

## Comparators (same pool, same actions, same dose, same objective)

1. **local** — always NoRead.
2. **exact-recurrence** — the repaired PR #1314-style selector: identical features and actions but
   **no `rank[r]`** and no learned descriptor. Its `same_prev` indicator carries the eight-class
   equality information the old reader had through the frozen palette.
3. **relational** — adds `rank[r]` and the learned `Q`.

A **matched categorical control** replaces `rank[r]` with a learned table on an arbitrary categorical
code whose assignment is *not* derived from any geometry, with targets defined independently of the
model's roots. It distinguishes "richer context helps" from "this group structure helps".

Direction/relative-feature interventions (identity-relation forced, relation order reversed,
descriptor disabled) show use of the mechanism but are not a substitute for the matched comparator.

## Falsifier and decision

Primary falsifier: the learned contextual relation does **not** add useful source-sensitive behaviour
beyond the competently calibrated exact-recurrence comparator on the same pool at the declared
natural-text/cost tradeoff.

Prospective decision, recorded before fresh scoring:

- **Positive** iff, on the held-out construction split, the relational reader's read precision
  exceeds the exact-recurrence comparator's by `>= 0.15` absolute with a paired-by-assignment
  interval excluding zero, **and** its mean actual action loss is lower by `>= 0.01` nats per
  candidate-bearing position, **and** all correctness/causality/identity/staleness controls pass.
- A gain present only on the wider pool, or only at a smaller amplitude, is attributed to that source
  and not to geometric ranking.
- Always-NoRead matching is an acceptable negative, not an attention gain.
- Corrected historical columns (+0.671 synthetic, +2.789 raw at the old 16-nat gain) are reported for
  comparison only; the new tolerances are the ones above.

Natural text is used **only** as a fit/regression panel here, never as a source of fresh evaluation.
