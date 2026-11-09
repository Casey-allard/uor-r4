# ORDER: one-model-end-to-end, claude-342

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Base: `67972456d`.

## Goal

**Count how far back the question sits from the scored rows of TRAIN, then take the decision review that claude-339's row 4c names.**

- **Why now.** claude-339 read row 4c: the carry did not reach the asked value on language. Its sheet says TRAIN's distances were not counted, and that the count says whether 32 rows reach the question in training. It also lost stopping, for a cause it could not isolate.

**The work:**
1. **A laptop counter**, `train-question-reach`, over TRAIN d3mwcap6 (tokens, response mask, manifest). For each root: scored rows with a question before them, and how many sit within 4, 8, 16, 32, 64 and 128 rows of the question's start and of its end, over all scored rows and over each reply's first row.
2. **The reading, fixed before the count.** On a dialogue root (`oasst2`, and `cap1` for mw-balp): 32 **reaches** the question when at least 80% of its scored rows have the question's start within 32 rows. Under that it **does not reach**. Windows are ignored, so the count is an upper bound on what training sees.
3. **The decision review**, reading the whole project first: a question-row placement on chat windows, a longer carry, or a control that separates the causes of the lost stopping (a no-ask run at the new trainer commit, and an ask arm without the pull). It is logged as `[claude] owner log:` on #3407, with its fresh review.

**Claim check:**
- **(a) Step:** step 4, one model trained, served and scored end to end, for the recall and stopping gaps.
- **(b) What it unblocks:** which training run comes next. No GPU is used in this unit.

**Not in scope:** any training run, new mechanisms, the CPU port (DeepSeek's lane).

## Cancel probe (run first)

1. claude-339 is merged and verified.
2. No open PR of mine, and none touching these files.
3. The TRAIN files match the manifest's hashes.

**Result (at claim):** pass.
- claude-339 merged as `67972456d9ded94737a0a7bb1d0b36798560024b` and was verified on fresh `main`.
- No open claude PR.
- TRAIN's two hashes match the manifest (`0273173e…`, `da506e85…`), checked after this ORDER was first committed and before the count.
- `loop next` offers only DeepSeek's CPU-repair direction, so this ORDER is hand-written under Casey's harness exception, as claude-331 to claude-339 were.

## Files in scope

- `crates/uor-r4-float-train/src/bin/train-question-reach.rs` (new)
- `docs/loop/tasks/one-model-end-to-end/**`
- `docs/research/notebook/float-training.md`

## Done when

1. The counter has a host test that pins its counts.
2. The REPORT gives the count per root, reads it by the rule above, and records the decision.
3. A fresh exact-head review passes the PR, which merges and is verified on `main`.

## Stop and decide

Nothing waits on Casey (§5.1). A decision that would change a bar goes through its own decision review and is logged.
