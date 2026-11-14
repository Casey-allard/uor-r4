# Prep sheet: pair rows at full weight, rare pairs raised by 1/R, four training runs (one-model-end-to-end, claude-374, 14 Nov 2026)

Lab: claude. Item: [one-model-end-to-end](../../roadmap/items/one-model-end-to-end.md) (#3407). Unit: [claude-374](../../loop/tasks/one-model-end-to-end/claude-374/ORDER.md), PR #3965. Follows claude-373 ([its sheet](one-model-end-to-end-rare-pair-rows-2026-11-14.md), PR #3964) and claude-370 ([its sheet](one-model-end-to-end-pair-rows-2026-11-13.md), PR #3961). The runs go ahead on this lab's own pre-run review (CHARTER §5.1) and are logged for the owner.

## Question

When each pair row is weighed by 1 + 1/R, so that frame pairs keep claude-370's full weight and a pair seen with one reply weighs 2, do the fact and number replies come out right more often than with claude-370's plain rows, without losing its exact replies?

- **What exists.** claude-373 trained claude-370's `pair` arm again on RTX 5090 pods: exact 519 and 520 of 2,218; fact and number rows ended with the answer word 85 and 85 of 188. Its `rare` arm (weight 1/R) fell to 75 and 71 fact rows and 378 and 452 exact. Its replies copied the turn's words and then did not stop. Its REPORT reads this as frame pairs carrying where the reply ends.
- **What is missing.** A weight that raises the rare pairs without lowering the frame pairs. With 1 + 1/R no row weighs less than in `pair`.

## Base artifact

- **The unit's build:** `cargo build --release --locked -p uor-r4-float-train --features cuda --bin train-in-space --bin fbs-snapshot`, `UOR_NVCC_ARCH=compute_120` (RTX 5090), at this unit's code commit `4aeef3c0957234a1e43f5a4f1fb4f8bc61b3efef`. Each pod builds it from a clean checkout of that commit and records the binaries' SHA-256; the REPORT gives both pods' hashes. One pod's build trains and scores both arms of its seed, so a per-seed comparison never spans two builds. The pods' hashes are not required to match (compiled identities in this crate can differ between clean rebuilds).
- **The control** is arm `pair`, trained again by the unit's build on the same pod as `boost`. Without `--pair-weight` the unit's code is claude-370's (a test pins the bag, the saved files and the forward bit for bit), so `pair` is claude-370's and claude-373's arm `pair`. claude-373's 5090 numbers (519, 520 exact; 85, 85 fact rows) are quoted in the REPORT beside this unit's `pair` as a reproduction reading, never as the control.
- **The recipe** is claude-370's arm `pair`: claude-358's `p` recipe with `--pair-rows 2`:
  `--objective cell --retain-rates 1 --decay-mult 1 --lr 0.001 --lr-end 0 --context 256 --width 256 --hamming-read 4 --batch 16 --turn-memory 8 --turn-memory-reply 1 --turn-memory-dropout 500 --samples 6 --reply-tokens 48 --hamming-copy-latest 1 --turn-memory-leave-copied 1 --hamming-ask 1 --hamming-place KIT --hamming-place-pull 1 --hamming-copy-longest 714,28 --pair-rows 2`, `--eval-every 2048`, 16,384 steps.

## Data split

The inputs are claude-370's, at its sheet's hashes, copied to each pod from the laptop (`local/claude/claude-370/in/`) into `/workspace/in374/` and hashed again there (`local/claude/claude-374/in-374.sha256`).

| Input | Path on the pod (under `/workspace/in374/`) | SHA-256 |
|---|---|---|
| TRAIN tokens | `tr/train/tokens.u16` | `e25775f3aad4548e238736e02a3185e6c57062b8962a4147aafb655a6d865f32` |
| TRAIN mask | `tr/train/response_mask.u8` | `d8ee0add4286293bf6471659f275e77346994accae41be53be8b47d8bd6e93d1` |
| P14 tokens (1,108 scored replies) | `p14/train/tokens.u16` | `b8ad9fe401226b6df8677d06b3f2ad60ea4ffdf6a0b8092970c69e57c4355fe4` |
| P14 mask | `p14/train/response_mask.u8` | `67eed533b50fb685d75875050cdd88205a34033d51f491ce19b36795ea75674b` |
| P23 tokens (1,110 scored replies) | `p23/train/tokens.u16` | `d2da2c2c46ce69c13a6ed06107b7511b448578a5997b3aa6e70070816b1b440d` |
| P23 mask | `p23/train/response_mask.u8` | `b40477356a820f48c7a69d2251b77e79a928f1dc752560a351adba3ddff03e05` |
| Place kit | `kit/train-places.json` | `8d28571a614e4e038fcb52e2661787f38af62933dbf7ddad2ad64582c8a0a9b3` |
| Tokenizer | `driver-data/tokenizer.json` | `d36d3e8700a123e620012df77de195f244fbdb4d05d9e1aa7e77fa9407590f89` |

- The pair dictionary and R are counted from TRAIN's user turns and their replies only. No panel is read while training.

## Arms

| Job | Recipe | Seed | Steps | Pod |
|---|---|---|---|---|
| `pair-s2`, `boost-s2` | the recipe; `boost` adds `--pair-weight boost` | 2 | 16,384 | lane a |
| `pair-s3`, `boost-s3` | the same | 3 | 16,384 | lane b |

`--pair-weight boost`: each pair row's weight is 1 + 1/min(R, 4096), with R the number of different replies (exact token sequences, from the header's last token to the token before the next user marker) of the TRAIN turns that hold the pair. R is claude-373's count, by the same function. The stored weight is claude-373's 1/min(R, 4096); the forward adds 1 to it when the rows carry the `boost` mark. The weights are fixed at start-up, printed as a census (R at min, 50 %, 90 %, 99 %, max), and saved with the rows and the mark. A reply's bag is the weighted sum of its turn's rows. All else is claude-370's.

