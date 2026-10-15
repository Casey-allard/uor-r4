# Stage-A admission instrument — D-A (2026-10-15)

This delivers the evaluation-only dependency in the [admitted sheet](../../harness/prep/m1-discrete-rung0-admission-2026-10-15.md). The standalone Rust instrument restores four hash-pinned seed-1 checkpoints and evaluates each at K=8..16. It has no proposal, apply, resume, baseline-training or owner-release command. Every derived device starts from the original checkpoint. Training and M2 remain HELD.

**Status: source implemented; host metadata tests pass; CUDA build and all 36 GPU evaluations NOT_RUN.** Claude's [design FIT](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6192687624) and the [Stage-A-only owner GO](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6192566334) are recorded. The remaining chain is Claude exact-source review → Kimi's bound instrument packet → evaluation. This source record is not a measured ceiling or CUDA qualification.

Line: starling read / exact addressed memory; count 0/3 unchanged; headline unchanged.

## Contract and identity

- The nested Cargo workspace under `instrument/` changes no existing model, kernel, driver, evaluator, training path or root Cargo file. It explicitly retains the repository's patched `cudarc` and `arrayref`. This also avoids Antigravity's concurrent `ReadSpec::Relation` source surface.
- `check FILE` authenticates the entire completion-checkpoint file against the four sheet pins before parsing its wrapper, cursor and seed/K metadata. All four inputs and their common lineage are checked before a CUDA context is created by `run`.
- Each evaluation derives a copy: only `config.spec.kept_cells`, snapshot `spec.kept_cells`, the resulting serialized-model hash, and the disclosed operational driver hash change. Parameters, seed, epoch, cursor, corpus bindings, model source, PTX and geometry pins remain original. A full post-evaluation snapshot equality check and zero-update/epoch checks refuse mutation.
- The existing driver hash at base `837306f53` is `70b45863860c25cf3579e76367b785ea4830edbc50e3902ef4a5ca2335bf7676`; the stored Q4 driver hash is `cb090e85e78805acd2b2347373340168a9e2f87aaf72a1dcb42f091e9ff1f7b4`. That operational difference already exists on main. D-A changes neither existing hash input nor compiled-model source. Both original and derived lineages are retained in each receipt. Model/PTX admission is never bypassed or rewritten.
- The instrument's `identity` command reports its executable, Rust source, manifest, lockfile, shared report helper and existing driver hashes; a CUDA build additionally reports compiled identity. It creates no CUDA context. Kimi must record these actual values from the final build, not reuse the host executable hash.
- The evaluator reads the pinned TRAIN corpus to reconstruct the masked count reference and the pinned DEV corpus for evaluation, using the existing GPU implementation. No learning occurs. This corrects the sheet's shorthand that Stage A reads only DEV/artifacts: both corpora must be present at their original config paths, with unchanged hashes. Claude must include this clarification in source review. No data relocation override is introduced.

## Controls and outputs

All four K8 evaluations finish and reproduce the five registered counts exactly before K9 starts. Values below are ordered top-1, top-8, strict wins, admitted strict wins, target-admitted rows:

