# Geometric emission block learning and research-guided continuation

September 12, 2026. References #973 and #964. The north star remains a shared learned geometric language model for local conversation/memory and coding/reasoning, with no transformer or mathematical matrix products in final serving. This record follows the [failed coordinate calibration](native_geometric_calibrated_emission_973.md); the original attempt stays sealed and retained `15baec48` stays authoritative.

## What the tools and existing work contributed

The project knowledge index and GitNexus were used for discovery. All four code graphs are September 3 snapshots, predating this shared core, so their references were checked against current source. The existing [mathematics](integration/architecture-2026-09/mathematics.md), [engines](integration/architecture-2026-09/engines.md) and [imports](integration/architecture-2026-09/imports.md) audits provide source navigation. The local research review and relevant Documents/Downloads material were also inspected; a structural geometry proposal alone does not supply a training method for the demonstrated calibration failure.

The useful immediate donor is Rust [`learn_code_pairs`](../crates/uor-r4-core/src/native_geometric/source_routing_training.rs): it proposes finite coordinates together, evaluates the real objective and restores rejected changes together. Its old correctness/hinge objective and feature representation are not imported. This shared-core successor keeps logistic likelihood and the original admission gates.

Published [Smooth Min-Max Monotonic Networks](https://proceedings.mlr.press/v235/igel24a.html) describes optimization stalls in min/max networks and studies smooth alternatives. That supports investigating the optimizer, not assuming that its theorem or empirical results apply to finite H4 caps. Here, complete finite block enumeration directly checks the unchanged hard operation; no smoothing is added to serving.

For later recurrent learning, [categorical reparameterization](https://arxiv.org/html/1611.01144v5#S2.SS2), [stochastic computation graphs](https://arxiv.org/html/1506.05254v3) and [differentiable logic gates](https://arxiv.org/abs/2210.08277) supply offline credit-assignment ideas. The [authors' logic-gate implementation](https://github.com/Felix-Petersen/difflogic) is a method reference, not an imported UOR runtime or language result. The local [train-soft/infer-hard archive](../research/ai-research/ai-router/router-research/tools/prime_transport/run_router_trainsoft_inferhard_v1.py) has dense attention/readout even in hard evaluation and no explicit straight-through bridge for state-ID argmax. Its name does not establish a compatible learned runtime. These distinctions guide the next learning investigation while preserving primary geometric mechanisms.

## Exhaustive saved-state diagnosis

A new Rust audit enumerates **3,484,800 legal settings per branch**: 120 landmarks and 11 thresholds on each ordered input, for both intersection and union. It retains probability margins rather than merging settings with equal hard predictions. It evaluates the complete class using `f64` logistic loss; hard-error counts are exact, while likelihood minima are numerical values with a declared replay tolerance.

| Saved branch | Coordinate calibration: errors / NLL sum | Unconstrained minimum: errors / NLL sum | Minimum NLL subject to existing gate |
| --- | --- | --- | --- |
| Root, 672 positions | 12 / 150.639696448391 | 12 / 150.639696448391 | Same |
| ASCII, 660 positions | 220 / 414.318812249522 | 211 / 406.900191409306 | Same |

The ASCII minimum meets the unchanged fewer-than-216 gate and improves loss. This is a demonstrated coordinate-calibration failure on these saved states, rather than a necessary objective/error tradeoff. The root was already optimal for this likelihood. The earlier minimum-hard-error witness (202) remains correctly reported as a different, worse-loss point. No descriptive audit witness was installed into a model.

The audit's `global_minimum_meets_gate` field refers to its selected witness; interpret existence of a gate-compatible tied minimum using the reported admissible-minus-global NLL gap and tolerance. Both gaps are zero here. `loss_error_frontier` stores the minimum numerical loss at each attainable exact error count; it is not a pruned Pareto set.

## Learning correction

The new offline `calibrate_emission_blocks` performs one exhaustive search for every observed training byte-tree node's full landmark/threshold/operator block. Selection uses unconstrained likelihood only. It retains the current setting for improvements below `1e-10`, checks selected loss against direct hard-branch replay, and returns an error without publishing a partial artifact if the bounded search cannot finish.

The search uses a compact histogram over the two raw signed angular ranks and target bit. For scores `S` in `[-9,9]`, it evaluates the finite survival identity `E[L(S)] = L(-9) + sum_{k=-8..9} P(S >= k) (L(k)-L(k-1))`. Intersection uses the joint upper tail; union uses the sum of marginal tails minus their intersection. This is an offline optimization of the same branch loss, not a runtime contraction or complete-state table.

A new optional artifact block-calibration configuration binds this method, one complete pass, parent, data digest, time limit and implementation identity. Original schema-2 coordinate artifacts retain their original bytes through pinned compatibility dispatch. Runtime branch scoring, recurrence, context access and the serialized geometric parameter domains remain unchanged. The new method changes learned parameters and training provenance, not the serving equation.

A separately frozen successor design preserves the original 12 construction documents, unopened six-document holdout and four generation prompts. Admission still requires root errors at most 12/672 and ASCII errors fewer than 216/660. Only after admission may one seed-443 joint fit run, capped at 100,000 proposals and 120 seconds. Fresh prediction must improve NLL by more than 2% against both original legacy and block-calibrated parent, preserve their byte accuracy, and show greater-than-1% defined context/state intervention effects. Four-stage actual generation remains a separate required assessment. No promotion follows from this authored smoke experiment.

## Execution

**Calibration: PASS_BLOCK_CALIBRATION.** One exhaustive sweep over 94 occupied nodes (327,571,200 settings) took 15.198 seconds. Block artifact `4d83b1bb` reduces construction loss from 3.473156 to 3.422394 and raises correct byte/EOS predictions from 65 to 80/672. Root errors remain 12/672 and ASCII errors fall to 211/660, satisfying the original gate.

**Joint fit: FAIL_CONTEXTUAL_TRANSFER_SMOKE.** The single fit exhausts 100,000 proposals in 31.444 seconds, accepts 149 and produces `ade9a1cf`. Construction NLL falls to 3.375391 but accuracy slips to 79/672. Accepted proposal counts are embedding 1, transition 0, query 29, key 28, read 61, phase 0, cap landmarks 27, cap thresholds/union 3, null 0. These counts describe this search; they do not prove the untouched families lack useful settings.

| Stage on the same 381-position fresh draw | Mean NLL | Correct byte/EOS predictions |
| --- | --- | --- |
| Legacy `5979f2e2` | 5.747049 | 2 |
| Prior coordinate `968cf841` | 3.685167 | 29 |
| Block-calibrated `4d83b1bb` | 3.707224 | 29 |
| Joint `ade9a1cf` | 3.663040 | 28 |

The joint fit improves NLL only 1.19185% over block parent (required >2%) and loses one correct prediction. Block calibration itself worsens fresh NLL relative to its coordinate parent despite better construction. Context-disabled NLL 3.703623 is 1.10791% above Full, meeting that scoped criterion. Prior-root-disabled NLL 3.673161 is only 0.27630% above Full, failing its criterion. Four-stage generation was executed under Full, ContextDisabled and StateDisabled (48 outputs). All four Full joint outputs are incoherent character sequences, each reaching 96 bytes without EOS. For example, the Rust prompt begins `ooehebenlboeiea lnel loiaanan...`; it is not executable Rust. No generated-program success is claimed or warranted.

The saved-state arithmetic audit, 18 focused shared-core tests and actual-candidate 512-step zero-allocation census pass. The focused tests precede two reviewed deadline-return guards; final compilation and actual artifact/model/generation/allocation execution use the corrected source. The serving integer-kernel source is unchanged. No broad language, reasoning, complete-serving-path or energy qualification follows.

This step used 331,322/450,000 model/build/test ms and 17,362,944 additional storage bytes within 256 MiB. All cumulative charges are retained; no model/storage allowance increase, paid compute or destructive cleanup occurred. [Tracked evidence](evidence/native_geometric_block_learning_973.json) contains exact artifact identities, sealed file sets and hashes, compiler/executable/source bindings, actual outputs and resource receipts.

**Next:** implement a bounded offline categorical credit-assignment prototype for the existing shared finite state/read parameters. Preserve actual hard geometric forward choices and export only discrete parameters. Check whether surrogate-proposed updates agree with enumerated actual hard-loss changes on a small development sequence before another joint-training campaign. State capacity, data scale and curriculum remain unresolved; this result does not justify adding more serving mechanisms or claiming an optimizer-only explanation for all language failure. Preserve the candidate and opened draw; no unchanged fit, gate relaxation or promotion.
