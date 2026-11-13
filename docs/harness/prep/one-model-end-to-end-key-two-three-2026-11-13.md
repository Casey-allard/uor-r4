# Prep sheet: the trained key on the two tokens before the link word (one-model-end-to-end, claude-361, 13 Nov 2026)

Lab: claude. Item: [one-model-end-to-end](../../roadmap/items/one-model-end-to-end.md) (#3407). Unit: [claude-361](../../loop/tasks/one-model-end-to-end/claude-361/ORDER.md), PR #3951. The run goes ahead on this lab's own pre-run review (CHARTER §5.1), inside the caps, and is logged for the owner.

## Question

Does a place-read model trained with its key on the two tokens before the link word, and with its question carried three rows, state a told value more often than the same model trained with the key on the two tokens nearest the place?

- **Today's key (claude-351, "near"):** a candidate place is keyed by the tokens one and two before it, and half of the question is the token before the asking row. By count that read reaches 436 of 562 places at best (claude-350).
- **The new key ("before-link"):** a candidate place is keyed by the tokens two and three before it, it is a candidate only when the token just before it is a link token, and the whole question is the ask heads' learned code, which now takes the question carry (three rows). The place pull trains that code toward the new key.
- **Why this key:** claude-360's counts say the two tokens before the link word carry the find (546 of 562 with every pairing kept). Step 1 of this unit cut the count form to what a trained code can hold, at most two targets a token: 482 to 530 of 562, above the 436 ceiling of today's read.
- **What is already known against a large gain:** in claude-359's first run, setting the place from the counted table with speak off moved the value questions' last word from 203 to 221 of 509 on one model; speaking the place moved it to 294. A better find alone was worth little at serving. This run asks whether **training** with the better key moves more than serving a better find did.

## Base artifact

- **Recipe:** claude-358's p arm, word for word (stage 3's recipe, the place read with its pull, `--hamming-copy-longest 714,28`). The k arm changes two things: `--hamming-place` names the link kit, and `--hamming-carry-question 3`.
- **The pin:** `f914310fc4357160e59697d275f65f99897ee191`, a pushed commit of this unit's branch. The trainer and `fbs-snapshot` are built at the pin on the pod; the exporter and the readers on the laptop at the pin.
- **The kit:** stage 3's `train-places.json`, SHA-256 `8d28571a614e4e038fcb52e2661787f38af62933dbf7ddad2ad64582c8a0a9b3`. **The link kit** is that file plus `"link":[261,268,315,404,435,466,517,519,583,915,931,1028,1681,1748]`: the 14 tokens that stand just before three or more labelled places of TRAIN (table v4's rule, fitted on TRAIN only). The pod builds it from the pinned kit and checks that nothing else changed (I-l).
- **Compared with, not rerun:** claude-358's three sealed p models, as exported then:

  | Model | Artifact, SHA-256 |
  |---|---|
  | p-s2 | `f4667934627e99d709533f30505058fdf07912ddf3cfc9ade6ded5f415e8c680` |
  | p-s1 | `9a209883ff74cdeb6936fc11714d4dd4bb63947775609754c6a5f678e39df9bd` |
  | p-s3 | `9615060f4db98a4cc5b21220e454428b816cda1d47256cf85091963692e83d20` |

## Data split

Written by `m-world-recall-corpus`, world v2, protocol 2, as in claude-358's sheet.

| Set | Writer arguments | tokens.u16 SHA-256 | Use |
|---|---|---|---|
| TRAIN, no line | `conversations=60000 seed=13 recall=off` | `e25775f3aad4548e238736e02a3185e6c57062b8962a4147aafb655a6d865f32` | training; the link tokens were fitted on it |
| First panel | `conversations=1000 seed=14 recall=off` | `b8ad9fe401226b6df8677d06b3f2ad60ea4ffdf6a0b8092970c69e57c4355fe4` | the pod's float pass (1,108 prompts); reported replies. **Step 1's counts were read on it**, so it carries no bar |
| Second panel | `conversations=1000 seed=16 recall=off` | `cd0c9c8e9f29662e4541a2cc9bce9adbb130b17a10304a2e76a8a15503bffd8f` | **the bar's panel** (981 prompts, 578 recall questions, 71 never-told). Step 1's counts and claude-360's census read it too (below) |

