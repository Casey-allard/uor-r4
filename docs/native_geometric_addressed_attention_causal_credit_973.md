# Addressed attention: causal credit comparison — #973

**SELECTED_CAUSAL_CREDIT_FOR_BOUNDED_LEARNING.** September13,2026. The frozen engineering selection gate passes: aggregate gate-gradient covariance falls47.4%, with no gate-cell increase and lower aggregate covariance in every other parameter family. All32 batches reproduce the retained paths, counts and aggregate losses. This selects a lower-variance estimator for a subsequent bounded learning test; **the prior model gate remains FAIL_ADDRESSED_ATTENTION_LEARNING_PILOT**. Retain15baec48; no parameter updates, new fit or promotion.

[Source-bound evidence](evidence/native_geometric_addressed_attention_causal_credit_973.json) binds the33 source/contract files, executed release binary, design, logs and complete40-file sealed attempt. The [prior stability diagnostic](native_geometric_addressed_attention_stability_973.md), [attention specification](native_geometric_addressed_attention_spec_973.md) and [UOR/Prism/NEMESIS/Spiralcore research](native_geometric_attention_reassessment_973.md) remain preserved at their exact scope. The knowledge map supplied the prior execution record; no broad research audit was repeated.

## Implemented change

The Rust [causal comparator](../crates/uor-r4-core/src/native_geometric/addressed_attention/causal_credit.rs) samples the original complete forward/observe trajectories and records sparse parameter-score events at each CE boundary. It computes both the old complete-trajectory SerialLoo gradient and causal cost-to-go gradient from these same four paths. Probabilities, RNG identities, legal masks, direct CE, normalization, exact memory, geometry and runtime decisions are unchanged. Existing learner, policy, artifact and serving source files remain byte-identical; only module registration and new diagnostic source are added.

An event before CE_t receives the normalized suffix beginning at t. The offered-symbol draw and subsequent OBSERVE/KEY events receive the suffix beginning at t+1. Each baseline averages the other three independent particles' suffixes at that same target boundary, then the four particle contributions are averaged. The mean direct CE derivative is unchanged. Final post-CE events receive zero future cost. Event ordinals do not align baselines across paths; actual target-loss boundaries do.

