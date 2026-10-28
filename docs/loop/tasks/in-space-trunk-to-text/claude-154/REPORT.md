# REPORT: in-space-trunk-to-text, claude-154

Lab: claude. Order: [ORDER.md](ORDER.md). Prep sheet: [in-space-trunk-to-text-opening-read-2026-10-28](../../../../harness/prep/in-space-trunk-to-text-opening-read-2026-10-28.md). TEST FITNESS: FIT, round 1, and PRE-RUN: PASS, by fresh non-author reviewers (owner log on #2028, comment `6473278753`).

The reading, by the sheet's bars: **(o), the opening read on the turn's first 8 tokens, is cell D**, with no flag F. Its openings, read as the mean of the last three checkpoints, are `206.0` and `156.3`. That is below its bar of `347.3`, and below claude-152's (a) itself (`302.0` and `228.0`). By the sheet, this configuration closes, and the next unit tries another way to carry the prompt's start to the reply. The item stays open.

**Speed.** On these four Vast hosts, the host set the speed more than the arm did. Context tokens/s in training:

| Arm | Seed 1 | Seed 2 |
|---|---|---|
| (o), turn anchor | `30,228` (pod 1) | `9,831` (pod 2) |
| (oL), last anchor | `28,844` (pod 3) | `20,186` (pod 4) |
| (a), claude-152, no opening read | `32,049` | `32,329` |

- **The fastest pod puts the read's cost at about 6% or less.** Pod 1 trained (o) at `30,228`, against (a)'s `32,049` on claude-152's pods. Those are different hosts, so this is a bound, not a measurement.
- **The other hosts ran slower whatever the arm:** pod 2 trained at a third of pod 1's rate on the same binary and model. This unit therefore makes no other speed claim.
- **Decode,** in this runner (float, 64 tokens after 16, 64 and 192 context tokens, re-running the window for each token), pod by pod:
  - (o) `165 / 156 / 147` tokens/s (pod 1) and `165 / 147 / 112` (pod 2);
  - (oL) `128 / 121 / 114` (pod 3) and `76 / 73 / 64` (pod 4);
  - (a) `212 / 198 / 180` and `211 / 198 / 180` (claude-152's pods).

  These figures mix the read's host-side work with the host's speed, so they set no cost for the read. There is no integer serving path for it yet.

## What did not work, first

- **The opening read does not lift openings.** `Ō`, the mean of `O` (of 1,226) at steps 12,288, 14,336 and 16,384:

  | Arm | Seed 1 | Seed 2 | Bar |
  |---|---|---|---|
  | (o), turn anchor | `206.0` (`195, 199, 224`) | `156.3` (`153, 168, 148`) | `≥ 347.3` against (a); `≥ 355.3` against (oL) |
  | (oL), last anchor | `196.7` (`184, 219, 187`) | `309.7` (`301, 298, 330`) | control |
  | (a), claude-152 | `302.0` | `228.0` | reference |

  - Bar 1 fails at both seeds, and so does bar 2: (oL)'s better seed is `309.7`, plus `45.6`. That is cell D.
  - Seed for seed, (o) is below (a) by `96` and `72`. The sheet reports this gain and does not bar it; it is about `1.4` to `1.8` of (a)'s seed SD each.
- **The trunk barely uses the turn read (I-5).** Each final checkpoint was re-scored with its opening read removed:

  | Arm | `T` with / without | `O` with / without | Copies with / without |
  |---|---|---|---|
  | (o) seed 1 | `9,320 / 9,295` | `224 / 223` | `1,820 / 1,804` |
  | (o) seed 2 | `9,487 / 9,468` | `148 / 145` | `2,081 / 2,069` |
  | (oL) seed 1 | `9,164 / 9,033` | `187 / 170` | `1,533 / 1,488` |
  | (oL) seed 2 | `11,055 / 10,772` | `330 / 281` | `3,313 / 3,229` |

  - The read reaches the evaluation at every arm, so I-5 passes.
  - Taking the turn read out of (o) costs `25` and `19` tokens and `1` and `3` openings, although its weight ends at `0.589` and `0.509`.
  - The same read on the last 8 tokens costs (oL) `131` and `283` tokens and `17` and `49` openings when it is taken out. The trunk uses recent tokens handed to it more than the turn's first words.
  - So (o)'s lower openings do not come from the read acting at test time. They come from how the trunk trained beside it, or from seed noise.
- **Classified: configuration, not instrument.** Every instrument check passed at every seed:
  - the anchor agrees with the probe at every DEV and TRAIN opening;
  - the read's two CUDA tests pass;
  - step 0 equals (a)'s;
  - the read is live in the evaluation.

  D closes this configuration only (CHARTER §4.1): the mean of the turn's first 8 table points, turned per slot, added after layer 2, at 16,384 steps.

  A candidate cause, not tested here: the read averages 8 points into one vector, so the trunk receives the user's words as one superposed point that layers 3–6 would have to separate.
- **No checkpoint meets the item's done-when 2.** `R16` is at most `0.029` at step 16,384 (from `0.73–0.75` at step 0), against `0.05`. Done-when 1 is met at two final checkpoints: (o) seed 1 (`O = 224`, `T = 9,320`) and (oL) seed 2 (`330`, `11,055`).
- **The replies still template.** At (o) seed 1, step 16,384:
  - "My lucky number is 5246." gets "Got it, your lucky number is 526.";
  - "Remember, the code word is kandgind." gets "Rem is kandgal.";
  - "Add 4 and 2." gets "A new is asking up with exciting can number wonderful feel when friend like.".

  (oL) seed 2 gets "Got it, your friend is 524." for the lucky-number prompt and "Anytime!" for "Add 4 and 2.". The user's numbers come back nearly, not exactly.
- **Fit at (o) seed 2 is below (a)'s:** `T = 9,487`, against `10,573` at the same seed (flag F is `T < 8,828`, so it is not set).

## What worked

- **The read is built and checked end to end.**
  - Its slots are exactly the probe's first-8 key at every opening: `1,226` of `1,226` DEV openings and `212,968` of `212,968` TRAIN openings (`--opening-bound`).
  - It enters the trunk with no effect at step 0 (I-2), and the evaluation sees it (I-5).
  - Its own tests pass on the GPU, and three negative gates bite.
- **Copies rose at (o) seed 1:** `1,820` of 4,130, against (a)'s `1,517`. At seed 2 they are `2,081` against `2,933`.
- **(oL) seed 2 fits best of any width-256 final checkpoint on this item:** `T = 11,055`. claude-152's best, (a), read `10,573`, and claude-146's and claude-151's step-16,384 checkpoints read `9,233` or less (claude-152's REPORT). Its seed 1 reads `9,164`, so this is one seed.
- **The measurement now separates "seen" from "used".** I-5's removal re-score shows how much a trained model depends on a read, at no training cost. It gives the next mechanism a direct check.

## What changed

- **`crates/uor-r4-float-train/src/flock_read_opening.rs`** (new):
  - `opening_slots`, the turn anchor (after the last `<|bos|>` or `<|eos|>`) and the last anchor;
  - `OpeningRead`, a learned quaternion per slot and lane (identity at the start) and a learned weight per coordinate (zero at the start). Its `apply` adds the mean of the kept slots' turned table points (the table detached), and it saves and loads to `opening_read.safetensors`.
  - Tests: the slot rule and save/load (host); zero weight is no read, and the read adds the mean of its turned slots (CUDA).
- **`geometric_stack_native.rs`:** the hook in `StackModel::hidden`, after layer 2 and beside the Hamming read.
- **`in_space_long.rs`:**
  - `LongConfig.opening_read` and `opening_anchor`, left out of `config.json` when the read is off;
  - the start, a separate optimizer, the checkpoint save and load, and `ArmGuard` cleanup;
  - `opening_weight_mean_abs` in `--inspect`;
  - the anchor check in `--opening-bound`;
  - host tests for the config keys, the checkpoint load and the anchor check.
- **`bin/train-in-space.rs`:** `--opening-read K` (1 to 16) and `--opening-anchor turn|last`, which is refused without the read.
- **The prep sheet, this REPORT and its notebook row.**

With neither flag, every file a run writes is unchanged: a host test pins the config keys.

## Measurements

### The runs

`train-in-space --objective cell --hamming-read 4 --opening-read 8 [--opening-anchor last] --retain-rates 1 --decay-mult 1 --batch 16 --steps 16384 --eval-every 2048 --lr 0.001 --lr-end 0 --seed S` on the pinned TRAIN at width 256. Each arm ran alone on one RTX 5090. Every pod built and tested its own binary from `060ba8127855db6ab1b198a7c59dd932e0b85b80`.

At step 16,384, seed 1 / seed 2:

| Arm | `T` (of 14,287) | `O` (of 1,226) | `Ō` (last 3) | Copies (of 4,130) | Rest (of 9,094) | Opening picks | `R16` | Opening read weight | Train loss |
|---|---|---|---|---|---|---|---|---|---|
| (o) | `9,320 / 9,487` | `224 / 148` | `206.0 / 156.3` | `1,820 / 2,081` | `7,361 / 7,327` | `83 / 65` | `0.0293 / 0.0284` | `0.589 / 0.509` | `0.634 / 0.638` |
| (oL) | `9,164 / 11,055` | `187 / 330` | `196.7 / 309.7` | `1,533 / 3,313` | `7,518 / 7,527` | `57 / 75` | `0.0248 / 0.0263` | `1.118 / 1.100` | `0.614 / 0.592` |
| (a), claude-152 | `9,328 / 10,573` | `306 / 245` | `302.0 / 228.0` | `1,517 / 2,933` | `7,600 / 7,490` | `73 / 80` | `0.0198 / 0.0098` | none | `0.604 / 0.618` |

- `R16 = shift(16) / shift(0)` from `--reach`. At step 0 it reads `0.746` at seed 1 and `0.730` at seed 2, the same for both arms.
- The Hamming read's weight ends at `0.610 / 0.544` in (o) and `0.472 / 0.511` in (oL).
- **The bars,** from the sheet:
  - bar 1 is `302.0 + 45.3 = 347.3`;
  - bar 2 is `309.7 + 45.6 = 355.3`, at `p = 309.7 / 1,226`;
  - flag F is `T < 8,828`.

**Instrument checks,** at every seed:

- **I-A:** at `060ba812` on the laptop, DEV `1,226 / 1,226` and TRAIN `212,968 / 212,968` openings agree with the probe's first-8 key. The same run reproduces the probe's `607` (first 8), `227` (first 4) and `161` (last 4).
- **I-1:** `131 passed; 0 failed; 5 ignored`, as pre-registered, on a CUDA run before any arm and on each of the four pods' builds.
- **I-2:** every arm's step-0 `dev_nearest` equals (a)'s at the same seed: `2/0/730` and `5/3/681`. It was checked on each 256-step probe and on each arm. Each `config.json` records width 256, 4 Hamming heads, `"opening_read": 8`, its anchor and the pinned TRAIN.
- **I-3:** both reads' weights end above 0 at every arm (in the table).
- **I-4:** all 17 losses of every arm are finite: 8 training-loss means and 9 DEV losses (`progress.jsonl`; the job log's I-4 line).
- **I-5:** every arm's re-score without the read differs from its re-score with it (the table above). The in-training evaluations that `Ō` is read from run on the thread where training set the read, as the Hamming read's did in claude-152.

### Evidence

- **Location:** `caseyallard/uor-r4-store:results/claude/opening-read-2026-10-28/`. It holds one tar per pod, named by pod and arm: `pod1-o-s1.tar`, `pod2-o-s2.tar`, `pod3-oL-s1.tar` and `pod4-oL-s2.tar`. Each holds the pod's job folder:
  - the job log and every step's out, err and exit;
  - each arm's config, progress, samples and checkpoints;
  - the probes, and reach, inspect, decode and the no-read re-score;
  - `uor-pod`'s job logs;
  - in pod 1's tar, the pre-run CUDA test run.

  `job.sh`, `cuda-tests.sh` and `watch.sh` are beside them.
- **Seals:** each tar has its own `.SHA256SUMS`, written on the pod. `SHA256SUMS-claude-154` lists all four, sha256 `2ef20cc6bfcf205cc47d616f2995a54452ca1e3c7cf7a1274db02ea3aac8e69c`:
  - `pod1-o-s1.tar`: `fb0ea491ef3c897594177ffaa39ec0a0a5906ac5a546da14f1effa67582a1315`;
  - `pod2-o-s2.tar`: `fdbc15b2ea6ce357cb165786e5146fd6c4b71888759a0b0ae3d978d9992c845b`;
  - `pod3-oL-s1.tar`: `28e36c3da3c6e6227fc97b3332090d18ec8d2bce18545d7967dbe01d830b03e0`;
  - `pod4-oL-s2.tar`: `8e9934e598e6dcd37ae00b75832fbe006e5edf25430d9aed9b53b34c6d5911ed`.
- **Checks:** every tar was read back from the store to the laptop and checked against the hash the pod wrote, before that pod came down.
- **Cost:** the runs took four Vast RTX 5090 pods, 3.82 pod-hours, about `$3.68`. The review's GPU checks added `$0.24` over 0.43 pod-hours: `$0.05` for the pod that ran them and `$0.19` for a create that came up exited and was deleted. About `$3.92` in all; every pod is down. The cheapest offers ran out after the first, so pods 2 to 4 cost `$0.99–1.11/h`:
  - pod 1, `$0.37` at `$0.563/h`;
  - pod 2, `$1.59` at `$0.989/h`;
  - pod 3, `$0.72` at `$1.094/h`;
  - pod 4, `$0.99` at `$1.106/h`.

## Checks run at the head

**The runs' code is `060ba812`.** After the runs, the in-unit code review led to one more code commit, `4b6eb43d`, which touches only `flock_read_opening.rs`:
- **Its tests:**
  - the summary test now covers every position, partial and empty slot lists included, with a general quaternion in every slot and lane;
  - new tests pin the slot tables' counts and masks (host), that gradients reach the rotation and weight but never the table (CUDA), and that `load` refuses an out-of-range read (host).
- **`load`** repeats `new`'s range checks on `k` and the width.

A valid read computes exactly what it computed in the runs: `apply`'s host tables moved into `slot_tables` unchanged.

- **CUDA library tests on an RTX 5090 at `4b6eb43d`:** `134 passed; 0 failed; 5 ignored`, all 7 of the read's tests among them. Sealed as `pod5-cuda-gates-4b6eb43d.tar` (sha256 `2e6ff66bafd367d7c5cefe851619d838b2b93f55d72251894d554ef34061a943`), read back and checked.
- **CUDA tests at the runs' commit `060ba812`:** `131 passed; 0 failed; 5 ignored`, on the pre-run CUDA run and on each of the four pods' builds.
- **Host at the head:** every host test target of the crate passes: 101 passed, 0 failed (99 at the runs' commit, plus the two new host tests).
- **Negative gates,** each set aside, run and restored:
  - host: the turn anchor forced to the window start; the config's skip rules removed; the checkpoint load removed; the slot count taken as `k`; `load`'s range check removed; the anchor check's offset moved by one. Each fails its test with exit 101.
  - GPU: the slot mask's multiply dropped from `apply` fails the summary test: exit 101, `the_opening_read_adds_the_mean_of_its_turned_slots ... FAILED`.
- **`uor-harness loop report --check`:** PASS. **`loop gates --full`:** 11 of 11 green.
- **In-unit reviews,** each by a fresh reviewer that never saw the plan:
  - **FIT:** FIT in round 1, with four cosmetic notes applied. **Pre-run review:** PASS, with two notes applied. Both are listed in the sheet.
  - **The code review of the module: CHANGES, for its tests only.** It found no defect in the code. The summary test checked two positions with every slot kept, so a wrong count or a dropped mask would have passed; it also noted axis-only rotations, `load`'s missing range checks and no gradient test. All four are fixed in `4b6eb43d`, with the gates above.
  - **The code review of the wiring: CHANGES, pending two checks.** It found no missed site.
    - **The forward hook has no unit test.** I-5 answers this: every arm's re-score with and without the read differs, on the GPU, and the hook is the only place the read reaches the forward pass.
    - **An old `config.json` might not load.** It cannot happen: `LongConfig` derives only `Serialize` and is never read back.
    - Its offset gate on the anchor check bites (above).
  - **The claims check of this REPORT and its notebook row: APPROVE.** It recomputed about 95 claims from the sealed tars and the pod ledger. Its four slips are fixed:
    - one reply matched to the wrong prompt;
    - the cost rounded down (`$3.676` is `$3.68`);
    - I-4's count of 17 with its source;
    - a "best of any width-256 run" claim, now scoped and cited.
  - **A fresh re-review at the final head:** APPROVE.

## Skills used

- **holding-the-goal.** Casey asked to "carry the prompt's first words to the reply's first token, scored against the same prompt-use test". The read hands the trunk exactly the tokens that name 607 openings by lookup, checked at every opening. The test is claude-152's, read the same way for the arms and the reference.
- **verify-before-done.** Each piece was proved before the runs: the anchor against the probe, the read's tests on the GPU, three negative gates, and step 0. After the runs, I-5 checked that the evaluation saw the read, which turned up the main finding: the trunk barely uses it.

## Still open

- **The next mechanism, by the sheet's D.** The sheet's next step is exact memory for the opening against a no-memory control (L-010). The design discussed with Casey on 28 Oct, a **prompt register**, carries the prompt's start in a learned way. Q learned 64-bit query codes, say 8, each pick the K best-matching positions of the user's turn by Hamming distance to codes from the trunk's state. Each query averages its picks into one slot, and every reply position reads the Q slots. This unit's I-5 finding shapes it:
  - keep the slots as separate vectors, not one mean;
  - let them reach the readout, not only an early layer;
  - check each slot by removal, as I-5 did.

  Its controls would be this unit's opening read (fixed positions) and a plain bag (uniform weights over the turn). Building and serving it costs Q × turn length bit counts once per user turn, and Q slot additions per generated token.
- **Openings swing between seeds and checkpoints** (here `156` to `310` across four arms). A bar on two seeds can only see large effects. More seeds, or a reading less tied to one panel, would let smaller gains count.
- **Speed on mixed hosts:** a later sheet can pin speed to one host type, or time every arm on one pod.
- **claude-152's four low-severity code-review notes** stay open for the next unit that touches that code.

## Plain summary

Casey asked to carry the user prompt's first words to the reply's first token. The test is whether the reply's first word follows the question; a lookup of the prompt's first 8 tokens names 607 of 1,226 openings, and the best model so far names about 300.

- **The read was built and works as designed.** At every reply it hands the trunk exactly those 8 tokens, checked against the lookup at every opening.
- **It did not help.** With the read, the model names `206` and `156` openings, against `302` and `228` without it, and the bar was `347`.
- **The model learned to ignore it.** Taking the read out of the trained model changes almost nothing: 25 and 19 tokens of 14,287. The same read on the last 8 tokens instead is used more.
- **So handing the trunk the words is not enough** when they arrive averaged into one point. The next design keeps the prompt's pieces separate and learns which to pick.

The unit cost about `$3.92` in all, `$3.68` of it the runs, and all pods are down.

## Done when

1. **Met.** The opening read is built. Its tests:
   - the slot test (i) is a host test, and its negative gate, the anchor forced to the window start, failed it (exit 101);
   - the zero-weight test (ii) and the summary test (iii) are CUDA tests, which passed on the pre-run CUDA run and on every pod's build (`131 passed`).

   The host suite is green (99 passed).
2. **Met.** `--opening-bound` prints the anchor check: DEV `1,226 / 1,226` and TRAIN `212,968 / 212,968` agree, with no disagreement to classify.
3. **Met.** (o) and (oL) ran at seeds 1 and 2 under one sheet with a fresh FIT and pre-run review. Openings were read as the mean of the last three checkpoints, for the arms and the reference alike.
4. **Met on merge.** This REPORT leads with tokens/s, and the result has one notebook row ([float-training](../../../../research/notebook/float-training.md)).

The item's own done-when is not met, so it stays open.