- The prompts are claude-359's two samples files: 1,108 rows, SHA-256 `79cfe22ce175830708376680e7ecbc5a4d29e5968c040af5a49180d4ecdd39cf`, and 981 rows, SHA-256 `a8e8de7249b08e9190be7eda27d9db077d1aee800cbfeb39f09beaadb6664906`. Every reply is read on the whole conversation, as claude-359's were.
- **The second panel is not unseen by the design.** Step 1's counted table was read on both panels (about 30 counted configurations; the kit's `cuts-1.table` and `cuts-2.table`), and claude-360's census, which named the offsets, read the second panel (565 of 578). The choice of the key's offsets, the link rule and the carry saw those counts. The three p models have read it too (claude-359). No model trained with the new key has read it, and nothing was fitted on its tokens: the link tokens are fitted on TRAIN. The lean is toward the new key, so it can help only "adds"; row 1 therefore needs a fresh panel (Bars).
- Both panels share the world and its templates with TRAIN. They test unseen conversations of the same kind, not open text.

## Arms

Two pods, one lane each, three stages a lane.

| Lane | Stages | Why |
|---|---|---|
| a | `pre`, `k-s2`, `k-s3` | the new key at seeds 2 and 3 |
| b | `pre`, `k-s1`, `p-s2` | the new key at seed 1; then today's key at seed 2 again at this pin (the pin check) |

- Each k model is compared with claude-358's p model of the same seed: the same data, steps (16,384) and recipe, apart from the key and the carry.
- **Two changes ride together,** the key's offsets with its link rule, and the carry. This run cannot separate them. The near key refuses the carry in code, so "carry only" is not a model that can be trained.
- Not run: a k arm with the carry off; a fourth seed; a k model with the place read off.

## Readings

**On the laptop, as the integer artifact,** exported as claude-358's were (`fbs-export --legacy-artifact --raw-gates --raw-transition --run-config config.json --context 8256`). Replies come from `raw-id-replies --mark-reply 1 --max-tokens 48 --whole-prompt` with no asks table: each artifact serves its own read.

