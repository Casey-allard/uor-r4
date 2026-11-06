# REPORT: one-model-end-to-end, claude-300

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Order: [ORDER.md](./ORDER.md).

## What did not work, first

- **The first pod launch ran no test.** The test script exited in under a second with `env: 'cargo': No such file or directory`: a non-login shell on the pod does not have `~/.cargo/bin` on its PATH. This was an instrument fault in my script, not a result. claude-298's job script sources `~/.cargo/env` and puts CUDA 12.8 on the PATH; the fixed script does the same. The empty first try was set aside on the pod as `t300-try1-nocargo`, before the reruns, and went with the pod.
- **The CUDA equivalence test does not run the trainer's row-setting code.** Review round 1 found this. The test's `rows` callback only records each forward's range, and its model has no turn memory, Flock, Hamming or register read. So an indexing error in the memory or reply rows, or a read that mixes windows within one forward, would not show there.
  - **The indexing is now covered on the CPU.** `forward_openings` and `forward_replies` build each forward's rows, and a CPU test checks them (see Checks).
  - **Window independence of the full recipe's reads is still untested.** The next sheet's smoke must compare one step's loss with `--micro-batch` equal to the batch against a split batch, on the real recipe, before its full runs.
- **The 1/k negative gate fails narrowly on the loss.** With each micro-batch weighted 1/4, the loss reads 1.004093 against the whole batch's 1.004219 (Nearest, not served): a relative gap of 1.3 × 10⁻⁴, against the test's 10⁻⁵ tolerance. The gap is small because the per-row losses in this fixture are close to each other. The gradient comparisons, which run after the loss assertion, were not reached in the negative run.

## What changed

- **`--micro-batch M`** for `train-in-space` (`LongConfig::micro_batch`; 0 is off and left out of `config.json`). `run_long` refuses an M that does not divide `--batch`.
- **`step_gradient`** runs a training step's windows forward and backward M at a time.
  - Each forward's loss is scaled by its share nᵢ/N of the step's scored rows, so the summed gradient is the whole batch's. One forward of the whole batch is left unscaled, so a run without the flag takes the same steps as before.
  - `add_grads` sums the forwards' gradients. Candle keeps only the variables' gradients after a backward, so no forward's activations should be held past its backward. No run here measured peak memory; the next sheet's smoke gates on it.
  - The reported loss is the forwards' weighted sum, which equals the whole batch's mean.
- **Each forward's turn-memory rows, reply rows and cut mask** are built before the step's clock starts, as they were before this unit. Each row is indexed at its window's place in its own forward. Memory dropout draws keep the window's place in the whole batch.
- `docs/loop/tasks/one-model-end-to-end/claude-300/`: the ORDER and this REPORT.

## Measurements

| run | pod | head | CUDA lib tests | equivalence test | negative gate (1/k) |
|---|---|---|---|---|---|
| 1 | vast-59434088, RTX 5090 (32,607 MiB), $1.00/h | `d1279cd68dc564607cc3e17a4cb756b55dcd2999` | 384 passed, 0 failed, 12 ignored | ok | FAILED at the loss assertion, as required |
| 2 | the same pod | `0029e153895589faa7d84fa8eb967f053a8e599d` | 385 passed, 0 failed, 12 ignored (adds the CPU row test) | ok | FAILED at the same loss assertion, 1.004093 against 1.004219 |

- **Spend:** about $0.25 on one pod, up 15:48Z to 16:03Z on 6 Nov. Its watcher released the lease and took it down at 16:03Z (L-004).
- **No training run.** This unit trains nothing, so no sheet and no notebook row. The batch-64 rerun is the next unit.

## Checks run at the head

- **The laptop checks below** ran in the session. Their output was not saved except for the CPU suite, which was rerun at `0029e153` into the main checkout's `local/claude/claude-300/cpu-tests-0029e153.out` (the crates are unchanged since).
- **CPU:** `cargo test -p uor-r4-float-train --lib --bins` passes: 283 library tests, plus every binary's tests.
  - `a_micro_batch_is_refused_unless_it_divides_the_batch_and_is_written_only_when_set`.
  - `each_micro_batch_names_the_whole_batchs_rows_at_its_own_place`: a batch of four shuffled windows, with one dropped, split into micro-batches of 1, 2 and 4. Shifted back by their forward's start, each micro-batch's opening and reply rows are the whole batch's.
  - **Its negative gate:** indexing rows by `b` in place of `b − lo` fails it.
- **CUDA, on the pod:** `micro_batches_weighted_by_their_scored_rows_give_the_whole_batchs_step` passes. Eight windows scoring 1 to 8 rows each give the same loss and every variable's gradient as four micro-batches of two, for Nearest (plain and with the served Q12 bridge) and Cell. The tolerances are 10⁻⁵ on the loss and 10⁻⁴ of each gradient's largest entry.
- **Clippy** on the CUDA build, with the stub `nvcc`, shows nothing on the changed lines. `cargo fmt` adds no drift: rustfmt shows 419 diff hunks in `in_space_long.rs`, against 420 on `main`.
- **Fresh review, round 1: APPROVE**, with one medium and two low findings:
  - **Finding 1 (medium):** the row code had no test. It is now covered on the CPU (above).
  - **Finding 2 (low):** the step clock had come to include the row building. It is built before the clock again.
  - **Finding 3 (low):** with M > 1, the per-forward stage marks and the step-0 diagnostics repeat once per forward. The profiler has not been checked against repeated stage names. Don't profile with `--micro-batch` until it is.

## Skills used

None.

## Still open

- **The batch-64 rerun:** (b64S) and (b64L) at `--batch 64 --micro-batch 16`, on a new sheet with a FIT and a D67 review. Its gates:
  - the smoke's peak memory below 90% of the card;
  - memory flat across the smoke's steps;
  - one step's loss on the real recipe the same with and without micro-batches, to the test's tolerance.
- **Profiling with `--micro-batch`** (review finding 3).

## Plain summary

The trainer can now build one large batch out of several small ones. It runs each small batch through the model, adds up their corrections, and makes one update. Only one small batch's working memory should be held at a time, so the batch-64 test that ran out of memory last time should fit on the same card. That is not measured yet; the next test checks it before it trains. A test on the graphics card shows the small batches give the same update as the large one. It fails, as it should, when the small batches are counted the wrong way.

References #3407.
