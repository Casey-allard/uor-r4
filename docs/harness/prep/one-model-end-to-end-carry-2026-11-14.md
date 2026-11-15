# Prep sheet: the recurrence's state carried across training windows, two arms, two seeds (one-model-end-to-end, claude-380, 14 Nov 2026)

Lab: claude. Item: [one-model-end-to-end](../../roadmap/items/one-model-end-to-end.md) (#3407). Unit: [claude-380](../../loop/tasks/one-model-end-to-end/claude-380/ORDER.md), PR #3979, roadmap step V1 ("train as served"). Follows V0, claude-376 (PR #3970). The runs go ahead on this lab's own pre-run review (CHARTER §5.1) and are logged for the owner.

## Question

When training carries the recurrence's state from one window into the next, with backprop truncated at K = 4 windows, does the model state a value told 256 or more tokens earlier more often under the served carry than the same recipe trained with every window starting from zero?

- **What exists.** Scoring and the integer session already carry the state across windows (claude-278's `PastWindowRule::Carry`, claude-288). Training does not: every window starts from a zero state, so the retention rates (`--retain-rates 1`) are fit to 192 tokens and never to the context the server reads.
- **What this unit adds.** `--carry-windows K` (commit `f0d74adb9739e0b2a59cd5ddea7526e311d75776`). It cuts each conversation back to back into windows. Each step holds B/K lanes of K consecutive windows. The scan starts each window from the state the lane's previous window ended in, which is zero at a conversation's first window. The convolution reads the previous window's last three inputs. One backward runs through the K windows, and then the state is detached. The Hamming reads and the turn memory are not carried: each window's reads see that window only.

## Base artifact

- **The unit's build**: `cargo +1.97.1 build --release --locked -p uor-r4-float-train --features cuda --bin train-in-space --bin fbs-snapshot`, `UOR_NVCC_ARCH=compute_89` (RTX 4090). It is built on `wick` from a clean checkout of the pin, this unit's code commit `f0d74adb9739e0b2a59cd5ddea7526e311d75776` (the chain refuses an unclean checkout). The binaries' SHA-256 go into `pre/binaries.sha256` and the REPORT. One build trains and scores every model, so no comparison spans two builds.
- **The control** is arm `off`: the recipe without `--carry-windows`, trained by the same build, on the same card, at the same time as `k4` of its seed. With the flag off, the pin's trainer is the base's. Pre's I-3 checks this on this card: a 64-step run gives the same `config.json`, losses and step lines as the base binary (`d5eb4a55656726b04ea79b06968f91905933c925`).
- **The recipe** is claude-358's `p` recipe, with two changes, both forced and both applied to both arms:
  - **W = 192, not 256.** `--past-window carry` scores at the checkpoint's own context. At `--context 256` the scorer refuses ("12 windows share a scoring forward"): the row stride where several windows share one forward has no test above 192. Training at 192 keeps scoring on the tested path, and the integer session serves any W.
  - **No place kit** (`--hamming-place`). The kit is bound to `corpus-noline`'s TRAIN, and this unit trains on another TRAIN (see Data split).
  - The recipe: `--objective cell --retain-rates 1 --decay-mult 1 --lr 0.001 --lr-end 0 --context 192 --width 256 --hamming-read 4 --batch 16 --turn-memory 8 --turn-memory-reply 1 --turn-memory-dropout 500 --samples 6 --reply-tokens 48 --hamming-copy-latest 1 --turn-memory-leave-copied 1 --hamming-ask 1 --hamming-copy-longest 714,28`, `--steps 16384 --eval-every 2048`. Arm `k4` adds `--carry-windows 4`.

## Data split

The `p` recipe's TRAIN (`corpus-noline`) spans no window: its longest conversation is 246 tokens, so a carry would train what the off arm trains (the ORDER's cancel probe). The runs therefore use claude-275's **long TRAIN** and its held-out panels, from the store (`results/claude/one-model-end-to-end-long-context-2026-11-04/inputs/`). They were copied to the laptop (`local/claude/claude-380/in/`) and to `wick` (`/workspace/in/c380/`), and are hashed again there by pre's I-1 (`inputs.sha256`, `driver.sha256`).

| Input | Path under `/workspace/in/c380/` | SHA-256 |
|---|---|---|
| TRAIN tokens (16,000 conversations, 1,703 to 2,048 tokens) | `train-long/train/tokens.u16` | `61019954ef82d7b33d2d5a101f436fece29452f8580f8f3268e35229ccd0cb3c` |
| TRAIN mask | `train-long/train/response_mask.u8` | `29b34d2c9ed3a56bf638a3afd6863e5744b2c1d3f838a1034f19a77d1e02818d` |
| p-long tokens (609 queries, held out) | `p-long/tokens.u16` | `8b2529b1a1385112ef41a42eca50dc982dc474f3b40e8948b8887b6b4158ab36` |
| p-long mask | `p-long/response_mask.u8` | `b17ff53fc37dfe2910b1ffca820538ca5473d1b0b707da3f06eb83c25c58fed0` |
| p-long queries | `p-long/queries.json` | `cf5c30b96e98f7770314f96be5114cf5eeb21e3f9657aeb84b4c1d6168ca2390` |
| q-tt tokens (617 queries, TRAIN's phrasing) | `q-tt/tokens.u16` | `84f1eb1d01a8de327153635e4367a69148e89b6e9ca55cc6c1dad6fc3197fe63` |
| q-tt mask | `q-tt/response_mask.u8` | `9359992750d3866d9b7ff30da52920f12b3648e0b8a0ddf5436b912333616978` |
| q-tt queries | `q-tt/queries.json` | `e40f1cfe5e45553270547420bbe556347ff8f8ea6d13a5418fbc7dd598c21b36` |
| Tokenizer | `/workspace/in/driver-data/tokenizer.json` | `d36d3e8700a123e620012df77de195f244fbdb4d05d9e1aa7e77fa9407590f89` |
| DEV tokens | `/workspace/in/driver-data/ft/dev/tokens.u16` | `0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b` |
| DEV mask | `/workspace/in/driver-data/ft/dev/response_mask.u8` | `a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078` |

- At W = 192 the carry cuts TRAIN into 172,262 windows; all 16,000 conversations run past one window, and the longest spans 11. The off arm keeps 172,143 usable windows. Both counts come from smoke 3 on `wick`.
- p-long holds 609 queries; 566 are at distance 256 or more, and 560 once six are left out of the bar: queries 0, 42, 93, 401, 471 and 487, whose answer TRAIN itself states (claude-275's census). No panel is read while training.

## Arms

| Job | Recipe | Seed | Steps | Card |
|---|---|---|---|---|
| `train-s1/off`, `train-s1/k4` | the recipe; `k4` adds `--carry-windows 4` | 1 | 16,384 | `wick`, side by side |
| `train-s2/off`, `train-s2/k4` | the same | 2 | 16,384 | `wick`, side by side |

The two arms of a seed share the card at the same time, so a slow card slows both. `k4` starts 120 s after `off`.

## Readings

- **Scoring.** Every model is scored by the unit's build in float on p-long and on q-tt, with `--score-rows CK --samples 16 --reply-tokens 32 --retain-rates 1 --seed S --context 192 --panel P=… --panel-queries P=…/queries.json --past-window carry`. Each pass writes `recall-P-memory.json` and `recall-P-no-memory.json`: one outcome per query in `queries.json` order, with `distance` and `stated` (the reply, cut at its end mark, states the answer as a whole word).
- **The reader** `local/claude/claude-380/read-380.sh` is fixed before any result: SHA-256 `aca56cb81348b25d6a07edb01d328c2e8c7e25bc549f50b1b27f31e37287bd8e`. It is checked again when the results are read and quoted in the REPORT. It is given the try it reads by name (`run/try-N`). It refuses to print a verdict unless pre and the try's last stage (`score-s2`) exited 0, every stage's I-2/I-4 and I-7 files read `ok`, every pass exited 0, the arms' outcomes pair query by query, p-long holds exactly 560 far, census-clean queries, and no other try beside it is a complete good try (its `score-s2.exit` reads 0). A try that failed or crashed part-way may sit beside the one read; a good try is never read beside another good one. Each verdict and each refusal is tested on synthetic run folders by `local/claude/claude-380/test-read-380.sh` (15 cases, SHA-256 `fd87d6088ed97e31ef38ed6c85ad5db352338e5f92d647e7c368be82d78a209d`).
- **Read without a bar:**
  - memory-on counts and every q-tt count (the same pairing);
  - stated by distance bin for each arm;
  - the last `train_loss_mean` and `dev_loss`;
  - seconds per step and peak card memory;
  - V0's panel for each model: exported by `fbs-export` (V0's arguments) and served on the laptop by `raw-id-replies --turn-memory 0` on P14 + P23 (V0's panels, SHA-256 `33bdd07a…a178d` and `ddccde6f…7d99`), giving exact replies and tok/s. V0's own numbers are quoted beside them, never as a control: the models here train on another TRAIN.

## Bars

The order's bar, its counts fixed before any run. Per seed s, on p-long with the memory out, over the far, census-clean queries (distance 256 or more, the six above out), paired between the seed's two arms:

- b(s) = queries stated by `k4` and not by `off`; c(s) = queries stated by `off` and not by `k4`.
- **Carries:** at both seeds b + c ≥ 10 and b − c > 2√(b + c). This is the paired sign test's two-sided 95 % line, in integers (b > c and (b − c)² > 4(b + c)).
- **Does not carry:** at both seeds b + c ≥ 10 and b ≤ c.
- **Floor:** b + c < 10 at both seeds. Too few queries tell the arms apart to call the carry either way.
- **Not called:** every other result, including one seed at the floor.

The turn memory is out for the bar because it can copy a value from anywhere in the conversation (L-010). The Hamming copy reads stay on in both arms alike, so the bar compares a model trained with the carry against one trained without it, served the same way, with the same reads; it does not isolate the recurrence's state from those reads.

## What each outcome unlocks

- **Carries:** V2 builds on it. The carried model is served end to end in integer, with the session's carry, and its replies are read. A third seed runs beside it.
- **Does not carry:** this configuration is closed (K = 4 at W = 192, this recipe, 16,384 steps), not the mechanism. The next reading is the near bins: whether `k4` gains at distances inside the gradient's reach (under 768 tokens) and loses beyond it.
- **Floor:** neither arm states far values often enough for the bar to see the carry. The carry is then read only by what is reported without a bar. The next unit raises far recall in both arms first, for example with more steps or a denser far-value TRAIN, before the carry is tested again.
- **Not called:** seed 3 of both arms, as its own unit with its own order and sheet. No arm, seed or step count is added to this unit after a result is read.

**Prediction, written before the runs** (a calibration record, not a bar): **Floor.** claude-276 trained at a 2,048 window on this TRAIN and stated 0 to 1 of 603 p-long queries; claude-281 found no far recall even on q-tt. A guess of under 10 stated by either arm at distance 256 or more, and `k4` within 1.7× `off`'s seconds per step.

## Instrument checks

- **I-1** every input at this sheet's hash on `wick` (pre).
- **I-2** each run's training log carries claude-358's marker line, as printed at W = 192 in smoke 3: `[hamming] the longest rule's marker 714,28 stands before every reply opening and nowhere else: TRAIN 891090, DEV 1226`.
- **I-3** carry off at the pin is the base. The base binary must hash to `2fa2a89d5d811c7a2bfee1101ec123b4162106daed4861376103e0a5bc0a655a` (pre ends otherwise). A 64-step run of it and one of the pin at W = 192 give the same `config.json`, the same `[step, train_loss_mean, dev_loss]` rows and the same step lines, timing cut. Pre ends non-zero otherwise.
- **I-4** each run's `config.json` is its arm's: the TRAIN hashes, context 192, width 256, steps, seed and recipe fields; `carry_windows == 4` for `k4` and no such field for `off`. `k4`'s log prints the line `[carry] K 4 windows a step, 4 lanes: TRAIN cut back to back into N windows over 16000 conversations, 16000 of them past one window…`, and `off`'s log prints no `[carry]` line.
- **I-5** the unit's GPU tests pass on `wick`'s card at the pin, and the two carry tests are named `ok` in the log:
  - `a_carried_window_is_the_whole_windows_forward`: a carried second window equals the whole two-window forward, bits equal, at retain on and off; a masked lane is a fresh start; a zero state differs; a nested frame and a wrong batch are refused;
  - `the_gradient_crosses_one_window_and_stops_where_detached`: a token in window 1 gets a gradient from window 2's loss when carried, and none when detached.
- **I-5n** with the scan's held state replaced by zeros at the pin (one line, by `sed`), the window test must compile and fail on its assert ": the carried second window" (the first window starts from zero either way, so the earlier asserts still pass), and the checkout is restored clean. Pre ends non-zero otherwise.
- **I-6** a job starts only when `wick`'s card shows under 2,048 MiB in use and under 10 % load. It waits, 5 minutes a look, for up to 6 hours, and never stops another job. `wick` is shared with a transcode and another lab.
- **I-7** each scoring pass:
  - exits 0;
  - holds both recall files with one outcome per query (609 or 617), every query decoded, and every reference stating its answer, so the grade can be met everywhere;
  - prints a `[past-window] … carry` line.
- **I-7s** before launch, smoke 4 runs the chain's own `score_one` and `score_ic` (copied from `chain-380.sh`) on smoke 3's `k4` checkpoint on p-long and q-tt, so I-7 meets both panels' real files before any long run. The chain is started only if both read `ok`. Smoke 4 read `ok` on both (p-long 609 of 609 and q-tt 617 of 617 decoded, every reference stating its answer, the carry line printed; q-tt took 1,235 s).
- Every stage writes `STAGE.exit`. The chain stops at the first stage that does not exit 0 and seals the run folder with every file's hash (`SHA256SUMS`). The run folder is new for each try and is refused if it exists (L-002).
- A failed I-check is an instrument failure. It is reported as that, never as "Does not carry" or "Carries" (L-007).

## Risks the build leaves open

- **Fewer conversations a step under carry.** `k4`'s 16 windows come from 4 conversations, and `off`'s from up to 16. Its gradient is more correlated, which may favour `off`. This is the price of a lane of K windows, and it is named in the REPORT.
- **The gradient reaches 768 tokens.** Values told further back can only be learned through what the forward state happens to keep, never through a gradient. The bins in the REPORT show where `k4` gains, if anywhere.
- **The reads are not carried.** The Hamming reads and the turn memory see one window in training. Under the served carry they read further, in both arms alike, so the bar reads them out.
- **Two seeds**, with same-seed pairs. The middle is not called.
- **A floor is the likely outcome** (see the prediction). It is reported as a floor, not as a negative about the carry.

## Minimum test power

The paired sign test needs b + c ≥ 10 discordant queries at a seed before it can call anything. At b + c = 10 it needs b ≥ 9 (b − c = 8 > 2√10 ≈ 6.3); at b + c = 30 it needs b ≥ 21. Under no effect, both seeds passing has a chance under 0.1 % (one tail of 2.5 % at each seed, 0.025²). With 560 far, census-clean queries, a true gain of 5 % of queries (28) with few losses clears the line at both seeds. A gain of 1 % does not, and lands at the floor. That is accepted, since a floor or a negative here closes a configuration, never the carry.

## Steps

1. **Pre** on `wick`: the pin's clean checkout and build; I-1; I-5; I-5n; I-3 (the base and the pin, 64 steps each) and a 64-step `k4` smoke; I-2 and I-4 on the smokes.
2. **Seed 1:** `off` and `k4` side by side (I-6 first); I-2 and I-4 on both; each run must exit 0, end with a finite loss and save its last checkpoint.
3. **Seed 1 scoring:** both models on p-long and q-tt under carry (I-7), and each model's `fbs-snapshot`.
4. **Seed 2:** steps 2 and 3 again.
5. **Seal** the run folder. Fetch it to the laptop, check the seal there, store it under `results/claude/one-model-end-to-end-carry-2026-11-14/`, then run the reader, the exports and the V0 panel serves on the laptop, one at a time.

## Cost

$0 in rent: `wick` is the owner's own RTX 4090. Smoke 3 timed each arm alone at W = 192: `off` 9 s and `k4` 14 s for 64 steps (0.14 and 0.22 s a step), with a peak near 6.5 GB of card memory for both side by side (smoke 1). Side by side, a seed's two runs take an estimated 1.5 to 2 hours, and its four scoring passes about an hour (smoke 3 scored p-long in about 765 s, both memory settings, with the decode cache on). The whole chain comes to an estimated 6 hours of card time. No Vast pod is rented, so the pod caps are not touched.

## Compute and stop rule

- **Where:** `wick` only, through `local/claude/wick/wick.sh job c380-chain-try-1 < local/claude/claude-380/chain-380.sh`. The laptop's completion watcher (`wick.sh wait` then `tail`) is armed at launch (L-004). `wick` bills nothing while idle; its card is released when the chain ends.
- **Stop:**
  - A run whose loss is not finite stops itself, and the stage fails. For `off` that is an instrument failure (the recipe has trained to its end before). For `k4` it is classified first (§4.2): the carried state adds at most K − 1 windows of backward, so a non-finite loss is first read as a defect in the build, which is repaired, and the run starts again as a new try. Only if no defect is found is it a result about this configuration, filed under "Not called".
  - A healthy run is never cancelled for time.
  - A training log that has not grown for 60 minutes is hung (an eval prints every 2,048 steps). The chain stops that run by its own pid; it is an instrument failure, and the chain starts again as a new try.
  - A scoring pass is capped at 4 hours. One that reaches the cap is run again with a longer cap, never read part-way.
  - A crashed or failed stage is never scored part-way. A retry is a new start with the next `TRY` (`try-2`), set before the start and never after, so it is a new folder, never a write into a sealed one (L-002). Every try is fetched; the reader is given the one it reads.
- **Disposal:** after the fetch and the store check, this unit's folders on `wick` (`/root/work/c380`, `/root/build/target-c380`, `/root/build/target-c380-pin`, `/root/build/src-f0d74adb9739e0b2a59cd5ddea7526e311d75776`, `/workspace/out/c380`, `/workspace/bin/c380-*`) are deleted by exact path. The inputs under `/workspace/in/c380` go last.

## FIT

- **First read** (fresh, non-author, 14 Nov), of the sheet, the order, the chain, the reader, the input sums and the code at the pin: NOT_FIT on one point.
  - The point: the reader refused any run folder with more than one try, while the stop rules plan a `try-2`, so a good retry could never be read. The reader now takes the try by name (the second read narrowed which sibling tries block it).
  - It traced as sound: the bar and the reader's integer form; every field the reader and the I-checks read, against the pin; the I-5n `sed` matching one line, and the test failing for the right reason; the outcomes as exhaustive.
  - Its two launch conditions are taken:
    - I-7 now meets both panels' real files before launch (I-7s, smoke 4).
    - The base binary's SHA-256 is pinned in I-3.
  - Its notes are taken:
    - the "memory out" sentence in Bars is corrected;
    - the reader checks `pre.exit` and the 560;
    - the binaries' copy fails pre if it fails;
    - I-5n greps the assert's message.
  - Its note that I-5's `decode_cache` filter runs every claude-278 GPU test is kept as it is: it fails safe.

- **Second read** (another fresh non-author reader, 14 Nov): NOT_FIT on one point.
  - The point: a try that crashed without reaching a stage's end (a killed job, a reboot) leaves no non-zero exit file, so the reader would have refused its retry. The reader now refuses a try beside the one read only when that try is complete and good (`score-s2.exit` reads 0). It is tested on 13 synthetic cases: four verdicts, three readable retries (beside a failed try, a crashed try and an empty one), and six refusals.
  - It confirmed the first read's points closed in the files:
    - smoke 4's `score_one` and `score_ic` are byte-identical to the chain's;
    - the I-5n message is in the pin's assert;
    - "memory out" matches what the bar reads.
  - Its notes:
    - I-3's `cmp` of `config.json` across two `--out` folders is settled by smoke 1, where both configs were identical.
    - The snapshot's stage exit is checked when the REPORT reads the snapshots.

- **Third read** (another fresh non-author reader, 14 Nov), of the reader's change and this FIT section: **FIT**, no blocking point.
  - It tested the reader on its own synthetic folders:
    - retries beside a failed, a crashed, an empty and a symlinked try are read;
    - two complete good tries are refused from either side;
    - a missing or malformed recall file, unpaired queries, 561 far queries, a missing `.ic` and a missing pre never reach a verdict.
  - It confirmed each claim of this section against the files.
  - Its two notes are taken:
    - the reader now also requires the read try's own `score-s2.exit` to read 0, so a try that failed at its last snapshot is not read;
    - the synthetic cases are a script in the kit (`test-read-380.sh`, 15 cases, all ok).
  - The reader's SHA-256 above is the file after that one line.

- **Try 1, an instrument failure in pre, repaired by this lab** (14 Nov, 23:52 UTC), recorded here before try 2 starts.
  - What passed: I-1; I-5 (33 tests, both carry tests `ok`); I-5n (the set-aside failed on "retain false: the carried second window"); I-3; both smokes trained.
  - What failed: pre ended 8 at I-4, "config.json is not arm off's". The check had two defects of its own:
    - The trainer writes no `context` field at 192, the sheet context (`skip_serializing_if = "is_sheet_context"`). So `.context == 192` could never hold. It now reads `(has("context") | not)`, which also refuses any other window.
    - The inner quotes of `has("context")` needed escaping inside the shell string.
  - The repaired check was run, copied from the chain, on try 1's real smoke configs:
    - both arms read `ok`;
    - it refuses swapped arms, context 256, K = 2 and the wrong seed;
    - `trained_ok` passes both smokes and refuses a diverged run and a wrong step count.
  - No bar, arm, step count or reading changed. Try 1 stays sealed as it is; try 2 is a new start (L-002), and the reader reads it beside try 1, which has no `score-s2.exit`.

TEST FITNESS: FIT
