# REPORT: float-kernel-fusion, deepseek-1 (the deferred fused-cost experiment)

Lab: deepseek. Item: [float-kernel-fusion](../../../../roadmap/items/float-kernel-fusion.md)
(#3678). [ORDER](./ORDER.md). Base: `f66f023d1`. Plan/FIT head: `820a1103b`; the final
delivery head is bound by the PR's exact-head review.

## What did not work, first

- **No GPU measurement ran, so there is no speed or default result.** The newest project
  direction, recorded by claude-307 after the owner answered "Yes, pipeline first", keeps new
  training runs behind the current model's export, integer-serving and parity path
  ([REPORT](../../one-model-end-to-end/claude-307/REPORT.md)). The experiment is registered as
  `NOT_RUN`, not PASS.
- **Two fresh FIT reviews rejected earlier drafts.** They found a non-terminating extension
  branch, an equality gate that omitted extension runs, bare test filters that could select zero
  tests, an invalid no-`nsys` launch-count lower bound, an understated cost, and a stale claim
  that more long training was imminent. All are repaired in the current sheet and ORDER.

## What changed

- **Size:** 5 files, +505/-11: docs 4 files +498/-11; other 1 registry file +7/-0.
  Tests, code and generated files are 0. The ORDER estimated no numeric line count, so the ratio is
  UNAVAILABLE.
- **The sheet is launch-ready but compute-gated.** Every default-on, default-off and one-extension
  outcome now has a finite result. Extension runs are named `A2, B2, B3, A3`; their model trees
  and normalized progress join the exactness gate; gain and noise are pooled over twelve windows
  per mode.
- **The instrument commands are non-vacuous.** The four exact `#[ignore]` test paths are named,
  and each must report exactly one passed test.
- **Launch counts are isolated from checkpoint evaluation.** Paired 1-step and 32-step smokes
  have the same two mandatory checkpoints, so their call and `nsys` differences isolate 31
  training steps. Without `nsys`, the report keeps observed calls, labels the source tensor-op
  count as forward-only, and marks total launches UNAVAILABLE.
- **No source, kernel, model, data or serving artifact changed.** The existing
  `--fused-quantize` option remains explicit and defaults off.
- **The FIT record** is in the sheet plus the live DeepSeek receipt:
  `TEST FITNESS: FIT - M2 #2030`, reviewer `codex`, date 7 November 2026.

## Measurements

- **No model or device measurement ran.** The only result is the documentation and readiness
  verdict: FIT at `820a1103b`, with compute held for the current project direction.
- **Host verification:** 298 float-train library tests passed; every-target CPU `cargo check`
  passed; Clippy reported only the pre-existing crate and vendor warnings.
- **No default changed:** `--fused-quantize` still defaults off. A later pre-run review and the
  registered short/long-context run are required before the training default can change.

## Checks run at the head

- `cargo test --release --locked -p uor-r4-float-train --lib`, with
  `CARGO_TARGET_DIR=local/deepseek/fusion-target`: **298 passed, 0 failed**.
- `cargo check --offline --locked -p uor-r4-float-train --all-targets`, with
  `CARGO_TARGET_DIR=local/deepseek/fusion-check`: **passed**.
- `cargo clippy --offline --locked -p uor-r4-float-train --lib --no-deps`: completed with only
  the crate's pre-existing warnings; no source changed here.
- `uor-harness prep-check`: the sheet reads **FIT** with all required sections.
- `git diff --check`: passed.
- Not run: CUDA tests, fused/chain training, training timing, `nsys`, model scoring, or a pod.

## Skills used

No system-provided Codex skill applied. The repository's CHARTER, AGENTS, loop, experiment
pre-registration and GPU-compute rules were applied directly. The fresh FIT review was run as a
separate read-only Codex process.

## Still open

1. When pipeline-first clears, run the D67 pre-run review at the then-current head and execute
   the sheet's short/long-context experiment.
2. Turn the training default on only after both shapes clear `2 x noise`; otherwise keep it off.
3. Measure the item's cached-decode half separately. This unit is training-time only.

## Plain summary

The fused quantizer implementation has a measurement plan that now passes independent FIT
review, including exact checkpoint equality, robust short/long-context timing, and an honest
launch-count method. It deliberately spent no GPU time because the owner’s current instruction is
to finish the training-to-serving pipeline before starting more training. No model behavior or
default changed.
