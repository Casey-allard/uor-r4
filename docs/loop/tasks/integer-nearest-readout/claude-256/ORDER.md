# ORDER: integer-nearest-readout, claude-256

Lab: claude. Item: [integer-nearest-readout](../../../../roadmap/items/integer-nearest-readout.md). Base: `175fca2a8`.

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

Q5 chain step 6, part 1, under claude-247's [decision card](../claude-247/DECISION.md): the replay driver on the laptop. The card's forced replay (its "Forced replay" section) runs a step (P, Q) whose picks at p differ:
- Q runs p's window with every decision in Φ forced to P's value;
- the first decision in p's causal prefix where Q differs from P ends the pass: a code or a rate bucket joins Φ and Q runs again; a kept set or a memory lookup stops the replay;
- at p's nearest, the step is accounted for when every forced decision was a near tie and the pick is equal or a near tie.

claude-254 built one pass's grade (`grade_pass`) and the verdict on a finished replay (`replay_accounts`). This unit builds the loop between them, and its R side:
- **`replay_step`** in `fbs_snapshot.rs`: one step's whole replay, given P's trace and table and a function that runs one pass of Q under Φ. It ends by the card's argument, and refuses a replay that runs past its prefix's decision count.
- **`fbs-toks ARTIFACT --decision-replay`**: the arithmetic step (T, R), with Q = R from the artifact, on the laptop. Each pass steps the window's prompt and its reference up to p under Φ.
- **claude-254's six round-2 points** (its REPORT's "Still open"), since the driver changes the grader next.

**Part 2, the next unit:** the structure step (F, T) needs Q = T on the GPU. That is `train-in-space`'s single-window float mode (each float pass is now a whole `score_rows` run, 204 s on F's s1), driving the same `replay_step`.

## Cancel probe (run first)

Cancel if origin/main already has a replay driver, or if an open PR touches this unit's files.

**Result, 2 Nov (`local/claude/claude-256/probe-grep.txt`, `probe-open-prs.txt`):** no driver exists, and the unit goes ahead.
- On origin/main at `175fca2a8`, "replay" appears in `fbs_snapshot.rs` only in `grade_pass`'s and `replay_accounts`' docs and tests. In `fbs-toks.rs`, `train-in-space.rs` and `in_space_long.rs` it appears only in the docs of `--decision-grade` and `--decision-force`. No code runs a pass, grades it and forces the next.
- The open PRs are this unit's draft #3540 and DeepSeek's #3539, which touches only `crates/uor-r4-cpu-train/**` and its own docs, plus `docs/harness/claims.json`.
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

1. **`replay_step`** runs one step's forced replay by the card's rule, with host tests on synthetic traces. A test fails when the loop does not force the decision a pass ends at (negative gate). The tests cover:
   - a replay that forces near-tie decisions and then reaches equal picks: accounted;
   - one that forces a clear decision: not accounted, although the picks end equal;
   - one that ends at a kept set: not accounted;
   - a pass that does not take a forced value: refused (I-Force);
   - the replay ends, with one pass per differing decision and one more: Φ grows at every pass by a decision of p's prefix that it did not hold.

   *Amended before any code was written.* The first draft asked for a refusal of a replay that runs past its prefix's decision count. No replay can reach one: `grade_pass` refuses a pass that does not take Φ's values, so each pass's first difference is a new decision of the prefix. The test of that end replaces it.
2. **`fbs-toks ARTIFACT --decision-replay`** runs the arithmetic step's replay with R as Q.
   - Without `--at`, it replays every offset of the reference's windows where the unforced picks differ, and counts the rest as equal.
   - It writes `uor-r4.decision-replay/1` new: each replay's passes, its forced set and whether it is accounted for.
   - A host test shows that a pass cut at p gives the same steps up to p as the whole window.

   *Amended during the work, before any reading ran.* T's trace reads on past the prompts file's reference: its first window holds 99 ids, against R's 50. So "every offset of the reference's windows" became every offset inside R's window. The reference's offsets past it are counted, not compared, and its tokens inside R's window must be R's (I-Align), with a test and two negative gates.
3. **claude-254's round-2 points**, each with a test where it is code:
   - a float pick that falls behind the other token in f64 by more than its f32 rounding bound is refused (negative gate);
   - on R, a pick that ties the other side's pick but has the larger id is refused (negative gate);
   - a negative margin on the reference side, within the bound, is accepted (negative gate: refusing it fails the test);
   - the CLI's refusals are tested: an `--at` named twice, a side with no table, an offset past the window;
   - `grade_pass`'s doc says the reference's entries and codes are checked even where forced;
   - a deciles file that names no trace is refused.
4. **Readings on the laptop, outside Q5's set, on DEV s1** (R's smoke's 308 windows, claude-249's `prompts-dev-s1-outside.json`). They read no Q5 position, and move nothing in done-when 3.
   - `--decision-check` of T's trace (`b1970581…`, claude-252) against T's table, R's artifact divided by 4096: the CLI's T loader on real data.
   - `--decision-deciles` of T's trace.
   - `--decision-replay` of the step (T, R) at every offset where the two picks differ, with the cell's G₁₀ (30,228, claude-244). Each replay is recorded as it ends.
5. **Gates:** the `fbs_snapshot` and `fbs-toks` tests and the `uor-r4-integer --features serving-wasm` suite, with no failure beyond the base's 36; fmt; clippy, with no new warning in a changed file; `loop gates --full`; `loop pr-check --local`.
6. **The REPORT** with the negatives first, one notebook row, and the item's bullet for this step.

## Stop and ask if

Stop and decide (never stop and wait):
- **T's check fails on real data:** an instrument fault. Repair it here if it is the grader's or the loader's; otherwise record it and leave the replay reading out.
- **A replay pass is refused** (I-Force, I-Align or the bound): an instrument fault, located and recorded. It is never counted as a result.
- **A replay of (T, R) is not accounted for:** that is a reading outside Q5's set, recorded as it is. It is not Q5's FAIL, and the bar is unchanged.
