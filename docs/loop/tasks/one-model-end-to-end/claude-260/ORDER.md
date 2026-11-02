# ORDER: one-model-end-to-end, claude-260 (the item's scope, widened for the copy read's key)

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Base: `5e50f3e19`.

## Goal

Casey, 2 Nov 2026: "your goal is to get us to a working chat model using the training and serving methods we have described - you need to figure out how to stitch all of that together". Asked "Trace the copy read on conv-a to confirm the cause before I build the new key?", Casey answered: "um yes I think so". The trace confirmed the cause ([claude-259's REPORT](../claude-259/REPORT.md), "Also found").

**Plan step (the claim check):** this unit is bookkeeping. It unblocks path step 4 of the lab's order (`local/claude/onboarding/run-claude.txt`): one model trained with all of its parts, exported, served and scored end to end. **What it unblocks in the model:** the copy heads' key. The next unit (claude-261) changes it in training, in the artifact and in serving.

**Why a unit of its own.** The loop's guard reads an ORDER's scope from the item file on `main`. The item's scope holds the float read (`flock_read_hamming.rs`) and the trainer, but not the files the key also changes:
- the float export (`fbs_export.rs`);
- the snapshot (`fbs_snapshot.rs`);
- the artifact format (`stack/format.rs`);
- the integer read (`stack/hamming.rs`);
- the synthetic fixtures and tests (`stack/synthetic.rs`, `stack/tests.rs`).

The guard refused the first edit to `fbs_export.rs` under the earlier form of this ORDER. So the scope merges first, and the key follows as claude-261.

## Cancel probe (run first; results)

1. **Already in scope?** `docs/roadmap/items/one-model-end-to-end.md` at the base `5e50f3e19` lists none of the six files. Not cancelled.
2. **Another lab's open order on them?** `gh pr list --state open` shows only this draft (#3578). DeepSeek's PR #3576, the last one on `stack/format.rs`, merged as this unit's base. Not cancelled.

Classed instrument (no model ran).

## Files in scope

- `docs/loop/tasks/one-model-end-to-end/**`
- `docs/roadmap/items/one-model-end-to-end.md`
- `docs/roadmap/README.md` (the render only)

## Done when

1. The item's `scope:` adds the six files above, and nothing else in the item changes.
2. `cargo run -p xtask -- roadmap render --check` is current.
3. The REPORT says which unit uses the scope, and the item stays open.

## Stop and ask if

Each of these is decided in the unit, never waited on:
- **The widened scope overlaps another lab's open order.** Narrow it to the files the key needs, and record why.
