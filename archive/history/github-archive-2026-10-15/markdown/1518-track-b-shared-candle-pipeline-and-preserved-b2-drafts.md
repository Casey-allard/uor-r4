# #1518 — Track B: shared Candle pipeline and preserved B2 drafts

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-29T20:47:39Z
- **Updated:** 2026-10-09T13:08:49Z
- **Closed:** 2026-10-04T04:48:24Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1518
- **Merged:** not merged
- **Merge commit:** 9e7704915c1604ca3cdb9d110092516e9b7188a2
- **Base / Head:** main / UOR-Foundation:codex/track-b-conversion
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Track B needs a differentiable Candle Llama and a fixed reference gate before attention conversion. This adds a pinned stock loader, an all-position batched model with pluggable attention, harmonic features, seeded bounded generation, and the unchanged CPU/Metal all-logit `1e-4` parity driver. These are offline comparator/training components, not a transformer serving candidate.

**Draft: complete dense checkpoint parity failed its numerical gate. The separate Metal harmonic floor check also failed.** Source checkpoint `2cec24948991b36afdcdb3972afecca9c6c4fe82` includes recorded results and targeted diagnostics; no failed threshold was relaxed.

Validation and remaining limitations:

- Main `3b7c2d54` (Metal kernels, checkpoint codec restoration and core repairs) is integrated at `97dcd0cf`. The manifest conflict retains both Track B and kernel features. Independent review confirms unchanged Track B operators/evidence. All 14 Metal fixtures passed; a 0.06s replay of the 13 non-benchmark fixtures captured actual Metal device initialization and forward/backward differences, distinguishing execution from skips. Four focused Track B model tests also passed. Build/test totals: 219.74s + 59.47s, maximum RSS 2,846,801,920 bytes. No full-model parity or performance qualification is implied.

