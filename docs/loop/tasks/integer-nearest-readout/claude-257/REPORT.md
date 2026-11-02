# REPORT: integer-nearest-readout, claude-257 (the structure step's replay, stopped)

Lab: claude. Item: [integer-nearest-readout](../../../../roadmap/items/integer-nearest-readout.md) (#3462). Order: [ORDER.md](ORDER.md). Rule: claude-247's [decision card](../claude-247/DECISION.md), as amended by claude-254's [decision card](../claude-254/DECISION.md). References #3462.

**The owner stopped this unit and the Q5 track on 2 Nov 2026, about 12:37Z.** Casey's words, in this lab's chat: "This whole Q track seems like an enormous waste of time and a derailment of our project"; then "Stop Q5 now (Recommended)". The item is held. This unit's code is not merged; it is kept as [`code-archived.patch`](code-archived.patch) in this folder.

## Tokens per second

No decode claim. Nothing served changed: the branch's code was taken back out before merge, so R decodes as at the base `5cf66a98d` (claude-243's 1,204 tok/s stands).

## What did not work, first

- **The unit was the wrong work.** The plan's first hurdle is one model end to end (PLAN.md's 29 Oct direction, step 4). Q1 to Q4 had already held (claude-246): R picks the reference token as often as the trained model. Q5 explains the leftover disagreements and unblocks no step. The lab's own run file forbade test-only units that unblock no step, and on 1 Nov the lab had estimated Q5 at four or five more units. It then ran claude-251 to claude-257 without bringing the bound back to Casey.
- **No replay was accounted for.** The smoke replayed F against T on DEV s1's 190 offsets where their picks differ (10,840 compared, in 130 windows). It was stopped by PID at 61 of 190: 0 accounted, 0 refused, all 61 ending at the nearest. That is a reading on DEV s1, outside Q5's set. It reads no Q5 position and is not Q5's FAIL.
- **The export's rounding alone flips a clear decision.** 57 of the 61 reach their first clear flip at step 0, layer 0, lane 23, at 1.19 × G₁₀: F's rate entry 12 against T's 13. All 57 are in windows whose first token is 0, so this is one shared step-0 decision seen 57 times, not 57 separate results. At step 0 both sides read the same token through the same code, so the difference is in the rounded weights only. Class: a reading of the export, not an instrument fault (I-Rerun held on every replay; nothing was refused).
- **The pass count.** The 61 replays took 117,180 passes (median 1,821, range 377 to 4,379), at about 0.13 s a pass, where the card priced a pass at milliseconds.

## What worked

- The run's own checks held. Pass 0 reproduced T's sealed window on every replay (I-Rerun), and no replay was refused.
- The stop lost nothing: the 61 finished replays were saved off the pod and stored with their checksums before the pod was deleted.

## What changed

- `docs/roadmap/items/integer-nearest-readout.md`: status `held`, with Casey's words and the stopped smoke's reading.
- `docs/research/notebook/m4-serving.md`: one row.
- This folder: this REPORT and `code-archived.patch`, the unmerged code: `train-in-space --decision-replay`, `fbs_snapshot.rs`'s trace reader and tally, and `fbs-toks`'s move to them (4 files, 1,549 insertions, 131 deletions, from `5cf66a98d` to `8189c0de0`). No code file differs from the base.

## Measurements

- **Inputs:** (r-drop)'s s1 checkpoint from `results/claude/one-model-end-to-end-2026-10-29/training-460fe46e/s1/`; F's and T's DEV s1 traces; F's deciles; G₁₀ 30,228 in R's units; the snapshot-s1 table.
- **Run:** one GPU on Vast (`vast-58792639`, $0.908/h), up about 4 h 22 min, about $3.96, deleted after the upload. Job 1 ran the replay at `e5f85fbab`; job 2 (the cross-head check at `8189c0de0`) was queued and stopped before it started.
- **Outputs:** `results/claude/integer-nearest-readout-replay-ft-2026-11-02/results-stopped.tar` (24.4 MB; SHA256 `487fe15c93ce94d52d9c09211086cdd3837cba920225fadb6908321607a2a5fb` in that folder's `SHA256SUMS`): the line file of 61 replays, the job logs, the tests' output and the count of differing offsets.
- **Far rate buckets by layer, over the 61:** 797, 9,003, 10,347, 7,782, 12,216 and 11,418 for layers 0 to 5.

## Checks run at the head

- `git diff 5cf66a98d -- crates/` is empty: no code change lands.
- `loop report --check`, `loop gates --full` and `loop pr-check --local` at the head (results in the PR).

## Skills used

None.

## Still open

- Nothing in this item is taken up. The next work is one model end to end, token → vector → vector → integer → token, trained on the GPU on the exact values the laptop serves, so that no trained-against-served gap is left to audit.
- The step-0 reading bears on that design: training on the rounded values the export serves removes the parameter half of the gap by construction.

## Plain summary

Casey stopped the Q5 track as a derailment. This unit was replaying, on the GPU, the places where the trained model and its rounded copy disagree. 61 of 190 had replayed when it stopped, and none was explained by near ties alone. In the windows that start with token 0, they disagree at the very first decision, because rounding the weights as exported moves a clear score there. The code is archived, not merged, and the item is held. The project moves to training one model end to end on the values it is served with.
