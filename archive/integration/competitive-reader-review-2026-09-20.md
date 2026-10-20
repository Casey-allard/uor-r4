# Principal review of PR #1321 — contextual utility is next, geometric attribution remains open

Review of merged source `a1fadd1f494733144c96f8419106a95ac77d8f61` and the retained `competitive-reader-1` experiment. Prepared September 20–21, 2026. PR #1321 head and merge have the same tree, `bf0d8df19e0d659848926b00e0e5a66f8eb03191`. The original [result](competitive-reader-result-2026-09-20.md) and [receipt](../evidence/native_geometric_competitive_reader_2026-09-20.txt) remain historical evidence; this review corrects their interpretation. The [canonical plan](project-track.md) owns the roadmap and the [single next prompt](deepseek-contextual-utility-step-2026-09-20.md) owns the constructive continuation.

## Decision

Keep the learned relative-H4 reader, exact occurrence memory and frozen local predictor. Add a small causal **contextual utility/strength controller**, with source-separated natural-text fitting. Do not promote this artifact, expand the ring, begin a two-hop scheduler, or replace the state with S7 harmonics on the strength of this run. The positive synthetic aggregate is useful evidence, but the experiment does not establish that ranking is solved, that the categorical comparator was competently learned, or that unique H4 structure caused the gain.

This review also repairs bounded source defects and exercises focused tests, so the next run can spend its effort on learning rather than another standalone instrumentation pass. These repairs change no retained artifact and produce no new full-fit quality result. The corrected construction is a new fixture version; its quality remains NOT_RUN.

## Retained numbers and their correct denominators

The saved construction contains 140 sequences and 1,946 candidate-bearing prediction positions; 761 positions have an admitted correct payload and 1,453 have multiple candidates. Saved mean hard-action losses remain local 12.6347, exact 10.1377, categorical 10.1513, H4 9.9582 and relation-channel lesion 10.2716 bits per candidate-bearing position. These are not all-token natural-language losses.

Independent aggregation of the saved per-sequence loss sums yields:

| Comparison | Difference, bits/candidate position | Sequence-cluster 95% interval | Scope |
| --- | ---: | ---: | --- |
| H4 minus exact | -0.179540 | [-0.270206, -0.093226] | Constructed aggregate advantage retained |
| H4 minus categorical | -0.193174 | [-0.293387, -0.097055] | Categorical map stayed frozen; not a matched learned-map attribution |
| H4 minus relation-channel lesion | -0.313458 | [-0.424553, -0.202072] | Post-fit ablation; query-dependent admission/features remain |

The original -2.4956 [-3.7642, -1.3010] is a mean **bits/sequence total**, not a per-token difference. The correcting [saved-data receipt](../evidence/competitive-reader-principal-review-2026-09-20.json) pins the inputs, 2,000 bootstrap draws and ratio-of-resampled-sums estimator. This calculation performs no model inference. Data-cluster intervals from one fit do not measure training-seed stability.

The prose counts 623/761 H4 and 592/761 exact are valid **covered decision success**. All 1,946 candidate positions take a read, so conditional all-read precision is only 623/1,946 = 0.3201 and 592/1,946 = 0.3042 respectively. The JSON `read_precision_covered = 1` is a counter bug: its numerator counts covered reads regardless of correctness. Correct counts must not be discarded because another field is wrong.

Natural-text loss remains **6.8002 -> 8.0910, +1.2908 bits/token** on eight documents/488 positions, with 178 reads. No natural-text fit was performed. This fails the declared +0.05 tolerance and `positive=false` remains unchanged. The old text panel is now an inspected regression panel, not fresh validation.

## Source and instrument findings

1. **Categorical learning did not occur.** In `competitive-reader.rs`, each trial code was evaluated, the improvement branch was empty, and the original code was restored. The geometric arm moved 52 roots; the categorical code-search change was zero. Independent reload repaired the older wrong-address defect, but it did not make this a learned-map comparator. The unnecessary whole-dataset loop also consumed about 164.6 seconds versus 0.245 seconds for geometric affected-position refinement.
2. **Shared function did not imply shared state.** Evaluation excludes the current input token from the ring; generation warmed and advanced the ring including it. This changes candidate availability and adjacency observations. The 220-position parity check compared the same evaluation observation to itself; it did not test the real generation caller. The old main evaluation also duplicated selection/boost logic rather than calling `predict_next`.
3. **The intended partner task was not the task actually generated.** The source used family A even when the randomly chosen direction required B. Among 140 final queries, 82 have a same-role source and 58 a partner source. The “absent” branch changes the target without removing the family source: all 140 contain one, and 26 final labels differ from every relevant-family payload. The absence counter was tautological. These errors do not erase the saved aggregate but prevent the intended relation/abstention claim.
4. **Most scored positions are intermediate tokens.** Only 140/1,946 positions are final queries (123 covered); 608 are keys, 599 roles and 599 values. Of the 761 covered positions, 489 are intermediate keys. Without per-position outcomes the saved aggregate gain cannot be attributed to the final relational question. Future reports separate these strata without hiding the all-position total.
5. **“Query blind” is too broad a label.** Setting the relation query root to identity leaves query-conditioned admission and exact features. Its loss increase supports reliance on that channel within this artifact, not a refitted query-free baseline or complete semantic counterfactual evidence.
6. **Timing and intervention scope were overstated.** The local probe still maintained the ring, and both probes divided 29 predictions by 30. The probe lacks representative active candidates. The future-token test wrongly required invariance at the changed current input; unchanged predecessors are the proper causal boundary. The altered-source test conditions on re-observed candidate availability/read and can mutate a source other than the original selection; its counts are not a complete paired intervention result.
7. **Artifact and inventory facts need correction.** The retained RLR2 is 4,340 bytes, not 8,904. Its original loader did not reject invalid root/code domains or mismatched vocabulary; the RLRK checkpoint omitted the categorical mode/map. Current retained-root inventory includes unlisted `sum.py`. Listed file sizes match; the saved-data audit does not claim independent BLAKE3 revalidation. Preserve the original root and put new outputs in exclusively claimed roots.

