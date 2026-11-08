# Prep sheet: (b64L) on claude-322's × 6 capped mix with a mixed pair of copy keys, `--hamming-copy-keys 2,1` (one-model-end-to-end, claude-323, 8 Nov 2026)

Lab: claude. Item: one-model-end-to-end (#3407). Order: claude-323. The step is Track A of Casey's "Both in parallel" (#2028 6715318106), under "Yes, pipeline first" (#2028 6709947526): the run is judged by the served-reply score.

TEST FITNESS: FIT WITH CHANGES, round 1 (a fresh non-author review, 8 Nov, before any pod; partial, stopped at its context budget). No blocker. Applied:
1. **"Lifted" has a margin set before the run:** a rise of at least 5 over claude-322's count at the same seed. A rise of 1 to 4 is "near" and reads row 5. The same seed shares init and window order, but the training paths part after the first update.
2. **A lift must be selective:** the asked-for value's rise must be larger than the other value's. Otherwise the row is 3c, "continuation lifted, selection not".
3. **Row 2b** reads a complete row at one seed only.
4. **Row 4 records stopping** beside it.
5. **"Lifted at a seed"** is defined before "lifted at both".

TEST FITNESS: FIT WITH CHANGES, round 2 (a second fresh non-author review, 8 Nov, before any pod). No blocker. Every combination of readings lands in exactly one row. Every claim round 1 did not reach was verified against its source:
- claude-322's readings;
- the 417,421 windows;
- the exporter, header and `complete` code on `main`;
- the pinned binaries. Both are dated after claude-268's `3641753d9` and claude-269's `000b35c6b` (3 Nov) and hold the `copy_keys` field.

Applied:
1. **A mean d below −58,212** turns rows 1, 2, 2b, 3 and 3s into a decision review on the cost.
2. **"Stopping held" is stated as a floor, not seed-paired,** with each seed's stops given beside claude-322's.
3. **The asked-for count includes the "both" rows,** so a first line holding both values still counts. Selectivity compares the asked-only and other-only counts.

**Why.** This is claude-322's decision review (row 5, #3407 6723513106): **recall is the binding gap.**
- claude-322's (b64L-d3mwcap6) stops on 109 and 86 of 256 without collapse (9 and 6), but **no reply is complete** at either seed.
- **Its replies copy the start of a two-token value and drop the rest.** The most common stopped reply at seed 1 is " Your friend's name is basalt lo." for the answer "basalt loggia".
- **The first-line count is low.** The reply's first line holds the asked-for value in 14 and 2 rows, and the prompt's other value in 11 and 6.
- **Its two copy heads both read with a one-token key,** the token before the copy position (the default). A one-token key reads "my friend is" and "my number is" alike (claude-259). After " lo" it has no key that says which value it is in the middle of.
- **A two-token key can continue a multi-token value, but alone it was right less often.** claude-267's oracle on DEV / p-w gave k1 3,859 / 12,371 and k2 3,506 / 10,971, and claude-266's (k2) arm never beat (b)'s copies.
- **The best rule in claude-267's oracle was k2 backed off to k1: 4,388 / 13,990**, about 14% over k1 alone. A mixed pair is the read's form of that rule: one copy head on key 2, one on key 1.
- **The mixed pair was built through training, export and the integer serve** by claude-268 and claude-269, with parity tests at `[2,1]` and `[1,2]`. It has never been trained on a served run.

## Question

At claude-322's recipe, mix, pin, seeds and steps, with only `--hamming-copy-keys 2,1` added:
- does the reply's first line hold the asked-for value more often than claude-322's at the same seed, at **both** seeds?
- does any reply become complete (an EOS stop on exactly an accepted answer)?
- do stopping, collapse and the held-out text hold against claude-322?

**What the comparison can say.** Against claude-322 at the same seed, **one thing changes:** the copy keys. The mix, recipe, pin, seed, steps and window order are the same, so a seed-paired difference is the keys'.
- It cannot say which head's key carries a change. A lift could come from the key-2 head alone, or from the pair.
- It reads one scale, one mix and one run length. A negative closes this configuration only (§4.1).

**What "complete" asks.** `served-reply-panel` counts a row complete only when it stops on EOS and the whole reply, after the protocol's trim, equals one of the row's accepted strings (`answers.accepted`, for example `basalt loggia.`; `accepts` in `crates/uor-r4-core/src/answer_oracle.rs`).
- **So a reply in a sentence never counts,** whatever value it holds. " Your friend's name is basalt loggia." would score 0.
- **This is why the first-line count is this run's first reading:** it reads the value whatever the reply's form. "complete" reads value and form together.

## Base artifact

- **The pin:** `ff5156bcc9539aa5bccd0319df3d96ee5dd5a71f`, as in claude-322. Every pod is brought up with `--ref` at the pin, and the job refuses any other source.
- **The pin trains mixed keys:**
  - `--hamming-copy-keys` is parsed by `train-in-space` and stored as `LongConfig.hamming_copy_keys` (`in_space_long.rs`);
  - its validation refuses the flag with `--hamming-read 0`, with `--hamming-copy-key` also given, or with one key for every head;
  - `HammingReads::with_copy_keys` (`flock_read_hamming.rs:744`) builds the reads. The read is device-generic candle code, so the CUDA run takes the path the CPU parity tests take;
  - the checkpoint's Hamming sidecar writes a `copy_keys` tensor (`flock_read_hamming.rs:883`);
  - `config.json` records `"hamming_copy_keys": [2, 1]` and leaves `hamming_copy_key` out at 1 (the pin's own test, `in_space_long.rs:13731`).
- **The recipe:** claude-322's (b64L), with one flag added:
  `--objective nearest --retain-rates 1 --decay-mult 1 --lr 0.001 --lr-end 0 --train-as-served 1 --context 192 --width 256 --hamming-read 4 --hamming-copy-keys 2,1 --batch 64 --micro-batch 16 --eval-every 1024 --steps 16384`, seeds 1 and 2.
  - `--hamming-read 4` has 2 copy heads (the first half) and 2 type heads, so the list has two keys: the first copy head keys on 2 tokens, the second on 1.
  - `--train-split`, `--train-sha256` and `--train-mask-sha256` are claude-322's.
- **The binary:** `train-in-space`, built on each pod at the pin, with its sha256 recorded.
- **The served artifact:** `cpu-fbs-export` (`724a3b78…`) on the laptop, from each seed's sealed `models/step-016384`, with `--raw-gates --raw-transition`.
  - It was built on 7 Nov, after claude-268 (#3624, 3 Nov). It reads the sidecar's `copy_keys` tensor of shape [heads / 2] and refuses one with both `copy_key` and `copy_keys` (`fbs_snapshot_cpu.rs` 801–816 on `main`).
- **The scorer:** `served-reply-panel` at bin `be3d4cb0…` (#3828), unchanged. Its artifact reader takes the header's `hamming.copy_keys` (`stack/format.rs:192`).

## Data split

**TRAIN is claude-322's `d3mwcap6`, unchanged,** read from the store at `results/claude/one-model-end-to-end-b64l-d3mwcap6-2026-11-08/train/`:
- `tokens.u16` `0273173e16e805706a9b3e816c3de46b824a53daf91be3b812be360f8e8e75a7`;
- `response_mask.u8` `da506e850145f55b746929e108e0fd240ddf753f6a90b23976c98e159a1cb293`;
- `manifest.json` `442ead2c92ae9a2e6b0b500ebbc12eb6e472f91c96465b6c1347fa80ac232ae9`;
- 417,421 usable windows (claude-322's I-3 read this count at both seeds).

Its build, census and provenance are as claude-322's sheet ([`one-model-end-to-end-b64l-d3mwcap6-2026-11-08.md`](one-model-end-to-end-b64l-d3mwcap6-2026-11-08.md), "Data split" and "Repetition and exposure") records them. mw-balp's licence and generator are still **not recorded**.

**The other inputs are unchanged from claude-322's sheet:** DEV (claude-261's `ft/dev`), the no-memory panel (claude-295's four folders, 1,460,697 rows, identity `5f88a5b3…`), and the served panel `m2-fresh256` (inputs `1de2fc28…`, labels `3b5c5477…`, plan `0ed1ab98…`, tokenizer `d36d3e87…`).

## Arms

| arm | TRAIN | copy keys | steps | source |
|---|---|---|---:|---|
| (b64L-D3), sealed | D3 | 1, 1 | 16,384 | claude-301 `runs-2/s{1,2}`; served scores claude-309 |
| (b64L-d3mwcap6), sealed | the mix | 1, 1 | 16,384 | claude-322 `runs/s{1,2}`; served scores and grades in `local/claude/claude-322/` |
| **(b64L-d3mwcap6-k21)** | the mix | **2, 1** | 16,384 | this sheet's run unit, seeds 1 and 2 |

One arm to train, at two seeds, one pod per seed. The steps are fixed: no gate cuts them.

**The one-key reference, claude-322, on the same measures:**

| seed | EOS stops | collapse (EOS-stopped) | complete | pairs | first line: asked / other / both | held-out d against (b64L-D3) | DEV at 16,384 |
|---:|---:|---:|---:|---:|---:|---:|---:|
| 1 | 109 | 9 | 0 | 0 | 14 / 11 / 0 | −15,456 (behind) | 7,041 |
| 2 | 86 | 6 | 0 | 0 | 2 / 6 / 0 | −25,627 (behind) | 6,958 |

## Readings

At seeds 1 and 2, from `served-reply-panel` on (b64L-d3mwcap6-k21)'s exported `step-016384`, on claude-322's measures:
- **The first-line count** (the first reading), by `firstline.jq` (`b15c778d…`, unchanged): the rows whose reply's first line, lower-cased, holds only the asked-for value, only the prompt's other value, and both. It is a substring match against the accepted answers with a trailing full stop dropped, so the full two-token value must be there. A reply that starts with `\n` has an empty first line.
- **Stops:** EOS, BOS, cap and context counts, of 256.
- **The panel's own measures:** rows complete (of 256) and pairs both complete and distinct (of 128).
- **Collapse:** among the EOS-stopped rows only, the count of the most common reply text, beside the number of EOS-stopped rows it is over; distinct reply texts and the most common reply over all rows.
- **Reply length:** the median of `rows[].reply_ids` lengths.
- **EOS stops per stratum** (length2, length4, length8, reassert, update).

**The held-out pairs:**
- `scale-probe grade` with `train=` the mix, on all 1,460,697 rows, with no `clean=`.
- **Against (b64L-D3),** claude-301's sealed grades `e17a13548d09304d901a93683cadb6c7695200261e86770e319003bfc976523a` and `3170c779def7ef376fdc11c96232f3f6ddb2809c436855121fd24be8ed7ab5a1`. "Behind" means the pooled verdict `behind`, or `mixed` with at least one seed `behind`, as in claude-322's sheet. Both seeds' positions and the mean pooled d are recorded.
- **Against claude-322's one-key grades,** `24d69af0455c9b18ab819bd73355e30ffa2f56db191f83e899ce0d8d4b092f7e` and `4407e2c2baf24ec94b1ba823316c109868f1fdd4322b4667ff136792518d3ec3`: the keys' own held-out cost. Its verdict, positions and d are recorded.
- **Descriptive,** each against the pinned grades `read-323.sh` checks before it pairs: claude-315's (b64L-d3mw), claude-319's (b64L-d3mw3), claude-321's (b64L-d3mwcap), claude-313's (b64L-mw), and the mix's own T2 and unigram.

Recorded beside them, descriptive only:
- **The role-header count:** rows whose reply contains `Assistant:`, and `User:`, among EOS-stopped rows and over all 256.
- **The DEV curve** (`dev_nearest.hits` every 1,024 steps) at both seeds, against claude-322's at both seeds (its seals are on the laptop) and the earlier arms' seed 1.
- the loss at the last step, within the arm only.

## Bars

Fixed before any run. They are claude-312's, unchanged.
- **Stopping:** at least **128 of 256** EOS stops at **both** seeds. claude-322: 109 and 86.
- **Collapse:** a seed is collapsed if, among its EOS-stopped rows, one reply text covers **at least 64**. claude-322: 9 and 6.
- **The panel's bar:** at least 104 rows complete and at least 52 pairs at both seeds. Every model scored so far has 0.
- **The held-out cost:** `behind` (b64L-D3) on the pooled verdict. claude-322 was behind at both seeds, mean d −20,541.5, inside claude-318's −58,212 ceiling.

**The decision rules, not bars,** fixed before the run (FIT rounds 1 and 2). At each seed, a **rise** is this run's count minus claude-322's at the same seed (asked 14 and 2, other 11 and 6, both 0 and 0):
- **The asked-for count** is the rows whose first line holds the asked-for value: asked only plus both. claude-322's is 14 and 2.
- **Lifted at a seed:** the asked-for count rises by **at least 5** (at least 19 at seed 1, at least 7 at seed 2).
- **Near at a seed:** it rises by 1 to 4. **Flat at a seed:** by 0 or less.
- **Selective at a seed:** the asked-only count rises more than the other-only count. Without it, a lift is the key-2 head continuing two-token values, not picking the asked-for one. The "both" count is recorded beside every row.
- **"Stopping held"** means at least **86** EOS stops at both seeds (claude-322's lower seed) and neither seed collapsed. Otherwise the keys cost stopping, and every row records it.
  - **It is a floor, not seed-paired** (FIT round 2): it guards against the keys breaking the stop, and is not the question. Each seed's stops are also given beside claude-322's at that seed (109 and 86).
- **The held-out cost turns a positive row into a decision review** (FIT round 2). If the mean pooled d against (b64L-D3) is below claude-318's ceiling of −58,212, rows 1, 2, 2b, 3 and 3s do not unlock their next unit directly. Their next unit is a decision review between the gain and the held-out cost. Every row records the mean d.
- **Why a margin:** the same seed gives the same init and window order (the keys draw no random numbers, and `epoch_order` does not read them). But the two runs part after the first update, because the key-2 head reads another token. So a seed-paired difference still carries seed-level noise, and the margin is there to keep a one- or two-row change out of row 3.

## What each outcome unlocks

Read top to bottom; the first row that matches decides.

| # | reading | the next unit |
|---|---|---|
| 0a | a seed NOT_RUN after its rerun, for an instrument fault (a gate, out of memory, setup, a lost pod) | **`blocked`.** Repair the instrument and rerun the seed under this sheet. No reading on one seed. |
| 0b | a seed NOT_RUN after its rerun, for a mechanism cause (a non-finite loss at a recipe claude-322 trained at both seeds) | **`diagnostic stop`,** a result about the mixed keys on this mix. Next: a decision review in the REPORT. |
| 0c | I-c, I-xc or I-k fails, or an export or score fails on the laptop with both seals present | **`blocked`, no pod.** Repair the export or scorer and re-read from the seals. |
| 1 | the panel's bar met at both seeds | **The served model answers.** Next: the trunk trains with these keys on the mix under its own sheet, at the next scale. |
| 2 | at least 1 complete row at both seeds | **The first exact answers.** Next: a decision review in the REPORT between the lever's next step (the keys at the next scale, or longer) and the reply's form, from the complete rows by stratum. |
| 2b | at least 1 complete row at one seed only (this includes the panel's bar met at one seed) | **The first exact answer, at one seed.** Next: a decision review between a third seed under this sheet and row 2's next step. The first-line rule is still read and recorded. |
| 3 | lifted and selective at both seeds, and stopping held | **Consistent with the mixed pair lifting the asked-for value's recall, without costing stopping.** The value is in more first lines, but not in the bare form `complete` asks. Next: a decision review in the REPORT between the reply's form (the bare value the panel accepts) and more of the lever, with the first-line count by stratum as its reading. |
| 3s | lifted and selective at both seeds, and stopping not held | **The mixed pair lifts recall but costs stopping.** Next: a decision review in the REPORT between the keys and the stop, with both seeds' stops beside claude-322's. |
| 3c | lifted at both seeds, and not selective at a seed | **Continuation lifted, selection not:** more first lines hold a full value, but the other value rose as much. Next: a decision review in the REPORT on the selection signal (which value a copy head reads), with the asked and other counts by stratum. Stopping is recorded beside it. |
| 4 | flat at both seeds | **At this scale and mix, the mixed pair does not lift the first-line recall.** It closes this configuration, not the lever (§4.1). Stopping held or not is recorded beside it. First the REPORT classes it as instrument or mechanism (§4.2), from I-3, I-k, I-c and I-xc. Next: a decision review in the REPORT, led by the decode-time read (claude-321's Still open), with recall as its question. |
| 5 | any other reading: lifted at one seed only, or near at a seed | **Mixed or near: no direction is read.** The REPORT gives both seeds' first-line counts and rises beside claude-322's and capped × 3's. Next: a decision review between a third seed under this sheet and row 4's levers. |

A seed that is NOT_RUN reruns once, sealing to the new root `runs-2/s{1,2}/` (L-002). A second NOT_RUN reads by row 0a or 0b, by the class of its last cause.

## Instrument checks

Registered before any pod goes up. They are claude-322's, with the copy keys added:
- **I-0:** `cargo test --release --locked -p uor-r4-float-train --features cuda --lib` passes on each pod at the pin, with at least one test run, none failed, and claude-300's three micro-batch tests passing by name. The pin's copy-key tests are in this suite.
- **I-p, the pin:** the job refuses any SHA argument but the pin (exit 2), and a source that is not a clean checkout at the pin (exit 3).
- **I-3:**
  - the mix's three files match the shas above, and the gate tests refuse claude-321's capped × 3 tokens, mask and manifest, the uncapped × 3 mask and manifest, D3's, and claude-314's × 1 files;
  - the panel folders and the DEV files match claude-306's pins;
  - each run's `config.json` names the mix's shas, width 256, read 4, **`hamming_copy_keys` `[2, 1]` with no `hamming_copy_key` other than 1**, batch 64, micro-batch 16, 16,384 steps (128 for the smoke), the pod's seed and the recipe's flags, with no turn memory. **The gate tests refuse claude-322's real one-key `config.json`,** a config with no copy keys, one with `[1, 2]`, and one with a copy key of 2 beside the list;
  - the trainer's "usable TRAIN windows" line in the smoke reads **417,421**. Any other count makes the arm NOT_RUN at that seed (row 0a).
- **I-1, the smoke:** 128 steps at `--eval-every 32`, at the pod's seed. It exits 0 with a finite loss and writes `models/step-000064`.
- **I-m, from the smoke:** the largest `mem.log` sample inside the smoke's window is below 90% of the card. claude-322's smoke peaks are the reference: the second key adds one token's lookup per copy position, not a tensor of the batch's size.
- **I-f, from the smoke:** the peak after the step-64 row is at most the peak before it plus 1,024 MiB.
- **I-t, recorded only:** r = (train_seconds at step 128 − at step 64) / 64; W = used + 1.05 × r × 16,384 + 300 s, written to `it.txt`.
- **The watcher:** as in claude-322: MAXSEC 14,400 s, a budget of PRICE × 4 h, PRICE at most $0.70/h. Neither limit stops anything (D67). A stall of 30 unchanged checks pauses the lease.
- **I-5:** the no-memory scoring's `.i5` reads `same` on every sub-panel, and DEV's rows agree across the four per-panel processes.
- **I-4:** each graded checkpoint scores the 1,460,697 rows with identity `5f88a5b3…`.
- **I-k, the copy keys, on the laptop at serve (new):** for each seed, the run's `config.json` (`hamming_copy_keys`), the export's snapshot (`hamming.copy_keys`) and the served artifact's header (`hamming.copy_keys`, from the length-prefixed JSON after the 8-byte magic) all read `[2,1]`. Otherwise that seed is not scored, and the reading is row 0c.
  - **It fails on a one-key record:** on claude-322's seed-1 files all three read `null` (8 Nov).
  - **What it cannot show before the run:** a passing record. No mixed checkpoint exists yet. The export and serve parity at `[2,1]` are claude-269's tests.
- **I-c, the scorer controls, on the laptop before any reading,** with `served-reply-panel` at `be3d4cb0…`, unchanged: claude-308's seed-1 (b64L-D3) artifact against `d02ec432…` (0 EOS stops), and claude-313's seed-1 (b64L-mw) artifact against `5175c29b…` (253 EOS stops).
- **I-x, the export:** each artifact's sha256 is recorded, and `served-reply-panel` accepts it.
- **I-xc, the export control, on the laptop before any reading:** `cpu-fbs-export` (`724a3b78…`) re-exports claude-313's sealed seed-1 `step-016384`, and the artifact's sha256 must equal `1c9ae43c…`.

## Steps

1. The ORDER; this sheet's FIT; the D67 pre-run review of the job, chain, seal, watch, seed and read scripts.
2. On the laptop:
   1. The job is claude-322's `job-322.sh` with three changes:
      - `SHAPE` adds `--hamming-copy-keys 2,1`;
      - I-3's per-run line adds `.hamming_copy_keys == [2, 1] and (.hamming_copy_key // 1) == 1`;
      - its comments name claude-323.
      `check_train`, the fetch (`MIXR` stays claude-322's store root, the input), the window gate, `end()` and the traps are claude-322's.
   2. Its gate tests and mutations, `bash -n` on every script, I-c and I-xc.
3. `uor-pod status`, recorded. The cross-lab caps must hold with these two pods included: at most 4 pods and $8/h. Then, per seed, `seed-323.sh SEED`:
   - one RTX 5090 Vast pod, at most $0.70/h, brought up with `--count 1 --ref` at the pin and leased under session claude-323;
   - its watcher started in a detached `screen` (`c323-wS<SEED>`) before any script ships;
   - the job, chain and seal scripts shipped, their hashes checked on the pod against this sheet's, then the chain started.
4. On each pod, in order: I-p, I-0, the build, the inputs and I-3; the smoke with I-m and I-f, then I-t; (b64L-d3mwcap6-k21) for 16,384 steps; the no-memory scoring of `step-016384`, one panel per process, then I-5.
5. Seal each pod to `results/claude/one-model-end-to-end-b64l-d3mwcap6-k21-2026-11-08/runs/s{1,2}/`. The root must be empty first (L-002). Read the seal back, then take the pod down.
6. On the laptop, `read-323.sh`:
   1. `ctrl`: I-xc and I-c; then `serve`: export each seal's `models/step-016384`, I-k, I-x, the panel, the role headers and the first-line count.
   2. `grade`: `scale-probe grade` with `train=` the mix, then the pairs, with claude-322's one-key grades pinned.
   3. `dev`: the DEV curves, with claude-322's at both seeds.

## Minimum test power

- **The first-line count** is the deciding reading, and it is noisy. claude-322's seeds read 14 and 2 asked; claude-321's capped × 3 read 25 and 76.
  - **That is why it is read seed-paired.** Each seed here shares claude-322's seed, mix, recipe, init and window order, so the keys are the only input that changes. The training paths still part after the first update, so the pairing narrows the noise but does not remove it.
  - **So a lift needs a rise of at least 5 at both seeds, and must be selective** (the asked-for value rising more than the other). A rise of 1 to 4 at a seed is "near" and reads row 5. Even a lift reads as "consistent with" a recall lift, and it leads to a decision review, not to a scale-up.
- **Stopping** is a count. claude-322's seeds read 109 and 86, a spread of 23. A seed within 32 of 128 (96 to 160) is reported as "near the bar".
- **Collapse:** a reading within 16 of 64 (48 to 80) is reported as "near the bar", with the count of EOS-stopped rows it is over.
- **The held-out pairs** use `scale-probe pair`'s band. The scorer is not bit-reproducible across pod instances (claude-317's I-q: 3 of 3,188 DEV hits), inside the two-seed spread.
- **"complete"** has been 0 on every model scored. One complete row at both seeds reads row 2. That threshold is deliberately low: the first exact answer is the event this panel has never recorded.

## Cost

- **claude-322's measured runs:** about $3.0 for both pods.
- **Here:** the second key changes which tokens one copy head keys on, not the step's tensor shapes, so the step time should match claude-322's. About **$3** for both. At the $0.70/h ceiling, claude-322's seconds cost about $3.4.
- **The budget notice, not a cap:** 2 pods × 4 h × $0.70 = $5.60. A healthy run is never cancelled for a self-set budget. The ORDER's bound is $8.

## Stop rule

- **I-p, I-0, I-3 or an input fetch fails:** the pod trains nothing. Fix the instrument and rerun at a new root.
- **The smoke, I-m, I-f or I-3's window count fails:** the arm is NOT_RUN at that seed.
- **An out-of-memory error, a non-finite loss, or a trainer exit other than 0:** NOT_RUN at that seed, classified as instrument or mechanism before it is called anything (§4.2).
- **A pod lost mid-run, or one that never reaches SSH within `uor-pod up`'s 900 s:** an instrument NOT_RUN. It is deleted, and one retry runs on another machine. (claude-322's sheet said 1,800 s; `uor-pod up` waits 900 s, as its REPORT records.)
- **The no-memory scoring fails or I-5 fails:** the served readings stand, and the held-out readings are re-scored from the seal.
- **Always:** the job seals on every exit, including a TERM, INT or HUP, and the watcher takes the pod down after `seal=0`.

## Changes from claude-322's sheet

1. **The recipe** adds `--hamming-copy-keys 2,1`. TRAIN, the pin, seeds, steps and bars are claude-322's.
2. **The question and the outcome map** are rewritten for recall: the first-line count, seed-paired against claude-322 with a margin of 5 and a selectivity rule, decides rows 3 to 5; "complete" decides rows 1, 2 and 2b.
3. **I-3** checks the copy keys in `config.json`, and its gate tests refuse claude-322's real config, no keys, `[1, 2]` and a second key flag. Five new mutations (the check dropped, the check at `[1, 2]`, the one-key clause dropped, `SHAPE` at `1,2`, `SHAPE` with no keys) must each be caught.
4. **I-k is new:** the copy keys in the config, the snapshot and the served header, at serve.
5. **The pairs** add claude-322's one-key grades, pinned. The DEV read adds claude-322's curves.
6. **The seal path** is `results/claude/one-model-end-to-end-b64l-d3mwcap6-k21-2026-11-08/runs/s{1,2}/`, and `runs-2/` for a rerun.
7. **The pods** are brought up by `seed-323.sh` under lease session claude-323, with the watcher's screen `c323-wS<SEED>`.
8. **The SSH wait** is stated as `uor-pod up`'s 900 s.

## Pre-run review

**NO GO, then GO after one fix,** 8 Nov: the D67 pre-run review, one round, a fresh-context reviewer, on the scripts in `local/claude/claude-323/` against claude-322's.
- **MAJOR 1, fixed:** `seed-323.sh` still checked the shipped job, chain and seal against **claude-322's** hashes. The check fails safe (exit 4), but each pod would have sat up and billed with no run, because the watcher's stall notice only pauses renewals. Every gate passed, since none read the seed script's pins.
  - Now `seed-323.sh` pins claude-323's job, chain and seal hashes (below).
  - **A new gate** in `gt/test-seal.sh` checks that the pins equal the current hashes of the three scripts beside it. It fails on claude-322's pins.
- **MINOR 2, fixed:** two history comments (`chain-323.sh` line 9, `watch-323.sh` line 87) say claude-322 again, the unit whose review those fixes came from.
- **MINOR 3, checked:** the export snapshot's `.hamming` is `FbsHammingSnapshot` (`crates/uor-r4-integer/src/fbs_snapshot.rs` ~396–412 on `main`). Its `copy_keys` field is serialized when present, so I-k's `.hamming.copy_keys` is the right path.
- **Everything else is as intended:**
  - every other hunk is an intended change;
  - `end()`, the traps, the watcher's stall, renew and down logic, seed's $0.70/h refusal and the seal's non-empty refusal differ only by renames;
  - L-002 holds, and claude-322's mix, grades and seals are only read;
  - I-3's clause passes a real mixed config (`hamming_copy_key` 1 beside `[2, 1]`; jq compares values, not layout);
  - I-k fails closed on any missing file or field.

**The gates at the current hashes, 8 Nov:**
- `gt/test-gates.sh` 55 ok (claude-322's 50 and the 5 copy-key cases);
- `gt/mutate.sh` 23 of 23 caught (claude-322's 18 and the 5 copy-key mutations). This was run by the reviewer at the job hash below; the fixes did not touch the job;
- `gt/test-seal.sh` 10 ok (claude-322's 7 and the 3 pin checks);
- `gt/test-watch.sh` 5 of 5 mutations caught;
- `bash -n` ok on every script.

**The script hashes** (sha256):
- `job-323.sh` `699b5c8adf429ae7ec9871fff150cfcd25f41a3b24cb63129a69bc6fe83c2859`
- `chain-323.sh` `7410f49839f78c2987fa3a4e6c7e6903ee11bb79fa1b2ead8d788e6a7b89defd`
- `seal-323.sh` `846800cfe02b8b894d63730eaea815222b0e61ab5b55253d22b2246dd4cdd437`
- `watch-323.sh` `52f2f7c1d37806759658e63cdcfae3648fa0d77fa549cbe39876b377e195af1b`
- `seed-323.sh` `02622dbf1457eadfb5d239448e184bb47774754bdd8837549b78425568c54ea2`
- `read-323.sh` `828e96b8c66a6c898de5ad56d79f1ef7747c8029843bee41f56c38462b72f4bf`
- `firstline.jq` `b15c778db57c2e65902e34987cf15748213128ab5d9bb72230f97f0ea58fb702`
