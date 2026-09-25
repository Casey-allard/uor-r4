# Offline Rust training foundation

This crate contains the offline recurrent-memory learner, its matched transport control, and differentiable research references. No serving crate depends on it. The recurrent learner trains state, causal memory reads/writes and token output jointly. The separate reference modes reproduce the retained #1017 ordinary transformer. Integer export remains a later integration step.

**Memory profile: allocation-conscious offline computation.** Construction,
autodiff graphs, whole-sequence worker gradients and serialization allocate.
Campaign dimensions, update count, worker count and checkpoints are explicit;
the process supervisor monitors RSS, storage and elapsed time. This crate makes
no heapless or allocation-free steady-state guarantee. Portable serving crates
retain their separate stricter contracts.

## Joint recurrent-memory learning

The [September 25 result](../../docs/integration/joint-recurrent-result-2026-09-25.md) completes the full paired campaign: 29,999,104 target visits per arm, including 21,381,120 at context 256. Both learners pass the frozen development likelihood/noncollapse/combined read-effect engineering gates. Ordinary has lower natural NLL; generated stories remain semantically unreliable. Integer export and geometric advantage remain unqualified. The [subsequent quantized continuation](../../docs/integration/quantized-recurrent-result-2026-09-25.md) completed all four matched branches but failed both likelihood-retention gates; ordinary packed generation also short-cycles. The [projected-shadow successor](../../docs/integration/projected-recurrent-result-2026-09-25.md) is also complete: both likelihood gates fail (+0.059801/+0.083525 versus continuous), while its other four limited gates pass. All negative candidates remain retained. The [completed fixed-checkpoint precision comparison](../../docs/integration/precision-factorial-result-2026-09-25.md) reproduces retained endpoints and finds parameter-family cost dominant. The [learned-code successor](../../docs/integration/learned-rounding-result-2026-09-25.md) now passes all five original hard-artifact gates in both arms. Its packed-minus-continuous gaps are+0.023708/+0.046477; both candidates are retained for trained bounded admission/transport. Individual complete-answer regressions and unreliable prose remain.

```text
uor-r4-training joint-fit CAMPAIGN_JSON NEW_REPORT_ROOT {cpu|metal} [SEALED_RESUME_CHECKPOINT]
uor-r4-training joint-evaluate CAMPAIGN_JSON SEALED_CHECKPOINT NEW_REPORT_ROOT {cpu|metal} {read|no-read} BATCH
uor-r4-training joint-compare BASELINE_ROOT QUAT_READ QUAT_NOREAD ORD_READ ORD_NOREAD NEW_REPORT_ROOT
uor-r4-training joint-export-hard SEALED_CHECKPOINT NEW_REPORT_ROOT [INITIAL_QAT_CAMPAIGN_JSON]
uor-r4-training joint-evaluate-hard SEALED_CHECKPOINT_OR_EXPORT EVALUATOR_JSON NEW_REPORT_ROOT cpu {read|no-read} BATCH
uor-r4-training joint-evaluate-shadow CAMPAIGN_JSON SEALED_CHECKPOINT NEW_REPORT_ROOT {cpu|metal} {read|no-read} BATCH
uor-r4-training joint-evaluate-precision SEALED_SHADOW_CHECKPOINT EVALUATOR_JSON NEW_REPORT_ROOT cpu {FF|QF|FQ|QQ} BATCH
uor-r4-training joint-round-calibrate ROUNDING_RECIPE_INPUT_JSON NEW_REPORT_ROOT
uor-r4-training joint-round-fit RESOLVED_ROUNDING_RECIPE_JSON NEW_REPORT_ROOT [SEALED_ROUNDING_CHECKPOINT]
```

The [frozen campaign](../../docs/integration/joint-recurrent-campaign-2026-09-24.md) specifies the graph, training exposure, resource ceilings, checkpoint selection and capability criteria. `JointModel` exposes the same causal core for differentiable unrolls and detached incremental sessions. It reads only earlier occurrences, updates recurrent state using the read, then writes the current contextual key/value. Exact observed token/occurrence identity is retained alongside learned vector compatibility. A normalized vocabulary/copy mixture supplies the language loss; targets enter only that loss.

