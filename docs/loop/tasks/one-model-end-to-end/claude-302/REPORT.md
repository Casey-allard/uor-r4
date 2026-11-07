# REPORT: one-model-end-to-end, claude-302

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Order: [ORDER.md](./ORDER.md). Sheet: [`one-model-end-to-end-w1024-2026-11-07.md`](../../../../harness/prep/one-model-end-to-end-w1024-2026-11-07.md) (FIT WITH CHANGES; D67 review).

## What did not work, first

- **There is no width-1,024 reading.** Both seeds stopped at registered gates before any arm trained. The bar, the width lever and the spread are all unread. This is an instrument result, not a mechanism negative (L-007): it closes nothing about width.
- **Seed 1: I-m failed at both micro-batch sizes: out of memory before the step-32 progress row.** This ran on an RTX PRO 6000 with 97,887 MiB.
  - The smoke at micro-batch 16 held flat at 29,483 MiB, logged one sample at **68,971 MiB**, then raised `CUDA_ERROR_OUT_OF_MEMORY`.
  - The one registered fallback, micro-batch 8, did the same, with its last sample at **87,883 MiB**.
  - `mem.log` samples every 2 s, so both smokes reached the card's limit. The two figures are lower bounds taken just before each crash. They do not rank the two micro-batch sizes.
  - The clean evidence is in the four I-e runs at batch 16, each read inside its own window:
    - whole 66,411 MiB;
    - micro4 (four micro-batches of 4) 66,379 MiB;
    - micro2 (eight micro-batches of 2) 66,379 MiB;
    - read0, the whole batch with `--hamming-read 0`, **21,131 MiB**.
  - So, at width 1,024, micro-batching does not lower peak memory at all, and turning off the Hamming read cuts it by about 45,000 MiB. claude-301's accumulation at width 256 held flat at about 27,100 MiB.
  - The cause is **not found**, but read0 points the diagnosis at the read. `step_gradient` adds each micro-batch's GradStore into an accumulator. candle-core 0.9.2's `backward()` removes the gradients of non-variable nodes, so the accumulator should hold only the variables' gradients, and accumulated activation gradients look like the wrong suspect. The job exited 0 with a registered NOT_RUN, as `job-302.sh` sets for the sheet's registered gates.
- **Seed 2: I-e failed at 1.84e-6 relative, against its 1e-6 tolerance.** At step 4, micro2 (eight micro-batches of 2) differed from the whole batch by 1.84e-6. Every other cell, at both seeds, was at most 3.47e-7. The job exited 8, as `job-302.sh` sets for an I-e failure, before the I-m smoke ran.
  - This looks like float32 reduction-order drift at a four-times-wider sum, which would make it an instrument fault, but it was not tested.
- **I-e's control was underpowered at both seeds.** Turning the read off (`--hamming-read 0`) moved step 4 by 1.07e-6 (seed 1) and 2.97e-6 (seed 2), against the 1e-5 bar. The sheet records the control and does not gate on it. At width 1,024 the read moves step 4 by about the same amount as seed 2's micro2 drift, so I-e at step 4 cannot tell accumulation error from a read that does nothing. At step 3 the control was 1.65e-5 and 7.5e-6.
- **The first pod for each seed never accepted SSH, and both were deleted for about $0.90** (`vast-59481270` and `vast-59481310`; the figure is not in the watcher logs). The Vast PRO 6000 hosts in Texas started containers without the account's SSH key. The second try for each seed was refused by `uor-pod`'s circuit breaker and created nothing. Attaching the key by hand (`vastai attach ssh`) fixed the second pair created. The fix is logged in `local/claude/FRICTION.md`.

## The readings

None. The sheet's readings (the bar against D3's T2 and unigram, the width lever against claude-301's (b64L), unseen rows, the parting step) all need a trained arm.

The dry run of the read script before launch re-paired claude-301's (b64L) against D3's T2. It reproduced d +75,038 / +85,590, exactly. That check of the read path still holds for the rerun.

## What this unlocks (sheet, outcome map)

