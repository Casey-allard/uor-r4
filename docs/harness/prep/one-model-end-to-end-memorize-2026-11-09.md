# Prep sheet: T0, the memorize test (one-model-end-to-end, claude-345, 9 Nov 2026)

Lab: claude. Item: one-model-end-to-end (#3407). Order: claude-345. First test of the aligned design ([DESIGN.md](../../loop/tasks/one-model-end-to-end/claude-345/DESIGN.md), section 8).

TEST FITNESS: FIT WITH CHANGES, all seven applied (10 Nov, a fresh-context reviewer that had not seen the plan; it checked the sampling and reference rules against `in_space_long.rs`). The changes: a reply-level fit reading R2r, so row 2 cannot be reached by token-level misses; the rows checked in a fixed order with row 1 narrowed; the rows-file count in row 0; the trained windows named and counted; row 3 reworded to this recipe at this budget; R1 worded as the last complete reply of each window; the step-0 control claim dropped. The sheet's commit is recorded before any pod ships. After the FIT one instrument check was added, I-s, a smoke that can only end in row 0; no reading or bar changed.

PRE-RUN REVIEW: PASS WITH CHANGES (10 Nov, a second fresh-context reviewer; it read the sheet, the pod kit and the trainer's argument parser, and ran the store and syntax gates). Four minor findings. Applied: each row now reads each bar at its lower seed, so a split between the seeds is row 3 and cannot fall to row 0; the launcher says so loudly when a pod may be up with no watcher; a sealed failure is held one hour, not six. Accepted as it is: a watcher whose pod has gone keeps logging "no answer" on the laptop until the session ends it (no pod, no cost). The reviewer took the project chain from this sheet's "Why" and did not re-read `PLAN.md`.

**POST-LAUNCH SHAPE CORRECTION (10 Nov).** The T0 pods started from sheet commit `3d5dee5a487d7d638e75d7b1eef7a3eb0f3f5336`. This later documentation commit exposes the registered `Steps`, `Cost` and `Minimum test power` content under the headings required by the harness shape rule. It changes no bar, sample, seed, corpus, recipe, pod, read or inference rule; the running cells remain governed by the commit above.

**Why.**
- No recorded run has asked whether the assembled model can fit a small set of dialogues and say it back in free-running decode. Every reply reading so far comes from models trained on millions of tokens and scored on prompts they had not seen, where a failure to fit and a failure to generalise look the same.
- The design names two gaps: no learned transformation between a state and the next (1,112,576 learned values, 1,048,576 of them the token table), and no read keyed by state. If the first is real, the model should fail to fit even 1,000 dialogues. That is a prediction this test can refute.
- It costs about three GPU-hours and no new code.

## Question

Trained from a fresh start on 1,000 controlled dialogues only, with today's recipe unchanged, does the model reproduce the last complete reply of each training window exactly, end marker included, when it writes the reply by itself? That is 1,072 of the set's 3,589 replies: mostly the final, recall reply of each conversation.

## Base artifact

- **Recipe:** claude-301's (b64L), unchanged, as claude-322 ran it: `--objective nearest --retain-rates 1 --decay-mult 1 --lr 0.001 --lr-end 0 --train-as-served 1 --context 192 --width 256 --hamming-read 4 --batch 64 --micro-batch 16`. No ask heads, no pull, no carry, no turn memory, as in claude-322.
- **The pin:** `6fcf57f7822286e5b6bd7bcfe42d668116b23afa`, `origin/main` on 9 Nov. The trainer is built at the pin on each pod. No code changes.

## Steps

4,096 training steps at `--eval-every 512`, from a fresh start (no `--init-from`). At batch 64 that is about 231 passes over the 1,133 trained windows. Each seed then scores `models/step-004096` with the readings below.

## Data split

- **TRAIN and the scored panel are the same set, on purpose.** It is in the store at `results/claude/one-model-end-to-end-memorize-2026-11-09/t0-set/train/`: tokens `c1c779611d4f6c571579fca39778880020c5f1e4df699f1fcd5c32671c5a6f90` (122,037 ids), mask `3658e61246a124db19d95b7a254bbea00bdad25924920aeb613049856fe4cea9`, manifest `1000883dc160c5eff77d9991ee3aeb5caacf7e0782827be834cbecaaaeae67bd`. Uploaded and read back from the laptop on 9 Nov with these three hashes.
- **How it was made:** `m-world-recall-corpus out=NEW tokenizer=tokenizer.json conversations=1000 seed=345` at the pin, every other argument at its default (world v2, protocol 2). This is the generator behind `mw-balp`, so the dialogues are the project's controlled memory dialogues, labelled synthetic.
- **What it holds:** 1,000 conversations (348 key-value, 306 relation, 255 other, 91 copy), 3,589 replies, 40,419 scored targets. The longest conversation is 244 tokens, and 143 are longer than the 192-token window. The longest reply is 48 tokens, end marker included.
- **Its windows,** counted on the laptop by the trainer's own rule (`build_document_windows` at 192, the rule the scorer also uses for a panel at 192): 1,142 windows, 1,000 first chunks and 142 second chunks. The trainer keeps the windows with at least 4 scored targets: **1,133 windows holding 40,400 scored targets.** The other 9 windows (19 targets) are scored but never trained.
- **Nothing is held out of the set.** Every training window comes from the set through `--train-split`. DEV is a different file: the trainer's in-run DEV, `ft/dev` under the data root at claude-261's pins (`0919e90f…555b`, `a838bb1a…b078`), read for the curve only.

## Arms

One arm, T0, at seeds 1 and 2: two runs, one RTX 5090 Vast pod each. There is no control arm: the set is the model's own TRAIN, so an exact memory of TRAIN scores 100% by construction. An untrained model is not scored on this panel.

## Readings

At each seed, on `models/step-004096`, from `train-in-space --score-rows` with `--panel t0=<the set> --context 192 --reply-tokens 48 --samples 1200 --retain-rates 1`. Training ran with `--train-as-served 1`, so the scorer's forward is the served graph in float.

- **R1, exact replies (decides).** With `--sample-opening last`, each window's last reply opening that has at least 16 tokens of prompt is decoded greedily (`samples-t0-greedy-memory.json`): 1,117 samples by the count above. A sample is **complete** when its reference ends with the end marker (1,072 expected, the same at both seeds, since it depends on the set alone; 1,071 of them in trained windows). A reference cut by the window's end is not complete and is counted apart (45 expected). A complete sample is **exact** when the decoded ids equal the reference ids over the reference's whole length; what the model writes after the end marker is not compared. R1 is exact over complete, with both counts stated.
- **R2, token fit under teacher forcing.** From `rows-t0-memory.json`: the share of scored rows whose nearest token is the target, over all 40,419 rows, with both counts stated. It is pooled, and most rows are template words, so it is not a content reading by itself.
- **R2r, reply fit under teacher forcing.** The share of R1's complete samples whose every reference token is a teacher-forced hit in `rows-t0-memory.json`, matched by corpus position. With a sound decode, R1 and R2r are one reading by two paths: on claude-339's sealed DEV samples the free-running reply left its reference exactly at the first teacher-forced miss in 16 of 16. So a gap between them is a decode-path finding, and a low R2r is a fit finding.
- **Recorded only:**
  - the same decode from the first reply opening of 128 windows (the default `--sample-opening first`, `--samples 128`; 933 windows are eligible), a second scoring pass; if that pass fails, the cell stands and the failure is reported;
  - the counts of samples, complete samples and cut samples against the 1,117, 1,072 and 45 expected; a difference is reported beside R1;
  - for the samples that are not exact, where the reply first leaves the reference: token 0, a later token, or only the end marker;
  - how many decoded replies contain an end marker at all;
  - the training curve (`progress.jsonl`) and the DEV row at step 4,096;
  - on the laptop, if the sealed checkpoint exports: three of the set's conversations typed into `uor-chat --stack`, shown as text.
- **Not measured here:** tokens per second, the integer runtime's agreement with the float scorer, anything on prompts the model has not seen, and the output-side table gradient (the gradient-norm mode does not take this recipe's width and read flags).

## Bars

Fixed before any run, and not moved after a reading. The lower seed decides each bar: below, R1, R2 and R2r each mean the smaller of that reading's two seed values, so the three may come from different seeds.

- **90%:** R1 at or above 0.90.
- **95%:** R2 at or above 0.95.
- **90%:** R2r at or above 0.90.
- **800:** fewer than 800 complete samples at a seed means the instrument did not decode enough of the set.

## What each outcome unlocks

The rows are checked in the order 0, 3, 2, 1, and the first that matches owns the cell. With each bar read at its lower seed, rows 3, 2 and 1 cover every reading between them.

| Row | Reading (each bar at its lower seed) | Classified as | Next |
|---|---|---|---|
| 0 | at either seed: a job step fails; an input's hash is not the sheet's; the trainer's usable-window count is not 1,133; the run does not end with `step-004096`; scoring's last-opening pass fails; `rows-t0-memory.json` does not hold 40,419 rows; or fewer than 800 complete samples | NOT_RUN, instrument | repair, then one rerun under root `runs-2` |
| 3 | R2 < 0.95; or R1 < 0.90 and R2r < 0.90 | this recipe at 4,096 steps does not fit the set under teacher forcing | the address memory's first test is this same set at the same steps and seeds, with the same bars. If R1 ≥ 0.90, the report says the last replies were said back and the miss is in the others |
| 2 | R1 < 0.90 and R2r ≥ 0.90 (with R2 ≥ 0.95) | the replies are fit under teacher forcing and the free-running decode loses them: a decode-path defect | the report gives the first step at which the decode leaves the forced path; the next unit repairs that step |
| 1 | R1 ≥ 0.90 and R2 ≥ 0.95 | the assembled model fits and says back the set | the design's capacity claim is not shown by fitting; the address memory and the state-keyed read are judged on held-out prompts only |

A row-1 pass does not show the model can converse. A row-3 reading closes this configuration only: this recipe, at this size and step count, on this set (CHARTER §4.1).

**Prediction, written before the run** (a calibration record, not a bar; written before the fitness review and left as it was): row 3 at both seeds, with R2 between 0.60 and 0.90 and most inexact replies leaving the reference at token 0.

## Instrument checks

- **I-p:** a clean checkout at exactly the pin on the pod.
- **I-0:** `cargo test --release --locked -p uor-r4-float-train --features cuda --lib` passes with at least one test run.
- **I-3:** the set's three files, `tokenizer.json` (`d36d3e87…0f89`) and DEV's two files match their hashes before training; the run's `config.json` names the set's token and mask hashes, width 256, the Hamming read 4, batch 64, micro-batch 16, 4,096 steps, the seed, and no ask heads, pull, hold or carry; the trainer prints 1,133 usable TRAIN windows.
- **I-s:** before the run, a 32-step smoke on the same set is trained, checked by I-3 and scored with the deciding pass's own arguments at 8 samples. It proves the window count and both command lines in about two minutes. Its numbers are never read; a failed smoke is row 0.
- **I-t:** the run ends exit 0 with a numeric last loss and `models/step-004096` present.
- **I-r:** `rows-t0-memory.json` holds 40,419 rows, and every reference position of every complete sample is found in it.
- **Seal:** each pod's `run/` is sealed with a file list and SHA-256 sums, put in the store under `results/claude/one-model-end-to-end-memorize-2026-11-09/<root>/T0/s<seed>/`, and read back; the pod goes down only after the laptop reads the same sums from the store and the required files are in the list. A run is never written into an existing root (L-002).
- **The sheet in force:** this sheet's commit is written beside the pod kit and in the REPORT before any pod ships.

## Minimum test power

Two independent seeds are run and every bar is read at the lower seed. The deciding free-running reading contains 1,072 complete reference replies, well above the project's 200-row power floor, plus 40,419 teacher-forced target rows. The sheet stops if fewer than 800 complete samples can be decoded, because the intended comparison would then be underpowered or the scorer broken. No bar is inferred from one reply or one seed.

## Cost

Two Vast RTX 5090 pods, one seed each, at most $0.70/h each. The expected total is under $2.50, with a notice at $2.10 per pod and at two hours. The standing project cap of four pods and $8/h applies.

## Compute and stop rule

- Two Vast RTX 5090 pods, one seed each, at most $0.70/h each (a dearer offer is released at once), leased 3 hours and renewed by a watcher while the run moves. Expected about 100 minutes a pod: build and tests, the smoke, about 35 minutes of training at claude-322's measured 0.48 s a step, then scoring, which is about as long again because the scorer also decodes DEV and a second decode rule in the same pass (sealed, not read). Expected spend under $2.50 in all; a notice at $2.10 a pod and at two hours. Inside the standing caps of 4 pods and $8/h.
- A watcher is armed before the job ships (L-004). A healthy run is never cancelled for passing a notice.
- A failed step makes the cell NOT_RUN with the step named. One rerun is allowed, under `runs-2`.
