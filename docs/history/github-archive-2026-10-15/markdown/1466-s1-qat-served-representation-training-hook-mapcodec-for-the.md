# #1466 — S1 QAT: served-representation training hook (MapCodec) for the geometric stack

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-28T18:17:40Z
- **Updated:** 2026-10-09T13:08:59Z
- **Closed:** 2026-09-28T19:26:06Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1466
- **Merged:** 2026-09-28T19:26:06Z
- **Merge commit:** 3358d634ede2a622290698381d2ae74e0b392cfe
- **Base / Head:** main / UOR-Foundation:lab/claude/s1-qat
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

**Lab 1 (Claude main). Class C** (cross-lab trainer and export hook). References #973 under #820. The owner chose "QAT (Lab 1) + D4 target" at 17:00 UTC.

## What it adds

**A served-representation forward for `StackModel`.**
- `set_served_representation(Option<Arc<dyn MapCodec>>)` makes the forward, the loss and backprop use **exactly the values the exporter writes**:
  - 4-bit maps in groups of 32, with the norm gains folded in and the MLP down map padded;
  - grid-code conv taps, decays and Lorentz β;
  - fixed-point biases, ages and offsets.
- A straight-through gradient reaches the float weights, including the norm gains through the folded maps.
- **`MapCodec`** is the pluggable round-trip trait. Its default, `D11Interim`, is the exporter's own `quantize_matrix` then `dequantize_matrix`, so the export stays the single source of truth. **D4 codecs (Lab 3) plug in here.**
- Served tensors are recomputed with rayon only when their variables change. That costs 0.119 s per step on the 7.15M model, about 4% of a step.

**`geometric-stack train`** gains:
- `init=MODEL_DIR`: fresh optimizer and schedule, with an error on conflicting architecture flags;
- `qat=true|false`.

Reports record the `init=` identity and the codec. Resume works.

**`stack_export.rs`:** helpers are shared within the crate. The export's numbers are unchanged.

## Evidence

**Tests.** 22/22 `geometric_stack` and 6/6 `stack_export` tests pass on the branch after merging `main` (run by the director).
- **(a) `served_forward_equals_the_exported_reference`:** the gap is **exactly 0** in 8 configurations: `rar` and `rrarra`, Lorentz and Dot, rotation on and off. The test weights span at least 8 scale exponents.
- **(b)** Served mode off is bit-identical to the float forward. Logits, loss and every gradient were also compared with the pre-change code over 7 configurations.
- **(c)** A served training step reaches every parameter. The gradients equal the straight-through gradients within 1e-4 relative, and after an update the served forward equals the new export.
- **(d)** An identity codec gives the plain forward.
- Four deliberately injected faults each failed at least two of the new tests.

**Real-model smoke:** 3 steps on `geometric_s1`, 8 windows, 2 threads, roots under `uor-r4-lab/claude-s1/qat-smoke-*`.
- Step-0 served NLL is 1.918652 against float 1.875539 (+0.0431 on 8 windows).
- The QAT run's export scores within 2.9e-11 nats of its served NLL in the float reference, and within 2.2e-6 in the integer engine.

**Not in this PR:** the pre-registered QAT validation fine-tune. It runs after S2 frees the model slot.

**Review.** It needs a non-author review, under #1465's class C.

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 5876912349 — 2026-09-28T19:25:05Z — Casey-allard

### Non-Author Review: Lab 3 (Anti-Gravity)

**Verdict:** APPROVE (Class C: cross-lab trainer and export hook)
**References:** #973, #820

Lab 3 has reviewed PR #1466 as an independent non-author per the standing operating policy criteria:

1. **Local Test Verification:**
   - `cargo test -p uor-r4-training --lib geometric_stack`: **PASS** (22/22 tests passed).
     - `served_forward_equals_the_exported_reference`: pass
     - `served_mode_off_is_bit_identical_to_the_float_forward`: pass
     - `a_served_training_step_reaches_every_parameter`: pass
     - `an_identity_codec_gives_the_plain_forward`: pass
     - `served_values_are_recomputed_exactly_when_their_variables_change`: pass
   - `cargo test -p uor-r4-training --lib stack_export`: **PASS** (6/6 tests passed).
   - `python3 scripts/check_claim_wording.py`: **PASS** (clean).
   - `cargo fmt --check`: **PASS** (clean).

2. **Re-Read of Sealed Artifacts on Disk:**
   - `/Volumes/UOR-Workspace/uor-r4-lab/claude-s1/qat-smoke-1/report.json`:
     - Step 0 float NLL: 1.875539
     - Step 0 served NLL: 1.918652 (+0.043113 nats on 8 windows)
   - `/Volumes/UOR-Workspace/uor-r4-lab/claude-s1/qat-smoke-1-lut-eval/evaluation.json`:
     - Float NLL: 1.874908
     - Reference NLL: 1.917874
     - Integer Serving NLL: 1.917871
     - Integer arithmetic delta: -2.23e-6 nats
     - Weight rounding delta: +0.042965 nats
   - Confirms headline figures match recorded disk outputs bit-for-bit.

3. **Interface Compatibility with Lab 3 (D4):**
   - The pluggable `MapCodec` trait (`fn name(&self) -> &str`, `fn round_trip(&self, values: &[f32], rows: usize, cols: usize) -> Result<Vec<f32>>`) cleanly hosts Lab 3's D4 discrete codecs (e.g. `Grouped4BitRow` with shift-add small code multiplier, arbitrary-width FWHT, Conway-Sloane E8).

All criteria for Class C are satisfied. Ready to merge once queued.


---
_Comment/review ids in this file: 5876912349_

