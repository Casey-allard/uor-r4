# REPORT: one-model-end-to-end, claude-296 (does the trunk learn human text?)

Lab: claude. Order: [ORDER.md](ORDER.md). Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Pin: `48c4bf384e4fa463fa869fef9a8a6e8d7396f067`. PR #3763.

Six runs at (256, D1) were trained, scored, sealed and graded: (m1), (lr3) and (near), at seeds 1 and 2, against claude-295's sealed control. The prep sheet is [`one-model-end-to-end-human-text-2026-11-06`](../../../../harness/prep/one-model-end-to-end-human-text-2026-11-06.md) (FIT, round 2). Every number names its source file. The seal is `results/claude/one-model-end-to-end-human-text-2026-11-06/runs/one-model-human-text.tar.gz`, sha256 `b0dd71f6bcf44b6d734047bb6bfe6c9d981212f6fb94f032a567ac5d95e9ddd9`.

**Claim check.** This is step 2 of Casey's one track (#2028 6675163438, 6 November: "Yes, one track"): "the classification unit (claude-296, one-model-end-to-end)". It classifies claude-295's flat training loss on human text, instrument before mechanism (§4.2), and names the next change to the shipped model's recipe.

## What did not work, first

- **The one-track bar is not met.** No arm beats T2 on the whole human panel.
  - (m1) against T2 reads `mixed` (d −3,530 / −8,764, band 5,234). That was expected: (m1) learns T2's own pick, so T2 bounds it (the sheet's "Not read here").
  - (near) against T2 reads `behind` (d −36,343 / −29,329, band 7,014). It reaches 85.6% / 88.4% of T2's 253,018 hits.
  - (near)'s lead over T2 on oasst2 is not on unseen rows. There it is `behind` (d −1,981 / −1,576, band 405), so the lead comes from the rows the unseen filter drops: an 8-token context found in TRAIN, or fewer than 8 tokens into a document.
- **(lr3) does not move.** Against the control it reads `inside` (d −3,209 / +24,987, band 28,196), and against the unigram `inside` too.
  - Its seeds land 32,933 rows apart (15,193 / 48,126). So a learning rate of 0.003 is noisy here: it is not shown better at these seeds.
  - This closes the configuration (cell objective, lr 0.003, (256, D1), 8,192 steps), not a mechanism (§4.1).
- **What (m1) does not show.**
  - It does not show that the trunk uses more than the last input token.
  - It cannot tell the targets' ambiguity apart from their smaller number. Under `mode:1`, 5,654,617 of the 6,865,294 targets the loss reads are replaced, and the distinct targets fall from 3,950 to 653 (I-m).
- **(near)'s gain is a reading, not yet a mechanism claim.** Why averaging over 8 rivals lets the trunk settle on ambiguous text is a hypothesis (Q2), and it has not been tested.
- **The cost passed the sheet's $3 cap: $3.20.**
  - The pod ran about 1 h 48 min (04:57Z–06:46Z) on a two-card host at $1.775/h. The sheet's plan was one card at $0.563/h, about $1.20.
  - The watcher's spend estimate passed $3 at about 06:39Z, while (near)'s two runs were finishing.
  - The D67 record replaced the sheet's "arms past the cap are NOT_RUN" with the charter's extend rule before the pod went up, so nothing was cut.
- **Templated DEV is erratic under (m1)** (in-run `dev_top1_hits`, read for no bar): 57 / 51 at step 1,024, then 89 / 8 at step 8,192, so seed 1 rose and seed 2 fell. DEV scores the observed templated targets, which (m1) never trains on.

## Q1: instrument or mechanism (read on (m1))

**Reading: mechanism, the objective.** This is the sheet's first Bar row:
- (m1) is `ahead` of the control;
- it is `ahead` of TRAIN's unigram;
- it is above the panel's best constant guess at both seeds.

Under fixed targets, the same trunk, recipe, loader, mask and inputs predict human text's real next tokens about 11 to 14 times as often as the control does. So the loader, the mask, the inputs and the trunk are not where claude-295's flat loss comes from. It comes from the cell objective, `relu(1 + max_r s(r) − s(target))`, under the real targets. That it fails there because many tokens can follow one context is a hypothesis (Q2), not tested here.

**The panel** (1,460,697 no-memory rows, row identity `5f88a5b3…`; `grade/*/grade.json`):

