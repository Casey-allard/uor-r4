# REPORT: one-model-end-to-end, claude-374 (pair rows at full weight, rare pairs raised by 1/R)

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Order: [ORDER.md](ORDER.md). Sheet: [one-model-end-to-end-boost-pair-rows-2026-11-14.md](../../../../harness/prep/one-model-end-to-end-boost-pair-rows-2026-11-14.md). PR #3965. Follows claude-373 (PR #3964).

The order asked for claude-370's pair rows to be weighed by 1 + 1/R, so frame pairs keep their full weight and the rarest pairs count at most twice as much. It also asked for four models, trained at two seeds with and without the weight, and scored against bars fixed before any model was trained. **The bar reads "Does not add."** The fact and number rows that end with the answer word go from 85 to 78 of 188 at seed 2 and from 85 to 79 at seed 3. That is a mean gain of −6.5, below the "does not add" line of 4. The exact-reply guard holds (4 and −1 of 2,218, limit 33), so sparing the frame pairs did keep the copying gain that claude-373 lost. This closes one configuration: 1 + 1/R, R capped at 4,096, one bag. It does not close weighing pair rows. It is a float reading on panels, and nothing here is a milestone claim.

## What did not work, first

1. **The weight did not reach the facts; they fell a little at both seeds.** Fact and number rows that ended with the answer word, P14 + P23, of 188: `pair` 85 and 85, `boost` 78 and 79. Read as exact replies, the same rows give 44 and 47 for `pair` and 40 and 41 for `boost`. So the drop does not come from the answer-word reading.
2. **Most of the loss is in two kinds** (answer-word rows, both seeds summed, `pair` then `boost`). Capital: 11 to 5 (of 68). Opposite: 8 to 4 (of 12). Add: 3 to 1. Sound gains (21 to 24), and bigger gains by 1 (25 to 26). Colour, legs and day after each move by 2 or less. No kind gains by more than 3.
3. **Capital is not answered by either arm.** Exact capital stays at 0 and 2 for `pair` and 0 and 1 for `boost`. The answer-word count for capital includes replies that only repeat the country. Those count because the reference also ends on it ("Oslo is the capital of Norway."). Replies, P14, seed 2:
   - "Which city is the capital of Norway?". `pair`: "The capital of Norway is the capital of Norway." `boost`: "The capital of Norway is the capital of Norway is the capital of Norway is the c…".
   - "What's the capital city of Kenya?". `pair`: "The capital of Kenya is is is is …". `boost`: "The capital of Kenya is is."
   - "What's the capital city of Spain?". Both: "The capital of Spain is hard."

   So most of the capital "loss" is `boost` looping past the country name where `pair` stopped on it. Neither arm knows any capital. The answer-word bar counted those stops as wins, and the order had fixed it that way.
4. **Exact replies churn without a direction.** P14 + P23, `boost` is right where `pair` is wrong on 33 rows (seed 2) and 33 rows (seed 3). It is wrong where `pair` is right on 37 and 32. Net exact: −4 and +1.
5. **Reading: a 1/R bonus raises almost every row, not only the informative ones.** Census R has p50 6, so the median pair gets 1 + 1/6 ≈ 1.17×, and every pair with R = 1 gets 2×. Pairs seen with one reply include pairs from single conversations, and those now count twice. Nothing in R marks the content pairs that should carry a fact ("capital, Kenya") as rarer than those. This is a reading of items 1–4, not a separate measurement; the share of R = 1 pairs that are noise was not counted.
6. **No instrument slip this time.** claude-373's two slips (a shared seal path and a relative reader OUT) are fixed in the kit: the lane is in every sealed path, and the reader makes GOT and OUT absolute. All four score stages read I-8 PASS. The `pair` control reproduces claude-373's `pair` exactly: exact 519 and 520, fact rows 85 and 85.
7. **Not run.** No serving or export (the integer export refuses pair rows, as in claude-370). No third seed. No arm, seed or step count was added after reading.

## What changed

