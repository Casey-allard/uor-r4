# ORDER: one-model-end-to-end, claude-335

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Base: `734450768`.

## Goal

**The question carry on B2: the ask heads at the readout read ask with the question, carried to the scored row.** [claude-334](../claude-334/REPORT.md) closed the hold (a carry of the found value into the depth-2 stream) for `h = 8`. B1 passes 512 / 512 because the readout read's ask heads ask at the scored row with the question's own point. At B2's scored row `H4` they ask with `H4`'s point, the same token in every episode, so no table row can name the episode's key. This unit carries the question's ask point to `H4`. Casey agreed to this as the next unit in chat on 9 Nov ("Yes, next unit").

1. **The code**, behind default-off paths in `crates/uor-r4-float-train` (builds and finite tests need no FIT, CHARTER §4 item 5):
   - `--hamming-carry-question h`: at the readout read only, each ask head's asked point at row `t` adds `Σ_{s=1..h} c[x_{t−s}] · (point(x_{t−s}) + a[x_{t−s}])`. Here `c` is a learned per-token carry gain, `(vocab,)`, starting at 0. The sum runs over rows of the same window only. The found-copy pull builds its question point the same way.
   - `--hamming-attract-at both`: the pull at the question rows and the scored rows together. `scored` and `question` are unchanged.
   - `−C`, an ablation with the carry off at evaluation.
   - `c` by token group in `recall.jsonl`.
   - Save and load of `c`. The decode step and the export refuse the carry, each by name.
   - Tests, each with a mutation that fails it:
     - at `c = 0` the read is unchanged;
     - a constructed carry reaches exactly the row `h` back, and no further;
     - nothing crosses a window;
     - the pull's gradient reaches `c`;
     - `−C` drops the carry alone.
2. **The sheet:** a one-page pre-registration with the arms, the bar against B2-n, and an outcome map read from the scored-row trace (`kept_ask` at position `scored`). That trace already reads B1's success. No stream rank decides a branch.
3. **One short pod** for the CUDA tests and their mutations, and the I-1 digests at the base and the head.
4. **A fresh non-author FIT of the sheet and the code together.** Two rounds at most. If round 2 reads NOT_FIT, the unit stops before the run, and the code still merges, default off.
5. **If FIT:** a D67 pre-run review, then the two-seed run on Vast (about $2, inside the caps), read against the bar, with a notebook row.

**Claim check:**
- **(a) Step:** the run-claude order's step 4, one model trained and scored end to end; the recall gap (11). Under claude-326's lab decision, no language run on this line starts until a read passes B2.
- **(b) What it changes:** if B2 passes, the model answers a question four tokens after asking it. That is the step between B1's exact read and a reply.

**Not in scope, named here:** serving the carry (the decode step and the export), and a language run. The carry needs only the last `h` rows' ids at a decode step, so serving it is a small later unit.

## Cancel probe (run first)

1. No open PR adds a question carry, a carry gain or a scored-and-question pull to the Hamming read.
2. claude-334's code, ORDER and REPORT are on `main` at the base (`734450768`).
3. **Compute:** the laptop CPU for the code and host tests; one short GPU pod for the CUDA tests (finite kernel tests, no FIT needed); the run only after FIT and a pre-run review.
4. **Disk:** at least 5 GiB free. One target dir in this worktree, removed on merge.

**Result (instrument, at claim):** pass. No open PR matches the terms. claude-334 merged as `7344507680f816ba81ae9589e4b7f923fa094030` and was verified on fresh `main`. 16 GiB free.

## Files in scope

- `crates/uor-r4-float-train/src/flock_read_hamming.rs`
- `crates/uor-r4-float-train/src/in_space_long.rs`
- `crates/uor-r4-float-train/src/geometric_stack_native.rs`
- `crates/uor-r4-float-train/src/bin/train-in-space.rs`
- `crates/uor-r4-float-train/src/fbs_export.rs` (added at work: the export's by-name refusal of the carry lives there; the item's scope already holds it)
- `docs/harness/prep/one-model-end-to-end*`
- `docs/loop/tasks/one-model-end-to-end/**`
- `docs/research/notebook/float-training.md`
- `docs/roadmap/items/one-model-end-to-end.md`

## Done when

1. The code is merged, default off, and verified on `main`. Each test passes, and its mutation fails.
2. Every CUDA test has run on a pod, and each CUDA mutation fails. The I-1 digests match between the base and the head.
3. The sheet has a FIT line from a fresh non-author reviewer who read the code.
4. If FIT and the run happens: both seeds are sealed to the store and read back, read against the bar, and recorded in a notebook row.
5. A fresh exact-head review passes each PR, and each one merges and is verified on `main`.

## Stop and ask if

Nothing waits on Casey (§5.1). Bound: one unit for the code, the sheet and the run. If B2 still fails, the REPORT names the reading and the line takes its outcome map's next step, or P2 if the map gives none.
