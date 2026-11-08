# ORDER: one-model-end-to-end, claude-315

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Base: `4ddc4cea9`.

## Goal

**Run the sheet claude-314 registered (#3839):** [`one-model-end-to-end-b64l-d3mw-2026-11-07.md`](../../../../harness/prep/one-model-end-to-end-b64l-d3mw-2026-11-07.md). The recipe is claude-301's (b64L) at pin `ff5156bcc`, with TRAIN set to the D3 + `mw-balp/train` mix, at seeds 1 and 2, on two RTX 5090 pods. Then the sheet's Step 6 on the laptop. The sheet's outcome map decides the reading, top to bottom.

Claim check: (a) the step is "pipeline first" (#2028, 6709947526): the next training run is judged by the served-reply score. (b) The change is TRAIN alone: D3 kept whole, with mw-balp's stopped dialogues added. A change in stopping or in held-out text is then credited to the mix.

**The pre-run review is GO** (the sheet's "Pre-run review"). The run goes ahead on it (§5.1, §5.10).

## Harness exception, recorded

`loop claim` refused: deepseek-25's PR #3838 holds the item's claim. That PR is a launch fix for an SFT job, not this run. So this ORDER is written by hand, as claude-313's was, under Casey's "i approve you to make harness exceptions as needed".

## Cancel probe (run first)

1. No open PR or claim runs (b64L) on the d3mw mix. #3838 does not.
2. The four scripts in `local/claude/claude-314/` hash to the sheet's pins.
3. The scorer binary hashes to `be3d4cb0…`.
4. `uor-pod status`: the caps hold with two more pods.

**Result, 7 Nov (instrument checks, all pass):**
1. The only open PR on the item is #3838. No open PR names (b64L), d3mw or the mix.
2. The hashes match the sheet: `job-314.sh` `ce50f738…`, `chain-314.sh` `7ca161a4…`, `seal-314.sh` `5a56d59f…`, `watch-314.sh` `565e5c3d…`.
3. `served-reply-panel-be3d4cb0` hashes to `be3d4cb0a7548409…`.
4. 0 pods running, $0/h of $8.00/h. 19 Vast 5090 offers; the cheapest is $0.563/h.

## Steps

The sheet's Steps 3 to 6. Each pod gets its own lease and its own `watch-314.sh`, armed with a completion monitor. The seals go to `results/claude/one-model-end-to-end-b64l-d3mw-2026-11-07/runs/s{1,2}/`.

## Files in scope

- `docs/loop/tasks/one-model-end-to-end/claude-315/**`
- `docs/research/notebook/float-training.md`

Scripts, seals and outputs live in `local/claude/claude-315/` and `local/claude/claude-314/`. None are committed.

## Done when

1. Both seeds are trained, sealed and read on the laptop, or the sheet classes each NOT_RUN.
2. The outcome row is named.
3. The REPORT is written, negatives first, with one notebook row.

## Stop and ask if

Nothing waits on Casey (§5.1). Spend is about $2.4–3.0, with a budget notice at $5.60. A healthy run is never cancelled for a self-set budget. Bound: one unit.
