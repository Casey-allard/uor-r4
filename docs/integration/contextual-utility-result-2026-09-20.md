# Learned contextual utility controller: global strength repaired, ranking still open

Executed September 20–21, 2026 from base revision `89ee803b` (PR #1322 merged). Prospective design and
frozen decision criteria: [contextual-utility-design-2026-09-20.md](contextual-utility-design-2026-09-20.md),
recorded before any held-out result. Machine-readable receipt:
[../evidence/native_geometric_contextual_utility_2026-09-20.txt](../evidence/native_geometric_contextual_utility_2026-09-20.txt).
Retained report root: `.uor-models/realtext-prior-2026-09-20/contextual-utility-2` (claimed, sealed,
verified — 0 unlisted files). Superseded attempt `contextual-utility-1` is retained, never resealed.

This is a development result on a repaired constructed population plus an inspected natural-text
regression panel. It is **not** a promotion, **not** fresh external language evidence, and **not** a
claim that attention, ranking or the geometric relation are solved.

## What was built

The served selector used to factorise `score(c, a) = bias + rank[rel(c)] + sb[a] + w·feats(c)`: one
positive strength bias per action, identical at every position and for every source. With three finite
strengths (0.0625, 1.0, 8.0 nats) the strongest won wherever a read was taken.

The new mechanism adds a **bounded causal action-dependent interaction** over the same finite action
set, with NoRead inside the interaction:

```text
A_ctx  = { NoRead } ∪ { a_0, a_1, a_2 }                       (unchanged actions)
bucket = f3 | (f1|f2) << 1 | f5 << 2 | (best > second) << 3   (16 frozen causal buckets)
score(c, a | b) = bias + rank[rel(c)] + sb[a] + w·feats(c) + ctx[b][a]     (a > 0)
noread(b)       = noread + ctx[b][0]
```

Every `ctx` entry is a signed 4-bit integer added by the same integer kernel that already adds `sb[a]`;
no multiplier, float or transcendental enters the served selector. The bucket is computed **once** from
the frozen ctx-free source scores and the admitted candidates' exact-context flags — never from the
target, coverage, labels, family ids or future tokens.

Fit protocol (exactly as preregistered): fit the existing trainer → quantize → **freeze** the integer
selector → fit only `ctx` by bounded discrete coordinate search on the same served objective, starting
from zero. `ctx = 0` reproduces the factorised baseline, which is verified **behaviourally** (an
all-zero table chooses identically on all 2,969 fit and 1,813 fresh positions), not by struct equality.
The artifact version moves to `RLR2` v3 carrying the table; v2 remains readable, confirmed against the
real retained `competitive-reader-1` artifact.

## Results

Construction (repaired fixture from PR #1322; its quality was previously `NOT_RUN`): 2,969 fit and
1,813 fresh candidate-bearing positions, 646 covered fresh positions, 21 absent queries.

| Arm | Fresh hard-action CE, bits/candidate position |
| --- | ---: |
| local (NoRead everywhere) | 12.6845 |
| exact recurrence | 10.7286 |
| categorical learned code map | **10.3936** |
| relational H4, global strength | 10.7185 |
| relational, relation-channel lesion | 10.7003 |
| **relational + contextual utility** | **10.4608** |

**Primary prospective comparison** — contextual minus frozen global strength, paired by sequence:
**−0.257644 bits/candidate position, sequence-cluster 95% interval [−0.327088, −0.182968]** over 140
sequences and 2,000 draws. The whole interval is below zero and the point exceeds the preregistered
0.05 margin, so `positive = true` under all five declared conditions: instrument checks pass; the arm
keeps reading (read rate 0.532 vs global 0.365); the natural-text tradeoff improves rather than worsens
(+0.2189 vs +0.2443 bits/token); emitted accuracy does not fall (0.1765 vs 0.1561); and the artifact
reloads exactly.

Five of the sixteen buckets concentrate the effect, and the learned table is interpretable: the
8-nat action is chosen only when the candidate is the most recent **and** agrees on the previous token
(bucket `0110`, `[−7,−7,−7,+7]`); the middle action when the candidate is recent
(`[−7,−7,+7,−7]`); the weakest action otherwise. The controller reads *more often* (965 vs 661 reads)
and still lowers hard-action loss, which is the opposite of an all-NoRead "improvement".

## The important negatives, stated plainly

1. **The H4 relation term does not reproduce its old advantage.** Relational minus exact on the
   repaired fresh panel is **−0.010151 bits, interval [−0.104470, +0.083675]** — spanning zero. The
   retired aggregate −0.179540 was measured on the construction whose partner/absence defects the
   principal review found. On a construction with genuine partner sources and genuine absence, the
   directed relative element is **not** significantly better than the exact-recurrence comparator.
2. **The nongeometric control is now the strongest static arm.** With the repaired acceptance branch
   the categorical code map finally moves (149 assignments, 44,268 evaluations, objective −4485.84 →
   −4973.43) and reaches 10.3936 bits — better than the geometric arm (10.7185) and better than the
   contextual arm (10.4608). Zeroing the relation channel (10.7003) also fails to hurt. So on this
   evidence the geometric relation carries no demonstrated unique benefit; the *useful* component is
   the learned contextual code/utility, not the H4 relative element. That is consistent with the
   review's warning and must not be re-narrated as unique-H4 superiority.
3. **Ranking, not strength, is now the dominant remaining loss.** The target-using opportunity bound
   decomposes the residual headroom: mean strength loss **0.0933** nats versus mean
   ranking/admission loss **1.1939** nats (≈12.8×). Where the correct payload is admitted (646
   positions), the relation channel already ranks it top at only **46.3 %** and the arm selects it at
   only **25.2 %** — while the best finite action belongs to that correct source at **100 %** of those
   positions. The strength controller did its job; it cannot repair selection.
4. **Natural text is still harmed in absolute terms.** +0.2189 bits/token on the inspected development
   panel, above the old 0.05 tolerance, though better than the global-strength arm. Note this panel is
   *inside* the declared text/construction fit mixture, so it is development regression evidence, and
   the old +1.2908 came from a construction-only fit on the old construction — the reduction is
   therefore not a like-for-like comparison.
5. **Generation remains degenerate.** The contextual arm reduces adjacent-token repetition on several
   prompts (e.g. 35→4, 38→0, 36→0) but no arm emits fluent prose. A teacher-forced CE gain is not
   general language capability.
6. **The declared active-candidate serving cost is still UNAVAILABLE.** The 32-token cost probe
   admitted **zero** candidates, so it measured the local path (≈1.37–1.41 ms/prediction, release) and
   not the reader path. Whole-path D0-b compliance is not claimed; only the served selector arithmetic
   is bounded to integer add/compare/table reads over signed 4-bit entries. The artifact grows by 64
   serialized bytes (4,408 vs 4,344) and 64 resident bytes.

## Instrument integrity

All controls passed: one shared `read_step`/`predict_next` path (203 positions, identical scores and
actions); independent S query-only reconstruction (0 mismatches / 1,813); future-token intervention
with invariance required at every candidate-bearing predecessor **including** the one immediately
before the changed token (876 compared, 131 of them that predecessor); stale reference after reset;
per-arm reload parity. The first attempt (`contextual-utility-1`) is preserved but superseded: its only
defect was a self-check that compared selectors by struct equality, so the empty and all-zero tables
were called unequal although they act identically. With that repaired the measurement is bit-identical
(10.460830 / 10.718474 / [−0.327088, −0.182968]), which is itself a reproducibility check on the
harness and the fit.

## Decision

Preregistered `positive` for the *strength* question: **true**. Preregistered falsifier: the residual
headroom is dominated by ranking loss, so the controller is **not sufficient**, though it is not the
wrong repair — it produced a significant, margin-meeting paired gain. Per the frozen rule this is
reported rather than relabelled, the working controller and interface are retained, and the smallest
missing observation is stated instead of sweeping thresholds: the competing sources at a query share
the query key and differ only in the preceding role token, so every admitted candidate has the same
exact-feature pattern and the same admission class. The only separating signal is the *partner
relation*, which a candidate-level exact-feature bucket cannot express. That makes a source-term
capacity or representation change a **new prospective experiment**, not a bounded tweak; no such
campaign was started here.

## Next step

One justified successor: a **prospective, matched-cost comparison of the source term at higher
resolution on a fresh construction draw** — the single shared scalar `rank[rel]` over the 120-root
descriptor versus a learned directed compatibility indexed at the `(query role, source role)` pair,
with the contextual utility controller frozen on both sides so the difference is attributable to
selection alone. Freeze the acceptance criteria and the draw before scoring, keep the categorical
comparator and the exact-recurrence comparator, and judge on selection success at covered positions
(the 46.3 % top-rank / 25.2 % selected gap measured here) as well as hard-action CE.
