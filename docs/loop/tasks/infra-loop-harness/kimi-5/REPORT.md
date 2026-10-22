# REPORT: infra-loop-harness, kimi-5 (repair: one missing umbrella glob)

Lab: kimi. Order: [ORDER.md](ORDER.md).

## What did not work, first

- The kimi-4 umbrella scope dropped one glob from claude's #2028 list: `crates/uor-r4-harness/tests/loop_shared_files*.rs`, the file claude-22 adds under this umbrella. Without it, `pr-check`'s `scope` rule would flag claude-22's own test file. Added, and named in kimi-4's ORDER Goal 3 and REPORT "What changed" (the MEDIUM).
- kimi-4's REPORT said "Result recorded here once run" instead of the actual result. Corrected: `pr-check --local` at `995286d33` was a full PASS (the LOW).
- Repair-chain housekeeping: with kimi-4 and kimi-5 ORDERs the PR would add two ORDERs, which both gates refuse. The chain is folded as kimi-3 did it — kimi-4's ORDER.md is removed at the head, its steps carried in this ORDER's history, and its REPORT stays with a pointer. The umbrella–pr08 overlap on `docs/roadmap/items/*.md` is the reviewer's note only, already on main.

## What changed

- `docs/roadmap/items/infra-loop-harness.md`: appended `crates/uor-r4-harness/tests/loop_shared_files*.rs` to the umbrella scope.
- `docs/loop/tasks/infra-loop-harness/kimi-4/ORDER.md` and `REPORT.md`: the two named-in corrections and the fold pointer; `kimi-4/ORDER.md` removed at the head.
- `docs/loop/tasks/infra-loop-harness/kimi-5/`: this ORDER and REPORT.

Size by class: docs 4 files, one scope line plus record text; no code, no tests, no generated files. The ORDER set no estimate (D48).

## Measurements

- `grep '^scope:' docs/roadmap/items/infra-loop-harness.md` → carries `crates/uor-r4-harness/tests/loop_shared_files*.rs`.
- `cargo run -q --locked -p xtask -- roadmap check` → 87 items OK; `render --check` → current.

## Checks run at the head

- `xtask roadmap check` and `xtask roadmap render --check`: pass.
- `uor-harness loop gates --quick`: GREEN (run before commit).
- `uor-harness loop report --check`: PASS for this REPORT.
- `uor-harness loop pr-check --local` at the kimi-5 head: `order` and `report` PASS; `scope` and `commit-order` FAIL for the same structural repair-chain reason flagged in kimi-3 — the rule reads the umbrella's scope at the merge base (the old narrow one), and kimi-4's folder is no longer the order's own folder, so the fold edits fall outside; the ORDER commit cannot be first on a repair branch. The scope check passed at `995286d33` before the repair.

## Skills used

- scope-guard: written edge, `git diff --stat` before content. The ORDER and the delta CHANGES record were the opener; the one-line fix was applied as directed.

## Still open

- claude-22's exemption build and its `loop_shared_files` tests; the one-ORDER/`commit-order` repair-chain gap remains the architect's to answer in the gates (the fold is the working pattern until then).
- Lane's standing question to Casey: whether kimi takes loop PR 8 (#2913) next; a pending owner question means wait.

## Plain summary

The delta review of the small follow-up found one missing file pattern in the umbrella item's list — the test file the architect's own next change will add — and a lazy line in the earlier report. Both are fixed, and the repair chain is folded to a single order the way the gates require.

References #2904.