| run | seed 1 | seed 2 |
|---|---|---|
| control (claude-295, `w256-d1`) | 18,402 | 23,139 |
| (m1) | **249,488** | **244,254** |
| (lr3) | 15,193 | 48,126 |
| (near) | **216,675** | **223,689** |
| TRAIN's unigram | 37,421 | 37,421 |
| TRAIN's T2 | 253,018 | 253,018 |
| the panel's best constant guess (token 223) | 38,032 | 38,032 |

The best constant guess was counted from the sealed rows (`const-all.json`). Taking the best token for each sub-panel separately gives 45,336; (m1) and (near) are above that too.

**(m1)'s paired readings** (`scale-probe pair`, clean = D1's seen set; `pair/m1-vs-*.json`):

| (m1) against | all rows: d at s1 / s2, band | unseen rows: d at s1 / s2, band | verdict |
|---|---|---|---|
| the control | +231,086 / +221,115, band 9,971 | +204,123 / +195,692, band 8,431 | **ahead** |
| the unigram | +212,067 / +206,833, band 5,234 | +190,709 / +185,677, band 5,032 | **ahead** |
| T2 | −3,530 / −8,764, band 5,234 | −3,377 / −8,409, band 5,032 | mixed |

All four sub-panels read the same way: ahead of the control and the unigram, mixed against T2.

**How much of T2's pick each run learned** (`model_hits − b` from the T2 pair, over T2's 253,018 hits):
- control: 10,359 (4.1%) / 14,046 (5.6%);
- (m1): 249,396 (98.6%) / 243,821 (96.4%). Of (m1)'s 249,488 hits at seed 1, only 92 are on rows T2 misses.

So (m1) is close to T2 itself.

**The prediction mix on wiki-00000** (`mix.jq` over the no-memory rows; the targets hold 3,081 distinct tokens):

| run | distinct tokens predicted | hits | a context-blind guess, same mix |
|---|---|---|---|
| control s1 / s2 | 2,893 / 2,864 | 4,620 / 6,055 | 2,275 / 2,124 |
| (m1) s1 / s2 | 404 / 369 | 69,606 / 68,121 | 4,338 / 4,353 |
| (lr3) s1 / s2 | 2,717 / 2,798 | 3,698 / 11,117 | 2,198 / 2,256 |
| (near) s1 / s2 | 2,567 / 2,560 | 50,138 / 51,563 | 3,669 / 3,291 |

- The control spreads human text's token mix across contexts, as claude-295 found. Its context lift over the blind guess is 2.0–2.9×.
- (m1) names each context's likeliest token, with a lift of about 16×. Its five commonest predictions at seed 1, as predicted / target counts, are the frequent tokens, over-predicted as a mode predictor does:
  - token 265: 32,414 / 9,191;
  - token 14: 31,181 / 10,845;
  - token 85: 26,496 / 8,197;
  - token 223: 20,215 / 8,979;
  - token 201: 19,322 / 6,851.

**The loss, within each run** (`train_loss_mean`, step 1,024 → step 8,192; `progress.jsonl`):
- control: 1.00386 → 1.00335 (s1) and 1.00384 → 1.00302 (s2), as claude-295 reported.
- (m1), against its own targets: 1.00248 → **0.37933** (s1) and 1.00248 → **0.51977** (s2).
  - It was still falling at step 8,192, from 0.42433 / 0.56632 at step 7,168.

## Q2: the learning rate and the averaged objective

| arm | against the control | against the unigram | Q2 reading |
|---|---|---|---|
| (lr3) | `inside`: d −3,209 / +24,987, band 28,196 (unseen: inside) | `inside`: d −22,228 / +10,705, band 32,933 | **does not move** |
| (near) | `ahead`: d +198,273 / +200,550, band 2,277 (unseen: +172,386 / +174,641, band 2,255) | `ahead`: d +179,254 / +186,268, band 7,014 (unseen: +158,972 / +164,626, band 5,654) | **moves, and reaches the unigram** |

