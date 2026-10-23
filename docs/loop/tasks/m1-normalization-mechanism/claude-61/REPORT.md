# REPORT: m1-normalization-mechanism, claude-61 (carry the normalization mode through snapshots, driver configs and export)

Lab: claude. Item: [m1-normalization-mechanism](../../../../roadmap/items/m1-normalization-mechanism.md) (#2842). Base: `61706a39e`, on PR #3071 after claude-60. `References #2842`; the item stays `ready`.

## What did not work, first

1. **My first two A8 attempts on the pod failed on staging. These were instrument faults, not results.**
   - Attempt 1 (`a8/`) failed because `local/driver/data/tokenizer.json` was not staged.
   - Attempt 2 (`a8-rerun/`) failed because the A8 table `address-cells/address-structured-v1.json` was not staged.
   - Both inputs are named in the A8 config. Attempt 3 (`a8-rerun2/`) staged them, with the tokenizer hash checked against the config pin `d36d3e87…` and the table rebuilt per the runbook, and ran clean.
   - All three attempts are kept in the results folder.
2. **My pod watcher never fired.** It ran `pgrep -f pod-3071.sh` over ssh, which matched its own `bash -c` command line, so it always read RUNNING. The job had finished at 17:07:55Z, two minutes in, and about 20 idle pod minutes (~$0.75) passed before I checked by hand. The pod's total was about 40 minutes at $2.227/h, roughly $1.48, inside the $1.10–1.70 Casey confirmed.
3. **Five other admission gates do not check the mode yet.** `attention_learning.rs:197/811`, `controller.rs:162`, `relation_qualification.rs:995` and `attention_audit_qualification.rs:1569` compare `read_initialization` but not `normalization_mode`, so a unit-snap model could pass them. The ORDER asked for no behaviour change there. The normalization sheet's arms run through the training driver, which does check the mode, so its runs are covered. Those five paths are a follow-up if a control arm is ever run through them.
4. **`validate_and_extract_cells`** (public) still accepts a non-default mode. Only `export_d11` refuses one.
5. **Merging rotates `source_sha256`,** together with claude-60. This is one rotation for the whole PR, sequenced by the PM before any A8 GPU stage (#2842, 6368080811).
6. **Pre-existing clippy debt is unchanged and not in touched lines.** It moved by line only: `training.rs:1941` (was `:1875`), `d11_export.rs:420` (was `:413`), `normalization_mechanism.rs:687`, and dead code without `cuda`.

## What changed

| File | Change |
|---|---|
| `crates/uor-r4-discrete/src/snapshot.rs` | `normalization_mode` with `#[serde(default, skip_serializing_if = "NormalizationMode::is_scale_matched_snap")]`. The golden test also checks the explicit-default hash domains and that the text has no `normalization_mode`. New: `unit_snap_control_mode_is_stored_and_reloaded`. |
| `crates/uor-r4-discrete/src/device.rs` | `snapshot()` records the mode instead of refusing. `from_snapshot`, the only rebuild-from-snapshot path, builds through `new_with_normalization`. The ignored GPU test's assertion (d) is now a round trip. |
| `crates/uor-r4-discrete/src/training.rs` | A `DriverConfig.normalization_mode` field with the same serde. `validate()` refuses `UnnormalizedControlV1`, and unit snap with a `Route` read. A mode-mismatch refusal sits at each of `read_initialization`'s three checks: `validate_checkpoint`, `admit_model` and evaluation admission. `new_model` builds devices with the mode. New: `normalization_mode_round_trips_and_checkpoint_mismatch_refuses`. |
| `crates/uor-r4-discrete/src/d11_export.rs` | `export_d11` refuses a non-default mode, since the integer engine has no unit snap. New: `export_refuses_a_non_default_normalization_mode`. |
| `attention_learning.rs`, `attention_anchor_device.rs`, `driver_qualification.rs` | The default field in config literals, so they compile. |
| `docs/labs/m1-normalization-mechanism/README.md` | The wiring section is updated to the current state. |
| this folder | ORDER (committed first) and this REPORT |

## Measurements

- **Crate tests.** `cargo test -p uor-r4-discrete --lib`: 227 passed, 0 failed, 1 ignored (pre-existing). The five new or changed tests pass on their own.
- **Default identity.** The golden-byte snapshot test passes unchanged. `model_snapshot_sha256` is equal with the field written explicitly and with it omitted. The text of a default snapshot and of a default `DriverConfig` contains no `normalization_mode`. The UORLUT01 round-trip test passes, so default export bytes are unchanged.
- **Negative gate.** With the `validate_checkpoint` mismatch check set aside, `normalization_mode_round_trips_and_checkpoint_mismatch_refuses` fails (a panic at `training.rs:1565`). After restoring it, the test passes.
- **Stub-nvcc check.** `NVCC=<stub> UOR_NVCC_ARCH=compute_120 cargo check --offline -p uor-r4-discrete --features cuda --all-targets` exits 0.
- **Ratchets.** `d23` 4,451, `d25` 93 and `d32` 31, each equal to its baseline. The PR diff removes no `cfg(any())` gate.
- **GPU run, approved by Casey (PR comment 6370260849; he confirmed the $2.227/h price in chat).** One RTX 5090 (`vast-57399787`), bootstrapped at `d6490303…` with the pod's parity check PASS.
  - **Inputs.** The pinned hashes matched for DEV `0919e90f…` / `a838bb1a…`, TRAIN `3dee858f…` / `b5e7352d…` and the tokenizer `d36d3e87…`.
  - **Unit-snap device test.** `cargo test --release -p uor-r4-discrete --features cuda --lib unit_snap_device -- --ignored`: 1 passed, 0 failed. That covers the unit-mode forward on canonical roots, the Route and `C_raw` refusals, and the snapshot round trip.
  - **A8 table.** `address-cells-codebook …/p8.json OUT structured` reproduced `file_sha256 6600bdb1…`, `table_sha256 4122cbab…` and `counts_sha256 755a5a81…`. So `StructuredV1` is unchanged at this head, which merges main and therefore includes #3070's `address_cells.rs` changes.
  - **A8 checkpoint-0 identity** (treatment config `a8-relation-seed1.json`; default mode, `s[12] = 0`). Every pinned value reproduced exactly:

    | Pinned value | Expected | Measured |
    |---|---|---|
    | `b3_psucc` (`attention-dev-b3`, `b3.psucc.target_admitted`) | 1,351 | 1,351 |
    | `b3_full` (`eval-corpus`, `aggregate.target_admitted_rows`) | 4,244 | 4,244 |
    | top-1 (`aggregate.top1_correct`) | 133 | 133 |
    | `occupied_cells` | 256 | 256 |
    | `distinct_vectors` (`coverage-census --token-cells`) | 4,096 | 4,096 |
    | tie mass (`within_cell_tie_mass.tokens_in_alias_groups`) | 0 | 0 |

    The run had `accepted_updates == 0` and `epoch == 0`.
  - **Results.** All 30 files are in the HF store at `results/claude/unit-snap-3071-20261023/`, with `SHA256SUMS`. The pod is deleted.

## Checks run at the head

`cargo test -p uor-r4-discrete --lib`, the stub-nvcc `cargo check --features cuda --all-targets`, `cargo clippy -p uor-r4-discrete --all-targets --no-deps` (touched lines clean), `uor-harness ratchet d23|d25|d32`, `loop report --check`, `loop pr-check --local`.

## Skills used

- **`scope-guard`:** the change stays in `crates/uor-r4-discrete/**` and the lab folder. The five ungated admission paths and the public extractor are recorded, not changed.
- **`solution-integrity`:** default identity is pinned by the existing golden bytes and the hash-domain equality, not just asserted.

## Still open

- **After merge:** update the A8 runbook's pinned `compiled_identity.source_sha256` to the post-merge digest (#2842, 6370010494).
- **The five admission gates** listed above, if a control arm is ever run through those paths.
- **The PM merges before any A8 GPU stage** (#2842, 6368080811).

## Plain summary

A `C_unscaled` model can now be configured, saved, reloaded and refused when mismatched. Existing checkpoints and configs are byte-for-byte unchanged. On a real GPU, the unit-snap mode runs and round-trips, and the default path reproduces every pinned A8 checkpoint-0 number exactly.
