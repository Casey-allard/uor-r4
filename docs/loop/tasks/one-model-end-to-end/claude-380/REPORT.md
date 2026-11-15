# REPORT: one-model-end-to-end, claude-380 (V1: the state carried across training windows)

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Order: [ORDER.md](ORDER.md). Sheet: [one-model-end-to-end-carry-2026-11-14.md](../../../../harness/prep/one-model-end-to-end-carry-2026-11-14.md). PR #3979. Follows V0, claude-376 (PR #3970).

The order asked whether training with the recurrence state carried from one window into the next (`--carry-windows 4`, the gradient through four windows) teaches the model to recall a value told before the current window. Two seeds of the carry (`k4`) and of the same recipe without it (`off`) were trained on `wick` and scored on p-long with the turn memory out.

**The answer, at the scope of this sheet: Floor, as predicted.** Neither arm states far values often enough for the bar to see the carry. On the 560 far (distance 256 or more), census-clean p-long queries, b + c is 0 at seed 1 and 7 at seed 2, under the bar's 10. The carry is built, tested and off by default; the trained runs say nothing either way about whether it carries recall. The notebook word is **inconclusive**.

## What did not work, first

1. **The bar sat at the floor, at both seeds.** Reader output, verbatim (`read-380.sh`, SHA-256 `aca56cb8…7bd8e`, run on `run/try-2`):

   ```
   seed  panel   memory     b   c   n  off  k4
   1     p-long  no-memory   0   0  560    0    0
   1     p-long  memory      0   0  560    0    0
   1     q-tt    no-memory   1   2  569    2    1
   1     q-tt    memory      2   5  569    9    6
   2     p-long  no-memory   5   2  560    2    5
   2     p-long  memory      5   2  560    2    5
   2     q-tt    no-memory   0   5  569    5    0
   2     q-tt    memory      1  11  569   11    1
   VERDICT: Floor (p-long, memory out, distance >= 256, census six out; seed 1 b 0 c 0, seed 2 b 5 c 2)
   ```

2. **The floor is the model's recall in general, not a far-distance effect.** Stated by distance bin, every query (p-long 609, q-tt 617), turn memory out, `off` then `k4`:

   | panel | seed | 0–255 | 256–767 | 768+ |
   |---|---|---|---|---|
   | p-long | 1 | 1/43, 0/43 | 0/138, 0/138 | 0/428, 0/428 |
   | p-long | 2 | 0/43, 0/43 | 1/138, 3/138 | 1/428, 2/428 |
   | q-tt | 1 | 2/45, 0/45 | 1/135, 1/135 | 1/437, 0/437 |
   | q-tt | 2 | 0/45, 0/45 | 1/135, 0/135 | 4/437, 0/437 |

   Inside the first window (under 192 tokens), where the carry changes nothing, both arms state 0 to 2 of 43–45. So the readout cannot tell the arms apart at any distance. Classified: the instrument is sound (smoke 4 and every I-check passed, the reader refused nothing); the reading is at the model's floor, which is the sheet's Floor outcome, not a mechanism negative.

3. **With the turn memory on, q-tt leans against the carry. No bar reads this.** Far q-tt queries stated, memory on: `off` 9 and 11, `k4` 6 and 1 (b − c: −3 and −10). On p-long the memory changes nothing for either arm (the same counts with it on and off). The memory copies a value from anywhere in the conversation (L-010). One reading is that the carry's training leaves the model leaning less on that copy, but this unit did not test it. It is a lead for the next unit, not a result.

4. **V0's exact-reply panel is lower for `k4`, at both seeds.** P14 + P23 with the turn memory dropped, exact of 2,218: `off` 106 and 114, `k4` 63 and 91. The `k4` models were trained on four-window streams of 1,700–2,048-token conversations; the panels are 256-token windows from claude-374's everyday TRAIN, which neither arm trained on. So this measures transfer to another corpus, not the carry. V0's own models (claude-358's `p`, trained on that corpus) got 171, 173 and 211; they are quoted beside, never as a control.

5. **The carry costs more than the sheet guessed.** Seconds per step, both arms side by side on one card: seed 1 `off` 0.308, `k4` 0.472 (1.54×); seed 2 `off` 0.355, `k4` 0.718 (2.02×). The sheet's guess was under 1.7×; seed 2 missed it. The card was shared during seed 2: seed 2's `off` p-long score took 4,021 s against seed 1's 396 s, with the same binary and inputs. So the seed 2 ratio carries an unknown load share. The chain took 8.5 h against the sheet's 6 h; no run was cut for time.

6. **Try 1 stopped in pre on an instrument check of my own.** I-4 read `.context == 192` from the run config, but the config leaves `context` out at 192 (`skip_serializing_if = "is_sheet_context"`). Pre exited 8 with no training. The check now reads that `context` is absent. It was tested on try 1's real configs (both arms read ok; swapped arms, context 256, K = 2 and seed 2 are refused). The repair is recorded in the sheet (commit `878d6352b`), and try 2 ran from it. Try 1's 72 files are kept beside try 2 in the store.

