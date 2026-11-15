# ORDER: one-model-end-to-end, claude-393 (a window-split gate credit, and the sheet for the first native read training run)

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407).
Base: `origin/main` at `0e7191cc4`, which holds claude-392 (#4011).

## Why this unit

D86 step 2 has its operator on host and device (claude-389 to 392), but no gate of the native read has been trained. Two things block that run:
- claude-392's Still open 2: the device credit kernel is serial over the rows;
- §4.5: a GPU training run needs a sheet with a non-author FIT.

Casey said "Yes, start it" (15 Nov 2026) to this unit.

## What is built

1. **The credit kernel, split by window.** In `native_vector.cu` the serial `native_q31_read_gate_credit` is replaced by two kernels.
   - `native_q31_read_gate_credit_partial`: one thread per (gate word, window). It sums that window's exact `__int128` conjugate products and writes each sum as two 64-bit words.
   - `native_q31_read_gate_apply`: one thread per gate word. It adds the windows' partials in window order and writes `sat(gate − clamp(credit >> (step_shift + 1), ±2^32))`.

   Integer addition is exact and associative, so no word can change. claude-392's three parity tests stay as they are. The entry list in `build.rs` follows the change.
2. **A timing test** at a run-sized batch (width 256, 64 windows of 192 rows), printing the device step's wall time. It runs on wick, at main's head and at this head.
3. **The pre-registration sheet** `docs/harness/prep/one-model-end-to-end-native-read-train-2026-11-15.md`: the first native training of the read's gates on the converted 16k-step Flock trunk states. It is judged on held-out picks and on `R_ask` at gain ×16. It covers the controls, seeds, cost and outcome map, ends with a `TEST FITNESS:` line, and passes `uor-harness prep-check`.
4. **A non-author FIT** of the sheet by a fresh-context subagent (§4.5), with its changes applied.

## Native-space five items (SKILL)

- **State:** the read's Q31 gates, weights and gain, unchanged.
- **Operator:** one definition, the host `train_pick_step`. The device twin is restructured, not redefined.
- **Replay oracle:** `Q31ReadTrainStep` equality, device against host (claude-392's tests).
- **Forward, gradient, update:** integer only, and the PTX admission rejects any float opcode.
- **Serialization:** unchanged.

## Checks (builds and finite kernel tests, §4.5: no sheet for these)

1. **Laptop:**
   - a stub-nvcc `cargo check` and clippy of `uor-r4-discrete --features cuda` (lib and `native_vector_cuda`);
   - `uor-harness prep-check` on the sheet.
2. **wick:** `UOR_REQUIRE_CUDA=1 cargo test --release -p uor-r4-discrete --features cuda --test native_vector_cuda`, the whole suite green; the timing at both heads.
3. **Negative gates on wick.** Each of these must fail a parity test:
   - skipping one window's partial in the apply;
   - dropping a partial's high word;
   - adding window 0's partial twice.

## Files in scope

- `crates/uor-r4-discrete/src/native_vector.cu`, `native_vector_device.rs`, `build.rs`, `tests/native_vector_cuda.rs`;
- the sheet;
- this folder;
- one notebook row in `float-training.md`.

## Stop list

- No training run: that is the next unit, after this sheet's FIT and its own pre-run review.
- No change to the host rule, the read or the session.
- The D41 pilot's `llama-server` on wick is not touched.

## Done when

The checks pass, the sheet has its FIT, a fresh review approves the exact head, and the PR merges.