## Readings

- **Scoring:** every model on P14 and P23 by its pod's build in float, with claude-370's arguments (`--score-rows CHECKPOINT --panel p=PANEL --sample-opening last --samples 1200 --context 192 --reply-tokens 48 --retain-rates 1 --seed 2`), so every pass scores the same rows.
- **The fact and number rows:** claude-370's `subst-370.sh` rule, run by this unit's copy `local/claude/claude-374/subst-374.sh` (SHA-256 `8384c47251e75be3b07754ab841e67b9f3451743f7616bf9ec3882bbb791b691`). It is claude-373's `subst-373.sh` with `rare` renamed `boost` and 373 renamed 374, and that diff goes into the REPORT. The reading script `local/claude/claude-374/read-374.sh` is fixed before any result is fetched: SHA-256 `48c9fbc9de2f2af44a3a8ad3e5efb9136b0b38001a3c95ad0a10942afba2a5c2`, checked again when the results are read and quoted in the REPORT. Run on claude-373's fetched results with the new arm named `rare`, it gives claude-373's published counts (85, 85, 75, 71 fact rows; 519, 520, 378, 452 exact; "Does not add").
  - These are the rows of the C kinds (capital, colour, day after, legs, opposite, sound) and the D kinds (add, bigger): 101 on P14 and 87 on P23. A row counts when the reply ended and holds the answer word, compared as whole words with letter case ignored. The answer word is the reference's last word that the turn does not hold; when the turn holds every reference word, it is the reference's first word. On the "bigger" rows the answer is in the turn, so the count by kind shows "bigger" separately.
