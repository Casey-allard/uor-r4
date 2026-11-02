# REPORT: integer-nearest-readout, claude-256 (the replay driver, arithmetic step)

Lab: claude. Item: [integer-nearest-readout](../../../../roadmap/items/integer-nearest-readout.md) (#3462). Order: [ORDER.md](ORDER.md). Rule: claude-247's [decision card](../claude-247/DECISION.md), as amended by claude-254's [decision card](../claude-254/DECISION.md). Q5 chain step 6, part 1. References #3462.

## Tokens per second

No decode speed is claimed, and no served path changes. The one serving-crate change is a read-only accessor, `IntegerStackModel::fbs_writer_table`. R's decode speed is still claude-243's: 1,204 tok/s median decode on R s1.

The driver runs offline. On the laptop (Apple M1), with the release `fbs-toks` built at `cdc016259` (sha256 `dea26754…`), after the review's fixes:

| Reading on DEV s1 (T's 621 MB trace, 308 windows) | Wall | Peak resident set (`time -l`) |
|---|---|---|
| `--decision-check` of T's trace | 4 s | 1.00 GB |
| `--decision-deciles` of T's trace | 3 s | 1.39 GB |
| `--decision-replay` of (T, R): 308 unforced R windows, then 2 replays, 14 passes in all | 18 s (the 14 passes: 723 ms, about 52 ms each) | 1.20 GB |

The first reading, at `3aa60ad71` before the fixes, took the same walls (the 14 passes in 733 ms, 1.15 GB for the replay).

## What did not work, first

- **The driver's first comparison would have refused every DEV window.** T's trace reads on past the prompts file's reference: its first window holds 99 ids, against R's 50. The first draft compared the reference's picks with R's unforced picks only when their counts were equal, and called anything else I-Align.
  - It was found by reading the inputs' first bytes, before any reading ran. Class: the instrument (this unit's driver), not a mechanism.
  - The repair is `differing_offsets`. It compares the reference's offsets inside R's window (the row's prompt and its reference), refuses a window whose tokens there are not R's or that stops short of R's last offset, and counts the reference's picks past R's window, not comparing them.
  - The ORDER records the amendment, made before any reading ran. A host test pins it, with negative gates G9 and G10.
- **The first reading's "beyond" count was not a count of picks.** It counted 19,050 reply steps past R's window, steps with no nearest among them. The count is now of steps with a nearest: 6,458. With the 4,382 compared, that is 10,840, which is T's nearest count in its own check (negative gate G11). That first run is kept in `read/run1/`. Its check and deciles files are byte for byte the second run's.
- **The first fresh review found five gaps in `--decision-replay`'s guards** (four slices, each APPROVE WITH NITS, no blocker). Class: the instrument. Each is fixed in `cdc016259`, with a test and a negative gate (G12 to G19):
  - one refused replay ended the whole run with nothing written, where the ORDER says each replay is recorded as it ends. A refused replay is now recorded with its refusal, the rest still run, and the command then fails;
  - nothing checked that the reference was T's trace, that its windows were at distinct positions, that the deciles were read from that trace, or that `--reference-table` was R's artifact (T's table is R's divided by 4096). Each is now refused before any replay;
  - nothing checked that pass 0, with no decision forced, picks R's unforced pick. It is now refused otherwise (I-Rerun);
  - an `--at` past its window was refused only when its replay ran. It is now refused up front, as are one past its reference and one past R's offsets;
  - the tests never ran T's and R's tables from a file, a T trace whose check could fail, or an `--at` named twice apart. They do now.
  - The readings were run again at the fixed code (`read-r2/`): the check and the deciles are byte for byte the first run's, and the two replays are the same (the sha256 of `.replayed` is equal), with 0 refused.
- **Three of those gates failed for the wrong reason** (round 2's fresh review, APPROVE WITH NITS). With its check set aside, each test was still refused by a neighbouring check, and failed only on the wording:
  - G14: the R trace was then refused by the deciles check;
  - G16: the table had no Hamming read;
  - G19: the reference ended at the window.
  - The test now gives each check an input only that check refuses: T's trace named side R with deciles that name it, a second valid artifact with the same reads, and a reference longer than the window. With the check set aside, G14 and G16 now see the input accepted, and G19 sees it refused only late, after a file is written (`gates-bin-r3.txt`). This is a test-only change, about 40 lines, made in the session without a third review round.
- **The ORDER's first done-when 1 asked for a refusal no replay can reach:** a replay that runs past its prefix's decision count. `grade_pass` refuses a pass that does not take Φ's values, so each pass ends at a new decision of the prefix. It was amended before any code was written, and the test of the replay's end replaced it.
- **Two of claude-254's tests asserted what this unit forbids.** They accepted float picks 1,024 and 1,280 behind the other side's in f64. Done-when 3 refuses a pick behind past f32's rounding bound, so both now expect the refusal, and new tests cover the cases inside the bound. The behaviour changed by the ORDER, not to reach green.
- **The 36 failing tests of `uor-r4-integer --features serving-wasm` are still red.** They are the base's own (claude-255's `head-failed-names.txt`, at its head merged as `175fca2a8`): the M1/M2/M3 challenger and timing tests and the frozen-panel CLI test. None is touched here.
- **The structure step (F, T) is not replayed.** It needs Q = T on the GPU: `train-in-space`'s single-window float mode, the next unit. Q5 is not read, and done-when 3 does not move.

## What worked

- **T's loader holds on real data.** `--decision-check` of T's DEV s1 trace (`b1970581…`, claude-252) against T's table, R's artifact divided by 4096, passes on all of it. That is 14,031,744 buckets, 584,656 codes and 10,840 nearests over 36,541 steps in 308 windows.
- **The replay of the arithmetic step accounts for both of its differences.** Of the 4,382 offsets inside R's window, T and R pick the same token at 4,380, as claude-252 read without forcing. The other two each replay to the nearest:
  - **Position 30248, offset 19 (step 80):** 11 passes. Nine rate buckets (layers 1, 2 and 5, steps 8 to 72) and one type-head query code (depth 2, step 80) were forced, each a near tie. The picks then agree (970).
  - **Position 41408, offset 3 (step 22):** 3 passes. Two rate buckets were forced (layers 4 and 3, steps 7 and 20), each a near tie. The picks then differ (T 289, R 3478), but both margins are within G₁₀ = 30,228 (T's 1.13, R's 11), so the step is a near tie.
  - Each forced bucket's value is between 6.3 × 10⁻⁶ and 2.2 × 10⁻⁴, against its family's G₁₀ of 0.0061 to 0.0093. The forced code's value is 2.0 × 10⁻⁴, against 0.354.
  - **Counts:** 2 accounted, 0 unaccounted, 2 ended at the nearest, 0 at a kept set, 0 at a memory lookup.
  - This is a reading outside Q5's set (claude-249's `prompts-dev-s1-outside.json`). It reads no Q5 position, and the bar is unchanged.
- **T's deciles** (14 families): rate buckets 0.0057 to 0.0093, copy-head codes 0.031 to 0.032, type-head keys 0.015 to 0.016, and type-head queries 0.172 and 0.354. These are close to claude-254's F deciles on the same windows.

## What changed

- **`replay_step`** (`fbs_snapshot.rs`) runs one step's whole forced replay. Each pass goes through `grade_pass`, and a code or bucket the pass ends at joins Φ for the next. The replay stops at a kept set, a memory lookup or the nearest, and is accounted for by `replay_accounts`. A pass error is returned as "pass i: …".
- **f32's rounding bound on a float pick** (`NearestTable::behind_bound`): γₙ(|s(u)| + |s(w)|) at u = 2⁻²⁴, plus the f64 term at 2⁻⁵³, where n is the table width. R's bound is 0.
  - In `grade_pass`, a float pick behind its rival by more than the bound is refused on either side.
  - In `check_nearest`, a float pick may trail its runner-up by at most the bound. The base took any order.
- **R's tie rule:** in `grade_pass`, R's pick that ties the reference's but has the larger id is refused.
- **`grade_pass`'s doc** now says the reference's entries and codes are checked even where forced.
- **`fbs-toks ARTIFACT --decision-replay`** runs the arithmetic step (T, R) with Q = R from the artifact. R's table is the model's own writer table (`fbs_writer_table`). Each pass steps the row's prompt and its reference cut at p, under Φ, through `forced_picks`. It writes `uor-r4.decision-replay/1` new.
- **The decision commands' refusals,** before any file is read: an unknown or doubled flag, an `--at` named twice, a side with no table (only F, R and T have one), an offset past the window, and a deciles file that names no trace.
- **`--decision-replay`'s refusals before any replay** (the review's fixes): a reference that is not side T, two windows at one position, deciles read from another trace than the reference (by sha256), a `--reference-table` whose bytes are not the artifact's, and an `--at` past its window, its reference or R's offsets. In each replay, pass 0 must pick R's unforced pick (I-Rerun). A refused replay is recorded as `{position, offset, refused}` and counted under `refused`; the command then fails after writing its file.
- **Size:** code +612 −70, tests +765 −10, in `fbs_snapshot.rs`, `bin/fbs-toks.rs` and `stack/session.rs`. The docs are this REPORT, the ORDER, one notebook row and one item bullet. The ORDER gave no size estimate.

