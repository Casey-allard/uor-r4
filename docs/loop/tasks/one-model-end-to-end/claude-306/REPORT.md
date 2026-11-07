# REPORT: one-model-end-to-end, claude-306

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Order: [ORDER.md](./ORDER.md). Sheet: [`one-model-end-to-end-d4-2026-11-07.md`](../../../../harness/prep/one-model-end-to-end-d4-2026-11-07.md) (FIT WITH CHANGES, rounds 1 and 2; D67 GO WITH FIXES, all applied).

## What did not work, first

- **The bar is not met on D4 at these seeds** (configuration only, §4.1; this closes nothing, L-007). (b64L) on TRAIN D4 is not distinguishable from D4's T2. The pair is `mixed`, at d +3,306 / +90,992 with a band of 87,686. Seed 2 is ahead and seed 1 is inside.
  - **On unseen rows** it is `inside`: −5,737 / +65,891, band 71,628.
  - **By sub-panel** (seed 1 / seed 2, band), on all rows:
    - oasst2 +2,573 / +15,011 (12,438), `mixed`;
    - wiki-00000 −6,024 / +17,948 (23,972), `inside`;
    - wiki-00020 +6,471 / +32,403 (25,932), `mixed`;
    - wiki-00040 +286 / +25,630 (25,344), `mixed`.
  - **On unseen rows by sub-panel:**
    - oasst2 −540 / +9,801 (10,341), `inside`;
    - wiki-00000 −6,832 / +13,837 (20,669), `inside`;
    - wiki-00020 +2,638 / +22,493 (19,855), `mixed`;
    - wiki-00040 −1,003 / +19,760 (20,763), `inside`.
- **The data lever does not move.** D4 against claude-301's (b64L-D3), at equal positions (201.3M) and equal steps:
  - on unseen rows, the deciding reading, it is `inside`: −58,807 / +4,794, band 63,601;
  - on all rows it is `inside` as well: −71,949 / +5,185, band 77,134;
  - oasst2 alone is `mixed` both ways: unseen −10,005 / −839 (band 9,166), all rows −12,106 / −1,183 (band 10,923). Seed 1 is `behind` there on its own, and seed 2 is inside;
  - each wiki sub-panel is `inside`, on unseen rows and on all rows.
  - No sub-panel reads `behind`, so the outcome map's row 2 does not apply.
- **The seeds disagree, and seed 1 is the run that falls short.**
  - Seed 1's no-memory top-1 is 259,749 (17.8%), against seed 2's 347,435 (23.8%) and D3's 331,698 / 342,250.
  - On DEV (`dev_nearest`) both D4 seeds dip after step 1,024, as claude-301's D3 seed 1 did:
    - D3 seed 1: 2,745 at step 1,024, 1,022 at 3,072, back to 3,001 by 10,240, and 3,185 at 16,384;
    - D4 seed 2: 1,531, then 1,011, then 2,896 by 10,240, and 2,905 at the end;
    - D4 seed 1: 2,737, falling to 671 at 4,096. It was back to only 1,154 at 10,240 and **was still climbing at the last step**: 1,763, 1,788, 1,938 and 1,990 over the last four evals.
  - This is a training-dynamics reading of the configuration, not an instrument fault. Seed 1's DEV and panel agree (its digests match, its i5 reads `same` on every sub-panel), and its gates passed. It is recorded, and not used to rescue the bar.
- **One slip on the laptop, no harm.** Seed 2's first watcher was started with a mistyped start epoch. It stopped on time at once (exit 6), with the pod kept and the lease renewed. It was re-armed from the recorded epoch file within a minute, and recorded in `local/claude/FRICTION.md`.

## The readings

| arm | seed | top-1 of 1,460,697 | vs D4's T2 d (band) | vs unigram d | oasst2 top-1 of 184,542 |
|---|---|---:|---|---|---:|
| (b64L-D3), claude-301 | 1 / 2 | 331,698 / 342,250 | vs D3's T2 +75,038 / +85,590 (10,552) ahead | +294,277 / +304,829 | 44,210 / — |
| (b64L-D4) | 1 | 259,749 (17.8%) | +3,306 (87,686) inside | +222,328 ahead | 32,104 (17.4%) |
| (b64L-D4) | 2 | 347,435 (23.8%) | +90,992 ahead | +310,014 ahead | 44,542 (24.1%) |

- D4's T2 gets 256,443 rows (17.6%); D3's got 256,660. The unigram gets 37,421 on both.
- **Bar verdicts:** against D4's T2, `mixed` on all rows and `inside` on unseen rows. Against the unigram, `ahead` on all rows, on unseen rows and on every sub-panel.
- **I-r passed:** `I-r: PASS, 60 fields equal`. That re-paired claude-301's (b64L-D3) grades against D3's T2 with the `scale-probe` build that grades D4 (`31610a00…`). Its `inputs_grade_sha256` equals the reference's: a1 `e17a1354…`, a2 `3170c779…`, table t2, clean `53000ba1…`.
- **I-s passed:** the lever's `clean_seen_sha256` is D4's `seen.json` (`d5443f95…`), and that file's excluded set is `511b0289…`.
- **The loss at the last step,** within the arm only: 0.9447 / 0.9171.
- **The checks behind the numbers:**
  - every grade carries the panel's row identity `5f88a5b3…`;
  - each seed's i5 reads `same` on every sub-panel;
  - each seed's DEV memory and no-memory digests agree;
  - I-3 is `ok` at both seeds;
  - every train and score exit is 0, and both chains end `job=0 seal=0`.

