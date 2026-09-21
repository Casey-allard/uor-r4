# Principal review: close finite reader-policy feasibility before adding state

September 21, 2026. Reviewed PR #1330 merge `4be53d88e7346d32d9a6e0a4584bb3cbc6f813a9`; its head `60e19c7d22d22adc49ed3bed8386f662eaf1087f` has the same tree `101437725408fd023092eb6b3b859c9edf0bb46f`. This review combines three independent source, saved-evidence and mathematical reviews, current authority, mechanism history and primary literature. It executes no new model. [Independent numerical audit](../evidence/policy-objective-principal-review-2026-09-21.json); [next execution prompt](deepseek-policy-feasibility-step-2026-09-21.md).

## Decision

The fit/serve feature repair is real, and the corrected policy fails all four declared outcome criteria. Preserve that negative. Preserve the previous artifact's small probability gain too: changing the learner does not refute an earlier artifact measurement.

Run **one terminal, bounded constrained-policy feasibility and selection experiment** on the present observations. Minimize natural-text loss while preserving present-query emissions and loss, and absent-query read counts and loss. First collect the missing counterfactual sufficient statistics once from development data. This is a substantive learning experiment, not another repair-only review or a sequence of mixture-weight sweeps. If infeasible, unresolved within its solver budget, or unsuccessful on fresh evaluation, leave this scalar-policy campaign. Identify a concrete missing observation or operation only from independent evidence; a solver timeout establishes a computational limitation. A useful partial result remains reusable.

This is a measured tradeoff among tested policies. It is not proof that the objective alone caused failure, that support is sufficient, or that the geometry needs more dimensions.

## Verified results and the limits of freshness

The delivered `reader-utility-4` has eight manifest members with valid sizes/BLAKE3 and no unlisted files. Three new artifact hashes, four reader-parent hashes, 32 reader document hashes and both modified source SHA256 values verify. The previous `reader-utility-2` remains valid. Attempt 3 is also sealed: attempt 4 repeats its documents, construction seed, panel values, generation, intervals, policies and artifact bytes after changing a rejection-probe bookkeeping check. This is faithful replay of the same result, not an independent second acceptance population. Documents are reader-separated, but originate in the frozen local prior's corpus; they are not globally unseen language.

All five declared document-bootstrap intervals reproduce exactly. Eight final reader documents contain 1,952 positions and 782 candidate-bearing positions.

| Observation | Verified value | Interpretation |
| --- | ---: | --- |
| Corrected H4 policy minus local text loss | +0.119319 bits/token [ +0.070211, +0.167862 ] | Text harm; +0.05 screen fails |
| Corrected categorical policy minus local | +0.153275 [ +0.094199, +0.207194 ] | Matched alternative also harms text |
| Frozen contextual parent minus local | +0.453197 [ +0.326104, +0.599774 ] | Corrected policy reduces the parent's harm |
| H4 policy minus contextual parent | −0.333879 [ −0.487228, −0.200464 ] | Useful relative improvement, still worse than local |
| Fixed one-nat boost minus local | −0.014769 | Simple dose comparator has lower text loss |
| Final-present correct emissions | 56/121 versus parent's 77/121 | Preservation fails |
| Final-absent reads | 19/19 versus parent's 5/19 | Absence criterion fails |
| Final-absent loss change versus local | +0.2969 versus parent's +0.1379 bits/query | Both harmful; corrected policy worse |

A retrospective saved-document interval for the fixed one-nat arm is [−0.024778, −0.005057] bits/token; for H4 minus fixed it is [+0.089268, +0.176336]. These are descriptive post-hoc comparisons, not new preregistered acceptance tests. On the exact old documents, fixed one nat gives −0.025483 versus PR #1328's −0.019477; local per-document sums are identical. A generic constant already matches/exceeds that old aggregate benefit. This limits any learned-policy advantage claim without erasing the old measurement or asserting complete behavioral equivalence. The misindexed artifact was not rerun on the new final population.

Whole construction has 2,418 positions: correct payload reads rise 446→516 while emitted correctness falls 355→319. The constant one-nat arm emits correctly at only one position. On six all-present generated construction prompts H4 gets the first answer token right 3/6 versus parent 4/6; absent generation is not covered. Natural-text generation remains degenerate. These are neither broad language nor integrated model results.

## What the repair establishes

`competitive-reader.rs` creates `PolicyConfig` before events (lines 3959–3973 at the reviewed revision). Library extraction and serving use the same configured thresholds. Actual-action regret receives the action actually served and has zero cost-versus-logit mismatches. Preservation now measures emitted answers; absence measures actual read counts and loss. These improvements make the four negative outcomes usable. Both recorded modified source hashes match the merged source, unlike the prior run's unresolved runner correspondence.

