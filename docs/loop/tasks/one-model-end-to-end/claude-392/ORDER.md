# ORDER: one-model-end-to-end, claude-392 (the CUDA twin of the Q31 read's gate update)

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407).
Base: `origin/main` at `db472d470`, which holds claude-391 (#4009).

## Why this unit

claude-391 built `Q31FirstPickRead::train_pick_step` on the host. Training qualifies on GPU (§3.5), and D85 requires one operator definition whose device implementation passes a bit-identical differential. claude-391's Still open 1 is this twin. Casey said "proceed with the roadmap as you have recommended" (15 Nov 2026).

## What is built

`NativeVectorEngine::read_train_pick_step` in `crates/uor-r4-discrete/src/native_vector_device.rs`, plus two kernels in `native_vector.cu`:

- **`native_q31_read_pick_loss`:** one thread per row. It finds the row's rival with the read's tie rule (`<=`, the target excluded) and computes the hinge with `row_margin` in `__int128`. It writes the rival, the loss words, and the three sign residuals: on the query, the target key and the rival key.
- **`native_q31_read_gate_credit`:** one thread per gate word. It runs over the rows in order and sums the exact conjugate products of the residuals with the lifted blocks in `__int128`. It writes the candidate gate word `sat(gate − clamp(credit >> (shift + 1), ±2^32))`. There are no atomics, and the order is fixed.

The host keeps what is scalar:
- the refusals, with the CPU's messages and order;
- the credit shift (`q31_read_credit_shift`);
- the `u128` loss sum;
- the strict-drop accept.

The picks before and after come from the existing device read.

## Native-space five items (SKILL)

- **State:** the read's Q31 gates, weights and gain, unchanged from claude-391.
- **Operator:** one definition, the host `train_pick_step`. The CUDA path is its twin and is admitted by the same integer PTX inventory, which is extended by the two entries.
- **Replay oracle:** `Q31ReadTrainStep` equality, device against host. That covers every gate word, `accepted`, both losses, both pick lists and the rivals, compared as raw words.
- **Forward, gradient, update:** integer only, `s64` and `__int128`, with no float opcode (the PTX admission rejects any).
- **Serialization:** unchanged.

## Checks (builds and finite kernel tests, §4.5: no sheet)

1. **Laptop:** a stub-nvcc `cargo check -p uor-r4-discrete --features cuda --all-targets`; the native-vector tests; clippy on the changed lines.
2. **wick** (RTX 4090), `UOR_REQUIRE_CUDA=1 cargo test -p uor-r4-discrete --features cuda --test native_vector_cuda`, comparing device against host:
   - claude-391's fixtures, chained step by step;
   - seeded batches at width 64 and 256, scale 2^18 and full range, random targets, margins 0 and >0, step shifts 0, 4 and 31;
   - an exact tie;
   - every refusal, matched by its message.
3. **Negative gates:** removing the tie rule, the minimum margin, the conjugate, or the `+1` in the shift on the device each fails a parity test on wick.

## Files in scope

`crates/uor-r4-discrete/src/native_vector.cu`, `native_vector_device.rs`, `build.rs` (the entry list), `tests/native_vector_cuda.rs`; this folder; one notebook row in `float-training.md`.

## Stop list

- No training run.
- No change to the host rule, the read or the session.
- The D41 pilot's `llama-server` on wick is not touched.

## Done when

The checks pass, a fresh review approves the exact head, and the PR merges.
