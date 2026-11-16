# REPORT: one-model-end-to-end, claude-397 (can C2′'s target be found from the stream at all?)

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Order: [ORDER.md](ORDER.md). PR #4022.

**Reading:** outcome 0 passes. Outcome 1 does not hold. Outcome 3 holds at ext-s1 b1, and the other three cells are inconclusive on R2. Outcome 4 reads "control fault" in all four cells.

In plain terms:
- **The stream does hold enough** for a reader that knows the task's rule to guess the other fact about 43% of the time. That is well above the bar's threshold of about 0.11, so C2′ was not impossible.
- **The native rule learned only the coarse part of the target.** It moved 94–99% of its picks onto the window's 8 fact rows, but spread them close to evenly over those 8. So it never learned which fact.
- **That spread fully explains the C2′ `chose_target` rise** that claude-395 read as a broken bar. It is not a drift toward the salient asked fact.

## What did not work, first

- **Outcome 1 does not hold, so this is not "the control asks more than the stream holds".** The uniform-guess ceiling `U` over the consistent facts, against the bar's threshold:

  | State | `U` b1 / b2 | Threshold (C0 + 0.10) b1 / b2 |
  |---|---|---|
  | ext-s1 | 0.449 / 0.429 | 0.106 / 0.106 |
  | ext-s2 | 0.463 / 0.421 | 0.120 / 0.116 |

  `U` is a reference, not a proven Bayes rate (ORDER, Definitions). It is also conservative, for two reasons (exact-head review):
  - `C_i` ignores whether the remaining facts can still be paired into episodes. Each earlier slot's other fact must lie in that slot's bucket, and the bucket multiset is {0, 0, 1, 1}.
  - So a full reader can sometimes narrow `C_i` further, and some rows counted as `K ≥ 2` are really `K = 1`.

  A smaller set only raises the reachable rate, so outcome 1's "does not hold" stands. R2's split between `K = 1` and `K ≥ 2` rows is slightly blurred by the same effect. Even so, C2′'s `chose_other` half can be met in principle.
- **The rule does not follow the identifiable part.** On rows where the other fact is the only consistent one (`K = 1`), C2′ hits it *less* often than on ambiguous rows:

  | Cell | C2′ hits, `K = 1` rows | C2′ hits, `K ≥ 2` rows | A's hits, `K = 1` rows |
  |---|---|---|---|
  | ext-s1 b1 | 1 / 35 | 51 / 477 | 0 / 35 |
  | ext-s2 b1 | 3 / 42 | 52 / 470 | 0 / 42 |

  At ext-s1 b1 this is outcome 3: 0.029 is at most 0.05 above A's rate of 0. At ext-s2 b1 the rate of 0.071 falls between outcome 2's and outcome 3's thresholds, so that cell is inconclusive. The b2 buckets have 12 and 8 `K = 1` rows, under the registered 30, so they are inconclusive (outcome 5).
- **Unique targets are rare.** `K = 1` on 1.6–8.2% of held-out queries and 1.6–7.5% of TRAIN queries. Most rows have `K` = 2 or 3. So bar 4b's `chose_other` half mostly rests on rows a reader can only guess.

## What changed

- `recall_other_candidates` (in `in_space_long.rs`) gives, at each two-class query, the set `C_i` of facts that are consistent with being its other fact, given the stream before the scored row:
  - not the asked fact;
  - not asked in an earlier slot;
  - at a distance inside the asked fact's bucket.

  `OtherFactCandidates::class` sorts a pick into asked, other, candidate, fact or row.
- `recall_other_ceiling` and the bin `recall-other-ceiling` rebuild the probe's held-out windows (or the TRAIN ones) from a seed. They check the windows against a run's target files and class saved picks, all from integer counts (a 3-decimal string is formatted from integers).
- The ORDER's done-when 1 is corrected at build time, before any count. The generator's `q − 2` clip never removes a fact, because every fact sits at `p ≥ 2`. The negative gate is therefore on the earlier-slot exclusion.

## Measurements

**Outcome 0:** all 49,152 rows at each ext state equal claude-395's `held-*.q31.ask.u32`, `.other.u32` and `.bucket.u8`. `o ∈ C_i` holds at every query (the function refuses otherwise).

**R1, `K` histograms** (`K` = 1 to 6; no row has `K` = 0, 7 or 8):

| Split | b1 | b2 |
|---|---|---|
| held ext-s1 | 35, 250, 165, 53, 8, 1 | 12, 253, 232, 14, 1, 0 |
| held ext-s2 | 42, 261, 154, 44, 11, 0 | 8, 248, 236, 20, 0, 0 |
| TRAIN ext-s1 (4,096 a bucket) | 308, 1953, 1348, 403, 80, 4 | 81, 2059, 1819, 134, 3, 0 |
| TRAIN ext-s2 | 296, 1969, 1322, 440, 66, 3 | 66, 2092, 1795, 140, 3, 0 |

TRAIN `U` is 0.452/0.427 (ext-s1) and 0.450/0.426 (ext-s2). claude-395's C2′ train `chose_other`, about 0.12, is far below it.

**R3, where the picks land** (held-out, 512 a bucket; asked / other / candidate / other fact / non-fact row):