- **V:** recall questions on which the reply states the told value whole, by claude-358's whole-word scorer, of 578 on the second panel.
- **D = V of the k model − V of claude-358's p model of the same seed.** **M** is the mean of D over seeds 1, 2 and 3. The p models read 320, 314 and 325 (mean 319.7).
- **Reported, no bar:** the same reading on the first panel (of 562; the p models read 300, 316 and 311); replies by kind on both panels (never-told replies that say so, value questions' last word, false "not told", other replies exact, replies that never end); the replies word for word for seed 2; each long run's `place.jsonl` (the pull's mean and the place's votes) beside claude-358's.

**On the pod, in float:** `train-in-space --score-rows` on each job's last checkpoint over the first panel at `--context 192`, greedy: the exact-reply count by kind, and the trained model's rows for I-f.

Not measured: open text, reworded questions beyond the world's templates, conversations longer than the context, tokens a second.

## Bars

Fixed before the run. They route the next unit; none stops the line of work.

- **Adds:** M at or above +40 of 578.
- **Does not add at this configuration:** M under +10.
- **Not called:** M from +10 to under +40.
- **A fresh panel behind "adds".** The design saw counts on the second panel, so row 1 is acted on only after the same M is read on a third panel: the same writer at `seed=17`, written after the run, read by no count and no model before. M there must be at or above 6.9% of its recall questions (+40 of 578 is 6.9%), after the pin check's offset is taken off when its size is 10 or more. Under that, the reading is row 3.
- The never-told reading has a prediction and no bar: nothing in this build teaches the reply to say "not told".

## What each outcome unlocks

Rows are checked from the top; the first that matches owns the cell.

| Row | Reading | Classified as | Next |
|---|---|---|---|
| 0 | an instrument check below fails | NOT_RUN, an instrument failure; no reading is classed by rows 1 to 3 | repair and rerun under a new root (a new pin if the code changed). A failed I-f or I-v is split by side first: the trainer's find against the reader's on the same rows |
| 1 | **Adds** | training with the before-link key and the carry states more told values | the key and the carry join the line's recipe and the k models become the line's models. It is called *replicated* only if all three D are above 0. The next unit trains what an open row speaks, starting from the k recipe |
| 2 | **Does not add** | this configuration closes (CHARTER §4.1): this key and carry, with the place as one vote among the copy candidates | the p models stay the line's models. The next unit trains what an open row speaks, starting from the p recipe. `place.jsonl` is read first, to say whether the find itself moved |
| 3 | **Not called** | not shown either way at three seeds | the per-seed table goes in the REPORT. The k recipe is carried forward only if M is above 0 and no named reading is worse than the same-seed p model's by more than 12 replies on two or more seeds; otherwise the p recipe. The named readings: V on the first panel (of 562); other replies exact on the second panel; replies that never end on the second panel (more is worse). Nothing closes |

**Prediction, written before the run and before the fitness review** (a calibration record): M between 0 and +45, "not called" the most likely row. Never-told replies that say so: 0 to 3 of 71 on each k model. The k smoke's rate within 1.5 times the p smoke's. Why small: the served-find result above. What I cannot size: whether a model trained on better finds learns to give the place's vote more weight.

## Instrument checks

- **I-p:** a clean checkout at exactly the pin on each pod.
- **I-g, the new code's first run:** `cargo test --release -p uor-r4-float-train --features cuda --lib -- longe leave_rule marker_check place carry` must pass, with nine named tests shown `ok` (the two copy-rule tests of claude-357 and the seven of this unit). Then six set-aside patches of the pinned checkout (`docs/loop/tasks/one-model-end-to-end/claude-361/neg/*.patch`, each checked against its SHA-256 in the job script) are applied one at a time to a scratch copy: the copy must compile and the test that pins that rule must fail. A test that passes with its rule out, a copy that does not compile, or a named test missing is row 0 before anything trains. The copy is deleted and the pinned checkout checked clean.
- **I-3:** TRAIN's, the first panel's and the kit's files match their hashes before training.
- **I-l:** the link kit equals the pinned kit plus the 14 link tokens above and nothing else; its SHA-256 is recorded by `pre` and each long run checks its kit against that record.
- **I-s:** two 64-step smokes at seed 2, k and p. Each must train and write a `place.jsonl` line at step 64 with a finite mean pull. On k that line shows that labelled questions with a link token before the place exist in the batches.
- **I-m, I-k:** the marker line and the keys-left lines of claude-358's sheet, on each smoke and each long run.
- **I-n:** each smoke's eight `[memory]` lines equal claude-358's p smoke's character for character: the key and the carry change nothing in turn memory's tables. The `step 0` lines are recorded beside claude-358's and are not a stop.
- **I-r:** each smoke's rate projects 16,384 steps inside 2.5 hours, or no long run starts.
- **I-b, the bins road:** on each smoke's checkpoint the snapshot and the export work, the artifact holds `"copy_longest":[714,28]` and a place section; **k's snapshot holds the link bit on exactly 14 tokens and its artifact the carry section, and p's neither**; the float readouts and the compare end on k as on p. Then the scoring pass's own arguments at full size on k's smoke checkpoint, which loads a checkpoint that holds the link tokens and the carry.
- **I-t:** each job ends exit 0 with `models/step-016384`, a snapshot and its `config.json`, and the config names its arm's kit and carry.
- **Pin check:** `p-s2` at this pin against claude-358's p-s2: the float pass's exact replies of 1,108 within 10 of 188, and V on the second panel within 15 of 314. Outside either, the k models are compared with nothing until the cause is found (row 0). Inside, the offset (V minus 314) is reported. **If its size is 10 or more, row 2 is not called (the reading is row 3), and row 1 needs M minus that size at or above +40.**
- **I-o, today's artifacts read as today:** the reader built at this pin, on claude-358's three p artifacts and the second panel, writes reply files whose SHA-256 begin `f17cc0082be8395f` (p-s2), `3cf266fea38bfc31` (p-s1) and `3400728f028789cc` (p-s3), claude-359's files. Otherwise row 0.
- **I-w:** the whole-word scorer marks 578 of the second panel's 578 references right.
- **I-h:** each k artifact holds a carry section and 14 link bits; the retrained p-s2 holds neither.
- **I-f, the row check, read before any reply:** `row-picks --context 192 --mark-replies 1 --float-rows` gives the share of scored rows on which an artifact chooses its trained model's token. It is run at this pin on claude-358's p-s2 artifact and on the retrained p-s2, each against its own float rows; the lower of the two is the reference. **A k artifact's share more than 0.010 under the reference is row 0:** an export or reader fault under the new key, not a reading of the key. A share above the reference is no fault. I-f reads every scored row, so a fault on the few value tokens moves it little; I-v is the check for that.
- **I-v, the served reply is the trained model's, read before any k reply on the bar's panel:** on the first panel's 562 recall rows, at the float pass's own window prompts (`raw-id-replies --mark-reply 1 --max-tokens 48`, no whole prompt), each artifact's served reply is compared with its float pass's reply, token for token up to the end mark. Two shares: of all 562 rows, the replies that are the same; and of the rows where the float reply equals the reference, the served replies that are that same reply. Measured on claude-358's three p models with claude-359's reader: 0.859, 0.852 and 0.772 of all rows; 0.988, 0.973 and 0.975 of the float-exact rows (82 of 83, 72 of 74, 78 of 80). **A k artifact under 0.70 on the first share or under 0.93 on the second is row 0:** the reader does not serve what was trained under the new key. The retrained p-s2 is read the same way; under either floor there, the fault is the reader at this pin (row 0). The trainer and the reader call one find routine, so a fault inside it is the same on both sides and I-v cannot see it; the nine tests and six set-asides of I-g are the check for that. I-v reads the model's 192-token window. The bar reads the whole conversation, where the float pass gives no reference: a reader fault under the new key that shows only past the window has no gate here (I-o checks the whole-conversation road for the near key only).
- **Seal:** each stage's `run/` is sealed with a file list and sums, put in the store under `results/claude/one-model-end-to-end-key-two-three-2026-11-13/run/<stage>/<try>/` and read back. A run is never written into an existing root (L-002). The lanes run under different try names.

## Risks the build leaves open

- **The float code of the key has never run.** The laptop cannot link the float crate's tests. I-g is where they first run, with their set-asides.
- **Three fresh reviews of the code found no defect and listed test gaps** (rotations are the identity in the pull's tests; one window only; the two ask heads hold the same code; the load path of a linked checkpoint has no unit test). The pod's scoring pass and I-b's link-bit count exercise the load path on a real checkpoint; the others stay open.
- **A served k model reads its key from its class table:** an export that lost the link bits would be read by the near key without an error. I-b and I-h count the bits.
- **A training window starts mid-conversation,** so a place within three tokens of the window's start is not labelled under the new key (a fourth position or later is needed).
- **The link tokens are a count from TRAIN,** a fixed list in the kit, not a learned quantity. Whether that sits inside CHARTER §3.1 for a served model is with the owner; this run reads the model, and serves nothing as a product.

## Minimum test power

- **Noise.** The three p models read 314, 320 and 325 on the second panel and 316, 300 and 311 on the first: a standard deviation between seeds of 5.5 and 8.2 replies. Taking 8.2 for both arms and no pairing by seed, D has a standard deviation near 12 and M near 7. Three models an arm is a loose figure.
- **The bars against it.** +40 is near 6 standard deviations of M, so chance does not reach it. A true +40 reads "adds" about half the time; a true +60 nearly always. A true 0 reads "does not add" about 9 times in 10; a true +25 reads "not called" about 9 times in 10.
- **An assumption.** The figures above take the k models to spread between seeds as the p models do. A wider k spread makes "not called" more likely at every true effect, and three seeds cannot test it.
- **What it cannot show:** which of the two changes did it; anything about open text.

## Steps

1. **On the laptop, before the pods:** the kit at the pin (job, chain, seal, watcher, gate), syntax-checked; the six set-aside patches apply at the pin and their patched sources type-check.
2. **On each pod, before anything trains:** I-p, I-g, I-3, I-l, the two smokes with I-s, I-m, I-k, I-n, I-r and I-b.
3. **The long runs of the Arms table,** each with its scoring pass, each sealed on exit.
4. **Each pod's four checks,** then that pod goes down.
5. **On the laptop:** I-o, I-w, the exports with I-h and I-t, the pin check, I-f and I-v; then the replies on both panels, V, D, M and the row. A third panel is written and read only if M reaches +40.

## Cost

Two pods at no more than $1.60/h each. claude-358's pod cost $0.91/h and its runs trained 32 to 34 minutes each with a scoring pass after. Expected: about 3 hours a lane, $6 to $10 in all. Ceiling: 6 hours a pod, $19.20. Two of the four pods and at most $3.20/h of the $8/h cap; `uor-pod status` is read first.

## Compute and stop rule

- Each pod is launched one at a time and watched to SSH and its job's start. The watcher is started before anything ships (L-004); it renews the lease and never cancels a healthy run. Time, spend and stall are notices only.
- A long run starts only if `pre` passed on its own pod and I-r projects it inside 2.5 hours. A healthy run past that is extended and the extension noted.
- Every stage seals `run/` on every exit, a signal included. A scoring pass is capped at 40 minutes and never fails its job.
- A pod goes down when its own four checks pass: the chain ended with every job 0 and every seal 0, the required files are in each seal's list, the store's sums read from the laptop equal the pod's, and no GPU process is left.

## FIT

Two fresh non-author reads of this sheet, each by a `delegate:reviewer` subagent given the sheet and no plan, and one of the pod scripts.

- **First read: FIT WITH CHANGES.** One HIGH: the row check I-f would not catch a reader fault on the few value tokens. Three MEDIUM: the pin check's tolerance was wider than the lower bar; row 3 named no readings and no margin; the sheet said no count had read the bar's panel, which was false. All were taken: I-v, the pin check's offset rule, row 3's named readings and margin of 12, the corrected Data split text and the fresh third panel behind "adds".
- **Second read: FIT.** Each finding marked met. Three LOW items, all taken into the text: I-v's wording ("before any k reply"), what I-v cannot see past the window, and the third panel's rule under a pin offset.
- **The pod scripts' read: launch OK with low items.** One MEDIUM, taken: the link kit is written under the try's own folder, so a retry on a reused pod does not meet an earlier try's file.

TEST FITNESS: FIT - research, #3407 (claude-361), two fresh non-author reads on 13 November 2026
