# REPORT: m1-normalization-mechanism, claude-60 (wire the `C_unscaled` arm into the GPU read kernel)

Lab: claude. Item: [m1-normalization-mechanism](../../../../roadmap/items/m1-normalization-mechanism.md) (#2842). Base: `fb6156b46`. `References #2842`; the item stays `ready`.

## What did not work, first

1. **No command can run the configuration yet.** The kernel and the device now support `C_unscaled`, but two links are missing:
   - The training driver and CLI have no way to choose the mode. `training.rs:~2598` builds devices with the default.
   - `Device::snapshot()` refuses a non-default mode. The snapshot schema has no mode field, so a `C_unscaled` model would otherwise reload as `snap_n` under the treatment's label. A training run needs checkpoints, so `C_unscaled` cannot train until the schema carries the mode.

   So prerequisite 1 of the claude-57 `NOT_FIT` is only partly cleared. The next unit:
   - carries `normalization_mode` in the snapshot schema, omitted when it is the default so that every existing snapshot's bytes stay the same;
   - adds the driver and CLI flag.
2. **The GPU test has not run.** `device::unit_snap_device_tests::unit_snap_device_word_runs_and_refuses` is `#[ignore = "requires a CUDA GPU"]`. It needs one pod, and Claude asks Casey before leasing it. The PR stays a draft until then.
3. **Merging rotates `source_sha256`.** `device.rs:408-416` hashes `device.rs` and `discrete.cu`, and restore refuses a mismatched snapshot (`:1106`). Snapshots made before the merge restore only at their own commit; runs pinned to their commit are unaffected. This is the first kernel-source change since #2601 (17 Oct). The merge timing is flagged to the PM on #2842 (6366985714).
4. **Pre-existing failures outside this change:**
   - **The k16 instrument.** `docs/labs/k16-timing-2026-10-15/instrument` fails `--all-targets`: its test target uses `include_str!` on two files that have moved to `archive/labs/`. It checks clean without `--all-targets`. The fix is outside this item's scope.
   - **Workspace clippy** stops in `uor-r4-source-interface` (`lib.rs:383/397/416`).
   - **Crate clippy** (`--no-deps`) still reports untouched lines: `d11_export.rs:413`, `normalization_mechanism.rs:687`, `training.rs:1875`, and dead code without `cuda`.
   - **`cargo fmt --check`** flags only `tests/flock_coupling_controls.rs`, which this PR does not touch.

## What changed

| File | Change |
|---|---|
| `crates/uor-r4-discrete/src/discrete.cu` | Spec word `s[12]`. `0` keeps `snap_n`; `1` sets `n := 1` at the three snap sites: tree (`3`), consensus (`found`) and output (`terms`). `discrete_validate` sets status 10 for `s[12] > 1`, and for `s[12] == 1` outside Flock or Relation. |
| `crates/uor-r4-discrete/src/normalization_mechanism.rs` | `NormalizationMode` derives `Default` (`ScaleMatchedSnapV1`) and gains `is_scale_matched_snap()` and `device_word()` (0, 1, or `Err` for `UnnormalizedControlV1`, which needs a ℤ⁸ carry). Adds four `unit_snap_word_*` tests. |
| `crates/uor-r4-discrete/src/device.rs` | `Binding` carries the mode; it derives no `Serialize`, so no serialized form changes. `new_with_normalization` refuses the unit word with `ReadSpec::Route`. Both spec-array sites append `s[12]`. `snapshot()` refuses a non-default mode. Adds the ignored GPU test. |
| `docs/labs/m1-normalization-mechanism/README.md` | A section on the wiring and on what is still missing. |
| this folder | ORDER (committed first) and this REPORT |

## Measurements

- **Mapping tests.** `cargo test -p uor-r4-discrete --lib unit_snap_word`: 4 passed.
- **Negative gate.** With `UnitSnapControlV1 => Ok(0)` planted, `unit_snap_word_control_is_one` fails (3 passed, 1 failed). After restoring, 4 pass.
- **Crate tests.** `cargo test -p uor-r4-discrete --lib`: 221 passed, 0 failed, 1 ignored (pre-existing).
- **Stub-nvcc check.** `NVCC=<stub> UOR_NVCC_ARCH=compute_120 cargo check --offline -p uor-r4-discrete --features cuda --all-targets` exits 0, so the GPU test compiles.
- **Ratchets.** `d23` 4,451, `d25` 93 and `d32` 31, each equal to its baseline.
- **Nothing else reads `s[12]`.** Only `discrete.cu` contains `s[12]` across every `*.cu`, `*.cuh` and `*_source.rs`. The spec array is built only at `device.rs:~899` and in `gpu_spec()` (`~1164`).

## Checks run at the head

`cargo test -p uor-r4-discrete --lib`, the stub-nvcc `cargo check --features cuda --all-targets`, `cargo clippy -p uor-r4-discrete --no-deps` (touched lines clean), `uor-harness ratchet d23|d25|d32`, `loop report --check` and `loop pr-check --local`.

## Skills used

- **`scope-guard`:** the change stays in `crates/uor-r4-discrete/**` and the lab folder. The k16 instrument break and the clippy debt are recorded, not fixed. `C_raw` and the sheet are untouched.
- **`solution-integrity`:** the snapshot refuses a mode it cannot record, rather than silently relabelling a control model as the treatment.

## Still open

- **GPU parity run.** One pod, `cargo test -p uor-r4-discrete --features cuda --lib unit_snap_device -- --ignored`. It needs Casey's approval.
- **The next unit:** the snapshot schema carries the mode (defaults omitted), plus the driver and CLI flag. Without them no command runs the `C_unscaled` arm.
- **Merge timing** against snapshot-pinned runs: the PM's call (#2842, 6366985714).
- **The k16 instrument's archived `include_str!` paths:** a separate fix.

## Plain summary

The GPU read kernel can now run the "unit snap" control: the same read with the snap's multiplicity fixed at 1. A new spec word selects it, and the default path is unchanged. A training run of that arm still needs the checkpoint format and the driver to carry the mode, which is the next unit. The GPU test is written and compiles, and waits for an approved pod.
