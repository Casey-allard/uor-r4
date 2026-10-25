# REPORT: m3-nonrelation-prose-panel, claude-98

Lab: claude. Order: [ORDER.md](ORDER.md). Repairs claude-94 (#3188).
References #2856.

## What did not work, first

- **claude-94 turned four harness lib tests red on `main`.** Its compiler's module doc said the compiler had "no score, probability, [the banned word] or matrix product anywhere". The D23 interlock counts that word even inside a `//!` doc comment, so the file was 1 above its baseline of 0.
  - The red tests were three `prlintrules` tests and `prgates::check_pr_tests::every_ported_gate_and_the_dispatcher_agree_with_python`, all 11 of whose differences were that one `prlint.softmax_ban` finding.
  - `loop pr-check`'s banned-presence rule skips comments, so it passed at #3188's head. #3188's CI did not run the harness lib tests, because the PR changed no harness file. claude-97's full harness run found it.
- **One more lib failure on `main` is not this unit's.** `prep::tests::the_real_sheets_are_readable` fails because `docs/harness/prep/d41-driver-math-pilot-2026-10-25.md` (#3190, antigravity) has no verdict line. That is antigravity's file, so it is noted for them on #2028 and left alone here.

## What changed

One line of `crates/uor-r4-training/src/sieve_frame_compiler.rs`'s module doc now reads "the decision has no score, no probability and no matrix product." No code changed.

## Measurements

None. This is a repair of an instrument reading.

## Checks run at the head

- **The cancel probe held at `b19b15f00`:** the word was at line 14, and the four tests failed with the interlock reporting that file `0 -> 1`.
- `uor-harness ratchet d23`: "OK: no softmax added. 4451 remaining across 165 files (baseline 4451)". On `main` before this fix, the same file was 1 above its baseline.
- `cargo test -p uor-r4-harness --lib -- prlintrules:: prgates::check_pr_tests::every_ported prep::tests::the_real_sheets`: 11 passed. The one failure is the d41 sheet test above, which fails identically on `main`.
- `cargo test -p uor-r4-training --lib`: 72 passed.
- The fresh review is in the PR's review record.

## Skills used

- holding-the-goal: a red interlock on `main` caused by this lab is repaired at once in its own task folder, not left for the next PR to trip over.

## Still open

- Antigravity's d41 sheet needs its verdict line before `prep::tests::the_real_sheets_are_readable` is green again.
- A lesson candidate: a lab PR that adds a new `.rs` file should run `uor-harness ratchet d23` (and d25) even when it touches no harness file. pr-check's comment-blind banned-presence rule does not see doc-comment words that the interlock counts.

## Plain summary

A comment in the new prose-panel compiler used a word the project's ban-checker counts even in comments, which turned four checks red. The comment is reworded, and the checks are green again.
