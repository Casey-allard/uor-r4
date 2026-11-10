# Prep sheet: the memory-line run (one-model-end-to-end, claude-345, 10 Nov 2026)

Lab: claude. Item: one-model-end-to-end (#3407). Order: claude-345, redirected by the owner on 10 Nov ([ORDER.md](../../loop/tasks/one-model-end-to-end/claude-345/ORDER.md), "Redirect").

TEST FITNESS: FIT WITH CHANGES, all applied (10 Nov, a fresh-context reviewer that had not seen the plan; it read the scorer's sampling rule and the panel writer at the pin). The changes: the bar reading D is taken over the replies that follow a memory line with a value, in matched pairs across the two panels, not pooled; the twin enters the bar as G; arm o is exported and read on the laptop too; a row was added for a laptop that shows what the panel does not; row 0 counts matched pairs, and completion is reported as a reading of the model; the counts come from a tested Rust reader; today's checkpoint is named as confounded.

PRE-RUN REVIEW: PASS WITH CHANGES (10 Nov, fresh-context reviewers in two rounds: the first ran out of context part-way, so the laptop scripts and the scorer and export questions went to two more, and the lab closed two of the open points by reading the pin: the run config is read as loose JSON, so the 29 Oct config loads, and per-head copy keys are in the integer snapshot and record). Applied: the gate retries its store reads; a held pod is held one hour, not six; a failed renewal is retried in five minutes; the watcher runs under `caffeinate` and its screen is checked before anything ships; a pod whose price cannot be read is taken down; an unused helper is removed. Declined: stopping the run when the smoke scoring pass fails, for the reason in I-s. Left open, and covered in the first minutes of each run by the smoke scoring pass and the control passes, neither fatal: whether the scorer takes turn memory and copy keys from the checkpoint, and whether it loads the 29 Oct checkpoint.

**Why.**
- A model that talks already exists. On 10 Nov the 29 Oct (r-drop) model, served as the integer artifact `a7r-s2-c8256.uorlut` on the laptop, held a 14-turn conversation: every reply ended on the end marker and the sentences were fluent. What failed was the value: "My best friend is Maya." got "Got it, your friend is 47."
- The serving program already writes a memory line before a reply (`uor-chat`, `/memory auto`: `System: Memory: Maya.`). The model ignores it: fed `Memory: Maya.` in the long conversation it still answered "Your friend is 47." Its TRAIN was written with `recall=off`, so it never saw the line.
- claude-259 named this on 2 Nov ("a TRAIN with memory lines needs no new code"), claude-261 deferred it, and the line of work then moved to human text. No run has trained this recipe on a TRAIN that has the line.

## Question

Trained with the recipe that produced the talking model, on the same controlled world with the memory line written before each recall reply, does the served model say the value the line names?

## Base artifact

- **Recipe:** the 29 Oct (r-drop) run's own `config.json` (`results/claude/one-model-end-to-end-2026-10-29/training-460fe46e/s2/r-drop-s2/`), unchanged: `--objective cell --retain-rates 1 --decay-mult 1 --lr 0.001 --lr-end 0 --context 192 --width 256 --hamming-read 4 --batch 16 --turn-memory 8 --turn-memory-reply 1 --turn-memory-dropout 500 --samples 6 --reply-tokens 48`, 16,384 steps, seed 2. It is not trained as served: that switch came later, and the artifact that talks was made without it.
- **The pin:** `4265dc3d6c94015ba97bcb1505ed8d57e339a39a`, `origin/main` on 10 Nov. The trainer and `fbs-snapshot` are built at the pin on each pod. No code changes.
- **Today's model, for comparison:** checkpoint `models/step-016384` of that 29 Oct run, and its laptop artifact `results/claude/integer-long-context-2026-11-05/c288/a7r-s2-c8256.uorlut`.

## Steps

16,384 training steps at `--eval-every 2048`, from a fresh start, after a 64-step smoke on the same inputs. `fbs-snapshot` then reads `models/step-016384`.

## Data split

All four sets come from `m-world-recall-corpus` built at the pin, world v2, protocol 2, every other argument at its default.

| Set | Writer arguments | Size | tokens.u16 SHA-256 | response_mask.u8 SHA-256 |
|---|---|---|---|---|
| TRAIN with the line | `conversations=60000 seed=13 recall=oracle` | 7,964,285 tokens; 39,088 lines (34,460 values, 4,628 none) | `ca348c12be97724e4d5bae2a19b29200938b298724fd74f9f56abfb3c0809574` | `eb4e0a81df32783e87486d40202d3bc84f6538d567c6ce04aeadaf0f424bc32a` |
| TRAIN without the line (the twin) | `conversations=60000 seed=13 recall=off` | 7,370,634 tokens; no lines | `e25775f3aad4548e238736e02a3185e6c57062b8962a4147aafb655a6d865f32` | `d8ee0add4286293bf6471659f275e77346994accae41be53be8b47d8bd6e93d1` |
| Panel with the line | `conversations=1000 seed=14 recall=oracle` | 131,619 tokens; 641 lines (562 values, 79 none) | `12d7e9dfc0c34778556f34ee2c811b8388e90e84bf137bf0cfffdf00aed6d61d` | `81719cbef3550f53d5cdbda853643073572d77249563c22fdd0c7eecd0a1e3e4` |
| Panel without the line | `conversations=1000 seed=14 recall=off` | 121,894 tokens; no lines | `b8ad9fe401226b6df8677d06b3f2ad60ea4ffdf6a0b8092970c69e57c4355fe4` | `67eed533b50fb685d75875050cdd88205a34033d51f491ce19b36795ea75674b` |

- They are in the store under `results/claude/one-model-end-to-end-memory-line-2026-11-10/` (`corpus/`, `corpus-noline/`, `panel-line/`, `panel-noline/`).
- Each pair is the same conversations: both TRAIN sets have 215,017 turns and 2,459,896 reply tokens, and both panels have 3,586 turns and 40,719 reply tokens. Only the line differs.
- The panels are held out by the writer's seed (14 against 13). They share the world and its templates with TRAIN, so they test reading the line on unseen conversations of the same kind, not open text. Exact duplicate conversations between seeds were not counted.
- The line is the oracle's: the writer puts the true value. The owner was told "sieve-written"; the writer has only `off` and `oracle`. At serve time the line comes from `uor-chat`'s sieve, which on the laptop wrote the exact value for all four direct questions and for one of three reworded ones.
- In-run DEV is `ft/dev` under the data root (`w2-inputs/2026-10-25/driver-data`), as in the 29 Oct run. Its curve is recorded and decides nothing.

## Arms

Seed 2 each (the seed of the artifact on the laptop), one RTX 5090 Vast pod each.

- **m:** the recipe on TRAIN with the line. The one-variable arm.
- **mk:** arm m with `--hamming-copy-keys 2,1`: the first copy head keyed on the two preceding tokens, the second on one (claude-268). "Your friend is 47" is the one-token key's known fault (claude-266): "is" was followed by 47 earlier in the conversation.
- **o:** the recipe on the twin TRAIN without the line. The no-memory control (L-010): same writer, seed, recipe, pin and pod type as arm m.

## Readings

**On the pod, in float** (`train-in-space --score-rows CK --panel p=<panel> --sample-opening last --samples 1200 --context 192 --reply-tokens 48 --retain-rates 1`): each panel window's last reply opening is decoded greedily, and each sample is written with its prompt, reply and reference ids and its corpus position. Eight passes: arms m, mk and o, and today's checkpoint, each on both panels.

- **Recall replies.** In the panel with the line, the samples whose prompt holds the line's ids just before its last `\nAssistant:` and whose line names a value, not `none`. About 560 are expected: the panel has 562 value lines, and 639 of its 641 lines sit before a conversation's last reply (counted on the laptop).
- **Matched pairs.** Each recall reply is paired with the sample of the same reply in the panel without the line: the same reference ids, and the same prompt once the line is taken out. The line moves window boundaries, so some replies have no partner; they are counted and left out.
- **E(model, panel):** exact replies over the matched pairs. A reply is exact when the decoded ids equal the reference ids, end marker included. The same share over all samples of a pass is reported beside it and decides nothing.
- **D(arm) = E(arm, line) − E(arm, no line):** what the line adds for that model on the same replies.
- **G(arm) = E(arm, line) − E(o, no line):** the arm with its memory against the twin that never had one, served without one. This is L-010's control.
- **Completion:** the share of decoded replies that contain the end marker, a reading of the model and not of the instrument.
- **The reader.** The counts come from a small Rust reader with a unit test on a hand-made sample file, committed in this unit before any E, D or G is quoted. Nothing is counted by an ad-hoc script.

**On the laptop, as the integer artifact** (`fbs-export --raw-gates --raw-transition --run-config config.json`, then `--context 8256`, served by `uor-chat --stack ... --protocol 2`), for arms m, mk and o and today's artifact:

- **L(model):** a fixed 12-line script, written before the run (`/memory auto`, four statements, the four direct questions, three reworded questions). L counts the direct questions whose reply contains the stated value as a whole word and no other stated value. Today's artifact: 1 of 4 (only "47").
- The 14-turn conversation of 10 Nov with `/memory auto`, and the 12-line script, both word for word in the REPORT for each model. This is the reading the owner judges by.

Not measured here: tokens per second, other seeds, open text, and the integer runtime's agreement with the float scorer beyond what L shows. Today's checkpoint was trained on `mw-balp`, whose writer arguments are not on record, so its passes show where the model stood and decide nothing.

## Bars

Fixed before any run. They route the next step; none stops the line of work.

- **0.10:** D at or above 0.10 and G at or above 0.10 mean the model reads the line and is better for it.
- **3 of 4:** L at or above 3.
- **200:** a needed pass with fewer than 200 matched pairs is not read.

## What each outcome unlocks

Read for arm m and for arm mk separately. Rows are checked from the top, and the first that matches owns the cell.

| Row | Reading | Classified as | Next |
|---|---|---|---|
| 0 | a job step fails, an input hash differs, there is no snapshot, the export fails, or a needed pass has under 200 matched pairs | NOT_RUN, an instrument failure | repair it and rerun; the panel passes alone can be rerun from the sealed checkpoint |
| 5 | D under 0.10 and L at or above 3 | the laptop shows the value and the panel does not | a panel check: the matched samples are read by hand before the cell is classed |
| 3 | D under 0.10 and L under 3 | this recipe at 16,384 steps does not read the line | the value needs a read keyed on the line: a copy head keyed on the line's marker, the design's state-keyed read |
| 4 | D at or above 0.10 and G under 0.10 | the model leans on the line and is no better than the twin without memory | the line only replaces what the context already gave; test where the context cannot give it (a value outside the window) |
| 2 | D and G at or above 0.10 and L under 3 | the float model reads the line and the laptop does not show it | separate export, long-context carry and the sieve with an explicit `/memory VALUE` and a 256-context export; repair that path |
| 1 | D and G at or above 0.10 and L at or above 3 | the served model says the value its memory line names | seeds 1 and 3 of that arm; then the exact address memory writes the line in place of the oracle, and the recipe moves to human text |

A row-3 reading closes this configuration only (CHARTER §4.1). Arm o's own D is recorded: at or above 0.10 it would mean the line is read without training on it.

**Prediction, written before the run** (a calibration record, not a bar; written before the fitness review and left as it was): arm m reads the line in float (D between 0.10 and 0.30) and reaches L = 2, row 2; arm mk reaches L = 3 or 4, row 1; arm o is within 0.05 of today's checkpoint on both panels.

## Instrument checks

- **I-p:** a clean checkout at exactly the pin on the pod.
- **I-3:** TRAIN's and both panels' files match the hashes above before training; the tokenizer's and DEV's hashes are written to `inputs.sha256` and compared on the laptop with the 29 Oct run's.
- **I-e:** the line is encoded as served. Done on the laptop, 10 Nov: `uor-chat` serves `\nSystem: Memory:` as ids `201 53 91 353 970 28 379 970 2852 28`; TRAIN holds that exact sequence 39,088 times, the writer's own count of lines, and 39,046 are followed within ten ids by `\nAssistant:`.
- **I-c:** each run's `config.json` equals the 29 Oct run's field by field, apart from the TRAIN hashes and, for arm mk, the copy keys.
- **I-s:** a 64-step smoke on the same inputs, then one 8-sample scoring pass on its checkpoint. A smoke that does not train is row 0. A smoke scoring pass that fails is flagged and does not stop the run: the laptop reading does not need the scorer, and the panel passes can be rerun from the sealed checkpoints. Its numbers are never read.
- **I-t:** the run ends exit 0 with `models/step-016384` and a snapshot.
- **Seal:** each pod's `run/` is sealed with a file list and SHA-256 sums, put in the store under `results/claude/one-model-end-to-end-memory-line-2026-11-10/run/<arm>-s2/<try>/` and read back. A run is never written into an existing root (L-002).
- The library tests are not run on the pod: no code is changed, and the smoke runs both command lines.

## Minimum test power

One seed per arm: a first reading, and no milestone claim rests on it. A row-1 arm is rerun at seeds 1 and 3 before the notebook records it as more than one seed. Each E rests on at least 200 matched pairs, the project's floor; about 560 are expected. L has four questions and is not a powered reading: it is the product reading, shown word for word, and the panel carries the count.

## Cost

Three pods at no more than $0.70/h each (cheapest offer $0.563/h on 10 Nov); a dearer pod is released and taken down at once. The 29 Oct run took about 4.5 h per seed. Expected: about 5.5 h and $3.10 per pod, $9.40 in all. Ceiling: 8 h per pod, $16.80. At most $2.10/h of the $8/h cap and three of the four pods.

## Compute and stop rule

- The watcher is started for each pod before anything ships (L-004). It renews the lease and never cancels a healthy run; time, spend and stall are notices only.
- The job seals `run/` on every exit, a signal included. Scoring passes are capped at 40 minutes each and never fail the job.
- A pod goes down when its own four checks pass: the chain ended with its seal, the required files are in the seal's list, the store's sums read from the laptop equal the pod's, and no GPU process is left. A failed job's pod goes down once its logs are sealed.
