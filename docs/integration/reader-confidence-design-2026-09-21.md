# Read-confidence influence interface — prospective design

> **Principal correction completing PR #1333:** [review](reader-confidence-review-2026-09-21.md) and [audit](../evidence/reader-confidence-principal-review-2026-09-21.json). Fit expressivity is retained, but both confidence arms fail fresh own-parent present CE and absent-read criteria; categorical parent is 81 present/six absent reads, not H4's 73/ten. Confidence rollout/interventions/timing are NOT_RUN. Three attempts repeat one exposed population; current source hashes verify. The exclusive stream-mixing diagnosis is unproven and the [next prompt](deepseek-read-conditioned-state-step-2026-09-21.md) tests a read-conditioned update/shared emission operation. Original dated design/measurements retain their historical scope.

September 21, 2026. Executes the [principal review](policy-obstruction-review-2026-09-21.md) and the
[constructive prompt](deepseek-reader-confidence-step-2026-09-21.md) on reviewed parent `e0d6296c`.
Recorded **before** the extraction, fit and any fresh outcome. Base revision and the source/executable
hashes are recorded in the run receipt; no stale base revision is asserted.

## Question

The coarse 32-address influence table discarded the retained scored reader's own Read/NoRead decision
and is jointly infeasible. Does **restoring that decision as an extra causal address bit** yield a
compact learned influence operator that improves natural-text loss while preserving the parent's
useful answers and absence behaviour?

## Observation semantics (target-free, label-free, future-free)

- Address `= utility_bucket * 2 + 1[D > 0]`, at most **64** entries, same four actions
  `{NoRead, 0.0625, 1, 8}` nats. `utility_bucket` is the existing five-bit partition under the
  configured `PolicyConfig`.
- `D = max_strength strength_score(candidate, relation, strength, parent_bucket) - noread_score(parent_bucket)`
  in widened `i64`. `parent_bucket` is the selector's own `bucket_of` (exact-context class + newest bit
  + its own single-candidate margin convention), never the utility bucket. `D` is a learned score
  difference, not a calibrated probability or log-odds.
- Exact sequence/absolute occurrence, payload position, version and relation encoding are preserved.

## Parent-preserving witness

`empty pool -> NoRead; otherwise the ungated top source at eight nats iff D > 0`. For the pinned H4
pair the retained `relational` and `relational_ctx` artifacts share roots, source weights/ranks, bias,
strength biases and NoRead intercept, and all contextual rows uniquely maximise the eight-nat strength
with a candidate-independent offset, so the top source agrees with `ungated_top_source`. Exact
source/action/strength parity against the independently loaded parent is **verified in-run** on causal
development prefixes (empty pool, one candidate, many candidates, ties) before any claim. The witness
table is an explicit candidate and the unsupported-entry fallback.

## Partitions

Fit (construction `SEED_FIT`, reader documents `[0,8)`) drives extraction and optimization; the tune
pools are held-out development checks; previously exposed final populations are development history.
The **fresh** evaluation uses a prospectively declared construction seed and the remaining eligible
`Dev` reader documents only, disclosed as a small honest final population. No recycled document is
called fresh.

## Objective, constraints and margins

Minimize complete-stream development natural-text loss (bits/token on the complete fit text stream)
subject to: present emitted-correct >= parent − `ceil(2·P_dev/121)`; present CE <= parent + 0.05
bits/position; absent reads <= parent; absent CE <= parent. The text screen is +0.05 bits/token and a
positive influence claim additionally needs an actual useful text gain, reported separately.

## Support, fallback and solver

Per-address support is the count of candidate-bearing fit positions; addresses below
`UTIL_MIN_SUPPORT` (20) take the witnessed parent rule. The corrected `solve_feasible` (Lagrangian
bound + bounded branch-and-bound, 45 s / 6,000,000-node cap) supplies an incumbent; the witnessed
parent rule is additionally evaluated as an explicit candidate, and the feasible candidate with the
lowest development text loss is selected. `UNRESOLVED` is not `INFEASIBLE`.

## Evaluation

The selected operator is exported as an artifact, resolved through the expected-manifest loader
(bytes, fit-input identity, feature contract, vocabulary) and evaluated once on the fresh
construction seed and remaining documents through the shared inference boundary. H4 and categorical
receive the same address capacity, actions and allowance; because the categorical source has no fitted
contextual interaction table, any cross-arm difference conflates geometry with that controller and no
geometry-advantage claim is made.

## Resources

Projection and extension recorded in the [resource ledger](resource-ledger-2026-09-19.md) before
execution: one extraction pass, two bounded solves, one bounded fresh evaluation, focused tests. No
paid/external compute. Physical energy remains UNAVAILABLE and whole-path D0-b is not claimed.
