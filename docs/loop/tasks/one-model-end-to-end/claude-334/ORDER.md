# ORDER: one-model-end-to-end, claude-334

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Base: `d7cc2b930`.

## Goal

**The carry lever on B2, second unit: the cuts and readings, the pod tests, the FIT, then the run.** [claude-333](../claude-333/REPORT.md) landed Unit A, the hold (`--hamming-hold`), default off. This unit finishes what claude-333's ORDER bounded at two units. Sheet: [one-model-end-to-end-carry-hold-2026-11-09.md](../../../../harness/prep/one-model-end-to-end-carry-hold-2026-11-09.md).

1. **Unit B, the cuts and readings**, behind default-off paths in `crates/uor-r4-float-train` (builds and finite tests need no FIT, CHARTER §4 item 5):
   - `−aS`, a new `Ablation` with classes `[[false, true], [false, false]]`, so the ask heads' write at the stream read is 0;
   - `−H`, `g := 0` at evaluation;
   - a capture of the chosen rows only (the `Q` rows and `H4`), copied to the host for those rows;
   - the six value ranks over the found rows, beside the found fraction, at the `Q` row and at `H4`;
   - `g` and `|w_y|` logged per checkpoint.
   - Tests: I-H5 and I-V, each with a mutation that fails it.
   - The two minors from claude-333's review: I-H3 asserts that row 6 found its candidate, and load checks `d2.hold`'s length against the ask rows.
2. **One short pod:** every CUDA test of both units (I-H1 to I-H3, the forward and gradient tests of Unit B), each CUDA mutation, and the I-1 digests at the base and the head.
3. **A fresh non-author FIT of the sheet and the code together.** Two rounds at most. If round 2 reads NOT_FIT, the unit stops before the run, and the code still merges, default off.
4. **If FIT:** a D67 pre-run review, then the two-seed run on Vast (about $1, inside the caps), read against the bar, with a notebook row.

**Claim check:**
- **(a) Step:** the run-claude order's step 4, one model trained and scored end to end; the recall gap (11). Under claude-326's lab decision, no language run on this line starts until a read passes B2.
- **(b) What it changes:** the model trains with the hold on, and the readings say where the found value is lost between the question row and the scored row: at the ask write, in the hold, in the stream, or at the readout.

**Not in scope, named here:** the integer export and the decode step refuse the ask heads and the hold, so this lever is not yet served. Put to Casey on 8 Nov as a candidate next unit, whatever B2 gives.

## Cancel probe (run first)

1. No open PR adds a hold, a carry, an ask-stream cut or value ranks to the Hamming read.
2. claude-333's code, ORDER and REPORT are on `main` at the base (`d7cc2b930`).
3. **Compute:** the laptop CPU for the code and host tests; one short GPU pod for the CUDA tests (finite kernel tests, no FIT needed); the run only after FIT and a pre-run review.
4. **Disk:** at least 5 GiB free. One target dir in this worktree, removed on merge.

**Result (instrument, at claim):** pass. The one open PR matching the terms is DeepSeek's #3885, docs only in `cpu-training-pilot`. claude-333's sheet, ORDER and REPORT are on `main` at `d7cc2b930`. 8.7 GiB free.

## Files in scope

- `crates/uor-r4-float-train/src/flock_read_hamming.rs`
- `crates/uor-r4-float-train/src/in_space_long.rs`
- `crates/uor-r4-float-train/src/geometric_stack_native.rs`
- `crates/uor-r4-float-train/src/bin/train-in-space.rs`
- `docs/harness/prep/one-model-end-to-end*`
- `docs/loop/tasks/one-model-end-to-end/**`
- `docs/research/notebook/float-training.md`
- `docs/roadmap/items/one-model-end-to-end.md`

## Done when

1. Unit B is merged, default off, and verified on `main`; each instrument check passes and its mutation fails.
2. Every CUDA test of both units has run on a pod, and each CUDA mutation fails; the I-1 digests match between base and head.
3. The sheet has a FIT line, from a fresh non-author reviewer who read the code.
4. If FIT and the run happens, both seeds are sealed to the store and read back, read against the bar, and recorded in a notebook row.
5. A fresh exact-head review passes each PR, and each one merges and is verified on `main`.

## Stop and ask if

Nothing waits on Casey (§5.1). Bound: this is the second and last unit of claude-333's two-unit bound. Past it, the unfinished parts are named in the REPORT, PLACE.md records them, and the line goes to the next plan step (P2).
