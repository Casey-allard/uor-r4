# ORDER: one-model-end-to-end, claude-279 (P3, cache unit 2: the Hamming read's step and the decode switch)

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Base: `f34f0b17c`. Draft PR #3673.

## Goal

Make scoring's decode use [claude-278](../claude-278/REPORT.md)'s exact step cache wherever the window fits, so a decode step stops re-running the whole window. Every scored file must stay byte-identical.

- **Plan step:** Casey's 4 November plan, P3 (faster scoring), cache unit 2 of 2. The order on #2028 (6634218223) reads: "the Hamming read's step, the decode switch where the window fits, and a full re-run on slide".
- **Why the switch pays at long windows:** at a 2,048 window, P2b's two queries files are about 98% of a scoring pass ([claude-277](../claude-277/REPORT.md)).
  - Of their 18,879 decode steps after the first, 18,721 (99.2%) fit the window. That count comes from p-long's `queries.json` and the scorer's own cut (`query_prompts`), with 31 steps after the prompt.
  - At a 192 window, 6,105 (32%) fit. The rest slide and keep the full re-run.
- **What it changes in the model:** nothing the model computes. It adds three things:
  - **The Hamming read's step.** The cache keeps each Hamming read's input residual for every position so far, plus each row's ids. A step then runs the read's own code over the whole window, with the query axis cut to the new position.
    - Everything before the cut is the same op on the same tensors as the whole window's: the keys, the values and the asks.
    - After the cut, every op works one query row at a time.
    - The selection's distances and tie keys are exact small integers.
  - **The decode switch,** in the batched decode (`decode_replies_with_mode`), which is the default for the samples and queries files.
    - It prefills the prompt and then steps, for as long as each step's window fits.
    - The first step that would slide the window, and any typed refusal, returns that group to today's whole-window re-run for the rest of its decode.
  - **`--decode-cache 0|1` on `train-in-space`, default 1.** At 0 the scorer keeps today's re-run, so one binary runs both sides of the gate on one pod.
- **Not in this unit:** the serial decode (`decode_reply_with_mode`: forced openings, served keys, read scopes and the writer). It keeps the full re-run. Its share of a pass is reported.

## Cancel probe (run first)

1. Does another open PR or order hold the float trainer's files?
2. Does a decode switch already exist?
3. Do the caps hold one pod for the gates (at most 4 pods and $8/h across all labs)?
4. Is the base `origin/main`?
5. Has Casey written on #2028 since the order (6634218223)?

**Result, at `f34f0b17c`: not cancelled.**
1. No. No open PR changes a path under `crates/uor-r4-float-train` (`gh pr list` over every open PR's files).
2. No. `in_space_long.rs` and `flock_read_hamming.rs` name neither `step_cached` nor `prefill_cached`.
3. Yes. `uor-pod status`: 0 pods, $0/h of $8/h; 5 verified 5090 offers, the cheapest at $1/h.
4. Yes. The base is claude-278's merge commit, `f34f0b17c76cac6c28e82fbc754163023948ba65`.
5. No. The newest notes on #2028 are the lab's own, the last being 6634218223.

## Files in scope

- `docs/loop/tasks/one-model-end-to-end/**`
- `docs/harness/prep/one-model-end-to-end*`
- `docs/research/notebook/float-training.md`
- `docs/roadmap/items/one-model-end-to-end.md`
- `crates/uor-r4-float-train/src/geometric_stack_native.rs`
- `crates/uor-r4-float-train/src/flock_read_hamming.rs`
- `crates/uor-r4-float-train/src/in_space_long.rs`
- `crates/uor-r4-float-train/src/bin/train-in-space.rs`

Every path is in the item's `scope:` list on `main`.

## Done when

1. **The Hamming read's step.**
   - The read takes the first query position. Whole-window calls pass 0 and run as before, op for op.
   - The step refuses the read's instruments, each a typed refusal: the decode cut, the row cut, the decision trace, and forced codes.
   - Copy key 2 and the type spin are either covered by the GPU gate or refused.
2. **The cache carries the Hamming reads.**
   - Per read: the input residual at every position so far, `(batch, positions, width)` at f32.
   - Each row's ids.
   - The prefill captures both from its own forward, and a step appends one position.
3. **The decode switch,** as in the Goal:
   - turn-memory reply rows named in step rows;
   - the opening row at the prefill, which is the whole-window forward;
   - `--decode-cache`.
4. **Unit 1's two deferred checks** (claude-278, Still open, item 2):
   - the carry compare asserts that both caches hold the same number of layers, inputs and reads before it zips them;
   - the old lowest-bit flip is rerun under the carry compare and reported, not gated.
5. **The gates, on the laptop:**
   - the crate's CPU tests pass;
   - the stub-nvcc CUDA check compiles every target;
   - ~~a decode test on the CPU: cache on and off give the same tokens, through a window slide.~~ **Amended 4 Nov, at the build:** the native trunk has no CPU path (the scan's `cpu_fwd` refuses with "D24: native model operation requires CUDA; no CPU reference"). So this test runs on the pod, as item 6's decode-switch gate. The laptop instead checks the refusals that need no forward pass: the Hamming read's four instruments, and reads set after the prefill.
6. **The gates, on one GPU pod.** The scoring passes re-score P2b's sealed checkpoints, so they run under a prep sheet with a fresh non-author FIT and a pre-run review (§4.5).
   - **Step equals full, bit for bit (G1 widened):** the random six-layer trunk with P2b's two Hamming reads, given random rotations, margins and weights. It crosses:
     - the retention reading, on and off;
     - the served surrogates, on and off;
     - the turn memory's reply table, on and off;
     - prompts of 4, 7 and 33 positions, 12 steps each;
     - batch 1 and 3.

     Every step's points and every carry, the reads' residuals included, must equal the whole window's.
   - **The negatives (G3 widened):** one flipped bit in a carried read residual fails the compare.
   - **The decode switch:** a random model with the reads and the turn memory decodes the same tokens with the cache on and off, through a window slide.
   - **The crate's CUDA lib suite** passes.
   - **The scoring passes:** P2b's seed-1 checkpoints with P2b's commands, L192, S192 and L2048, each with `--decode-cache 0` and `1` on one pod.
     - Every output file is byte-identical.
     - The printed lines are identical but for the decode seconds.
     - The REPORT gives tok/s per file for both sides.
7. **The REPORT,** negatives first and tok/s first, with one notebook row and a line on the item.

## Stop and ask if

- **A step's bits differ from the whole window's,** and the cause is an op whose result depends on the launch's shape. Then the Hamming read stays refused in the step, and the trunk-only cache still serves models without the read. The REPORT names the op.
- **Any scored file differs between cache on and off.** Then the default goes to 0, and the REPORT names the first file and row that differ.
- **The pod needs more than about $3.**
