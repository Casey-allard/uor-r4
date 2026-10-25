# REPORT: carrier-credit-hard-selection, claude-110 (results)

Lab: claude. Order: [ORDER.md](ORDER.md). Sheet: [`carrier-credit-hard-selection-2026-10-25.md`](../../../../harness/prep/carrier-credit-hard-selection-2026-10-25.md). Code: #3203 (claude-103).
Closes #2819.

## What did not work, first

- **Credit: tie or loss, so this configuration is negative (D22).**
  - On FRESH, ON leads both model-blind controls at both seeds by 103 to 131 hits out of 65,536. Those leads are ungraded: the registered noise floor is 1,989, so neither seed counts. Under the sheet's rule, ON does not beat either control.
  - The configuration closed is hard-selection carrier credit by summed integer hinge: 16,404 batches, the seeded-device codebook, and K = 8 Relation reads.
  - A negative closes a configuration, not the mechanism. Carrier credit stays PARKED.
- **Selection: tie or loss.** CONTROL-F trails OFF at both seeds (−41 and −2,034 on FRESH). #2831 and #2836 keep their schedule.
- **The noise floor rests on one arm.** OFF at seed 2001 scores 2,173 on FRESH and 493 on DEV. Every other arm, OFF at seed 1001 included, scores 139 to 274 and 32 to 54.
  - It is not a scoring fault: DEV and FRESH are scored independently and agree. A fault in that training run is not ruled out.
  - OFF accepts only 1,410 to 1,471 updates in 16,404 batches. Every carrier and control arm accepts 11,770 to 13,273.
  - Each seed starts from its own seeded codebook. One reading, untested, is that seed 2001's start was already good and OFF barely moved it, while the update-heavy schedules wore it down. No arm scored checkpoint 0 per seed.
- **ON's top-8 is below CONTROL-U's.** On FRESH it is 848 and 864, against 1,009 and 1,008, at both seeds, though ON's top-1 is higher.
  - This is ungraded, and far inside OFF's own top-8 seed spread (897 against 5,123).
  - It is not below CONTROL-F's at seed 1001 (848 against 795).
- **ON's choices concentrate.** The audit shows only 183 and 173 distinct tokens, and the most-chosen token (id 1 at both seeds) took about 40% of ON's vector batches.
  - One hypothesis, untested here, is that crediting by summed hinge favors frequent tokens.
  - The run did not measure the chosen tokens' frequency, or link the training-time choices to evaluation top-8.
  - A frequency-normalized credit is one next configuration that would test it.
- **Every learned arm loses to the count tables.**
  - On FRESH, the unigram table scores 5,711 and the next-word table 30,815. On DEV they score 1,226 and 6,300.
  - The best learned arm is OFF at seed 2001, with 2,173 on FRESH and 493 on DEV.
