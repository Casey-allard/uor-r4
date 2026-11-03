# ORDER: one-model-end-to-end, deepseek-9 (batched scorer)

Lab: deepseek. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Base: `b6ebf7002`.

## Goal

Add an opt-in batched path to `train-in-space --score-rows` for the full-scope nearest decode rules. Prompts are grouped only when their full lengths are equal, so the first unit needs no padding and preserves recurrent state, memory-row indices and reply-table keys.

The owner chose this unit after scoring spent over an hour on one CPU core while the GPU was partly idle: "Yes, scorer next". It supports Claude's copy-key run and the subsequent chat baseline without editing Claude's active copy-key branch.

## Cancel probe

1. **Already built?** At `b6ebf7002`, `decode_reply_with_mode` decodes one prompt at a time. No equal-length batching exists. Not cancelled.
2. **A direct replacement in flight?** Claude's #3579 also edits `in_space_long.rs` and `train-in-space.rs`, but the owner asked for support work and this branch changes only the scoring region away from the copy-key config and read code. It must merge forward after #3579 and must not alter that branch.
3. **Inputs and controls are known.** The pinned DEV prompt set is available locally and yields 310 prompts in 75 exact-length groups, with a largest group of 32 and mean group size 4.13. Not cancelled.

## First-unit boundary

Batch only when all of these hold:

- the objective decodes through the whole-table nearest read;
- the read scope is full;
- no forced opening is set;
- no served point key is set;
- the requested `--batched-decode 1` flag is on.

Every other arm falls back to `decode_reply_with_mode`. The default remains 0 until a CUDA exact-parity run passes.

## Files in scope

- `crates/uor-r4-float-train/src/in_space_long.rs`
- `crates/uor-r4-float-train/src/bin/train-in-space.rs`
- `docs/loop/tasks/one-model-end-to-end/deepseek-9/**`
- `docs/research/notebook/float-training.md`
- `docs/harness/claims.json`

## Done when

1. Equal-length prompts form rectangular groups without padding; every prompt appears once and each group respects the existing evaluation batch/context budget.
2. Full-scope nearest `Greedy`, `FlockRank`, `FlockCell` and `CycleStep` decodes can use one forward per group per step.
3. Forced-opening, served-key and scoped decodes retain the serial path.
4. Flock sampling keeps each prompt's original global sample index, and BOS/EOS termination preserves all active-row state.
5. Host grouping tests pass, the full float library passes under its required `CARGO_TARGET_DIR`, and the CUDA equality test compiles for the pod.
6. The REPORT records GPU parity as `NOT_RUN` on this host and makes no speed claim until it runs.

## Stop if

- Claude's copy-key branch changes the scoring region in a way that cannot be merged forward without semantic conflict.
- CUDA equality fails for a rule; that rule stays serial and is reported, never asserted equivalent.
- The batch path changes prompt order, memory keys or sampling indices by construction.