- **Exact replies**, by panel; **replies that did not end** (48 tokens, no end mark).
- Read without a bar: exact facts, replies by kind and by need on P14, the weight census, rows with a non-zero entry, seconds per step, the last `train_loss_mean` and `dev_loss`, and twelve replies word for word from each model (claude-370's twelve rows).

## Bars

The order's bars, with their counts spelled out before any run.

- **The number.** For seed s, gain(s) = fact and number rows counted for `boost-s` minus those for `pair-s`, out of 188.
- **The guard.** For seed s, the drop in exact replies over the two panels, `pair-s` minus `boost-s`, out of 2,218. 1.5 points is 33 replies.
- **Adds:** gain(2) ≥ 10 and gain(3) ≥ 10, and the drop is 33 or fewer at each seed.
- **Does not add:** (gain(2) + gain(3)) / 2 < 4.
- **Not called:** every other result. That covers a mean gain of 4 or more without both seeds at 10 or more, and gains that pass with a drop above 33 at a seed.

## What each outcome unlocks

- **Adds:** the next unit serves the weighted table, starting from claude-370's open item (a smaller integer form of the 48 M values, and a read in `uor-chat`). A third seed runs beside it.
- **Does not add:** this weighting is closed (1 + 1/R on claude-370's placement), not the mechanism. Together with claude-373 it says the fact rows are not reached by re-weighing pairs alone. The next reading is whether the fact rows' pairs (for example capital with a country) have small R in the census. If they do not, the address is the gap, not the weight, and the next unit adds triples.
- **Not called:** seed 1 of both arms, as a separate unit with its own order and sheet. No arm, seed or step count is added to this unit after a result is read.

**Prediction, written before the runs** (a calibration record, not a bar): gain between −5 and +15 per seed; the drop between −15 and +30; `boost` within 5 % of `pair`'s speed (claude-373's weight cost nothing measurable).

## Instrument checks

- **I-1** every input at this sheet's hash on each pod.
- **I-2** the trainer's marker line on every run, as in claude-370: `the longest rule's marker 714,28 stands before every reply opening and nowhere else: TRAIN 215017, DEV 1226`.
- **I-3** is not used. claude-373's I-3 reused claude-370's `wick` checkpoints; here every model is trained on the pods by the unit's build.
- **I-4** a 64-step smoke with `--pair-weight boost` from the pod's build, before any long run:
  - its weight census line prints R at the five quantiles, and R varies: p90 is above 1 and max is above p50. A census with every R at 1 would make `boost` the `pair` arm with every row doubled, so it is an instrument failure and the long runs do not start;
  - its training log prints `[pair] weight kind: boost` exactly once;
  - its dictionary SHA-256 is claude-370's (`0b064028…b82e`, the full value in claude-370's training log);
  - its checkpoint is scored on each panel, and both passes pass I-8 below as `boost`: rows with a non-zero entry, the dictionary, 90 or more in 100 replies reading a bag, the census and the kind (so the checkpoint holds the weights and the `boost` mark). If either pass fails, pre ends non-zero, the long runs do not start, and this sheet is amended first.
  - Every I-4 check gates the long runs: pre ends non-zero when any one fails.
- **I-5** the unit's GPU tests pass on each pod's GPU, at the code commit. They include: a boosted bag is the sum of (1 + 1/R) × row and a one-reply pair weighs 2; `--pair-weight replies` rows are claude-373's and write no mark; a boosted file round-trips and a mark without weights is refused; without the flag the bag is claude-370's bit for bit; a boosted reply decoded with cached steps equals the whole-window reply.
- **I-6** before each job starts, the launcher reads the GPU from the host. The only compute process may be this unit's own trainer, and the memory in use, plus the smoke's peak, plus 4 GiB must fit the card. Otherwise the job does not start.
- **I-7** every job's folder is new and is sealed with its files' hashes and its exit code. The lane is in every sealed store path (`…/run/LANE/STAGE/TRY`), so the two pods never write one path (claude-373's slip 1). Every scoring pass of a panel scores the same rows as its pod's pre stage's pass of that panel; otherwise it is an instrument failure.
- **I-8 every scoring pass reads the trained rows of its checkpoint, with the weight kind it was trained with.** For every scored checkpoint, of both arms:
  - the training log's last rows line shows rows with a non-zero entry, above zero;
  - the scoring log's dictionary SHA-256 equals the training log's;
  - on each panel, 90 or more in 100 scored replies read a bag;
  - the scoring log's line `[pair] loaded weight replies: …`: for `boost` it is the training log's `[pair] weight replies: …` census, character for character; for `pair` it reads `none`;
  - the scoring log's line `[pair] loaded weight kind: …`, exactly once: `boost` for `boost`, with the training log's `[pair] weight kind: boost` exactly once; `none` for `pair`. A `boost` checkpoint that lost its mark would load as claude-373's 1/R rows and read `replies` here; one that lost its weights and kept the mark is refused at load; one that lost both reads `none`. Each fails I-8.
  - Each `boost` run's training census equals I-4's census character for character, and equals claude-373's census (`187413 pairs; R min 1 p50 6 p90 48 p99 545 max 4096 (cap 4096)`), since R comes from TRAIN alone; this is compared on the laptop when the results are read by this lab, by `grep` of the fetched logs, and the comparison is quoted in the REPORT.
  - The job checks all of I-8 into each score stage's `i8.txt`; the reader prints no verdict unless every `i8.txt` reads PASS, refuses a model or panel with more than one pass under the fetched folder (a retried try beside it), and prints no verdict when the check is skipped.
  - A failed I-8 is an instrument failure: it is reported as that, never as "Does not add" or "Adds" (L-007).

## Risks the build leaves open

- **Two seeds,** with same-seed pairs. The middle is not called.
- **Adam rescales a row's step.** A weight above 1 scales the row's gradient, and AdamW divides that scale out, so a raised row still moves about one rate a step. Its contribution, weight × row, grows up to twice as fast. Frame pairs are unchanged from `pair`.
- **R is exact per reply string.** Rephrasings of one answer count as different replies, so R overstates how many answers a pair predicts, and some content pairs weigh closer to 1 than their meaning would give. The census is reported so this can be read.
- **The rows are 48 M values,** as in claude-370. Nothing here is served.
- **Seconds per step** are reported, never barred.

## Minimum test power

On the 188 rows, claude-373's two `pair` seeds on the 5090 both counted 85, and claude-370's on the 4090 counted 86 and 89; its `b` seeds differ by 17 (71 and 54). The line of 10 at each seed clears the `pair` seed spread, but not the `b` spread. With binomial noise on 188 rows (about 7 rows standard deviation per model), 10 at each seed is about one standard deviation of a same-seed difference. Under no effect, both seeds clearing 10 is a few percent, and a true gain of 10 still often lands under "Not called" or "Does not add". That is accepted because a negative here closes a configuration, never the mechanism (§4.1), and the middle is not called. Two seeds do not measure the spread of one seed trained twice; a called "Adds" is followed by a third seed before anything is served.

## Steps

1. **Pre,** on each pod: I-1; the unit's build; I-5; I-4; seconds per step and peak card memory.
2. **`pair-sN` then `boost-sN`,** one at a time per card.
3. **Scoring** by the pod's build: both models of its seed on P14 and P23.
4. **Seal** each stage to the store, **fetch** to the laptop, check the store's sums and hash each seal on the laptop; the readings; the REPORT.

## Cost

Two RTX 5090 Vast pods (offers near $0.56/h each). claude-373 trained a run in about 2,680 s (6.1 steps a second) and peaked at 4,496 MiB of card memory. A pod's build, tests, smoke, two runs and four scoring passes come to about 2.5 hours, so about $3 in all, inside the caps (4 pods, $8/h).

## Compute and stop rule

- **Where.** Two Vast pods of one GPU kind (RTX 5090) through `scripts/pod/uor-pod`, leased and inside the caps, with a completion watcher armed at launch (L-004). `wick` is not used: on 14 Nov another lab's job holds its card, and this lab does not start beside another lab's job (I-6). Every model of the comparison comes from one GPU kind.
- **How.** Each lane is a detached chain on its pod (`chain-374.sh`), one job script a stage with a log and an exit file. The laptop's watcher (`watch-374.sh`) renews the lease every 10 minutes and waits for the chain's exit file. Lane b is launched only after lane a's `up` returns, so the caps check is never raced.
- **Stop.** A run whose loss is not finite stops itself. For arm `pair` that is an instrument failure (claude-370 and claude-373 ran it to its end) and it starts again. For arm `boost` it is classified first (§4.2): with every weight between 1 and 2 the rows add at most twice what they add in `pair`, so a non-finite loss is first read as a defect in the build and repaired, with the run started again. Only if no defect is found is it a result about this configuration, and then it falls under "Not called".
  - A healthy run is never cancelled for time.
  - A training log that has not grown for 60 minutes is hung (an eval prints every 2,048 steps). The job stops its own trainer by its pid; that is an instrument failure, and the run starts again as a crashed run does.
  - A scoring pass is capped at 40 minutes. One that reaches the cap is run again with a longer cap, never read part-way.
  - A crashed run is an instrument failure. It starts again from step 0 in a new folder (L-002) and is never scored part-way.
- **A pod is taken down** only after every stage it sealed is fetched and its seal hashed on the laptop, and the laptop's hash equals the store's (`gate-down-374.sh`).

## FIT

- **First read** (fresh, non-author, 14 Nov): NOT_FIT on two points. The job's I-8 compared only the weight lines, so a `boost` checkpoint with zero rows, another dictionary or few bag readers kept its mark and census and could read "Does not add" (L-007); and I-4's smoke checks did not gate the long runs, one (the step-0 line beside a pair smoke) had no pair smoke to compare with. The job's I-8 now checks the training log's last rows line, the loaded rows and dictionary against training, and 90 in 100 bag readers, each tested to fail on claude-373's real logs broken one at a time; the smoke's I-8 now ends pre non-zero; the step-0 sub-check is replaced by the dictionary SHA-256. Its notes are taken: the reader refuses a second pass of one model and panel and prints no verdict when I-8 is skipped (its SHA-256 above is the new file's, and it still gives claude-373's published counts); the census comparison is named as this lab's, quoted in the REPORT.
- **Second read** (another fresh non-author reader, 14 Nov): FIT, no blocking point, for the sheet, the order, the job script, the reader and the seal's lane paths. It confirmed the first read's points are closed in the files. Its notes are taken: the job's I-8 comments describe all of I-8, and an unparsable bag line now fails with its own reason. It did not read the launcher, the watcher, the fetch or the down gate, so a third reader read those.
- **Third read** (another fresh non-author reader, 14 Nov), for the launcher, the watcher, the fetch, the down gate and the input hashes: NOT_FIT on two points. The down gate required all five stages even when the chain skipped some (a failed pre, a crashed run's scoring stage), so a pod would have been held forever; and the watcher released the lease before `down` and exited whatever `down` returned, so a failed `down` would have left an unleased, unwatched pod (L-004). The gate now checks every stage the lane ran (its `STAGE job=` line) and always `pre`, and refuses without the stage list, tested on a crashed run, an unfetched `pre` and a failed `pre`; the watcher now runs `down` first, retried with the lease renewed until it succeeds, and releases after. Its notes are taken: the pods' input folder is `/workspace/in374/` (the table is corrected), and lane b launches after lane a's `up` returns.
- **Fourth read** (another fresh non-author reader, 14 Nov), for the third read's two fixes: FIT, no blocking point. The gate's `^STAGE job=` match is anchored and agrees with the chain's stage list, written whole before the exit file; `down` is retried with the lease renewed and the lease is released only after. Its notes are taken where cheap (the watcher unsets the gate's test folder variable); a failed fetch or seal holds a watched pod for a person to act, as before.

TEST FITNESS: FIT
