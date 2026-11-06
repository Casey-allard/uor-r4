# REPORT: one-model-end-to-end, claude-298

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Order: [ORDER.md](./ORDER.md). Sheet: [`one-model-end-to-end-d3-batch-2026-11-06.md`](../../../../harness/prep/one-model-end-to-end-d3-batch-2026-11-06.md).

## What did not work, first

- **Every cell ran out of GPU memory: verdict blocked.** (b64S) and (b64L) both stopped at both seeds with `Candle(DriverError(CUDA_ERROR_OUT_OF_MEMORY))`. Each stopped 47–84 s of process wall after launch, including the step-0 DEV evaluation. The only progress row is step 0, and the failing step was not recorded. No cell trained, so nothing was graded or paired.
  - The sheet classes out of memory at batch 64 as an instrument fault, not a reading (its Stop rules).
  - **The question stays open:** does batch 64 narrow the seed spread or move the trunk past D3's T2? Nothing here bears on the mechanism or on batch size as a lever.
- **The sheet's memory estimate was wrong by orders of magnitude.** It said width 256 at context 192 and batch 64 is "about 3 MB of activations a layer, so this is not expected".
  - In fact the smokes peaked at **32,108 MiB (seed 1) and 31,696 MiB (seed 2)** of the RTX 5090's 32 GB (`gpu.csv`; the card's total is not in the seal).
  - In each full run memory climbed from about 22 GB to 31–32 GB within about 40 s, and then the run failed:
    - s1 (b64S) 22,220 to 31,116 MiB;
    - s1 (b64L) 22,476 to 32,044 MiB;
    - s2 (b64S) 22,800 to 31,696 MiB;
    - s2 (b64L) 22,480 to 30,864 MiB.
  - **Whether this footprint is steady or grows per step is not separated by this run's evidence.** Memory is sampled about every 5 s.
  - The batch-16 control points to a footprint that scales with batch, not per-step growth. claude-297's seed-2 run (its first run's `gpu.csv`, two cards) held 5.3–7.9 GB for two hours, with peaks of 11.4 / 11.6 GB, and did not grow across its 32,768 steps. Four times that working set is about 22–32 GB, with peaks past the card.
  - The accumulation sheet's smoke must still show memory flat across steps before its full runs.
- **The job read peak memory but did not gate on it.** The sheet said "the smoke's peak memory is read from `gpu.csv`", and nothing stopped on it. A 98% peak in the smoke should have stopped the job before the full runs. The next sheet gates on it.
- **I-t passed and cut (b64L) to 16,384 steps.** The rate was 0.564 / 0.540 s a step, with a projected wall of 16,358 / 15,730 s, against 21,000. The gate worked as registered; it measured time only.

## The readings

None. No cell produced a checkpoint, so no reading in the sheet's Readings section was taken.

I-r, the control check, was run on the laptop before the pods. Re-pairing claude-297's (n3L) against D3's T2 with `clean=` D3's seen set reproduces d **+16,207 / −21,768**, spread **37,975**, exactly. (n3L)'s parting step is **4,096**, at a resolution of 1,024 steps. Seed 1 is read from its sealed `progress.jsonl`; seed 2 from the lost first run's logs, which match the rerun exactly at step 31,744. The parting step is the first progress row with |loss s1 − loss s2| > 0.005 (0.00533). The gap reaches 0.0296 by step 31,744. Both stand as the control's numbers for the next sheet.

## What changed

- `docs/harness/prep/one-model-end-to-end-d3-batch-2026-11-06.md`: the sheet, FIT WITH CHANGES round 1 applied.
- `docs/loop/tasks/one-model-end-to-end/claude-298/`: the ORDER and this REPORT.
- `docs/research/notebook/float-training.md`: one row, verdict **blocked**.
- No code changed. The trainer is pinned at `1f2bd76ce58503f25ef24e415dd193bf57086ddb`.

## Measurements

| seed | pod | $/h | smoke peak MiB | (b64S) | (b64L) | I-t rate s/step | spend |
|---|---|---:|---:|---|---|---:|---:|
| 1 | vast-59428271, RTX 5090, CZ | 1.000 | 32,108 | OOM after 84 s | OOM after 53 s | 0.564 | $0.39 |
| 2 | vast-59429269, RTX 5090, KR | 0.563 | 31,696 | OOM after 47 s | OOM after 48 s | 0.540 | $0.12 |

- **Total spend:** $0.51 of the $14 cap. Both pods were taken down by their watchers on `seal=0` (L-004).
- **Seals,** both read back and checked against their SHA256SUMS on the laptop:
  - `results/claude/one-model-end-to-end-d3-batch-2026-11-06/runs/s1` `d3-batch-s1.tar.gz` `f14e9ff579739699a77c7d237afb52470cbe32d90ac7380b471428fbf1b2aef0`;
  - `runs/s2` `d3-batch-s2.tar.gz` `77cbc331df65aee89a7f9fbe31175a09019a18e2598fac03166084ef0c31142d`.
- **The D67 pre-run review** passed with five changes, all applied before launch (#3407 6687704640). The four script sha256s on both pods matched the laptop's.

## Checks run at the head

- The seals' SHA256SUMS check OK on the laptop, at both seeds.
- I-r reproduces claude-297's (n3L) pair exactly (above).
- The scripts passed `bash -n` and the banned-literal scan before launch. Their sha256s matched on both pods.

## Skills used

None.

## Still open

- **The next unit: batch 64 by gradient accumulation,** as the sheet's Stop rule names.
  - The trainer (`crates/uor-r4-float-train/src/in_space_long.rs`) has no accumulation flag.
  - The unit adds one: it splits each step's 64 windows into k micro-batches of 16 and takes one AdamW step on their combined gradient.
    - The loss is a mean over the scored rows (`mean_all`), and the scored count can differ by micro-batch. So each micro-batch's loss is weighted by nᵢ/N, its share of the step's scored rows, not by 1/k. With that weighting the combined gradient equals the batch-64 one.
  - Its test must show that k = 4 micro-batches of 16 give the same step as one batch of 64, to float tolerance, on a small case.
  - Its sheet gates on the smoke's peak memory, below 90% of the card.
- **A memory gate for every sheet on this trainer:** the job stops before the full runs when the smoke's peak exceeds 90% of the card.

## Plain summary

The batch-64 test could not run. At batch 64 the model fills the whole graphics card, and every training run stopped on out-of-memory within a minute and a half, on both seeds, for $0.51. That says nothing yet about whether a larger batch helps. The next step adds gradient accumulation: the same batch of 64 is built from four batches of 16, which fit, and summed before each update. Then the same test runs again.

References #3407.
