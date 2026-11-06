# REPORT: one-model-end-to-end, claude-301

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Order: [ORDER.md](./ORDER.md). Sheet: [`one-model-end-to-end-d3-accum-2026-11-06.md`](../../../../harness/prep/one-model-end-to-end-d3-accum-2026-11-06.md) (FIT WITH CHANGES, rounds 1 and 2).

## What did not work, first

- **Run 1 stopped at I-e, the equivalence check: underpowered, an instrument fault.** Both pods stopped before any arm trained, for $0.38.
  - Accumulation itself matched: four micro-batches against the whole batch differed by 1.3e-8, 8.4e-9, 8.9e-8 and 4.8e-9 relative at steps 1 to 4.
  - The control failed its bar. Turning the read off (`--hamming-read 0`) moved the loss by 0, 7.0e-7, 1.0e-5 and 6.8e-5, against a bar written for step 1.
  - The cause: the read's output weight starts at zero, so the read cannot move step 1. I-e was amended to a tolerance of 1e-6 and a control at step 4 of at least 1e-5. That amendment passed a second FIT round, which made each pod run I-e at its own seed so that seed 2 tests the gate on rows it was not set on.
  - Run 1 is sealed at `runs/`, and run 2 at `runs-2/`.
- **(b64S) is not distinguishable from D3's T2.** Its pair is `inside` at d −19,092 / +26,078, with a band of 45,170. Its sub-panels are inside, except wiki-00000, which is mixed. It does not meet the bar. This closes nothing (L-007).
- **The spread is not decided at two seeds.** "Batch narrows the spread" needs both batch-64 arms' spreads under 18,988 and (b64L)'s parting step later than (n3L)'s.
  - (b64L)'s spread is **10,552**, but (b64S)'s is **45,170**.
  - (b64L) parts at step **3,072**, earlier in steps than (n3L)'s **4,096**. It is a thin call: the gap there is 0.00539 against the 0.005 line ((b64S) 0.00735), one progress row from not parting. In windows seen, 3,072 steps at batch 64 is 196,608, against (n3L)'s 65,536 at 4,096. The rule compares steps.
  - It is not "does not narrow" either: that needs (b64L)'s spread at least 37,975.
  - (b64S) ran at both seeds, so the rule reads both arms (sheet, Readings). The question "is the seed spread gradient noise" stays open.
- **I-t cut (b64L) to 16,384 steps, so the equal-steps reading is lost** (sheet, outcome map). (b64L) makes half (n3L)'s updates at four times its batch, so it sees 201.3M positions to (n3L)'s 100.7M.
  - The win below is therefore a larger batch **with twice the tokens**, not batch alone.
  - At equal tokens, the batch lever is `inside`: (b64S) against (n3L), d −35,299 / +47,846, band 83,145.
  - The tokens lever at batch 64 is `ahead`: (b64L) against (b64S), d +94,130 / +59,512.
- **I-e's control passed on seed 2 with a thin margin.** The read moved step 4 by 1.85e-5 against the 1e-5 bar, 1.85× the bar. On seed 1 the margin was 6.8×, a replay of run 1's rows to the last digit. Accumulation stayed within 1.3e-7 on both seeds.
- **Seed 2's rate sat at the edge of the drop line.** I-t measured 0.698 s a step, against the 0.70 s at which (b64S) would have been dropped.
- **Two slips on the laptop, both repaired before any reading:**
  - Extracting both seals into one folder overwrote seed 1's shared `gpu.csv`, `mem.log` and `job.log` with seed 2's. The per-cell files have distinct names and were not touched. Seed 1's shared files were extracted again into their own folder. Peak memory is read from `mem.log` (the job's 2 s sampler) inside each cell's `.window`, not from `gpu.csv` as claude-298's Readings name. `mem.log` is the file I-m and I-f gate on.
  - The `scale-probe` binary had been removed with its target directory under the disk rule. It was rebuilt from `docs/labs/scale-probe/instrument` at this branch, whose instrument is unchanged from the pin. I-r below reproduces claude-297's pair exactly with it.