| Cursor | Expected K8 counts | Authority |
|---|---|---|
| 4,101 | 96 / 1090 / 177 / 41 / 2201 | [Q1 verification](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6174365494) |
| 8,202 | 94 / 967 / 178 / 42 / 2131 | [Q2 verification](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6178914118) |
| 12,303 | 96 / 953 / 214 / 44 / 1954 | [Q3 verification](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6182703265) |
| 16,404 | 149 / 747 / 105 / 65 / 1920 | [Stage-A sheet](../../harness/prep/m1-discrete-rung0-admission-2026-10-15.md#stage-a-design) |

The CPU audits stored GPU observations only: 224 batches (223×64+15), 14,287 rows, B3 nesting, row/aggregate agreement, constant count reference, and per-row monotone target admission as K increases. Strict wins retain the zero-rival exclusion; their curve is reported without asserting monotonicity.

The output directory must be new, have an existing parent outside the checkpoint directory, and have no sealed-report ancestor (`attempt.json` or `manifest.json`), including through symlinks. The shared `report_output::claim`/`seal` protocol reserves and seals it. Records are named `evaluation-04101-k08.json`, etc.; `report.json` holds the complete 36-point table and smallest passing K. Its G-A FAIL scope is eval-time widening of K8-trained checkpoints only; K>8-trained lineages remain untested. No outcome automatically releases Stage B.

Every derived record keeps the original checkpoint SHA, original and derived lineage/model hashes, executable/source identities, metrics and all evaluation fields. Repeated row names are represented once in `row_columns`; `rows` contains the corresponding arrays, with `batch_row_counts` and `batch_aggregates` retaining batch boundaries and aggregates. Unknown row or batch fields refuse rather than disappear. This is a lossless receipt encoding, not a model compression claim. The synthetic fixture fits; **real output size is unmeasured**. A runtime 49 MiB record cap reserves 1 MiB for the summary/seal under the sheet's 50 MiB budget.

Any pin, control, snapshot, accounting, schema or storage mismatch returns nonzero. Once the report directory is claimed, the instrument attempts to write and seal `INSTRUMENT_FAULT` with the error; already-written records remain. Filesystem failures can prevent sealing and are visible errors. A fault is not a mechanism finding. No wider-K work follows a failed K8 control.

## Build and operation handoff

Run from the exact reviewed checkout in an owned worktree. Before CUDA build, restore the recorded toolchain: the [completion build](../rung0-completion-release-2026-10-14/README.md#open-items-before-any-release) used nvcc 12.8.93, Rust 1.97.1 and `UOR_NVCC_ARCH=compute_120` on RTX 5090. Reproduce the compiled-model/PTX pins; if they differ, diagnose the build instead of rewriting checkpoint bindings. CUDA compilation has not been checked on this laptop (no nvcc).

```sh
export UOR_LAB=codex
export CARGO_TARGET_DIR=local/stage-a-target
stage_a_manifest=docs/labs/stage-a-admission-instrument-2026-10-15/instrument/Cargo.toml
cargo test --locked --offline --manifest-path "$stage_a_manifest"
cargo clippy --locked --offline --manifest-path "$stage_a_manifest" --all-targets -- -D warnings

# Kimi's instrument build, after Claude exact-source review; no training command.
UOR_NVCC_ARCH=compute_120 cargo build --release --locked \
  --manifest-path "$stage_a_manifest" --features cuda
local/stage-a-target/release/stage-a-admission identity
local/stage-a-target/release/stage-a-admission check \
  /workspace/uor-r4/rung0-completion-20261014/seed1-attempt-1/checkpoint-16404.json

# Only after packet identities, artifacts, corpus paths and pod admission verify.
# /workspace/stage-a-output must exist; sweep-1 must not exist.
local/stage-a-target/release/stage-a-admission run \
  /workspace/uor-r4/rung0-completion-20261014/seed1-attempt-1 \
  /workspace/stage-a-output/sweep-1
```

Preserve/upload the whole sealed output and identity record before pod cleanup. Kimi publishes all four trajectories and the G-A disposition. The recorded Stage-A owner ceiling is **< $0.50 all-in, one RTX 5090**, not a training release.

## Observed validation

- Eight host tests passed: pinned-input refusal; exact K8 controls/monotone admission; output ancestry/symlinks; exclusive claim and sealing; derived metadata preserving parameter payload/cursor/pins; compact receipt round trip and unknown-schema refusal; B3/zero-rival/aggregate/bounds accounting. These tests do not execute a model or CPU reference.
- Offline locked build and instrument Clippy with `-D warnings` passed. The existing non-CUDA dependency emits 32 dead-code warnings; this is not a warning-free workspace claim.
- The retained Q4 file was read-only authenticated locally: SHA `4477eb5c2480b0b85cc94036a2e15b0d22b654e25deee9c8717aa6e30f8ef7bf`, cursor 16,404, 407 accepted updates, `cuda_context_created: false`. This verifies artifact metadata, not its evaluation result.
- D23 and D25 source ratchets passed: 1641 historical softmax sites and 93 dense-product sites respectively, no increase. Neither is a whole-runtime qualification.
- A parallel source review found no blocking defect; its unknown-batch-schema nit was fixed. CUDA-specific compilation, PTX admission and the full-panel sweep remain **NOT_RUN**, pending the explicit review/build chain above.
