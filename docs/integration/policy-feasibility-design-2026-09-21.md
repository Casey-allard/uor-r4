# Joint finite-policy feasibility — prospective design

> **Principal review completing PR #1332:** the [review](policy-obstruction-review-2026-09-21.md) and [audit](../evidence/policy-obstruction-principal-review-2026-09-21.json) independently certify the development count obstruction. The original generic solver required fixed-objective/tolerance repairs for feasible cases; source correspondence, digest completeness and saved-statistics scope are qualified. Both attempts are sealed; no fresh final was evaluated. The collision is in the coarse five-bit influence observation, not all H4 geometry. The [successor](deepseek-reader-confidence-step-2026-09-21.md) restores the existing parent read advantage with an explicit witness before learning useful dose. Original dated measurements are retained below.

September 21, 2026. Executes the principal review's terminal decision and the
[constructive prompt](deepseek-policy-feasibility-step-2026-09-21.md) on the reviewed parent
`e0f99fcc` (PR #1331). This note records the objective, domains, normalization, constraints, support
handling, solver, selection rule and evaluation populations **that the run was executed against**.
The criteria were taken from the principal review's own declared outcomes and the executed policy
contract before the counterfactual statistics were extracted; the extracted result did not choose
them. The run's own record is the sealed root
`.uor-models/realtext-prior-2026-09-20/policy-feasibility-2`.

## Question

Does **one deterministic action table over the existing causal observations** improve complete-stream
natural-text loss while preserving present-query emission and absence behaviour? This settles joint
feasibility of the current scalar-policy class; it is not another mixture-weight, width or
threshold sweep.

## Frozen inputs

Parent artifacts `relational-learning-4/artifacts/*.rlr2` (SHA256-pinned), E `head-projection-3`
`empirical.cpl2`, S `s-attribution-3` `separable_older_query_read.cpx3`, the pinned tokenizer and
corpus, ring 128, candidate limit 24, the tune-derived gap thresholds and the immutable `PolicyConfig`.
Four actions `{NoRead, 0.0625, 1, 8}` nats (`AMP_SHIFTS [6,10,13]`, `f_bits = 10`). H4 and
categorical source maps/rankers are matched.

## Domains and normalization

- **Objective domain** — natural reader text. The **complete** fit text stream (all positions, empty
  pools included) is the denominator; the objective is total text CE change in bits, reported as
  bits/token. Non-candidate positions contribute 0 for every action.
- **Present domain** — the `final_present` construction stratum of the fit pool.
- **Absent domain** — the `final_absent` construction stratum of the fit pool.
- The **tune** pools (construction `SEED_TUNE`, reader documents `[8,16)`) are held-out development
  checks, not optimization inputs. Fit and tune are document-disjoint.

## Counterfactual sufficient statistics (extracted once)

At every candidate-bearing position the ungated top source and its bucket are taken from the
configured selector, and the **actual integer-path** outcome of all four actions is recorded
(`payload` logit `saturating_add`-boosted, integer argmax tie rule): CE change in bits, emitted token,
correctness and read indicator. The ideal identity
`delta_loss(a) = log(1 + p*(exp(a)-1)) - a*1[payload == target]` is asserted against those logits at
every position; the maximum absolute residual is reported. Statistics are accumulated per
`(bucket, action, stratum)` and per reader document. The frozen parent's realized present/absent
behaviour is accumulated on exactly the same populations.

## Constraints (development fit populations)

For a table `x[b,a] in {0,1}`, `sum_a x[b,a] = 1`:

- `present emitted-correct >= parent − m`, with `m = ceil(2 * P_dev / 121)` — the review's
  "at most two fewer correct answers" translated prospectively to the development present count
  `P_dev`; exact counts and rates are reported;
- `present CE <= parent + 0.05 bits/position`;
- `absent reads <= parent`;
- `absent CE <= parent`.

A text-harm screen of `+0.05 bits/token` and "improves over local" are reported separately from the
constraints; a text optimum above the screen is an obstruction, not a reason to spend a final pass.

## Support handling

The **declared operational class** keeps the fitted rule: buckets with weighted fit support
`>= UTIL_MIN_SUPPORT` carry their own action; sparse/unseen buckets are pinned to the declared global
fallback opcode. A **wider-class diagnostic** on the same saved statistics frees every bucket and is
labelled separately. Failure of the restricted class is not failure of the wider class.

## Solver

A multiple-choice binary program over 32 buckets, solved in Rust by a Lagrangian lower bound
(coordinate ascent) plus a bounded branch-and-bound over the free buckets with sound optimistic
feasibility and objective pruning. Caps: 4,000,000 nodes and 45 s per solve. Status is one of
`OPTIMAL | FEASIBLE_SUBOPTIMAL | INFEASIBLE | UNRESOLVED`. A node/time cap yields `UNRESOLVED`, not
`INFEASIBLE`; `INFEASIBLE` requires an exhausted search. The infra-classical construction uses
floating point offline only.

## Candidate selection and evaluation

Select the feasible incumbent with the lowest development text loss if it satisfies every behavioral
constraint and the declared development text screen; then one frozen fresh evaluation. If a sound
bound or an exhausted search shows the constrained class cannot meet the constraints or the screen,
deliver the obstruction without a final pass. No second final draw.

## Remaining control repair folded into this run

A small **expected-manifest loader** resolves each exported artifact by on-disk byte hash, structural
load, fit-input identity, configured feature contract and vocabulary **before** returning a predictor.
Drifted thresholds, a wrong expected byte hash, a wrong expected fit-input identity and truncated
artifact bytes are each exercised as rejections through that consumer.

## Resources

Projection recorded in the [resource ledger](resource-ledger-2026-09-19.md): one extracted pass over
the frozen development populations, one bounded solve per arm, plus the existing harness; no full
model arm sweep per optimizer iteration. Physical energy remains UNAVAILABLE and whole-path D0-b
compliance is not claimed.
