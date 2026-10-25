# REPORT: m1-w2-code-utilization, claude-95

Lab: claude. Order: [ORDER.md](ORDER.md).
Closes #3179. References #3189 (the pending notebook row and the guard gap).

## What did not work, first

- **The notebook row is not written. It is tracked in #3189.** The loop guard refuses the edit to `docs/research/notebook/m1-discrete-attention.md`. `loopguard.rs` (around line 2820) allows only:
  - the ORDER folder;
  - the item file;
  - `docs/harness/mistakes/`;
  - FILEMAP;
  - the shared files.

  It does not allow the item's `notebook:` page, though `docs/loop/README.md` ("Scope, without a line cap") lists "new rows on its notebook page" as always allowed, and `loop pr-check`'s scope rule accepts it. I did not work around the guard. #3189 records the defect, its fix, and the row to append once the guard allows it, directly after the 2026-10-25 W2 row:

  > | 2026-10-25 | W2 code-utilization diagnostic on the stored W2 capture: host post-processing only, no GPU. Precedence 0 reproduced every W2 number exactly, three ways. Arm R was not run: the Q1–Q3 checkpoints are missing. | Code-count admission 9,064 against the writer's 1,920 and a fixed most-frequent-cells list's 5,935; code-table gain over the last data doubling 69 (projection 3,552); code × query table 7,262 against the bigram's 6,300 | The sheet's §5 paired floor, (c − b)² > max(576, 4(b + c)), per question; QB's line is W2's H_law line (about 6,435) | positive (QA A1a, QB B2, QC C1: admission is recoverable from the code (sets not load-matched); the registered data-scaling projection (3,552) is below W2's H_law line, so G1 is declined; the code adds information beyond the last token. Follows the W2 row above) | [m1-w2-code-utilization](../../labs/m1-w2-code-utilization/README.md) |

  The item is set to `status: done` anyway: the rest of its done-when is met, the full record is in [the lab README](../../../../labs/m1-w2-code-utilization/README.md) and the sealed record, and leaving the item `ready` would re-offer a finished run. The row's tracker is #3189, not #3179.
- **The exact-head review, split in two, asked for changes; all are made.**
  - **The claims reviewer returned CHANGES** at `6138c93ff`.
    - It asked for the notebook gap to be tracked (now #3189).
    - It asked for the QB headline to read as the registered projection, not as a forecast.
    - It found six percentages that `bc` had truncated instead of rounding: 82.1, 67.9, 8.8, 23.4, 25.7 and 51.3.
    - It asked for a note that DEV query determinacy is inflated by single-row codes.
  - **The code reviewer returned APPROVE with three should-fix items,** all now done.
    - The admission and joint logic is now in tested functions (`ranked_for`, `in_top`, `joint_prediction`). The reviewer's planted fault, `in_top` taking k − 1 cells, now fails `in_top_respects_the_row_k`.
    - Arm R's reason now says R was not attempted.
    - The README states that `run-b.tar` was authenticated outside the tool.
  - **Its nits are also in:** the A_T ≤ A_Ω bound is checked per code, DEV rows may not repeat an admitted cell, and two more leave-one-out edge cases are tested.
- **I typed the record's commit by hand once, and it was wrong.** The second run's `--discrete-commit` began with the right 9 characters, but the rest was made up. I deleted that output, unpublished, and reran with `"$(git rev-parse HEAD)"`. The sealed record's commit equals `git rev-parse HEAD`.
- **The first sealed record, `a0af69ae…`, is replaced by `d9aa6cd1…`.** Every number is identical: a `jq` diff with the commit and arm R removed is empty. Only the recorded commit and arm R's wording differ.
- **The first builder subagent wrote nothing.** The guard makes subagents read-only without a suborder in `local/claude/<unit>/orders/`. The builder also reported starting at about 150k tokens of context, at its hand-back mark, so this session built the instrument itself.
- **This unit was first numbered claude-94.** `loop claim` gave the same number to `m3-nonrelation-prose-panel` (#3187), so this unit is claude-95: `0f9ae0647` renames the folder, and `efe524657` retitles the ORDER. Because the claude-95 ORDER came in by rename, not as a first commit of its own, `loop pr-check` gives one advisory WARN (`commit-order`, D76).
- **Arm R is NOT_RUN, as registered.** The lab found no Q1–Q3 checkpoint in the HF store's `local-archive/2026-10-20/uor-r4-local.files.txt` (no `checkpoint-4101`, `8202` or `12303`, padded or not) or in `archive/storage/`. The tool does not search for them.
- **QA's admission sets are not load-matched,** as the FIT reviewer foresaw. A_T admits a median of 988 candidate tokens against the writer's 255, so part of its gain is a larger candidate set. The README gives the load quartiles beside the band.

## What changed

The item carries no size estimate, so there is no ratio to report. Nothing changed under `crates/`.

| Class | Files | Lines |
|---|---|---|
| Code | 2 | +1,748 (`lib.rs` 1,647, `main.rs` 101) |
| Tests | 1 | +39 |
| Generated | 1 | +344 (`Cargo.lock`) |
| Docs | 5 | the lab README +170; the item status line +1/−1; the rendered roadmap page +3/−3; this task folder (ORDER 51, REPORT) |

- **The instrument:** `docs/labs/m1-w2-code-utilization/instrument/`, a detached workspace with a read-only path dependency on `uor-r4-discrete` (no `cuda`). It uses exact integers only.
  - **Load and authenticate:** the pins and byte sizes of the stream, sidecar, capture and W2 record, then the checks on ranges, distinct slots and conservation.
  - **Precedence 0:** 11 W2 numbers, each computed three ways: from W2's public functions, from the sealed record, and from the instrument's own tables. Six arms are also compared row by row with W2's own scoring.
  - **QA, QB, QC and the descriptive arms,** exactly as the sheet defines them. Arm R is recorded as NOT_RUN.
  - **Determinism:** the CLI runs the diagnostic twice and refuses any difference.
  - **13 unit tests,** plus an env-gated real-capture test.
- **`docs/labs/m1-w2-code-utilization/README.md`:** the numbers, what they unlock, the instrument, the artifacts, how to reproduce, and the limits.
- **The item:** `status: done`, with the roadmap page re-rendered.

## Measurements

The tool ran on this laptop, host only, about 6 s per run. Its inputs were the authenticated `w2-out-b/` from `run-b.tar` `6aa0a766…` (checked with `shasum -a 256` before extraction) and W2's `record.json` `69717971…`.

- **Precedence 0.**
  - All 11 checks agree with the sheet's values, W2's functions and the record.
  - All 6 row comparisons differ on 0 rows.
  - Σk_r = 114,296, F² = 18,288, M = 6,300.
- **QA = A1a.**
  - A_T reaches 9,064 against A_W's 1,920, with (b, c) = (307, 7,451). Against A_U (5,935) the pair is (882, 4,011).
  - A_G reaches 12,214 and A_Ω 13,726.
  - **A_U beats A_W:** a fixed list of the 8 most frequent cells admits more target cells than the trained address key.
  - Of T's hits, 798 fall on admitted rows and 2,616 on non-admitted rows.
- **QB = B2.**
  - T on the stride subsamples: 2,785, 2,981, 3,176, 3,345, 3,414.
  - The last pair is (b, c) = (235, 304), so d_T = 69 and T_proj = 3,552, below W2's H_law line (about 6,435). **G1 is declined.**
- **QC = C1.**
  - J reaches 7,262 against G's 6,300, with (571, 1,533).
  - 11,727 DEV rows have a seen joint key, and J_m2 reaches 7,251.
- **Record:** `results/claude/w2-utilization-2026-10-25/record.json`, sha256 `d9aa6cd1a9a46876d4e50f1143d481e810fdcfb216beeccf6422ce1b7d70db90`. A fresh `uor-store get` returns the same sha256.

## Checks run at the head

Recorded in the PR's review record.
- `cargo test --release` passes all 13 unit tests.
- The real-capture test passes, with byte-identical runs.
- The planted `in_top` fault fails its test.
- `cargo fmt --check` is clean, and clippy reports nothing in the instrument.
- `grep -rnwE "f32|f64" src tests` finds nothing.
- `loop report --check` and `loop pr-check --local` pass.

## Skills used

- holding-the-goal: QB and QC are measured against the M1 bar, the T2 table's 6,300.
- verify-before-done: precedence 0 first, then the bands, then the record round-trip.

## Still open

- **The notebook row** is in #3189. It waits on the guard allowing the item's `notebook:` page.
- **Decision reviews these bands feed.** None is taken here.
  - **#2834 (full-vector address key) as the next M1 unit.** Its references are A_T 9,064, the ceiling A_Ω 13,726, and the fixed list A_U 5,935, which the trained key loses to.
  - **A read-change card,** with J = 7,262 as its reference.
  - **Stage T's "only if H_law" gate.**
- **G1** is declined.
- **Scratch data** in `local/claude/w2-utilization/`, about 90 MB with the build, is removed after the merge.

## Plain summary

The model reads its context into a 16-bit code and predicts the next word from that code alone. This run used only data the earlier W2 run had saved, to ask where that code loses information.
- **Admission.** The step that picks which groups of words to consider finds the right group on 1,920 of 14,287 rows. A rule keyed on the code would find it on 9,064, and even a fixed list of the most common groups finds it on 5,935, though both pass on more candidates.
- **More data.** The registered projection for a code-keyed table with more training data is 3,552 rows, far below the line where W2's verdict would change. Its gain had shrunk to 69 rows per doubling. So the extra GPU run is declined.
- **The previous word.** The code combined with the previous word predicts 7,262 rows, more than the previous word alone (6,300). So the code holds useful information, but it does not keep the previous word itself.