- **The run cost far more than the sheet's $6.** Pod B was a 2×5090 at $2.171/h for about 7.1 hours (about $15). Pod C was a 1×5090 at $0.563/h for about 8.1 hours (about $5). Pod A was a 2×5090 for about 5.5 hours; its rate was not recorded here.
  - Each arm's time per batch grew as it trained (#3203's REPORT).
  - The run went ahead under the rule on self-set budgets (owner log #2819 `6396426933`).

## What changed

Docs only (this item's scope holds no crate path since #3227):
- **[`grade.json`](../../../../labs/carrier-credit-hard-selection/grade.json):** the `carrier-arms grade` record over the eight arms.
- **The lab README:** the Results section.
- **[`docs/mechanisms/carrier-credit.md`](../../../../mechanisms/carrier-credit.md):** one row in "What has been tried".
- **`docs/research/notebook/m1-discrete-attention.md`:** this result's row. It also carries `m1-address-key-full-vector`'s row from #3230, because the installed loop guard refuses that item's own notebook page.
- **The item is set `done`,** and the roadmap is re-rendered. The render also corrects `main`'s stale status counts, left after #3230's merge combined two count edits.

## Measurements

The grader is the merged `carrier-arms grade` (#3203). It applies the registered rule: a beats b when a − b exceeds the seed noise and (a − b)²·N > 8·h·(N − h). The run outputs are in the HF store under `results/claude/carrier-credit-2026-10-25/`:
- `podA/`, 79 files;
- `podB/`, 31 files;
- `podC/`, 28 files.

Each folder was verified after download. The selector's CUDA controls passed on the GPU twice:
- 9 of 9 (c1–c9) in phase 1 on all three pods, at the run commit;
- all 10, c10 included, on pods B and C at merged `main`, run after their arms finished.

Pod A's `seed-*-seq` jobs end with EXIT 1 and `FAULT: arm run` by design: the job stopped there, because the CONTROL-U folders had moved to pod B (owner log #2819 `6396426933`). Its ON records are complete, at 16,404 batches each, under the same release as pods B and C.

| Arm (seed 1001 / seed 2001) | FRESH top-1 (of 65,536) | FRESH top-8 | DEV top-1 (of 14,287) | DEV top-8 | Accepted updates |
|---|---:|---:|---:|---:|---:|
| ON, `carrier_credit_v1` | 274 / 269 | 848 / 864 | 54 / 53 | 178 / 191 | 11,771 / 11,770 |
| CONTROL-U, `carrier_uniform_target_v1` | 171 / 166 | 1,009 / 1,008 | 32 / 37 | 221 / 220 | 13,215 / 13,135 |
| CONTROL-F, `carrier_uniform_row_v1` | 143 / 139 | 795 / 941 | 35 / 41 | 188 / 223 | 13,196 / 13,273 |
| OFF, `interleaved_read_every8_v1` | 184 / 2,173 | 897 / 5,123 | 37 / 493 | 201 / 1,063 | 1,471 / 1,410 |
| Unigram / next-word count tables | 5,711 / 30,815 | — | 1,226 / 6,300 | — | — |

- **The registered noise:** 1,989 on FRESH and 456 on DEV.
- **FRESH gaps:**

  | Gap | Seed 1001 | Seed 2001 |
  |---|---:|---:|
  | ON − CONTROL-U | 103 | 103 |
  | ON − CONTROL-F | 131 | 130 |
  | CONTROL-F − OFF | −41 | −2,034 |
  | ON − OFF | 90 | −1,904 |

  Without the noise term, the binomial condition alone holds for ON over both controls at both seeds. It is reported, not graded.
- **The selection audit** found ON's choices informative, so credit is graded and not "instrument":
  - zero-credit fallbacks: 0 at both seeds;
  - ON chose the same token as CONTROL-F on 1,025 and 982 of 14,354 vector batches (7.1% and 6.8%), and as CONTROL-U on 253 and 240;
  - distinct tokens: 183 and 173;
  - the top token's batches: 6,112 and 5,703.
- **Labels:** both are "none", so there is no DEV-only gain to call panel fit.

## Checks run at the head

- `loop report --check` and `loop pr-check --local`: PASS.
- **A fresh claims check** of this REPORT, the lab README, the tried row and both notebook rows: CHANGES, now fixed.
  - It re-verified all three pods' SHA256SUMS and every number in the eight records. It recomputed the decision and the audit from the records.
  - It found two wrong facts (the control counts, and a cost figure) and three overclaims ("beats" for ungraded leads; "narrows top-8"; frequency as a fact rather than a hypothesis). It also asked for pod A's by-design FAULT to be explained.

## Skills used

- holding-the-goal: the outlier was classified as a model property before it was read as a verdict (§4.1, §4.2), and the registered rule decided.

## Still open

- **A frequency-normalized credit** (credit per token, compared by cross-multiplication). This run's audit points there.
- **Scoring checkpoint 0 per seed** would test whether OFF at seed 2001 kept a good start or learned one.
- **The noise floor** is two OFF seeds. A next configuration should register more seeds, or a noise term that one outlier cannot set alone.

## Plain summary

I tested a rule that picks which word's vector to adjust next from the errors the model just made, against two blind rules and the old fixed order, at two seeds. The new rule led the blind rules by about a hundred correct answers out of 65,536. But one run of the old fixed order happened to score ten times higher than anything else, and the pre-registered test counts that spread as noise, so the result is a tie: the lead does not count. The new rule's top-eight guesses were also below one blind rule's. This version is closed. A version that weights rare words fairly is one next try.
