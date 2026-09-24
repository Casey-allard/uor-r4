# Causal continuation: repaired feedback, exact training resume and compiled readout

Session start: September 24, 2026, 00:00:07 UTC (September 23 in New York). Base: `7fb232ad`. This result advances the five principal-continuation workstreams without changing the terminal objective: fully transformerless geometric language modeling with exact addressed evidence and learned geometric operations replacing floating-point matrix multiplication at serving.

## Decision and scope

Retain a **causal-feedback research candidate**, not a general-language release or silent replacement of the previous best prose artifact. Explicit training-only crossed feedback repairs the measured source-fingerprint shortcut on 16/16 source/feedback combinations, including four combinations excluded from gradients, while preserving the historical temporal32/train36/class4/held-out3 panels. The matched ordinary-supervision arm remains at8/16. New-source prediction improves substantially over the parent, but the causal candidate is slightly worse than ordinary broad training on loss. Dialogue and generated Rust still fail.

The sparse compiled read plan reproduces all tested scores and generations and reduces the measured pre-tokenized prompt-plus-generation time by about10.4x on the same models. It is an optional compiled read interface, not a claim that every application caller now uses it or that energy was measured.

[Complete machine evidence](../evidence/causal-continuation-summary-2026-09-24.json) and [verification](../evidence/causal-continuation-verification-2026-09-24.json) accompany this report. Read [the exact handoff](causal-continuation-handoff-2026-09-24.md) before resuming.

## 1. Causal emitted-token learning

The ordinary examples correlate selected evidence with the copied token, so they cannot identify which channel drives the following class word. The new `train_intervened_batch` supplies a **training-only** copied-feedback intervention while keeping selected evidence, typed facts and copy length fixed. It does not grant runtime permission to copy an unowned value. The unchanged default training path and the identical-feedback intervention agree in focused tests.

Both arms start from the same retained warm model and run256 updates. Each update contains three repository prose windows, three new-source prose/code windows, one authored dialogue/code example, two grounded examples and two class-copy examples. Ground/intervention weight is4. Learning rate decays0.01 to0.001; sampling seed20260924. All parameters, optimizer state and order are recorded.

Four opposite-class combinations, `feedback_index == (source_index+2)%4`, are withheld from the sampled crossed gradient examples. The16-case panel is nevertheless used as an exposed selection gate; its four withheld cells are **gradient-withheld, not an untouched final acceptance population**. Historical grounded held-out cases are likewise now regression data.

| Candidate | Repo development bits/target | Crossed feedback | Gradient-withheld cells | Temporal / train / class / historical held-out |
| --- | ---: | ---: | ---: | --- |
| Retained warm parent |6.273706960|8/16|0/4|32/32,36/36,4/4,3/3|
| Ordinary broader training |6.325266266|8/16|0/4|32/32,36/36,4/4,3/3|
| Crossed-feedback broader training |6.343260311|**16/16**|**4/4**|32/32,36/36,4/4,3/3|

The interventional candidate progresses8/16,8/16,14/16,16/16 at updates64,128,192,256. The old crossed diagnostic now tracks the feedback-only substitution and does not track the fingerprint-only substitution. Its no-Copy-event control still matches the actual case; event-specific dependence is not established.

Both selected artifacts are step256, chosen by new-source Tune loss among candidates satisfying their declared gates. The causal arm adds approximately0.01799 bits/target versus ordinary training on repository development and0.06955 versus the parent. This is a measured tradeoff for causal behavior and domain adaptation, not a universal loss improvement. Only one training seed is used here; the fresh-process replay is a determinism test, not another independent training seed.

## 2. Exact continuation is implemented and exercised

`TrainingCursor` records data/tokenizer hashes, RNG state, next batch, complete schedule length and exact learning-rate endpoints. `TLK1` stores the latent f32 bits, both Adam moment families, optimizer configuration/step and cursor, with a source binding and SHA-256 checksum. Parsing bounds the header and whole file, validates dimensions before allocation, and rejects nonfinite state, invalid schedules, wrong source/data/tokenizer, truncation and corruption.

A separate process resumed the actual intervention run from batch128. At batch256, **`final.tlk`, `final.tlx` and `selected.tlx` were byte-identical** to uninterrupted training. The validation records, selected candidate and generated outputs also match. The control companion preserves prior validation history and the previously selected model, rather than resetting selection on resume. Unit tests additionally check uninterrupted versus restored latent/moment equality.

A served `.tlx` alone is still not a checkpoint. Exact replay requires the bound source, data, schedule and checkpoint companion. Extending the completed256-step schedule is a new optimization phase, not replay of the old one.

## 3. Broader training and final-source evaluation

The new training sources are *Treasure Island*, *A Tale of Two Cities*, and two Rust Book chapters on types and functions. Tune sources are *The Time Machine* and the Rust Book control-flow chapter. The final sources were declared before download and the candidate choice was frozen before their first scoring: *The Adventures of Tom Sawyer*, *Heart of Darkness*, and the Rust Book ownership chapter. Source bytes and the Rust Book commit are retained and hashed. Rust chapters are separated by source file within one book, **not independent source families**.

Each final source contributes16 evenly spread64-token windows, with8 observed prefix tokens:896 targets each,2,688 total. No final-source gradients or candidate selection were used.

