# Prep sheet: the address table's missing seeds, and the snapshots of its six models (one-model-end-to-end, claude-366, 13 Nov 2026)

Lab: claude. Item: [one-model-end-to-end](../../roadmap/items/one-model-end-to-end.md) (#3407). Unit: [claude-366](../../loop/tasks/one-model-end-to-end/claude-366/ORDER.md), PR #3956. The runs go ahead on this lab's own pre-run review (CHARTER §5.1), inside the caps, and are logged for the owner.

## Question

Served as integer artifacts on the laptop, do the models trained with the address table give more right everyday replies than the saved models of the same seed?

- **What exists.** claude-346 trained three address models at 16,384 steps: `e8-s2`, `e8-s3` and `token-s2`. In float they give more exact everyday replies than the same recipe without the table (of 520: 119 without; 142 and 177 for `e8`; 141 for `token`). Its bar on all 1,108 replies read "not called" (M +29.5 against +30), and its sheet names seed 1 of `e8` as the next run.
- **What is missing.** Each arm needs three seeds beside the three saved models, and no address model has a snapshot, so none can be exported.
- **This sheet covers two kinds of pod visit:** the three missing training jobs, and a short snapshot job for the six address checkpoints and `b-s2`. The snapshot job runs in two passes, each on its own short-lived pod: the four checkpoints already sealed, then the three new seeds when their lanes have sealed.

## Base artifact

- **Recipe:** claude-358's p arm with `--address-memory ARM`, word for word as in [claude-346's sheet](one-model-end-to-end-address-memory-2026-11-13.md) (its Base artifact and Arms).
- **Training pin:** `f941a33fd1a4eea751705d1cd6021af35452a4b7`, claude-346's pin, so the three new models are trained by the code that trained the three sealed ones.
- **Snapshot pin:** `b036715ac6982df1b8479274703d2272e8fb6966`, this unit's branch with main merged in, pushed as the branch `claude/address-snapshot-pin`. The ORDER's Done-when 1 tests pass there on the laptop. Writing it here changes no question, arm, reading or bar. The pin moved once on 13 Nov, from `78705230ef176f325e890d901ce881893c1d6d9b`: the one commit between them takes the fresh code reads' findings (FIT, second read).
- **Found on the laptop on 13 Nov, before any address artifact was read:** `b-s2`'s sealed snapshot exports to p-s2's artifact byte for byte (SHA-256 `f4667934627e99d709533f30505058fdf07912ddf3cfc9ade6ded5f415e8c680`). At seed 2 the two pins train the same model, so `b-s2` is p-s2 and its served replies are p-s2's.
- **Sealed address models** (claude-346, float scoring pass, replies equal to the wanted reply of 1,108): `e8-s2` 218 (`run/e8-s2/try-3`), `e8-s3` 254 (`run/e8-s3/try-3`), `token-s2` 219 (`run/token-s2/try-4`), under `results/claude/one-model-end-to-end-address-memory-2026-11-13/`.
- **Two pins, and the check between them.** The saved p models were trained at claude-358's pin `f694d162d9b04d1226fd17c0a81825f00e9da007`; every address model is trained at `f941a33f…`. claude-346 ran the saved recipe with no address table at `f941a33f…` and seed 2 (`b-s2`, sealed at `run/b-s2/try-4`): 188 exact of 1,108 in float, claude-358's figure for p-s2. That check is one seed and in float. So `b-s2` gets a snapshot with the address checkpoints and is served beside p-s2 as a same-pin reading, with no bar.
- **Compared with, not rerun:** claude-358's three saved models, as exported then.

  | Model | Artifact, SHA-256 |
  |---|---|
  | p-s2 | `f4667934627e99d709533f30505058fdf07912ddf3cfc9ade6ded5f415e8c680` |
  | p-s1 | `9a209883ff74cdeb6936fc11714d4dd4bb63947775609754c6a5f678e39df9bd` |
  | p-s3 | `9615060f4db98a4cc5b21220e454428b816cda1d47256cf85091963692e83d20` |

