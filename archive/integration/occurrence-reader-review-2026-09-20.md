# Principal review — preserve the local gain and learn an exact-occurrence read

Date: 2026-09-20. Reviewed delivery: [PR #1312](https://github.com/UOR-Foundation/uor-r4/pull/1312), merged as `0bca39b541e0ee56cc58bfff65a625a7854d5433` at 18:41:07 UTC. Its head `0b7703842116473d21a2365971694ee2e216612a` and merge have identical tree `482a10a49d4046a817622b0c4396534e2f271c92`. DeepSeek's implementation and report were already committed; this review corrects interpretation and defines the successor. The [original result](s-attribution-result-2026-09-20.md), [executed prompt](deepseek-separable-attribution-step-2026-09-20.md) and all sealed attempts remain evidence.

**Decision: build one bounded BPE exact-occurrence reader over the frozen local scorer `E + u(b)`.** Retain S and its corrected artifact, but do not spend another run refining this attribution panel or training a reset-only 120-state summary. The [next execution prompt](deepseek-occurrence-reader-step-2026-09-20.md) gives DeepSeek discretion over a small geometric selector, fitting procedure and useful diagnostics while requiring causal source access, bounded serving work and honest controls.

## What the experiment establishes

The latest run performed no optimizer updates. Its canonical retained root is `.uor-models/realtext-prior-2026-09-20/s-attribution-3`; `result.json` has SHA256 `994a1a15080ecc0c524aaa6ae6d93428c01f6fe4c5fd4d18ee80394d32a92165`. It reports exact reproduction of all sixteen old loss vectors and the six retained S generations. The metadata repair changes only the tokenizer field, preserving numerical parameters. The inference seam removes duplicate history folding and the local arm's unnecessary older fold. These are useful repairs.

| Frozen condition | Old panel, bits/target | New positions, bits/target |
| --- | ---: | ---: |
| E | 7.170816 | 7.048239 |
| S11: E + history row + query row | 7.032228 | 6.919775 |
| Qonly: E + query row | **7.022581** | **6.912660** |
| Honly: E + history row | 7.171048 | 7.045576 |

On matched older-present targets, Qonly minus S11 is −0.009978 bits on the old panel and −0.007348 on the new panel; their paired intervals exclude zero. The new interval, [−0.012050, −0.002327], is not contained in the declared ±0.01 equivalence band. We can retain an aggregate numerical improvement without declaring equivalence or rejecting a promising result merely for missing a practical threshold. M01, the offline fit-average history comparator, beats S11 by 0.023622 and 0.018695 bits respectively. The new primary interval is [−0.022344, −0.015019]. Its construction has the small population defect described below.

The old panel has 36 documents, 288 windows and 17,342 targets; the new panel has 35 documents, 277 windows and 17,451 targets. These are different positions in the same open-development documents, not independent-document qualification. Donor support is much smaller than the full panel. The preserved exact-tail donor findings do not establish a beneficial individual-history effect, and all retained generation conditions remain repetitive.

The justified architectural statement is **no demonstrated net predictive benefit from this frozen individual-history branch at this training dose and support**. The local query branch is useful on these panels. That is sufficient to move toward a different memory mechanism. It does not prove that the state contains no information, that all subpopulations are harmed, or that geometric memory is intrinsically ineffective.

Nor does M01 prove that length dependence is the useful ingredient. There is no length-independent fit-mean comparator. Averaging can remove harmful variation or supply a generic calibration offset; cross-entropy is convex in logits, so averaging unhelpful logit variation can improve loss without identifying a length mechanism. The donor preserves local context and length, but its null result does not identify the unique reason for silence. We do not need another panel campaign to resolve that secondary attribution question before building useful contextual access.

## Corrections and their scope

**Cost is hot evaluator timing, not direct serving latency.** In `bin/query-read-attribution.rs`, lines 322–323 precompute all 120 × 4,096 residual logit rows. Lines 334–339 memoize complete E logits in an unbounded hash map; lines 1338–1344 reuse those entries after warm-up. This differs from `QueryHard::int_logits` in `learner/query_read.rs:535`, which executes the parent and ternary residual computation. The report's “complete numerical path” and “no fit-time cache” language is therefore inaccurate.

The saved 16,390 parent-cache entries contain **268,533,760 bytes** of i32 logits before container overhead. Each precomputed residual table contains **1,966,080 bytes**. Reporting the 454,788-byte parent and 53,555-byte CPX3 artifact does not account for this working set. Preserve the recorded 0.530–0.800 ms timings as memoized evaluator measurements. A bounded compiled table may be a legitimate future implementation, but requires explicit artifact/construction/working-set costs and an actual served path. The next reader should measure its declared direct or explicitly bounded compiled path; no rerun of all attribution losses is needed. Physical energy remains unavailable.

**M01 includes terminal states outside the training-target population.** Lines 869–879 iterate `2..w.len()`, while the shared target iterator in `prior_learning.rs:167–189` predicts only through position `n−2`. Saved counts sum to 253,017; the 4,096 consumed windows contain 248,921 older-present prediction targets (`257113 − 2×4096`). There is one extra terminal state per window. Of the 4,096 extras, 4,062 appear at length 62, unused by these evaluation panels; **34 affect the evaluated lengths 1–61**. This is a scoped estimator-population deviation, not target leakage. Preserve M01's reported numbers with this qualification. A correction, if needed for a numerical claim, should derive only the corrected means and affected comparator from pinned inputs in a new report root. The next mechanism does not depend on M01 being a serving candidate.

**The automatic label is unsuitable for reuse.** Lines 1382–1385 call a point within ±0.01 and an interval crossing zero “equivalence”; equivalence requires the whole paired interval inside the declared band. The old-panel harm check at line 1381 uses the lower endpoint below zero instead of the upper endpoint below zero. Neither rescues the failed history-lead condition here because the new primary effect is strongly negative. Correct these predicates when the evaluator is next touched; report the evidence-supported architectural decision separately from the sealed label.

**Provenance and interface claims need precise scope.** The executing binary and two changed source files are hashed, an improvement over the prior run. However, `utc_start` is sampled during report writing at line 1411, and phase marks stop before the main evaluation. These are not complete start/phase receipts. The production tokenizer loader rejects the placeholder; the public legacy-import helper relies on caller-side hash checks rather than enforcing its own hash allowlist. The actual runner performs those checks. The checked row API validates token IDs but returns `Ok(None)` for an invalid index; supplied older-state indices remain an internal precondition. Valid pinned inputs make these interface omissions nonblocking for this result.

M01 intervention invariance was not explicitly instrumented, and the selection manifest was written before scoring but sealed with the whole root afterward. These are disclosed scope omissions. They do not justify repeating the entire experiment. Preserve the successful source/metadata parity checks, exact logits, saved vectors and negative generations rather than converting every reporting correction into another full run.

## Why an occurrence reader is the coherent successor

A single 120-state register contains at most `log2(120) ≈ 6.907` bits. Group translation is bijective: distinct states remain distinct under the same future translation. Reset introduces noninvertibility but does not increase state capacity or retain arbitrary source payloads. A four-bit Continue/ResetTo mechanism remains a plausible maintenance primitive. Calling it “addressability” would conflate forgetting with retrieval.

We should separate **geometric selection metadata** from **exact retained payload and occurrence identity**. The next reader should keep a bounded ring, admit a bounded set of causal source occurrences, select with one shared learned geometric rule including NoRead, then use the selected exact token through a sparse integer emission contribution. Freeze E and the S query component; bind the resulting local condition as a declared baseline rather than silently promoting an ablation format. Predict before observing the target. Equal token values must not collapse distinct source occurrences, and stale slot references must be rejected by sequence identity.

This reuses established project work instead of opening a disconnected model path:

| Existing source | Reusable contract |
| --- | --- |
| `native_geometric/memory_types.rs` | Bounded memory configuration, exact references and occurrence schemas |
| `native_geometric/memory_runtime.rs:63–118` | Causal token observation, sequence/slot retention and successor indexing |
| `memory_runtime.rs:190–201` | Stale and overwritten reference rejection |
| `memory_runtime.rs:214–237` | Transported local source/query paths, excluding unrelated intervening text |
| `native_geometric/source_routing.rs:38–101` | Shared geometric encoding and relative angular/equality scoring |
| `source_routing.rs:164–202` | Candidate selection separated from exact payload access and causal commit |
| `native_geometric/role_read.rs` | No-source/read choice and committed occurrence identity |

The historical word/role path has different tokenization and authored behavior. Reuse its invariants or extract a small shared primitive with an explicit BPE adapter; do not imply whole-model compatibility merely because both use H4. A successful exact-copy panel is a necessary instrument check, not evidence of general prose, reasoning or instruction following. Include competing sources, changed source values, absent/evicted sources and read-disabled controls, plus actual generated continuations and a bounded real-text read probe. Report admission coverage separately from selection and emission success.

## External research and practical freedom

[Sparse Delta Memory](https://arxiv.org/abs/2607.07386), July 2026, scales recurrent state through sparse reads and writes to explicit memory and reports better recall under controlled compute. It supports separating memory capacity from work per token. Its Gated DeltaNet basis and numerical operations are not imported as a compliant UOR runtime.

[Engram](https://arxiv.org/abs/2601.07372), revised July 2026, investigates deterministic lookup for local patterns alongside neural computation. It supports preserving inexpensive local prediction while allocating contextual work elsewhere. Its transformer/MoE experiments do not establish a transformerless solution or prove that our lookup tables fit processor cache.

[Zoology](https://arxiv.org/abs/2312.04927) connects real-language recall deficits with multi-query associative recall and distinguishes easy synthetic success from useful contextual retrieval. It motivates multiple competing bindings and natural-text evaluation. Its reported model comparisons are not measurements of this project.

[PD-SSM](https://arxiv.org/abs/2509.22284) studies structured sparse transitions capable of finite-state tracking. It motivates richer noninvertible updates, but its column-one-hot/complex-diagonal construction and state/readout assumptions do not transfer an arbitrary-FSA guarantee to a 120-state Continue/ResetTo scheme.

These sources motivate a testable design choice, not a theorem that it will succeed. DeepSeek may adjust the small selector and fitting plan based on source inspection, measured admission coverage and bounded diagnostics. It should record why an adjustment improves the experiment, retain honest negative outcomes and avoid optimizing the test labels. Practical margins guide allocation; an informative subthreshold result can justify a targeted follow-up. Causality, identity, held-out separation and truthful cost accounting remain essential.

Full CPQK continuation is required before resuming or refitting the QueryTrainer path. It is not a prerequisite for unrelated work solely because that checkpoint exists. The actual next learner must have an appropriately complete continuation boundary if its planned fit can require resume; a bounded exact/discrete fitter may need a much smaller state. Do not spend the successor repairing an unused training framework.

## Delivery, storage and resource state

This review ran no model forwards, training or Rust builds; its numerical checks reaggregated saved evidence. The live cumulative ledger is **178,538,565 / 180,500,000 ms**, leaving **1,961,435 ms** at this snapshot. The next prompt requires a complete refreshed projection and any standing-authorized extension recorded before use. It does not silently consume or reset allowance.

The separate [storage cleanup record](storage-cleanup-2026-09-20.md) owns actual disk measurements and removed-path receipts. Freeing regenerable build/cache data must preserve research, sealed reports, models, source freezes and active tool state. Issues #973/#820/#963/#964 remain open: bounded attribution and artifact repair do not complete conversation, coding, generalization or energy acceptance. The roadmap now advances one concrete learned contextual read while retaining the stronger local baseline and the historical exact-memory path.