## What this unlocks (sheet, outcome map)

The bar is not distinguishable from T2, and the unseen lever reads `inside`. That is **row 6**:

> **Not met on D4 at these seeds** (configuration only, §4.1). The trunk keeps D3. Next: the tokens lever on D3, since claude-301 met the bar there with room.

- The trunk keeps D3. (b64L-D3) stays the trunk's recipe, "met; unseen ahead".
- **The next unit is the tokens lever on D3:** (b64L) at 32,768 steps, under its own sheet and FIT.
- That sheet should read the DEV curve as well as the endpoint. D4 seed 1's curve was still rising at 16,384 steps. It is the same reading at the trunk's own budget on D4 that the tokens lever asks of D3, and the next sheet may name it in its own outcome map.

## What changed

- `docs/harness/prep/one-model-end-to-end-d4-2026-11-07.md`: the sheet, with both FIT rounds and the D67 fixes applied.
- `docs/loop/tasks/one-model-end-to-end/claude-306/`: the ORDER and this REPORT.
- `docs/research/notebook/float-training.md`: one row, verdict **inconclusive**. The bar reads `mixed`, which the sheet's Bar records as "not distinguishable at these seeds", never as a negative (L-007). Row 6 picks the next unit; it closes nothing.
- No code changed. The trainer is pinned at `ff5156bcc9539aa5bccd0319df3d96ee5dd5a71f`.

## Measurements

| seed | pod | $/h | result | I-t s/step | training s | peak MiB (I-m) | spend |
|---|---|---:|---|---:|---:|---:|---:|
| 1 | vast-59529348, RTX 5090 | 0.563 | trained, scored, sealed | 0.632 | 10,409 | 26,832 of 32,607 | $1.81 |
| 2 | vast-59529378, RTX 5090 | 0.563 | trained, scored, sealed | 0.636 | 10,489 | 26,928 of 32,607 | $1.81 |

- **Total spend:** $3.62, against the sheet's $11.50. Each pod was taken down by its watcher on `seal=0` (L-004).
- **I-t projections:** 12,967 / 13,054 s, under 16,000 s, so no watcher extension was needed.
- **I-f:** 26,768 then 26,832 MiB, and 26,608 then 26,928 MiB.
- **Seals,** each read back on the pod and checked against its SHA256SUMS on the laptop:
  - `results/claude/one-model-end-to-end-d4-train-2026-11-07/runs/s1` `d4-train-s1.tar.gz` `dfa0b6e41d7ecfc0f21c93e55ea66b5074a0fea57d094640bb2552560c3a096a`;
  - `runs/s2` `d4-train-s2.tar.gz` `aa029cefe45bcd7357a0d40e7478ac27932f3c2e8cc54fc0913a82017775524f`.
- **Notes on #3407:** the FIT rounds, the D67 review (6702930341) and the launch (6702990617).

## Checks run at the head

- Both seals' SHA256SUMS check OK on the laptop.
- I-r: PASS, 60 fields equal, with equal input shas. I-s: PASS.
- Every `scale-probe grade` and `pair` exited 0.
- Before launch:
  - `gt/test-gates.sh`: 22 ok, 0 FAIL;
  - `gt/mutate.sh`: 7 of 7 caught;
  - `gt/test-seal.sh`: 7 ok, with 2 of 2 mutations caught;
  - `bash -n` ok on every script.

## Skills used

None.

## Still open

- **The next unit:** the tokens lever on D3, (b64L) at 32,768 steps, under its own sheet and FIT.
- **D4 at a longer budget:** whether D4 needs more steps than D3 at this recipe is not read here. Seed 1's DEV curve was still rising. A sheet that wants that reading trains D4 at the tokens lever's budget.
- **The seed spread:** at 87,686, the bar's band is about 8× claude-301's 10,552 on D3. Two seeds do not say whether D4 widens the spread or seed 1 is an outlier.

## Plain summary

The model was trained on 2.65 times more human text (D4) with the same recipe and the same number of steps. It did not get better. One of the two training runs did about as well as the smaller-text model (23.8% next-token accuracy). The other did much worse (17.8%), about level with the simple word-pair table (17.6%). That run's accuracy was still rising when training ended, so it seems to need longer to learn from the bigger text. Because the two runs disagree, the result counts as not distinguishable from the table. The project keeps the smaller training text. The next step trains on it for twice as many steps. The unit cost $3.62.

References #3407.
