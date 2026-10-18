# D37: offline-training float allowance, 2026-10-18

**Line:** governance (no model line). **Count:** not applicable; no headline is moved. **Lab:** claude.

## What happened

The owner revised D32 in Claude's session on 18 October (about 02:35Z). The exact words are in [D37](../../integration/DECISIONS.md#d37--floating-point-is-legal-in-offline-training-only-d32s-learning-path-float-ban-is-revised); the relay to the PM is [#2029 6244744499](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6244744499).

**Why it was needed.** The path directive ([#2029 6244066845](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6244066845)) said "float legal in offline training". D32 items 1 and 5 said the opposite. Claude flagged the conflict ([#2029 6244710766](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6244710766)).

**The owner's scope answers, in the same session:**
- full float learning offline (forward passes, gradients and optimizers);
- archived F32/F64 sheets return only by re-registration and a fresh FIT;
- the integer boundary is the export (D11 serving, with a reported export gap);
- the October 14 complex-plane zeta-zero exception is kept separately, as written.

The owner also gave the reason behind every ban: exclude expensive operations, in service of fast local training and serving that approaches frontier capability through the geometric method.

## Changed

- **`docs/integration/DECISIONS.md`:** new D37, plus a supersession note under the D32 heading. D32's text is otherwise unchanged (the file is append-only).
- **`docs/integration/agent-execution-policy.json`:** schema `/6` → `/7`.
  - Set to true: `offline_autodiff_training_allowed`, `offline_scalar_floating_point_geometric_arithmetic_allowed`, `offline_training_floating_point_allowed`, `offline_training_floating_point_instruments_allowed` and `offline_floating_point_gradients_and_optimizer_allowed`.
  - Added `served_model_floating_point_allowed: false`.
  - Removed the three keys that asserted the old ban: `model_floating_point_allowed`, `floating_point_model_instruments_allowed` and `floating_point_gradients_and_optimizer_allowed`.
  - Updated `learning_contract`.
  - Unchanged: the matmul keys (`training_matrix_multiplication_allowed`, `dense_matrix_multiplication_allowed`, `serving_matrix_products_allowed`) stay false.
- **`crates/uor-r4-core/src/bin/r4-policy-check.rs` (the CI architecture-commitment check):**
  - It now asserts schema `/7`, plus the serving commitments: served-model float, serving float, serving-kernel float, serving transcendentals and serving multiplier instructions are all false.
  - The matmul commitments are unchanged.
  - New test `served_float_true_is_rejected`.
- **Current-authority text in AGENTS.md, `agent-execution-policy.md`, STATUS.md and the README** now cites D37. Dated records are unchanged.

## Not changed

- The D32 discrete path (`crates/uor-r4-discrete`) and its D29 integer token scans (`scripts/check_native_cuda_vendor.rs`).
- The D23/D25 ratchets.
- The Dose 1 sheet, packet and runner.
- Training, M2 and Stage B stay HELD; Q4 stays NOT_EXECUTABLE.

Nothing was compiled for the model, and no compute ran. Cost: $0.

## Checks

The results of the commands below are posted on the PR at the exact head:
- `cargo test --locked -p uor-r4-core --bin r4-policy-check` (3 tests);
- `cargo run --locked -p uor-r4-core --bin r4-policy-check -- .`;
- `cargo fmt --check` on the changed file;
- `python3 scripts/check_claim_wording.py`;
- `uor-harness ratchet d23` and `uor-harness ratchet d25`;
- `uor-harness check`.