**(near)** is `--objective nearest`: the target's hinge against each of the 8 nearest wrong tokens, averaged (`in_space_long.rs`, `rival_hinge` and `nearest_loss_on`). The control's (cell) objective keeps only the hinge of the single best-scoring rival. (near) trains on the **observed** targets, so this is the trunk learning human text as it is written.
- **On every sub-panel it is ahead of the control and the unigram at both seeds.** Against T2 (`pair/near-vs-t2.json`):

  | sub-panel | (near) s1 / s2 | T2 | all rows: d at s1 / s2, band, verdict | unseen rows: d at s1 / s2, band, verdict |
  |---|---|---|---|---|
  | oasst2 | 32,200 / 32,648 | 30,972 | +1,228 / +1,676, band 448, **ahead** | −1,981 / −1,576, band 405, behind |
  | wiki-00000 | 50,138 / 51,563 | 70,676 | −20,538 / −19,113, band 1,425, behind | −19,206 / −17,926, band 1,280, behind |
  | wiki-00020 | 71,392 / 73,915 | 75,584 | −4,192 / −1,669, band 2,523, mixed | −2,950 / −1,056, band 1,894, mixed |
  | wiki-00040 | 62,945 / 65,563 | 75,786 | −12,841 / −10,223, band 2,618, behind | −10,977 / −8,902, band 2,075, behind |
  | whole panel | 216,675 / 223,689 | 253,018 | −36,343 / −29,329, band 7,014, behind | −35,114 / −29,460, band 5,654, behind |

  On oasst2, the conversational replies, (near) is ahead of T2 on all rows at both seeds, but behind it on the unseen rows. A row is unseen when its 8-token context is not in TRAIN (the instrument's `seen`). So the oasst2 lead is on contexts TRAIN holds, or on rows fewer than 8 tokens into a document.
- **It is not T2.** Of T2's 253,018 hits, (near) shares 71,268 (28.2%) / 81,945 (32.4%). It is right on 145,407 / 141,744 rows where T2 is wrong, and wrong on 181,750 / 171,073 where T2 is right.
- **Its prediction mix** on wiki-00000 keeps 2,567 / 2,560 distinct tokens, with a context lift of 13.7× / 15.7×. Its five commonest predictions at seed 1, as predicted / target counts:
  - token 223: 38,588 / 8,979;
  - token 201: 29,411 / 6,851;
  - token 14: 18,152 / 10,845;
  - token 265: 15,589 / 9,191;
  - token 19: 13,839 / 4,158.
- **Its loss** (on the averaged objective, so not comparable with the other arms) falls slowly: 1.00259 → 0.99367 (s1) and 1.00253 → 0.99295 (s2). It was still falling at step 8,192.
- **A hypothesis, not tested.** Under (cell), each row's pull toward its target equals the push from its single nearest rival. Where many tokens follow one context, the likeliest token is the nearest rival on most rows, and pushes the query off it as hard as its own rows pull it in. Under (near), the push from any one rival is an eighth of a row's weight, while the pull toward the target comes from every active hinge. So the query can settle where the targets are densest.

**(lr3)'s loss** (cell objective): 1.00675 → 1.00396 (s1) and 1.00672 → 1.00117 (s2). Its prediction mix stays the control's kind: 2,717 / 2,798 distinct tokens, with a context lift of 1.7× / 4.9×.

## Departures from the sheet (declared in the D67 record, #3407 6677290978, before the pod went up)

- **One seed per card on a two-card pod, not side by side on one card.**
  - Two runs side by side on one 5090 measured 0.15 s a step, against 0.074 s for one run alone, so side by side saved no time.
  - The pod (vast-59369171, Quebec, 2 × RTX 5090 at $1.775/h) ran seed 1 on GPU 0 and seed 2 on GPU 1. Each card ran (m1), then (lr3), then (near).
  - Every run trained alone on its card, as the control did.
- **The cost.** The charter's rule (never cancel a healthy run because a self-set budget ran out) replaced the sheet's "arms past the cap are NOT_RUN". The watcher's stops were set at $4.50 and 5 hours. The run came to $3.20 (above, first).
- **I-c, record only.** The control's own recipe was smoked at this pin and compared with claude-295's sealed pod-A smoke.
- **Scoring ran one panel per process**: claude-295's own repair (`rescore-295.sh`). DEV's agreement across the four processes was checked.
- **I-0 ran `--lib`** with CUDA on the pod. The crate's 281 lib and 19 bin tests passed on the laptop.
- **`fbs-snapshot` was not built**, since no reading here needs it.
- **The watcher took the pod down itself**, after the store read-back `cmp` matched (L-004).

## What changed

- **`--train-targets observed|mode:K`** (K from 1 to 4), in `train-in-space.rs` and `in_space_long.rs` (`d2564408d213a33642eb91c2d15ded864fb68ebd`).
  - `LongConfig::train_targets` is applied in `load_train` through `train_windows` and `fixed_targets`.
  - It is recorded in `config.json` and left out when `observed`.
  - It is refused with the turn memory and its reply table, with `--recall`, and without `--out`.
- **The `[targets]` line (I-m)** prints:
  - the split's scored total;
  - the usable windows' total;
  - the replaced count;
  - the distinct targets before and after;
  - and, for K of 2 or more, the context hidden by k. A target is counted only under the k it was taken at, not under every smaller k whose context it also hides.
- **Five unit tests.** The negative gate: loosening the tie rule to "at least" fails two of them.
- **The prep sheet**, at `docs/harness/prep/` (`48c4bf384e4fa463fa869fef9a8a6e8d7396f067`, the pin).
- **This REPORT and one notebook row** in `docs/research/notebook/float-training.md`.
- **The pin and claude-295's pin.** Between `e052abf4104fc33ba28a489cf0dfdc4e94ab70b1` and this one, only two things differ:
  - `uor-r4-cpu-train`, which the trainer does not link;
  - this flag.

  `Cargo.lock` and the scale-probe instrument are unchanged, so the laptop's grader grades both pins alike.
- **Off the repository:**
  - the seal, in the store;
  - the job, chain, seal, watch and reading scripts in `local/claude/claude-296/` (uncommitted): `job-296.sh`, `chain-296.sh`, `seal-296.sh`, `watch-296.sh`, `read-296.sh`, `read-extra-296.sh` and `mix.jq`.

## Measurements

### Instruments

- **I-0:** 382 passed, 0 failed, 12 ignored, on the pod with CUDA at the pin (`tests.out`). The sheet's five tests each read `ok`:
  - `fixed_targets_count_scored_positions_with_the_self_count_and_ties`;
  - `fixed_targets_back_off_and_stop_at_the_document`;
  - `train_windows_count_the_whole_split_and_observed_changes_nothing`;
  - `mode_one_is_a_brute_force_t2_at_every_scored_position`;
  - `train_targets_parse_serialize_and_refuse_the_turn_memory`.

  The trainer binary is `train-in-space-48c4bf38`, sha256 `528a26342dbd83b05fee3b410b880cd6cf2827c5b4f5bce95a89679cd31fb98a` (`binaries.sha256`).
- **I-1:** all three 64-step smokes exited 0 with a finite loss, 31–33 s each (`job.log`, 05:06–05:07Z).
- **I-3:** every input matched the sheet's shas: D1's tokens and mask, the four panels and DEV (`i3-*.txt`).
- **I-m.** The (m1) smoke and both (m1) runs print exactly this (`im-smoke.txt`, `m1-s*.im`):

  `[targets] mode:1: split scored 6865470, window scored 6865294, replaced 5654617, distinct 3950 -> 653`

  - The split's scored total equals 6,865,470, D1's scored count on the scale-probe sheet. It was also counted on the laptop from the files, past `tokens.u16`'s 64-byte header.
  - The 176 targets between the split's total and the windows' are in the windows `load_train` drops.
- **I-c (record only).** The control's own recipe, smoked at this pin, reproduces claude-295's sealed pod-A smoke exactly at step 64 (`ic.txt`):
  - loss 1.0043511670082808;
  - top-1 5, top-8 46, nearest 7.

  So the new flag leaves the control's training unchanged.
- **I-4:** all six graded checkpoints score the panel's 1,460,697 rows, with row identity `5f88a5b3295ba5d684a8c75206787642fd65a2377d2dbb268992ec5c71a621eb`.
- **I-5:** all six checkpoints' `.i5` files read `same` on every panel and on DEV (30 of 30 lines). DEV's rows agree across the four per-panel processes (`*.dev-agree`).
- **The seal.** Read back on the pod (`cmp`) and again on the laptop:
  - `SHA256SUMS` OK;
  - every extracted file matches `SHA256SUMS.files`.

  Two grading steps were checked against it:
  - (m1)'s and (lr3)'s rows were graded early from a copy taken off the pod, and all 32 copied rows files are byte-identical to the sealed ones;
  - every grade above was made from the seal.

### Timings and the pod

- **Training:** 1,607–1,626 s per run, each alone on its card: about 0.197 s a step at 8,192 steps. claude-295's control took 1,474 s.
- **Scoring:** about 4.5 minutes per checkpoint, as four panel processes of about 70 s each.
- **The cards:** mean utilization 74% / 71%, peak memory 11,280 / 11,600 MiB (`gpu.csv`).
- **The pod:** up at 04:57:35Z. The chain exited `job=0 seal=0` at 06:45:51Z, and the pod was down at 06:45:57Z ($3.20).

## Checks run at the head

- **The head** is this PR's final commit: the REPORT, the notebook row, a merge of `origin/main` (none of whose changes touch this PR's files), and the review's fixes. No code changed after the pin.
- **The laptop gates** at the head:
  - `loop report --check --order docs/loop/tasks/one-model-end-to-end/claude-296/ORDER.md`: PASS;
  - `loop pr-check --local`: PASS (order, scope, banned-presence, commit-order, report, owner-paths, rendered-lock). Its three D76 advisories:
    - the prep sheet's `## Stop rule` heading, outside `rule-sources.json`;
    - the ORDER's size, 5,259 B against a 4,096 B budget;
    - `docs/start/FILEMAP.md`'s stale rows, which this PR does not touch.
