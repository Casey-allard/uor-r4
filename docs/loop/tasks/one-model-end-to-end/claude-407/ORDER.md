# ORDER: one-model-end-to-end, claude-407 (lift the 64-row cap on CUDA scoring and time rows-K at 256 to 4,096 rows)

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md).
Base: claude-406's branch `claude/student-throughput-v2` (PR #4043). It reuses that unit's speed test, and this branch is rebased onto `main` once #4043 merges.

## Why this unit

Casey, 16 Nov 2026, in claude's chat: "we must figure out how to make this training on the corpus much faster and still run on the cuda on the wick 4090", then "start the row-cap unit on wick while CI runs".

claude-406 measured rows-K scoring on one 64-row D87 student batch on wick's RTX 4090:
- a call costs at least about 81 ms, flat from K = 1 to K = 32;
- so the 64-row cap holds it near 780 targets/s, about 28 days per pass over ~1.87B targets (a lower bound: scoring only).

What the ~81 ms floor is made of was not measured. If it is per-row work with GPU room left, more rows per call raise targets/s. This unit lifts the cap and measures that. It is a finite kernel test: scoring only, no training step, no commit. So under §4.5 it needs no sheet.

## Where the cap sits

Four checks refuse more than 64 rows. None comes from a kernel limit:
- `Device::admit_batch` (`crates/uor-r4-discrete/src/device.rs:1448`), which every forward, proposal and rows-K call runs;
- `Device::carry` (`device.rs:1480`);
- `Device::carry_windows` (`device.rs:1494`);
- `PreparedCorpus::batch` (`crates/uor-r4-discrete/src/corpus.rs:255`).

The kernels launch one thread per row or per (coordinate, row, lane), with `launch_config(items)`, and loop over rows by index. The carrier selection loops over rows in strides of its 256 threads. Rows-K allocates `rows × K × 16` cost and flag words. At 4,096 rows and K = 64 that is 4,194,304 words of each, about 32 MB of cost words.

## Native-space items (D85)

1. **State schema.** No change. The test reads the existing `Device` snapshot at checkpoint 0 and changes no state.
2. **Shared operators.** These are the existing counterfactual kernel and its rows-K form. No kernel source changes; only the host-side row bound moves.
3. **Replay oracle.** The baseline `propose` loss words, which claude-372 and claude-404 checked against the CPU oracle at 64 rows. At every row count, every rows-K coordinate that has a baseline must match it word for word. Every other coordinate must give identical words at every K.
4. **Forward, gradient, update.** Forward and counterfactual scoring only. No choose, apply or epoch change; the test asserts the epoch is unchanged.
5. **Serialization.** No change.

## Design

- **The cap.** One constant, `MAX_BATCH_ROWS = 4096`, in `device.rs`, used by the three device checks. `corpus.rs` uses it too. The refusal texts name the new bound, and `qualification.rs`'s `inventory_bounds` expectation follows the `admit_batch` text.
- **Pins.** Editing `corpus.rs` moves `training::driver_source_sha256`, pinned in `driver_source_pin.rs`. Editing `device.rs` and `lib.rs` moves the device `source_sha256`. Both are recomputed in this PR, and the full `uor-r4-discrete` test suite runs, not `--lib` only.
- **The batch.** `student_batch` becomes `student_batch_rows(rows)`, with `student_batch()` kept at 64 rows, so claude-404's parity test is unchanged. Each batch starts at the stream's middle eligible target.
- **The test.** claude-406's `#[ignore]` speed test gains a row loop over 64, 256, 1,024 and 4,096 rows. At each size:
  - **Coordinates**, fixed before any timing as in claude-406: the first 64 VectorBlocks of the batch's target tokens, and the Route model's routes.
  - **Arm A** (baseline `propose`) runs on a subset only, because one call takes about 4.7 s at 64 rows and grows with rows: 4 coordinates per kind at 64 and 256 rows, 2 at 1,024, 1 at 4,096.
  - **Arm B** (rows-K) runs every coordinate at K = 1, 8, 16, 32 and 64, one warm-up and five timed repeats, median.
  - **Parity** as in item 3.
- **The three test fixes from claude-406's Still open item 4:**
  - rates come from the coordinates actually scored, so a short last call no longer overstates them;
  - the planted target shift goes through the same parity predicate, and all 16 coordinates must be caught;
  - the test runs `nvidia-smi --query-compute-apps` before and after and prints it.
- **Output, per row count, kind and K:** milliseconds per call, row-coordinates per second, and targets per second at one call per batch.
- **Run.** wick's RTX 4090 only, with progress lines and the pipe status recorded.

## Files in scope

- `crates/uor-r4-discrete/src/device.rs`: the constant and three checks;
- `crates/uor-r4-discrete/src/corpus.rs`: one check;
- `crates/uor-r4-discrete/src/qualification.rs`: one expected text;
- `crates/uor-r4-discrete/src/driver_source_pin.rs`, and any other pin the full suite names;
- `crates/uor-r4-discrete/src/cpu_oracle_parity.rs`: `student_batch_rows`;
- `crates/uor-r4-discrete/src/student_throughput.rs`;
- `docs/loop/tasks/one-model-end-to-end/claude-407/**`;
- one row in `docs/research/notebook/cpu-training.md`.

## Stop list

- No training run, no apply and no pod; wick's 4090 only.
- No kernel source change.
- The trainer's `batch_rows` bound (`training.rs:307`), the `uor-r4-discrete` CLI's window bound (`main.rs:172`), the `baseline_reuse` 64-bit row mask and the per-call host syncs stay as they are. A larger trainer batch is a later unit, after this measurement.

## Done when

1. The full `uor-r4-discrete` test suite passes on the laptop, and the CUDA test suite builds.
2. The speed test runs on wick's 4090 at all four row counts, with 0 loss-word mismatches, every status 0, and all 16 planted coordinates caught.
3. The REPORT gives the timing table and the projected days per corpus pass at each row count, labelled lower bounds.
4. `loop pr-check --local` passes.
