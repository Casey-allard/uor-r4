# ORDER: one-model-end-to-end, claude-313

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Base: `ab4a5b328`.

## Goal

**Run the sheet claude-312 registered (#3831):** [`one-model-end-to-end-b64l-mwbalp-2026-11-07.md`](../../../../harness/prep/one-model-end-to-end-b64l-mwbalp-2026-11-07.md). The recipe is claude-301's (b64L) at pin `ff5156bcc`, with only TRAIN changed to `mw-balp/train`, at seeds 1 and 2, on two RTX 5090 pods. Then the sheet's Step 6 on the laptop. The reading is decided by the sheet's outcome map, top to bottom.

Claim check: (a) the step is "pipeline first" (#2028, 6709947526): the next training run is judged by the served-reply score. (b) What it changes is TRAIN alone, so a change in stopping is credited to the corpus.

**The pre-run review is GO** (the sheet's "Pre-run review"). The run goes ahead on it (§5.1, §5.10).

## Harness exception, recorded

`loop claim` refused: deepseek-21's PR #3833 holds the item's claim. That PR is EOS-rank and per-turn diagnostics, not this run. This ORDER is written by hand, as claude-309's was, under Casey's "i approve you to make harness exceptions as needed".

#3833 also edits `served-reply-panel.rs`. This run scores with the binary the sheet pins, `be3d4cb0…`, saved at `local/claude/claude-312/served-reply-panel-be3d4cb0`, whatever merges meanwhile.

## Cancel probe (run first)

1. No open PR or claim runs (b64L) on `mw-balp/train`. #3833 does not.
2. The four scripts in `local/claude/claude-312/` hash to the sheet's pins.
3. The scorer binary hashes to `be3d4cb0…`.
4. `uor-pod status`: the caps hold with two more pods.

## Steps

The sheet's Steps 2 to 6. Each pod gets its own lease and its own `watch-312.sh`, armed with a completion monitor. The seals go to `results/claude/one-model-end-to-end-b64l-mwbalp-2026-11-07/runs/s{1,2}/`.

## Files in scope

- `docs/loop/tasks/one-model-end-to-end/claude-313/**`
- `docs/research/notebook/float-training.md`

Scripts, seals and outputs live in `local/claude/claude-313/` and `local/claude/claude-312/`, and are not committed.

## Done when

1. Both seeds are trained, sealed and read on the laptop, or each NOT_RUN is classified by the sheet.
2. The outcome row is named.
3. The REPORT is written, negatives first, with one notebook row.

## Stop and ask if

Nothing waits on Casey (§5.1). Spend is about $4, with a budget notice at $5.60. A healthy run is never cancelled for a self-set budget. Bound: one unit.
