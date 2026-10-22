# REPORT: m1-a8-erosion-diagnosis, claude-42 (close the item: notebook row and status)

Lab: claude. Base: `8d488457e`. Order: [ORDER.md](ORDER.md), committed first (`a17c5bfc6`). The cancel probe found the item `ready`, and no erosion row in `m1-discrete-attention.md`. So the task stands.

## What did not work, first

- **The diagnosis does not single out a cause.** The notebook verdict is therefore **inconclusive**, the notebook's word for "the evidence cannot separate the explanations".
  - The host data rules out one route: no cell was emptied or filled.
  - It shows that whole-vocabulary placement churn is not higher in the arm that lost more reach.
  - Placement stays a suspect on the row-weighted measures, and the read is not cleared.
- **The item could not close without three fixes outside it:**
  - #2997 added its `notebook:` field;
  - #2999 fixed `pr-check`, which refused any row added inside a notebook's table. This branch first failed `scope` on exactly that, then passed once `main` was merged in.
  - the corpus inputs had to be found inside the pod data tars (#2996).

## What changed

| Commit | What |
|---|---|
| `a17c5bfc6` | This ORDER, alone. |
| `cd480b71e` | One row appended to `docs/research/notebook/m1-discrete-attention.md`. `status: done` in the item. `docs/roadmap/README.md` re-rendered. |
| this REPORT's commit | This REPORT. |

The done-when, "a diagnosis note … that states which causal claims it does and does not support; recorded as one notebook row", is met as follows:
- **The note:** `docs/labs/m1-a8-erosion-diagnosis/README.md`, built in #2992 (design), #2994 (controls, Q1, Q3) and #2996 (P_succ, predecessors, Q5).
- **The record:** this notebook row.

## Measurements

None new. The row restates #2996's results:
- **output:** sha256 `789e3aa8…`, reproduced independently by DeepSeek;
- **tokens moved:** 80 against 91;
- **P_succ rows with a moved target:** 1,461 against 1,369;
- **row-weighted predecessor distance:** 8.564 → 10.123 against 9.799;
- **controls:** all seven pass.

## Checks run at the head

- `cargo xtask roadmap check`: 88 OK, 23 done. `roadmap render --check`: current.
- Claim-wording on the notebook: passed.
- `loop report --check` and `loop pr-check --local`: see the PR body.

## Skills used

scope-guard: one row, one status line, and the shared roadmap render.

## Still open

- **Separating placement from the read needs admission.** That means a device run under its own FIT sheet, for example Relation's Q1 table with FrozenZero's gates, and the reverse. It would be a new item, not this one.
- **Cleanup:** about 908 MB of census inputs under `local/claude/a8-census-data/` and `local/claude/a8-corpus/`, deleted after this merges.

## Plain summary

The diagnosis is written up and logged. The saved data shows where words moved, but not whether those moves or a change in the attention caused the extra loss. Only a GPU test can settle that, so the logged verdict is "inconclusive".
