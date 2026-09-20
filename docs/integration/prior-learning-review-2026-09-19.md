# Prior-learning review after PR #1294

Date: September 19, 2026. Audited source: `c0ca7482782a5d5e11d25aa8c6d5b03f0ce4cd07`. This is a source, artifact-identity and primary-literature review; no model was executed. The [next DeepSeek prompt](deepseek-prior-learning-step-2026-09-19.md) specifies the corrective implementation and conditional learning run. The [previous review](cold-context-review-2026-09-19.md) retains the architectural motivation.

## Decision

Do not launch an unchanged multi-epoch run. Keep the implemented exact-token prior and causal-memory architecture, but first repair the numerical/data experiment. Then train **one prior-only model as a contextual residual above a frozen, fit-only quantized unigram bias**, with representative deterministic exposure and a saved learning curve. Memory remains disabled during this fit; diagnose its existing artifact without retraining it. Extra dose is conditional on demonstrated correct learning, not inferred from a token/parameter ratio.

The constant bias supplies an explicit statistical baseline, not geometric intelligence. Holding it fixed provides a controlled starting point. The learned residual can still make a context-independent marginal correction; fixed context-permutation and position interventions must establish useful contextual association on the exported integer computation. Geometric routing/state/transport remains the programme objective. This local prior is not a replacement for long-context state, exact provenance memory or learned geometric composition.

## Verified delivery and preserved result