## Data split

| Use | Where | `tokens.u16` SHA-256 |
|---|---|---|
| TRAIN (the model's own corpus, no memory line) | `results/claude/one-model-end-to-end-memory-line-2026-11-10/corpus-noline/train` | `e25775f3aad4548e238736e02a3185e6c57062b8962a4147aafb655a6d865f32` |
| The pod's scoring panel (generator seed 14, 1,108 prompts) | `results/claude/one-model-end-to-end-memory-line-2026-11-10/panel-noline/train` | `b8ad9fe401226b6df8677d06b3f2ad60ea4ffdf6a0b8092970c69e57c4355fe4` |
| The bar's panel | the same writer at `seed=23`, written on the laptop after the ORDER's commit (13 Nov 2026, 14:12:07Z) | committed to this unit's task folder (`panel-23.sha256`) and pushed before any served reply of any model is read, so "read once" can be checked |

- The place kit and the tokenizer are claude-346's (`8d28571a…`, `d36d3e87…`). The pre stage also reads OASST2 for two of its smokes; no reading here uses it.
- **No model has read the seed-23 panel,** and nothing is fitted on it. The counted table that every model is served with (claude-365) was counted on TRAIN.
- The panel shares its world and templates with TRAIN: it tests unseen conversations of the same kind, not open text. 229 of 236 everyday user turns of the seed-22 panel stand in TRAIN word for word.

## Arms

Training, one job a pod, each after its pod's pre stage:

| Job | Pod | Address | Steps | Seed |
|---|---|---|---:|---:|
| `e8-s1` | a | `e8` | 16,384 | 1 |
| `token-s1` | b | `token` | 16,384 | 1 |
| `token-s3` | c | `token` | 16,384 | 3 |

The snapshot job (after the served form is built): for each of the six address checkpoints, `fbs-snapshot` at the snapshot pin writes the trunk snapshot, and beside it a rows file holding the table's rows in the whole steps the trainer's own scoring rounds them to. The snapshot names that file by arm, shape and SHA-256. `b-s2`, which has no rows, gets the same snapshot without them, compared byte for byte with the one its training job wrote at claude-346's pin. Nothing trains.

**Changed from this sheet's first form (13 Nov).** It promised a served float readout "with the rows set on the trunk". The snapshot tool as built refuses a float readout for an address checkpoint: its readout path does not add the rows, so the readout would describe the trunk without its table. No float readout is taken. What the readout was for, the row check and the check that the rows are read, is done on the laptop from the picks each training run's own scoring pass sealed (Readings; Instrument checks, I-d).

- **Rules check (§3).** As in claude-346's sheet: the `e8` address is the E8 root system the project's cells use; the read is 32 row lookups and adds a layer and token; no squashing function; no state is multiplied with a table. `token` is a learned table addressed by a fixed hash of the input token: it was the `e8` arm's control, and whether it may be served in a product is the owner's reading of §3.1.
- Not run: a no-address job (claude-358's three are the pairs); the `sign` arm; a fourth seed.

## Readings

**On the laptop, as integer artifacts,** exported as the saved models were (`fbs-export --legacy-artifact --raw-gates --raw-transition --run-config config.json --context 8256`) and served with the counted table as merged by claude-365 (`raw-id-replies --mark-reply 1 --max-tokens 48 --whole-prompt`, `asks-floors.json`, `--place-speak 1 --place-serve-again 1`).

- **G:** everyday replies of the seed-23 panel (not a repeat row, not a recall or never-told question) that are the reference or a reply TRAIN gives the same user turn word for word, and end.
- **D** = G of an address model − G of the saved model of the same seed. **M** = the mean of D over an arm's three seeds.
- **The row check:** the share of the scored rows on which the artifact picks the token its own float model picks, beside the saved models' share (lowest 0.9860). The float picks are the ones the model's training run sealed from its scoring pass (`score/STAGE/rows-p-memory.json`), as claude-358's check reads them.
- **Reported, no bar:** told values stated; never-told questions answered "not told"; replies that never end; the nine-turn conversation word for word; tokens a second with and without the rows; `b-s2`'s G beside p-s2's (the same recipe at the two pins).
- **Where this runs, and the rule (CHARTER §3.6).** §3.6 allows serving the artifact on a client device and says "panel scoring, qualification and milestone evaluation of an exported artifact remain GPU research compute". These readings are the product served on the laptop with its replies read and counted by kind, as claude-356 to claude-365 did in merged units. They choose which models the line builds on next. They qualify nothing: no milestone claim, no qualification bar and no charter bar rests on them. Whether a count by kind of served replies is "panel scoring" under §3.6 is the owner's reading; it is named in the REPORT and put to the owner with a proposed wording. If the owner reads it so, the counts are redone on GPU compute before anything is built on them.

**On the pod, in float (reported, no bar):** each new job's exact-reply count of 1,108 from `--score-rows`, its last `train_loss_mean` and `dev_loss`, and its `address.jsonl` census.

## Bars

The ORDER's, fixed before any served reply of an address model is read.

| Outcome, for an arm | Word |
|---|---|
| M ≥ +10, told values stated not down by more than 3 a seed, never-told "not told" not down by more than 2 a seed | adds |
| M < +3 | does not add at this configuration |
| anything else | not called |

- **Row 0.** An address artifact whose row check is more than 0.010 under 0.9860 is an export or reader fault, not a reading. A new job that fails I-n or I-a is that job's row 0.
- **M needs all three seeds.** A failed job, a job at its row 0 and an artifact at Row 0 have no D. The job is rerun under a new try root, or the export repaired, before that arm's M is read; until then the arm has no word.
- If both arms add, the line's models become the arm with the higher M; within 3 of each other, the arm with fewer replies that never end.

## What each outcome unlocks

Keyed on the two arms' words together; one row applies.

| `e8` | `token` | Next |
|---|---|---|
| adds | does not add, or not called | The `e8` models become the line's models. Next: the state-keyed read on top of them. |
| adds | adds | The ORDER's rule names the arm. If `e8`: as the row above. If `token`: the `token` models are the line's research models, and nothing built on them is served as a product before the owner's §3.1 reading; the `e8` models, which also add, stay the set that may be served meanwhile. Both results go to the owner with that question. |
| does not add, or not called | adds | Nothing is served before the owner's §3.1 reading. The next arm addresses after a mix of the state, so the state's address carries what the token hash does. |
| not called | does not add, or not called | Each not-called arm's replies are read by kind to find which everyday turns moved; no fourth seed before that. |
| does not add | not called | As the row above, for `token`. |
| does not add | does not add | The float gain on exact replies does not reach served replies. An instrument question first (§4.2): the row check and the cells' agreement with float. A negative on this configuration only (§4.1). |

An arm with a Row 0 artifact has no word: the export or the reader is repaired first, and no reply of that artifact is read.

**Prediction, written before the runs** (a calibration record): `e8-s1` 200 to 260 exact of 1,108 in float; `token-s1` and `token-s3` 200 to 240; M for `e8` between +5 and +25; M for `token` between 0 and +15; `e8` models keep more replies that never end than the saved models.

## Instrument checks

- **Training pods:** claude-346's pre stage, unchanged (I-p, I-g, I-3, I-s, I-n, I-z, I-a, I-r and the scoring pass's arguments). Each long job repeats I-n and I-a on its own log and files.
- **Snapshot pod.** I-p: a clean checkout at the snapshot pin. I-g: every `e8_address` test passes on the pod's GPU, among them the three this unit adds (the tensor decode names the served cell; the token cells are the served hash; the served image holds the rounded rows), and the export's refusal test. I-c: each checkpoint's files have the sums of its seal, and its sealed run ended 0. I-n: each address snapshot's note matches its rows file (arm, 8-wide blocks, 240 cells, six layers, width 256, 23,592,988 bytes, SHA-256); `b-s2` has neither a note nor a rows file, and the job says whether its snapshot equals the one its training job wrote.
- **I-d, changed (13 Nov): on the laptop, not the pod.** Each address artifact's row check is taken beside the same snapshot exported with all-zero rows of the same shape. If the trained rows are read, and at the cell and layer the trainer wrote them to, the artifact agrees with its float run's sealed picks at the saved models' level, and the all-zero export agrees on fewer rows. An artifact below the row check's floor is not read for G, D or M until the cause is found.
- **Done on the laptop, 13 Nov, at real size.** `b-s2`'s sealed snapshot exported with an all-zero table of the recipe's shape (23.6 MB) gives the same 120 replies, id for id, as without it, served with the counted table. The exporter refuses a note without its rows, rows without a note, and a rows file with one byte changed. Reading a prompt goes from 3,392 to 2,977 tokens a second with the table, and writing a reply from 1,222 to 1,121, both timed at the same moment.
- **Laptop.** The integer cell of a block equals the float trainer's on shared vectors, walls and ties included; the read adds the row the address names and no other; an artifact with no rows reads byte for byte as today. Each fails with its rule set aside.

## Risks the build leaves open

- The float trunk works a block's cell out in 32-bit floats and the reader in integers. On a near tie they can name different cells; the row check measures what that costs.
- GPU training is not bit-exact between runs. The new seeds are trained at claude-346's pin for that reason.
- The seed-23 panel shares its templates with TRAIN, so G says the model answers turns of the kind it trained on, not that it answers open questions.
- The `token` arm fills its rows differently from `e8` and has no band, as claude-346's sheet says.
- G counts a reply right only when TRAIN gives it for that turn word for word; a right reply in other words counts as wrong for every model alike.
- **The counted table was set on the saved models.** claude-365 chose `asks-floors.json`'s floors while reading the saved models on the seed-22 panel. On told values and never-told questions, the two guard readings, that favours the saved models; an address model can lose a guard for that reason and not for its own.
- **The pins differ** (Base artifact). At seed 2 they train the same model to the byte (found 13 Nov). Seeds 1 and 3 have no such check: no run of the saved recipe exists at claude-346's pin for them.
- **Where the read sits in a layer is not pinned by a laptop test.** The laptop's tests pin the cell rule and that the named row and no other is added. Only the row check on trained models tests that the reader adds the rows at the point of the layer where the trainer does.
- **The artifact's schema with a table, `uor-r4.lut-stack/11`, is a research schema.** It has no stack contract, and an artifact with a table is about 43 MB against 19.3 MB.

## Minimum test power

On the seed-22 panel the three saved models' G is 137, 134 and 136 of 236: a spread of 3. The address models' spread is not known in served form. In float the two `e8` seeds differ by 35 exact everyday replies of 520, a standard deviation near 25 from two values (one degree of freedom, so itself a rough figure). Scaled to 236 turns by the square root of the count, that is about 17 a seed, and about 10 for M over three seeds. Two errors follow:

- With no true gain, M ≥ +10 comes up about one time in six.
- With a true gain of +10, M < +3 comes up about one time in four, so "does not add" is wrong that often.

That is enough to choose which models the line builds on next, and not enough for a milestone claim.

## Steps

1. Launch pod a (`try-1`), watch it reach its chain, then pods b (`try-2`) and c (`try-3`). Each pod's watcher is armed before anything ships. `uor-pod status` is read first.
2. Each pod runs pre, then its one job; each stage is sealed to `results/claude/one-model-end-to-end-address-seeds-2026-11-13/run/STAGE/TRY` and read back. A pod goes down when its chain ends with every job 0 and every seal 0 and the four checks pass.
3. The snapshot pin and the snapshot job's kit (`snapjob-366.sh`, `chainsnap-366.sh`, `sealsnap-366.sh`, `upsnap-366.sh`, and the watcher with its gate `gate-snap-366.sh`) take their own fresh read. Then pass 1: one pod, session `claude-366-s`, `try-4`, takes the snapshots of `e8-s2`, `e8-s3`, `token-s2` and `b-s2` from claude-346's seals. Each model is sealed to `…/snap/MODEL/try-4` and the job's record to `…/snap/record/try-4`, each read back; the pod goes down when the chain ends with the job 0 and every seal 0 and the four checks pass. Pass 2, when the three lanes have sealed: the same on a new pod, `try-5`, for `e8-s1`, `token-s1` and `token-s3`. One model's failure does not stop the others; it is recorded and that model is not read.
4. On the laptop: export and the row check; the seed-23 panel is written and its SHA-256 committed and pushed; then the readings, the REPORT and the notebook row.

**Run note, 13 Nov 2026, 16:15Z (after the lanes were launched).** Lane b's pre on `try-2` did not pass: its scoring pass on the e8 smoke reached the job's 2,400-second limit (exit 124) on a slow host; the same pass took about 20 minutes on lanes a and c. This is an instrument failure and no reading of a model: `token-s1` was not trained on that pod. The failed pre is sealed at `…/run/pre/try-2` and read back on the laptop; the pod was held, then taken down. Lane b is rerun on a new pod as `try-6` (session `claude-366-b`; `try-4` and `try-5` stay the snapshot passes'). The job, its limit and its pin are unchanged; a second failure of the same kind is rerun again under a new try. In step 3, `token-s1` is therefore taken from `try-6`. A checkpoint whose lane has sealed when pass 1 is launched may go in pass 1 and not pass 2. Only which pass takes a checkpoint changes; no question, arm, reading or bar does.

**Run note, 13 Nov 2026, 17:05Z.** Lane c sealed `token-s3` at 17:04Z, about an hour before lane b's rerun can seal `token-s1`. Pass 2 (`try-5`) therefore takes `token-s3` alone, and a third pass (`try-7`) takes `token-s1` once `try-6` has sealed. The snapshot job, its pin and its checks are the same in every pass; a pass costs about $0.10 (pass 1 cost $0.09). Only the number of passes changes; no question, arm, reading or bar does.

## Cost

Three RTX 5090 pods, each refused above $1.60/h. A pod runs pre, about 57 minutes of training at the measured 0.21 s a step, and a scoring pass of up to 40 minutes: 1.7 to 2 hours. Then the snapshot job: two passes of one pod each, under an hour a pass (a build, the tests, then for each checkpoint a fetch of about 80 MB and a seal of about 70 MB), each pod refused above $1.60/h. $4 to $13 in all; claude-346 paid $2.97 and $3.29 for its two lanes. Three training pods at once are at most $4.80/h; with the first snapshot pass beside them, four pods at most $6.40/h, inside the caps of 4 pods and $8/h. The three lanes as launched cost $3.14/h together.

A snapshot pod whose chain ends unsealed, or whose four checks fail, is held for the session and not taken down. Its watcher renews the lease for at most 2 hours (1 hour when the failure itself is sealed), then stops; the last 2-hour lease runs out and the reaper takes the pod. Left unanswered, a held pass therefore costs at most 4 more hours, $6.40 at the $1.60/h ceiling. Every input of a snapshot pass is a seal in the store, so a failed pass is rerun on a new pod and loses nothing.

## Compute and stop rule

Vast through `scripts/pod/uor-pod`, sessions `claude-366-a`, `claude-366-b`, `claude-366-c` for the lanes and `claude-366-s` for the snapshot job. A snapshot pod's watcher gives its time notice at 90 minutes and its spend notice at $3. The snapshot launch checks that the watcher is alive (its screen, its opening line for this pod, no exit line) before anything ships; a pod with no watcher after 30 seconds is released and taken down. Two limits of the launch are known and watched by hand: the $1.60/h refusal comes only once the pod is up, because the price is known only then; and a launch that ends with a `Creating` line and no pod name (exit 5) can leave a pod up with no watcher, so the script prints `uor-pod status` and the session takes that pod down at once. A stall, spend or time notice is never a stop: a healthy run is extended and the notice is recorded. A failed pre stops its pod's job. A failed scoring pass is said and does not fail its job. No run writes into an existing root (L-002).

## FIT

A fresh reviewer (claude, fresh context, 13 Nov 2026) read this sheet at `ed38134a6da31b243826f376d67c28087eeecdf0` beside the ORDER, the kit's job and chain scripts and CHARTER §3. It ran nothing. Verdict: **fit with changes.** It checked the arms against the kit and the bars against the ORDER line by line, and both match: the three lanes run `pre` then their one job, the job refuses any pin but `f941a33f…`, and the seeds, steps and address arms are the table's.

**This first FIT covers the three training lanes.** The snapshot job's pin and kit did not exist then; their own fresh read is the second FIT below.

Its nine findings, all taken in this revision:

1. *Medium.* The outcome table's rows overlapped and left a gap. It is now keyed on the two arms' words together, "neither adds" is both "does not add", and the case where both add and `token` is higher is written.
2. *Medium.* The saved models and the address models were trained at different pins, unsaid. Base artifact now gives both pins and claude-346's check (188 at seed 2), and `b-s2` is served beside p-s2.
3. *Medium.* §3.6 names panel scoring of an exported artifact as GPU research compute, and the sheet cited nothing for reading served replies on the laptop. Readings now gives the rule's words, what these counts are used for, the merged precedent, and that the reading of §3.6 is the owner's.
4. *Medium.* No rule for an arm with fewer than three readable seeds. Bars: M needs all three.
5. *Medium.* The snapshot pod's kit was not in what was read. This FIT is scoped to training.
6. *Low.* The power paragraph did not say how the float spread was scaled, and gave one error of two. Both are written.
7. *Low.* The seed-23 panel's SHA-256 was left for the REPORT. It is committed and pushed before any served reply is read.
8. *Low.* The counted table's floors were set while reading the saved models. Named in Risks.
9. *Low.* 1.3 hours a pod was low. Cost now counts pre and the scoring pass; the kit's stale header is corrected.

**Second FIT: the snapshot job.** A second fresh reviewer (claude, fresh context, 13 Nov 2026) read this sheet at `631009f444e301e3b573d3670b67dcb77e439b80` beside the snapshot kit (`snapjob-366.sh`, `chainsnap-366.sh`, `sealsnap-366.sh`, `upsnap-366.sh`, `watch-snap-366.sh`, `gate-snap-366.sh`) at the pin `78705230…`. It ran nothing. Verdict: **fit with changes.** Its six findings, all taken:

1. *Medium.* The launch did not check that the watcher was alive before it shipped the job. It now does (Compute and stop rule), and takes down a pod that has no watcher.
2. *Medium.* `b-s2`'s comparison with its training job's snapshot was skipped in silence when the seal holds none. The job now writes "not compared" and the path it looked at.
3. *Low.* A fetched seal's sum list was checked, but not that it lists the tar. The job now refuses a list that leaves the tar out.
4. *Low.* The sheet did not say what a held pass can cost. Cost now says, and the snapshot pod's hold is 2 hours where a lane's is 6.
5. *Low.* The price refusal comes only after the pod is up, and an exit-5 launch can leave a pod unwatched. Both are inherited from the lanes' launcher; Compute and stop rule now names them and that they are watched by hand.
6. *Low.* A copy of the run's source pin was unchecked. A failure there is now a not-run record for that model.

Three more fresh reads on 13 Nov covered the code at the pin `78705230…`: the integer rows module, the float side, and the reader, format and exporter. Each passed with low items and none found a wrong result. The commit that takes their findings is the one the pin moved by: a size worked out with checked steps, three more refusal tests, a test that the served step reads the rows in every layer (it fails when the read is cut to layer 0), the snapshot tool's refusal to write over a snapshot, and doc lines. That commit was not read again by a reviewer; the laptop's tests at the new pin and I-g on the pod check it, and the unit's exact-head review reads it before the merge.

TEST FITNESS: FIT - research, #3407 (claude-366), a fresh non-author read at ed38134a of the three training lanes and a second fresh non-author read at 631009f4 of the snapshot job and its kit, every finding of both taken