7. **The seal missed one file by two lines.** `chain.log` fails its `SHA256SUMS` line, because the chain wrote "sealed 518 files" and "chain done" after hashing. With those two lines dropped, the log hashes to the sealed `718f5caf…`. The other 517 files match. No result file is affected.

8. **Peak card memory was not recorded.** The chain logs the card only when it waits for it to be free. The one reading is a spot check at seed 1 step 2,048: 9,142 MiB with both arms training.

9. **Not run:** a third seed, the near-bin reading the "Does not carry" outcome names, and an integer-served carried session. Under Floor the sheet names none of them.

## What changed

- **`crates/uor-r4-float-train/src/in_space_long.rs`:** `--carry-windows K` (default 0, off).
  - On, each step's batch of B windows is B/K streams of K back-to-back windows of one conversation (`CarrySchedule`, `carry_stream_windows`).
  - Each window's scan starts from the state its stream's previous window ended in (zero where a window opens its conversation). One backward runs through the K windows; the carry is then detached.
  - `check_carry_flags` refuses the reads, controls and cuts the carry does not reach, and micro-batches.
- **`geometric_stack_native.rs`:** the recurrence's carry hook. A thread-local `WINDOW_CARRY` holds the state, passed through one zero-transition scan position, with the convolution's padding taken from the previous window's last three inputs.
- **`bin/train-in-space.rs`:** the flag's parsing only.
- **Merging `origin/main` (claude-381's vector reads, claude-385, codex-8/11, codex-1):**
  - Two conflicts, both new fields and flags side by side; both kept.
  - The carry now also refuses `--starling-read`, `--prime-route-read` and `--state-flock-read`: the carried step's forward was never checked with them. The refusal test gains that case.
  - The trained runs used none of these reads; they ran at the pin `f0d74adb9`.
- The prep sheet, this REPORT and one notebook row. No scan kernel, export, integer session or other lab's file is changed.

## Measurements

**Runs.** `wick` RTX 4090, shared. Pin `f0d74adb9739e0b2a59cd5ddea7526e311d75776`.
- **Recipe:** claude-358's `p`, place kit dropped, W = 192, batch 16, 16,384 steps, `--retain-rates 1`, seeds 1 and 2.
- **TRAIN:** claude-275's long TRAIN, 16,000 conversations. `k4` cut 172,262 carry windows, the longest conversation 11 windows; `off` had 172,143 usable windows.
- **Scoring:** float, `--past-window carry --context 192`, p-long (609 queries) and q-tt (617).

| seed | arm | train_loss_mean | dev_loss | dev top-1 of 14,287 | s/step |
|---|---|---|---|---|---|
| 1 | off | 0.5594 | 2.5936 | 363 | 0.308 |
| 1 | k4 | 0.5328 | 2.4244 | 345 | 0.472 |
| 2 | off | 0.5621 | 2.4513 | 225 | 0.355 |
| 2 | k4 | 0.5598 | 2.7287 | 346 | 0.718 |

DEV is 428 short windows from a zero state, so its loss says nothing about the carry.

**Loss by window under the carry** (p-long, memory out, from `past-window-p-long-no-memory.json`; first window, then every later window inside 2,048):

| seed | arm | first window loss | later windows loss | later windows nearest hits of 84,159 |
|---|---|---|---|---|
| 1 | off | 0.5568 | 0.6276 | 65,623 |
| 1 | k4 | 0.5300 | 0.6000 | 65,542 |
| 2 | off | 0.5703 | 0.6454 | 65,293 |
| 2 | k4 | 0.5671 | 0.6332 | 64,532 |

`k4`'s later-window loss is lower by 0.028 and 0.012. Its first-window loss is lower too, by 0.027 and 0.003. At seed 1 almost all of the gap is there before the carry has anything to carry (0.027 of 0.028); at seed 2 only about a quarter is (0.003 of 0.012), leaving about 0.009 the first window does not explain. Its nearest hits past the first window are slightly fewer. No bar reads these numbers.

**V0's panel** (P14 + P23, 2,218 rows, `raw-id-replies --turn-memory 0`, greedy, 48 tokens, reply marked, one process at a time, load average 2.4 to 3.2; scored by `rows-380.sh` and `subst-380.sh`, claude-376's scripts with only names and paths changed):

| model | exact of 2,218 | ended | fact rows, answer word / exact, of 188 | prompt ids/s | served tokens/s |
|---|---|---|---|---|---|
| off-s1 | 106 | 2,137 | 17 / 9 | 4,538 | 1,497 |
| k4-s1 | 63 | 2,130 | 7 / 4 | 4,533 | 1,612 |
| off-s2 | 114 | 2,038 | 16 / 6 | 4,541 | 1,519 |
| k4-s2 | 91 | 1,822 | 28 / 16 | 4,547 | 1,439 |

