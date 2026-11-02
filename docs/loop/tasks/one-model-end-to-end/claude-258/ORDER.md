# ORDER: one-model-end-to-end, claude-258 (train as served)

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Base: `7663cf2cef3e67fb386996c87337c0b91349a4a6`.

## Goal

Casey, 2 Nov 2026: "Stop Q5 now (Recommended)", to "Stop the Q5 track now and move to one model trained on the exact values it is served with?". Before it: "I thought we had agreed on returning to the end-to-end same-value trained as served method? I thought we were using vectors and designing attention and token prediction?"

**Plan step (the claim check):** PLAN.md's 29 Oct direction, step 4, "One model trained with all of them, exported, served on the laptop and scored on prompts, with tokens/s". This unit changes the model that trains: training reads the values the integer artifact serves, so the trained and the served model hold the same values.

## This unit

Unit 1 of 3 (the bound is in the lab's place record): build training as served. Unit 2: the run's prep sheet, FIT and pre-run review. Unit 3: the GPU run, export, laptop serving, M1's bars and tok/s. No trace, audit or new instrument in this track.

**Claimed by hand.** `loop claim` refused: the item is claimed by PR #3554 (deepseek-1), whose scope is `docs/labs/one-model-end-to-end/**` and its task folder only. No open PR changes a file this unit edits (checked 2 Nov, ~13:50Z). Casey, 1 Nov: "if kimi is holding on to something or any other lab is, you may supercede them to get whatever done". This unit never edits #3554 or its files.

## Cancel probe (run first)

1. Does `train-in-space` already train on rounded values? (`grep -n round_ crates/uor-r4-float-train/src/in_space_long.rs crates/uor-r4-float-train/src/bin/train-in-space.rs`). If yes, use it; the build part cancels.
2. Which training loop (r-drop)'s recipe runs (`objective: "cell"`, turn memory, Hamming read), and do `round_tables_as_exported`, `round_hamming_as_exported` and `round_memory_as_exported` (`fbs_export.rs`, public) accept that loop's model on its device?

## Files in scope

- `crates/uor-r4-float-train/src/in_space_long.rs`
- `crates/uor-r4-float-train/src/bin/train-in-space.rs`
- `docs/loop/tasks/one-model-end-to-end/claude-258/**`
- `docs/roadmap/items/one-model-end-to-end.md`
- `docs/research/notebook/float-training.md`

## Done when

1. **`train-in-space --train-as-served`** (training only): before each step's forward, the trunk tables, Hamming reads and turn-memory weights are rounded as the export rounds them; the gradient at those values updates a float master copy; the saved checkpoint holds the rounded values.
2. **A test:** a few-step host run with the flag saves a checkpoint that rounding as exported leaves unchanged: zero export gap by construction. Negative gate: without the flag, the same run's checkpoint changes under rounding.
3. **A test:** without the flag, training is byte for byte as at the base.
4. The float-train tests pass, and the stub-nvcc cuda check passes.

## Stop and ask if

- the export's rounding cannot run inside the training step without a new instrument;
- the rounded step needs a mechanism change (a new loss, a new parameter), not a number-format change.
