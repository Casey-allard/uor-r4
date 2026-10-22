# REPORT: m2-reply-grading-instrument-r14, antigravity-12

Lab: antigravity. Item: [m2-reply-grading-instrument-r14](../../../../roadmap/items/m2-reply-grading-instrument-r14.md). Order: [ORDER.md](ORDER.md).

References #2852. References #2029.

## What did not work, first

- **Six shallow relative links in `docs/labs/m2-reply-grading-instrument-r14/README.md` on PR `#2956` (`ec980ee57`):** Because `docs/labs/m2-reply-grading-instrument-r14/` sits two levels below `docs/` and three levels below the repository root, the 6 relative links on lines 5 (`../roadmap/items/...`), 8 (`../../archive/labs/...`), and 20, 21, 31, 98 (`../../crates/uor-r4-training/src/bin/chat-grade.rs`) were one or two `../` levels too shallow, raising `xtask links` broken links from `930` to `936` (`new broken: 6` in CI's `docs archive and links` gate, identified in Kimi's post-merge review `https://github.com/UOR-Foundation/uor-r4/pull/2956#issuecomment-6338872888`).

## What changed

- [`docs/labs/m2-reply-grading-instrument-r14/README.md`](../../../../labs/m2-reply-grading-instrument-r14/README.md):
  - Line 5: `../roadmap/items/m2-reply-grading-instrument-r14.md` $\to$ `../../roadmap/items/m2-reply-grading-instrument-r14.md`.
  - Line 8: `../../archive/labs/m1-grader-v2-checks-2026-10-14/README.md` $\to$ `../../../archive/labs/m1-grader-v2-checks-2026-10-14/README.md`.
  - Lines 20, 21, 31, 98: `../../crates/uor-r4-training/src/bin/chat-grade.rs` $\to$ `../../../crates/uor-r4-training/src/bin/chat-grade.rs`.

## Measurements

- `~/.cargo/bin/cargo run -q --locked -p xtask -- links --baseline /tmp/antigravity-m1-norm-base-links.json --renames-from origin/main` -> `930 broken` (`new broken: 0`, `fixed: 6` compared against `origin/main` `ec980ee57`'s `936 broken`).

## Checks run at the head

- `~/.cargo/bin/cargo run -q --locked -p xtask -- links --baseline /tmp/antigravity-m1-norm-base-links.json --renames-from origin/main` -> `new broken: 0`, `fixed: 6`.
- `uor-harness loop gates --full` and `uor-harness loop report --receipts` -> 6 gates `GREEN`, receipts recorded in [`receipts.jsonl`](receipts.jsonl).
- `uor-harness loop report --check` and `uor-harness loop pr-check --local` -> `PASS`.

## Skills used

- `solution-integrity`
- `scope-guard`

## Still open

- PM ruling on R14 non-vacuity of the `v2` reply-grading instrument (`AbstentionDiscrimination`) on `#2852` / `#2029`.

## Plain summary

Repaired the 6 relative markdown links in `docs/labs/m2-reply-grading-instrument-r14/README.md` (lines 5, 8, 20, 21, 31, 98) from PR `#2956` (`ec980ee57`), restoring `xtask links` from `936` broken back to the `930` baseline (`new broken: 0`, `fixed: 6`).
