# Review after PR #1300: recover the trained artifact's evidence before another fit

Audited merge: `74fef0886ca3b14ff90943c8677d6b7815064198`, September 20, 2026 UTC. This review inspected source, retained files, archive identities and primary research, and performed arithmetic on saved losses. No Rust build, model forward or training was executed. The [next comprehensive prompt](deepseek-frozen-prior-step-2026-09-20.md) carries the implementation and frozen-artifact evaluation.

## Decision

**Keep the trained step-512 prior and replay its evaluation after repairing occurrence identity, document aggregation and population accounting. Do not repeat training merely to fix the verdict sign or impose a different 4 MiB corpus.** A smaller refit would change the experiment; it cannot validate the retained result. The exact corpus snapshot and final artifact are available, so the cheapest decisive action is evaluation-only recovery, missing statistical references and a frozen generation-loop diagnostic.

The loss reduction is a meaningful positive observation. The advertised complete gate PASS is not supported: its permutation changes target frequencies and its purported document bootstrap resamples windows. Correcting the sign alone is insufficient. These are evaluation defects, not evidence that the learned parameters failed or that quantization/geometry is impossible.

## Verified delivery, artifacts and actual population

PR [#1300](https://github.com/UOR-Foundation/uor-r4/pull/1300) is merged. Head `00957c4f870b275f87f090e570ef5f5c7000879d` and merge share tree `d34505e5023af907d99c68244614e1e54e5d0a06`. Owner checkout is clean. #973/#820/#963/#964 remain open with no linked project items.

Retained root: `/Users/casey.allard/uor-r4/.uor-models/realtext-prior-2026-09-20/attempt-1`. All seven manifest members match their BLAKE3 digests and byte counts; there are zero unlisted files. File integrity does not certify the evaluator's semantics.

| Object | Bytes | SHA256 |
| --- | ---: | --- |
| Final `prior_realtext.cpl2` | 454,788 | `cd5a3aa1b804ccc3582a4a25090e25c862489e0bc4e5bb86c1d1f363a7338a00` |
| Final `prior_realtext.ckpt` | 19,192,104 | `1c3dfedb2248ec76c21f320b7ecec585c7a522a2d47e35ed5dc9eaca2454bff2` |
| Derived tokenizer, V4096 | 94,330 | `a7ac75b68aa997fe7cc338d25d843157f2dc9d4265fb2f958ee779bb04828d6f` |

The sibling `inputs/docs` archive contains 850 files / 88,016,240 bytes, all independently matching the Git blobs at corpus commit `e9c04e80`, with no missing/extra/mismatched files. The runner accepts only UTF-8 Markdown: **429 documents / 7,609,837 bytes (7.26 MiB)**, not an 86 MB training-text population. Its hash buckets contain 355 fit docs / 6,385,331 bytes, 38 unused tune docs / 641,183 bytes and 36 development-pool docs / 583,323 bytes. The 4 MiB selection constraint was exceeded; the larger archive size describes materialization/storage, not model input.

The fit pool has 38,987 windows / 2,446,208 scored targets. That is **available fit data and the frozen-bias exposure**, not optimizer-consumed targets. The checkpoint records step 512, cursor 4096, pass 0 and a valid 38,987-entry permutation; its first 4,096 window IDs are distinct (10.51% of fit windows). Exact consumed targets require summing those windows' actual n−1 counts, bounded above by 258,048. The four timing-probe updates belong to a discarded trainer and are separate charged work.

The evaluator actually scores **32 documents × three opening windows = 96 windows / 6,048 targets**. It selects 36 IDs, takes three opening windows each, then globally truncates at 96; the last four documents never contribute. Their exclusion is deterministic, not stratification across positions. Token offsets are 0/64/128 for the retained windows. The next runner must verify these reconstructed identities against the original code and archive, then persist them.

Only the final artifact/checkpoint were saved. Steps 0/256 are recorded in-memory quantized-core observations, not reloaded artifact evaluations. The result stores aggregate window losses, not full integer logits. Do not claim saved intermediate models or scores that are absent. The source-revision string is the corpus base, not a complete binding to the dirty training implementation/binary.

## What remains positive and what is withdrawn

| Checkpoint | Recorded unpermuted micro bits/target | Frozen quantized constant | Exact full-fit unigram |
| --- | ---: | ---: | ---: |
| 0 | 9.13163 | 9.13163 | 9.0599 |
| 256 | 8.2182 | 9.13163 | 9.0599 |
| 512 | 7.14247 | 9.13163 | 9.0599 |

The sign correction gives mean improvements of 1.9174 bits over exact unigram and 1.98915761 over the quantized constant. Independently regrouping saved step0/512 window sums into 32 documents, each 189 targets, and applying the same 2,000-draw bootstrap seed `0x9E3779B9` gives a nominal document interval **[1.84387043,2.12714975]** for the latter gain. This read-only arithmetic supports the narrow positive loss result. It does not repair the missing permutation evidence or make correlated repository documents an independent benchmark. Exact-unigram per-document losses were not saved, so their corrected interval still requires replay.

The following claims are withdrawn pending corrected evaluation: the complete primary gate PASS, the 2.8744-bit marginal-preserving permutation penalty, its confidence interval, the 5,805 applied-association count, 36 evaluated documents, and three distinct generation documents. Original results and receipts remain preserved.

## Source-derived evaluation defects

References below use the audited source `crates/uor-r4-core/src/bin/prior-learning-realtext.rs`.

1. **Window rows are labeled document rows.** `evaluate` 150–174 pushes a loss/count pair once per window. `paired_interval` 179–217 resamples those 96 rows. `doc_macro` is consequently window-macro. Aggregate by actual document identity first, then bootstrap aligned document sums/counts. The macro number happens to equal true document-macro here because each actual document contributes the same 189 targets; the implementation is still wrong for unequal document weights.

2. **The permutation collapses occurrences.** Lines 416–453 shuffle targets within each window, then store replacements in `HashMap<(doc,prev,cur),target>`. Repeated contexts overwrite earlier occurrences within/across windows. Evaluation substitutes the last replacement target for every matching context. This is neither the requested fixed-target context permutation nor an occurrence-preserving target permutation. Counts of changed assignments are computed before overwrite and do not describe the applied control.

   An immediate falsifier is already in the report: step 0 is a context-independent predictor, yet its permuted CE is 9.1364 instead of 9.13163. If targets stay fixed, any context permutation leaves a constant predictor's loss exactly unchanged, apart from numerical summation tolerance. Use observation keys `(document identity, window token offset, target offset)` and a saved bijection over occurrence indices within document/PAD strata. Prediction caches may share identical contexts; observations must retain multiplicity.

3. **Sampling does not match the claim.** Lines 327–335 take each document's first few windows and truncate the combined list. No long-document interior sampling occurs. At 579 the first three windows become generation prompts; all nine saved generation rows have `doc_slot=0`. These are three prefixes of one document. Recover the exact original panel first, then evaluate a separately named panel spread through document lengths/positions and three genuinely distinct documents.

4. **Provenance and persistence are incomplete.** Lines 411–412 hash tokenizer digests plus a caller-supplied corpus-revision string; no actual window manifest, configuration or training-source identity enters it. Lines 624–628 save only the final model/checkpoint, and evaluation used in-memory cores. The CPL2 field named tokenizer digest actually holds this composite. Treat that as a legacy field with explicit interpretation; do not falsely validate it against the derived-tokenizer SHA. Bind the recovered input manifest, old artifact and new evaluator externally in the new report.

5. **Missing references and diagnostics remain missing.** No interpolated backoff, consumed-exposure or unseen-pair reference ran. The gate omitted the predeclared support checks. File-write errors for several essential artifacts/tokenizers are ignored. Require successful writes and a complete expected-file set before sealing; no new reporting framework is needed.

## Numerical repairs and remaining scoped corrections

The integer-only trace and ordinary CPCK v3 permutation restoration are implemented. The declared effective-weight STE remains source-consistent; this audit found no new forward/gradient scale defect in the active default path. The fitting improvement should not be reclassified as failure merely because evaluation needs repair.

Two cheap source corrections can accompany the evaluator. `prior_learning.rs:245–246` expresses widened ternary scaling using integer multiplication by powers of two; use signed shifts instead, preserving values. Compiler elimination of multipliers has not been measured. `core_view:738–748` still bypasses `PriorCore::validate`, contrary to the receipt; call shared validation on construction as well as loading. Check exact retained-artifact score/generation parity after these changes. They do not justify retraining.

The gradient fixture is useful but derives its reference from the implementation's trace, uses a 40-step learned fixture and a loose absolute tolerance, and does not independently assert upper-clamp saturation. No corresponding derivative defect was found. A literal fixture with known intermediates and meaningful coordinate-specific tolerances belongs with subsequent gradient changes, not as a new research campaign before frozen evaluation.

Checkpoint v3 still discards seed, accepts unspecified expected identity, and lacks cursor/frozen-bias identity checks. Preserve it and recover exposure from its serialized permutation. Frozen evaluation does not need an optimizer update or a resume test. Before a future training continuation, bind the actual reconstructed manifest and complete identity, then test on-disk continuation. Do not describe that future obligation as already complete.

## Generation, geometry and the next mechanism decision

The retained final samples enter token-ID 32 repetition; earlier samples also repeat 198. For this stateless two-token predictor, greedy generation is a deterministic transition `T(a,b)=(b,argmax Z(a,b))`. On at most V² pair states, a run that never terminates eventually cycles. A repeated z is a fixed point precisely when `argmax Z(z,z)=z`. This proves eventual cycling, not why the observed trajectories enter a short cycle, nor that all prompts collapse. Any bounded deterministic state machine has a related eventual-cycle property; adding geometry alone cannot abolish it.

Inspect reached pairs, exact top scores/ties, margins and fit/consumed-fit successor counts before changing the decoder or objective. Compare the same prompts with the corrected count references. If the model's loop preference is unsupported by data, optimization/calibration or quantization is a plausible seam; if references share it, local information and greedy selection deserve attention. Neither conclusion follows solely from a repeated string. No repetition penalty, token blacklist, temperature change or output rewrite should disguise this diagnostic.

The geometric roadmap remains explicit: recover a reliable frozen local baseline; then test **one learned ordered prefix-state/read channel** against same-tail/different-prefix and matched capacity/cost controls, followed by exact occurrence/version-memory integration and common-artifact conversation/coding. A local prior cannot distinguish identical tails; a 120-state root carries at most 6.91 bits. Exact finite group-ID transitions avoid continuous rounding drift, but not wrong learned actions, finite-state collisions or poor credit assignment. Preserve prime/zeta/R4/H4 orientation and the derived golden/Galois companion; noncommutativity is an operator property, not a language result. The memory path must remain distinct from coalescing modulo buckets.

After this one corrected replay, choose the mechanism from the measured reference/collapse gap; do not make further prior retraining, broad audits or perfect fluency an automatic prerequisite to a bounded state experiment. A corrected negative also informs that choice. No new state or decoder is fitted in the current next task.

## Targeted research refresh

- [Holtzman et al., Neural Text Degeneration](https://arxiv.org/abs/1904.09751) shows that likelihood-based prediction and useful decoding can diverge. Its sampling result does not diagnose this model or establish a D0-b sampling kernel.
- [He et al., Exposure Bias versus Self-Recovery](https://aclanthology.org/2021.emnlp-main.415/) measures prefix perturbations and recovery in its models. It cautions against labeling every repetition trace an exposure-bias failure without a causal test.
- [Shakerinava et al., Diagonal SSM Expressive Limits, revised August 2026](https://arxiv.org/abs/2603.01959) characterizes group tracking in a specified finite-precision complex-diagonal SSM class and separates representability from learnability. It motivates preserving noncommuting transition options; it is not an impossibility theorem about UOR or arbitrary recurrent systems.
- [Chung et al., Error Control Dynamics, May 2026](https://arxiv.org/abs/2605.07755) separates symbolic representability from robust state separation under affine recurrence. Its assumptions make it a warning to measure drift/error correction, not proof that exact discrete lookup learns the correct transitions.
- [Li, RiLM, September 9, 2026](https://arxiv.org/html/2609.10305v1) is relevant new research on tied distance decoding, but is **research-only** here. The inspected design uses dense composition and floating hyperbolic operations with O(Vd) vocabulary scoring, and supplies no low-bit/M1 result. In flat space, `-||h-e_v||²/τ` equals `(2 e_v·h-||e_v||²)/τ` up to a shared logit offset: it is a tied affine decoder, not elimination of a contraction. Its stated dense d×2d composition is O(d²), despite an O(d) complexity description. The reported hyperbolic advantage is dataset/vocabulary dependent. Queue a matched decoder comparison only after a measured readout bottleneck and explicit integer lowering; do not replace the current model from its small-vocabulary perplexity headline.

The [existing mathematical audit](architecture-2026-09/mathematics.md) still supplies actual finite transport/identity mechanisms. This is a decision-focused source review, not an exhaustive claim about all available mathematics.

## Resources

Live recorded ledger: **152,238,565 / 154,400,000 ms**, remaining **2,161,435 ms (36.02 minutes)**. Preserve the latest 1,200,000 ms charge as recorded; approximate build-cycle timing was not independently reconstructed. This review made no model charge or limit change. The next prompt proposes 1,800,000 ms for focused evaluator work, input recovery, frozen scoring/references, loop diagnostics and delivery reserve. Record actual projection/storage before use; necessary local extensions remain owner-authorized when documented before consumption. Preserve the 128 MiB margin, unique artifacts, sealed roots and owner's checkout; no deletion or paid compute.
