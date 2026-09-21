# Geometric computation: improved association and corrected composition diagnosis

**Principal correction:** read the [review](geometric-computation-review-2026-09-21.md) and [independent audit](../evidence/geometric-computation-principal-review-2026-09-21.json). Numerical tables retain the original runs. Zero feature aliases do not establish linear impossibility; the reduced composition rule and split were invalid for the claimed witness/transfer. Source corrections are unrun as a model experiment.

September 21, 2026. Executes the [geometric-computation brief](deepseek-geometric-computation-step-2026-09-21.md) on
reviewed `origin/main` at `036e9c4310ca3d8d2440ac367c577a1df7edf58f` (PR #1336, verified equal to the
merged head). Isolation worktree `.worktrees/geometric-computation-step`; owner checkout untouched.

This run **executes the principal's repaired learner end to end for the first time** (the review states
`No new complete model fit ... is claimed by these source repairs`), adds the **served-feature
collision diagnostic** the review required, and builds and runs a **relation-composition instrument**
for the successor capability. All populations are the already-exposed regression seeds; **no fresh
final evaluation is claimed**.

## 1. Corrected lifecycle executes and improves the deployed predictor (regression)

`--mode=contextual-emission`, roots `geometric-computation-1` and (with the new diagnostic)
`geometric-computation-2`, both sealed with 0 unlisted files.

Identical results in both runs, so the corrected path is deterministic:

| Arm | dev /180 | tune /120 | final /120 | dev pairs both /90 | final pairs both /60 |
| --- | ---: | ---: | ---: | ---: | ---: |
| local / NoRead / UpdateDisabled / ReadDisabled / scalar-copy | 0 | 0 | 0 | 0 | 0 |
| development constant | 32 | 13 | 15 | 0 | 0 |
| H4 read-conditioned (**corrected learner**) | **61** | **41** | **33** | **12** | **7** |
| matched cyclic C120 | 54 | 25 | 21 | 8 | 2 |
| categorical selected-value table | 164 | 112 | 108 | 81 | 54 |

The corrected learner is better than the pre-repair report on the same exposed populations
(H4 50/36/25 and 1/60 pairs, C120 42/25/20 and 4/60). The declared screen still **fails**:
H4 7/60 final pairs against the 54/60 table. Reader localisation is 163/180 dev, 111/120 tune,
108/120 final correct exact occurrence. The frozen-row ceiling is still 0/180.

**One committed served objective improves at every stage** (calibrated fixed-point bits):

| Stage | H4 | C120 |
| --- | ---: | ---: |
| initial | 19.8629 | 19.8629 |
| served output, before map search | 10.8529 | 10.2985 |
| served after map search | 9.3995 | 9.2976 |
| post-map refit incumbent (must equal the previous row) | 9.3995 | 9.2976 |
| served final | **7.8697** | **8.2947** |

`post_map_output_refit_executed = true`, the two output hashes differ, serving uses 64 rows with 691
nonzero ternary coefficients, 1080 accepted ternary flips (592 pre-map, 488 post-map) and 7 accepted
map changes. The pre-map to post-map refit contract held on both arms, and independent reload parity
held for every position. `distinct_payload_alias_pairs` is 0 before and after, so the collision-aware
map objective did not buy margin by merging required value distinctions.

Both loaded artifacts are exercised after reload; three loaded three-token continuations ran with
1/3 correct first answers and degenerate text (`info-f`, `-review`, `user-s`). Useful prose remains
unqualified.

## 2. The served feature does **not** alias on the association instrument

The review's unmet diagnostic was: *do source-correct examples with different targets collide in the
actual feature `R(q0*T[r]*V[v]) - R(q0)`?* Measured on the deployed residual:

| Split | source-correct positions | distinct service features | features with conflicting targets | table-on-feature ceiling |
| --- | ---: | ---: | ---: | ---: |
| dev | 163 | 50 | **0** | **163 / 163 (100 %)** |
| final | 108 | 46 | **0** | **108 / 108 (100 %)** |