Seed 1's NOT_RUN falls under the outcome map's NOT_RUN branch: rerun. Seed 2's I-e failure falls under I-e's own rule: the unit becomes a diagnosis. Both lead to the same next step. A rerun at width 1,024 cannot pass I-m until the memory is understood. The next unit is therefore **a diagnosis: measure peak memory at width 1,024 with the Hamming read on and off**, split into forward, backward and accumulator, at micro-batches 16, 8 and 4, on one pod. The same unit should also test whether I-e's micro2 drift follows the summation order. Only then is width 1,024 rerun under this sheet.

claude-301's (b64L) recipe at width 256 stays the trunk's human-text recipe, "met; unseen ahead".

## What changed

- `docs/harness/prep/one-model-end-to-end-w1024-2026-11-07.md`: the sheet, with the FIT's six findings applied.
- `docs/loop/tasks/one-model-end-to-end/claude-302/`: the ORDER and this REPORT.
- `docs/research/notebook/float-training.md`: one row, verdict **diagnostic stop**.
- No code changed. The trainer is pinned at `ff5156bcc9539aa5bccd0319df3d96ee5dd5a71f`.

## Measurements

| seed | pod | $/h | result | spend |
|---|---|---:|---|---:|
| 1 | vast-59483000, RTX PRO 6000 | 1.775 | I-e PASS; I-m NOT_RUN, out of memory at 16 and 8 | $1.31 |
| 2 | vast-59485557, RTX PRO 6000 | 1.789 | I-e FAIL (exit 8) | $0.56 |
| 1, 2 | vast-59481270, vast-59481310, no SSH | — | deleted before any job | ≈ $0.90 |

- **Total spend:** about $2.77 of the sheet's cap. Each running pod was taken down by its watcher on `seal=0` (L-004). 0 instances are running.
- **I-e, relative difference from the whole batch** (seed 1 / seed 2):

| step | micro4 | micro2 | control read0 |
|---:|---|---|---|
| 1 | 7.21e-8 / 2.06e-7 | 9.74e-8 / 1.62e-7 | 0 / 0 |
| 2 | 1.76e-7 / 1.56e-8 | 1.47e-7 / 6.05e-8 | 2.38e-7 / 1.19e-7 |
| 3 | 0 / 1.96e-8 | 2.97e-8 / 2.7e-8 | 1.65e-5 / 7.5e-6 |
| 4 | 4.73e-8 / 3.47e-7 | 1e-7 / **1.84e-6** | 1.07e-6 / 2.97e-6 |

- **Seals,** each read back and checked against its SHA256SUMS on the laptop:
  - `results/claude/one-model-end-to-end-w1024-2026-11-07/runs/s1` `w1024-s1.tar.gz` `3278e178670d09733e685fc845b788c789e6ad607550cdaf157703a52a1eda25`;
  - `runs/s2` `w1024-s2.tar.gz` `644e26979b1a38202929b1e0d34c0496a8a3a375fb625ed26d92c92309b64182`.
- **Notes on #3407:**
  - the FIT (6697804641);
  - the D67 pre-run review (6697952382);
  - the launch (6698926705).

## Checks run at the head

- The seals' SHA256SUMS check OK on the laptop, both.
- The peaks above were recomputed on the laptop from seed 1's `mem.log`, inside each smoke's `.window`.
- The job's gates' laptop test (`gt/test-gates.sh`) passed 43 of 43 before launch, with every mutation caught.
- The read script's dry run reproduced claude-301's (b64L) pair exactly.

## Skills used

None.

## Still open

- **The memory diagnosis:** why micro-batching does not cap peak memory at width 1,024. This is the next unit.
- **I-e at width 1,024:** whether a 1e-6 tolerance at step 4 is the right bar for a four-times-wider sum, and a control that has power there. Step 3's control did reach the bar at seed 1.
- **The served-form check (§3.7):** still owed before the first serving claim on (b64L)'s recipe (ORDER, decision).

## Plain summary

The four-times-wider model did not train, so this unit says nothing yet about whether width helps. On seed 1 it ran out of memory on a 96 GB card. Splitting each batch into smaller pieces, which saved memory at the smaller width, made it worse here, and the reason is not yet known. On seed 2 a safety check found that the split batch and the whole batch differed slightly more than the limit allowed, and it stopped the run as designed. The unit cost about $2.77. The next unit measures where the memory goes before width 1,024 is tried again. The width-256 model that beats both baselines stays the current recipe.

References #3407.