This implements the downstream-cost option already specified from [Schulman et al., Gradient Estimation Using Stochastic Computation Graphs](https://arxiv.org/html/1506.05254v3). Its accounting supports the declared expectation; it does not guarantee a finite-sample variance reduction or language improvement. Sparse event storage avoids a dense parameter vector at every position or a second runtime replay. The full serving boundary remains bounded integer/table geometric execution with no mathematical matrix products or transformer backbone.

## Frozen comparison and actual result

Two fixed construction records (train/memory-a/0,train/sub-9-2/0), initial and64-update checkpoints, eight four-particle replicates per cell: **32 batches /128 trajectories**. Seed1973 and document identity record_slot*8+replicate match the prior diagnostic. Neither checkpoint changes. No new corpus, temperature, learning rate or parameter selection is introduced.

All old/new serialized particle traces and event counts match exactly. Whole/prompt/response losses match the old sealed reports within the predeclared2e-14 JSON numeric tolerance; old estimator statistics match within1e-12 relative/absolute tolerance. Historical per-position losses and dense gradients were not retained, so their historical parity is not claimed. Focused tests separately check bitwise complete old/new gradients, direct credit and loss on a small actual trajectory. Both estimators use the same direct credit in the full comparison.

The selection rule was fixed before execution: causal full-gradient gate covariance must not increase in any cell, its ratio of four-cell sums must be at most0.80, and each other family's ratio at most1.10. Near-zero denominators use the declared1e-24 criterion. These are engineering utility tolerances, not statistical significance thresholds. Full-gradient covariance includes direct/score cross-effects; score-only covariance and paired differences are also reported.

| Parameter family | Causal / old aggregate full-gradient covariance | Reduction |
| --- | ---: | ---: |
| gates | 0.525886 | 47.4% |
| root_heads | 0.540052 | 46.0% |
| extent_head | 0.471084 | 52.9% |
| control_head | 0.533739 | 46.6% |
| emission_head | 0.748622 | 25.1% |
| null_heads | 0.552326 | 44.8% |

The four gate-cell ratios are **0.493555,0.463314,0.697771,0.519991** (memory initial/final, coding initial/final). Every gate case improves; every other family also improves in each measured cell. Gate pairwise cosine rises to0.00140–0.00595 but remains small. Paired gradient mean differences and standard errors are descriptive; this sample does not prove a nonzero population learning signal. A reduction in variance is not proof the next fit will learn useful generated answers. The previously measured sparse sampled/deterministic context overlap is unchanged because the paths themselves are unchanged.

## Validation and costs

**47 focused release tests pass**, including causal suffix/baseline coefficients, exhaustive four-particle analytic expectation, actual arithmetic publication/cancellation future credit, terminal-event exclusion and bitwise old/new path/gradient/direct/loss parity. Five report drivers are intentionally ignored in the ordinary run; the separately invoked comparison report executes and passes. No old pilot or V3–V7 run is repeated. Formatting and capability-wording checks accompany delivery; queue compatibility acknowledgements are not tests.

The complete comparison takes **8.534075s internally**, with7.930721s across its batch calls and8,748ms charged. Maximum retained tape capacity is32,001,856 bytes (30.52MiB), within the64MiB tape cap; no extra forward replay occurs. Sampled comparison RSS145,326,080 bytes. Build/tests cost113,012ms, peak sampled RSS2,628,304,896 bytes. Total **121,760/180,000ms**. These are scoped process measurements, not inference latency/allocation or energy results.

Shared cumulative ledger123,237,971/132,950,000ms; parent3,487,398/3,550,000ms after the pre-recorded120,000ms local extension. Step storage20,680,704 bytes before documentation/delivery within96MiB; final local receipt owns delivery/storage/engineering totals. The128MiB stop margin remains reserved. No paid compute or cleanup.

Both original checkout heads/dirty status, retained model and25 prior sealed roots/806 files verify unchanged, plus the new40-file sealed attempt. Initial parameter digest d29bb3f055f015161c782015ae5113872d47effa93744d9363f4d3f5fe72891b and final witness96a0faff5a6d439857e800007fa0d5a43d20c00a8c744f3649c503c605abf515 remain unchanged. No trained model export, fresh holdout, general language/coding qualification or model promotion follows this estimator result.

## Next action

Run one matched64-update causal-credit learning pilot from the same saved initial parameters as the old pilot, changing only full-trajectory credit to the selected causal suffix-LOO estimator. Preserve16construction/8development records, their order, initialization7341/eventseed973, four particles, SGD rate0.05,64-position window and original independent behavior/retention/control acceptance. Bind a separate causal training/checkpoint identity without changing old loaders or relabeling the old witness. Reuse saved old initial/final outputs and compare every response and formerly correct symbol; do not rerun the old fit. Check checkpoint/export and complete deterministic interpreter parity, then evaluate generated responses with ReadDisabled, ExactPayloadMasked and StateTransportDisabled controls; compile/execute generated programs only when their output qualifies under the fixed rules. No dose increase, corpus change, parameter sweep, automatic retry fit or model promotion follows estimator selection. Proposed complete240000ms (140000build/preparation/checks,40000fit,30000evaluation,30000correction),two buildthreads/one modelprocess,4GiBRAM,128MiBnewstorage,128MiBstopmargin. Refresh62602ms parent balance and current storage; record necessary preauthorized local time/storage extensions before use. Broader retained-model qualification remains separate.

#973 remains actively assigned/open; #964/#820 remain open. Notes and precise restart instructions remain in the established project-local handoff, under shared-core-first-step/addressed-attention-causal-credit-1/. Preserve the previous negative pilot and all original artifact paths.
