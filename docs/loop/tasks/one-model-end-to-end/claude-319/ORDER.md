# ORDER: one-model-end-to-end, claude-319

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Base: `2ef27b81f`.

## Goal

**Run the sheet claude-318 registered (#3846):** [`one-model-end-to-end-b64l-d3mw3-2026-11-08.md`](../../../../harness/prep/one-model-end-to-end-b64l-d3mw3-2026-11-08.md). The recipe is claude-301's (b64L) at pin `ff5156bcc`, with TRAIN set to D3 + `mw-balp/train` × 3, at seeds 1 and 2, on two RTX 5090 pods. Then the sheet's Step 6 on the laptop. The sheet's outcome map decides the reading, top to bottom.

Claim check:
- **(a) The step is "pipeline first"** (#2028, 6709947526), Track A of Casey's "Both in parallel" (#2028, 6715318106). The next training run is judged by the served-reply score.
- **(b) The change is TRAIN's ratio alone:** mw-balp three times in place of once. A change in stopping, collapse or held-out text against (b64L-d3mw) is then credited to the ratio, with the confound the sheet names (fewer D3 passes, 2.93 against 4.73).

**The pre-run review is GO** (the sheet's "Pre-run review", round 2). The run goes ahead on it (§5.1, §5.10).

## Harness exception, recorded

`loop claim one-model-end-to-end` refused: deepseek-29's draft PR #3847 holds the item's claim. That PR repairs and extends a reply-SFT fine-tune; it does not pretrain (b64L), and it touches none of this unit's files (its own task folder, `docs/labs/one-model-end-to-end/reply-sft-2026-11-07/**`, its reply-SFT prep sheet and `claims.json`). So this ORDER is written by hand, as claude-313's and claude-315's were, under Casey's "i approve you to make harness exceptions as needed".

## Cancel probe (run first)

1. No open PR or claim runs (b64L) on the d3mw3 mix.
2. The five scripts in `local/claude/claude-318/` hash to the sheet's pins.
3. The scorer binary hashes to `be3d4cb0…`.
4. The store holds the mix at the sheet's path, and the seal root `runs/` is empty (L-002).
5. `uor-pod status`: the caps hold with two more pods.

**Result, 8 Nov (instrument checks, all pass):**
1. The only open PR on the item is #3847 (deepseek-29, reply-SFT). No open PR names (b64L), d3mw3 or the ratio.
2. The hashes match the sheet: `job-318.sh` `5dd570e6…`, `chain-318.sh` `5f591616…`, `seal-318.sh` `a49720dc…`, `watch-318.sh` `067eb369…`, `read-318.sh` `d393a6f7…`.
3. `served-reply-panel-be3d4cb0` hashes to `be3d4cb0a7548409…`.
4. `uor-store ls` lists `train/` with `manifest.json`, `response_mask.u8` (48.2 MB) and `tokens.u16` (96.3 MB); `runs/` returns 404.
5. 1 pod running (claude-317's `vast-59652743`, $0.563/h), so 3 pods and about $1.69/h with these two, inside 4 and $8/h. 15 Vast 5090 offers; the cheapest is $0.563/h.

## Steps

The sheet's Steps 3 to 6. Each pod gets its own lease and its own `watch-318.sh` in a detached `screen`, with a completion waiter. The seals go to `results/claude/one-model-end-to-end-b64l-d3mw3-2026-11-08/runs/s{1,2}/`.

## Files in scope

- `docs/loop/tasks/one-model-end-to-end/claude-319/**`
- `docs/research/notebook/float-training.md`

Scripts, seals and outputs live in `local/claude/claude-319/` and `local/claude/claude-318/`. None are committed.

## Done when

1. Both seeds are trained, sealed and read on the laptop, or the sheet classes each NOT_RUN.
2. The outcome row is named.
3. The REPORT is written, negatives first, with one notebook row.

## Stop and ask if

Nothing waits on Casey (§5.1). Spend is about $3.3, or $4.1 at the $0.70/h ceiling, with a budget notice at $5.60. A healthy run is never cancelled for a self-set budget. Bound: one unit.