- **`crates/uor-r4-float-train/src/pair_rows.rs`:** a `boost` mark on the saved weights. `with_boost_weights` turns the stored 1/R into a forward weight of 1 + 1/R: `add_bags` uses `affine(1, 1)` on the weights. `weight_kind()` returns `none`, `replies` or `boost`, and it is saved and loaded with the rows. Tests cover these:
  - the bag is the boosted sum: a pair seen with one reply adds 2× its row (`a_boosted_bag_is_the_sum_of_one_plus_inverse_count_rows`);
  - a cached decode step equals the whole window, with R = 8 weighing 1.125 (`a_boosted_cached_step_equals_the_whole_window`);
  - save and load keep the `boost` mark, and a mark without weights is refused (`boost_round_trips_and_a_mark_without_weights_is_refused`);
  - `replies` rows write no mark.

  No test compares the unflagged path bit for bit with the old output. That path's code is unchanged in the diff.
- **`in_space_long.rs`:** `pair_weight_boost` and the kind prints.
- **`bin/train-in-space.rs`:** `--pair-weight replies|boost`. Training prints `[pair] weight kind: boost`; scoring prints `[pair] loaded weight kind: none|replies|boost` (I-8).
- No panel, recipe, bar or saved model of another unit is changed.

## Measurements

Every model used claude-358's `p` recipe, 16,384 steps and `--pair-rows 2`; `boost` adds `--pair-weight boost`. All four ran on RTX 5090 pods, built from `4aeef3c0957234a1e43f5a4f1fb4f8bc61b3efef`:
- lane a: `vast-60655091`, seed 2;
- lane b: `vast-60655525`, seed 3.

They were scored in float by the trainer's own pass on P14 (1,108) and P23 (1,110).

| model | exact P14 | exact P23 | exact of 2,218 | did not end | fact rows ended with the answer word, of 188 | fact rows exact |
|---|---|---|---|---|---|---|
| pair-s2 | 258 | 261 | 519 | 187 | 85 | 44 |
| boost-s2 | 258 | 257 | 515 | 179 | 78 | 40 |
| pair-s3 | 260 | 260 | 520 | 210 | 85 | 47 |
| boost-s3 | 261 | 260 | 521 | 182 | 79 | 41 |

Bars:
- gain −7 (seed 2) and −6 (seed 3), mean −6.5;
- guard (`pair` − `boost` exact): 4 and −1 (limit 33), holds.

**Does not add.**

Answer-word fact rows by kind, both seeds summed (`pair`, `boost`):

| kind | pair | boost |
|---|---|---|
| capital | 11 | 5 |
| opposite | 8 | 4 |
| add | 3 | 1 |
| day after | 20 | 18 |
| legs | 54 | 52 |
| colour | 28 | 27 |
| bigger | 25 | 26 |
| sound | 21 | 24 |

**Weight census.** I-4, TRAIN: 187,413 pairs; R min 1, p50 6, p90 48, p99 545, max 4,096 (cap 4,096). It is the same in both `boost` training logs, compared on the laptop. The `boost` scoring loads read kind `boost`; the `pair` loads read `none`. Each training log ends with all 187,413 rows nonzero (rms 0.0302 to 0.0307). Every scoring pass shows each sampled reply reading a bag (1,108 of 1,108 and 1,110 of 1,110).

**Speed.** Training seconds for 16,384 steps:
- `pair`: 2,849 and 2,926 (5.75 and 5.60 steps a second);
- `boost`: 2,587 and 2,672 (6.33 and 6.13).

Each lane trained `pair` first and `boost` second on the same pod. So the gap reads as host warm-up and sharing, not as `boost` being faster; claude-373 saw no gap. Card memory peaked at 4,496 to 4,560 MiB for every model.

