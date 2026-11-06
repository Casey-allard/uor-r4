# ORDER: one-model-end-to-end, claude-297

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Base: `92ea7ccdf45d7924f6cd3cac1498d262ee6ce692`.

## Goal

Casey's one track (#2028 6675163438, 6 Nov 2026, "Yes, one track"): the trunk's no-memory top-1 must beat its TRAIN's unigram and its T2 on the human held-out panel, beyond the paired band.

claude-296 (#3763) found that the cell objective is what fails on human text. `--objective nearest`, (near), learns it: 216,675 / 223,689 of 1,460,697 rows, behind T2's 253,018, with its loss still falling. This unit runs (near) with more steps and more data as a 2×2 at width 256, seeds 1 and 2, all four cells at the base. It follows the sheet [`one-model-end-to-end-steps-data-2026-11-06`](../../../../harness/prep/one-model-end-to-end-steps-data-2026-11-06.md):

| cell | TRAIN | steps |
|---|---|---|
| (n1S) | D1 | 8,192 |
| (n1L) | D1 | 32,768 |
| (n3S) | D3 | 8,192 |
| (n3L) | D3 | 32,768 |

## Claim check

- **(a) The step.** This is step 3 of the one track: "whatever it finds, until the bar is met". claude-296 merged as `92ea7ccdf45d7924f6cd3cac1498d262ee6ce692`.
- **(b) What it changes in the model.** It tests whether the shipped trunk's recipe, (near), meets the one-track bar with more steps or more data. If it does not, it shows which lever moves it.

## Cancel probe (run first)

Run at 07:11Z on 6 Nov, before any work. Class: instrument.
1. **No hold.**
   - There is no `local/loop/STOP`, and #2028's labels are `roadmap` and `roadmap:tracker`, with no `loop:pause` (`loop next`: kill switch off).
   - #3407's last four comments are DeepSeek's copy-key note and claude-296's intent, D67 and merge notes.
2. **The inputs are in the store:**
   - D1 and D3 in `results/claude/scale-probe-2026-11-05/inputs/` (D3: tokens 53.3 MB, mask 26.7 MB, manifest);
   - the panels in `results/claude/real-text-corpus-2026-11-05`;
   - D1's and D3's seen sets on the laptop, from claude-295.
3. **The caps:** `uor-pod status` shows 0 pods running, $0/h of $8/h, and 14 RTX 5090 offers from $0.908/h.

Result: GO.

## Done when

1. **The sheet** is at `docs/harness/prep/`, and its FIT is posted on #3407.
2. **The D67 pre-run review** of the job and watch scripts passes in a fresh context. It is posted on #3407 before a pod goes up.
3. **The runs**, or the stop rule's NOT_RUNs, are sealed under `results/claude/one-model-end-to-end-steps-data-2026-11-06/`, each pod's with a `SHA256SUMS` that was read back. The pods are down.
4. **The REPORT** gives, negatives first:
   - each cell against its own TRAIN's T2 and unigram, by the sheet's Bar, on all rows and on unseen rows;
   - the levers;
   - I-0 to I-5 and I-s;
   - one row in `docs/research/notebook/float-training.md`.

   It ends `References #3407`.

## Stop and ask if

"Ask" means decide, record it here and in the REPORT, and go on; nothing waits.
- **The sheet's stop rule:** an I-0 or I-3 failure stops that pod. A failed cell is NOT_RUN at its failing step.
- **The cap:** $14 and 4 hours. When a watcher's stop fires on a healthy run, extend it and renew the lease; never cancel it.
- **Casey says "hold":** no further launches.
- **The laptop's free disk falls under 5 GiB:** downloads pause until my own store-sealed copies are cleared.

## Files in scope

- `docs/harness/prep/one-model-end-to-end*`
- `docs/loop/tasks/one-model-end-to-end/**`
- `docs/research/notebook/float-training.md`
- `docs/roadmap/items/one-model-end-to-end.md`