The quaternion and Householder-pair arms share dimensions, parameter initialization and data windows. Their local transport scales are matched; their global function families differ. These floating-point offline learners do not establish exact Hamiltonian dynamics or the final integer serving cost. Prime/zeta admission and typed integer execution remain subsequent integration work. Both earlier packed recipes fail likelihood retention. The learned neighboring-code successor passes the original five engineering gates; its numerical execution remains F32.

Each checkpoint includes named model parameters, AdamW moments and per-variable clocks, model/optimizer configuration, exact source/data/tokenizer identities, and the next counter-seeded training window. Resume rejects changes to the model or sampler. The optional `stop_file` requests a checkpoint between updates. `max_process_seconds` stops new updates; the campaign separately reserves final evaluation, save/reload and sealing time.

A separately declared `training_window_transition` can change batch/context while preserving their product, exact model and optimizer, and global step. It binds the actual parent checkpoint/campaign hashes and prior exposure; ordinary resume still rejects undeclared changes. The current correction retains a matched64-token warmup and continues with256-token training to match evaluation. Report both phase exposures. The changed dimensions define new sampled windows, and old/new retained-batch losses are different samples.

`cpu_gradient_shards` defaults to one and accepts one, two or four; more than one is CPU-only and requires a divisible batch. Workers split complete sequences along the batch dimension, share immutable parameters, and return gradients in a fixed order. The caller forms the weighted mean before one global clip/AdamW update. Resume and evaluation bind the selected shard count; floating-point reduction order is not claimed bitwise equal to an unsplit batch. Measured M1 execution selected two workers per arm with both arms concurrent and nested backend thread limits of one. Four workers per arm and the tested Metal path were slower.

`joint-evaluate` loads the actual checkpoint, runs free continuations and all frozen source-edit pairs, and then scores every evaluator-v2 development position. NoRead applies from the start of every prompt/block. Per-target records use the explicit44-byte format in `evaluation-report.json`; probability flooring or answer repair is not added by evaluation. The current natural-story probes measure exploratory task transfer. Their failure alone does not diagnose the read mechanism.

`joint-evaluate-precision` is a CPU, read-enabled, evaluation-only intervention
on a sealed floating checkpoint whose quantization ramp has completed. The first
letter enables/bypasses parameter quantization; the second enables/bypasses all
five declared interface grids. All four modes retain the same stored shadows,
scales and clocks. The mode follows prepared tensors and incremental sessions;
training, saving/export and cross-mode session reuse are rejected. Mixed modes
report both switches rather than one quantization-strength scalar. FF and QQ
must reproduce the retained shadow and packed endpoints before interpreting
QF/FQ. None of these views implements integer serving. See the
[fixed-checkpoint plan](../../docs/integration/precision-factorial-plan-2026-09-25.md).

### Quantized continuation and packed evaluation

`joint-round-calibrate` resolves the declared training-only gradient-norm
normalization before any optimizer update. `joint-round-fit` learns one shared
alpha per original parameter coordinate, choosing between its legal floor/ceil
codes while preserving the original dyadic grids and interface quantizers.
Exact-grid coordinates have one choice. Full recurrent language gradients reach
the soft parameter values without a second parameter quantization. Final hard
codes use alpha's sign, with the original away-from-zero tie convention.
The [frozen recipe](../../docs/integration/learned-rounding-plan-2026-09-25.md)
declares the global coordinate-mean penalty, schedule and exact normalization.

Rounding checkpoints store alpha, new Adam moments/clocks, the immutable parent
identity, resolved recipe and continuation cursor. Resume preserves this whole
state. Parent optimizer step8,348 and rounding update count512 are separate
clocks; packed reports retain the added learning lineage. Only the complete
fixed recipe writes the independently materialized packed candidate, whose
existing loader requires no alpha or original shadow file. Its computation
still uses the floating numerical emulator; low-bit storage does not qualify
integer execution or terminal parameter sparsity.

The optional `projection_transition` binds a fully quantized parent checkpoint,
campaign and quantizer specification. Its entry projects floating parameter
shadows into their existing ranges while witnessing unchanged hard values,
full-context probabilities, optimizer moments/clocks and next sampled batch.
`joint_campaign` calls `QuantizationSpec::project_parameters` after each AdamW update;
interior values are preserved rather than rounded. The policy is retained in
checkpoints, export/evaluation and paired comparison, with incompatible policy
removal, rebinding or schedules rejected. Missing policy preserves historical
behavior. See the [projection plan](../../docs/integration/projected-recurrent-plan-2026-09-25.md)
for the executed 512-update continuation and unchanged five gates.