Same-seed previous-construction admission means reproduce the preceding audit: 0.2006360298 nats/candidate position and 0.8316569771 nats/full-stream position. New-seed means 0.1945948513 and 0.8316367257 are a different sample, not rounding differences. Both denominators now have their proper populations, including 608 empty pools in the new full stream. Oracle opportunity is not an achieved improvement.

The corrected fitted table's histogram is `[0,0,25,7]` for NoRead, weak, one-nat and eight-nat actions. H4 has 16 supported buckets, 13 unobserved buckets and three observed below the support threshold. Weighted support is not independent document support. No served-bucket trace permits excluding support as a cause.

The selected objective's tradeoff is already visible in fitting: H4 text loss is +0.131000 bits/token while fixed one nat is −0.030979; construction improves −1.910342 versus −0.325066 bits/position, with 570 versus seven correct emissions. Reconstructed equal-domain **candidate-bearing-event** mixed cost is −0.789851 nats/event for the fitted table versus −0.172578 for constant one nat. The table is doing better on what it was asked to optimize. That scalar objective does not impose the separately required text, answer and absence behavior. Equal candidate-bearing domain weights are not equal full-stream domain weights when admission fractions differ.

## The finite question that should be settled next

For a fixed development prefix, source learner and local logits, record the bucket b, selected exact source reference c, cluster identity and the four counterfactual outcomes: loss change, emitted correctness under the real integer tie rule, and read indicator. For selected payload probability p and positive boost a, the ideal score-space identity is

`delta_loss(a) = log(1 + p * (exp(a) - 1)) - a * I(payload == target)`.

Verify it against the actual integer-logit path, including quantization and saturation; use the observed effective boost if different. NoRead has zero change. Targets and stratum labels may supervise offline training, never a serving address. Empty pools contribute the fixed local result to full-stream denominators.

Choose one deterministic action per bucket: `x[b,a] in {0,1}`, `sum_a x[b,a] = 1`. On pinned teacher-forced prefixes the following are linear sums in x:

- Minimize complete-stream development text loss, with declared document/token normalization.
- Constrain present-query emitted accuracy to preserve the parent within the prospectively declared margin.
- Constrain present-query loss to parent plus its declared margin.
- Constrain absent-query reads and loss to no worse than parent.
- Evaluate text harm containment and useful transfer separately; a text optimum can still be above the acceptable loss.

The current mixed costs and raw/weighted bucket support are serialized, but per-action emitted counts and stratum-conditioned loss/read counts are not. Executed-policy aggregates cannot reconstruct them. One development extraction must save bucket × action × stratum × document/sequence contributions before any solve. No repeated model passes are required for optimizer iterations.

Start with optimistic per-position and per-bucket emission bounds. Distinguish the operational class with frozen support/fallback restrictions from every deterministic table on these observations. Failure of the restricted class is not failure of the wider class. Use a bounded exact search or solver with incumbent, bound and numerical tolerance. There are 128 binary variables but up to 4^32 tables: naive enumeration is not a small task. An infeasible relaxation can certify deterministic infeasibility with sound tolerances; a feasible fractional solution does not provide a deterministic serving policy. Solver timeout is UNRESOLVED and does not establish a missing observation or representation defect. A valid feasible incumbent may still be evaluated if the prospective selection rule permits its recorded bound; distinguish that from proved optimality. A lambda sweep does not certify the Pareto frontier.

A selected deterministic candidate gets one frozen fresh evaluation only when it satisfies the behavioral constraints and declared development text screen. A certified optimum above that screen is an obstruction, not a reason to spend a final pass. Empirical development feasibility is not generalization. Teacher-forced additivity does not extend to rollout: changed outputs change later prefixes, source sets and buckets. Test generated complete responses and absence alongside the loss measurements.

## Remaining instrumentation work inside the constructive run

These defects qualify claims but do not explain away the measured negative:

1. The current loader checks threshold equality/table length and the runner compares reloaded selectors/fit digests before scoring. Complete dependency binding is unfinished. The swapped-policy probe checks byte inequality, not rejection by a consumer; the wrong-data probe compares the original digest to a made-up wrong one. Manifest self-hash verification occurs after scoring. Bind actual artifact bytes, configuration and complete fit dependencies through an expected-manifest loader before returning a predictor; exercise rejection there. The input digest must cover source references, sequence boundaries, local dependencies and configuration. Label fit identity separately from evaluation receipt identity.
2. The intervention's neighbor-change counter includes the deliberately edited source, making 140/140 uninformative about other neighbors. Exclude it and count changed/added/removed neighbors. `lost_support` currently means a missing observation row; measure loss of admission of the original occurrence. The separate 140/140 still-admitted result remains valid here.
3. Actual loss/logit equality is checked, but the new decomposition identity is not asserted at each consumed position. Add that direct assertion or narrow the claim.
4. Timing is short and fixed-order. Construction's paired median is **+6.584 microseconds total**, not negative; text's is negative and has no candidates. Neither resolves the reader increment. Physical energy stays UNAVAILABLE; whole-path D0-b remains unqualified. Do not open a profiling campaign around an unqualified model.

