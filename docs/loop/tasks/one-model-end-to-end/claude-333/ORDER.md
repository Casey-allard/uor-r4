# ORDER: one-model-end-to-end, claude-333

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Base: `2b33d5745`.

## Goal

**The carry lever on B2: the code first, then the FIT, then the run.** [claude-332](../claude-332/REPORT.md) stopped at the sheet [one-model-end-to-end-carry-hold-2026-11-09.md](../../../../harness/prep/one-model-end-to-end-carry-hold-2026-11-09.md), NOT_FIT after two rounds.
- Every finding was about a reading, a check or the outcome map. None was about the lever.
- Its lab decision: write the code and its tests first, and have the FIT read the sheet and the code together.

1. **The sheet:** apply claude-332's exact-head review findings (four minors, two nits):
   - the undertrained test at an extended arm's last checkpoint;
   - (r1h)'s found rows keyed on the `Q` row's found flag;
   - a not-finding branch of its own;
   - the once-only extension for every outcome;
   - "equal as numbers" at `g = 0`.
2. **The code**, behind default-off flags in `crates/uor-r4-float-train`. Builds and finite tests need no FIT (CHARTER §4 item 5).
   - **Unit A, the hold:**
     - `--hamming-hold h`, which requires `--hamming-ask 1`;
     - the per-token table `g`, saved as `d2.hold`;
     - the held term inside the stream read's combine, before the `q16` rounding;
     - the decode step's refusal clause.
     - Tests: I-1, I-H1 to I-H4, each with a mutation that fails it.
   - **Unit B, the cuts and readings:**
     - `−aS`, a new `Ablation` with classes `[[false, true], [false, false]]`;
     - `−H`, `g := 0` at evaluation;
     - the six value ranks over the found rows, beside the found fraction, at the `Q` row and at `H4`;
     - `g` and `|w_y|` logged per checkpoint.
     - Tests: I-H5 and I-V, each with a mutation that fails it.
3. **A fresh non-author FIT of the sheet and the code together.** Two rounds at most. If round 2 reads NOT_FIT, the unit stops before any GPU, and the code still merges, default off.
4. **If FIT:** a D67 pre-run review, then the two-seed run on Vast (about $1, inside the caps), read against the bar, with a notebook row.

**Claim check:**
- **(a) Step:** the run-claude order's step 4, one model trained and scored end to end. This is the recall gap (11). Under claude-326's lab decision, no language run on this line starts until a read passes B2.
- **(b) What it changes:** the model gains a learned hold, which carries the value the ask heads choose at the question row to the scored row. That is the step between choosing a fact from the prompt and saying it in the reply.

## Cancel probe (run first)

1. No open PR adds a hold, a carry or an ask-stream cut to the Hamming read.
2. claude-332's sheet and REPORT are on `main` at the base (`2b33d5745`).
3. **Compute:** the laptop CPU for the sheet, the code and the tests. A GPU only after FIT and a pre-run review.
4. **Disk:** 15 GiB free, under 30. One target dir in this worktree, removed on merge.

**Result (instrument, at claim):** pass. No open PR matches hold, carry, Hamming or ask; claude-332's sheet, ORDER and REPORT are on `main` at `2b33d5745`; 15 GiB free.

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

1. The sheet carries claude-332's review findings.
2. The code is merged, default off, and verified on `main`.
   - Each instrument check passes, and its mutation fails.
   - The default paths are unchanged (I-1).
3. The sheet has a FIT line, from a fresh non-author reviewer who read the code.
4. If FIT and the run happens, both seeds are:
   - sealed to the store and read back;
   - read against the bar;
   - recorded in a notebook row.
5. A fresh exact-head review passes each PR, and each one merges and is verified on `main`.

## Stop and ask if

Nothing waits on Casey (§5.1). Bound: two units for the code, the FIT and the run, which may land in separate PRs. Past the bound, the unfinished parts are named in the REPORT.