- At `dcb8b107`, 17 registered CPU unit tests and three parity-gate tests passed. [Executed logs and review](https://github.com/UOR-Foundation/uor-r4/tree/c9d8b8eeda72434f3b1f4a085e4877f8a678736a/validation/track-b/dcb8b107).
- At `648ba939`, all 45 reference positions and 294 candidate rows completed. Stock CPU/Metal maximum logit errors were `0.0014320612 / 0.0085887909`; shared CPU/Metal maxima were `0.0070133209 / 0.0067539215`, against `1e-4`. [Sealed evidence and independent review](https://github.com/UOR-Foundation/uor-r4/tree/8286a5cbfafa1945353692bf19dfcd6c145c8842/validation/track-b/648ba939-parity).
- The unchanged Metal harmonic floor failed; its separate largest-arm gradient check passed. A common-input diagnostic identified the final contraction as dominant for that fixture, without establishing a general Metal defect. [Evidence](https://github.com/UOR-Foundation/uor-r4/tree/5578d316e649d9a3d06e62f798562916f7ee68b1/validation/track-b/648ba939-metal).
- The eight-token dense trace reproduced all reference/CPU/Metal final logits bit for bit. Common-input layer-12 and block-11 replays show incoming-state effects and amplification through the gated product/down projection; attention substitutions do not constitute a repair. [Block result and limitations](https://github.com/UOR-Foundation/uor-r4/blob/b2cf9744/docs/evidence/track-b-block11-replay-2026-09-30.md).
- At `93085dd5`, the common-input down replay reproduced both saved outputs, and CPU/Metal outputs were byte-identical on each input. At position 5, exact down arithmetic retains most of the difference between the candidate inputs. This rules out a backend contraction difference for these fixed inputs, not total error against the full reference. Build 194.62s; replay 3.18s. [Result](https://github.com/UOR-Foundation/uor-r4/blob/b2cf9744/docs/evidence/track-b-down-replay-2026-09-30.md), [independent review and execution receipts](https://github.com/UOR-Foundation/uor-r4/tree/d0518ed4757e96ab7cf3f8941a463769704e60ba/validation/track-b/93085dd5-down-replay).

The anchored reference-tail reconstruction at `0db94dbf` matches all 4,608 original residual values bit for bit. Local interventions identify incoming after-attention differences at the selected token and normalization arithmetic across the window. [Result](https://github.com/UOR-Foundation/uor-r4/blob/9e3b4eaefcaf23fc5b7a8263cd82d36def1074b1/docs/evidence/track-b-mlp-replay-2026-09-30.md), [independent review](https://github.com/UOR-Foundation/uor-r4/tree/da6d37228264a20bd9b4be2cfa742c776921c0a4/validation/track-b/0db94dbf-mlp-replay).

The RMS scalar-order candidate `d87485a8` passed four focused model tests but regressed the actual CPU window and left Metal unchanged. The prior RMS source was restored exactly in `f1cb000f`; no broader run followed. [Negative and restoration](https://github.com/UOR-Foundation/uor-r4/blob/f138f51731cba6ba39549173668e32730c9617c8/docs/evidence/track-b-rms-candidate-2026-09-30.md), [independent review](https://github.com/UOR-Foundation/uor-r4/tree/69976184b5787e8e38142ea8cee9ab7c24f4a290/validation/track-b/d87485a8-rms-window).

The local numerical-correction trial is complete. A council decision on the next coherent arithmetic/comparator path is needed before another variant; the original gate remains failed. Source integration and peer reviews continue. No unchanged reference rerun or B2 fit. Unregistered drafts remain unqualified.

The docs-only successor updates the lab routing snapshot and labels the initial repair table historical. It also replaces stale blanket build-hold instructions with the owner’s direct-check/internal-cache corrections; no model source or numerical gate changes.

References #1515, #1509.


## Timeline (13 entries; chronological: conversation comments, reviews, review comments)

### comment 5898789702 — 2026-09-29T21:00:25Z — Casey-allard

Independent non-author adversarial review of **05a909d3ec468c71bd21871544fd7a025001b53d** found no blocking source/contract findings for draft publication. The reviewer did not author these changes or run builds/models.

Reviewed boundaries: native GQA mapping, batch/time indexing, split-half RoPE, gradient attachment through later frozen layers, post-WO capture, future-only right padding and per-item RNG stability, corrected top-k/top-p normalization, harmonic packing/floor diagnostics, and schema-2 coverage. The report projection matches 421 raw rows × 49,152 × 4 = 82,771,968 bytes, below 96 MiB.

This review **does not establish compilation, test passage, checkpoint parity or generation qualification for the new shared implementation**. Those remain NOT_RUN while storage admission is blocked. The earlier stock-loader checks and sealed interrupted attempts retain their separately bound source/executable scope. Lab 1's required numerical-headline reread and protected delivery gates remain outstanding.


### comment 5913717061 — 2026-09-30T14:50:20Z — Casey-allard

Track B integration checkpoint: existing PR #1518 now at `dcb8b107f499395c65b86404d67e9e83916054fd`, incorporating main `dc82874723c45fca9ffe7b9fa80f80ee9d3fe497`. Resolved dependency union (retain both candle-transformers and half), preserved current peer status, and reconciled the Track B record with owner-authorized direct small checks. Untracked local recovery material preserved.

Executed: offline locked dependency metadata, actual PR-delta whitespace check, touched Track B Rust formatting, and claim-wording gate PASS. This is not compilation or loaded parity evidence.

Next direct check: the three `track_b::conversion::tests` unit tests (tiny synthetic weights, no downloaded model), source above; separate internal cache `/Users/casey.allard/.cache/uor-r4-track-b-direct/target`, one Cargo worker / one numerical worker, 600 seconds maximum, 2 GiB sampled process-tree RSS ceiling, 2 GiB new cache ceiling, 40 GiB internal free floor plus 128 MiB margin, stop on critical host pressure. No runner reservation or waiting on another Cargo lane. DeepSeek's existing G1a process is left running. Initial pressure is normal (1), internal free about 45.8 GiB. Build/test cost will be recorded once from measured elapsed time. If the bounded compilation fails, preserve the exact compiler diagnostics and repair only relevant source.

Full CPU+Metal all-logit 1e-4 parity remains NOT_RUN and required before B2 transfer. Existing B2 drafts remain unqualified. Also closed redundant PR #1545 only after verifying its sole file has identical blob `1a95040c5b009bde6ccf06166ae19355b0e8d436` on main; branch/history retained.


### comment 5913829718 — 2026-09-30T14:56:24Z — Casey-allard

# Bounded independent Track B dense-path source review

Reviewer: Codex `/root/second_council_review`, delegated by `/root`.
Date: 2026-09-30.
Exact source head: `dcb8b107f499395c65b86404d67e9e83916054fd`.
Worktree: `/Users/casey.allard/.codex/worktrees/track-b-conversion/uor-r4`.
Scope: `track_b/model.rs`, `track_b/conversion.rs`, directly used loader/RoPE functions, model-source caller definitions at this revision, and locally pinned Candle 0.9.2 implementation. No broad history, infrastructure, model-quality or numerical qualification audit.

**Disposition: no concrete defect requiring source correction found in the requested masking, head mapping, RoPE positions or supported-checkpoint loading boundaries.** This is bounded source review, not an all-logit parity PASS or a claim that the 1e-4 gate will pass. Do not relax that threshold or advance B2 based on this receipt.

## Checks and source evidence

1. **Causal mask and layout.** `model.rs:368–388` validates B*T and token bounds, creates the flattened embedding rows and uses absolute query/key ranges `0..time`. `model.rs:552–572` masks exactly `key > query`, retaining the current token and every earlier token. `model.rs:391–395,425–443` consistently maps flattened [B*T,W] through [B,T,H,D] and back, preserving batch/time order. `model.rs:540–549` applies -infinity to future scores before softmax over keys. This agrees with the inclusive model-source causal prefix (`lib.rs:1755–1782`) and Candle's square full-prefill mask (`llama.rs:218–227`). Right padding cannot affect earlier positions under this causal rule; no general left-padding or variable-position API is promised.

2. **GQA mapping.** `model.rs:505–536` checks compatible native KV shapes and integral H/KV grouping. Its expansion is [B,KV,group,T,D] followed by contiguous flattening, so query head h uses KV head floor(h/group). That matches model-source `lib.rs:1615–1617,1759` and Candle `utils.rs:28–37` (concatenate repeated sequences within each KV head, then reshape). Q and O keep full width; K/V use KV_heads*head_dim. No head-swapping or reduced-width source projection was found.

3. **RoPE positions and convention.** `model.rs:380–407,482–499` generates frequencies using the same F32 theta/exponent construction and position-by-frequency matmul as pinned Candle `llama.rs:155–160,198–207`. Both Q and K use positions starting at zero for a fresh full prefix. Shared `kappa_llama.rs:542–553` rotates split halves `(a*cos-b*sin, b*cos+a*sin)`, matching the native Hugging Face oracle branch in model-source `lib.rs:1694–1729` and Candle's non-interleaved rotary path. Interleaved/scaled/partial RoPE configurations are rejected before loading by conversion config parsing. This establishes source convention alignment, not bitwise backend equivalence.

4. **Conversion cache traversal.** `conversion.rs:126–166` clones the empty cache for each call and advances each singleton at its true absolute position. Full prefill starts at zero; split prefill continues only with singleton cached inputs. This avoids Candle 0.9.2's square-mask limitation for a multi-token tail after an existing cache (`llama.rs:300–351`). The upstream model returns only the final input position (`llama.rs:505–514`), and the conversion wrapper collects all positions through singleton stepping. The shared model deliberately has no incremental-cache API and returns all full-prefix positions.

5. **Loader/shape compatibility.** Shared load applies conversion's configuration checks before invoking the Kappa loader (`model.rs:168–181`). Supported configuration is bias-free SiLU Llama, integral/even head geometry, unscaled split-half RoPE, consistent optional head_dim, finite positive RMS epsilon/theta and bounded context (`conversion.rs:182–255`). Both use the expected row-major HF projections and tie the vocabulary head to embedding when configured (`model.rs:218–270`, Candle `llama.rs:517–526`). Kappa's loader enforces exact tensor inventory/shapes and finite BF16/F32 decoding (`kappa_llama.rs:329–445`); BF16 conversion is an exact upper-bit expansion into F32. The conversion loader prevalidates required weights before entering upstream block construction (`conversion.rs:87–107,258–298`), whose pinned implementation otherwise unwraps a load error.

## Important qualification limits

- Shared RMS normalization, RoPE and softmax are differentiable primitive compositions (`model.rs:473–498,547–549`); stock Candle uses fused/inference operations, and the oracle uses its own exact-executor arithmetic. Shared full-prefix/batched matrix reductions differ from the singleton path. Small floating-point differences can accumulate. Whether every logit is within **1e-4** on **both CPU and Metal** remains an executed, artifact-bound question. No failure or pass is inferred from equivalent formulas.
- The strict shared loader supports the declared exact single-file BF16/F32 inventory. A different checkpoint with shards, extra tensors (including a redundantly serialized tied head), or F16 weights may be rejected even if another loader accepts it. This is a declared supported-input restriction, not evidence that the pinned SmolLM2 source is incompatible. I did not read or load the model artifact in this review.
- Full-prefix positions begin at zero; no cached multi-token continuation or arbitrary offset support should be inferred. Hook output shape/dtype checks do not establish arbitrary custom attention's causal correctness.
- The existing parity example retains fixed 49,152-vocabulary windows and per-position all-logit comparison at 1e-4. I inspected it for the caller scope; no report root or model was executed. Tiny deterministic tests and a successful build, including work being run by the parent, are not loaded full-model CPU/Metal parity or language qualification.

## Exact source identities

- `crates/uor-r4-training/src/track_b/model.rs`: `a9f518ca9f8703c96ab0033d768c574a4091aa729807a0af37beebf5d769a672`
- `crates/uor-r4-training/src/track_b/conversion.rs`: `d11e4fe980d67f99589e69ef42f6902d3bd9b7d4a8d1894fac4b4171bc33c3d4`
- `crates/uor-r4-training/src/kappa_llama.rs`: `11679e44690c7ab772ee2675f426061308ad9ea1a2753fa2c14b02f599672792`
- `crates/uor-r4-model-source/src/lib.rs`: `66c39e1ac98c71dd10d42c1f4853f8f18bbcb67d390bf63391ccfde28b3bf11b`
- `crates/uor-r4-training/examples/track-b-parity.rs`: `75628dffe8fd14f27fa86614cfa5468c78405ba233edf4b53965f98ad273cfc3`
- `Cargo.lock`: `5194d70bd8259baf071e56b21b541caafbfdd0e0c3ccc4f8373d516c90ea7c0f`
- Local pinned Candle `candle-transformers-0.9.2/src/models/llama.rs`: `b3f438e3e12449122c27fe6884dbdbe3246a1f551d4cdac10b67d5f11c70c446`. Cargo.lock pins candle-transformers 0.9.2. Also read this package's `utils.rs` and candle-nn 0.9.2 `rotary_emb.rs` for the called operators.

## Work and independence

Read-only source review, no Cargo/builds/tests/model runs, no edits to source, no GitHub writes, no runner work. Only this review artifact was written. Non-author of the reviewed Track B source. Used the UOR project workflow's source/evidence discipline; current owner numerical policy takes precedence over stale stricter wording in that skill. Historical memory was used only to locate the pinned parity context and its cached-tail pitfall; code and gate definitions were checked directly at the reviewed revision.


### comment 5913917802 — 2026-09-30T15:01:24Z — Casey-allard

Direct build continuation: attempt 1 stopped at its 600-second wall bound while optimizing uor-r4-core; tests NOT_RUN, no compiler error, sampled process-tree peak after monitor recovery 1,717,264 KiB. Dependencies and about 0.7 GiB cache are retained. The one-off monitor encountered a transient du vanished-file error; same process was re-observed without duplication, but its actual exit status is unknown. Under standing local allowance authorization, prospectively add up to 900 seconds for one cached continuation (total build ceiling 1,500 seconds), same source and 1-worker/2-GiB RSS/2-GiB cache/40-GiB floor plus stop margin and critical-pressure limits. No cumulative ledger limit increase needed. This is completion of an unfinished build, not an unchanged model experiment. The monitor now tolerates transient du file removal and stops its own child in finally.

### comment 5913960755 — 2026-09-30T15:03:45Z — Casey-allard

Measured adjustment: the cached continuation stopped after 56.042 s when core compilation briefly reached 2,134,480 KiB sampled RSS, just above the initial 2-GiB estimate; tests still NOT_RUN. Prospectively raise this direct compilation allowance to 3 GiB (one worker), retaining the existing total 1,500-second build budget, cache/floor and critical-pressure limits. This is an estimate correction supported by observed compiler memory, not a machine-wide hold or a test-criteria change. The remaining build allowance is about 842 seconds. No runner or other lab is blocked.

### comment 5914260281 — 2026-09-30T15:20:32Z — Casey-allard

**Executed validation at dcb8b107: 20 focused checks passed.**

- 17 registered Track B CPU tests: conversion/cache handling, shared-model causality and batch isolation, language-loss gradient across frozen layers, layer capture, harmonic feature identities/gradients/positivity, and generation.
- 3 parity-driver logic tests: all-cell finite checks, registered coverage/mode counts, and storage parser.
- No full model checkpoint or Metal qualification was run. Full CPU+Metal all-logit1e-4 parity remains required before B2. Unregistered B2 drafts remain NOT_RUN.

[Actual test logs, compiled binary hashes, source binding and independent source review](https://github.com/UOR-Foundation/uor-r4/tree/c9d8b8eeda72434f3b1f4a085e4877f8a678736a/validation/track-b/dcb8b107).

The internal cache compiled successfully after the owner's instruction to finish useful work instead of stopping at conservative estimates. All earlier interrupted attempts remain in the receipt. Afterward Codex removed two verified inactive incremental-cache directories; observed physical free space increased by 628,384 KiB (about0.60GiB). Warm dependencies, binaries, source and unique research were preserved. Next is Metal-specific validation and the unchanged checkpoint parity; no repeat of the completed CPU tests is needed absent a relevant change.


### comment 5914396545 — 2026-09-30T15:27:58Z — Casey-allard

Metal compilation succeeded, but the existing floor test FAILED at d32/L3/perturbation0.01: actual1.5497062122449279e-6 versus expected1.0000000117348499e-6 (unchanged tolerance5e-7 plus the declared relative term). This is a real scoped numerical failure, not infrastructure. Next is one tiny cross-backend diagnostic: identical normalized F32 rows, CPU/Metal feature construction, F64 host dot of each, CPU/Metal matmul and Metal elementwise reduction. No delta, arm or tolerance changes. The diagnostic is explicitly ignored by ordinary tests and does not replace acceptance. Full dense checkpoint parity is a separate prerequisite; no model-quality conclusion from this feature fixture.

### comment 5915099564 — 2026-09-30T16:10:10Z — Casey-allard

Shape-preserving diagnosis completed; source and result are pushed to PR1518 at `884ee22aafe378b853add44cc4cefcdb628cad74` (measured diagnostic source `34371f15de2176a237a913a6944a95be07baec08`).

- Reference/shared CPU/shared Metal final logits all reproduce their corresponding sealed-parent window **bit for bit**: zero differing bits across 393,216 elements per path.
- Original fixed eight-token window and tensor shapes retained. Captured all 30 layers, 720 Q/K/V comparisons per backend, and reference residuals.
- At worst token position 5, CPU Q/K/V maximum errors rise from approximately `5.67e-5 / 6.68e-5 / 7.51e-5` at layer 11 to `2.24e-3 / 3.33e-3 / 5.42e-4` at layer 12. Metal shows the same amplification boundary.
- This localizes an interval containing layer-11 attention/output/residual/MLP and layer-12 normalization/projections/rotation. It does **not** identify an isolated MLP or normalization defect.
- Warm build 4.03 seconds; diagnostic 86.03 seconds, peak process RSS 1,278,197,760 bytes. Fresh report sealed and verified. Reviewer found a status-only gap for mismatched anchors; successor now emits `ANCHOR_MISMATCH`/exit2, compile passed in 4.06 seconds. All anchors in the actual measured run matched.

[Executed metadata, manifest and logs](https://github.com/UOR-Foundation/uor-r4/tree/7adcdbec9bffd74c38e60a6c35d190d688e1b801/validation/track-b/34371f15-layer-trace). [Readable result and retained identities](https://github.com/UOR-Foundation/uor-r4/blob/884ee22aafe378b853add44cc4cefcdb628cad74/docs/evidence/track-b-layer-trace-2026-09-30.md).

Next: replay layer-12 RMS/QKV using all eight saved reference post-layer-11 residual rows and identical weights. Separate local normalization/projection error from propagated residual error; if local error is small, inspect block 11 using saved post-layer-10 inputs. These saved traces avoid another full exact-reference run. Original parity remains failed; no threshold change, B2 fitting or language qualification. No Codex job remains running.


### comment 5970937858 — 2026-10-03T16:12:01Z — Casey-allard

Owner-directed delivery reconciliation: retained source and negative parity diagnostics remain preserved on this draft. Track B stays parked under D18/current plan; failed complete 1e-4 parity is not fixed by merging. A future evidence-only extraction can be delivered on fresh main without activating numerical drafts. This audit neither reruns reference compute nor treats this draft as accepted native chat.

### comment 5972526115 — 2026-10-03T19:04:42Z — Casey-allard

**Merged current main into this parked draft: `8f9d49a9`** (a merge, no rebase or force push; the owner approved the push). Claude lab, prompted by the Auto-fix conflict report.

**Conflicts**
- `crates/uor-r4-training/src/lib.rs`: kept both module declarations (`temporal_compiler` from main, `track_b` from this branch).
- `STATUS.md`: took main's 1 October version. It supersedes this branch's 30 September lab table and already records Track B as parked.

**Checks at the merged tree** (Runpod Linux x86, rustc 1.97.1, `git archive` of `8f9d49a9`)
- `cargo fmt --all --check`: exit 0.
- `cargo check --workspace --all-targets --exclude uor-r4-workbench`: exit 0.
- `cargo test --release -p uor-r4-training --lib -- track_b temporal_compiler`: 21 passed, 1 failed, 1 ignored.
  - The failure is `track_b::harmonics::tests::packed_features_backpropagate_through_normalization_and_match_finite_difference` (`autodiff=0.040845722, finite_difference=0.041186806`).
  - It is **pre-existing, not merge-caused.** The same test at the pre-merge head `2cec2494`, on the same machine, fails with identical values. The merge does not touch `track_b/`.
  - It may be specific to the platform (earlier passing runs were on the M1). No tolerance was changed.

The failing `Validate delivery evidence (status publisher)` check is the advisory receipt gate, not a required check. The draft stays parked; no Track B work resumes.


### comment 5972899910 — 2026-10-03T19:53:19Z — Casey-allard

**Merged current main again: `aa0628df`** (a fast-forward merge; no rebase or force push). After #1648–#1651 landed, `crates/uor-r4-training/Cargo.toml` conflicted. Resolution: keep main's new `cuda` feature (#1649) and this branch's `cpu-accelerate` with `candle-transformers/accelerate`. `Cargo.lock` is main's lock, re-resolved offline with `cargo metadata` (+42 lines: `candle-transformers`, `fancy-regex`, `serde_plain`).

**Checks at `aa0628df`** (Runpod Linux x86):
- `cargo fmt --all --check`: exit 0.
- `cargo check --workspace --all-targets --exclude uor-r4-workbench`: exit 0.
- `cargo check --release -p uor-r4-training --features cuda --all-targets` (CUDA 12.6): exit 0.
- `cargo test --release -p uor-r4-training --lib -- track_b temporal_compiler`: 21 passed, 1 failed. The failure is the same pre-existing `track_b::harmonics::…finite_difference` case (identical values at the pre-merge head `2cec2494`).

The draft remains parked.

### comment 5973819439 — 2026-10-03T21:52:10Z — Casey-allard

**Merged current main again: `88c1f5f2`** (no rebase or force push; Auto-fix conflict report).
- `current-state.md` takes main's split page (#1657).
- This branch's Track B edits (the 29 September conversion section and the parity sentence) are applied to `docs/history/current-state-2026-09-25-to-2026-10-02.md`, where that text now lives. Same-directory links get the `../integration/` prefix.
- `Cargo.toml` keeps both the `uor-matmul` dev-dependency and #1660's `[[bin]]`.
- `Cargo.lock` resolves with `--locked`.

**Checks on the pod:**
- `cargo fmt`: exit 0.
- Workspace check: ok.
- `--features cuda,parquet-input` check: ok.
- `track_b` tests: 16 passed, 1 failed. The failure is the same pre-existing `harmonics …finite_difference` case with identical values.

The draft remains parked.

### comment 5976691009 — 2026-10-04T04:48:23Z — Casey-allard

Closed on owner direction (4 October): Track B is parked. The branch `codex/track-b-conversion` stays on GitHub with its evidence and is synced with main as of `88c1f5f2`. Reopen to resume.


---
_Comment/review ids in this file: 5898789702, 5913717061, 5913829718, 5913917802, 5913960755, 5914260281, 5914396545, 5915099564, 5970937858, 5972526115, 5972899910, 5973819439, 5976691009_

