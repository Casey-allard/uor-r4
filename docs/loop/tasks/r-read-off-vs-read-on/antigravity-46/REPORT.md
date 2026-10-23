# REPORT: r-read-off-vs-read-on, antigravity-46

Lab: antigravity. Item: [r-read-off-vs-read-on](../../../../roadmap/items/r-read-off-vs-read-on.md) (#2885). The item status remains `ready` because Step 5 (GPU evaluation and notebook rows) requires a non-author `TEST FITNESS: FIT` review on `docs/harness/prep/r-read-off-vs-read-on-DRAFT-2026-10-23.md`, PM/owner bar and checkpoint-option registration on `#2885`, and Casey's approval in chat (CHARTER §4.5, §5.1).

## What did not work, first

1. **Calling `PreparedDriver::load` directly in the host `C7` test:** The first draft of `c7_committed_ten_arm_battery_configs_are_valid_and_distinct` in `crates/uor-r4-discrete/tests/read_off_ablation_controls.rs` called `PreparedDriver::load(config)` on the committed pod config files (`docs/labs/r-read-off-vs-read-on/evidence/{a8,p8}-*.json`), which failed on the laptop with `No such file or directory` because `PreparedDriver::load` opens `/workspace/uor-r4/local/driver/data/mw-balp/train/tokens.u16`. We replaced that call in `C7` with `config.validate()` plus distinct `config_sha256` checks across all 10 committed arm configs, and verified `PreparedDriver::lineage_sha256()` separation across `ZeroGatesV1`, `RouteBypassV1`, and `RoutePermutedActionsV1` inside `crates/uor-r4-discrete/src/training.rs` using `metadata_fixture`.
2. **`attention-dev-b3` rejecting `F-Off` and `Route` arms under `attention_learning::validate_config`:** In `crates/uor-r4-discrete/src/attention_preflight.rs`, `dev_inputs` (called by `attention-dev-b3`) previously called `attention_learning::validate_config(&config)`, which required `Flock8`/`Relation8` with `ActiveLeafV1` and would have rejected the `F-Off` (`ZeroGatesV1`) and `Route` (`ZeroGatesV1`, `RouteBypassV1`, `RoutePermutedActionsV1`) arms in the paired battery. We added `validate_dev_b3_config` in `attention_preflight.rs` so `attention-dev-b3` admits all five paired read-on/read-off/permuted arms while `capture_dev` continues to enforce `attention_learning::validate_config`.
3. **`PreparedDriver::clone` in `training::tests::initialization_defaults_and_cross_schedule_lineage_refuse` (`E0599`):** Caught by DeepSeek's review on PR `#3097` (`issuecomment-6374960610`), `crates/uor-r4-discrete/src/training.rs:1562` called `route_base.clone()` on `PreparedDriver` (which does not implement `Clone`), added after the initial `--lib` run. We replaced `route_base.clone()` with `metadata_fixture(1, ReadSpec::Route)?`.

## What changed

- [`crates/uor-r4-discrete/src/lib.rs`](../../../../../crates/uor-r4-discrete/src/lib.rs): Added `ReadInitialization::RouteBypassV1` (`route_bypass_word() == 1`, `ReadSpec::Route` only) and `ReadInitialization::RoutePermutedActionsV1` (`permuted_route_action(action) = (5 * action + 1) % 64`, `ReadSpec::Route` only), preserving `is_zero_gates()` for `ZeroGatesV1` only.
- [`crates/uor-r4-discrete/src/discrete.cu`](../../../../../crates/uor-r4-discrete/src/discrete.cu): Wired `s[13]` (`RouteBypassV1`) in `route_vector` (refusing `s[13] > 1` or `s[13] == 1 && kind == 0`, and writing `input_block(..., length - 1, block, ...)` directly to `output + block * 8` when `s[13] == 1`) and in `discrete_validate` (`s[13] > 1 || (s[13] != 0 && s[8] != 0)`).
- [`crates/uor-r4-discrete/src/device.rs`](../../../../../crates/uor-r4-discrete/src/device.rs) and [`crates/uor-r4-discrete/src/main.rs`](../../../../../crates/uor-r4-discrete/src/main.rs): Passed `read_initialization.route_bypass_word()` as `s[13]` in `spec_gpu` and `gpu_spec`, applied and readback-verified `permuted_route_action` on `routes` when `read_initialization == RoutePermutedActionsV1`, refused `Parameter::Route` under `RouteBypassV1`, and added the `#[ignore = "requires a CUDA GPU"]` device test `route_read_off_and_permuted_controls_run_and_refuse`.
- [`crates/uor-r4-discrete/src/training.rs`](../../../../../crates/uor-r4-discrete/src/training.rs), [`crates/uor-r4-discrete/src/attention_preflight.rs`](../../../../../crates/uor-r4-discrete/src/attention_preflight.rs), and [`crates/uor-r4-discrete/src/d11_export.rs`](../../../../../crates/uor-r4-discrete/src/d11_export.rs): Validated `RouteBypassV1` and `RoutePermutedActionsV1` on `ReadSpec::Route` with `CanonicalVectorThenReadV1` in `DriverConfig::validate`, admitted the paired `Route` and `Flock`/`Relation` arms in `attention-dev-b3` (`validate_dev_b3_config`), and refused `RouteBypassV1` and `RoutePermutedActionsV1` in `export_d11`.
- [`crates/uor-r4-discrete/tests/read_off_ablation_controls.rs`](../../../../../crates/uor-r4-discrete/tests/read_off_ablation_controls.rs), [`docs/labs/r-read-off-vs-read-on/evidence/`](../../../../labs/r-read-off-vs-read-on/evidence/), [`docs/labs/r-read-off-vs-read-on/README.md`](../../../../labs/r-read-off-vs-read-on/README.md), and [`docs/harness/prep/r-read-off-vs-read-on-DRAFT-2026-10-23.md`](../../../../harness/prep/r-read-off-vs-read-on-DRAFT-2026-10-23.md): Staged the 10 zero-update arm configs (`{a8,p8}-{r-on,r-off,r-perm,f-on,f-off}.json`) and `read-off-battery.sh`, added control `C7` (`c7_committed_ten_arm_battery_configs_are_valid_and_distinct`), and updated the lab README and pre-registration sheet.

## Measurements

- **`read_off_ablation_controls` (`C1`–`C7`):** `7 passed; 0 failed` (`0.01s`).
- **`uor-r4-discrete --lib` unit test suite:** `229 passed; 0 failed; 1 ignored` (`85.96s`), including the new `ReadInitialization`, `DriverConfig`/`lineage_sha256`, `validate_dev_b3_config`, and `export_d11` refusal tests.
- **Negative gate check:** Setting `c7_committed_ten_arm_battery_configs_are_valid_and_distinct` to call `PreparedDriver::load` on pod paths failed as recorded in `## What did not work, first`, and altering `RouteBypassV1` schedule/mode in `training.rs` or `attention_preflight.rs` is rejected by `validate()` / `validate_dev_b3_config()`.

## Checks run at the head

- `~/.cargo/bin/cargo test --locked -p uor-r4-discrete --test read_off_ablation_controls` -> `7 passed; 0 failed`
- `~/.cargo/bin/cargo test --locked -p uor-r4-discrete --lib` -> `229 passed; 0 failed; 1 ignored`
- `sh -n docs/labs/r-read-off-vs-read-on/evidence/read-off-battery.sh` -> `PASS` (exit `0`)
- `uor-harness loop gates --full` -> `11 / 11 GREEN`
- `uor-harness loop report --check` -> `PASS`
- `uor-harness loop pr-check --local` -> `PASS`

## Skills used

- `uor-research-architect`: Wired the `RouteBypassV1` (`s[13]`) and `RoutePermutedActionsV1` controls and staged the 10-arm battery script while preserving integer-only geometry, `Snapshot`/`DriverConfig` hash domains, and D11/training-release guards.
- `scope-guard`: Kept all changes strictly inside the declared scope (`crates/uor-r4-discrete/**`, `docs/harness/prep/r-read-off-vs-read-on*`, `docs/labs/r-read-off-vs-read-on/**`, and this task folder).
- `solution-integrity`: Verified that `RouteBypassV1` and `RoutePermutedActionsV1` cannot leak into `export_d11` or legacy training releases (`controller.rs:162`), and that `attention-dev-b3` accepts all 10 paired arms while `capture_dev` remains gated by `attention_learning::validate_config`.

## Still open

- **PM / owner checkpoint & bar registration (`#2885`, CHARTER §5.2):** Selection of Option A vs Option B and registration of the paired `Delta_top1` / `Delta_adm` bar on `#2885`.
- **Non-author `TEST FITNESS` review (`#2885`, CHARTER §4.5):** Independent lab `TEST FITNESS` review of [`docs/harness/prep/r-read-off-vs-read-on-DRAFT-2026-10-23.md`](../../../../harness/prep/r-read-off-vs-read-on-DRAFT-2026-10-23.md).
- **Step 5 GPU evaluation & notebook rows (`#2885`, CHARTER §4.5 / §5.1):** After `TEST FITNESS: FIT`, bar registration, and Casey's approval in chat, execute `docs/labs/r-read-off-vs-read-on/evidence/read-off-battery.sh` on one leased GPU pod and record the resulting notebook rows in `docs/research/notebook/m1-discrete-attention.md`.

## Plain summary

This order completes Step 4 of `r-read-off-vs-read-on` (`#2885`): it wires the `Route` read-off (`RouteBypassV1`, `s[13] = 1`) and geometric permuted-action (`RoutePermutedActionsV1`, $g'(a) = (5a + 1) \bmod 64$) controls into `uor-r4-discrete` (`discrete.cu`, `device.rs`, `training.rs`, `attention_preflight.rs`, `d11_export.rs`), stages all 10 zero-update arm configs and `read-off-battery.sh` under `docs/labs/r-read-off-vs-read-on/evidence/`, and verifies all host unit tests and gates.
