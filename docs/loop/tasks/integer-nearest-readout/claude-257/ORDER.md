# ORDER: integer-nearest-readout, claude-257

Lab: claude. Item: [integer-nearest-readout](../../../../roadmap/items/integer-nearest-readout.md). Base: `5cf66a98d`.

## Goal

The item's done-when reads:

> 1. **The artifact carries its readout,** with a negative gate. An artifact without it decodes byte for byte as at the base, and an engine without it refuses a nearest artifact.
> 2. **The nearest step** serves `stack_whole_table_nearest`'s rule under the field, with a test against a reference scan and a negative gate.
>    - The browser path (`wasm_serve`'s step) gives the same token as the session's nearest on one artifact, with a test.
>    - The serving audit (`audit_zero_matmul_serving.py --stack`) and the WASM build pass.
> 3. **End-to-end token parity passes its bar, under a prep sheet with a fresh FIT and a pre-run review.**
>    - Integer greedy of (r-drop) step 16,384, memory on, exported raw (`fbs-export --raw-gates --raw-transition`, R), against the float greedy decoded again on the GPU with the read's tie rule on (F′: claude-220's arm-G command at the unit's head), on DEV and p-w at seeds 1–3.
>      - claude-232's snapped export (S) and claude-220's stored replies (F) are kept as readings.
>      - Both changes are claude-243's [decision card](../../loop/tasks/integer-nearest-readout/claude-243/DECISION.md), logged on #3462; Casey can reverse it. "Done" applies to the raw export, and the item then names its recipe as the served artifact's.
>    - **The bar is claude-245's [decision card](../../loop/tasks/integer-nearest-readout/claude-245/DECISION.md),** logged on #3462; Casey can reverse it. It replaces claude-243's first-token ⌊0.03 n⌋, which was registered against claude-227's 0.33% residual of a snapped export. Q1, Q2 and Q4 are read under teacher forcing on claude-244's counted positions, and Q3 on claude-243's free decode. A LOSS of any class fails this bar for that artifact.
>      - R picks the reference token as often as F′ does: no loss beyond 3 row-clustered SE (Q1), and at most half a point (Q2);
>      - the same holds at the opening, on free decode (Q3);
>      - R's differences sit at its own near ties (Q4: κ ≥ 4/9).
>      - **Q5:** every clear-margin disagreement is also accounted for by an exact trace, under claude-247's [decision card](../../loop/tasks/integer-nearest-readout/claude-247/DECISION.md), logged on #3462; Casey can reverse it.
>      - First-token and whole-reply agreement stay reported as readings; claude-243's MISS on the old bar is recorded as it is.
>    - **The item is done only when the bar passes.** On a miss, the sheet's row is recorded, the item stays `ready`, and the next unit locates the gap.
>    - Integer decodes run on the laptop: product serving (CHARTER §3.6), under the owner's 31 Oct allowance for this chain. The float side runs on the GPU.
> 4. **Tokens per second,** with the nearest scan counted (4,096 rows a token), and the resident memory.
> 5. **One notebook row, and a REPORT with the negatives first.**

## This unit

Q5 chain step 6, part 2, under claude-247's [decision card](../claude-247/DECISION.md): the forced replay of the **structure step (F, T)**. Its reference side P is F, and Q is T, F's path on the evaluation copy rounded as exported (claude-252). claude-256 built `replay_step` and drove it for the arithmetic step (T, R) on the laptop. This unit drives the same `replay_step` with Q = T on the GPU, in one process:
- **`train-in-space --decision-replay CHECKPOINT`** loads T's copy once and replays each chosen offset. Each pass runs only the replayed window's own eval batch (the `SHEET_EVAL_BATCH` windows its split's sealed run batched it with), with that window traced under Φ. The card prices a pass on T at "a single-window forward on the GPU, milliseconds". The whole `score_rows` run that `--decision-force` needs is 204 s on F's s1 (claude-253).
- **Each replay is appended to a line file as it ends**, and the whole record is written at the end. This is round 2's MINOR 2 on claude-256, since the float passes run on a rented GPU.
- **The pieces both drivers read move to the library unchanged:** the trace reader, the deciles parser, the offset's step, the side tables and the replay tally. `fbs-toks --decision-replay`'s record stays the same.

**Then, the next unit:** the sheet that registers claude-247's rule with claude-254's amendment, with a fresh FIT and a pre-run review, and then the GPU run that reads Q5 (done-when 3).

## Cancel probe (run first)

Cancel if origin/main can already replay the structure step in one process, if `--decision-force` passes would fit the run anyway, or if an open PR touches this unit's files.

**Result, 2 Nov (`local/claude/probe.txt`): nothing replays the structure step, and whole-run passes do not fit, so the unit goes ahead.**
- **Code:** on origin/main at `5cf66a98d`, the float crate holds no `decision-replay`, `decision_replay` or `replay_step`. `--decision-force` runs one forced set through a whole `score_rows` run.
- **Cost:** that run took 204 s on F's s1 (claude-253, `base.wall`).
  - On DEV s1, T's pick equals R's on 4,380 of 4,382 offsets of R's range and F's on 4,298 (claude-252), so F and T differ on at least 82 of them.
  - At one pass each, that is at least 82 × 204 s ≈ 4.6 h for one cell's smoke, and drift makes a replay several passes. The card's GPU budget is under an hour for the whole Q5 run.
- **Open PRs:** DeepSeek's #3543 and Kimi's #3541 touch none of this unit's files.
- **Class: the instrument** (the trace tooling), not a mechanism.

## Files in scope

- `crates/uor-r4-integer/src/fbs_snapshot.rs`
- `crates/uor-r4-integer/src/stack/format.rs`
- `crates/uor-r4-integer/src/stack/session.rs`
- `crates/uor-r4-integer/src/stack/kernels.rs`
- `crates/uor-r4-integer/src/stack/tests.rs`
- `crates/uor-r4-integer/src/stack/synthetic.rs`
- `crates/uor-r4-integer/src/bin/fbs-export.rs`
- `crates/uor-r4-integer/src/bin/fbs-toks.rs`
- `crates/uor-r4-float-train/src/fbs_export.rs`
- `crates/uor-r4-float-train/src/bin/fbs-snapshot.rs`
- `crates/uor-r4-integer/src/stack/hamming.rs`
- `crates/uor-r4-float-train/src/flock_read_hamming.rs`
- `crates/uor-r4-float-train/src/geometric_stack_native.rs`
- `crates/uor-r4-float-train/src/cuda_stack.cu`
- `crates/uor-r4-float-train/src/in_space_long.rs`
- `crates/uor-r4-float-train/src/flock_read_memory.rs`
- `crates/uor-r4-float-train/src/geometric_stack_native_cuda.rs`
- `crates/uor-r4-float-train/src/bin/train-in-space.rs`
- `docs/roadmap/items/integer-nearest-readout.md`
- `docs/harness/prep/integer-nearest-readout*`
- `docs/labs/integer-nearest-readout/**`
- `docs/loop/tasks/integer-nearest-readout/**`
- `docs/research/notebook/m4-serving.md`

## Done when

1. **`train-in-space --decision-replay CHECKPOINT`** runs `replay_step` for the step (F, T), with T from the copy at `CHECKPOINT` as Q, F's trace as P, F's deciles and table, and the cell's G₁₀.
   - **Each pass** runs the replayed window's own eval batch, with only that window traced, under Φ.
   - **I-Rerun on the float side:** pass 0 (Φ empty) must reproduce T's sealed window at that position byte for byte, or the replay is refused.
   - **Without `--at`,** it replays every offset where F's and T's unforced picks differ, over the steps where both windows hold a nearest. It reads those from the two sealed traces, so the listing itself runs no forward.
   - **Before any forward, it refuses:**
     - a reference that is not side F, an other trace that is not side T, or a checkpoint that is not a copy rounded as exported;
     - two windows at one position in either trace;
     - deciles read from another trace than the reference;
     - two windows that do not line up (I-Align: the same ids, `reply_start` and nearest steps);
     - a named offset that lies past its window or holds no nearest.
   - **A refused replay** is recorded with its refusal, the rest still run, and the command then fails, as `fbs-toks`'s does.
2. **The record is streamed.** Each replay is appended to `replay-{split}.lines.jsonl`, created new, one JSON entry a line, flushed as it ends. The whole `uor-r4.decision-replay/1` record follows at the end.
3. **The shared pieces live in the library,** with unchanged behaviour: the trace reader, the deciles parser, the offset's step, the side tables, and the replay tally (each replay's entry and the counts). `fbs-toks`'s existing replay tests pass unchanged.
4. **Tests:**
   - host tests of the tally and the line file;
   - CUDA tests on the synthetic model:
     - a pass on the window's batch reproduces the batched run's traced window byte for byte;
     - the driver end to end, with P as Q's own trace: no replay without `--at`, and one accounted one-pass replay with it, each written to the line file and the record;
     - its refusals.
   - **Negative gate:** with the I-Rerun check set aside, the test that feeds the driver a T trace with one edited value fails.
5. **The GPU smoke on DEV s1, outside Q5's set:**
   - the CUDA tests by name;
   - then `--decision-replay` of the step (F, T) at every offset where F's and T's picks differ. The inputs are F's base trace over DEV s1's 308 windows (`9e7c9bf3…`, claude-253), T's trace over the same windows (`b1970581…`, claude-252), F's deciles (claude-254) and G₁₀ = 30,228.
   - It is a reading: it reads no Q5 position and moves nothing in done-when 3.
   - It records the counts, the passes and the wall time per pass, and I-Rerun on every replay.
   - The results are saved off the pod and put in the store, and the pod goes down.
6. **Gates:**
   - the changed crates' host suites, with no failure beyond the base's;
   - stub-nvcc `cargo check --features cuda --all-targets` of the float crate;
   - fmt, and clippy with no new warning in a changed file;
   - `loop gates --full` and `loop pr-check --local`.
7. **The REPORT** with the negatives first, one notebook row, and the item's bullet for this step.

## Stop and ask if

Stop and decide (never stop and wait):
- **Pass 0 does not reproduce T's sealed window:** an instrument fault, never a reading. Locate it (the batch, a global the sealed run set, or the build), repair it here, and rerun.
- **A replay is refused** (I-Force, I-Align or the bound): an instrument fault, located and recorded, never counted.
- **A replay of (F, T) is not accounted for:** a reading outside Q5's set, recorded as it is. It is not Q5's FAIL, and the bar is unchanged.
- **The passes project the Q5 run past an hour:** record the projection, and take the card's resume-from-saved-state as the next unit before the sheet.
