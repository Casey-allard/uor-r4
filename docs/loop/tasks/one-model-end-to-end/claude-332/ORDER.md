# ORDER: one-model-end-to-end, claude-332

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Base: `70f52ffe5`.

## Goal

**The carry lever on B2: the fresh FIT, the code and the run.** [claude-331](../claude-331/REPORT.md) wrote the sheet [one-model-end-to-end-carry-hold-2026-11-09.md](../../../../harness/prep/one-model-end-to-end-carry-hold-2026-11-09.md):
- a held ask write (`--hamming-hold 8`), with the cuts `−aS` and `−H`;
- value-rank readings along the value's path.

The sheet read NOT_FIT after two rounds. Every finding is applied in the text but not reviewed, so this unit starts with a fresh FIT.

1. **Apply claude-331's exact-head review findings to the sheet first:**
   - **Major:** rank (r1hs) against `q_v = w_y ⊙ p_v` at the best non-negative scale, not against the raw `p_v`, and pin that case in I-V.
   - **Minors:**
     - the export refusals already exist (`fbs_export.rs:402–405`, `:527–529`), so only decode needs a new clause;
     - state outcome 1's first-match intent against outcome 2;
     - give outcome 4's last branch a next step.
   - **Nit:** 0.017–0.394 is the read weight in general, not `w_y` alone.
2. **A fresh non-author FIT of the sheet.** Two rounds at most. If round 2 reads NOT_FIT, the unit stops at the sheet, and the REPORT names the remaining findings.
3. **If FIT: the code**, behind `--hamming-hold`, default off, in `crates/uor-r4-float-train`.
   - The sheet's instrument checks (I-1, I-H1 to I-H5, I-V) are unit tests, each with a mutation that makes it fail.
   - The default paths are byte-identical (I-1).
4. **If the code merges inside the unit:** a D67 pre-run review, then the two-seed run on Vast (about $1, inside the caps), read against the sheet's bar, with a notebook row.

**Claim check:**
- **(a) Step:** the run-claude order's step 4, one model trained and scored end to end. This is the recall gap (11). Under claude-326's lab decision, no language run on this line starts until a read passes B2.
- **(b) What it changes:** the model gains a learned hold, which carries the value the ask heads choose at the question row to the scored row four tokens later. It is the step between choosing a fact from the prompt and saying it in the reply.

## Cancel probe (run first)

1. No open PR adds a hold, a carry or an ask-stream cut to the Hamming read.
2. claude-331's sheet, ORDER and REPORT are on `main` at the base (`70f52ffe5`).
3. **Compute:** the laptop CPU for the sheet, the FIT, the build and the tests. A GPU only after FIT and a pre-run review.
4. **Disk:** 16 GiB free, under 30. One target dir in this worktree, removed on merge.

**Result (instrument, at claim):** pass. The only open PR matching hold, carry, Hamming or ask is this one (#3881); claude-331's sheet, ORDER and REPORT are on `main` at `70f52ffe5`; 15 GiB free.

## Files in scope

- `crates/uor-r4-float-train/src/flock_read_hamming.rs`
- `crates/uor-r4-float-train/src/in_space_long.rs`
- `crates/uor-r4-float-train/src/geometric_stack_native.rs`
- `crates/uor-r4-float-train/src/bin/train-in-space.rs`
- `crates/uor-r4-float-train/src/fbs_export.rs`
- `docs/harness/prep/one-model-end-to-end*`
- `docs/loop/tasks/one-model-end-to-end/**`
- `docs/research/notebook/float-training.md`
- `docs/roadmap/items/one-model-end-to-end.md`

## Done when

1. The sheet carries claude-331's review findings and a FIT line from a fresh non-author reviewer.
2. If FIT:
   - the code is merged;
   - its instrument checks pass, and their mutations fail;
   - the default paths are byte-identical.
3. If the run happens, both seeds are:
   - sealed to the store and read back;
   - read against the bar;
   - recorded in a notebook row.
4. A fresh exact-head review passes each PR, and each one merges and is verified on `main`.

## Stop and ask if

Nothing waits on Casey (§5.1). Bound: this is the second of the line's two units. The sheet, the code and the run may land in separate PRs. Whatever is unfinished at the end of this unit is named in the REPORT, with the next unit's first step.
