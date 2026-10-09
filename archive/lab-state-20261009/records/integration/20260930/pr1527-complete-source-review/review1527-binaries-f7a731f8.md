# PR #1527: two S2 binaries, exact-head source review

**Verdict: REQUEST_CHANGES for the candidate NLL evaluator and evidence/claim boundaries below.** This supplements, and does not repeat or supersede, [the existing helper/documentation review](review1527-f7a731f8.md).

Reviewed 2026-09-30 by the independent `history_science` reviewer. GitHub confirmed head `f7a731f8f88376ae9be270b230cf98c3dace4f8b`, base `ea03435808c67791b76ae0e9868978e3188cba04`. Full source read: `s2-behavior-codec.rs` (1,296 lines) and `s2-behavior-distortion.rs` (1,083 lines), with targeted reads of `stack_dialogue::{development,reply_panel,check_panel}`, `StackModel::{load,hidden,forward,target_nll}`, capture sites and head selection. No full-history or unrelated source audit was repeated.

## Required corrections

### P1 — Codec development NLL uses a different output head from the candidate being reported

In [`s2-behavior-codec.rs:1097–1098,1133–1160`](https://github.com/UOR-Foundation/uor-r4/blob/f7a731f8f88376ae9be270b230cf98c3dace4f8b/crates/uor-r4-training/src/bin/s2-behavior-codec.rs#L1133), `development(&diag_model, ...)` computes both baseline and candidate NLL. However, the exported head's `MapSpec` deliberately has `var_name: None` (`585–590`), so the candidate replacement loop never installs a head into that model. The separately dequantized `diag_base_head` or `head_comp_tensor` is only passed to `greedy_reply_with_head` afterward (`1154–1160`).

The actual callee confirms the mismatch: [`stack_dialogue.rs:237`](https://github.com/UOR-Foundation/uor-r4/blob/f7a731f8f88376ae9be270b230cf98c3dace4f8b/crates/uor-r4-training/src/stack_dialogue.rs#L237) calls `model.target_nll`, which calls `forward` (`geometric_stack.rs:1031`); float-mode `Params::head` returns `embedding.weight` (`2092–2095`). This diagnostic model has unit final norm and the dequantized **embedding**, whereas the exported output head separately includes final-norm folding and its own quantization.

Consequences are source-determined: the head-only compensated arm cannot change this reported NLL at all, because it changes no variable that `development` reads. The combined head+recurrence arm's NLL measures the recurrence change without its advertised head change. Even baseline `response_nll_161` is not the same surrogate model used for its greedy replies. Those values are written into candidate, baseline and selected report fields (`1211,1261,1273`).

**Repair:** calculate response NLL using the same explicit head and body as the corresponding surrogate reply evaluator, or use an existing artifact-derived reference that faithfully represents both. Do not overwrite the tied embedding to simulate the output head, since that would also change the input embedding. Add a tiny focused fixture in which changing only the output head changes target NLL while leaving input embeddings fixed. Preserve old values as outputs of the old evaluator; do not silently relabel historical NLL as corrected candidate evidence.

### P2 — The codec's “served float” / “kernel agreement” comparator retains unquantized non-matrix parameters

[`s2-behavior-codec.rs:665–699`](https://github.com/UOR-Foundation/uor-r4/blob/f7a731f8f88376ae9be270b230cf98c3dace4f8b/crates/uor-r4-training/src/bin/s2-behavior-codec.rs#L665) installs decoded artifact matrices into `diag_model`, but copies bias, decay, convolution, age, beta and offset values directly from the original float checkpoint. The candidate artifact, by contrast, retains all baseline tables (`301–327`). Thus this is a mixed reference: artifact matrices with original floating-point scalar/table parameters. It is useful for isolating weight-map effects, but does not isolate the numerical kernel from complete exported-parameter effects.

The binary currently calls this “served float forward,” derives `survives_export` from its reply comparison (`714–725,1171–1173`), and writes `kernel_agreement_turns` (`1254`). Disagreement can include scalar/table quantization differences, so the kernel-only attribution is unsupported.

**Repair options:** use an artifact-derived reference incorporating its actual scalar/table values for a served-reference comparison, or label the current quantities explicitly as integer-versus-matrix-dequantized/original-scalar surrogate agreement. The latter is a sufficient narrow correction if that is the intended diagnostic. The distortion binary's mixed intervention model is appropriate to its declared weight-map-only attribution, provided it is not promoted to whole-export/kernel evidence. No new model evaluation is requested to change the claim scope.

### P2 — Preserve the computed row-level behavior and the remaining result-determining inputs

Both binaries compute actual reply IDs, but discard their detailed observations. Codec creates `float_panel`, `baseline_int_panel`, each candidate panel and each surrogate panel, then saves only aggregate counts/NLL in `report.json` (`1227–1283`). Distortion stores teacher prefixes/replies and compares every intervention, then saves only aggregate rankings (`1037–1071`). Consequently a sealed output root from these programs cannot independently show which turns match, which previously matching turns regress, or which actual reply generated a reported flip. These observations have already been computed; retaining them needs no additional model work.

In addition, both identify the input model solely by `model.safetensors` hash (`codec:415,1236–1237`; `distortion:362,1045–1046`), although `StackModel::load` consumes `config.json` and its configuration changes execution (`geometric_stack.rs:1116–1141`). Distortion records only the heldout token **path**, not its digest or the `windows`/`WINDOW_TIME` selection specification (`1037–1066`), although these determine every activation moment (`633–664`). This gap is separate from the tokenizer and executable fields already repaired in the prior review.

**Repair:** save the already-generated per-turn token IDs and match outcomes keyed by request/turn and arm, along with the actual head/body/reference identities and development response selection used. Bind the model config and, for distortion, heldout token bytes plus the window count/length/selection rule. Use the existing claimed report root and sealing mechanism. Where old outputs do not exist, state row-level behavior or input binding as unavailable; do not manufacture it or require an automatic replay of the historical study.

### P2 — Distortion accepts zero windows and reports missing activation evidence as zero error

`windows` accepts any parsed `usize`, including zero (`s2-behavior-distortion.rs:137–140`). With `windows=0` and a sufficiently long token file, the capture loop does not run, `moments` remains empty, and all maps that require a capture site get no activation result (`638–662,684–745`). Ranking subsequently converts those missing results to `0.0` (`986–994`). The study then finishes and seals a report containing numerical zero activation errors despite observing no activations. Also, a token file shorter than 256 reaches unchecked subtraction/slicing (`636,644`).

**Repair:** reject zero windows and a token stream shorter than `WINDOW_TIME` before capture. Treat a missing required capture site as an error or explicit unavailable metric rather than zero. These are cheap argument/coverage regressions; no model or broad suite is necessary to establish the guards.

## What the inspected source does support

- Both binaries call `report_output::claim` before loading models (`codec:411`, `distortion:356`) and call the repository seal and complete verification functions on the completed/failed attempt (`codec:1293–1295`, `distortion:1080–1082`). They do not overwrite an existing sealed output root through this code path. Returning the original execution error after sealing is not a success claim.
- Codec actually writes each candidate `.lut` and independently loads it with `IntegerStackModel::load` before generating replies (`1112–1130`). Its integer reply counts are therefore structurally tied to loaded candidate artifacts, subject to the unexecuted source-review limitation and the missing retained row details above.
- Distortion compares interventions under fixed prefixes generated by its float reference; codec uses each model's own free-running history through `reply_panel`. These are different, legitimate diagnostic quantities. Neither is an authored-answer correctness or general language-quality evaluation.
- Capture sites provide pre-gain normalized inputs for folded matrices and the actual MLP-down input (`geometric_stack.rs:616–629,826–839,959–960`). The activation-moment construction and quadratic form follow that intended coordinate convention. This review found no reason to replace that mechanism with a broad new audit.
- Runtime `git rev-parse HEAD` is explicitly labeled unverified build-source context, and the binary hash is recorded. These are appropriately scoped observations, not a proven source-to-executable link.

## Nonblocking scope and validation

The codec's hard-coded six-layer arms and 14-match historical assertion make it a study-specific evaluator, not a generic model benchmark. Its `selected` arm optimizes turn matches only; retain that as a diagnostic selection, not an acceptance/promotion gate. General CLI hardening, generic architecture support, new performance work and a complete new evaluator framework are outside this review. The permissive zipped JSON comparison is not raised as a separate blocker here because the actual codec callers generate panels from the same validated request list.

The scoped `git diff --check` for both binaries passed. Cargo, compilation, tests, model generation and GPU work: **NOT_RUN** under the production hold. No source edits, GitHub writes, issue closure or merge action occurred. Historical numeric reports remain retained; the findings restrict what the affected metrics establish and define bounded source corrections, rather than authorizing a repeated research campaign.
