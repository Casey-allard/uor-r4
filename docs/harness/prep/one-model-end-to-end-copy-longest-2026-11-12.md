# Prep sheet: the longest-after-opening copy rule trained (one-model-end-to-end, written by claude-357, 12 Nov 2026)

Lab: claude. Item: [one-model-end-to-end](../../roadmap/items/one-model-end-to-end.md) (#3407). Written in claude-357, which runs nothing. **The run (unit claude-358) starts only when the owner says run.**

The build is [claude-357's](../../loop/tasks/one-model-end-to-end/claude-357/ORDER.md). The rule's laptop readings are [claude-356's](../../loop/tasks/one-model-end-to-end/claude-356/REPORT.md). The recipe is stage 3's, from the [place-read sheet](./one-model-end-to-end-place-read-2026-11-11.md).

## Question

Does a place-read model **trained** with the longest-after-opening copy rule say a remembered value more often than the same model trained with the one-token rule and only **read** with the new rule?

The rule: among a copy head's exact matches, the latest whose token before the key also equals the row's previous token; with none, the latest exact match; at the row that chooses a reply's first token, the latest exact match only. claude-356 served it on stage 3's three place-read models, which were trained without it, and recall rose on a fresh panel (245 to 268, 278 to 304 and 237 to 270 of 560). No model has been trained with it.

## Base artifact

- **Recipe:** stage 3's four commands, word for word, with one change: `--hamming-copy-longest 714,28`, the two token ids of the reply marker (the end of `Assistant`, then `:`). The flag does three things:
  - the copy heads choose by the rule at every row of a training window except a reply's opening row;
  - turn memory's leave rule asks the copy read by the same rule at later reply rows, so the reply table holds different keys (stage 3 logged `reply keys left to the copy read 160416 of 327257` on all four jobs);
  - the exported artifact carries `copy_longest` in its header and serves the rule as its own, finding opening rows by the marker.
- **The pin:** claude-357's merge commit on `main`, named in the launch note. The trainer and `fbs-snapshot` are built at the pin on the pod. The exporter, `raw-id-replies`, `row-picks` and the whole-word scorer run on the laptop at the pin.
- **The kit:** stage 3's `train-places.json`, SHA-256 `8d28571a614e4e038fcb52e2661787f38af62933dbf7ddad2ad64582c8a0a9b3`.
- **Compared with, not rerun:** stage 3's four sealed models, each exported at the pin from its sealed snapshot with `"hamming_copy_longest":[714,28]` added to its `config.json`. The weights are stage 3's; the reader's rule is the one the retrained artifacts carry. claude-357 made these four artifacts on the laptop two ways, by the exporter and by inserting the header field into stage 3's artifact, and the two agree byte for byte:

  | Model | Stage 3's artifact, SHA-256 | With the header field, SHA-256 |
  |---|---|---|
  | p-s2 | `ae4055a765d60ab44afa7aad0f927a6b321a3a1fc176daf8691acc144316ebe7` | `9d490ee971fc493e7ece7925fb8fe145a6e51d6c261c3f90830549ae739bcb39` |
  | p-s1 | `14c44f899863e437328df8c970430906f6b85d5bf81b7ef0d44ab26f83b72141` | `a1ce89a411e37046b34ab377c221be22e6eca54b027e0b16ee8a1a9c24927515` |
  | p-s3 | `22857c36b25596c4f35804ad233c1e7a24b4a29c784509d0c818e8a8f6c0c50f` | `33baf85eb076b1fd5115bc40e35577dead760abd1c672479c507df1381adb2b9` |
  | n-s2 | `f9fa134ebe1763ad48720ddb0fa352aced05109b17f21e83aae1604edd4eb14b` | `e0e3a6908d21687fcdae75e08d6ab7df4313d23cd3b99b3394e9b49772b67da4` |

## Data split

Written by `m-world-recall-corpus`, world v2, protocol 2. The hashes are of the files on the laptop.

| Set | Writer arguments | Tokens | tokens.u16 SHA-256 |
|---|---|---|---|
| TRAIN, no line | `conversations=60000 seed=13 recall=off` | 7,370,634 | `e25775f3aad4548e238736e02a3185e6c57062b8962a4147aafb655a6d865f32` |
| Scoring panel, for the row check only | `conversations=1000 seed=14 recall=off` | 121,894 | `b8ad9fe401226b6df8677d06b3f2ad60ea4ffdf6a0b8092970c69e57c4355fe4` |
| Reading panel, no line | `conversations=1000 seed=16 recall=off` | 121,601 | `cd0c9c8e9f29662e4541a2cc9bce9adbb130b17a10304a2e76a8a15503bffd8f` |
| Reading panel's twin, for the scorer's places | `conversations=1000 seed=16 recall=oracle` | 131,442 | `735dfad303e03d82ebe55b309948b1095b61656247db725b4a4c7c289d17eda3` |

- **The reading panel has been read by no model.** It was written on the laptop in claude-357 by claude-356's panel kit (generator SHA-256 `3974e5d5a952d4aaaf5fd1e64f1009bbf0f6725cb2e9972309b32cd88c13e97f`). The seed 15 panel is not used: claude-356 read the comparison models on it.
- Its places kit is `panel-places.json`, SHA-256 `261df2c01546231e6219de7a34d2a7e1f6a30df0a012b8b4e216626b283d4055`. Its prompts at the 256 window are `prompts-256/prompts-p.json`, 981 prompts, SHA-256 `139d3c8ac861ccf059e3ef8b17f4f04a4dde812875fba7ffb46342fe1ea99980`.
- **Left out:** rows 143 and 965, whose prompts equal a prompt of the seed 14 panel. Neither is a recall question. No prompt equals a seed 15 prompt.
- **578 recall questions** (the rows the whole-word scorer scores; it marks all 578 of the panel's own references right) and **401 other rows.**
- The panel is held out by the writer's seed and shares the world and its templates with TRAIN. It tests unseen conversations of the same kind, not open text.
- The reply marker stands before every reply opening and nowhere else in TRAIN (215,017 openings), in the trainer's DEV split (1,226) and in the scoring panel (3,586), counted on the laptop.

## Arms

One pod, four jobs one after another, in stage 3's order.

| Job | What | Why |
|---|---|---|
| p-s2 | stage 3's p-s2 command plus the flag | the first reading |
| n-s2 | stage 3's n-s2 command (the place read off) plus the flag | the control: whether a gain needs the place read (L-010) |
| p-s1, p-s3 | stage 3's commands plus the flag | whether p-s2's difference repeats |

- **Each retrained model is compared with stage 3's model of the same name,** exported with the header field. The two share the seed, the data, the 16,384 steps and the recipe. They are trained at two pins: stage 3's at `93e089ff256654e75a19641b7e2fbd2bc8e24524`, the retrained ones at claude-357's merge commit. **That they differ by the flag only rests on two things:**
  - **The commits between the pins.** Three touch the trainer's crate, in two changes, and none touches a crate it links (`uor-r4-training`, `uor-r4-core`, the lock file). `1966da68b` (claude-352) changed the `--panel-prompts` mode only: the prompt writer takes `--context` and `--sample-opening`. In a training run its two changed lines in `main` pass the same arguments as before. The other two, `367e05e04` and `183bfabd0`, are claude-357's own change. With the flag off its three new paths do nothing: the leave rule returns the old function on its first line, the copy read's choice gets no extra key term, and the marker check returns at once.
  - **A flag-off smoke at the pin (I-n),** whose start-up lines must equal stage 3's.
- Not run, and what that leaves unsaid: no arm with the copy read's rule on and the leave rule left as it was, so a result cannot say which of the two did what. No arm with the second token after a bare-space opening spared. No fourth seed. One control seed.

## Readings

**On the laptop, as the integer artifact,** exported as stage 3's was (`fbs-export --legacy-artifact --raw-gates --raw-transition --run-config config.json --context 8256`). Replies come from `raw-id-replies --mark-reply 1 --max-tokens 48` on the reading panel's 256-window prompts, with `--place-rows 1 --places panel-places.json` on the p artifacts and **no `--copy-rule`**: each artifact serves its own rule. The eight artifacts (four retrained, four of stage 3 with the header field) are read by one reader binary on the same prompts.

- **R:** recall replies the whole-word scorer marks right, of 578.
- **D = R of the retrained model − R of stage 3's model with the header field,** one for each job. **M** is the mean of D over p-s2, p-s1 and p-s3.
- **O:** of the 401 other rows, the replies equal to the reference token for token up to the end token. Its change is taken the same way as D.
- Reported, no bar: the control's D and O; each retrained artifact's R under `--copy-rule latest-exact` (what the rule adds at serving on a model trained with it); the nine-turn conversation of claude-356's REPORT, word for word, on retrained p-s2 beside stage 3's p-s2; the repeat-a-string rows; `fbs-toks --read` on the two p-s2 artifacts, in the same minute on one machine.

**On the pod, in float:** `train-in-space --score-rows` on each job's last checkpoint over the scoring panel at `--context 192`, greedy. It gives the trained model's choice at each scored row, for the row check (I-f). The pass refuses the 256 window, as in stage 3.

Not measured here: open text, reworded questions, the first token of a value, conversations longer than the context.

## Bars

Fixed before the run. They route the next step; none stops the line of work.

- **0.98, the row check (I-f):** an instrument bar, read before any reply.
- **29 of 578:** M at or above +29, which is 0.05 of the recall questions, rounded up.
- **Two seeds:** at least two of the three p jobs have D above 0.
- **10 other replies:** on no p job does O fall by more than 10.

## What each outcome unlocks

Rows are checked from the top, and the first that matches owns the cell.

| Row | Reading | Classified as | Next |
|---|---|---|---|
| 0 | any instrument check below fails, or an export fails | NOT_RUN, an instrument failure. No reading is classed by rows 1 to 3. | repair it and rerun under a new root, and under a new pin if the pinned code changed. A laptop-side check (I-o, I-w, I-c, I-h) is repaired and the readings retaken from the sealed checkpoints. **A failed I-f or I-u is split by side before any repair:** I-g's two tensor tests (the trainer's choice against the host's), I-u's two exports (the rule against no rule) and the split of differing rows by copy head and step that `row-picks` prints. A fault in the trainer's tensor rule is repaired in the trainer and the jobs rerun under a new pin; a fault in the export or the reader is repaired on the laptop and the readings retaken from the sealed checkpoints. |
| 1 | **Better:** M at or above +29, at least two D above 0, and no O down by more than 10 | training with the rule adds to reading with it | the flag joins this line's recipe and the retrained artifacts replace stage 3's as the line's models. It is called *replicated* only if all three D are above 0. Then the wrong first token of a value, which the rule does not touch. |
| 2 | **Worse:** M at or under −29 | training with the rule under this recipe loses recall | this configuration closes (CHARTER §4.1): the rule trained, with the leave rule following it. The rule stays a header field set at export on stage 3's models. The trainer's `reply keys left` count is read first: the leave rule is the first suspect. |
| 3 | **Level:** anything else | training with the rule is not shown to add to reading with it | stage 3's models stay the line's models, with the header field set at export, which needs no training. Nothing closes. Whether to train the rule further (more seeds, or the arm that separates the copy read's rule from the leave rule's) is a routing choice put to the owner with the per-job table. If M reached +29 and the two-seed or the other-replies bar failed, the REPORT names which. |

A Level reading closes nothing: see "Minimum test power" for what it can and cannot say.

**Prediction, written before the run and before the fitness review** (a calibration record, not a bar): D between −10 and +40 on each p job; M between 0 and +30; Level the most likely row, Better possible. Why small: the rule changed 26 to 33 of 560 recall replies when served, so training meets few rows where the two rules choose differently. What I cannot size: what the leave rule's change does to the reply table.

## Instrument checks

- **I-p:** a clean checkout at exactly the pin on the pod.
- **I-g, the tensor code's first run:** before anything trains, `cargo test --release -p uor-r4-float-train --features cuda --lib -- longe leave_rule marker_check place` on the pod. The output must show `the_longest_choice_takes_the_longer_match_except_at_an_opening` and `the_longest_rule_reads_what_followed_the_longer_match_except_at_an_opening` as `ok`: both were written and type-checked in claude-357 and have never run. **A failed test, a test build that does not compile, or either of the two tests missing from the output is row 0 and stops the run before any training.** The two tests must also be able to fail on the pod: in a scratch copy of the checkout (the pinned checkout stays clean), the one-line patch `docs/loop/tasks/one-model-end-to-end/claude-357/i-g-set-aside.patch` of the pinned checkout (SHA-256 `6d5536f61aee576fd4b218b48ffa6fa17bf1307214d0e536b9737c3a693de923`) takes the longer-match term out of the choice in `choose_sorted_asking`, and both tests must then fail. The patched copy must compile, as it does under the laptop's stub compiler; a copy that does not is row 0 too. A test that passes with the term out is row 0. The copy is deleted before anything trains.
- **I-3:** TRAIN's, the scoring panel's and the kit's files match the hashes above before training.
- **I-m, the marker:** each run's log holds the trainer's line `the longest rule's marker 714,28 stands before every reply opening and nowhere else: TRAIN 215017, DEV 1226`. The trainer refuses to start otherwise. Each scoring pass's log, the smoke's included, holds `[longest] the checkpoint reads by the marker 714,28: p has it before every reply opening and nowhere else: 3586`: the line is printed only when the checkpoint itself carries the marker, and the pass refuses a panel that fails the check. A scoring pass without the line is row 0.
- **I-s:** a 64-step smoke of p and of n with the flag, on the same inputs. Each must train; p's folder must hold a `place.jsonl` line for the smoke's checkpoint and n's must hold none, as in stage 3.
- **I-r, the rate, decided before the long run starts:** the rule adds one equality table for each copy head at every forward. From p's smoke rate, 16,384 steps are projected (stage 3's jobs trained in 1,997 to 2,052 seconds). If the projection is over 2.5 hours, the long run is not started: row 0.
- **I-k, the keys left:** each run's log holds `opening keys left to the copy read 259 of 13454`, stage 3's line, since opening rows keep the latest rule. Its `reply keys left to the copy read N of 327257` must have the same total and an N of at least 80,208, half of stage 3's 160,416. Under that is row 0: arm rk fell there, its copy read left 158 keys against 6,040. **An N equal to 160,416 is also row 0:** that is stage 3's count, what a leave rule the flag never reached would print. N is reported.
- **I-n, off is stage 3 at the pin:** a third 64-step smoke, stage 3's p-s2 command with no flag. Its eight `[memory]` lines must equal those of stage 3's p-s2 log character for character (among them `reply keys left to the copy read 160416 of 327257` and `TRAIN reply rows past their openings 2244751, named 1484964`); otherwise row 0. Its `step 0` line, the DEV read before any update, must equal stage 3's apart from the time: `dev top-1 4/14287 nearest 5 (openings 3/1226, commonest 161) top-8 27 admitted 526 dev_loss 2.78726 (token 1.00243 address 1.78483)`. Stage 3's p-s2 and n-s2, two jobs of one seed, printed that same line, so it repeats on one pod. If it differs here, the smoke is run a second time: two flag-off smokes that agree with each other and not with stage 3 are row 0; two that differ from each other show the line does not repeat on this pod, which is reported, and the `[memory]` lines and the commit list carry the claim. The flag-on p smoke's `step 0` line is reported beside it.
- **I-b, the bins road:** on p's smoke checkpoint, `fbs-snapshot`, `fbs-export` and `fbs-compare` agree as they do on `main`, and the artifact's header holds `"copy_longest":[714,28]`.
- **I-o, off is main on a real export:** each of stage 3's four sealed snapshots, exported with the pinned exporter and its own `config.json`, has the SHA-256 of stage 3's artifact; with the field added to the config, the SHA-256 in the Base artifact table. Both held on the laptop at claude-357's head.
- **I-w, the scorer:** the pinned whole-word scorer marks 578 of the reading panel's 578 references right.
- **I-c:** each run's `config.json` equals stage 3's for the same job field by field, apart from `hamming_copy_longest`.
- **I-h:** each retrained artifact's header holds `"copy_longest":[714,28]`, and `raw-id-replies` without `--copy-rule` gives the same replies as with `--copy-rule longest-after-opening` on rows where the model writes no reply marker of its own.
- **I-t:** each job ends exit 0 with `models/step-016384`, a snapshot and its `config.json`.
- **I-f, the row check, read first:** for each retrained artifact, `row-picks --context 192 --mark-replies 1 --float-rows` over the scoring panel: the share of scored rows on which the artifact chooses the trained model's token. **Under 0.98 on any of the four is row 0:** an export or port fault, not a reading of the rule. Stage 3's four models read 0.984 to 0.990 ([claude-354](../../loop/tasks/one-model-end-to-end/claude-354/REPORT.md)). This is also the first time the trainer's rule and the reader's rule meet on a trained model. It stands in place of stage 3's whole-reply bar of 0.10, which no model passes; that was put to the owner with the plan.
- **I-u, the rule is in the trained read, read with I-f:** each retrained snapshot is exported a second time with `hamming_copy_longest` taken out of its `config.json`, so that artifact reads by the latest rule. `row-picks` runs on both exports against the same float rows. **The export with the field must differ from the trained model on fewer rows than the export without it, on each of the four; otherwise row 0:** the trained read did not use the rule, or the reader's rule is not the trainer's. This is the check that fails if the flag reaches the config and the header and never changes what the model reads. It was calibrated on the laptop at claude-357's head on stage 3's four models, which were trained and scored under the latest rule, where the sign must be the other way:

  | Stage 3 model | Rows compared | Differ, as exported (latest rule) | Differ, with the field (the rule) | Differ at opening rows, both |
  |---|---|---|---|---|
  | p-s2 | 40,719 | 670 | 1,351 | 34 |
  | p-s1 | 40,719 | 520 | 1,227 | 34 |
  | p-s3 | 40,719 | 597 | 1,206 | 32 |
  | n-s2 | 40,719 | 504 | 1,139 | 31 |

  A model that reads by one rule and is served by the other differs on 609 to 707 more rows of 40,719, and on no more opening rows, where the two rules are one. So the check separates the two cases by several hundred rows. The first column equals claude-354's counts for the same four artifacts. The last column is `row-picks`' count at step 0, the first row of each reply.

- **Seal:** each job's `run/` is sealed with a file list and SHA-256 sums, put in the store under `results/claude/one-model-end-to-end-copy-longest-2026-11-12/run/<job>/<try>/` and read back. A run is never written into an existing root (L-002).

## Risks the build leaves open

- **The tensor lines of the rule are unrun.** The laptop's backend refuses tensor work. I-g is where they first run.
- **Two changes ride on one flag,** the copy read's choice and the leave rule. This run cannot separate them.
- **The row check has never been a registered bar.** Its 0.98 sits 0.004 under the lowest of four models.
- **A training window starts mid-conversation,** so a row near the window's start has no token before its earliest matches and falls back to the latest match more often than the served model does.
- **A served model that writes `Assistant:` inside a reply opens a new reply for the reader.** It happened on 1 of 984 replies on two of stage 3's four models.
- **The exporter does not compare the checkpoint's rule with the config's.** The snapshot does not carry the rule. The trainer refuses a resumed run whose marker differs, and I-u would show a mismatch.
- **Two trainer paths have no laptop test:** the marker check at the trainer's start and the scoring pass's check of its panel. Both need the trainer's device. The smokes run them first, and I-m requires the line each prints.
- **The scoring pass checks its `--panel` splits for the marker; the pinned fresh-v2 split is not checked.** This run does not score it.

## Minimum test power

- **Noise.** Stage 3's three p models, read with the rule on the seed 15 panel, gave 268, 304 and 270 of 560: a standard deviation of 20 replies between seeds of one recipe. It rests on three models, two degrees of freedom, so it is a loose figure. It was measured on the seed 15 panel, of 560, and is applied here to the seed 16 panel, of 578. It also takes no account of the pairing by seed, which would make D's spread smaller, so the power stated below is on the low side. If training the same seed with one more flag keeps none of that seed's character, D has a standard deviation near 29 replies and M near 17, which is 0.029 of 578.
- **The bar against it.** +29 is about 1.7 standard deviations of M. With no true effect, M reaches +29 about 1 time in 20, before the two-seed bar is applied.
- **What the run can show.** A large gain (+60 or more) reads Better nearly always, and harm of that size reads Worse. **A true gain of the predicted size reads Level more often than not:** by these figures M reaches the bar about 13 times in 100 at a true +10, about 1 time in 2 at +29, and about 74 times in 100 at +40, before the two-seed bar is applied. Level therefore says "under +29 on this panel at three seeds", not "no effect".
- The control has one seed and is not a powered reading.

## Steps

1. **On the laptop, before the pod:** claude-352's kit under new names, with the pin and the flag; the pinned builds; I-o and I-w at the pin; the four comparison artifacts at the hashes above; the set-aside patch of I-g at its hash.
2. **On the pod, before anything trains:** I-p, I-g with its set-aside, and I-3. Then the three 64-step smokes (p and n with the flag, p without) with I-m, I-s, I-r, I-k, I-n and I-b, and the scoring pass's own arguments at full size on p's flag-on smoke checkpoint.
3. **The four jobs of the Arms table,** each with its scoring pass and each sealed on exit.
4. **The pod's four checks,** then the pod goes down.
5. **On the laptop:** the exports with I-c, I-h and I-t. **Then I-f and I-u, before any reply is read.** Then the eight artifacts' replies on the reading panel, R, O, D and M, and the rows.
6. The reported readings.

## Cost

One pod at no more than $1.60/h. Stage 3 ran on one RTX 5090 at $0.908/h and its jobs trained 33 to 34 minutes each, with a scoring pass capped at 40 minutes after each. Expected: the tests, the smokes and four jobs in 4 to 6 hours, which is $3.60 to $5.50 at stage 3's rate and at most $9.60 at the $1.60/h ceiling. Ceiling: 10 hours, $16. One of the four pods and at most $1.60/h of the $8/h cap.

## Compute and stop rule

- One pod is launched and watched until it reaches SSH and starts its job. The watcher is started before anything ships (L-004). It renews the lease and never cancels a healthy run; time, spend and stall are notices only. The laptop is kept awake for the run; the chain and the seals run on the pod and need no laptop.
- A long run starts only if I-r projects it inside 2.5 hours. A run that started and is still healthy past that is extended and the extension noted; it is never cancelled for time.
- The jobs seal `run/` on every exit, a signal included. A scoring pass is capped at 40 minutes and never fails its job.
- The pod goes down when its own four checks pass: the chain ended with its seal, the required files are in the seal's list, the store's sums read from the laptop equal the pod's, and no GPU process is left.

## FIT

Read by someone who did not write this sheet; each verdict is recorded below word for word.

### First read, of the sheet at `619f9bc97cf9efcfd9033a955d33b57dbda327ff`: NOT FIT

A fresh `delegate:reviewer` subagent of the Claude Code desktop session, with the sheet and the code at that commit and nothing else:

> NOT FIT
>
> 1. BLOCKING. Instrument checks, I-k. Nothing can fail if the flag reaches the config and the header but never changes training. I-k passes at N = 160,416, which is exactly stage 3's count and what an unchanged leave rule would give. I-m only shows that the flag was parsed. I-c and I-h read the config and the header. I-f may not catch it either, because the rule changed only about 5% of recall replies. That silent fault would read Level, and Level's next step is "No further training of this rule". That is a mechanism call made on an instrument fault (L-007). To clear it, make the run fail as row 0 if N equals 160,416, or log how many training rows had their copy choice changed by the rule and require that count to be above 0.
>
> 2. BLOCKING. Arms and Base artifact. The sheet says the retrained models and stage 3's models "differ by the flag in training only", but no check supports that. The retrained models train at claude-357's pin. Stage 3's models trained at stage 3's pin. I-o covers only the exporter. Nothing shows that the pinned trainer with the flag off trains as stage 3's trainer did. A change to the trainer between the two pins would land in D and be read as the rule's effect. To clear it, list the trainer commits between the two pins and show that none touches the flag-off path, or add a flag-off smoke at the pin and compare its log with stage 3's.
>
> 3. Non-blocking. What each outcome unlocks, row 0. The text says a laptop-side check, I-f included, is repaired and the readings retaken from the sealed checkpoints. But an I-f failure can come from the trainer's tensor rule just as well as from the reader. To clear it, say that I-f below 0.98 needs a split by side first, using I-g's tests, before the pod-side or the laptop-side repair is chosen.
>
> 4. Non-blocking. Minimum test power. The 20-reply standard deviation comes from three models, so it is very loose: two degrees of freedom. "About as often as Better" holds only near +29. At a true +10, M reaches the bar about 13% of the time; at +40, about 74%. To clear it, state those two figures.
>
> 5. Non-blocking. Bars and row 3. Bars says no bar stops the line of work. Row 3 says "No further training of this rule", on a reading the sheet itself calls underpowered. To clear it, name row 3's next step as a routing choice for the owner, not a close.
>
> 6. Non-blocking. Cost. Six hours at the $1.60/h ceiling is $9.60, which does not fit "under $8". To clear it, base the expected figure on the $0.908/h rate, or raise it.
>
> The checked figures hold:
> - 0.05 of 578 is 28.9, rounded up to 29.
> - 578 + 401 = 979, which is the 981 prompts less rows 143 and 965.
> - The sample standard deviation of 268, 304 and 270 is 20.2. Times √2 that gives 29 for D, and divided by √3 it gives 17 for M.
> - +29 is 1.7 standard deviations of M, about 1 time in 20 one-sided.
>
> Every flag, test and log line the sheet names exists at this commit:
> - `--hamming-copy-longest` and `--score-rows` in train-in-space.
> - The two longest tests.
> - `--copy-rule`, including `latest-exact` and `longest-after-opening`, and `--mark-reply` in raw-id-replies.
> - `--float-rows` and `--mark-replies` in row-picks.
> - The three log lines in in_space_long.rs.

What changed for each point:

| Point | Change |
|---|---|
| 1 | I-k: an N of 160,416 is row 0. New I-u: the retrained read must agree with its export under the rule on more rows than with the same weights exported without it, calibrated on stage 3's four models. I-m: each scoring pass must print the checkpoint's marker, a line the scoring pass now prints. I-g: the two tensor tests must fail on the pod with the longer-match term taken out. |
| 2 | Arms: the two trainer commits between the pins and what each changes with the flag off. New I-n: a flag-off smoke at the pin whose start-up lines must equal stage 3's. |
| 3 | Row 0: a failed I-f or I-u is split by side before any repair. |
| 4 | Minimum test power: the three figures (13, 50 and 74 in 100) and the two degrees of freedom. |
| 5 | Row 3: nothing closes; further training of the rule is a routing choice put to the owner. |
| 6 | Cost: the expected figure at stage 3's rate and at the ceiling. |

### Second read, of the sheet at `8a1f935c743d21550bc29787cf1cb977c23f462c`: FIT

Another fresh `delegate:reviewer` subagent, which had not seen the first read's session, with the sheet, the code at that commit, stage 3's logs and the calibration receipts:

> FIT
>
> 1. Non-blocking. Arms, "The commits between the pins". The sheet says "Two touch the trainer's crate", but three commits in the range touch it: `1966da68b`, `367e05e04` and `183bfabd0`. The text then counts claude-357's two commits as one change ("The other is claude-357's own change"), so the claim itself holds. To clear it, write "three commits, two changes: 1966da68b, and claude-357's 367e05e04 and 183bfabd0".
> 2. Non-blocking. Instrument checks, I-g. The negative gate depends on "the kit's one-line patch", and this read cannot see that patch at the pin. To clear it, name the patch's path and SHA-256 in Steps 1, so the pod applies a known patch to the scratch copy.
> 3. Non-blocking. Minimum test power. The sd of 20 comes from the seed 15 panel (of 560) but is applied to the seed 16 panel (of 578). The null rate is about 1 in 25 one-sided (z = 1.76), which is close enough to "1 in 20". It also ignores the pairing by seed, which would likely make D's sd smaller. The direction is conservative, so the statement stays honest. To clear it, add one clause naming the panel difference.
> 4. Non-blocking. Instrument checks, I-u. The calibration table's last column is the step-0 bucket of `row-picks`, while the sheet calls it "opening rows". To clear it, say so in a footnote.
>
> No blocking point remains. Every bar and check can fail. A flag that never reaches training now fails three independent checks: I-k (N = 160,416), I-u (sign) and I-m (scoring line). Each lands in row 0, and every row names a next step.
>
> First-read points:
> 1. cleared (I-k equality is row 0; I-u calibrated and can fail; I-m scoring line; I-g set-aside)
> 2. cleared (commit list plus I-n flag-off smoke against stage 3's eight [memory] lines and step 0 line)
> 3. cleared (row 0 splits I-f/I-u by side)
> 4. cleared (13/50/74 in 100, two degrees of freedom stated)
> 5. cleared (row 3 closes nothing; owner routing)
> 6. cleared ($3.60–$5.50 at $0.908/h, $9.60 at ceiling)
>
> Facts:
> 1. Holds in substance. The range has three commits: 1966da68b, 367e05e04 and 183bfabd0. In train-in-space.rs, 1966da68b changes `context_applies(..., score.is_some() || panel_prompts_on)` and replaces the `--sample-opening` refusal with `prompt_cut_applies`. Both act the same in a training run (panel_prompts_on false, last_opening None). Its in_space_long.rs hunks touch only `PanelPrompt`, `split_prompts`, `panel_prompts` and tests. None of these runs during training. The sheet's "two commits" wording is loose (point 1).
> 2. Holds. `copy_names_ruled` returns `copy_names_by` on `None`. `check_run_markers` returns `Ok(())` at once without the flag. `room = time` and no key term are added when `longer` is `None` (flock_read_hamming.rs:3425, 3440–3443).
> 3. Holds. in_space_long.rs:1496 (reply keys), :9517 (checkpoint reads by the marker), :5834 (stands before every reply opening). Lines 5834 and 1496 carry the `[hamming]` and `[memory]` prefixes.
> 4. Holds. Each log has 8 `[memory]` lines, identical between p-s2 and n-s2. The two step 0 lines are identical and equal the line I-n quotes.
> 5. Holds. Plain 670/520/597/504, field 1351/1227/1206/1139, so the field export differs on 681/707/609/635 more rows (609–707). Step-0 counts are equal in both (34/34/32/31).
> 6. Holds. sd 20.2, D 28.6, M 16.5. P(M ≥ 29) is 0.125 at +10, 0.50 at +29 and 0.75 at +40. The null rate is 0.04.
> 7. Holds. 4–6 h at $0.908/h is $3.63–$5.45, quoted as $3.60–$5.50. At $1.60/h, 6 h is $9.60 and 10 h is $16.

Its four notes were taken after the read, and no bar, reading or outcome row changed: the three commits are named in Arms; the set-aside patch is a file of the pinned checkout, named with its SHA-256 in I-g and in step 1; the panel difference and the pairing are named in Minimum test power; the calibration table's last column is named as step 0.

TEST FITNESS: FIT - research, #3407, second read by a fresh non-author reviewer after a first NOT FIT (claude-357)