| Picks | ext-s1 b1 | ext-s1 b2 | ext-s2 b1 | ext-s2 b2 |
|---|---|---|---|---|
| C2′ | 73 / 52 / 88 / 290 / 9 | 67 / 57 / 85 / 297 / 6 | 70 / 55 / 64 / 292 / 31 | 69 / 71 / 92 / 249 / 31 |
| C2 (random) | 7 / 2 / 2 / 21 / 480 | 5 / 3 / 3 / 17 / 484 | 1 / 0 / 0 / 0 / 511 | 0 / 0 / 0 / 0 / 512 |
| A (asked, chosen shift) | 500 / 1 / 2 / 9 / 0 | 498 / 1 / 5 / 7 / 1 | 508 / 0 / 0 / 4 / 0 | 510 / 1 / 1 / 0 / 0 |
| C1 (float pick) | 504 / 2 / 2 / 3 / 1 | 496 / 1 / 2 / 12 / 1 | 496 / 2 / 2 / 11 / 1 | 494 / 1 / 5 / 10 / 2 |

- **Outcome 4.** C2′'s asked count `A` against `2P`, where `P` is the per-fact rate on the consistent set: 73 ≤ 108.8, 67 ≤ 118.1, 70 ≤ 96.9, 69 ≤ 130.7. `A ≤ 2P` in every cell.
- **Descriptive, not registered.** C2′ puts 503, 506, 481 and 481 picks on fact rows. An even spread over 8 facts gives the asked fact 62.9, 63.3, 60.1 and 60.1. The observed 73, 67, 70 and 69 are 1.1 to 1.2 times that.

**Classification (§4.1, §4.2).**
- **C2′'s `chose_target` half.** claude-395's "C2′ `chose_target` rose 0.12" is what any rule that finds fact rows, without the hidden pairing, produces. The control cannot distinguish "follows its targets" from "drifts to the salient fact" there. That is an instrument (control-design) fault in that half of bar 4b.
- **C2′'s `chose_other` half.** It is reachable for a reader that applies the rule, but this read did not learn that rule. Whether one fixed set of L1-nearest gates *can* express "the asked fact's bucket, not asked before" is not settled. That is the gate-family question (outcome 3's next step).
- **claude-395's verdict stands under its registered bar.** No bar is changed here.

Results are in the HF store at `caseyallard/uor-r4-store:results/claude/one-model-end-to-end-other-fact-ceiling-397-2026-11-16/`: four JSON files with `SHA256SUMS`. held-ext-s1 is `ee8b2e12…`, held-ext-s2 `3ed32f2c…`, train-ext-s1 `218e2db6…` and train-ext-s2 `4db20b7a…`. The bin `recall-other-ceiling`, built at `72360111e`, is `8f019acc…`.

## Checks run at the head

- `uor-r4-float-train` lib: `recall_other_candidates_follow_the_stream` and `recall_other_ceiling_reads_real_windows` pass (2 passed). With the earlier-slot exclusion set aside, the first fails, because slots 2 and 3 gain facts 100 and 20. It passes again once the exclusion is restored.
- `recall-other-ceiling` bin: `the_flags_parse_and_refuse` passes.
- The real-windows test checks that a changed target word is refused, and that picks copying the other fact class as `other` on every row.
- `git diff --check` is clean. Integer counts only: the one division is integer rounding into a string.
- Laptop only: a host count over the task generator and saved picks. No model computes and no device is used (§3.5).

## Skills used

- `solution-integrity`: the readings and the outcome map were registered in the ORDER before any count. The one correction (done-when 1) came at build time, before any count, and is stated.
- `scope-guard`: one function, one bin and the docs. No bar or status changed.

## Still open

**The registered unlock** is outcome 3, at ext-s1 b1: the gate-family reachability probe, item 2 below. Item 1 is a new proposal outside the map. The map grants a card only for "1 with 2 or 4", and outcome 1 does not hold. It rests on outcome 4's control-fault reading of the `chose_target` half alone.

1. **A decision card for bar 4b's `chose_target` half** (§5.1; the top session approves it after a decision review). The half cannot discriminate on this task. The card should propose a control whose target the stream determines and a content match could find, for example the fact asked in the previous slot, whose relation is in the stream as that slot's `Q(K)`. Its GPU arm needs a sheet addendum and a non-author FIT (claude-398).
2. **The gate-family question.** Can one fixed set of Q31 gates with an L1-nearest pick express a positional rule ("inside the asked fact's bucket")? A host probe of the best reachable `chose_other` on the `K = 1` rows would answer it.
3. Only C2′'s held-out picks were saved, not its TRAIN picks, so R2 and R3 are held-out only.

## Plain summary

claude-395's other-fact control asked the read to find "the other fact of the same episode". The task hides which facts pair up, so on most questions two or three facts fit equally well. A reader that knows the rule could still guess right about 43% of the time.
- The trained read learned only to look at fact rows. It picked among the 8 facts almost evenly, which is why its pick of the asked fact "rose", from 2% to about 14%.
- So that part of the safeguard can't tell a rule that follows its targets from one that drifts.
- The other part, actually finding the other fact, was reachable in principle, and the read did not learn it.
- The next steps are a better-designed control and a test of what fixed gates can express.