- **At the pin** (`48c4bf384e4fa463fa869fef9a8a6e8d7396f067`):
  - the crate's 281 lib and 19 bin tests passed on the laptop;
  - a stub-CUDA `cargo check` passed;
  - the negative gate for the tie rule failed two tests, as it should;
  - CI read 10 SUCCESS, 13 SKIPPED and 0 FAILURE;
  - I-0 on the pod passed 382 tests with CUDA.
- **The pod** is down (06:45:57Z), after its seal was read back. GPU spend was $3.20.
- **Reviews, each in a fresh context:**
  - the sheet's FIT, round 2, on #3407;
  - the D67 pre-run review, #3407 6677290978;
  - the exact-head review of the code and a claims check of this REPORT and the notebook row against the sealed evidence, recorded by `loop review` against the seal's sha256.

## Skills used

None applied.

## Still open

- **The one-track bar**, beating TRAIN's unigram and T2 on the whole human panel, is not met. The next unit is on it, by Casey's direction ("whatever it finds, until the bar is met").
- **The lead is (near).** It learns human text from the real targets and is at 86–88% of T2. It is ahead of T2 on oasst2's rows as a whole, though behind on oasst2's unseen rows. Its loss was still falling at step 8,192. The next unit's sheet weighs:
  - (near) with more steps and data (D3), the cheapest knobs;
  - more rivals in its average;
  - (near) on longer-context targets (`mode:2` or `mode:3`). This first measures each mode table's own panel top-1 on the laptop as its ceiling.
