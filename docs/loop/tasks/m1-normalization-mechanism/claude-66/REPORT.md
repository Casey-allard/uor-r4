# REPORT: m1-normalization-mechanism, claude-66 (a separate normalization run path on the proxy trunk)

Lab: claude. Item: [m1-normalization-mechanism](../../../../roadmap/items/m1-normalization-mechanism.md) (#2842). The item status is unchanged; no compute is cleared and nothing is released.

## What did not work, first

1. **No arm can run yet, by design.** `normalization` and `normalization-resume` refuse "normalization sheet not yet registered" before any corpus load or CUDA work, because `NORMALIZATION_SHEET` is `None`. The sheet (`docs/harness/prep/m1-normalization-mechanism-DRAFT-2026-10-22.md`) is still `NOT_FIT`, and its authors are editing it. Pinning its hash in code now would turn main red on every edit. Registration is the later one-line PR.
2. **The "one-line registration" in the ORDER was not true as first written.** A design scout found that `attention-check-plan` and `attention-owner-release` are bound to the attention scope, so a normalization run could never have its admission built. The ORDER was amended before the work landed, and the unit adds `normalization-check-plan` and `normalization-owner-release`.
3. **`N_snap` duplicates the A8 run.** Its config is byte-identical to the A8 attention config (same `config_sha256` and lineage); only the scope string differs. The sheet authors should say whether `N_snap` reuses the A8 run or runs again under the normalization scope.
4. **DEV scoring of the control arms is not built yet.** The DEV-B3 analysis (`attention_preflight.rs:272`, `attention-dev-b3`) refuses non-default modes, and `attention_transfer` stays attention-only. The sheet's §6 step 5, scoring `C_unscaled` and `C_raw` on DEV, needs that next unit.
5. **`driver_source_sha256` moved, as expected,** and is re-pinned in `prime_zeta.rs` in this PR: `791a555d…` → `ab0691a4f815fb094978854dd4e01b808e5741c656245a45e9368bbc51286f58`. I recomputed it from the committed sources in `include_bytes!` order, and the three bins that pin it pass with it.
6. **Two local gates fail at the base, in files this unit does not touch.**
   - `cargo clippy -- -D warnings` stops on existing findings elsewhere, in a dependency crate and other discrete files.
   - `cargo fmt -p uor-r4-discrete --check` flags `tests/flock_coupling_controls.rs`, and `cargo fmt` would rewrite it.

   The changed files are clippy- and fmt-clean.

## Cancel probe

At the base, every trunk entry refuses `C_unscaled` and `C_raw`:
- `attention` refuses at `attention_learning.rs:198`;
- `pilot` and `redirect` admit only Route, which `training.rs:242` refuses for non-default modes, and `controller.rs:163` checks too;
- `completion` must equal the pilot config.

The task stands.

## What changed

| File | Change |
|---|---|
| `crates/uor-r4-discrete/src/attention_learning.rs` | **Registration.** New `Registration { Attention, Normalization }` and `const NORMALIZATION_SHEET: Option<…> = None`. |
| | **New un-gated functions.** `validate_normalization_config` admits exactly the A8 proxy trunk (Relation8 sweep 1 `LearnV1`, K8, A8 pinned table, nearest-address objective, active-leaf init, interleaved read-every-8, seed 1 or 2) with any of the three modes. `normalization_scope` gives `m1-normalization-{n_snap\|c_unscaled\|c_raw}-injective-6600bdb1306d-relation-k8-seed{n}-batches0..16404`, using an exhaustive mode→label match, so a new mode cannot be admitted silently. |
| | **Registration helpers.** `registration_scope`, `registration_sheet` (Normalization: "not yet registered"), `validate_registration_config` and `require_registration_scope`. |
| | **Storage mode check.** `validate_initial_storage_mode` requires the stored mode to equal the entry's mode. `validate_initial_storage` is unchanged in behaviour, as a wrapper with the default mode. |
| | **Threading.** `reg` is threaded through plan, release, progress, save and run. `run_registration`, `check_plan_registration` and `owner_release_registration` are added. The existing `run`, `check_plan`, `owner_release`, `scope`, `registered_sheet` and `validate_config` keep their behaviour. `run_registration` checks config, then sheet, then scope before `PreparedDriver::load`, so attention runs now also refuse those inputs earlier than `release` would. |
| | **Tests.** Six new tests. |
| `crates/uor-r4-discrete/src/main.rs` | `normalization`, `normalization-resume`, `normalization-build-identity` (prints `normalization_scope`, `normalization_mode`, `sheet: NOT_YET_REGISTERED`), `normalization-check-plan`, `normalization-owner-release`, and their help lines. |
| `crates/uor-r4-discrete/src/attention_transfer.rs` | Passes `Registration::Attention` at its 3 call sites. The transfer path stays attention-only. |
| `crates/uor-r4-discrete/src/prime_zeta.rs` | Re-pins `driver_source_sha256`. |
| this folder | ORDER (committed first, amended before the work) and this REPORT |

## Measurements

- **Every discrete target** (`cargo test --locked -p uor-r4-discrete`) exits 0:
  - lib: 235 passed, 0 failed, 1 ignored;
  - `address-cells-codebook`: 6 passed;
  - `prime-zeta-codebook`: 7 passed;
  - `prime-zeta-qualify`: 6 passed (these three pin the driver hash);
  - `flock_coupling_controls` 7, `read_off_ablation_controls` 7 and `writer_bucket_controls` 6 passed.
- **New and existing attention tests** (`--lib attention_learning`): 41 passed. They include:
  - `normalization_admits_three_modes_on_a8_trunk_only`;
  - `normalization_scope_differs_per_mode_and_from_attention`;
  - `attention_scope_strings_unchanged` (the literal Parent, Injective and A8 scope strings);
  - `normalization_refuses_not_yet_registered`;
  - `normalization_plan_scope_separates_entries`;
  - `initial_storage_requires_the_entry_mode`.
- **Negative gate, run by me.** With `attention_learning.rs:958` changed from `stored_mode != json!(mode)` to compare against the default, `initial_storage_requires_the_entry_mode` FAILED (0/1). Restored, it passed (1/1).
- **CUDA-gated code compiles.** `NVCC=<stub> UOR_NVCC_ARCH=compute_120 cargo check --offline --locked -p uor-r4-discrete --features cuda --all-targets` finished with no errors. Clippy, with and without `cuda`, reports nothing in the four changed files.
- **Ratchets at baseline:** D23 4451, D25 93, D32 31. The added lines contain no `f32`/`f64`, softmax, squashing, matmul or GEMM tokens.
- **The A8 lock is unchanged.** No definition of `scope`, `registered_sheet` or `validate_config` was removed, and their outputs are pinned by `attention_scope_strings_unchanged` and the existing refusal tests.

## Checks run at the head

The measurements above, plus `loop report --check`, `loop pr-check --local` and `git diff --check`.

## Skills used

- **`scope-guard`:** only `crates/uor-r4-discrete` and this folder. `normalization_mechanism.rs`, `discrete.cu`, `device.rs`, `snapshot.rs`, `training.rs` and the A8-locked callers are untouched.
- **`solution-integrity`:** the storage-mode test fails with its fix set aside. The driver-hash re-pin is a recomputed value, and the bins that pin it pass.

## Still open

- **Registration.** Once the sheet is `FIT`, set `NORMALIZATION_SHEET` to the FIT sheet's `(sha256, title)` and add its hash test.
- **DEV scoring** of `C_unscaled` and `C_raw` (point 4): the next unit.
- **For the sheet authors.** §6 can now name `discrete-stack normalization-build-identity <config>` and `normalization <config> <admission> <out>` per arm and seed, and should state whether `N_snap` reuses the A8 run (point 3).

## Plain summary

The normalization test now has its own entry, which accepts all three normalization arms on the A8 proxy trunk and records the arm in its scope and saved state. The A8 path stays locked. Nothing can run until the sheet is approved and registered.