| Frozen candidate | Three-source bits/target |
| --- | ---: |
| Retained warm parent |11.471785118|
| Ordinary broader training |**10.598857531**|
| Causal broader training |10.614216096|

The causal model improves over the parent on all three files by0.857569 bits/target overall. It trails the ordinary-supervision control by0.015359. With only three source clusters, no narrow general-language confidence claim is made. All original figures are in the evidence, including per-source sums; these sources are now exposed regression material.

The finite authored instruction test uses numbers8–11, versus0–7 in training. **0/4 dialogue answers are exact and0/4 unmodified generated Rust outputs compile.** A separate supplied compiler control compiles, and is not counted as model output. The model outputs were not repaired, wrapped, or replaced with expected code; none was executed because all failed compilation. Generated continuations remain repetitive and often drift into repository formatting. Source-prediction improvement is not useful conversation or coding.

## 4. Learned relative action and the corrected ordinary control

The new finite learner fits one Q8 action per typed relation from supplied query vectors, candidate keys and selected-key labels. It receives no hidden generating-action IDs.64 one-step fit examples support1,024 tests with larger novel vector values and operation compositions of lengths1–6. The Q8 learner scores1,024/1,024; disabled transport scores165/1,024 and reversing operation order scores630/1,024.

An initial384-operator ordinary signed-permutation learner scores864/1,024 under a ranking-margin objective. Its fit contains ties that underidentify the operation. Breaking those ties by the already supplied positive-query distance, without new labels or parameters, gives **1,024/1,024 as well**. The audit reproduces the original Q8 and weak-control counts before testing this repair. The initial apparent predictive gap is therefore not a defensible unique geometric advantage.

This is an implemented learned relation-conditioned vector-selection component, not learned natural-language attention in the lexical model. Its14-byte serialized Q8 model and the ordinary control's40-byte logical parameter representation are different encodings, not a general memory-efficiency theorem. The synthetic task was generated from Q8 structure. The causal and prose gains above are not attributed to this separate learner.

## 5. Compiled sparse readout and real generation-path cost

`TlReadPlan` converts the packed ternary output map into row offsets, signed column indices, row shifts and biases. Its successful `score_into` call uses caller-owned buffers and performs zero allocations in the dedicated allocator census. The original model remains available and default callers are not silently switched.

Three measured runs each compare4,196,352 individual output scores and all complete generated sequences. Each uses eight already-tokenized16-token prompts followed by up to48 generated tokens, nine interleaved rounds per implementation. Prompt-state ingestion, recurrent updates and their allocations are included; tokenization, model loading and one-time plan compilation are excluded.

| Idle repeat, same model in both arms | Original median us/generated token | Compiled median | Ratio |
| --- | ---: | ---: | ---: |
| Retained warm parent |1,280.669|122.829|10.426x|
| Causal candidate |1,286.328|123.589|10.408x|

The initial run overlapping an other-core compiler gave10.468x and is retained, not substituted for the idle repeats. This is equal-output cost evidence on these prompts, **not a quality-matched comparison with a frontier model, whole-application benchmark or energy measurement**.

For the causal candidate, the plan visits152,395 nonzero entries instead of inspecting602,406 packed coefficients. Its logical array payload is370,366 bytes, added to the retained packed model; this is not total allocator capacity or a smaller stored model. Parent plan compilation took about3ms. A separate AArch64 inspection covers211 instructions in the named `TlReadPlan::score_into` symbol and finds none of the enumerated multiply/divide or floating-arithmetic mnemonics. It does not certify all callees or the entire executable. The recurrent update still allocates.

## Verification, preservation and remaining work

The final verification reruns61 focused tests:20 transferable-learner/checkpoint/intervention/read-plan,2 relative-action,6 Hamilton,7 integer-prior/ratio,25 runner and1 allocation test. The attempted Cargo integration-test command began compiling unrelated package binaries and was stopped; the allocation census was then compiled directly against the pinned library and executed successfully. This is not a full-repository-suite claim.

A fresh native verifier checked11 sealed report roots and a newly claimed retained-candidate root. Exact real-data checkpoint replay, candidate hashes, source/data bindings and final-source populations are recorded. The remote connection briefly failed during report assembly and returned before delivery; no experiment was inferred from that interruption.

The retained root is `.uor-models/causal-continuation-2026-09-24/`. Its causal candidate SHA-256 is `1ac700658a476e356ff64db95003277f30c27b3d181ca183444dd7621c5d955f`. Its final full checkpoint SHA-256 is `673c551f32c8c2786aa7127a6ea3ccfbe4f0aa5219a25c3f572c39ad07f20f94`. The prior warm and ratio artifacts remain intact.

Two redundant worktrees were removed only after checking zero modified, untracked and ignored files and retaining their commit-bearing branches. Cleanup raised physical free space from52.123GB to54.668GB. The final verification measured54.630GB. The owner's tracked diff and54 of55 snapshotted file hashes match; one live `.omo` continuation file changed independently and was not restored. Delivery-time space and cumulative work are recorded separately.

**Next independent responsibilities:** expand causal copying to new values, spans, histories and the integrated session; train useful dialogue/code generation on broader structured supervision; connect learned relative geometry to genuine language/context retrieval with matched controls; carry exact optimizer continuation into longer experiments; integrate the compiled read plan into selected runtime callers and measure complete quality-matched device cost. These remain work, not capability claims.