- **§3.1.** (m1) is a diagnostic. Adopting count-table targets as the training recipe needs its own decision card, which argues §3.1 (the sheet). (near) needs no card: it is an existing objective on the real targets.
- **The code review's findings** (APPROVE WITH NITS, exact head, fresh context). None changes a number above. The code stays at the pin, because the runs were trained there. The next unit takes them before any `mode:K` rerun:
  - **I-m is read, not enforced** (minor). The sheet's stop rule ("the run stops before training" if the split's scored count is not 6,865,470) held here only because the line was read: the trainer prints the count but does not refuse a wrong one. The fix is a refusal such as `--expect-split-scored N`, or a check in the job script.
  - **The self-count test** (minor). `fixed_targets_count_scored_positions_with_the_self_count_and_ties` would also pass an implementation that left the position out of its own count. The brute-force T2 test (at k = 1) and the backoff test's `[1, 2]` case (at k of 2 or more) pin the rule instead. The fix is a case where leaving the position out changes the winner.
  - **Nits:**
    - the binary's refusals (`--score-rows`, no `--out`, `--recall`) have no test;
    - `mode:+1` and `mode:01` parse (`config.json` writes `mode:1`).

## Plain summary

The project's model had never learned real human writing: on Wikipedia and OASST2 text its training stalled at the very start. This unit found out why.

Given one fixed answer per context (the word that most often follows the previous one), the same model, data and training code learned well. It got the next word right about 11 to 14 times as often as before, about as often as that "commonest follower" table itself. So nothing was broken in the data path. The problem was the training rule.

That rule pushes each guess away from its single closest wrong answer. In real text many words can follow, so even the best guess is "wrong" most of the time, and gets pushed off.

A different rule the trainer already had, which spreads that push over the eight closest wrong answers, made the model learn real text as written. It got the next word right on about 220,000 of 1.46 million held-out positions, against about 20,000 before. That is 86–88% of the commonest-follower table. On the conversational replies (OASST2) it beats that table, but only where the reply's recent words also appear in the training text; on new contexts it is still behind. A faster learning rate did not help.

The project's bar, beating that table on all held-out text, is not met yet. The next unit builds on the second rule. GPU time cost $3.20.

References #3407.
