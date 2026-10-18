# Typed `rows_v1` `DriverConfigRecord` Schema Compatibility in `attention_reference.rs` — 2026-10-18 (`antigravity / ag1`)

## Summary & Scope

Delivers the narrow host-only typed schema update in `crates/uor-r4-harness/src/attention_reference.rs` identified by independent static review on `#2029` ([comment `6246867414`](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6246867414)):

- Defines `CounterfactualPath` (`LanesV1` default, `RowsV1`, `#[serde(rename_all = "snake_case")]`, `is_lanes_v1(&self) -> bool` / `is_lanes(&self) -> bool`, plus `CounterfactualPathRecord` type alias) matching `crates/uor-r4-discrete/src/training.rs:58-69`.
- Adds `#[serde(default, skip_serializing_if = "CounterfactualPath::is_lanes_v1")] pub counterfactual_path: CounterfactualPath,` to `DriverConfigRecord` in the exact field position between `relation_learning_policy` and `seed` (matching `crates/uor-r4-discrete/src/training.rs:85-86`).
- Preserves strict `#[serde(deny_unknown_fields)]` on `DriverConfigRecord`, `DriverLineageRecord`, and `DriverCheckpointRecord`:
  * Legacy `lanes_v1` configurations (omitting `counterfactual_path` or passing `"counterfactual_path": "lanes_v1"`) deserialize to `CounterfactualPath::LanesV1` and reserialize without `"counterfactual_path"`, preserving byte-identical canonical `config_sha256` (`c446aa7fcc926bc951e8c39929794e45f749f760c739f74ec8347dbe4bf9f216` / `1435f01644dddc8e678b5268533d30edb16b6905563d8a3fa122b0623f1071c3`), `lineage_sha256`, and Domain C `driver_checkpoint_sha256`.
  * Explicit `"counterfactual_path": "rows_v1"` configurations deserialize to `CounterfactualPath::RowsV1` and reserialize with `"counterfactual_path":"rows_v1"` between `relation_learning_policy` and `seed`, reproducing the exact producer `config_sha256` (`c194cd6e4f3e5b127e6ef5dbe3294aa893a205518ed62a632da7236e57c72228` for `learn_v1`, `15a10bac9eb04bb84daaa71f9c580cfaa2e28151f05616da768aa9fc784d866f` for `frozen_zero_v1`) and `lineage_sha256` (`f1403a8894563c99706b69fe501e066f1ca62951ddef5d870bf8df409d491a53` for `learn_v1`, `480c5bafd3a07ddfafd02bde47f71bfbce3544e677da0c7e289158ac4a2210c9` for `frozen_zero_v1`) from `consumer-panel-rows-treatment-v1/report.json` (`7f59e7c42155620069cc2416a7e1a1fddfae358331b6b737d7a16a02a41b4578`), and authenticating Domain C `driver_checkpoint_sha256` in `verify_driver_checkpoint_domains`.
  * `paired-dev` requires `rel.lineage.config.counterfactual_path == ctrl.lineage.config.counterfactual_path` (since `expected_ctrl_config` clones `rel.lineage.config` and overrides only `relation_learning_policy`).
  * Unknown `counterfactual_path` variants or extra fields on `DriverConfigRecord` continue to be rejected.

## Outcome & Cost Boundary

- Host-only Rust consumer schema update (`$0`, no GPU, no pod, no model compute, no training or evaluation).
- Both `rows_v1` Dose 1 arms on `vast-56510253` hit the registered `Q1_STOP_FOR_DIAGNOSIS` diagnostic gate at batch `4,096` ([`#2029` comment `6247068871`](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6247068871)); no paired full-dose `16,404` synthesis run is executed or claimed here (`learned_attention_result = NOT_YET_SCORED`, `mechanism_verdict = NOT_ISSUED`, `fresh_transfer_evaluations = 0`).
