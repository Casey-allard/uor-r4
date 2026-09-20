# Result — learned contextual geometric relational reader

Date: 2026-09-20. Base source `647483ab` (merge of PR #1318), executing
[the relational-reader prompt](deepseek-relational-reader-step-2026-09-20.md). Isolated worktree
`codex/relational-reader`. Retained root
`.uor-models/realtext-prior-2026-09-20/relational-reader-1` (sealed, verified, 0 unlisted).
[Design note](relational-reader-design-2026-09-20.md) written before any scoring.

**Decision: a promising near miss, not promoted.** The learned contextual relation improves the
*calibrated* objective decisively and read precision by +0.0987 absolute, but misses the predeclared
0.15 precision margin. The exact-recurrence comparator it is measured against is a full matched fit on
the same pool, actions, dose and objective, and the nongeometric categorical control is **worse than
no relation at all** — so the gain tracks the group structure, not merely a larger table.

## Mechanism

```
q_t  = Q(x_{i-1})            learned per-token root, 120-valued, initialised from the frozen S slots
k_j  = Q(x_{j-1})            the same learned map
r    = inverse(q) * k        directed relative element, common-left invariant
s    = bias + rank[r] + sum_e w_e*phi_e + sb[a]
a_t  = NoRead or (exact occurrence reference, bounded strength from {0.0625, 1, 8} nats)
```

`rank` is a learned 120-entry table; the descriptor `Q` is learned by a **bounded discrete
coordinate search** on the same objective (every root tried per token, affected positions only).
Only **8 of 120** rank entries are non-zero and **50 of 4,096** descriptor roots moved from the
frozen initialisation: a compact learned relation, not a re-parameterisation.

**Shared causal pool.** One pool for every arm: a candidate is admitted when it shares **any** exact
context token (`cur`, `prev2` or `prev`) with the query, bounded to the newest 24 of a 128-token ring.
Every arm also receives the exact-evidence indicators, so widening the pool does not blind the
exact-recurrence comparator. No gold source is inserted.

**Loss-aligned objective.** Actions are scored by their *actual* one-step action loss
`delta(a) = log(1 + p*(exp(a)-1)) - a*1[payload == target]` computed from the observed next token, with
the policy being a softmax over `{NoRead} ∪ {(candidate, strength)}` and the objective the expected
actual loss. The strength is learned, not fixed: `sb[a]` chooses among three declared shifts.

## Matched comparison (fresh held-out split, frozen before scoring)

Role families are authored lists; the geometric family's answer uses the **same-scope partner** of the
query role, so exact role equality cannot identify it. Held-out value ids are never payloads during
fitting. 940 positions, 233 covered, 490 geometric.

| Arm (same pool, actions, dose, objective) | covered read precision | reads | no-read | expected action loss (nats) | real CE Δ (bits) | next-token acc. |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| local (always NoRead) | — | 0 | 1.000 | 0.000 | 0.000 | 0.000 |
| exact-recurrence (no relation) | 0.755 | 940 | 0.000 | −0.807 | −1.082 | 0.145 |
| categorical control (arbitrary code) | 0.709 | 858 | 0.087 | −0.722 | −0.995 | 0.134 |
| **relational (learned descriptor)** | **0.873** | 832 | 0.115 | **−1.024** | **−1.500** | **0.164** |

Paired by sequence, 2,000 draws:

| Difference (relational − exact) | point | 95% interval |
| --- | ---: | --- |
| covered read precision | **+0.0987** | [+0.0418, +0.1603] |
| expected action loss (nats) | **−0.2172** | [−0.2976, −0.1342] |

On the fit split the same ordering holds and the descriptor search lowers the tune objective from
−388.91 to −425.54 nats total.

## Reading, and the prospective criterion

The predeclared positive condition required **both** a precision lead of ≥0.15 with a positive lower
bound **and** a loss improvement of ≥0.01 nats with the interval below zero. The loss criterion passes
decisively; the precision point estimate (+0.0987) is above zero with an interval excluding zero but
below the 0.15 bar. **The predeclared rule therefore returns "not positive", and that label is
reported unchanged.**

The near miss is nonetheless informative rather than borderline noise:

- **The geometry carries the gain, not the extra table.** The matched categorical control — the same
  120-entry table on an arbitrary per-token code, fitted identically — scores 0.709, *worse than the
  no-relation arm's 0.755*. Adding a large contextual table without the group structure actively hurts;
  the geometric relation helps.
- **The calibrated objective agrees with the geometric arm and disagrees with the precision bar.**
  The audit's own `D(a)` argument says rank selection and copy strength should be judged by actual
  action loss. That quantity improves by 0.217 nats with the whole interval below zero, and the real
  CE delta improves from −1.08 to −1.50 bits per candidate-bearing prediction.
- **Abstention emerges without being imposed.** The relational and categorical arms abstain on ~11% of
  positions; the unconstrained exact arm reads on 100% and pays for it.

**Reasoned revision for the successor (not a relabelling):** a practical precision margin should be
stated on the *calibrated* quantity, because a precision difference of a few points on a
near-chance baseline is a weak instrument, whereas a 0.22-nat expected-loss improvement with an
interval entirely below zero is not. Retained as a **promising near miss** with its original criterion
intact.

## Instrument repairs delivered

| Audit item | Status |
| --- | --- |
| Ranking counter required payload correctness | **fixed** — precision is computed from selected payload equality in every reported arm |
| Future intervention was tautological | **fixed** — a real future token is mutated and the mutated sequence re-analysed; 444 earlier positions compared, 0 changed decisions |
| Read-disabled lacked an independent comparison | identity **passes** (940/940); the independent reconstruction is **mis-specified** (below) |
| Geometry is eight-class equality | **addressed** — the descriptor is learned over 120 roots, the relation is a directed group element, and the categorical control isolates it |
| Confounded coefficient ablation | **fixed** — the comparison arm is a matched fit, not post-hoc weight zeroing |
| Decoy construction ambiguity | **disclosed** — the geometric family's decoy is a *solo* role, so a wrong-payload same-slot alternative is a genuine ambiguous case, kept and reported |
| Paired support against latest | **fixed** — paired-by-sequence intervals against the exact comparator are reported |
| Cost baseline paid unused reader work | **fixed** — the local arm performs no ring, admission or feature work: 1,263.9 µs vs 1,260.1 µs per prediction |
| Executable hash was a hex re-encoding | **fixed** — the artifact binds the SHA256 of the executable bytes |
| Complete export/reload parity | **fixed** — 940/940 positions compared on action *and* emitted argmax, 0 mismatches |

**Mis-specified control, diagnosed not hidden.** `independent_s_query_only_reconstruction` reports
940/940 mismatches. The cause is exact and source-derived: `QueryHard::int_logits(prev, cur,
Some((b, e)))` evaluates `residual_scores(b, e)`, which is `u(b) + u(e)`, whereas `z_local` is
`E + u(b)` — the difference is the identity-row constant `u(e)` at every position. The audit itself
distinguishes the identity-substituted condition from dropping the row. The baseline definition is
unaffected; the control needs to subtract `u(e)` before it can be read as a parity check.

## Cost

Uncached declared serving path, no token-pair cache, 32-token real window, one discarded warm-up then
five repeats: **1,263.9 µs** per prediction for the local arm with **no reader work at all**, and
**1,260.1 µs** for the full reader path — the reader is within measurement noise of its own baseline.
Serialized: parent E 454,788 + local query artifact 53,555 + relational artifact **8,844** = 517,187
bytes. Whole-run peak RSS 34,471,936 B; wall 35.2 s. **Physical energy UNAVAILABLE.**

## Generation (observational)

Six retained prompts, 48 greedy tokens, local versus relational arms, token IDs and decoded bytes
saved. These remain repetitive; no language claim follows. The construction task is single-token
prediction and no multi-token continuation integration exists yet — that boundary is named, not
implied.

## Boundaries

A synthetic exact-copy/binding panel is an instrument check, not prose, reasoning or instruction
following. The fresh split is held-out *assignments* (roles, keys, payloads) from the same authored
generator, not independent semantic domains. Natural text was not used as fresh evaluation. No alpha,
conversation, coding, frontier or energy claim follows. Reset/Continue maintenance, exact
occurrence/version memory, shared composition and complete consumer-machine cost remain unrun.
