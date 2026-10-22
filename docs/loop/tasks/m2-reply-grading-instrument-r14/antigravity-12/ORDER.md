# ORDER: m2-reply-grading-instrument-r14, antigravity-12

Lab: antigravity. Item: [m2-reply-grading-instrument-r14](../../../../roadmap/items/m2-reply-grading-instrument-r14.md). Base: `ec980ee57`.

## Goal

The item's done-when reads:

> A scope note on `#2029` records, for each of the three R14 requirements, whether the v2 grader meets it and on what evidence, and says what is still missing. Each missing part is built and tested, or the note says plainly that it stays out of scope. For abstention scoring, the note shows a constant-reply control run and the result. A non-author reviews the note and the instrument. The PM, not this item, rules whether the instrument passes R14.

## Cancel probe (run first)

Ran `~/.cargo/bin/cargo run -q --locked -p xtask -- links` at `origin/main` (`ec980ee57`), which reports `936 broken` (up from `930 broken` at `3e3967c76` before PR `#2956` merged) due to the 6 relative links in `docs/labs/m2-reply-grading-instrument-r14/README.md` (lines 5, 8, 20, 21, 31, 98) identified by Kimi's repair card on `#2028` (`https://github.com/UOR-Foundation/uor-r4/pull/2956#issuecomment-6338872888`).

## Files in scope

- `crates/uor-r4-training/**`
- `data/panels/**`
- `docs/labs/m2-reply-grading-instrument-r14/**`

## Done when

1. All 6 relative links in `docs/labs/m2-reply-grading-instrument-r14/README.md` (lines 5, 8, 20, 21, 31, 98) are repaired (`../../roadmap/items/m2-reply-grading-instrument-r14.md`, `../../../archive/labs/m1-grader-v2-checks-2026-10-14/README.md`, and `../../../crates/uor-r4-training/src/bin/chat-grade.rs`).
2. `~/.cargo/bin/cargo run -q --locked -p xtask -- links --baseline /tmp/antigravity-m1-norm-base-links.json --renames-from origin/main` reports `new broken: 0` and `fixed: 6` (returning `main`'s broken link count from `936` to `930`).
3. `uor-harness loop gates --full`, `uor-harness loop report --receipts`, `uor-harness loop report --check`, and `uor-harness loop pr-check --local` pass.

## Stop and ask if

- Any file outside the declared scope needs modification.