The [rung 2 plan](../../docs/integration/quantized-recurrent-plan-2026-09-25.md)
specifies frozen parent-calibrated dyadic scales, signed four-bit multiplicative
parameters, wider signed sixteen-bit additive offsets and fixed-point recurrent
interfaces. `quantization_transition` binds the exact continuous parent,
optimizer step and ramp. Resume preserves scale specification, optimizer moments,
window dimensions and global sampler. Every development measurement uses full
quantization, with continuous-shadow scores recorded separately on the same
prefix. Training alone uses the declared identity-to-hard ramp.

`joint-export-hard` writes packed codes, integer exponents and bound model
configuration. Its loader reads no floating shadow checkpoint. The optional
campaign argument is for initial calibration of an exact continuous parent;
it performs zero optimizer updates. Export verifies actual full-context output
and fixed-seed generation equality, then records loaded text and smoke-scoped
interface clipping counts. `joint-evaluate-hard` uses this packed loader; a
checkpoint input first creates a separate sealed packed child. `joint-evaluate-shadow`
disables quantizers on the same QAT floating parameters for a measured gap.

These are **F32 numerical emulators of quantized values**. Dense matmul,
normalization, nonlinearities and probability arithmetic remain floating.
Integer execution, bounded admission, sparse parameter access and energy are
separate later work. The signed unit transport coordinates are rounded without
off-grid renormalization; exact group closure is not asserted. Packed parameter
size does not establish measured process RAM or serving speed.

The descriptive `joint-state-drift` example accepts `QAT_CHECKPOINT PACKED_EXPORT EVALUATOR_JSON NEW_REPORT_ROOT`. It claims and seals an independent root, verifies bitwise packed-parameter agreement with the quantized selected shadows, and records finite coordinate/norm differences by all 256 positions on the original 64 tune windows. It uses the shared model implementation with quantizers enabled/disabled; it does not isolate transport error or add an acceptance threshold. The result receipt binds the executed source and binary.

Evaluation accepts batches1–32. `joint-compare` joins all249,856 targets against the two sealed baseline evaluations and four learner/control evaluations. It verifies identities and original means, reports paired differences and horizon slices, and preserves the original generation/probe hashes. Rung1 selected by recorded F32 `quick_loss` tune scores; rung2 reports the frozen final common step. The comparison never reselects or decides generation quality. Hard and shadow modes are explicit, with matching Read/NoRead artifact/specification and cross-arm schedule/layout checks.

On Apple hosts, optional `cpu-accelerate` enables the pinned, corrected Candle BLAS adapter for offline training. Its four-line source difference and all upstream identities are retained in [the vendor note](../../third_party/candle-core-0.9.2/UOR-PATCH.md). CPU/Rust-gemm and Metal remain explicit alternatives. Backend-specific numerical behavior is reported; cross-backend bitwise training identity is not claimed.

## Integrity mode

```text
uor-r4-training integrity SNAPSHOT TOKENS_U16 OUT_JSON {cpu|metal} WINDOW
```

`WINDOW` is 8–64 inputs; the command reads exactly `WINDOW+1` little-endian token IDs from offset zero and hashes the full source store. Use a new `OUT_JSON` file for each attempt. The file is created exclusively after argument validation and before model loading. Completed and failed results are retained. No optimizer step or checkpoint modification occurs. Finite-difference perturbations are temporary and restored.

The retained snapshot is `/Users/casey.allard/uor-r4/.uor-models/research/issue-1017/export`. Its `model.safetensors` SHA-256 is `cf988a6b5f722b614740b0843807a41feff8276571c846bbee85d410a0030d84`. Loading rejects other weights or shapes. All 56 F32 tensors become named differentiable variables. The tied output reuses `model.embed_tokens.weight`.

CPU is the default build. Optional `metal` enables Candle Metal; a requested device never falls back silently. Optional `reference-accelerate` enables the existing Rust oracle's Apple Accelerate comparison path. Builds used for evidence set `UOR_BUILD_SOURCE_COMMIT` to the actual committed source. Candle core and nn are intentionally pinned to **0.9.2**, an established published release; this is not a claim that it is the newest release.

