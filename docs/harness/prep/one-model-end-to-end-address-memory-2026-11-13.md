# Prep sheet: the E8 address memory trained in the model, and the model trained on people's conversations (one-model-end-to-end, claude-346, 13 Nov 2026)

Lab: claude. Item: [one-model-end-to-end](../../roadmap/items/one-model-end-to-end.md) (#3407). Unit: [claude-346](../../loop/tasks/one-model-end-to-end/claude-346/ORDER.md), which built the part (PR #3913). The run goes ahead on the lab's own pre-run review inside the GPU caps (CHARTER §5.1), logged for Casey, who can reverse it.

## Question

1. **Does the E8 address memory, trained inside today's model on the model's own corpus, change what the model says?** The reading is the count of replies equal to the wanted reply on the 1,108-prompt scoring panel, against the same recipe without it.
2. **Does today's model learn anything from people's own conversations (OASST2), and does the address memory change that on held-out conversations?**

Why now. Today's model has about 1.1 million learned values, 1,048,576 of them the token table. The address memory adds 11,796,480, read by the state's own E8 cells. It was built on 9 Nov, and its tests passed on an RTX 5090 on 10 Nov (483 passed, 0 failed; record at `results/claude/one-model-end-to-end-address-2026-11-10/gpu-tests/try-1`). It has never been trained in the model. Every training run so far read one template generator's text. OASST2 has been in the store since 5 Nov and no run has read it.

## Base artifact

- **Code:** `38e209ee2ed733ae1a54732d06b614ab4d6da7cd`, PR #3913's head: the address memory of `cb9d32f99e1c81034aba128fc16f9fb732fefa9b` with main merged in, which brings claude-357's copy rule.
- **Code, amended 13 Nov 2026 after try-1 (an instrument repair; no question, arm, reading or bar changes):** the pin is now `f941a33fd1a4eea751705d1cd6021af35452a4b7`, two commits on the pin above, both in `crates/uor-r4-float-train/src/` (`e8_address.rs`, `flock_read.rs`).
  - **What stopped try-1.** Pod a's pre stage ended at I-s: the `e8` smoke ran out of GPU memory in its first training steps (`EXIT 5`, sealed at `results/claude/one-model-end-to-end-address-memory-2026-11-13/run/pre/try-1`). The `e8` and `sign` arms work each cell's index out on the GPU from the residual, so the index carried the decode's history. candle then put an op on the rows' gradient, and the optimizer's running means, built from that gradient, kept every step's forward pass allocated. `e32c5e8fbe69cdc3d962595f82959ac32d68f269` cuts the index and the optimizer's gradients from the graph and adds two GPU tests.
  - **What was slow.** With that repaired, every address arm trained four times slower than the plain recipe: 0.52 s a step against 0.125 s. candle's kernel for the gradient of an indexed read runs one serial loop over the indexed dimension for each element of the others; read flat by row, that was 256 loops over the batch's 131,072 block entries, six times a step. The new pin reads the same table a block at a time, by gather: 8,192 loops over 4,096 positions, summed in the same order.
  - **No number the model computes changes.** An integer index never took a gradient, and the stored rows, the census and the read's values are as they were. On pod a (RTX 5090) the 54 GPU tests of I-g's list pass at the new code, the exact-value tests of the read and of its gradient among them. With the optimizer's cut set aside its test fails; at `e32c5e8f`, with both cuts set aside, both new tests failed.
  - **Measured on pod a, batch 16, peak GPU memory of 32,607 MiB.** At the old pin: no address 3,628; `token` 4,780; `collapse` 4,428; `e8` and `sign` out of memory (`e8` also at batch 8 and 4, with the band off, and fed one window at a time; at batch 1 `e8` 17,708 and `sign` 17,484, still allocated after training ended). At the new pin: no address 3,596; `token` 5,452; `sign` 9,132; `e8` 9,452 over 48 steps, and 5,516 with the band off.
  - **Measured training seconds at the new pin, batch 16:** `e8` 48 steps in 10 s (25 s at `e32c5e8f`); `sign`, `token` and the band-off `e8` 16 steps in 3 s (8 s); no address 16 steps in 2 s.
  - **Records:** `results/claude/one-model-end-to-end-address-memory-2026-11-13/run/memory-check/try-1` (96 files: each run's memory trace, output and error, the build and test logs, and the six result logs).
- **Recipe:** claude-358's p arm, unchanged: `--objective cell --retain-rates 1 --decay-mult 1 --lr 0.001 --lr-end 0 --context 256 --width 256 --hamming-read 4 --batch 16 --turn-memory 8 --turn-memory-reply 1 --turn-memory-dropout 500 --samples 6 --reply-tokens 48 --hamming-copy-latest 1 --turn-memory-leave-copied 1 --hamming-ask 1`. On the model's own corpus it adds `--hamming-place KIT --hamming-place-pull 1 --hamming-copy-longest 714,28`. On OASST2 it adds neither: that text has no marker.
- **Measured with this recipe** at `f694d162d9b04d1226fd17c0a81825f00e9da007` (claude-358, float scoring pass): replies equal to the wanted reply, of 1,108: **188** at seed 2, **188** at seed 1, **225** at seed 3. Last `train_loss_mean` and `dev_loss`: 0.5447 and 2.5630 at seed 2; 0.4981 and 2.5471 at seed 3.

## Data split

| Use | Store root | Tokens | `tokens.u16` SHA-256 | `response_mask.u8` SHA-256 |
|---|---|---:|---|---|
| TRAIN m (the model's own corpus, no memory line) | `results/claude/one-model-end-to-end-memory-line-2026-11-10/corpus-noline/train` | | `e25775f3aad4548e238736e02a3185e6c57062b8962a4147aafb655a6d865f32` | `d8ee0add4286293bf6471659f275e77346994accae41be53be8b47d8bd6e93d1` |
| Scoring panel m (generator seed 14, 1,108 prompts) | `results/claude/one-model-end-to-end-memory-line-2026-11-10/panel-noline/train` | | `b8ad9fe401226b6df8677d06b3f2ad60ea4ffdf6a0b8092970c69e57c4355fe4` | `67eed533b50fb685d75875050cdd88205a34033d51f491ce19b36795ea75674b` |
| TRAIN o (OASST2, train) | `results/claude/real-text-corpus-2026-11-05/oasst2/train` | 3,773,419 | `8bdcabfb41284a54575ea2f730f532a896b29dc69250098957b46c6bcae40b90` | `a2b0b2a3ce8cc7491b8ced86eda487353ba4e70da0f9aa5a63161ca6802de7a1` |
| Scoring panel o (OASST2, held out) | `results/claude/real-text-corpus-2026-11-05/oasst2/heldout` | 214,758 | `02a04e97bbb8936c993bf5eff0f29c171ebeae88988217a4100fdc77701f169e` | `9c5bb6567c6a4cded2dfb2280ae4acdc798e283a3beae8aaebc695656e1b4833` |

The place kit is `results/claude/one-model-end-to-end-place-read-2026-11-11/kit/train-places.json` (`8d28571a614e4e038fcb52e2661787f38af62933dbf7ddad2ad64582c8a0a9b3`). The tokenizer is the w2 driver data's (`d36d3e8700a123e620012df77de195f244fbdb4d05d9e1aa7e77fa9407590f89`), with its `ft/dev` as the trainer's DEV. On the OASST2 jobs the trainer's DEV lines are still that DEV and are not a reading. OASST2 is conversations people wrote; its manifest's filter drops every message marked synthetic, and no model-written text is in this run (§3.10). A conversation tree is held out whole: the side depends only on a hash of the tree's id (50 per mille; 278 held-out threads against 5,071), so no held-out prompt has its own thread in TRAIN.

## Arms

| Job | Pod | TRAIN | Address memory | Steps | Seed |
|---|---|---|---|---:|---:|
| `e8-s2` | a | m | `e8` | 16,384 | 2 |
| `e8-s3` | a | m | `e8` | 16,384 | 3 |
| `o-b` | b | o | none | 4,096 | 2 |
| `o-e8` | b | o | `e8` | 4,096 | 2 |
| `b-s2` | b | m | none | 16,384 | 2 |
| `token-s2` | b | m | `token` | 16,384 | 2 |

- `e8`: each layer adds, for each of the state's 32 blocks of 8 coordinates, the row of that block's nearest E8 root (240 cells a block; band 0.141; rows start at zero and step at `lr/32`).
- `token`: the same table, addressed by a seeded hash of the position's own input token. It keeps the table's size and takes the state out of the address.
- `b-s2` is claude-358's seed 2 job again at this pin. The no-address job at seed 3 is claude-358's own (225).
- **Rules check (§3).** The address is the E8 root system the project's cells already use. No squashing function: the band's `max(0, ·)` is a clamp at zero (§3.2). The read is 32 row lookups and adds a layer and token; no state is multiplied with a table (§3.3). §3.6 excludes "a dense model disguised as lookup tables": a row here is chosen by one block's nearest root and added whole, as the token table's row is chosen by a token.

## Readings

All from the float scoring pass on each job's last checkpoint (`--score-rows`, greedy, turn memory on, context 192, 48 reply tokens), and from the job's own logs. Counts are made by kit scripts that write logs.

- **R1, exact replies (m jobs):** of the 1,108 panel prompts, replies equal to the wanted reply up to the end token.
- **R2, loss:** the last `train_loss_mean` of `progress.jsonl` (all jobs) and the last `dev_loss` (m jobs only).
- **R3, the address census (address jobs):** `address.jsonl` at each checkpoint: cells in use a block, the share of reads whose cell moved, the share inside the band.
- **R4, held-out next token (o jobs):** over the held-out panel's reply rows, the share whose nearest token is the true one (`rows-p-memory.json`). Two reference points are read beside it: the share of those rows whose true token is the commonest one, and the same R4 on the pre stage's 64-step `o-e8` smoke, a model that has barely trained.
- **R5, replies word for word (o jobs):** the first 20 held-out prompts with the wanted reply and the model's, read by a person. A reply counts as word-ordered when it reads as at least one grammatical English clause; the count of the 20 is given, and 10 or more is the R5 trigger: a count that chooses the next run, not a finding about the model. The share of replies that end inside 48 tokens is given with it.
- **R6, cost of the part:** training seconds for 64 steps, each arm.

## Bars

Question 1. `D2 = e8-s2 − b-s2` and `D3 = e8-s3 − 225` on R1, and `M` their mean.

| Outcome | Word |
|---|---|
| `M ≥ +30` and both `D > 0` | adds at this size (seeds 2 and 3) |
| `M ≤ −30` and both `D < 0` | hurts in this configuration |
| `\|M\| < 15` | no move at this size |
| anything else (including `\|M\|` from 15 up to 30, and a larger `\|M\|` whose two `D` differ in sign) | not called |

- **Control.** `C = e8-s2 − token-s2`. It is read only when the word is "adds". `C ≥ 30`: the control holds. `C < 15`: it fails, and the gain is a second table's size and not the state's address. Between: not called. It is one run against one run, with a standard deviation near 30, so it is a weak control: with no true difference `C` alone reaches 30 about one time in six. `C` is also read only after "adds", and it shares `e8-s2` with `D2`, so one lucky `e8-s2` lifts both: after a false "adds", a false "holds" comes up roughly one time in three, or more often.
- **Pin check.** If `b-s2` is outside 188 ± 20, the two pins do not train alike. `D3` is then not paired, and the word is taken from `D2` alone: adds at `D2 ≥ +60`, hurts at `D2 ≤ −60`, no move at `|D2| < 30`, otherwise not called. The REPORT says so.

Question 2 has no bar. Its rows below choose the next run's recipe; they are not findings, and nothing is claimed from them on the milestone.

## What each outcome unlocks

| Outcome | Next |
|---|---|
| adds, and the control holds | Seed 1 of `e8` and of `b` first. If it repeats, the address memory joins the base recipe; then its integer form and export (the exporter refuses it today); then the state-keyed read on top of it. |
| adds, and the control fails or is not called | `token` at seed 3 to settle the control. If it fails again, a second per-token table does the same and the state's address is not yet used: the next arm reads after a mix of the state (lexlean-r4's v9 mixes with a Hadamard transform before it codes a block). |
| not called | `e8` and `b` at seed 1, a third pair, before any new arm. |
| no move, and e8-s2's train loss is below b-s2's by 0.05 or more | The rows fit TRAIN and do not carry to the panel. Next: the `sign` arm and the band; a negative on this configuration only (§4.1). |
| no move, and e8-s2's train loss is not below b-s2's by 0.05 | The rows are not learning. An instrument question first (§4.2): the `lr/32` step and R3's moved share. |
| hurts | This configuration is closed. Next arm: a smaller row step. |
| Q2, read first, R5: fewer than 10 of 20 replies are word-ordered in both o jobs | Size and context are the limit: the scale ladder comes before more data. |
| Q2, R5: 10 or more are word-ordered in either o job (the R5 trigger) | A longer run on OASST2 with Wikipedia comes next. `o-b` can be exported and talked to on the laptop at once. |
| Q2, R4: o-e8 is above o-b by 1 point or more | With Question 1 at "no move" or "not called", the run R5 chose carries the address memory. |
| Q2, R4: the two are less than 1 point apart | With Question 1 at "no move" or "not called", the run R5 chose goes without it. If o-e8's last train loss is also below o-b's by 0.5 or more, the REPORT says the rows memorise 3.8 million tokens. |
| Q2, R4: o-e8 is below o-b by 1 point or more | With Question 1 at "no move" or "not called", the run R5 chose goes without it, and the row step is the next arm on real text. |

Who decides whether a later run carries the address memory: Question 1's word, when it is "adds" (the run carries it, until seed 1 says otherwise) or "hurts" (it does not). R4 decides only otherwise, and under 1 point apart the run goes without it.

**Prediction, written before the run** (a calibration record, not a bar): `M` between −10 and +50; e8-s2's train loss below b-s2's by 0.1 or more; `token-s2 − b-s2` within ±20; o-e8's R4 above o-b's by 1 to 6 points; neither o job writes sentences a person would call a reply.

## Instrument checks

The pre stage on each pod runs these and ends the pod's jobs if one fails.

- **I-p.** A clean checkout at the pin.
- **I-g.** Every test of `e8_address.rs` (the file's own count, 17 or more) and the trunk's cache test pass on the pod's GPU, with the copy rule's tests.
- **I-3.** Every input's SHA-256 is this sheet's.
- **I-s.** Five 64-step smokes train: no address, `e8` and `token` on m; no address and `e8` on o.
- **I-n.** The no-address smoke on m prints claude-358's eight `[memory]` lines and its step 0 line.
- **I-z.** Each address smoke's step 0 line equals its no-address twin's: the rows start at zero.
- **I-a.** An address job's last checkpoint holds `address_memory.safetensors`, and `address.jsonl` has a line at each checkpoint for six layers. A job with no arm writes neither.
- **I-r.** The `e8` smokes project at most 9,000 s for 16,384 steps and 3,000 s for 4,096. Past that no long job starts.
- **The scoring pass** runs at full size on both `e8` smokes' checkpoints before any long job depends on it.

Each long job repeats I-n (m jobs) and I-a on its own log and files; a failure is that job's row 0 and is said.

## Risks the build leaves open

- The rows train in float. They are not rounded to the served grid in this run, and the exporter refuses them, so the address models cannot be served on the laptop yet. `b-s2` and `o-b` can.
- Scoring reads at context 192 and training at 256, as in claude-358.
- The o jobs make about 4.4 passes over 3.8 million tokens, and the address memory has 11.8 million values: it can memorise. R4 and R5 are read on held-out conversations for that reason.
- GPU training is not bit-exact between runs or pins. The pin check covers the comparison with claude-358's seed 3.
- R5 is read by one person.
- The `token` control leaves three things open. A position's token already has its own row in the token table, so a control that holds shows the state's address beats a second per-token table, not that it beats size in general. The two arms fill their rows differently (a token's cell is read each time the token occurs; an E8 cell by whichever states fall in it), so the step a row effectively takes differs. And `token` has no band (0): no wall gradient reaches the state.
- The job script also knows a `sign-s2` stage. No pod runs it in this sheet.

## Minimum test power

The three same-recipe seeds of claude-358 gave 188, 188 and 225: a standard deviation of about 21, from three runs only, so it is itself loosely known. Whether pairing by seed removes that swing is not known. If it does not, `M` has a standard deviation of about 21, so `M ≥ 30` with both differences positive comes up by chance in under one run in ten. That is enough to choose the next unit and not enough for a milestone claim: a result called "adds" is repeated at seed 1 before any claim rests on it.

## Steps

1. Launch pod a (`try-1`), then pod b (`try-2`) once pod a's chain holds its GPU. Each pod's watcher is armed before anything ships.
   - **Amended 13 Nov 2026.** `try-1` stopped at I-s (see Base artifact). `try-2` never started: its host gave no SSH in 2,400 s and the pod tool deleted the pod. Lane a runs again as `try-3` on the same pod a, with a clean checkout at the new pin in a new folder and a new root; lane b runs as `try-4` on a new pod. Nothing else in the steps changes.
2. Each pod runs its pre stage, then its jobs in the Arms table's order. A failed long job does not stop the jobs after it.
3. Each stage is sealed to `results/claude/one-model-end-to-end-address-memory-2026-11-13/run/STAGE/TRY` and read back. The address rows of checkpoints before the last stay on the pod.
4. A pod goes down when its chain ends with every job 0 and every seal 0 and the four checks pass.
5. On the laptop: fetch each stage, count by kit script, read R5, write the REPORT and the notebook row in PR #3913.

## Cost

Two RTX 5090 pods, each refused above $1.60/h. Pod a: about 1.6 to 5.2 hours. Pod b: about 2 to 4.5 hours. At $0.60 to $0.90/h that is $2 to $9. Two pods are inside the caps of 4 pods and $8/h; `uor-pod status` is read at launch.

## Compute and stop rule

Vast through `scripts/pod/uor-pod`, sessions `claude-346-a` and `claude-346-b`. A stall, spend or time notice is never a stop: a healthy run is extended and the notice is recorded. A failed pre stops its pod's jobs. A failed scoring pass is said and does not fail its job. No run writes into an existing root (L-002).

## FIT

### First read, of the sheet at `50766bf56e36679f88ffed5d8e023f6c638256d0`: FIT WITH CHANGES

A fresh reviewer who did not write the sheet (claude, fresh context) read it against the job script and claude-358's three sample files. It confirmed 188, 188 and 225 of 1,108, a standard deviation of 21.4, the stated chance for `M ≥ 30` (about 0.08 with no pairing), the Arms table against `job-346.sh` and `chain-346.sh`, and no conflict with CHARTER §3. Its six findings, each taken:

1. **The control could not decide.** `e8-s2 − token-s2` is one run against one run (standard deviation near 30), and a cut at 15 is half of that. Now: holds at 30 or more, fails below 15, not called between, and the sheet says it is weak.
2. **Next steps out of order, and outcomes with no row.** Seed 1 now comes before the part joins the base recipe; "not called" has a row; question 2 has a row for each case.
3. **A large `M` could be called "no move"** when the two differences had different signs. Now that case is "not called" unless `|M| < 15`.
4. **Question 2 had a bar in practice, and no reference point.** Its rows are now named as recipe choices and not findings; R4 is read beside the commonest-token share and a barely trained model; R5 has a written criterion; the held-out split by conversation tree is stated; `dev_loss` is no longer read on the o jobs.
5. **Confounds of the `token` control** are named in Risks.
6. **The pin check's wording** now gives each threshold by name.

### Second read, of the sheet at `828457d76aea27e11aeaa70c4122cfce19e087a2`: FIT WITH CHANGES

A second fresh reviewer (claude, fresh context) read the changed sheet. Six smaller findings, each taken:

1. **Question 2's two readings could point at different next runs.** Now R5 is read first and chooses the run (the scale ladder, or a longer real-text run), and R4 only says whether that run carries the address memory.
2. **No row for e8's train loss above b's.** The second "no move" row now reads "not below b's by 0.05".
3. **The control's error rate was understated.** `C` is read only after "adds" and shares `e8-s2` with `D2`; the sheet now gives the rate of a false "holds" after a false "adds".
4. **The prediction named `T`,** which the sheet no longer defines. It now names `token-s2 − b-s2`.
5. **Two sets of rows overlapped or left a gap.** The last Bars row now reads "anything else" with its two cases named, and the R4 rows meet at "less than 1 point apart".
6. **"Writes word-ordered text" read as a finding.** It is now "the R5 trigger", a count that chooses the next run.

### Third read, of the second read's changes only, at `4dcd7ad61a23d70fcc5618f6a784b187661f57b6`: FIT WITH CHANGES

A third fresh reviewer (claude, fresh context) read the delta. It found findings 2, 4, 5 and 6 resolved, and the Bars table free of gaps and overlaps. Its remaining points, each taken in its own wording:

1. **One row still sent the next run two ways** (more text first, against the scale ladder first). That row now only says the run goes without the address memory.
2. **Who decides whether a later run carries the address memory** was open between Question 1's word and R4. One sentence under the table now says: Question 1's word when it is "adds" or "hurts", R4 only otherwise.
3. **The control's false-"holds" rate was a little low.** It reckoned about 0.36 to 0.4; the sheet now says one time in three, or more often.
4. **Wording:** replies are "word-ordered" and the count is the trigger; the loss rows name the pair `e8-s2` and `b-s2`.

None of these changes what the run does or how Question 1's result is called: they are all in the rows that choose the next run.

### Fourth read, of the third read's changes, at `781f20a6df5f16f9dd905accef93c0efdde14b4a`: FIT

A fourth fresh reviewer (claude, fresh context) read the Readings, Bars and outcome sections and the last delta: every outcome gets one word and one next step that conflicts with no other, and the third read's points are taken. Its one optional note is taken too: a later run carries the address memory on "adds" until seed 1 says otherwise.

### Fifth read, of the 13 Nov amendment and its code, at `9bd036446f355b1d87093c9836160c202809e75e`: FIT

A fifth fresh reviewer (claude, fresh context, 13 Nov 2026) read the code change from the old pin to `f941a33fd1a4eea751705d1cd6021af35452a4b7` in `e8_address.rs` and `flock_read.rs`, and this sheet's amendment. It read code only and ran nothing. Its five checks all passed:

1. **The read picks the same rows.** Block `b`'s cell `c` is still row `b × cells + c`, for every arm, the one-cell collapse arm included.
2. **The gradient is the same.** Each row receives the same contributions in the same order of positions as before; no gradient ever went through the index, before or after.
3. **The GPU layouts are ones the kernels accept.**
4. **The census counts the same row index as before.**
5. **The amendment changes no question, arm, reading or bar**, and nothing in the code contradicts that.

Its two notes, both low:

- The new history test has one block, so it does not itself read through more than one block. The exact-value read and gradient tests do: they run at two blocks and six positions, and passed on pod a with the rest (54 passed, 0 failed, log `fix2-try-1.log` in the memory-check record). Both smokes also trained at the run's 32 blocks.
- It asked that the optimizer's test be shown to fail with its fix set aside. It was, on pod a: that test FAILED with the fix set aside and passed with it in place (same log).

TEST FITNESS: FIT - research, #3407, a fresh non-author read at 9bd03644 of the 13 Nov instrument repair, after four earlier reads with every finding taken (claude-346)