## The readings

**The one-track bar is met by (b64L): met; unseen ahead.** Its no-memory top-1 is ahead of D3's T2 and of D3's unigram beyond the paired band, at both seeds. That holds on the whole panel and on every sub-panel, the human oasst2 panel included, on all rows and on unseen rows.

| arm | seed | top-1 of 1,460,697 | vs T2 d (band) | vs unigram d | T2's share | parting step | peak MiB |
|---|---|---:|---|---|---:|---:|---:|
| (n3L), claude-297 | 1 / 2 | 272,867 / 234,892 | +16,207 / −21,768 (37,975) inside | — | 47.3% / 36.0% | 4,096 | — |
| (b64L) 16,384 steps | 1 | 331,698 (22.7%) | +75,038 (10,552) ahead | +294,277 ahead | 66.9% | 3,072 | 27,179 |
| (b64L) | 2 | 342,250 (23.4%) | +85,590 ahead | +304,829 ahead | 67.1% | | 27,276 |
| (b64S) 8,192 steps | 1 | 237,568 (16.3%) | −19,092 (45,170) inside | +200,147 ahead | 55.7% | 3,072 | 27,083 |
| (b64S) | 2 | 282,738 (19.4%) | +26,078 | +245,317 ahead | 56.4% | | 27,116 |

D3's T2 gets 256,660 rows (17.6%), and the unigram 37,421.

- **(b64L) against T2, by sub-panel** (seed 1 / seed 2, band), every one `ahead`:
  - oasst2 +13,342 / +14,857 (1,515);
  - wiki-00000 +13,357 / +15,502 (2,145);
  - wiki-00020 +27,736 / +31,335 (3,599);
  - wiki-00040 +20,603 / +23,896 (3,293).
