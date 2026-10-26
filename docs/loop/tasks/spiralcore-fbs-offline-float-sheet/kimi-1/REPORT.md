# REPORT: spiralcore-fbs-offline-float-sheet, kimi-1

Lab: kimi. Order: [ORDER.md](ORDER.md).
References #2891.

## What did not work, first

- `uor-harness ratchet d23` failed on `crates/uor-r4-float-train/src/spiralcore_fbs_sheet.rs` (0 -> 1) after #3252. Four harness library tests were red, blocking generated render PR #3257.
- The only `softmax` token in that file was in the comment `No softmax, squashing or dense matrix product occurs.` This was a scanner/comment defect, not a mechanism failure.

## Cancel probe

Ran `uor-harness ratchet d23` and searched the file. The scan reported exactly this one new file above baseline, and the only matching token was in that comment. A comment-only repair was sufficient; no code, model, sheet, bar, hold or GPU change was needed.

## What changed

- Replaced the offending sentence with `The arithmetic stays within the declared geometric contract.`
- No executable code, item status, model behavior, dependency, bar or hold changed. Size: code/comments 1 file +2/-2; ORDER +27; REPORT +46; generated 0.

## Measurements

- `uor-harness ratchet d23`: PASS — `OK: no softmax added. 4451 remaining across 165 files (baseline 4451).`
- `cargo test -p uor-r4-harness --lib`: PASS — 676 passed, 0 failed, 1 ignored in 254.42s.
- No GPU run, model evaluation or panel scoring ran.

## Checks run at the head

- `git diff --check`
- `uor-harness ratchet d23`
- `cargo test -p uor-r4-harness --lib`
- `uor-harness loop report --check`
- `uor-harness loop pr-check --local`
- `uor-harness loop gates --full`

## Skills used

- `uor-research-architect` for scope and evidence discipline.
- `working-discipline` for live-state verification and exact checks.

## Still open

- Full gates, in-unit fresh review, CI, queue, fresh-main verification and cleanup. After this lands, merge/render fresh main into #3257 and re-review if its head changes.

## Plain summary

A comment contained a prohibited token and tripped the source scanner. Rewording the comment fixes the scanner without changing the model or any experiment.