**Identity.**
- Binaries are the same on both pods: `train-in-space` `e869d490cc34831aab8d95a9feb2425cd2c18edc0a010b8baa44324061b88310`; `fbs-snapshot` `cd3e8a984770686491395f9b586a71605535104ef27b9332cf010653d7bb4acb`.
- Toolchain: cargo 1.97.1, rustc 1.97.1.
- Dictionary SHA-256: `0b0640280493d20d412d17cb2a16bebbd798bd8d6b13dfbf47042d046b69b82e`, the same as claude-370's and claude-373's, in all four training logs.
- Scored-row identities are the same on both pods: P14 `6944401c…e282`, P23 `b40ce94f…efd1`.
- Reader SHA-256 values match the sheet: `read-374.sh` `48c9fbc9de2f2af44a3a8ad3e5efb9136b0b38001a3c95ad0a10942afba2a5c2`, `subst-374.sh` `8384c47251e75be3b07754ab841e67b9f3451743f7616bf9ec3882bbb791b691`.

## Checks run at the head

| Check | Result |
|---|---|
| `pre` on both lanes, at `4aeef3c09`: GPU tests of `uor-r4-float-train` | 580 passed, 0 failed, 18 ignored (all 32 pair-rows tests) |
| I-1 (inputs' hashes), I-5, I-2 and I-4 (census), both lanes | pass; census equal on both lanes |
| I-8 on the `boost` smoke (P14, P23), both lanes | PASS |
| I-8 at every score stage (kind, rows, dictionary and bag lines) | PASS at all four |
| Each `boost` training census against I-4, on the laptop | equal at both seeds |
| Every stage's SHA256SUMS against the fetched tarballs on the laptop; gate-down before each pod went down | every stage of both lanes matches; both pods down (exit 0); `uor-pod status` 0 running |
| `uor-harness prep-check` on the sheet at `e4823001d` | `FIT FIT yes` |

**Review.** The run sheet passed its FIT on the fourth fresh read. The first read found that I-8 checked only the weight lines and that the smoke did not gate. The third found that the down gate held a pod whose stages were skipped and that the lease was released before `down`. Each was fixed and tested on claude-373's real logs or on a fake folder before compute. This REPORT and the code at the head take a fresh-context exact-head review before the queue.

## Skills used

None.

## Still open

- **Weigh shape and content apart, by what the pair is, not by how rare.** Neither 1/R (claude-373) nor 1 + 1/R (here) moves facts. Rarity does not tell a content pair from single-conversation noise. The next form to try keys the row on the question's content word: the noun after "capital of", the operand of "add". It would not key on all pairs. Alternatively, keep claude-370's plain bag for the reply's shape and give a second, content-only bag its own seam.
- **Facts are still not reached by pair rows in any of the three forms.** Exact capital is 0 to 2 of 68 across all six models of claude-370, claude-373 and this unit. The reply has the frame ("The capital of Kenya is") and no answer.
- **The answer-word bar counts a reply that only echoes the question's last noun.** For capital, both arms get credit for "…the capital of Norway." The verdict here does not rest on it, because the exact fact rows fall too. A next sheet should bar on exact fact rows, or on the answer word excluding words of the turn.
- **`load` should refuse rows whose kind disagrees with the checkpoint's config.** Two cases, both low:
  - the config says weighted, but the rows file has no `weights` tensor (carried from claude-373);
  - the config says `boost`, but the rows file has lost its mark, so it loads as `replies`.

  Today only the printed kind and I-8 catch these (review).
- **Counting and number work** still need a mechanism on the operands.

## Plain summary

claude-370 gave the model one row of numbers per pair of words the user typed, and it got better at saying the user's words back. claude-373 scaled each row down by how many different answers the pair appears with. That broke where replies end. This unit kept every row at full strength and only added a bonus for rare pairs, at most double.

The copying gain survived this time: exact replies stayed the same, 515 and 521 against 519 and 520. Facts did not improve. Slightly fewer replies ended on the right word (78 and 79 against 85 and 85 of 188), and the model still knows no capital city. A bonus for rare pairs mostly raises one-off noise, and it cannot find the few pairs that carry a fact. The next try has to key on which word the question is about, not on how rare a pair is.
