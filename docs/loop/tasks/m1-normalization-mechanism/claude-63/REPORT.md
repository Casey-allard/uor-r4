# REPORT: m1-normalization-mechanism, claude-63 (check the normalization mode at the five remaining admission gates)

Lab: claude. Item: [m1-normalization-mechanism](../../../../roadmap/items/m1-normalization-mechanism.md) (#2842). Base: `3df3798e4` (main, after #3071). `References #2842`; the item stays `ready`.

## What did not work, first

1. **Only one of the five gates is pinned by a test that fails without it.** That is the `attention_learning.rs` `validate_config` gate.
   - **Controller gate:** it is redundant with `DriverConfig::validate`, which already refuses a non-default mode with a Route read, so no test can isolate it.
   - **The other three:** the relation INIT LOCK, `validate_initial_storage` and the audit `baseline_facts` sit on device or CUDA paths. Their shared JSON helper, `stored_mode_is_default`, is tested directly.
2. **These gates require the default mode as a constant, not "equal to the config".** That mirrors how they already require `ActiveLeafV1`, and changes no default behaviour. It also means the A8 attention, controller, relation and audit paths refuse a `C_unscaled` or `C_raw` model. That's intended: the normalization sheet's arms run through the training driver, which compares the mode against the config. If a control arm is ever meant to run through one of these paths, its line needs relaxing on purpose.

## What changed

| File | Change |
|---|---|
| `crates/uor-r4-discrete/src/attention_learning.rs` | `validate_config` requires `config.normalization_mode.is_scale_matched_snap()`; the existing error text is kept, because `attention_audit_qualification.rs:1267` quotes it. `validate_initial_storage` (CUDA) refuses a stored non-default mode. |
| `crates/uor-r4-discrete/src/controller.rs` | `validate_registered_data` refuses a non-default config mode. |
| `crates/uor-r4-discrete/src/relation_qualification.rs` | The INIT LOCK requires `snapshot.normalization_mode.is_scale_matched_snap()`. |
| `crates/uor-r4-discrete/src/attention_audit_qualification.rs` | `baseline_facts` refuses a stored non-default mode. |
| `crates/uor-r4-discrete/src/normalization_mechanism.rs` | `pub(crate) fn stored_mode_is_default(&Value)`, which treats a missing key as the default, since the field is omitted when default; plus a unit test. |
| this folder | ORDER (committed first) and this REPORT |

## Measurements

- **Crate tests.** `cargo test -p uor-r4-discrete --lib`: 228 passed, 0 failed, 1 ignored (pre-existing).
- **Negative gate.** With the `validate_config` mode line removed, `attention_requires_its_own_scope_initialization_and_pins` fails. After restoring it, the test passes.
- **Stub-nvcc check.** `cargo check --offline -p uor-r4-discrete --features cuda --all-targets` exits 0.
- **Ratchets.** `d23` 4,451, `d25` 93 and `d32` 31, each equal to its baseline. No `cfg(any())` gate is removed.

## Checks run at the head

`cargo test -p uor-r4-discrete --lib`, the stub-nvcc `cargo check --features cuda --all-targets`, `cargo clippy -p uor-r4-discrete --all-targets --no-deps` (touched lines clean), `uor-harness ratchet d23|d25|d32`, the `cfg(any())` removal grep, `loop report --check` and `loop pr-check --local`.

## Skills used

- **`scope-guard`:** every change is a refusal added beside an existing `read_initialization` check, and default behaviour is unchanged.

## Still open

- Device-level tests for the three CUDA-path gates, if those paths ever gain a cheap test harness.

## Plain summary

No model saved under a control normalization mode can now slip through the A8, controller, relation or audit admission paths; each refuses it as it refuses a wrong initialization.