PR [#1294](https://github.com/UOR-Foundation/uor-r4/pull/1294) is merged. Reviewed head `41213a558a12562406d2dca72e30a970da6b1a02` and merge `c0ca7482` have identical tree `f93707884f4233a30c93f757dae22f2b8b3580e3`.

The retained joint artifact is 614,542 bytes, SHA256 `fb780ff6ff19eec58f861135004250ae8874107099e2a071f7bfd1b4318b5a39`, verified at both `/Users/casey.allard/uor-r4/.uor-models/cold-prior-2026-09-19/cold_prior_joint.cpr` and `/tmp/cp6/cold_prior_joint.cpr`. Both directories contain only this artifact. A durable data manifest, raw per-target results, prior-only artifact and optimizer checkpoint were not delivered there; the runner writes none of these. Other raw logs, if recovered, must be identified separately.

The implementation does contain full-ID position tables, an absent-prefix row, bounded ReLU, separate channel downshifts, a shared ternary decoder and a causal modulo-pair memory. The reverse memory credit appears correctly ordered for the scored positions. Valid constructed artifacts round-trip exactly. These are useful implementation results. They do not qualify train/serve probability parity, convergence or general language.

## Principal corrections

### The displayed loss is clipped nats, not bits

`cold-prior-pilot.rs::evaluate` and the reference calculations use natural logarithms without division by `ln(2)`. Training and evaluation also use `-ln(max(p_target,1e-9))`, which caps loss at about 20.7233 nats (29.8974 bits), while the gradient remains the ordinary uncapped softmax gradient.

| Recorded label | Actual recorded clipped nats/target | Arithmetic conversion to clipped bits/target |
| --- | ---: | ---: |
| Unigram | 6.4019 | approximately 9.2360 |
| Quantized constant | 6.4522 | approximately 9.3086 |
| Prior-only | 6.7011 | approximately 9.6676 |
| Joint | 6.8941 | approximately 9.9461 |
| Joint with memory disabled | 6.7006 | approximately 9.6669 |
| Warm memory loss increase | 4.4072 | approximately 6.3582 |

These are conversions of rounded historical values, **not new evaluation results** and not uncapped cross-entropies. The failed point-estimate directions survive unit conversion. However, the implementation applied numerical gate constants as nats despite their specification in bits. Replace the loss with stable offline `logsumexp(z)-z[target]`, using the declared fixed-point logit units, and convert explicitly for bits/target. A zero-logit V4096 fixture must report exactly 12 bits; highly wrong logits must not hit an artificial loss ceiling.

### Training and export use different biases

The trainer at `cold_prior.rs:683` uses raw floating master bias; export at line 509 rounds to i32, then serving adds those integers. A disabled-channel fixture with biases `(0.49,-0.49,0)` exposes the mismatch immediately. The current bias is also not restricted to the adopted <=4-bit coefficient convention. Integer storage alone does not establish that bound.

Use one explicitly scaled integer-logit forward definition in training and serving, with a declared STE backward and a bounded low-bit bias representation. A signed <=4-bit coefficient plus bounded power-of-two scale metadata is consistent with the existing low-bit construction; disclose metadata precision and total bytes. Do not silently retain unrestricted i32 learned coefficients. Version the changed artifact convention. Legacy CPR1 can be retained and evaluated under its original unit-logit definition without becoming newly compliant.

### The data schedule does not establish undertraining as the cause

The two arms consume the first 2,048 of 23,559 fit windows in sorted document order. Each arm sees 131,072 input tokens but **126,976 scored targets**, because training/evaluation score target indices 1 through n−2: 62 targets at length 64. The receipt's `1..=61` range is wrong. The last target is excluded even though the forward loop constructs its context; the new experiment should score all 63 next-token targets with one explicit iterator.

Count controls use all fit windows, not the candidate's consumed subset, and include additional positions. Thus the candidate arms share exposure with each other, but their count reference is only target-population matched. Development uses the first six full windows per document: 19 documents belong to the split, but 18 contribute scored windows. Neither the training prefix nor document-opening development slices are representative by construction.

Low dose makes convergence unknown. A parameter count, nonzero gradient or worse-than-unigram score does not prove that more epochs fix the model. Sample a deterministic shuffled fit-window schedule, publish actual consumed IDs and draw development windows across documents rather than only their openings. Maintain both consumed-exposure and full-fit count references, labeled separately.

### Several controls and claims are unsupported

| Source finding | Correction |
| --- | --- |
| Two-token count reference uses `(w[i-2],w[i-1])` to predict `w[i+1]` | Correct context is `(w[i-1],w[i])`. The first-order baseline can use `w[0]` at the first prediction. The old trigram loss cannot diagnose smoothing alone. Reuse the repaired backoff implementation instead of introducing another add-one estimator. |
| Bootstrap uses roughly 2.5th/97.5th percentiles but prints 90% | Approximately a 95% interval for a document-macro statistic. For the token-micro gate, resample document loss sums/counts and recompute their ratio; label the estimand and interval. Training-seed uncertainty remains separate. |
| Warm fraction below 10% is declared insufficient | This threshold was not justified. Use paired uncertainty and counts of contributing documents. Warm/cold masks do match in the current ablation because occupancy writes continue with the memory channel disabled. |
| Memory is said to be harmful because of modulo aliases/random buckets | Harm is an observed clipped-loss difference; its cause is untested. Separate exact-context warm reads from alias-only reads and inspect stream/logit magnitudes and successor conflicts. |
| Mean absolute logit around six is called healthy | Softmax is invariant to adding a common constant. Absolute logit magnitude cannot establish calibration. Inspect centered spread, target margin, entropy, max probability and loss tails. |
| Separate downshift normalization is assumed to balance channels | It only reduces large magnitudes. Prior values near 0–2 can coexist with memory values approaching 63. Measure their separate contributions before attributing harm to addressing or learning capacity. |
| New STE test only requires nonzero gradient; bound fixture does not reach the clamps | Independently verify derivative values with nonunit scales, actual shifts, saturated/unsaturated masks and repeated-write credit. Old geometric-core gradient tests do not cover this new prior implementation. |
| Export/reload matches, so computation is qualified | It proves reconstruction of valid fixtures, not training-forward parity or loader range safety. Validate shapes/shifts/flags/elements, checked size arithmetic, metadata and trailing bytes within the supported format. |
| Report is a frozen manifest and resumable run | Output is created late with overwrite-permitted writes. CPR1 stores no masters, moments, optimizer step or schedule cursor; it discards seed/tokenizer metadata on load. Use existing claimed report roots and real checkpoints. Do not call a restart from quantized weights a faithful resume. |

The context-position interventions, unseen-fit-pair stratum and warm comparison against the separately trained prior were not delivered as specified. Generation was printed as IDs; the committed receipt abbreviates further prompts. Preserve failures, but record full prompt/output IDs and decoded bytes next time.

## Selected recovery experiment

Retain V4096, dv128, two position tables, bounded nonlinearity and shared low-bit readout. Disable memory for learning. Use a frozen quantized fit-unigram bias in both the constant control and contextual model. Prefer an initial **zero exported contextual residual**, so step-zero model probabilities equal the constant control. Keep nonzero context features and verify that the output gradient can move latent output weights across quantization thresholds, after which prior-table credit should appear. Zero output weights initially block feature credit by design; all-zero tables with no learnable path would be a vacuous control.

Define integer scores `Z = B + R(context)` and offline probability logits `z = Z * 2^-F`, with one artifact-bound exponent F. Construct bias from scaled log fit probabilities, `B[r] ≈ 2^F * (ln p_fit[r] + c)`, with declared smoothing/common offset and low-bit quantization. Measure its quantization gap rather than equating it with the exact unigram. New bias coefficients must obey D0-b with explicit scale metadata. Choose the convention once from training-only calibration, then freeze it before development scoring. The same integer Z and exponent must be used in training-forward, export evaluation and greedy serving. Apply the derivative factor `2^-F` once at integer-score credit; subsequent propagation already carries it. Greedy serving can directly take `argmax(Z)` with no scale multiply. Omitting or duplicating this factor creates another credit error.

This is a calibrated starting condition and controlled learning experiment, not a new promise that the model will succeed. A balanced tiny contextual task must first show exported context-dependent learning beyond its constant marginal. If it fails, a small Rust floating-forward comparator for the same architecture can distinguish optimization/quantization from representation; that comparator is offline diagnostic material, never serving.

Then run a prior-only learning curve on representative data, initially 256/512 steps, extending to 1,024 and at most one full shuffled pass only when measured learning and the complete budget permit. At the old timing, 23,559 windows require 2,945 batch-8 steps, about 1,682 seconds (28 minutes) of training alone. New-path timing, evaluation and checkpoint costs must replace this projection. Save both serving artifact and real resumable trainer checkpoint. No joint-memory retraining or automatic multi-epoch successor is included.

## Research refresh

This targeted refresh addresses probability calibration and low-bit optimization. It does not replace the earlier geometric/import review or claim exhaustive coverage of current mathematics.

- [On Calibration of Modern Neural Networks](https://proceedings.mlr.press/v70/guo17a.html) supports directly measuring/calibrating predictive distributions. Temperature scaling is a diagnostic option, not evidence that absolute logit magnitude establishes calibration; floating calibration parameters require an explicit integer/dyadic serving realization here.
- [Learned Step Size Quantization](https://arxiv.org/abs/1902.08153) treats quantization scale and its gradient as substantive optimization choices. It motivates checking scale/credit, not importing unconstrained floating scales or claiming an ImageNet result transfers to UOR-R4.
- [BitNet, JMLR 2025](https://jmlr.org/papers/v26/24-2050.html) is evidence that low-bit language learning can work under an appropriate recipe. Its transformer architecture and training scale are not our mechanism or a convergence guarantee.
- [Continual Quantization-Aware Pre-Training](https://arxiv.org/abs/2502.11895) studies a higher-precision-to-ternary curriculum and optimizer-state retention. If a matched small control exposes a quantization-learning gap, this is a more motivated successor than unlimited unchanged ternary dose. No such gap is measured here yet.
- [Sparse-BitNet, March 2026](https://arxiv.org/abs/2603.05168) studies joint sparsity/quantization. It does not repair a unit error, unquantized bias or biased sampling and is not adopted in this run.
- [CAT-Q, June 2026](https://arxiv.org/abs/2606.26650) operates on pretrained high-precision LLMs. Its calibration-sample efficiency excludes obtaining those pretrained weights; it is not evidence that a native language model can be learned from scratch with that sample count. It remains an optional future declared offline-reference direction.

A geometric comparison becomes useful after the local learner demonstrably conditions on context. Then choose a specific directed group action, multi-component learned packaging or geometric recurrence against matched controls. Neither arithmetic lookup nor a fixed invertible permutation by itself establishes a geometric advantage. Preserve prime identities, orientation and the separate exact-memory path throughout.

## Resources and scope

Recorded JSON: `149038565 / 154400000 ms`, remaining `5361435 ms` (89.36 minutes). This review changed no balance and ran no Rust build/model. Reused the existing clean isolated full worktree on `codex/prior-learning-recovery`; original checkout and unique material preserved. Host free space was approximately 43 GiB, not a complete storage certification.

The prompt proposes a 3,600,000 ms complete recovery tranche, including necessary repairs, bounded diagnostic/replay, prior learning curve, checkpoint/evaluation and reserve. Record it before execution and revise before overruns under standing local-extension authorization. No paid compute, destructive cleanup or silent cap increase. #973 remains active/open and #820 programme/open; neither has linked project-board items at inspection.