## Measurements

- **Inputs** (`read/inputs.sha256`):
  - R's artifact `a7r-s1.uorlut` (`f86a7856…`, claude-243's seal);
  - the prompts `prompts-dev-s1-outside.json` (`5e0bbf75…`, claude-249);
  - T's trace (`b1970581…`, claude-252, in the HF store at `results/claude/integer-nearest-readout-trace-t-2026-11-02/`).
- **Outputs** (`local/claude/claude-256/read-r2/`, run by its `run-readings.zsh` on the code committed as `cdc016259`; `uncommitted-crates.diff` there is byte for byte that commit's change to the crates):
  - `check-t.json` `e4123f04…`;
  - `deciles-t.json` `d303c4b3…`, naming T's trace;
  - `replay-t-r.json` `ea154fa9…`, with G₁₀ 30,228 (claude-244's cell).
  - The first run, at `3aa60ad71`, is kept in `read/` (`replay-t-r.json` `0a12291d…`). Its replays are the same; the file differs only by the new `refused` count, the deciles' path and the time.
- **Negative gates,** each setting aside one fix's own lines, running its test, expecting a failure and restoring the file byte for byte:
  - **The library** (`gates/run-gates.zsh`, `gates-lib.txt`):
    - G1: the replay forces the decision each pass ends at.
    - G2: the grade refuses a float pick behind past the bound (2 tests).
    - G3: the trace check holds a float pick to its runner-up within the bound (2 tests).
    - G4: R's larger-id tie is refused.
    - G5: a negative reference margin inside the bound is accepted.
  - **The CLI** (`gates/run-gates-bin.zsh`; `gates-bin.txt`, then `gates-bin-r2.txt` and `gates-bin-r3.txt` with the review rounds, all fourteen):
    - G6: a deciles file with no trace is refused.
    - G7: a side with no table is refused.
    - G8: each pass hands Φ to R.
    - G9: a longer reference window is compared over R's offsets.
    - G10: other tokens inside R's window are refused.
    - G11: only picks are counted past R's window.
    - G12: an `--at` named twice apart is refused.
    - G13: a refused replay is recorded and the run goes on.
    - G14: a reference that is not T's trace is refused.
    - G15: deciles read from another trace are refused.
    - G16: a table that is not the artifact is refused.
    - G17: a pass 0 that does not pick R's unforced pick is refused (I-Rerun).
    - G18: two windows at one position are refused.
    - G19: an `--at` past its window is refused before any replay.
  - All nineteen fail with their fix set aside, and pass restored.

## Checks run at the head

The suite and the readings ran at `cdc016259`, the last commit that changes code outside the tests. `23d4ee94a` after it changes only `fbs-toks`'s `forced` test module, which builds only with `fbs-synthetic`, so neither the suite nor the release binary includes it. The bin tests and the fourteen CLI gates ran at `23d4ee94a`. The other commits change only docs.

- `cargo test -p uor-r4-integer --features serving-wasm --no-fail-fast` at `cdc016259`: 636 passed, 36 failed and 4 ignored (`suite-cdc016259.log`). The 36 names are the base's, with none new and none fixed. The 24 `fbs_snapshot` tests all pass. The runs at `3aa60ad71` and `b07851a2b`, before the review's fixes, gave the same counts (`suite-head.log`, `suite-b07851a2b.log`).
- `cargo test -p uor-r4-integer --features serving-wasm,fbs-synthetic --bin fbs-toks`: 15 passed. The `forced` module builds only with `fbs-synthetic`, so the suite above does not run it.
- `cargo clippy -p uor-r4-integer --features serving-wasm,fbs-synthetic --all-targets`: no warning in `fbs_snapshot.rs` or `bin/fbs-toks.rs`. The one in `stack/session.rs` (line 736, a manual `Option::map`) is the base's, not on this unit's lines.
- `cargo fmt -p uor-r4-integer`: applied, no other file changed.
- `uor-harness loop gates --full` and `loop pr-check --local`: listed in the PR's review record.

## Skills used

None.

## Still open

- **The structure step (F, T)** (Q5 chain step 6, part 2): `train-in-space`'s single-window float mode, so that Q = T runs one window per pass on the GPU, driving the same `replay_step`. Each float pass is now a whole `score_rows` run (204 s on F's s1).
- **Then the sheet** registers claude-247's rule with claude-254's amendment verbatim, with a fresh FIT and a pre-run review, and the GPU run reads Q5. Tracked in the item's done-when 3 (#3462).
- **The WASM build** in done-when 2 was not run here. This change touches no WASM-only code.
- **The library review's low-severity points,** left for the structure step's unit, where float sides of other widths arrive:
  - `behind_bound` does not guard n·u ≥ 1. This is out of reach below a table width of 2²⁴, and R's is 256.
  - For a width that is not a power of two, the sum's bound needs γₙ₊₁. R's 256 is a power of two.
  - A NaN margin passes `margin < 0.0`.
  - No test replays to a memory lookup.
- **The replay file is written once, when the run ends.** Each replay is in it, refused ones too, but a run killed partway loses those already done. At 14 passes in 723 ms this costs nothing today. The structure step's float passes are whole GPU runs (204 s each on F's s1), so that unit should append each replay to a line file as it ends (round 2's review).
- **With `--at`, a window's alignment is left to the grade:** `grade_pass` refuses a pass whose ids are not the reference's (I-Align), and the replay is recorded as refused. The up-front check that runs without `--at` (`differing_offsets`) is not run on named windows. A refused entry also carries no `step`. Both are low severity (round 2's review).
- **Seven scratch folders were left** in the macOS temp folder: `uor_fbs_toks_replay_<pid>`, 2.0 MB in all. One is from the new file test's first failing run and six from the gates' failing runs, before the test removed its folder on a failure too. The loop guard refuses any write outside the worktree, so they were not deleted here. The test now removes its folder however it ends; a later gate run left none.

## Plain summary

The integer model and the float model it was exported from sometimes pick different words. Q5's rule asks whether each such difference is only a near tie. To answer it, the step is replayed: the integer model runs again with its internal choices forced, one at a time, to the float model's, until the two either agree or one choice turns out not to be close.

This unit builds that replay loop and runs the integer half on the laptop. On 308 test windows, the integer model and its float twin with the same rounded numbers disagreed at 2 of 4,382 words. Both replays ended in near ties. One needed ten tiny forced choices before the words matched. In the other, after two forced choices, the two words still differ, but both are within the near-tie bound. These windows are outside Q5's set, so Q5 is not yet read. Along the way the replay found that the float records run longer than the integer ones, and now compares only the part both cover.
