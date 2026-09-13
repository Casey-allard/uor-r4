# Tied categorical learning over complete geometric trajectories

September 12, 2026. References #973 and #964. **FAIL_TIED_CATEGORICAL_METHOD_SMOKE; no promotion.** Retain `15baec48`. This follows the [occurrence-credit failure](native_geometric_credit_assignment_973.md), using the same experimental parent `ade9a1cf` and four already-open 32-byte prefixes. The [canonical plan](integration/project-track.md#current-implementation-sequence--owner-adopted-september-12) remains the shared geometric language core.

The new Rust offline learner samples two shared discrete parameters once per complete evaluation, so every repeated use and every changed future access contributes to the same hard-trajectory cost. It learns a factorized categorical distribution using a leave-one-out score-function gradient, frozen probabilities within each minibatch, and bounded logit updates. This follows the estimator construction in [Gradient Estimation Using Stochastic Computation Graphs](https://arxiv.org/abs/1506.05254); the paper supports the method, not a UOR-R4 performance claim. The finite-difference test checks the expected estimator on a nonseparable two-variable objective.

Each of four seeds uses 32 batches of 64 samples, step 40, initial probability 0.5 on each parent root, and the remaining probability shared across 119 alternatives. Logits are centered and clipped to [-30,30]; no clipping occurred. Parameters 1176 (transition) and 1930 (read) were selected from the saved route trace before scoring their joint alternatives. Every arm uses 2,050 actual full-model evaluations: initial, 2,048 draws and final witness replay. Fixed-distribution joint sampling and greedy uniform coordinate search have the same call budget. After all twelve arms finish, a 14,400-setting exhaustive reference measures joint costs. The oracle never supplies a learner update or installed artifact. Early generation follows each artifact construction; all artifacts then receive complete construction and already-open development evaluations with Full, ContextDisabled and StateDisabled controls.

| Frozen condition | Actual result | Decision |
| --- | --- | --- |
| Final distribution improves expected loss in at least 3/4 seeds | 4/4 | Met |
| Exported hard candidate strictly beats fixed joint sampling in at least 3/4 seeds | 0/4: three ties, one loss | Failed |
| Exported hard candidate strictly beats coordinate search in at least 3/4 seeds | 4/4 | Met |
| Mean fraction of available oracle improvement at least 0.5 | 0.788002 | Met |

All learned runs export parameter values `[119,76]`, changing only read1930. Their diagnostic mean NLL improves 3.535634 → 3.512336 on 132 targets. This equals the best read-only replacement; their coordinate-baseline advantage does not establish that simultaneous changes were necessary. Fixed sampling reaches the same result in three seeds and a better pair `[105,6]` at 3.510977 in one. The descriptive joint oracle minimum is `[83,50]` at 3.506068. No oracle witness is installed.

Distribution expected NLL decreases from 3.545553 to 3.538130–3.538262, but remains above the unchanged hard parent's 3.535634. Improving an exploratory distribution and finding a good hard sample are different results. Learned sampling has not demonstrated an advantage over fixed sampling under this frozen budget.

| Full intervention | Parent | All four learned exports |
| --- | --- | --- |
| Complete construction NLL, 672 targets | 3.375391 | 3.467544 |
| Complete construction correct | 79/672 | 67/672 |
| Open development NLL, 381 targets | 3.663040 | 3.656606 |
| Open development correct | 28/381 | 27/381 |

The small-panel improvement regresses the complete construction objective. All sixteen Full candidate continuations remain incoherent and reach the 96-byte limit without EOS. Two prompt outputs change and two remain byte-identical to the parent. The context-disabled panels are unchanged; state-disabled results and all raw generations remain in the evidence. No general language, coding, energy or full-path qualification is established. Four distinct CIDs bind distinct training seeds/configurations even though the exported serving parameters and observed outputs are equal.

The integer serving region is byte-identical to the prior source. Probabilities and gradients are offline only. Optional tied-training provenance is mutually exclusive with the previous fit configuration; artifact loading preserves the old block-calibrated implementation and validates the new metadata. Parent bytes and prior sealed report file sets are preserved.

Validation: 27 focused shared-core tests pass on final source, including estimator, bounds, deterministic callback counts, serialization and older path checks. The actual experiment completes once (24,600 comparison evaluations, 14,400 oracle evaluations, separate behavioral verification) in 2.281 seconds internally; its method gate is failed. The actual saved candidate passes a separate 512-step zero-allocation census. The first compilation failed on a misplaced harness deadline guard; the corrected build passed, and the final source was rebuilt and retested after the independent guard review. All charges are retained. No fresh draw, full-model campaign or V3–V7 replay ran.

Exact source/compiler/binary, four artifacts, frozen design, all distributions, exhaustive costs, controls, generated bytes, independent reviews and cumulative resource receipts are bound in [tracked evidence](evidence/native_geometric_tied_learning_973.json). Originals stay in `shared-core-first-step/tied-learning-1/` under the established project-local handoff.

**Next action:** one bounded comparison using the complete 672-target construction objective to select updates, with simple fixed-distribution joint proposals and a matched coordinate baseline from the unchanged `ade9a1cf` parent. Freeze block selection, calls, seeds, acceptance and resources before execution; judge complete-objective loss and accuracy plus retained/open controls and actual generation. The 132-target panel remains diagnostic. This addresses the demonstrated objective mismatch; it does not assume that pair updates, optimizer choice, capacity or curriculum alone resolve language learning. Do not scale or tune this failed categorical sampler, install descriptive oracle roots, select on the opened development panel, or relax promotion acceptance.

Resource receipt: 299,956/540,000 ms model/build/test, parent cycle 1,632,664/2,100,000 ms, shared ledger 121,383,237/132,950,000 ms. A 500,000 ms local allocation increase was recorded before use within the unchanged cumulative ledger ceiling. Added storage 14,635,008 bytes within 256 MiB; parent growth 1,380,044,800 bytes within 2 GiB, retaining the 128 MiB stop margin. Peak sampled process-tree RSS 2,486,157,312 bytes. No paid compute or destructive cleanup. Final engineering/delivery receipts append locally.
