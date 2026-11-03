# ORDER: one-model-end-to-end, claude-264 (round the served projections to 2^-16, then re-read I-3)

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Base: `45fad9191`. This ORDER is hand-written under the umbrella reading (cancel probe 2).

## Goal

claude-263 found that the integer first token misses the float one on 112 and 137 of 310 prompts (bar 8). A read of the code found one rounding site on the integer side only:
- **Integer.** `stack_fbs_gather` converts every FBS projection output to the residual's 2^-16 grid, rounding half up (`to_exp_i32`).
- **Float.** The served surrogate's `linear` leaves it raw f32, so `rec.gate` reaches the decay score and the icosian snap unrounded.

This unit adds that rounding to the surrogate, test first. It then re-scores claude-262's two checkpoints in float and re-reads I-3 against claude-263's stored integer replies.

**Plan step (the claim check):** order step 4. I-3 is the door to the full-length served run. The change is in the served forward that trains and scores, so served training gets it too.

## Cancel probe

1. **Already done?** No: main has no projection rounding under the served bridge.
2. **The lock.**
   - **The refusal**, 11:17:57Z, verbatim: "loop claim: refused: item one-model-end-to-end is already claimed by PR #3583 (docs/loop/tasks/one-model-end-to-end/claude-261/ORDER.md)".
   - **#3583** is a draft at `90bfc731a`, unchanged since 09:52Z.
   - **Overlap check.** Its real diff from merge base `e2470ec4d` is 6 files: `train-in-space.rs`, `in_space_long.rs`, `claims.json`, `deepseek-9/{ORDER,REPORT}.md`, and `float-training.md`. Only the notebook is shared, and it takes added rows only.
   - **The basis.** The umbrella reading and its six conditions are claude-263's decision review (ALLOW, 09:48Z, logged on #2028).
3. **Inputs.** claude-262's seal `d04505ac…00e3` and claude-263's serve seal `7216cd08…96e` are in the store, read back on 3 Nov.

Classed instrument: the miss is a parity fault (§4.2).

## Files in scope

- `crates/uor-r4-float-train/src/hamilton_sheet.rs`
- `crates/uor-r4-float-train/src/geometric_stack_native.rs`
- `docs/harness/prep/one-model-end-to-end-i3-projection*`
- `docs/loop/tasks/one-model-end-to-end/claude-264/**`
- `docs/research/notebook/float-training.md` (added rows only)
- `docs/roadmap/items/one-model-end-to-end.md`

## Done when

1. `served_projection_q16` rounds `linear`'s FBS output only under the served bridge. A test fails with it set aside and passes with it, and the plain, Export and D11 forwards are unchanged.
2. The prep sheet is FIT, and its pre-run review is PASS.
3. Both seeds are re-scored on one 5090 at the PR head. The outputs are sealed and read back, and the pod is down.
4. I-3 is re-read at both seeds by the sheet's bar, against claude-263's integer replies.
5. A REPORT with negatives first, one notebook row, and the item's line updated.

## Stop and ask if

Each is decided in the unit, never waited on:
- The rounding changes a non-served forward: fix it before any pod.
- I-3 is partial or unchanged: the next unit reads the remaining sites.
- #3583's files grow into scope: re-check them before every push.
