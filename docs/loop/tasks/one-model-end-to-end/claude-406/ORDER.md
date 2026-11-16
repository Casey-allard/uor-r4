# ORDER: one-model-end-to-end, claude-406 (how fast the discrete model's CUDA scoring runs on a D87 student batch)

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md).
Base: claude-404's branch `claude/student-native-step`. It reuses that unit's student batch, and this branch is rebased onto `main` once claude-404 merges.

## Why this unit

Casey, 16 Nov 2026, in claude's chat: "we must figure out how to make this training on the corpus much faster and still run on the cuda on the wick 4090", then "great ok so begin the next part then".

### Where the time goes now

The trainer's baseline counterfactual path launches one block of 16 threads for every proposed parameter (`crates/uor-r4-discrete/src/device.rs:2207`, `launch_config(16)`). Each thread is one candidate lane, and it walks every row and position serially. An RTX 4090 has 16,384 CUDA cores. The records show the cost:
- the counterfactual kernel is 99.996% of proposal wall time (`docs/labs/k16-timing-2026-10-15/README.md`);
- M1's runs reached 66–533 targets/s on an RTX 5090 (`docs/labs/carrier-credit-hard-selection/README.md`);
- K16 Stage T reached about 50,900 context tokens/s on 2×5090 (`docs/research/notebook/m1-discrete-attention.md`).

The D87 corpus projects to about 1.87B scored targets in our tokenizer (claude-403). At about 300 targets/s, one pass takes about 70 days.

### The faster path already exists

The rows-K path (`rowsk_device.rs`) scores up to 64 coordinates of one kind in three launches, with one thread per (coordinate, row, lane): up to 64 × 64 × 16 = 65,536 threads. The trainer accepts `coordinate_block` up to 64 on the `rows_v1` path (`training.rs:292-303`), and BVG-K commits from such a block. Neither has been timed on the 4090 or on corpus text.

This unit measures the gap before anything is built. It is a finite kernel test: scoring only, no training step, no commit. So under §4.5 it needs no sheet.

## Native-space items (D85)

1. **State schema.** No change. The test reads the existing `Device` snapshot at checkpoint 0 and changes no state.
2. **Shared operators.** These are the existing counterfactual kernel and its rows-K form, which the trainer already uses. No kernel source changes.
3. **Replay oracle.** The baseline `propose` loss words; claude-372 and claude-404 check those against the CPU oracle. Every rows-K coordinate's 16 loss words must equal the baseline's, word for word.
4. **Forward, gradient, update.** Forward and counterfactual scoring only. No choose, apply or epoch change; the test asserts the epoch is unchanged.
5. **Serialization.** No change.

## Design

- **The test.** One `#[ignore]` CUDA test in a new module, `crates/uor-r4-discrete/src/student_throughput.rs` (`cfg(all(test, feature = "cuda"))`). It reads `UOR_STUDENT_STREAM` and builds claude-404's 64-row batch from the middle eligible target.
- **The model.** V4096/B4/C192, Route read, seed 1, checkpoint 0.
- **Coordinates.** Fixed before any timing, never chosen by loss:
  - the first 64 VectorBlocks, by index, that belong to the batch's target tokens;
  - Routes 0..63.
- **Arms**, each with one warm-up and five timed repeats, synchronized, median reported:
  - **A.** Baseline `propose`, one coordinate per launch, over the K coordinates.
  - **B.** Rows-K `score_rows_k` at K = 1, 8, 16, 32 and 64.
- **Parity.** For every coordinate, B's status is 0 and its 16 loss words equal A's.
- **Planted fault.** One target is moved in B's input only; at least one coordinate must then mismatch.
- **Output, per kind and arm:**
  - milliseconds per call;
  - coordinates per second;
  - row × coordinate evaluations per second;
  - batch targets per second under the rule "one K-block per 64-row batch" (64 ÷ seconds per call).
- **No interference.** The GPU must hold no other process during timing. The test records `nvidia-smi` before and after.

## Files in scope

- `crates/uor-r4-discrete/src/student_throughput.rs`, which is new;
- one `mod` line in `crates/uor-r4-discrete/src/lib.rs`;
- `crates/uor-r4-discrete/src/cpu_oracle_parity.rs`: visibility only, `pub(crate)` on claude-404's student-batch helpers (`StudentBatch`, `student_batch`, `STUDENT_SPEC`, `STUDENT_SEED`, `device_with`, `windows`);
- `docs/loop/tasks/one-model-end-to-end/claude-406/**`;
- one row in `docs/research/notebook/cpu-training.md` or the discrete notebook.

## Stop list

- No training run, no apply and no pod; wick's 4090 only.
- No kernel source change and no change to trainer defaults.
- Lifting the 64-row cap, a device-resident stream, removing host syncs and the timed training run are later units, named in Still open.

## Done when

1. The test runs on wick's 4090 with 0 loss-word mismatches, and the planted fault is caught.
2. The REPORT gives the timing table and the projected days per corpus pass for each arm.
3. `loop pr-check --local` passes.