The bounded source repair covers shared mode-aware refinement, genuine categorical update/tie tests, map validation, versioned categorical continuation, prefix-state conventions, shared evaluation, fixture truth/absence, loss-unit and precision counters, cutoff semantics and actual timing counts. RLRK v2 preserves mode, learned codes and initial-map provenance; legacy v1 is explicitly geometric-only. The [validation receipt](../evidence/competitive-reader-repair-validation-2026-09-20.json) records 16 passing module tests and four passing runner checks, including eight generated steps from the retained artifact with matching real generation/evaluation callers. Both smoke arms repeat token32; this is not useful prose. Formatting, claim wording and diff checks pass. Full experiment replay and a useful new artifact remain NOT_RUN in this review.

## Why contextual utility is the right next learning problem

For a selected payload with local probability p, an additive logit boost a changes observed-token loss by

`delta(a) = log(1 + p*(exp(a)-1)) - a*I[payload == target]`.

This exact reference assumes only the selected logit changes and does not saturate/overflow; the actual exported quantized action remains authoritative. Conditioned on the causal observations and the source-selection event, let r be the probability that the selected payload is correct. Then the expected difference is `D(a) = log(1 + p*(exp(a)-1)) - r*a`, with `D'(a) = p_a-r` and `D''(a) = p_a*(1-p_a) >= 0`. For an arbitrarily small positive boost, benefit requires r > p. For a finite action a, the exact condition is `r > log(1+p*(exp(a)-1))/a`. The ideal nonnegative bounded boost is `clip(logit(r)-logit(p), 0, A)`; with no upper clipping its optimum gain when r > p is `-KL(Ber(r) || Ber(p))`.

This explains why “read precision above chance” does not justify an eight-nat boost. It also explains why highest correctness probability and highest predictive utility can select different sources. Ranking, admission, observation quality and distribution shift remain possible bottlenecks; global strength is a demonstrated structural defect, not a proved sole cause of language harm.

A small empirical action-cost table is a direct implementation: collect true offline loss changes per causal bucket and finite action, including zero; choose the action with lowest learned expected cost, with tune-selected shrinkage/fallback for sparse buckets. Average the actual per-example costs, not the nonlinear formula at the mean p. Export integer decisions/table indices. Training/evaluation may calculate probabilities and logarithms; serving need not. A raw score margin is an available feature, not a calibrated probability. A learned cost table need not be marketed as a calibrated probability estimator.

First freeze source choice/pool and compare learned global strength with contextual strength. A cheap offline finite-action opportunity diagnostic can distinguish selected-payload strength error from wrong ranking or missing admission. It is a target-using diagnostic, never a serving feature or model-quality result. Permit one justified joint adjustment if that evidence calls for it; do not declare ranking complete by fiat.

Relevant primary literature supports adaptive retrieval integration and cautions against treating perplexity as generation: [Drozdov et al., 2022](https://aclanthology.org/2022.findings-emnlp.218/) study when retrieval helps and adaptive weighting; [Wang et al., 2023](https://aclanthology.org/2023.emnlp-main.929/) test the disconnect from open-ended generation. [Popordanoska et al., 2024](https://proceedings.mlr.press/v238/popordanoska24a.html) distinguish calibration from other predictive properties. These are mechanism lessons, not adoption of their dense language backbones or proof of UOR-R4 advantage.

## All mathematical bridges remain in the programme

The [cross-mechanism synthesis](geometric-attention-mechanism-synthesis-2026-09-20.md) remains the reuse map. Exact prime/UOR addresses carry identity; finite H4 relations carry directed compatibility; retained versions/payloads carry memory; learned scalar utility determines whether and how strongly a read affects prediction. These roles are complementary. Common-left invariance of `q^-1 k` is an algebraic fact, not proof of learned syntactic invariance or a full quaternionic-Hopf implementation.

Structural role/scope banks and retained fiber enter when recency or observation aliasing limits useful access. Dependent query updates and Read/Emit/Stop then combine evidence, with source interventions and derived outputs as separate requirements. S7/E8 or harmonic coefficient banks enter for a witnessed representation, retention or storage/cost advantage, compared with direct slots at equal memory. Unit directions in R8 lie on S7; the E8 lattice is not the whole sphere. Harmonic orthogonality is under an inner product over the domain, not immunity of a finite quantized superposition to all collisions, shared modes or nonlinear updates. Quantum-spin analogies require an explicit classical state/operator; scalar utility does not require supersymmetry or twin primes. Existing mathematical corrections, including `G2/SU(3) = S6`, remain in the Hopf direction note.

The order is useful contextual access with learned utility -> structural persistence where needed -> dependent attention/composition -> broader language and executed Rust -> quality-matched laptop scale and complete-path energy. Cost profiling happens throughout. The new math is preserved and assigned jobs; none of the new evidence identifies a missing dimension that would justify replacing the current read before testing its integration.

## Knowledge continuity

The project knowledge service stores provenance-bound snapshots, not model memory or authority. This review, the next prompt, active plan/current-state extracts, validation, mechanism map and owning issue snapshots should be imported as a compact public batch pinned to the delivery revision. Search for `competitive-reader-review-2026-09-20` or `deepseek-contextual-utility-step-2026-09-20`, then verify the revision against live main; generic phrase search can rank older snapshots first. Preserve historical positives/negatives and follow correction links. Do not duplicate the whole historical corpus or treat a retrieved old execution instruction as current.