- **On oasst2,** the human panel, (b64L) seed 1 gets 44,210 of 184,542 rows (24.0%), against T2's 30,868 (16.7%).
- **(b64L) on unseen rows:** against T2, d +56,353 / +64,841 (band 8,488), ahead on every sub-panel. Against the unigram, ahead on every sub-panel.
- **The levers** (pair, no table, clean = D3's seen set; not a bar):
  - batch at equal steps, (b64L) vs (n3L): `ahead`, +58,831 / +107,358 (48,527);
  - batch at equal tokens, (b64S) vs (n3L): `inside`, −35,299 / +47,846 (83,145);
  - tokens at batch 64, (b64L) vs (b64S): `ahead`, +94,130 / +59,512 (34,618).
- **The loss at the last step:**
  - (b64L) 0.9331 / 0.9205;
  - (b64S) 0.9592 / 0.9694.
  Losses are read within an arm only; arms are compared on the parting step.
- **The checks behind the numbers:**
  - Every grade carries the panel's row identity `5f88a5b3…`.
  - Every cell's i5 reads `same` on each sub-panel.
  - Each cell's DEV memory and no-memory digests agree.
  - Every train and score exit is 0.
  - I-r re-pairs claude-297's (n3L) against D3's T2 to d +16,207 / −21,768, exactly.

## What this unlocks (sheet, outcome map)

(b64L)'s recipe becomes the trunk's human-text recipe, carried with its qualifiers: "met; unseen ahead", at **16,384 steps with batch 64 by accumulation, 201.3M positions**. That recipe is claude-298's with `--batch 64 --micro-batch 16` at 16,384 steps.

The sheet's next unit is **width 1,024 on this recipe**, with unseen rows read again there, plus the served-form check (§3.7). Casey parked serving and speed work for claude. The served-form check is a part of the bar, not serving work, so the next sheet's decision review records that it is in scope.

The spread question, and whether the gain comes from batch or from tokens, are not this bar's to settle. The equal-tokens lever reads `inside`, so a larger batch at the same tokens is not shown to help. More tokens at batch 64 is shown to help.

## What changed

- `docs/harness/prep/one-model-end-to-end-d3-accum-2026-11-06.md`: the sheet, with both FIT rounds applied.
- `docs/loop/tasks/one-model-end-to-end/claude-301/`: the ORDER and this REPORT.
- `docs/research/notebook/float-training.md`: one row, verdict **positive**.
- No code changed. The trainer is pinned at `ff5156bcc9539aa5bccd0319df3d96ee5dd5a71f`.

## Measurements

| run | seed | pod | $/h | result | I-t s/step | spend |
|---|---|---|---:|---|---:|---:|
| 1 | 1 | vast-59438382, RTX 5090 | 1.000 | stopped at I-e, underpowered | — | $0.20 |
| 1 | 2 | vast-59438379, RTX 5090 | 0.908 | stopped at I-e, underpowered | — | $0.18 |
| 2 | 1 | vast-59440058, RTX 5090 | 0.986 | both arms trained and scored | 0.633 | $4.84 |
| 2 | 2 | vast-59440060, RTX 5090 | 1.00 | both arms trained and scored | 0.698 | $5.48 |

- **Total spend:** $10.70 of the $14 cap. Every pod was taken down by its watcher on `seal=0` (L-004).
- **Training time:** (b64L) 10,924 / 12,008 s; (b64S) 5,444 / 6,035 s.
- **Run 2's gates,** seed 1 / seed 2:
  - I-e worst step 8.9e-8 / 1.3e-7;
  - I-e control 6.8e-5 / 1.85e-5;
  - I-m (b64L) 27,083 / 26,988 MiB and (b64S) 26,923 / 26,924 MiB, of 32,607;
  - I-f 26,859 then 27,083, and 26,988 then 26,924 MiB.
- **Seals,** each read back and checked against its SHA256SUMS on the laptop:
  - `results/claude/one-model-end-to-end-d3-accum-2026-11-06/runs-2/s1` `d3-accum-s1.tar.gz` `f7852f2540e58c17b2311d39fb47f8e53471e2cda8d6136f81b034cdf6bf5bba`;
  - `runs-2/s2` `d3-accum-s2.tar.gz` `1f24f03a4b638f7ad6fafd184f980bca324de758085a62e5b5d0dfb3a4f6ab32`;
  - run 1: `runs/s1` `c8be4b08…`, `runs/s2` `4297401c…`.
- **Notes on #3407:**
  - the D67 pre-run review and the FIT rounds;
  - run 1 and the amendment (6690228466);
  - run 2's gates (6690663761).

## Checks run at the head

- The seals' SHA256SUMS check OK on the laptop, all four.
- I-r reproduces claude-297's (n3L) pair exactly.
- Every `scale-probe grade` and `pair` exited 0.
- The I-e and memory gates' laptop test (`gt/test-gates.sh`) passed before launch, including run 1's real rows.

## Skills used

None.

## Still open

- **The next unit:** width 1,024 on (b64L)'s recipe, with unseen rows read again and the served-form check (§3.7), under its own sheet and FIT.
- **The spread:** "batch narrows the spread" is undecided at two seeds. The next sheet reads parting steps again at width 1,024.
- **Batch or tokens:** the equal-tokens lever is inside. A unit that wants to tell them apart would run batch 16 at 201.3M positions (65,536 steps).

## Plain summary

The model now beats both simple baselines it was measured against, on held-out human and Wikipedia text, at both seeds, by more than the run-to-run noise. The baselines are a table of the training text's word pairs (T2) and its single most common words (the unigram). Training with an effective batch of 64, built from four batches of 16 so it fits on the card, for 16,384 steps, gets 22.7–23.4% next-token accuracy, against the word-pair table's 17.6%. It also wins on the text the training set never contained. The shorter run at the same batch does not beat the table, so the extra training text matters. Whether the larger batch alone also helps is not shown. The whole unit cost $10.70. The next step trains the same recipe four times wider.

References #3407.