A table refitted separately on each split's labels can label these observed source-correct features. This does not prove that a development-fitted decoder transfers, nor that a linear readout is infeasible. The complete residual predictor also receives example-dependent local logits. Optimization, coefficient/support constraints and output interface remain possible causes of the gap. The independent audit verifies receipt consistency, but the old association rows omit q1/full features for independent histogram reconstruction.

`distinct_update_signatures` finds 8/6 ambiguous update signatures affecting **30/180 development and 23/120 final positions**. The original 21/17 summed distinct labels, not positions. These are equal update inputs; the whole predictor may still differ through local logits.

## 3. Relation composition: instrument, obstruction and negative

New mode `--mode=relation-composition` computes labels from query operation and retrieved content. The submitted reduced mode executed `(op+vi) mod 7`, while its tests and report cited an order-10 power witness. At (1,6), the witness gives h^7 rather than the required identity class. It also withholds values 6/7 under all operations, leaving their selected-value meanings untrained. These are not held-out combinations of familiar primitives. The repaired future fixture uses witnessed modulo 10 and balanced operation×value cells; it has not been fitted in this review.

**Obstruction (measured).** The served geometric relation interface exposes only **2 distinct directed
relative elements** across all 14 declared role pairs (`mode = Geometric`, descriptor roots are rich —
119, 113, 82, 0, 77, 9, 78, 84, ... — but `inverse(root_q)*root_k` collapses to {80, 85}). Eight distinct operation labels cannot all be separately encoded through these two `T[rel]` inputs alone; the local predictor still receives query-role information. The instrument the instrument
records `relation_budget_saturating = true` and the full relation-exposure table rather than
asserting a wider budget.

At the reduced two-operation budget (class modulus 7, 4 repeated contexts per cell):

| Arm | dev /48 | held-out /16 |
| --- | ---: | ---: |
| local / NoRead, UpdateDisabled, ReadDisabled | 0 | 0 |
| H4 composition | 15 | **0** |
| matched cyclic C120 | 15 | 0 |
| development constant | 8 | 4 |
| payload-only table (blind to the operation) | 24 | 4 |
| relation+payload two-input table | 48 | 4 |

No held-out state was reached in development. The saved feature-only oracle is 42/46 development and 16/16 held-out, fitted separately to those labels. Zero held-out features overlap development; a development-fitted feature table reaches 4/16 by fallback. The 0/16 model result on this malformed task is retained, without a linear-infeasibility or compositional-transfer conclusion.

The earlier eight-operation construction retains a narrower observed collision result: 44 source-correct positions produce 10 distinct features with a same-split feature-majority score of 12. Its total arm hit count also equals 12, but missing per-position outputs prevent establishing that these are the same successful positions or that the complete predictor is at a ceiling.

## 4. Corrected interpretation and next task

The completed corrected fit and improved deployed association are useful. No general linear readout impossibility is established. A learned result decoder is a constructive candidate because lexicalization need not share the old difference-of-features interface. Use the actual relative computed state when the frame is nuisance, preserve meaningful context, and distinguish a valid identity result from NoRead. Separate observed-query operation from source-compatibility relation when more operation distinctions are needed. Test combinations of grounded primitive operands with an exact witness and actual causal generation controls. See the [next research brief](deepseek-derived-state-decoder-step-2026-09-21.md).

## Exclusions and scope

Bounded authored instruments, not general language or reasoning. All populations are exposed
regression seeds. The composition split and witness do not support the originally claimed familiar-primitive transfer test. A future valid pass would be a composition-circuit result, not a language-advantage claim. The
composition-arm negative is at 48 development / 16 held-out positions and is exploratory. Energy
`UNAVAILABLE`; whole-path D0-b not claimed. Sealed roots
`.uor-models/realtext-prior-2026-09-20/{geometric-computation-1,geometric-computation-2,relation-composition-1,relation-composition-5}`
(0 unlisted each); the empty claim directories `relation-composition-{diag,diag2,3,4}` hold no evidence.
Resources and charges: [resource ledger](resource-ledger-2026-09-19.md).
