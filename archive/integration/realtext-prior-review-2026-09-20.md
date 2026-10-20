# Review after PR #1298: advance the learned prior to real text

September 20, 2026 UTC (September 19 local). Audited source: `a3c5403587bd27b32399622ec0bebc010447b9f2`. This is a source, retained-evidence and literature review; no Rust build, training or model evaluation was executed. The [next complete execution prompt](deepseek-realtext-prior-step-2026-09-20.md) carries the implementation and experiment.

## Decision

Keep the repaired learner and run the first corrected **prior-only real-text learning curve**. Repair mid-pass continuation and remove floating training diagnostics from the shared serving forward as part of that runner. Add the missing focused derivative assertion while touching that path. These are bounded implementation prerequisites within the same task, not grounds for another stand-alone repair campaign or a new architecture.

The small learned gate now passes. A floating comparator was conditional on its failure and is unnecessary. The next uncertainty is whether exported low-bit parameters learn contextual prediction on unseen documents beyond a fit-only marginal distribution. Memory remains disabled to identify that effect. Geometry remains the architectural destination, but this local prior does not yet use it predictively.

## Verified result and its scope

PR [#1298](https://github.com/UOR-Foundation/uor-r4/pull/1298) is merged. Head `5083f62a3af771f6eec2e856a063f5d20d2f0e15` and merge `a3c54035` share tree `0f8fd13556f39a92dff9713ca51b36d65abc7e35`. The owner's main checkout is clean. Live #973/#820/#963/#964 remain open; none has a linked GitHub project item.

Retained report: `/Users/casey.allard/uor-r4/.uor-models/prior-learning-gate-2026-09-20/attempt-1`. All six manifest members match their BLAKE3 hashes and sizes, with zero unlisted files. Independently recomputing CE and argmax from its saved integer scores reproduces the reported curve without executing a model. The source uses the predetermined step 2,000 endpoint and evaluates the permutation without refitting. The recorded manifest inventories the retained files; it is not a complete source/data/optimizer experiment manifest.

| Retained object | Bytes | SHA256 |
| --- | ---: | --- |
| `prior_only.cpl2` | 264 | `c020b1e131a615bc3ce2b2cf857d83b7465fe3c7c50002528bd85732cc6b928b` |
| `prior_only.ckpt` | 5,196 | `b90eff2df5898aca35712c8bbc18097fcd924a03d14e17dd43d523fd127519ef` |
| `result.json` | See retained file | `94329a4881323838a999d4f123109639d1ea6b9714461cc3b9bb6ca12ac2b807` |

| Learned checkpoint | Accuracy, all 16 fitting contexts | Mean bits/target |
| --- | ---: | ---: |
| 0 | 0.2500 | 2.0000 |
| 128 | 0.9375 | 1.3546 |
| 512 | 1.0000 | 0.2107 |
| 2,000 | 1.0000 | 0.04973758 |
| Final artifact, permuted context/target association | 0.4375 | 4.78358068 |

The permutation penalty is **+4.73384310 bits/target**. Context-disabled accuracy/CE remain 0.25/2, and both single-position knockout accuracies remain 0.25. The separately authored capacity witness yields 16/16 and approximately 0.0772 bits, consistent with `log2(1+3 exp(-4))`. The learned arm is separate from this answer-coded witness. These are all-pairs fitting and instrument results, not generalization, grokking, language or geometric advantage.

The repaired loss and declared effective-weight STE are source-consistent on the active default configuration: `dL/dZ=(p-onehot)*2^-F/ln(2)`, readout-master credit is this times integer hidden activation, hidden credit uses the effective shifted output weights, and embedding credit applies the ReLU mask and inverse normalization shift. The batch divides by scored targets once. The old extra Adam threshold is removed and has independent scalar reference tests. This review found no new missing scale factor in that computation.

Two historical interpretations need precision. The old 0.375 result never established a quantization limit; simultaneous task/forward/optimizer repairs and the new pass do not isolate each defect's causal contribution. The unclamped `trace` was an **intermediate #1298 refactor defect** caught by the capacity witness. The merged #1296 serving and training paths already clamped activations; this new bug cannot be retroactively named as a demonstrated cause of the old score. Preserve the raw receipt and read this correction with it.

## Corrections needed inside the next implementation

Line references below refer to the audited merge and are not permanent API identifiers.

1. **Mid-pass resume is not exact.** `checkpoint_bytes` (891–935) omits `perm`; `resume_from` clears it (1052). `next_batch` (865–870) then reshuffles from the advanced RNG and resets cursor zero. It repeats/omits exposure after a mid-pass restore. The existing test saves at a pass boundary, where next batches happen to agree; it also misses a pass-counter difference. Persist the permutation or a sufficient reconstruction state and test mid-pass, partial-batch and cross-pass continuation with complete state and consumed IDs. The loader discards `_seed` and the toy uses zero data identity. Bind real experiment/source/tokenizer/data identities and validate the expected seed/bias/config before use.

2. **Integer scores do not establish an integer-only served call graph.** `int_logits` calls `trace`; `trace` allocates `f64` masks and computes `dec=1.0/(1u64<<sh) as f64` (225–247). Whether a particular optimized binary eliminates these unused auxiliary operations was not measured. Store the activation mask as bool/u8 and the normalization shift as an integer in the one shared trace; convert only in offline backpropagation. Keep greedy serving free of floating diagnostics without restoring two predictive definitions. Allocation freedom and whole-path instruction compliance remain unqualified until actually checked.

3. **The twelve tests do not cover every claimed prerequisite.** The parity test shares a correct forward but does not independently assert the embedding/readout STE derivatives or demonstrate each advertised active normalization/clamp condition. The resume test compares selected fields, not the complete state. Add one hand-computed gradient fixture and actual full-state continuation comparisons. No new 2,000-step research arm is needed solely to repeat the positive result.

4. **Finish the active input envelope with a small explicit fix.** `norm_bits=0` disables normalization while validation assumes a unit hidden bound; loaded embedding shifts of 30 can overflow their sum before clamping; the constructed core bypasses loader checks. Restrict unused configurations or use a checked wider embedding sum and derive the actual bound. Validate the constructed and reloaded model consistently. Reject nonfinite/invalid optimizer fields, negative second moments, invalid bias codes and mismatched identities before mutating a trainer. This is not a general hostile-file hardening campaign.

5. **Do not copy the toy's count initialization.** The gate initializes counts to one, keeps `total=0`, then calls a helper that adds one again. Its balanced classes still produce a harmless flat bias; real text would not. Start raw counts at zero, assert `sum(counts)=total`, count the declared training targets and apply smoothing once. The toy's inventory seal also lacks an explicit source/config/mask/permutation identity. Preserve it; supply the complete bindings in the new real-text attempt rather than modifying a sealed root.

## The real-text question and experiment

Use the current two-position exact-token tables, bounded ReLU and ternary readout at V=4096, dv=128, F=10, norm_bits=6, frozen fit-only bias, length 64 and batch 8. Keep one initialization/optimizer configuration. The prefix-derived tokenizer is fixed from an existing tokenizer; it is not trained on the new corpus. Save the derived bytes and both identities. Use whole-document fit/tune/development partitions, exact-content duplicate groups, pinned source snapshot and windows spread across document lengths/positions. Repository documentation is a scoped technical-prose corpus, not instructions, broad web text or a clean independent benchmark.

Checkpoints 0/256/512 form the primary curve. 512 full batches contain 258,048 scored targets, not 262,144 predictions. Use actual counts for partial windows/batches and report distinct windows and token-row exposure. Measure complete new-path cost before admitting the run; the older 0.5649 s measurement belongs to another trainer and is not this runner's timing.

The constant quantized bias is necessary but insufficient: a learned residual could merely repair marginal quantization error. Therefore compare with the **exact smoothed fit-only unigram** as well, and evaluate the same artifact with complete context pairs permuted against fixed targets. Prefer within-document permutations, keeping PAD/non-PAD strata separate, so document-topic information is retained while local alignment changes. Freeze/save the actual permutation, verify changed associations and report non-permutable strata. Position knockouts are diagnostics with potentially out-of-distribution inputs, not substitutes for this control.

The predetermined step-512 development gate remains at least 0.10 bits/target better than both exact unigram and identical quantized bias, with paired document-resampling intervals excluding zero, plus a positive permutation penalty whose interval excludes zero. Report token-micro and document-macro separately; document-cluster bootstrap intervals are nominal on correlated repository documents and are not final held-out qualification. Corrected tune-selected backoff references show local statistical headroom, not Bayes ceilings and not the primary learning gate. Include consumed-exposure and full-fit labels and unseen-fit pairs. Preserve raw greedy continuations, including collapse, from fixed prompts.

If this dose fails but training improves, it does not prove an architectural limit. One predeclared conditional continuation, at most step 1,024 or one shuffled pass, can distinguish an early learning curve from a flat failure. Its result is exploratory; do not retroactively move the step-512 gate. A fixed-artifact regression or a flat exported curve should instead trigger a targeted data/credit/scale diagnosis. No width/seed sweep, memory arm or new family is needed in this run.

## Roadmap and mathematical role of geometry

The dependency order is now **small contextual fitting established → representative exported real-text baseline → older-prefix-sensitive geometric state/read → exact-memory integration and useful conversation/coding → complete M1 cost and product qualification**. Local-context learning is a bounded component experiment, not a replacement goal or an endless trigram scaling programme. Replication and wider data belong to qualification before promotion, not an automatic second arm in this pilot.

A two-token prior satisfies `P(next|history)=P(next|last two tokens)`. Identical tails therefore force identical predictions; on a population, the optimal benefit from older history is `I(next;older history|tail)`. This does not make a fitted count score an information-theoretic ceiling. The stored modulo-120 element table is unused by the current predictor.

After a usable baseline or a specifically diagnosed representational limitation, the best motivated geometric candidate is **one explicitly learned ordered prefix-state/read channel**. Specify state capacity, token-conditioned action, reset/write semantics and how its selected read affects the existing decoder. Reuse actual finite 2I/H4 transport and retain orientation; do not substitute arbitrary hash magnitude for semantic distance. A group register `g_t=g_(t-1)*a(x_t)` can distinguish order when the chosen actions do not commute, but 120 states carry at most `log2(120)≈6.91` bits. It is a lossy summary, not exact memory. The golden/Galois companion is derived structure, not an independent capacity multiplier.

Test equal-tail/different-prefix pairs, disabled/reset state and same-multiset changed order, followed by real-text improvement against a capacity/cost-matched simpler state or longer local context. This separates receiving more information from a geometric advantage. Reversible transport also needs explicit forgetting/overwrite semantics; noncommutativity alone does not learn relevance. Keep occurrence/version-addressed memory separate from coalescing modulo value buckets. These are recommendations for the subsequent mechanism decision, not authorization to combine all of them into the current pilot.

## Research refresh and applicability

The existing [mathematical audit](architecture-2026-09/mathematics.md), [import audit](architecture-2026-09/imports.md) and [previous optimization review](learning-contract-review-2026-09-19.md) remain the source map. Targeted primary-source refresh found no reason to replace a learner that has just passed its corrected small task:

- [Bengio et al., A Neural Probabilistic Language Model](https://www.jmlr.org/papers/volume3/bengio03a/bengio03a.pdf) supports shared token representations and learned local context as a meaningful baseline beyond memorized n-grams. Its floating model and data results do not certify this ternary model.
- [Zhu et al., Scalable MatMul-free Language Modeling, revised July 2025](https://arxiv.org/abs/2406.02528) supports studying quantized parameter movement and recurrent alternatives. Its model, training scale, elementwise operations and hardware results are not this project's integer-only serving contract or M1 energy evidence.
- [Grazzi et al., Unlocking State-Tracking in Linear RNNs Through Negative Eigenvalues, revised March 2025](https://arxiv.org/abs/2411.12537) studies restricted finite-precision transition classes and shows benefits of signed/state-tracking transitions. Its assumptions do not establish an impossibility for arbitrary nonlinear or addressed-memory systems.
- [Yang et al., PaTH Attention, revised February 2026](https://arxiv.org/abs/2505.16381) gives a concrete precedent for input-dependent ordered accumulated transformations. Its Householder/attention machinery is not directly a D0-b kernel; finite-group transport is a UOR hypothesis requiring measurement.
- [Cheng et al., Engram, revised July 2026](https://arxiv.org/abs/2601.07372) treats static local lookup as complementary to dynamic contextual computation. Its Transformer/MoE backbone, gates and scale do not establish a stand-alone lookup language model or justify adopting them here.

This is a decision-focused review of available primary sources, not a claim of exhaustive knowledge of all mathematics or research.

## Resources and delivery

Live recorded ledger is **151,038,565 / 154,400,000 ms**, leaving **3,361,435 ms (56.02 minutes)**. Preserve the 1,200,000 ms charge as recorded; this review did not independently reconstruct its roughly described build durations. No charge or limit change was made. About 43.4 GiB filesystem free was observed; this is not the complete storage inventory.

The next prompt proposes a complete 3,000,000 ms tranche including repairs/builds, preparation, training, evaluation/generation and reserve. Refresh the storage accounting and record the projection before execution. Necessary local extensions are already authorized when their reason, increment and new limit are recorded before use. A session/model-token stop is distinct from exhausting this local wall-time ledger. Preserve the 128 MiB margin, owner checkout, negative candidates and sealed roots; no deletion or paid compute.
