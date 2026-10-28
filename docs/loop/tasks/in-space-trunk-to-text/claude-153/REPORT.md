# REPORT: in-space-trunk-to-text, claude-153

Lab: claude. Order: [ORDER.md](ORDER.md). Prep sheet: [in-space-trunk-to-text-wide-cell-2026-10-28](../../../../harness/prep/in-space-trunk-to-text-wide-cell-2026-10-28.md). TEST FITNESS: FIT, round 2, and PRE-RUN: PASS, by fresh non-author reviewers (owner log on #2028, comment `6470227073`).

The reading, by the sheet's bars: **(w), width 512 with 8 read heads on the pinned TRAIN, is cell D against claude-152's (a), with no flag F.** Copies clear every bar at seed 2 only, and openings clear at neither seed. By the sheet, width 256 stays, and the next unit goes after the openings directly. The item stays open.

**Speed.**

- **Training,** each arm alone on one RTX 5090, at main's `e2f15d3f`. Context tokens/s at seeds 1 and 2:

  | Arm | Model | tokens/s | Time for 16,384 steps |
  |---|---|---|---|
  | (a), claude-152 | width 256, 4 read heads | `32,049` and `32,329` | about 26 minutes |
  | (w) | width 512, 8 read heads | `17,033` and `16,888` | `49.2` and `49.7` minutes |
  | (w0) | width 512, no read | `38,578` and `39,039` | `21.7` and `21.5` minutes |

  - **Width 512 with 8 heads trains at 52–53% of (a)'s rate.** The sheet projected about `16,800`.
  - **The read is most of a width-512 step:** without it, the same trunk trains 2.3 times faster.
- **Decode,** in this runner: float, 64 tokens after 16, 64 and 192 context tokens. The runner re-runs the whole window for each token, so this times the runner, not a served decode.

  | Model | Seed 1 (16 / 64 / 192 context) | Seed 2 |
  |---|---|---|
  | width 256, 4 heads, (a) (claude-152) | `212 / 198 / 180` tokens/s | `211 / 198 / 180` |
  | width 512, 8 heads, (w) | `149 / 139 / 119` | `152 / 141 / 121` |
  | width 512, no read, (w0) | `348 / 345 / 336` | `318 / 299 / 288` |

  **(w) decodes 28–34% slower than (a),** seed for seed. Without the read, width 512 decodes faster than (a)'s width 256 with it. There is no integer serving path for the read yet, so there is no integer decode figure.

## What did not work, first

- **Width 512 does not move the openings.** At step 16,384, `O` (of 1,226) is `209` and `347`, against a bar of `352`. The no-read control reads `329` and `253`. No arm in this unit or claude-152 has cleared its openings bar.
- **Copies pass at one seed only, so they do not pass.** At step 16,384, of 4,130 copied rows:

  | Copies bar | (w) seed 1 | (w) seed 2 | Bar |
  |---|---|---|---|
  | 1. against (a) | `2,394` ✗ | `3,265` ✓ | `≥ 3,021` (`2,933 + 87.5`) |
  | 2. against the no-read floor | `2,394` ✓ | `3,265` ✓ | `≥ 1,115` (`1,031 + 83.4`) |
  | 3. copy gain over rest gain | `−13.1` against `+0.2` points ✗ | `+8.0` against `+0.5` points ✓ | the copy gain must be larger |

  Seed 1 fails bars 1 and 3. The sheet counts a pass at one seed as no pass.
- **Classified: configuration, not instrument.** Every instrument check passed at every seed (below), and the binary is claude-152's, byte for byte. D closes this configuration only (CHARTER §4.1): width 512 with 8 heads, 16,384 steps, on the pinned TRAIN, read against (a)'s better seed.
- **No checkpoint meets the item's done-when 2.** `R16` is `0.034` and `0.032` for (w), and `0.017` and `0.010` for (w0), against `0.05`. Every final checkpoint meets done-when 1 (`O ≥ 197` and `T ≥ 7,002`).
- **The replies are fluent templates, and none brings back the user's words.** At (w) seed 1, step 16,384:
  - "Remember, the code word is kandgind." gets "Okay, I'll remember that.";
  - "My lucky number is 5246." gets "Got it, your friend is76.";
  - "Add 4 and 2." gets "A new bike! That sounds wonderful.".

  At seed 2, the first prompt loops ("um following told you you you …"), and "My lucky number is 5246." gets "Got it, you grew up in Dum.". The no-read control at seed 1 gets the template right with the wrong number: "Got it, your lucky number is 72.".

  Copied rows are scored with the true reply so far as context. A greedy reply builds on its own earlier tokens, so one early miss derails what follows. That is how 79% of copied rows can be named at seed 2 while no free reply here repeats a number.
- **Openings swing within a run, not only across seeds.** Between adjacent checkpoints, (w0) seed 1 went from `205` to `329`, and (w) seed 1 from `258` to `228` to `209`. A bar read at one checkpoint carries that noise too.

## What worked

- **Fit rose to the best yet.** `T` is `10,152` and `11,140` of 14,287. That is `+824` and `+567` over (a) at the same seed, and above every earlier final checkpoint on this item (claude-152's best was `10,573`). Flag F (`T < 8,828`) is far off.
- **Seed for seed, (w) is ahead of (a) on copies and reach.** This is not the bar, which asks both seeds to beat (a)'s better seed.
  - **Copies:** `2,394` and `3,265`, against (a)'s `1,517` and `2,933`.
  - **`R16`:** `0.034` and `0.032`, against (a)'s `0.020` and `0.010`.

  - **On copies and `T`,** (a)'s own seeds differ by `1,416` copies and `1,245` in `T`, more than these gains. So the direction favors width, and this unit cannot confirm it. The sheet stated this limit before the runs.
  - **On `R16`,** the gain is larger than (a)'s spread: (a)'s seeds differ by `0.010`, and (w) leads by `0.014` and `0.022`. `R16` is not one of the sheet's bars, so this changes no cell. It is the one difference here that clears (a)'s seed noise, though it stays below the item's `0.05`.
- **At width 512, the read carries the copying.** (w) against (w0), the same trunk without the read:
  - copies `+1,363` and `+2,267`;
  - `T` `+1,054` and `+2,141`;
  - `R16` about two and three times larger.

  Openings do not follow: (w0) seed 1 reads `329`, more than (w) at seed 1.
- **The no-read floor is stable across seeds:** (w0) copies `1,031` and `998`, and `T` `9,098` and `8,999`.

## What changed

No code. This unit adds the prep sheet, this REPORT, its ORDER and one notebook row.

## Measurements

### The runs

`train-in-space --objective cell --retain-rates 1 --decay-mult 1 --batch 16 --steps 16384 --eval-every 2048 --lr 0.001 --lr-end 0 --seed S --width 512 --hamming-read H` on the pinned TRAIN, with `H = 8` for (w) and `H = 0` for (w0). Each arm ran alone on one RTX 5090; (w0)'s two seeds ran one after the other on one pod. Every pod built and tested its own binary from main's `e2f15d3fcc91a6042b77fbcf717a79c39eaf215d`. All three builds have sha256 `5e2f5d01cc3b897f11fb06f238033bcdb869d3ca99eb3dcba3157732cb30202b`, the same bytes as claude-152's four builds from `d102c594`.

At step 16,384, seed 1 / seed 2:

| Arm | `T` (of 14,287) | `O` (of 1,226) | Copies (of 4,130) | Rest (of 9,094) | Opening picks | `R16` | Read weight | Train loss | Cell |
|---|---|---|---|---|---|---|---|---|---|
| (w) | `10,152 / 11,140` | `209 / 347` | `2,394 / 3,265` | `7,615 / 7,641` | `52 / 68` | `0.0339 / 0.0316` | `0.630 / 0.617` | `0.649 / 0.614` | D |
| (w0) | `9,098 / 8,999` | `329 / 253` | `1,031 / 998` | `7,798 / 7,769` | `44 / 31` | `0.0166 / 0.0104` | none | `0.649 / 0.654` | floor |
| (a), claude-152 (reference) | `9,328 / 10,573` | `306 / 245` | `1,517 / 2,933` | `7,600 / 7,490` | `73 / 80` | `0.0198 / 0.0098` | `0.508 / 0.559` | `0.604 / 0.618` | C |

- `R16 = shift(16) / shift(0)` from `--reach` on the step-16,384 checkpoint. At step 0 it reads `0.726` and `0.661`, the same for (w) and (w0) at each seed.
- **Over training,** at steps 2,048 to 16,384:
  - (w)'s `T` rises over the run: `5,998` to `10,152` at seed 1, with one dip at step 14,336, and `8,992` to `11,140` at seed 2, at every checkpoint.
  - (w)'s copies rise at seed 1 (`843` to `2,394`, with the same dip). At seed 2 they peak at step 2,048 (`3,431`) and end at `3,265`.
  - Openings move up and down by as much as `124` between adjacent checkpoints.

**The bars,** from the sheet: openings `≥ 352`, copies `≥ 3,021` against (a), and flag F at `T < 8,828`. The floor bar, `≥ 1,115`, is `best(C_w0) + 3·√(N·p(1−p))` = `1,031 + 83.4`.

**Instrument checks,** at every seed:

- **I-1:** `123 passed; 0 failed; 5 ignored` on each of the three pods' builds, as on claude-152's pods. The cell objective's negative gate was not rerun: the code and the binary are claude-152's, byte for byte.
- **I-2:** every arm's step-0 `dev_nearest` equals claude-152's (c) at the same seed: `1/0/756` at seed 1 and `3/1/659` at seed 2. It was checked on each 256-step probe and again on each arm. Each arm's `config.json` records width 512, its head count (`8` or `0`) and `"train_split": null`.
- **I-3:** (w)'s read weight ends above 0 at both seeds (`0.630` and `0.617`).
- **I-4:** all 17 losses of every arm are finite: 8 training-loss means and 9 DEV losses (`progress.jsonl`; the job log's I-4 line).

### Evidence

- **Location:** `caseyallard/uor-r4-store:results/claude/wide-cell-2026-10-28/`. It holds one tar per pod: `pod1-w-s1.tar`, `pod2-w-s2.tar` and `pod3-w0.tar` (both (w0) seeds). Each holds the pod's whole job folder:
  - the job log and every step's out, err and exit;
  - each arm's config, progress, samples and checkpoints;
  - reach, inspect and decode;
  - the probes, and `uor-pod`'s own job logs.

  The job script and the watcher are beside them as `job.sh` and `watch.sh`.
- **Seals:** each tar has its own `.SHA256SUMS`, written on the pod. `SHA256SUMS-claude-153` lists all three, sha256 `5763836dc7f0a6a1d2ae5a29ce5a324c7b646c8e9a622a89321713ef8ba83fd8`:
  - `pod1-w-s1.tar`: `f9124eb313326479430e9794d2e699555862b8b1f834059ac188f81f64f0d82b`;
  - `pod2-w-s2.tar`: `f87ca34cf2842af598f4cc27cd0c2d33f6772c1384e67b7d9ce57df92dcfedcc`;
  - `pod3-w0.tar`: `5897a9e6a691340b33a182d7d9e8b09c063231ea3e7768853d49da8e731311eb`.
- **Checks:** every tar was read back from the store to the laptop and checked against the hash the pod wrote, before that pod came down.
- **Cost:** 3.50 pod-hours, about `$2.57` in all. All pods are down.
  - Pods 1 and 2: Vast RTX 5090s at `$0.563/h` (`$0.71` and `$0.62`).
  - Pod 3: an RTX 5090 at `$1.093/h` (`$1.02`).
  - A fourth create, at `$1.106/h`, came up exited and was deleted after 12 minutes; the pod ledger counts it at `$0.22`. (w0) seed 2 ran on pod 3 instead, the sheet's fallback.

## Checks run at the head

This unit changes no code. `git diff e2f15d3fc -- crates/ Cargo.lock` is empty at the head; its commits touch only the ORDER, the sheet, this REPORT and the notebook.

- **CUDA library tests on an RTX 5090,** on each of the three pods' builds: `123 passed; 0 failed; 5 ignored`.
- **`uor-harness loop report --check`:** PASS.
- **`uor-harness loop gates --full`:** 11 of 11 green.
- **In-unit reviews,** each by a fresh reviewer that never saw the plan:
  - **FIT round 1: NOT FIT.** The one blocker was flag F at seed 2, inside (a)'s seed spread. It was fixed, with three smaller points (listed in the sheet).
  - **FIT round 2: FIT, and the pre-run review: PASS.** Its notes are applied (in the sheet).
  - **The claims check of this REPORT and its notebook row: CHANGES.** It confirmed about 60 claims against the sealed tars and the pod ledger, and found:
    - one wrong statement: (w)'s `R16` lead was called smaller than (a)'s seed spread, and it is larger. Fixed here and in the notebook row;
    - one claim it could not confirm: the count of 17 losses in I-4, now stated as 8 training-loss means and 9 DEV losses;
    - one cosmetic gap: the fourth pod's rate, now given.

    A fresh re-review of the fix at the final head: APPROVE.

## Skills used

- **holding-the-goal.** Casey asked for one run: "cell objective, pinned TRAIN, width 512 with 8 read heads, seeds 1 and 2, plus a width-512 no-read control." The sheet ran exactly those four arms, scored by claude-152's test against claude-152's (a), and changed nothing else.
- **verify-before-done.** The code identity was checked twice: the diff from claude-152's commit is empty, and the binaries hash the same. Every bar was fixed before the runs. Every tar was read back against its pod's hash before the pod came down.

## Still open

- **The openings, by the sheet's D.** No arm in two units moves them past the bar. claude-145's probe names `607` openings from the prompt's first 8 tokens, beyond the trunk's reach: `R16` is at most `0.034`, against the item's `0.05`. The next unit carries the prompt's first words to the reply's first token directly, rather than widening the trunk.
- **Read openings over several checkpoints.** They swing by up to `124` between adjacent checkpoints of one run. A later sheet can bar the mean of the last three checkpoints rather than the last one alone.
- **The direction for width is unconfirmed on the bars.** Seed for seed, (w) is ahead of (a) on fit and copies by less than (a)'s seed spread, and on reach by more (`R16`, not a bar). More seeds, or a lower-noise reading, would settle it. By the sheet, width 256 stays until then.
- **Free replies do not show the copying.** Teacher-forced copies reach 79%, but no greedy reply repeats a number. A decode-side test of copying (does the free reply contain the user's token?) would measure what a user sees.
- **Growing a trained trunk instead of restarting** waits until the recipe settles (Casey, 28 Oct: "lets settle before we push").
- **claude-152's four low-severity code-review notes** stay open for the next unit that touches that code.

## Plain summary

Casey asked for the one combination claude-152 left unrun: the margin objective, the original training text, and a trunk twice as wide with 8 read heads, plus the same wide trunk with no read as a control.

- **The wider trunk fits best of any run so far.** It gets `10,152` and `11,140` of 14,287 tokens right, against `9,328` and `10,573` at width 256.
- **It does not pass the pre-registered test of using the prompt.** At one seed it repeats the user's words more than the best width-256 run (`3,265` of 4,130). At the other it does not (`2,394`), and both seeds had to pass.
- **A reply's first word still does not follow the question.** No run has moved that.
- **The read does the copying:** the same wide trunk without it repeats about a quarter of those words.
- **The wider model costs about half the training speed, and decodes 28–34% slower.**

By the sheet, the trunk stays at width 256, and the next step goes after how a reply starts. The runs cost about `$2.57`, and all pods are down.

## Done when

1. **Met.** The prep sheet has a fresh non-author FIT (round 2, after a NOT FIT round 1 was fixed) and a fresh pre-run review (PASS), logged on #2028 (`6470227073`).
2. **Met.** (w) and (w0) ran at seeds 1 and 2 at main's `e2f15d3f`, each pod building and testing its own binary. The results are sealed in `results/claude/wide-cell-2026-10-28/` and were read back before each pod came down.
3. **Met.** This REPORT leads with tokens/s, training and decode against context. It gives (w)'s cell, D, and the item's done-when 1 and 2 on each final checkpoint: 1 is met at all four, and 2 at none. The greedy replies are recorded at every checkpoint in each arm's `samples.md` and `samples.jsonl` (done-when 3).
4. **Met on merge.** The result has one notebook row ([float-training](../../../../research/notebook/float-training.md)).

The item's own done-when is not met, so it stays open.