## How the larger geometry roadmap changes

The roadmap gains a terminal decision, not another indefinite gate stage. The current signed-H4 source learner, exact occurrence memory and scalar influence mechanism remain useful components. The next bounded result selects the missing operation:

| Measured obstruction | Smallest justified direction |
| --- | --- |
| Suitable selected sources but conflicting influence decisions in one bucket | Inspect omitted existing causal information first: absolute source/NoRead confidence, directed relation, or local probability/normalization. Equal selected-token gaps need not imply equal probabilities. |
| Identical recent observations require different answers after older scope changes | Learn finite content-dependent role/scope transport, retaining exact payload/version references. Compare against equal-byte ordinary state and existing H4. |
| Correct payload missing from the admitted pool | Repair the witnessed admission/retention failure; do not tune a dose to compensate for absent information. |
| Correct retrieval cannot produce the required derived answer | Advance shared read–update–emit and dependent composition, requiring output absent from source payloads and causal first-read interventions. |
| Memory behavior improves but free prose remains degenerate | Improve the local generative learner jointly with contextual operators; better copying alone does not supply general prose. |

Learned structural persistence, dependent attention/composition, broader language and executed Rust, then qualified scale/energy remain the ordered responsibilities in [project-track](project-track.md#structural-memory-and-geometric-representation-follow-up). Later operations need not wait for an impossible scalar-policy screen; this last feasibility experiment tells us what to stop refining.

The [current/retired mechanism synthesis](geometric-attention-mechanism-synthesis-2026-09-20.md) and [Hopf/spin direction](structural-memory-hopf-direction-2026-09-20.md) retain all proposed bridges. Signed H4/Spin composition can implement directed relative transport by finite tables. Hopf base observation requires separately retained fiber for reconstruction. S7, normalized E8 codes and harmonic coefficient banks are conditional representation choices, not automatic extra memory or orthogonal syntax. Finite quantized channels still need learned contextual assignment, collision/interference tests and exact identity outside the approximate state. Scalar fields give compatibility/utility values; no supersymmetry, twin-prime or general-relativity mechanism has been derived that supersedes the present task. A useful new algebraic mechanism is welcome when it specifies an operator, resource cost and discriminating experiment.

## External research and what transfers

[GRAPE, revised May 2026](https://arxiv.org/abs/2512.07805) motivates group actions and compositional relative encoding; [PaTH, revised February 2026](https://arxiv.org/abs/2505.16381) motivates content-dependent accumulated transport. Our inference from these sources is to test a finite learned H4 state when an older-context alias is measured. Their attention implementations and reported gains do not qualify this Rust model or authorize dense serving.

[Constrained cost-sensitive reductions](https://proceedings.mlr.press/v80/agarwal18a.html) and [complex-metric constrained learning](https://www.jmlr.org/papers/v25/22-1137.html) support formulating separate behavioral requirements directly. Some reductions yield randomized classifiers; we require a concrete deterministic table. [Cotter et al.](https://proceedings.mlr.press/v97/cotter19b.html) motivate separating optimization and constraint validation. None establishes generalization for our small adaptively inspected panels.

[Wang et al., EMNLP 2023](https://aclanthology.org/2023.emnlp-main.929/) document that retrieval perplexity gains need not improve open-ended generation and that generated-prefix retrieval can deteriorate. This supports keeping actual rollout as a separate endpoint, not treating one-token CE as general language. These sources guide the experiment; they are not evidence for UOR-R4 performance.

## Resources and delivery scope

Live ledger at this review: **192996749 / 194900000 ms**, leaving **1903251 ms (31.72 minutes)**. No model build/fit/inference debit, extension or external spend is added here. Necessary local extensions are already owner-authorized when fully projected and recorded before use; the prior decision not to spend is not proof a successor is unaffordable.

Read-only inventory at 2026-09-21T13:22:37Z: **38537052160 bytes free (38.54 GB / 35.89 GiB)**, reserve **36766079385 bytes**, only **1.77 GB above reserve**. Shared target is 8.78 GB; models at least 20.98 GB with three protected-path errors; knowledge 1.96 GB. APFS/overlapping allocations are not additive. **Zero deletions**; preserve unique evidence, sealed roots, downloads and the owner's checkout. Reuse valid builds and project actual growth before the next run; an old 6 GB build allowance would exceed current headroom. Retain the 128 MiB stop margin in addition to the active reserve policy.

This review updates the front doors and current direction, preserves historical results with explicit corrections, and supplies the next prompt. No new model capability is claimed. Live issues #820/#973/#1139/#962/#963/#964 retain their broad open acceptance; use revision-pinned knowledge for recovery and live state before execution.