V0's `p` models, for scale only: exact 171, 173 and 211; about 4,400 prompt ids/s and 1,450 served tokens/s. Each artifact here is 83,920,448 bytes against V0's 19,345,344: the long TRAIN's snapshot holds more rows. Prompt reading is within 0.3% across the four.

**Artifacts** (SHA-256, `fbs-export --legacy-artifact --raw-gates --raw-transition --run-config config.json --context 8256`):
- off-s1 `b1bee188b8a973282a72826c3578b34ace39f0460463facbf1549a96c9cd4eed`
- k4-s1 `732cf965add04e4cff4393db270040569699ccd4a07f68d2bcdf1c8678fb2713`
- off-s2 `225460f4feb8eecfbd54ba144f7df198f6efc5a05c31b0af9e5d96aada72f2ed`
- k4-s2 `b32c8f404fa71fe0a38f91866cbc200356c63b0ca1f736cf2af9e3660e17b0a4`

**Store.** Both tries are under `results/claude/one-model-end-to-end-carry-2026-11-14/run/` (592 files uploaded, 592 listed back).

## Checks run at the head

| Check | Result |
|---|---|
| I-1, the inputs' sums on `wick` | all OK |
| I-5, the GPU tests at the pin (`--lib`, the carry, decode-cache and gradient tests among them) | 33 passed, 0 failed |
| I-5n, the carried state set aside in `a_carried_window_is_the_whole_windows_forward` | fails, "retain false: the carried second window" |
| I-3, carry off at the pin against the base binary, 64 steps at W 192 | losses and step lines byte-identical; config identical |
| I-2/I-4/I-7, every stage | ok (12 of 12) |
| Every training and scoring pass's exit | 0 |
| The reader and its test script | SHA-256 `aca56cb8…7bd8e` and `fd87d608…a209d`, as the sheet fixed; 15 synthetic cases ok before the run |
| The seal | 517 of 518 match; `chain.log` matches with its two post-seal lines dropped |
| Merged head: `cargo test --release --locked -p uor-r4-float-train --lib -- carry` (laptop) | `--lib` 429 passed, `--bin train-in-space` 16 passed; the carry filter 16 passed; the vector-read refusal set aside, its test fails, restored, passes |
| Merged head `c736230fb` on `wick`: I-5's command (`--features cuda --lib --include-ignored carry decode_cache`) | 34 passed, 0 failed; the carried-window and gradient tests ok |

Binaries: `fbs-export` `6289f49885b31a3b2efdeb407711fe2ca8454f6be7d4f74e3fe2332a6e2d69e2`, `raw-id-replies` `c3335ad0238c6222601c5f48f35cec90295052e510c607905114fdf131e63aa4`.

## Skills used

`.agents/skills/native-space-cycle/SKILL.md` was read before the training work. The unit is a float training run: the carry is training-side only, and the native integer session is not changed. Kit (not committed): `local/claude/claude-380/` (`chain-380.sh`, `read-380.sh`, `test-read-380.sh`, `serve-380.sh`, `rows-380.sh`, `subst-380.sh`).

## Still open

- **Raise far recall in both arms first** (the sheet's Floor outcome), then test the carry again. Two ways the sheet names are more steps or a denser far-value TRAIN. The first-window bin (0 to 2 of 43) says even near values are not stated with the turn memory out, so the first target is the model's own recall at any distance.
- **The memory-on q-tt lean** (finding 3): whether carried training makes the model lean less on the turn memory's copy. It needs its own bar.
- **The carry's cost.** 1.5× to 2.0× a step side by side. It was not profiled.
- **A test of the trainer's carried step** (this unit's own exact-head review). The GPU tests call the recurrence's carry frame directly. The CPU tests cover the cut, the schedule and the flags. So nothing would fail if `carry_step_gradient` stopped detaching the state it hands to the next step: the graph and card memory would grow with every test green. The code as written detaches it (`in_space_long.rs`, the last line of `carry_step_gradient`), and both `k4` runs finished 16,384 steps beside an `off` run on one 24 GB card. Card memory was read only once (finding 8), so this is not a measurement of flat memory. A GPU test asserting that the returned carry tracks no op, and that a lane opening a conversation is zeroed, is owed before the carry is used again.
- **Not audited with the carry:** `--fbs-arm`, `--writer-anchor` and `--served-forward`, which no carried run used. Under bf16 the carry's zero state is F32 and is refused on the first step, loudly.
- **An integer-served carried session** (V2 under "Carries") stays unbuilt until a carry is shown.

## Plain summary

We taught the trainer to pass the model's running memory from one stretch of a conversation into the next, as the served model already does, and trained two models that way and two without it. Then we asked each model about values it was told earlier in long conversations. Almost no model answered almost any of them, near or far, so the test could not tell the two kinds apart. That is the result we predicted before the runs. The new training option works, is off unless asked for, and costs 1.5 to 2 times as much time a step. The next step is a model that recalls told values at all, before the carry is tested again.
