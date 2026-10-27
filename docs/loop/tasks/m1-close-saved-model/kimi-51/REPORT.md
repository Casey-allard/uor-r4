# REPORT: m1-close-saved-model, kimi-51

Lab: kimi. Order: [ORDER.md](ORDER.md).

References #2837 and #3319. Host-only instrument repair; no milestone or issue closes.

## What did not work, first

Harness CI on the render PR failed `preprules::tests::the_real_sheets_pass_the_shape_rule`: the M1-close prep sheet lacked Base artifact, Data split and Steps blocks. The executed M1 result was not implicated.

## What changed

The existing checkpoint/control content is under Base artifact, the pinned corpus under Data split, and the GPU protocol under Steps. Cost/power and verdict are renumbered consistently. No result, bar, threshold, evidence, code or issue state changed. Size: docs 3 files +80/−6 and receipts 1 file +11/−0; no generated files. The ORDER gave no numeric estimate, so a ratio is not available.

## Measurements

No experimental measurement. `cargo test --locked -p uor-r4-harness the_real_sheets -- --nocapture`: 3 passed, covering the formerly failing shape rule and both real-sheet readers.

## Checks run at the head

- Real-sheet tests: 3 passed.
- `git diff --check`: PASS; only heading/section changes.
- `CARGO_TARGET_DIR=local/target ~/.cargo/bin/cargo test --locked -p uor-r4-harness --no-fail-fast`: 677 passed, 1 ignored, including the formerly failing shape rule.
- `uor-harness loop gates --full`: all 11 gates GREEN at `839cd7f0f`.
- `uor-harness loop pr-check --local`: PASS at `839cd7f0f`, with no advisory warnings.
- `uor-harness loop report --check --order docs/loop/tasks/m1-close-saved-model/kimi-51/ORDER.md`: PASS at `839cd7f0f`.
- Fresh exact-head review, required checks, queue and fresh-main verification are required.

## Skills used

- **systematic-debugging:** read the exact CI failure and repaired the heading shape rather than weakening the test.
- **solution-integrity:** preserved the executed M1 evidence and kept this separate from the render PR.

## Still open

- Merge this repair forward into #3319 and rerun its harness CI.
- The closed-#2820/ready-item mismatch remains separate.
- Pre-existing bar labels (a)–(c) in the sheet differ from the D39/roadmap definitions. This PR does not redefine them or claim the records are the same five-bar claim; any reconciliation needs a separate decision card.
- The sheet's fresh thresholds (`6,796`, `1,401`, `221`) differ from the fresh-panel/evaluator records (`6,582`, `1,240`, `229`). They are preserved, not silently corrected; which floor was registered before the sealed run needs a separate evidence review.
- Public README/milestone status differs from the roadmap and executed sheet. This PR does not reconcile that status or claim M1 closure. Those records and the retained 4-table negatives/archive remain untouched.

## Plain summary

The prep sheet now uses the section names the harness requires. Nothing about the completed M1 experiment changed.
