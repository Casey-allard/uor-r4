# ORDER: one-model-end-to-end, claude-322

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Base: `d1f749e7e`.

## Goal

**Train (b64L) on D3 plus the k = 8 capped `mw-balp/train` six times, and see whether stopping returns when the scored EOS per window seen matches the × 1 mix's.** This is claude-321's decision review (row 5, #3855):
- claude-321's capped × 3 mix stopped on 1 and 0 of 256. It saw about 0.82 scored EOS per window, against 1.06 for claude-315's × 1 mix, which stopped on 177 and 122;
- so the cap and the exposure could not be separated;
- the × 6 capped mix was counted before that run: 444,761 scored EOS over 417,421 usable windows, **1.07 per window seen**, (b64L-d3mw)'s density.

So this run holds the exposure at × 1's and keeps the cap. Whether stopping comes back then says whether the cap itself, and not the exposure, took it away.

1. **The mix** is `scale-probe mix fraction=1/1` over D3's four roots, then the k = 8 capped split six times (`cap1` to `cap6`). It must rebuild claude-321's counted × 6 mix byte for byte: tokens `0273173e…`, mask `da506e85…`, manifest `442ead2c…`. It goes into the store under this unit's root.
2. **The prep sheet** `docs/harness/prep/one-model-end-to-end-b64l-d3mwcap6-2026-11-08.md`. It follows claude-321's sheet in everything but TRAIN: claude-301's (b64L) recipe, seeds 1 and 2, 16,384 steps, one RTX 5090 Vast pod per seed. It pins:
   - the mix's shas, its build and its window count, 417,421;
   - the job, chain, seal, watch, seed and read scripts' shas;
   - the export, serve and score path, unchanged;
   - a fresh, non-author FIT and the session's pre-run review.
3. **The run and the reading:** `m2-fresh256` served replies, scored as in claude-321, with the no-memory panel pair against (b64L-D3). Recorded beside it: claude-321's capped × 3 and claude-315's × 1. **The first-line count** (the asked-for value, the prompt's other value, both) is a named descriptive reading, as claude-321's REPORT asked.

**The bars are unchanged from claude-321's sheet:**
- stopping: at least 128 of 256 EOS stops at both seeds;
- collapse: the most common EOS-stopped reply occurs under 64 times at both seeds;
- the panel: 104 complete and 52 pairs;
- the held-out pair is not `behind`.

Claim check:
- **(a)** This is Track A of "Both in parallel" (#2028, 6715318106), under "pipeline first" (#2028, 6709947526). The run is judged by the served-reply score.
- **(b)** It changes only TRAIN. No bar moves.

## Cancel probe (run first)

1. **No open PR trains a × 6 capped mix.** The one open PR is #3860 (deepseek-40, a fresh-panel opening probe on DeepSeek's own trunk).
2. **The inputs are present:**
   - D3's four train roots in the store (`results/claude/real-text-corpus-2026-11-05/*/train/`);
   - claude-321's k = 8 capped split (`3dee858f…` / `445e8744…`);
   - `scale-probe mix` (bin `31610a00…`);
   - claude-321's job, chain, seal, watch, seed and read scripts, as reviewed there.
3. **Compute:** no pod is running, so two stay inside 4 pods and $8/h. Expected spend is about $3, as in claude-321 ($3.01).
4. **Disk:** 3.9 GiB free on the laptop. The mix is 219 MB.

**Result, 8 Nov 12:40Z:** 1 to 4 hold as written. Two findings from the probe:
- the D3 mix built from claude-314's `d3-rebuild` is refused as a root (`manifest lacks source`), so the build reads D3's four roots from the store;
- `uor-pod status` warns that its Vast adapter is not built, so it does not list Vast pods. It is built before any pod comes up.

## Files in scope

- `docs/labs/one-model-end-to-end/**`
- `docs/harness/prep/one-model-end-to-end*`
- `docs/loop/tasks/one-model-end-to-end/**`
- `docs/research/notebook/float-training.md`
- `docs/roadmap/items/one-model-end-to-end.md`

## Done when

1. The mix rebuilds claude-321's three shas, and the store's copy reads back equal.
2. The sheet is committed with the mix's shas, its FIT and its pre-run review, before any pod comes up.
3. Both seeds run, are sealed to the store, and are scored. The REPORT gives the outcome row, negatives first, with one notebook row.

## Stop and ask if

Nothing waits on Casey (§5.1).
- **If the pre-run review fails:** the run does not start, and the REPORT says why.
- **If a seed is NOT_RUN for an instrument fault:** one repair and one rerun inside this unit.

Bound: one unit, two pods, $8 of spend.
