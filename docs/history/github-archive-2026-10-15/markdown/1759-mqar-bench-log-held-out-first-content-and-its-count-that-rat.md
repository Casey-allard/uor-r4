# #1759 — mqar-bench: log held-out first-content and its count; that rate was never printed

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-05T20:39:02Z
- **Updated:** 2026-10-09T14:16:57Z
- **Closed:** 2026-10-05T21:24:44Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1759
- **Merged:** 2026-10-05T21:24:44Z
- **Merge commit:** e69ab89390ea40c5882a0ad8826ebbcd2d1b2c02
- **Base / Head:** main / UOR-Foundation:bench-heldout-first-content
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## What this fixes

The per-step training log printed `first-content` for the **in-class** set (via `summary()`, `fact.rs:883-900`) but only `full` for the **held-out** set (`fact.rs:987`). The held-out first-content rate is the instrument's own headline — `first_content()` exists specifically to skip the whitespace-only leading piece of a numeral value (`fact.rs:270-276`, `numerals[0]` = `[" ", "0"]`) — and it was the one number a reader of run logs could not see.

The consequence is on the record. The 2026-10-05 session handoff diagnosed the geometric stack's transport failure from `first_piece_accuracy ≈ 0.29`, read out of `report.json`. For the sealed `ro-full-base` run those two fields are:

| field | held-out, step 1200, n=96 |
|---|---|
| `first_piece_accuracy` | 0.2917 — **28/96** |
| `first_content_piece_accuracy` | **0.0104 — 1/96** |

All 28 `first_piece` hits are in the `value.numeral` marginal and are the leading-whitespace token; the `value.word1`/`word2` marginals (62 items) are 0. The run's own `log.txt` says `first-content 0.010–0.052` on all 14 lines and never contains the string `first_piece` — the log had taught the reader the right metric name, and the missing held-out print is what sent them to the wrong field.

## Changes

1. **Held-out log line** now prints `full`, `first-content`, the item count, and the occurrence counts for both:

   before: `| held-out full 0.010 (12s)`
   after: `| held-out (n=96) full 0.010 1/96 first-content 0.010 1/96 (12s)`

2. **`first_content_correct`** is emitted in the tally record. Only rates were emitted before, so a 1/96 was readable as a rate with no denominator available anywhere in the report.

## Scope and safety

- **Observability only.** No computed metric changes; the added JSON field is additive and no existing key, value or denominator is touched. Existing readers of `first_content_piece_accuracy`/`full_accuracy` are unaffected.
- Diff is one file, **+12/−2**, and `git diff --name-only origin/main..HEAD` lists only `crates/uor-r4-training/examples/mqar_bench_step2/fact.rs`. No unrelated reformatting.

## Checks

- `cargo build --release -p uor-r4-training --example mqar-bench` — clean (exit 0; only the three pre-existing dead-code warnings).
- `rustfmt --edition 2021 --check` on the changed file — clean.
- **Exercised the changed path**, not just compiled: a 60-step `layout=fact` run into a fresh sealed root `bench-hofc-smoke` (`steps=60 batch=4 final_sequences=8 seed=7`) produced the new line above with `n=96` and `1/96`. A second run on the final formatted artifact (`bench-hofc-smoke2`) produced identical numbers.
- Cross-checked against 140 sealed roots under `~/uor-r4-worktrees/reports/`: `full_accuracy ≤ first_content_piece_accuracy` in all 128 observed curve points (equality in 112), and all six mechanism arms land at 1–5 correct out of 96 on the canonical metric — i.e. the five handoff nulls hold on the canonical metric too, and the instrument cannot resolve them at n=96.

## Follow-up not included here

`curve_sequences` defaults to **8** (`mqar-bench.rs:1156`), so the curve evaluates **96 items** with per-cell counts of 2–9, while the final evaluation uses `final_sequences=256` → **1536 items**. The curve is underpowered ~16× relative to the final eval, and the curve is what diagnoses are read from. `final_eval_seconds ≈ 4.3 s` on a 466 s run, so raising the curve to 256 sequences costs roughly **+50 s (~11% wall) for 16× the power**. Deliberately **not** changed in this PR: it would break comparability with the 315 existing roots, so it is the owner's call.

References #820


## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 6002503089 — 2026-10-05T20:39:34Z — Casey-allard

## Exact-head self-review — `faf58c138d760650a77bf11a5d95fc515215fa68`

Owner direction permits self-review with a substantive check list. Reviewed at the exact head above.

### Correctness of the change

- **`first_content_correct`** is `self.first_content`, an existing `usize` field of `FactTally` incremented once per item in `tally()` (`fact.rs:618`). It shares the same `total` as the other rates, so the count and the rate cannot disagree. No new state, no new denominator. Additive JSON key; no existing key renamed, removed or re-valued.
- **Log line** reads `held_out["marginals"]["all"]` — the same object the old line read `full_accuracy` from, so no change in which aggregate is reported. `as_u64()`/`as_f64()` fall back to `0`/`NAN` on absence, matching the existing style and introducing no `unwrap`/`expect` on a recoverable path.
- **No computed metric changed.** The only writes are a format string and one added JSON field. Nothing in the training, evaluation or scoring path is touched.

### Checks executed

| check | result |
|---|---|
| `cargo build --release -p uor-r4-training --example mqar-bench` | exit 0; only the 3 pre-existing dead-code warnings |
| `rustfmt --edition 2021 --check <file>` | clean |
| CI `fmt / clippy / tests / no_std / κ` | **pass** |
| CI `cargo audit`, `wasm-pack`, `Gate C`, `fuzz smoke` | pass |
| changed path **exercised**, not just compiled | 60-step `layout=fact` run, fresh root `bench-hofc-smoke`: emitted `held-out (n=96) full 0.010 1/96 first-content 0.010 1/96` |
| re-run on the final formatted artifact | `bench-hofc-smoke2`, identical numbers |
| tested artifact == committed artifact | `git diff HEAD -- <file>` empty; blob `786ec479` |
| diff scope | `git diff --name-only origin/main..HEAD` → exactly 1 file, +12/−2; no unrelated reformatting |

### Evidence claims in the description

Every figure was machine-checked against 140 sealed roots before posting (25 assertions, all pass):

- `full_accuracy ≤ first_content_piece_accuracy` in **128/128** observed curve points; equality in 112. The 16 strict cases are all `full < first_content` and confined to `ng-full-ngram-{2,4}` (14) and `select-top-4` (2). The inequality is mathematically forced (`full ⊆ first_content`); the equality pattern is the empirical part.
- The 28/96 first-piece hits are **all** in `value.numeral` (34 + 34 + 28 = 96 partitions the held-out set exactly), and `value.numeral` first-content is 1/34. Word1/word2 are 0/62 on both.
- `0.2949` occurs exactly twice across all 140 roots, both as single `lock-s2` buckets; `6.4469`/`6.1042` occur nowhere. `ro-full-base` first_piece is 0.2917 at every post-zero point (not "at every point" — step 0 is 0.0104).
- `gate-save` vs `ro-full-base` both 28/96 at 600/1200 steps; five seeds 21/33/29/26/27.

### Scope judgement

Deliberately **not** included: raising the `curve_sequences` default (would change comparability for 315 existing roots and alter run cost) and any change to the gate/pointer mechanism. Both are called out in the description for the owner rather than done silently.

No blocking issues found.



---
_Comment/review ids in this file: 6002503089_