The forward pass composes differentiable RMSNorm, half-rotation HF Llama RoPE, causal softmax, Q/K/V/O projections and SwiGLU. It avoids Candle's inference-only fused softmax/RMSNorm helpers. Every logit is compared; the report includes per-position maximum error and top-one identities, all parameter gradient norms, Q/K/V gradient availability for all six layers, and three finite-difference coordinates chosen as the largest absolute gradient in layer-zero Q, K and V. The finite-difference tolerances and parity ceiling are constants in the source, fixed before execution. Full final-position logits are retained for independent inspection. The reference oracle reports its actual selected arithmetic owner, including an exact-mode environment override if present.

## Population comparison and historical generation

```text
uor-r4-training ngram-fit EVALUATOR_JSON NEW_REPORT_DIR
uor-r4-training ngram-evaluate EVALUATOR_JSON SEALED_FIT_DIR NEW_REPORT_DIR
uor-r4-training reference-evaluate EVALUATOR_JSON NEW_REPORT_DIR {cpu|metal} BATCH
uor-r4-training reference-generate EVALUATOR_JSON NEW_REPORT_DIR {cpu|metal}
uor-r4-training verify SEALED_REPORT_DIR
```

[Evaluator v2](../../docs/integration/reference-evaluator-v2.json) pins the two inherited training stores, the #1017 checkpoint/tokenizer, exact development positions, calibration grid, generation prompts and historical output identities. Every input is rehashed. Each new report directory is claimed before loading a model, and its complete file set is sealed and verified on success or failure. Preserve the fitted `model.ng5` and its separate `count-selection.json`: the latter fixes the calibrated discount and cache mixture.

The count comparator uses raw five-gram counts, lower-order distinct left extensions from unpruned raw types, a fixed interpolated discount, deterministic count pruning and a normalized unigram floor. Pruned and discounted mass backs off. It is not the multi-discount modified Kneser–Ney algorithm. Training sequences remain separate; evaluation history and the token cache reset at the declared 256-position boundaries. The first 64 development blocks choose the pure-count discount, then the cache mixture with that discount held fixed. Settings are saved before scoring the comparison tail. This tail is separate from current count calibration but was previously exposed during neural checkpoint selection.

Reference evaluation uses detached parameters and batch-isolated causal tensors. It records every actual target logit and full-vocabulary log-normalizer, and checks batch isolation, future-input causality and reproduction of historical population likelihood. Generation recomputes the prefix without a KV cache and preserves every actual sampling decision. Five seeded outputs are persisted before historical golden files are opened; token, response and stop equality are compared separately from the old R4 audit metadata. Its timing is an offline reconstruction cost, not optimized serving throughput.

## Scope and next integration

A successful integrity report establishes agreement for that token window and an operative gradient through **soft** attention. It does not establish hard admission, quantization parity, general language ability, a geometric advantage or final serving compliance. The prior #1017 confirmation set has already been opened; it can serve as a historical regression set, never a fresh final draw. Future corpus evaluation/training modes should extend this crate and preserve separate development and final protocols.

The old safetensors/config/tokenizer and Rust reference loader are reused directly. The Python pickle checkpoint and optimizer state are not loaded. A future quantized bridge must explicitly specify its surrogate, gradient path, code usage and hard exported evaluation rather than assuming autodiff differentiates argmax.

Primary implementation references: [Candle 0.9.2 API](https://docs.rs/candle-core/0.9.2/candle_core/), [Candle training example](https://github.com/huggingface/candle/blob/main/candle-examples/examples/mnist-training/main.rs), [tensor operations and inference-only softmax](https://github.com/huggingface/candle/blob/main/candle-nn/src/ops.rs). The actual retained architecture is implemented in `tools/r4-softmax-trainer/src/r4_softmax_trainer/model.py`; this crate reproduces it in Rust as an offline comparator.

Executed September 24: the pinned 32-position reference check passed on CPU and Metal; see [the compact evidence](../../docs/evidence/reference-autodiff-integrity-2026-09-24.json) and [evaluator manifest](../../docs/integration/reference-evaluator-v1.json). That integrity mode enforces reference weights and architecture and emits full input identities; its complete manifest was verified independently. The comparison modes enforce evaluator v2 directly. The [current state](../../docs/integration/current-state.md) owns the reference results and the separate recurrent learner's outcome and next execution.
