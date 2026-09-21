# Joint finite-policy feasibility: a proven obstruction, not a mixture-weight question

> **Principal review completing PR #1332:** the [review](policy-obstruction-review-2026-09-21.md) and [audit](../evidence/policy-obstruction-principal-review-2026-09-21.json) independently certify the development count obstruction. The original generic solver required fixed-objective/tolerance repairs for feasible cases; source correspondence, digest completeness and saved-statistics scope are qualified. Both attempts are sealed; no fresh final was evaluated. The collision is in the coarse five-bit influence observation, not all H4 geometry. The [successor](deepseek-reader-confidence-step-2026-09-21.md) restores the existing parent read advantage with an explicit witness before learning useful dose. Original dated measurements are retained below.

September 21, 2026. Executed from reviewed parent `e0f99fcc` (PR #1331). Prospectively declared
[design](policy-feasibility-design-2026-09-21.md); [principal review](policy-objective-review-2026-09-21.md);
[constructive prompt](deepseek-policy-feasibility-step-2026-09-21.md). Delivered root
`.uor-models/realtext-prior-2026-09-20/policy-feasibility-2` (sealed, verified, 0 unlisted, manifest
`a28803c4e8c1e86049293eb74ad1fe35f5117e690db95e67461a7a548ed31fac`). The superseded attempt
`policy-feasibility-1` holds identical measurements and is retained with a valid seal and a superseded reporting defect (its
infeasible-arm reporting printed a non-selected search path); it is never reused as the delivered root.

## Decision

The joint constrained problem is **INFEASIBLE** for both frozen source arms in the declared class
**and** with the fallback relaxation (every bucket free). The search exhausted without hitting its
node or time cap. There is no deterministic per-bucket action table over these observations that keeps
present-query emissions near the parent while keeping absent-query reads at the parent's level.

Per the declared rule, the obstruction is delivered **without a fresh final evaluation**. This ends
the same-feature scalar-policy campaign. The counterfactual statistics and both solves are preserved.

## The measured obstruction

Development fit populations (construction `final_present` 197 positions, `final_absent` 23;
complete fit text stream 1,952 tokens, 944 candidate-bearing). Parent reference `relational_ctx` on
exactly those populations: **present emitted-correct 149/197**, present CE change **−1931.32 bits**,
**absent reads 7/23**, absent CE change **+3.497 bits**. Translated preservation margin
`m = ceil(2·197/121) = 4` correct answers; present loss margin `0.05·197 = 9.85` bits.

| Constraint | Required | H4 best attainable (any table, buckets free) | Binding? |
| --- | ---: | ---: | --- |
| present emitted-correct | >= 145 | 149 | no (alone) |
| present CE change (bits) | <= −1921.47 | −1937.55 | no (alone) |
| absent reads | <= 7 | 0 | no (alone) |
| absent CE change (bits) | <= +3.50 | 0.00 | no (alone) |

Each constraint is reachable on its own; their **conjunction is not**. The decisive interaction is
between the first and third rows. An exact 0/1 knapsack over the saved statistics
(choose which buckets may read, `absent reads <= 7`, maximize present emitted-correct) gives:

| Arm | Max present emitted-correct under the absent budget | Required | Outcome |
| --- | ---: | ---: | --- |
| `h4_policy` | **74** | 145 | infeasible |
| `categorical_policy` | **71** | 145 | infeasible |

The two Rust solves agree: `h4_policy` `INFEASIBLE` (exhaustive, 57 nodes), `categorical_policy`
`INFEASIBLE` (exhaustive, 421 nodes); the fallback-relaxed wider class is also `INFEASIBLE`.

## Why it is infeasible: one bucket, conflicting per-position decisions

The absent queries are not isolated from the present ones. Their coarse buckets coincide:

| Bucket | 0 | 1 | 5 | 8 | 9 | 13 | 17 | 20 | 24 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| absent positions | 3 | 6 | 1 | 1 | 5 | 1 | 3 | 1 | 2 |

Buckets 1 and 9 alone carry 11 of the 23 absent positions, yet the present-query emissions that must
be preserved depend on reads inside that same bucket set. A single action per bucket cannot read the
present positions while not reading the absent positions it shares a bucket with. The parent reaches
149 present-correct with only 7 absent reads precisely because its scored path makes a **per-position**
decision, not a per-bucket one — it has information (absolute source/NoRead confidence and the
directed relation) that the five-bit bucket observation discards.

The fitted tables show the same failure directly on the development population: their realized absent
reads are **23/23** (they read every absent position), versus the parent's 7. This is the mechanism
consistent with the previous exposed final panel's 19/19 absent reads against the parent's 5. The mixed objective selected that harmful policy, but objective changes alone cannot repair the frozen class: this is an information collision in the coarse influence representation. Freeing unsupported buckets leaves the obstruction; this does not establish an H4 capacity limitation.

## Text is not the blocker

The text objective alone is satisfiable and slightly favourable: the per-bucket text optimum is
**−0.0366** (`h4`) / **−0.0349** (`categorical`) bits/token, and the constant one-nat comparator gives
−0.030979 / −0.030671, both improving on local. The pair of present-emission and absent-read constraints already makes the class
infeasible without text. This does not prove joint text/present feasibility. The scalar objective omitted preservation constraints, while the observation interface also discards the information needed to satisfy them.

## Verified mechanics

- **Counterfactual identity.** The ideal score-space identity is asserted against the actual integer
  logits at every one of **5,442** extracted positions; maximum absolute residual **1.42e-13 bits**.
  All four actions are computed per position from the real integer path (integer argmax tie rule,
  `saturating_add` boost), then aggregated into retained fit bucket/action matrices. Individual source/occurrence rows, tune matrices and per-document counterfactual arrays were not saved. The principal repair uses effective rather than nominal boost at saturation; original non-saturated measurements are retained.
- **Control repair delivered.** An expected-manifest loader now verifies the on-disk byte hash, the
  fit-input identity, the configured feature contract and the vocabulary before returning a predictor.
  `expected_manifest_loader` records **0 failures**, and four rejections are exercised through that
  consumer: drifted gap thresholds, a wrong expected byte hash, a wrong expected fit-input identity,
  and truncated artifact bytes. `reload_failures = 0`.
- **Comparators unchanged.** The final-panel numbers reproduce PR #1331 exactly
  (`h4_policy` +0.1193 bits/token, present 56 vs parent 77, absent 19 vs 5, one-nat −0.0148), so the
  new statistics pass did not disturb the shared path.

## The smallest evidenced missing distinction

The obstruction is a **within-bucket separability** failure: the current observation cannot tell a
position where a read emits the target from a position where the same read is an absent-query
misfire. Per the review's own ordering, the successor is therefore *not* persistent state and *not*
more dimensions. It is to **inspect the omitted existing causal information first** — the absolute
selected-source / NoRead confidence, the directed relation identity, or the local normalization that
distinguishes those positions — and admit it into the bucket or the decision as one bounded
operation, with the same matched comparator discipline.

Everything else on the roadmap is unchanged: structural correspondence/persistence, dependent reads
and derived output, broader prose, executed Rust, then qualified scale and energy remain the ordered
responsibilities in [project-track](project-track.md).

## Resources and scope

Charges, projection and the recorded extension are in the
[resource ledger](resource-ledger-2026-09-19.md). This is a **development feasibility** result, not
generalization. Teacher-forced additivity does not extend to rollout. Physical energy is UNAVAILABLE
and whole-path D0-b compliance is not claimed; the new code is offline analysis plus the same served
integer selection path. No second final draw was taken.
