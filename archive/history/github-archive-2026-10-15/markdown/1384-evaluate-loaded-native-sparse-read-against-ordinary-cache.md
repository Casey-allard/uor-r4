# #1384 — Evaluate loaded native sparse read against ordinary cache

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-24T15:38:52Z
- **Updated:** 2026-10-09T13:17:43Z
- **Closed:** 2026-09-24T18:15:59Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1384
- **Merged:** not merged
- **Merge commit:** 6082721189f99781acf6e26a6f25cbe2a4db31d8
- **Base / Head:** codex/realtext-sparse-recall-20260924 / UOR-Foundation:codex/native-sparse-read-20260924
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Scope

Integrates a bounded exact-pair store and a fitted, four-bit NoRead/recent/previous source selector beside the unchanged loaded native `TlModel`. A selected read emits an exact owned token and feeds that token and Copy event through the native recurrence. The original checkout and prior artifacts remain untouched.

## Predeclared source-separated result

The selector fails its promotion gate. On two new Project Gutenberg sources (2,032 next-token positions), loaded NoRead / ordinary recent cache / learned selector / count reference top-one counts are 84 / 323 / 319 / 115. On two separate Rust modules (2,032 positions), they are 193 / 675 / 674 / 409. The fitted table reads on 1,498 of 1,504 recent hits and chooses an older source only six times. Four source-only edits change first copied output while NoRead stays fixed, but none of eight actual eight-token continuations is wholly correct. These are source-exposed development results, not general language or geometric-advantage evidence.

## Verification and cost

One offline release build and one claimed/sealed final run succeeded. The runner checks causal write/read order, artifact hashes, gate serialization/reload, read-disabled equality and source edits. An independent CSV tally reproduced all 4,064 positions and aggregate top-one/read counts. `cargo fmt --all --check`, `git diff --check` and `python3 scripts/check_claim_wording.py` passed. No full lib test harness was run.

The current model still scans 611,814 packed weight slots per full generated step; a selected read skips the 602,406-slot readout for that step while retaining the dense transition. No whole-path latency or energy measurement was made. The report and complete scope/limitations are in `docs/integration/native-sparse-read-result-2026-09-24.md` and the sealed root `/Users/casey.allard/uor-r4-investigations/native-sparse-read-20260924/final1`.

Stacked on #1383. References #820.


## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 5819634033 — 2026-09-24T18:15:58Z — Casey-allard

Delivered as part of owner-authorized consolidated PR #1386, merged through the protected main queue at `b020f34a09816ba235984cedde1c50f4c8e9cae2`. The main tree equals reviewed stack head `d1f5524c1d2b302cf2c07fea6b47309460b85f66` exactly (tree `c68b8b332080d09884b27191731b56296dc4a1f4`). Closing this component PR as incorporated; its experimental limitations remain as recorded. Programme #820 and #973 remain open.



---
_Comment/review ids in this file: 5819634033_

