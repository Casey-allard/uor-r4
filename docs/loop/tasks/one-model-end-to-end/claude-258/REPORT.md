# REPORT: one-model-end-to-end, claude-258 (train as served)

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Order: [ORDER.md](ORDER.md). Cancel probe: [PROBE.md](PROBE.md). References #3407.

Unit 1 of the 3 Casey's direction bounds (2 Nov 2026): the build. Plan step: PLAN.md's 29 Oct direction, step 4, one model trained, exported, served on the laptop and scored with tokens/s. What it changes in the model: with `--train-as-served 1`, training reads the values the integer artifact serves, so the trained and the served model hold the same parameter values.

## Tokens per second

No decode claim: nothing served changed. Training cost, measured on one RTX 5090 over 64 steps of (r-drop)'s recipe at seed 1: 7.97 s with the flag against 6.85 s without, about 17 ms a step more (+16%). Over a 16,384-step run that is about 5 more minutes of training, on a reading that includes the first steps' warm-up.

## What did not work, first

- **The ORDER's "few-step host run" could not run on the host.** D24/D29's CPU backend refuses arithmetic: three of the first draft's tests failed with "CPU model compute is unavailable". The few training steps moved to a CUDA test, ignored on hosts and run on the pod. The host tests keep what needs no arithmetic: a saved step's zero export gap, a refused rounding giving every value back, the switch-off path, and the CLI refusals.
- **Done-when 3 was shown, not measured against the base.** Without the flag, `round_if_served` returns `None` before touching a value. The host test `training_without_the_switch_rounds_nothing` pins that (no value moves, and `config.json` leaves the field out). No run's bytes were compared with a run from a base binary.
- **The first pod job stopped at its own check.** It grepped `config.json` for the compact `"train_as_served":true`, but the run writes the file pretty-printed (`"train_as_served": true`), so it ended with EXIT 5 after both runs had finished cleanly. A fault in the job, not the build: `job2.sh` went on from that check with the same runs and binaries (their sums checked), and exited 0.
- **No `cargo test` pins `run_long`'s own wiring** (the fresh review's finding 1). The CUDA test replays the order in test code: round, forward and backward, give back, optimizers, then a save through a test helper that rounds. So deleting the checkpoint's rounding from `run_long`, or moving the give-back after the optimizers, would leave every test green. What pins the wiring today is the pod's real runs:
  - **The checkpoint's rounding.** Without it, `on`'s step 0 trunk would be the float initial values, which the export changes (as it changes `off`'s); it changed nothing.
  - **The step's rounding.** `on` and `off` start from the same values, and every checkpoint gives its float values back, so without the step's rounding the two runs would train alike. Their step-64 train losses differ (1.0041624 against 1.0042354). That is on the condition that training is deterministic at one seed, as claude-216's probe showed byte for byte. The extra ~17 ms a step points the same way.

  This is evidence from one run, not a regression test.
- **Training's forward is still not the served forward.** The step runs with `with_gate_mode(.., export_mode = false)`, so the D11 export mode is not in training: the Q12 gate, the icosian snap, and the Q12 hidden before the writer. This unit closes the parameter half of the trained-against-served gap only.
- **Left behind on the laptop:**
  - three temp folders from the negative-gate runs, `/var/folders/29/pd94_f9169d2zr7bg7xvqg8m0000gn/T/claude-258-none-69152`, `claude-258-none-18885` and `claude-258-saved-68801` (the tests write under the system temp folder and clean up only on success);
  - a stray `crates/uor-r4-float-train/local/claude/target/.rustc_info.json` (2 KB, untracked), written by a test that ran cargo with a relative `CARGO_TARGET_DIR`.

  The loop guard refused to delete any of them (outside the worktree, or out of scope). None is committed. A FRICTION line records the second.

## What worked

- **Zero export gap, on the GPU.** `fbs-snapshot --round-as-exported` left all 6 files of the `on` run's steps 0 and 64 unchanged, sha256 for sha256. On the `off` run it changed the trunk at step 0, and at step 64 the trunk, the Hamming read and both turn-memory files.
- **The CUDA test** (`three_steps_trained_as_served_save_a_step_the_export_leaves_unchanged`) passed. Its checks:
  - before each forward, the trunk is on the export's grid when the flag is on and off it when it is off;
  - after three real steps the float master is off the grid;
  - the saved step's export gap is zero in all three parts with the flag, and nonzero in all three without it.
- **The negative gates failed as they should.**
  - On the pod: with `round_if_served` returning `None`, the CUDA test fails at its first check (`in_space_long.rs:14179`). With the source restored and checked clean, it passes again.
  - On the host, each fix set aside failed its test, and passed again once restored:
    - the rounding (`[true, true, true]` where `[false; 3]` is wanted);
    - the give-back after a refused rounding;
    - the run's refusal of reads the export does not carry.
- **A refused rounding loses nothing.** If a Hamming margin would round to zero or below, the export refuses it. `ServedValues::round` then puts every value it had rounded back as it was before returning the error, so the run stops with the export's message and its float values untouched.
- **The recorded run says how it trained.** `config.json` holds `"train_as_served": true` only when the flag is on.

## What changed

- `crates/uor-r4-float-train/src/in_space_long.rs`:
  - `LongConfig::train_as_served`;
  - `ServedValues` (rounds through `fbs_export::round_tables_as_exported`, `round_hamming_as_exported` and `round_memory_as_exported`, keeps the float values, gives them back);
  - `round_if_served`, called around each step's forward and backward and around each checkpoint's scoring and save;
  - a refusal when the flag meets a read the export does not carry (the Flock, content and opening reads, the prompt register, the type spin), or folded or reflected rate scores, whose runs the export refuses outright (`run_rate_reading`; added after the fresh review's finding 2);
  - four host tests and one CUDA test.
- `crates/uor-r4-float-train/src/bin/train-in-space.rs`: the `--train-as-served 0|1` switch, for training with `--out` only.
- `docs/research/notebook/float-training.md`: one row.
- This folder: ORDER, PROBE, REPORT.

## Measurements

- **Run:** 1 × RTX 5090 on Vast (`vast-58828367`, $0.975/h), up about 13 minutes, about $0.21, deleted after the upload. Source `90b1a89767dfa2224281518d7e7ac597bbbfa72a`, built `--locked` on the pod.
- **Inputs:** TRAIN, DEV and the tokenizer from `w2-inputs/2026-10-25/driver-data/`, their sums in `data.sha256`.
- **Recipe:** `--objective cell --hamming-read 4 --retain-rates 1 --decay-mult 1 --batch 16 --steps 64 --eval-every 64 --lr 0.001 --lr-end 0 --turn-memory 8 --turn-memory-reply 1 --turn-memory-dropout 500 --seed 1`, plus `--train-as-served 1` for `on`.
- **Outputs:** `results/claude/one-model-end-to-end-train-as-served-2026-11-02/` in the HF store. `results.tar` (429 MB, SHA256 `6242004fccfd9581e08703a13e9d576f93e409240d1de184a6aca974fd9be238`) holds:
  - both runs and the four rounded copies;
  - the tests' output;
  - the copies' verdict files;
  - the jobs and the n1 patch;
  - a `SHA256SUMS` of 94 files.

  `pod-logs.tar` holds the wrapper, bootstrap and reaper logs. Both are in the folder's `SHA256SUMS`.
- **The two runs' scores at step 64 are not results:** 64 steps, one seed, nothing registered.

## Checks run at the head

- **Host:**
  - `~/.cargo/bin/cargo test -p uor-r4-float-train`: 205 lib tests (the CUDA test is not compiled without `--features cuda`), every binary's tests and both integration suites, 0 failed;
  - the stub-nvcc `--features cuda` check: exit 0;
  - `cargo fmt --check`: no diff in this unit's lines.
- **Pod** (`--features cuda`, release):
  - the library suite, 281 passed, 0 failed, 7 ignored;
  - the CUDA test, passed;
  - n1 failed as it should;
  - the restored source passed.
- **After the pod run:** the refusal of folded and reflected rate scores. It is host-tested, and its negative gate failed as it should: with the two conditions set aside, the refusal test fails at `in_space_long.rs:14283`. It changes no training path, so the GPU evidence at `90b1a8976` stands for it.
- **The fresh-context adversarial review** at `94cb80c8b`: APPROVE WITH NITS. It found no correctness defect in the round, forward and backward, give-back and optimizer order. Finding 2 is fixed here; findings 1 and 3 are under "Still open"; the nits (done-when 3 shown by inspection, test temp folders) are named above.
- `uor-harness loop report --check`, `loop gates --full` and `loop pr-check --local` at the head: results in the PR.

## Skills used

None.

## Still open

- **Unit 2:** the prep sheet, a non-author FIT and the pre-run review for (r-drop) trained as served at 3 seeds, 16,384 steps. Recipe choice for that sheet: the 29 Oct training sheet's (r-drop), or the 30 Oct cut-train sheet's, which adds `--reply-read-train all`.
- **Unit 3:** the GPU run, the export, laptop serving, M1's bars and tok/s. Then back to Casey.
- **The forward's half of the gap.** Training does not yet run the D11 export mode's forward (the Q12 gate, the icosian snap, the Q12 hidden before the writer). Whether the served model needs it is unit 3's reading: it scores what the laptop serves against what trained.
- **Any value the export refuses stops the run mid-step** (the fresh review's finding 3), with its float values untouched and up to `--eval-every` steps unsaved. The cases:
  - a Hamming margin that rounds to zero or below;
  - an embedding value past Q12's i16 range;
  - a grid code out of range for an RMS gain or a convolution tap;
  - an i32 overflow in the Hamming read or the turn memory.

  None did in 64 steps. Unit 2's sheet names them for 16,384 steps.
- **A test that runs `run_long` itself on the GPU** (finding 1): a short `--train-as-served` run whose rounded copy is checked, on a tiny data folder or the pinned one. The cheapest form is unit 2's pre-run smoke at the sheet's head, the same two-arm check as this unit's `job.sh`.

## Plain summary

Training can now run on exactly the numbers the laptop serves. Before each step, the model's tables, its Hamming read and its turn memory are rounded the way the export rounds them, and the step learns at those values. The float copy the optimizer updates is kept and given back after each step. On the GPU, a short training run with this switch saved checkpoints that the export's rounding leaves byte for byte unchanged; without the switch, the same rounding changes them. It costs about 16% more training time. Next comes the real three-seed run's sheet and review.
