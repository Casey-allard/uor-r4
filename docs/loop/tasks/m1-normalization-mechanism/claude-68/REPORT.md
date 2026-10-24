# REPORT: m1-normalization-mechanism, claude-68 (trace scorer for the structural-norm bar term)

Lab: claude. Item: [m1-normalization-mechanism](../../../../roadmap/items/m1-normalization-mechanism.md) (#2842). The item status is unchanged; no compute is cleared and nothing is released.

## What did not work, first

1. **The command has never run.** `normalization-dev-structure` is CUDA-only, it drives the device over DEV, and no GPU was used: it is compile-checked against the stub nvcc only. Running it is part of the sheet's released run (FIT, then PM dispatch, then Casey's release). The pure scorer it calls is unit-tested.
2. **No unit test covers the command itself.** It restores a device and runs `forward_traced`. Its restore and batching code is copied from `capture_dev` and checks the same things.
3. **The ORDER's signature was wrong.** It named a `DEV_EVAL` argument, but rows come from the DEV corpus exactly as in `capture_dev`, so the command is `CONFIG CHECKPOINT OUT`. The ORDER is amended.
4. **The builder handed back partway.** It reached its context limit after the pure scorer. A second builder wrote the command from the first one's notes, so the work landed in two commits.
5. **`driver_source_sha256` moved** (`attention_preflight.rs`): `f8cdd526…` → `63b276f7…`. It is re-pinned in `prime_zeta.rs`, and the three bins that pin it pass.

## Cancel probe

At the base, no command reported per-stage root validity, norms or endpoint distance over DEV. #3119 marks this bar term `UNSCORED`. The task stands.

## Kernel

**No kernel change.** The Flock/Relation forward already traces, per (row, block), the final output root (`rec[1]`) and each level's consensus root (`lv[2]`) (`discrete.cu` 121–128, 223–268). `Forward::vectors()` returns the real output blocks. The command checks that the trace layout constants equal `device::FLOCK_TRACE_*` before reading.

## What changed

| File | Change |
|---|---|
| `crates/uor-r4-discrete/src/normalization_mechanism.rs` | `structural_summary(trace, stride, rows, blocks, stages, outputs)` and `StructuralSummary` (with a checked `absorb`). |
| | **Stage outputs.** The final outputs come from the real output blocks. Every level whose kept count is non-zero contributes its consensus root, checked through `orbit_root_coords`. |
| | **Root and norm checks.** A root must be an index in [0, 16) (`ABSENT` and ≥ 16 are invalid). A norm failure is a vector that misses L1 = 4, L2² = 8 or L∞ = 2; norms are measured, not assumed. |
| | **Endpoint distance.** Each row's endpoint distance to all 256 cells is binned over {0, 4, 8, 12, 16}, with an off-set count. `met` requires zero failures. |
| | Two tests. |
| `crates/uor-r4-discrete/src/attention_preflight.rs` | `normalization_dev_structure` (CUDA) plus a non-cuda stub. The steps run in this order: |
| | 1. the normalization config lock; |
| | 2. `PreparedDriver`, the checkpoint check, and a compiled-identity check through the shared `check_compiled_identity`, which `dev_inputs` now calls with the same "DEV B3" text; |
| | 3. a restore checked as `capture_dev` does; |
| | 4. DEV batches tied to `dev_row_identities`; |
| | 5. `forward_traced` per batch, checking that the stride is 4 + 28 × the config's stage count and that `vectors()` length is `rows × blocks × 8`; |
| | 6. a check that the snapshot is unchanged at the end. |
| | It emits schema `uor-r4.normalization-dev-structure/1` with the summary, `judged = (mode == n_snap)` and `met` (null when not judged). |
| `crates/uor-r4-discrete/src/main.rs` | `normalization-dev-structure CONFIG CHECKPOINT NEW_REPORT_DIR` and its help line. |
| `crates/uor-r4-discrete/src/prime_zeta.rs` | The re-pin. |

## Measurements

- **Every discrete target** (`cargo test --locked -p uor-r4-discrete`) exits 0:
  - lib: 240 passed, 1 ignored;
  - bins: `address-cells-codebook` 6, `prime-zeta-codebook` 7, `prime-zeta-qualify` 6;
  - tests: 7, 7 and 6.
- **New tests** (`normalization_mechanism::tests::structural_summary_*`), 2 passed:
  - `…counts_valid_stage_outputs`: exact counts, and a histogram checked against a convolution of per-endpoint root distances (sum 512);
  - `…flags_bad_outputs`: catches an `ABSENT` root, root 16, a non-root output block and an off-set endpoint distance.
- **Negative gate, run by me.** With `trace_root`'s range filter set to accept any id, `structural_summary_flags_bad_outputs` FAILED (0/1). Restored, both passed (2/2) on a clean tree.
- **CUDA stub check.** `cargo check --features cuda --all-targets` with the stub nvcc finished with no errors. Clippy, with and without `cuda`, reports nothing in the added lines, and the changed files are rustfmt-clean.
- **Ratchets at baseline:** D23 4451, D25 93, D32 31.

## Checks run at the head

The measurements above, plus `loop report --check`, `loop pr-check --local` and `git diff --check`.

## Skills used

- **`scope-guard`:** only `crates/uor-r4-discrete` and this folder. No `.cu`, `build.rs` or `device.rs` change, which was the stop condition.
- **`solution-integrity`:** the norms are measured from vectors, not assumed from root ids, and the root check fails with its fix set aside.

## Still open

- The real-GPU run, under the released sheet.
- Combining the two seeds.
- The non-inferiority margin (a PM/owner card).
- Sheet registration.

## Plain summary

The last unscored bar term, "every stage output keeps its exact norms", now has a scorer that reads the model's own trace and output. Nothing has run on a GPU yet.
