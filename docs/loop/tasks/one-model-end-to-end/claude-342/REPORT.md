# REPORT: one-model-end-to-end, claude-342

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). ORDER: [ORDER.md](ORDER.md). Base `67972456d`. No GPU was used and no model was trained.

**In one line:** By the rule fixed before the count, 32 rows does not reach the question in TRAIN (1.2% of oasst2's scored rows, 76.3% of the capped mw-balp's, bar 80%). The bigger finding is in the sealed replies of claude-339: 240 to 255 of 256 end in a repeated-token loop, with a median onset at reply position 3 to 5 (claude-322: 130 and 140, onset 11). So the next unit checks the serving path, then runs a cause ladder at the new trainer commit.

## What did not work, first

- **By the pre-stated rule, 32 does not reach the question on either dialogue root.** At least 80% of a root's scored rows needed the question's start within 32 rows. `oasst2` has 1.2% and `cap1` has 76.3%, so both miss. The run-based count misses too (0.8% and 34.3%).
- **The count as first built measured the wrong thing on the capped roots (instrument, repaired).** It treated a whole unscored run as the question. `reply-cap` keeps a capped reply as unscored context, so on `cap1` to `cap6` one run can hold several whole turns: the first-turn run averages 84.9 rows (a one-off count over `cap1`'s tokens, not in the JSON), against a question of 11 (median). That first count read 34.3% for `cap1`.
  - **The repair:** the tool also reads the question from the tokens, from the word after the last `User:` before a reply to the word before the `\nAssistant:` header. It placed every reply on `oasst2` and `cap1` (0 not placed). The run-based numbers stay in the JSON, as the ORDER asked for them. The rule and its bar did not change, and both definitions miss, but I changed the instrument after seeing its first output.
- **The rule counts every scored row, and that limits what it can say about the carry.** `oasst2`'s replies run to hundreds of rows, and a row deep in a long reply is far from the question whether or not it needs it. The rule cannot say which rows need the question.
- **The count does not settle the carry's reach.** The first-row figures are at a reply's first row, and the value comes later (about 7 rows on the panel), so the share at the value's row is unmeasured and can be well under 93.1%. On human chat a longer carry would reach more: `oasst2`'s reply first rows go from 47.7% at 32 to 79.8% at 64. Windows are also ignored, so all of this is an upper bound on what a 192-token window sees.
- **Most scored rows have no question.** The three wiki roots hold 24,181,749 of the mix's 33,364,756 scored rows (72.5%; by tokens they are 33.1% of the mix), with no `User:` turn before them. Only 13.7% of scored rows (4,582,684) have a question's first word within 32 rows.
- **The six capped roots are one set of counts.** `cap1` to `cap6` have the same token and mask hashes, so the mix weighs that one data set six times.
- **The first fresh reviews sent this back.** The code review found no wrong number, but the distances behind the decision had no test that could fail. The claims and decision review found the ladder could not separate a serving-path fault, the read table had dropped half of the stopping bar, and the log was incomplete. A second round corrected two overclaims (a median onset read as a count, and a stated cause for the lost stopping) and required parity on the panel's own decode. All are fixed below.

## The sealed replies loop (laptop, claude-339's scores)

A reply "loops" here when its last 24 token ids repeat with a period of 1 to 8 ([`loop-class.jq`](loop-class.jq); inputs hashed in [`loop-class.txt`](loop-class.txt)). Onset is the reply position (from 0) where the repeat begins, over looping replies only. All at step 16,384 unless named.

| artifact | EOS | looping | period 1 | median onset | onset at most 5 | latest onset |
|---|---|---|---|---|---|---|
| claude-322 s1 | 109 | 130 | 89 | 11 | 23 | 33 |
| claude-322 s2 | 86 | 140 | 90 | 11 | 19 | 35 |
| L-a s1 | 6 | 243 | 228 | 3 | 182 | 38 |
| L-a s2 | 0 | 255 | 190 | 5 | 138 | 29 |
| L-aC s1 | 2 | 247 | 205 | 3 | 168 | 38 |
| L-aC s2 | 0 | 248 | 221 | 3 | 182 | 40 |
| L-aC `−C` s1 | 4 | 240 | 195 | 3 | 163 | 36 |
| L-aC `−C` s2 | 0 | 247 | 225 | 3 | 172 | 37 |

- **Replies now loop earlier and far more often.** The top reply at L-a seed 1 is ` Your friend's name name name name…` on 172 of 256 rows. claude-322 loops too, on 130 and 140 rows, with a median onset 6 to 8 positions later and far more stops. Onset spreads widely, so the median is not every reply.
- **Looping and stopping can't be ordered from this.** No reply here both stops with EOS and is counted as looping, so loops are counted only among replies that never stopped: claude-322 loops on 88% and 82% of those, L-a on 97% and 99.6%.
- **It is not the role-header failure.** claude-321 lost stopping by writing `\n\nAssistant:` instead of EOS. In L-a and L-aC, 0 of 256 replies carry that string at step 16,384.
- **It is there by step 8,192** (L-a: 234 and 248 of 256 looping), so more steps at this recipe are not indicated.
- **Why it happens is not known.** L-a's sealed comparison against claude-322, a row-level grade from the float `--score-rows` path on held-out human text, reads `mixed` (d +3,222 inside the band, +7,795 ahead), so its next-token prediction on that text is not worse while its served replies loop. That cannot separate a serving fault from a training fault that shows only when the model runs free. claude-339 did not measure float-to-integer parity on these asking checkpoints. A training cause, such as the pull teaching a copy of the token just written, is an untested alternative.

## The decision

**Decided: the next unit (1) checks float-to-integer parity on L-a's sealed `step-016384`, then (2) runs a cause ladder at pin `721a4784f`.** This goes outside the two options row 4c names, and the log says so. It is my own decision review (§5.1), logged as `[claude] owner log:` on #3407, and Casey can reverse it. No bar changes.

**What this step is for.** Step 4 is one model trained, served and scored end to end. This moves no milestone by itself. It decides whether the ask heads, the only mechanism that has passed the recall task, can stay on language.

**Pivot note (§4.4).** claude-321, claude-322 and claude-339 each left the panel's complete rows at 0, the line's headline. So the ask-head line gets one decisive step: this ladder. Full success only restores claude-322's stopping, still with 0 complete rows. That is still the right spend, because a reply stuck in a loop to the length cap cannot be complete, and every later recall reading needs replies that are not. If the ask heads themselves read as the cause at both seeds, with parity clean, there are no more tuning runs at this recipe, and the next unit is a pivot decision on the recall route.

| option | what the evidence says | call |
|---|---|---|
| **A longer carry** | Not ruled out by the count: `oasst2` would gain. But the panel's question is within 26 rows, so 32 was in reach of the panel while row 4c still read, and most looping replies begin to repeat before the 7-token lead-in that comes before the value. | Later. It can't be read while replies loop. |
| **A question-row placement** | The most direct recall lever left: the question-row `both` placement passed at both seeds on the recall task, while the scored placement L-a already uses split (B2-aC: seed 2 passed, seed 1 did not). It needs new trainer code: `--hamming-attract-at question` and `both` are refused on a language run. | After the ladder. |
| **A stronger EOS share** | Named by the sheet's row 3 for a different reading. TRAIN is the same as claude-322's, which stopped on 109 and 86, so the share does not explain this loss. | Rejected for now. |
| **A decode-time stop at the role header** | 0 of 256 replies carry `\n\nAssistant:`, so it has nothing to end. | Rejected. |
| **Parity, then a cause ladder** | L-a differs from claude-322 in the ask heads, the pull, the trainer commit (20 commits touch the trainer crate between the pins) and the serving path for asks. None is separated. | **Next.** |

**What deepseek-264 already covers (merged while this PR was open).** On real L-aC pairs it found the CPU port and the integer engine equal (8 of 8 reports) and reproduced the `−C` artifacts byte for byte (4 of 4). That compares two engines running the same exported file, so it lowers the odds of an arithmetic bug in one engine. It does not compare the trained float with that file, and it was not run on L-a, so the parity step stands.

**The steps,** under one sheet, FIT and pre-run review. Parity first, at about the $0.07 of pod time a parity run took in claude-336. That run traced 8 fixed 192-token sequences, which would miss a fault that shows on short contexts, so this one must compare float and integer on the panel's own prompts and on their decode, where the loops start. Disagreement means any Hamming question or key bit that differs from the float trace rounded as exported, row by row, with the float fed the integer's decoded reply ids, so it cannot run free past a first flip. Rounding alone moved 21 of 3,072 depth-2 type decisions against the unrounded float in claude-336, and that is not a fault. Then, on L-a's recipe, TRAIN, seeds and step count, at pin `721a4784f` (the merged crate differs from it on the non-finite path only):

| arm | flags | what it isolates |
|---|---|---|
| **N0** | no ask heads, no pull, no carry | the trainer commit, against claude-322, also read against claude-322's sealed dev curve at the same seeds (an exact match clears the commit far more strongly than a floor) |
| **N1** | ask heads, no pull, no carry | the ask heads, against N0 |
| L-a (sealed) | ask heads and pull | the pull, against N1 (not paired with it, as claude-339's sheet warns of separate runs) |

**How it will read.** "Holds" is claude-339's whole stopping bar: at least 64 EOS at each seed and no collapse. The first matching row decides. `loop-class` is reported for every arm beside these, since an arm can hold 64 EOS and loop on the rest, and it decides no row. The next sheet owns the final wording, and its bars may not be weaker than these.

| # | reading | next |
|---|---|---|
| P | the integer disagrees with the float rounded as exported (defined above) on L-a's asks, on the panel's prompts or their decode | **`blocked`:** a serving fault. Repair and re-serve the sealed checkpoints before any ask reading stands. N0 can still run. |
| 1 | N0 does not hold, at both seeds | The commit lost it. Bisect the 20 commits. No ask run until N0 holds. |
| 2 | one seed holds and the other does not, on N0 or N1 | **Mixed:** no direction read. Report both seeds and take a second decision review. |
| 3 | N0 holds, N1 does not at either seed, but N1 is at least 32 EOS above L-a at both | Partial: both the ask heads and the pull cost stopping. A decision review weighs the two. |
| 4 | N0 holds, N1 does not at both seeds | The ask heads did, with parity clean. The pivot decision above. |
| 5 | N0 and N1 hold, at both seeds | The pull did. The placement work starts from the ask heads and changes the pull. |

## What changed

- `crates/uor-r4-float-train/src/bin/train-question-reach.rs` (new), its seven tests and six negative gates.
- [`train-question-reach.json`](train-question-reach.json), [`loop-class.jq`](loop-class.jq), [`loop-class.txt`](loop-class.txt), this REPORT, the ORDER's hash note and the notebook row.

## Measurements

Tool: `train-question-reach` over TRAIN d3mwcap6, whose `tokens.u16` and `response_mask.u8` match the manifest's hashes (`0273173e…`, `da506e85…`). About 4 s on the laptop. Percent of rows whose question's first word (read from the tokens) is within that reach:

| root | rows | 4 | 8 | 16 | **32** | 64 | 128 | median question |
|---|---|---|---|---|---|---|---|---|
| `oasst2`, all scored rows | 3,234,805 | 0 | 0 | 0 | **1.2** | 6.1 | 18.1 | 28 |
| `oasst2`, reply first rows | 7,784 | 0 | 0.7 | 10.3 | **47.7** | 79.8 | 94.7 | |
| `cap1`, all scored rows | 991,367 | 0 | 0 | 9.9 | **76.3** | 92.2 | 100.0 | 11 |
| `cap1`, reply first rows | 70,672 | 0 | 0 | 48.4 | **93.1** | 99.9 | 100.0 | |

- A row's distance counts from the question's first word. Every reply first row reaches the question's last word within 8 rows (it is always 7 back, past the 6-token header).
- The panel's question is 6 to 13 tokens (median 10), and its first word is at most 26 rows back from the row emitting the value (claude-339's sheet). `cap1`'s median question of 11 matches that family.

## Checks run at the head

Run on the final code; the later commits change only text and evidence files.

- `cargo test --offline -p uor-r4-float-train --bin train-question-reach`: 7 passed.
- **Negative gates:** six breaks across four lines, one at a time, each failing a test meant to catch it, and the file restored: the first-row asked distances (zeroed, and one row short), the question's last word, the run-based end distance, and the `first < header` guard (`<=`, and `first + 1 <`). The first-row distance to the question's last word has no boundary test. It is always 7 by construction, and the real data reads 0 at reach 4 and all at reach 8.
- `train-question-reach` exits 0 on TRAIN. It refuses a mismatch against the manifest's scored count or a bad mask value.
- **Self-checks in the output:** 0 replies not placed on `oasst2` and `cap1`, and per-root scored counts equal to the manifest's. The per-root scored rows sum to the mix's 33,364,756.
- **The tool** builds no model and no tensor, and it uses no GPU.

## Skills used

- None beyond the loop itself: the ORDER, the fresh exact-head reviews and the decision review.

## Still open

- **Parity on L-a's sealed checkpoint, then the ladder** (N0 and N1, two seeds each): the next unit, under its own sheet.
- **Why replies loop:** unknown. The parity and the ladder may narrow it, but the ladder is read on stopping, not on loops.
- **The question-row placement:** needs new trainer code, and comes after the ladder.
- **A windowed count at the value's row** is not measured, if the carry's reach comes back as the question.
- **mw-balp's 35 paraphrase files** are not published, and **`test-watch`** still depends on signal timing.

Closes nothing: #3407 stays open.

## Plain summary

We wanted to know whether a 32-token memory is long enough to hold the question while the model writes its answer. So we counted, in the training data, how far back each question sits.

In the synthetic conversations that resemble our test, the start of the question is within 32 tokens at the beginning of 93% of replies. In real human chat it is 48%, because people ask longer questions, and 72.5% of the positions the model trains on are encyclopedia prose with no question at all. By the strict rule we set beforehand, 32 tokens does not reach (76.3% against 80% on the synthetic data).

Reading the last model's answers turned up something more urgent. Almost all of them get stuck repeating, typically one word ("name name name…"), until the length limit, and more than half start within the first six tokens. The model before it did this less often and later. We don't yet know why.

So the next step is a small controlled experiment. First we check that the model behaves the same when served as it did in training, then we take the new pieces out one at a time and see which, if any, brings stopping back. Nothing was trained in this step.
