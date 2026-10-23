# REPORT: m1-position-mechanism, kimi-32

Lab: kimi. Order: [ORDER.md](ORDER.md).

References #2843

## What did not work, first

The sheet's current status still said `NOT_FIT`, and its post-registration update still called the slice hash `PENDING`, after all three prerequisites had been resolved. The expected `data/ft-dev.tar` path was absent in this worktree and the owner checkout; Kimi's preserved local tarball supplied the input for reproduction. The finite build emitted existing dead-code warnings, but no test or gate failed. No GPU wiring, GPU run, order-effect measurement, mechanism verdict, item completion or owner release was attempted.

## What changed

The current sheet status now records Kimi's non-author `TEST FITNESS: FIT`; a post-freeze review resolves the owner-bar, DEV-pin and frozen-slice findings while preserving the earlier `NOT_FIT` verdict and update as history. The ORDER narrows scope to the sheet and this task folder. Three documentation/task-record files change; there are no source, test or generated-file changes. The ORDER gave no numeric size estimate, so a size ratio is not applicable.

## Measurements

- `cargo test -p uor-r4-discrete --lib position_mechanism`: **5 passed, 0 failed**, 225 filtered out.
- Preserved DEV tarball SHA-256: `45e93e32918f8ad5c977ad3a98926b561173049aacd8bfce2b6e2e49a0390b5c`.
- Extracted DEV tokens SHA-256: `0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b`; response mask SHA-256: `a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078`.
- `order-slice-freeze TOKENS MASK OUTPUT` on that pair: 14,287 rows scanned, 1,620 selected, 17,789 bytes, SHA-256 `f76dc5c7ff6c359dd4c9158abc280ff29105cb611f6c4c2bdba6fdfed0035b3a`; `cmp` against `docs/labs/m1-position-mechanism/order-slice-v1.tsv` returned equal.
- `uor-harness prep-check`: this sheet is `FIT`, `complete=yes`, with no missing blocks.

## Checks run at the head

- `git diff --check`: pass.
- `uor-harness mistakes find -f docs/harness/prep/m1-position-mechanism-DRAFT-2026-10-22.md -f docs/loop/tasks/m1-position-mechanism/kimi-32/ORDER.md`: six applicable lessons surfaced; no deletion, prohibited teacher or comment edit was performed.
- `uor-harness loop gates --quick`: 3/3 GREEN.
- `uor-harness loop gates --full`: 11/11 GREEN.
- `uor-harness loop report --check`: PASS.
- `uor-harness loop pr-check --local`: the pre-commit run failed only `report` because this REPORT was not yet at the head; the committed-head rerun passed every rule.

## Skills used

`uor-research-architect` and `superpowers:using-superpowers`.

## Still open

The roadmap item remains `ready`. GPU wiring and the registered two-seed order test remain unrun under #2843; any GPU work still needs the named trunk dependency, PM dispatch and Casey's D40 release. This FIT establishes configuration readiness only, not that the mechanism helps order.

## Plain summary

The position experiment's plan had become inconsistent: its detailed data section named the registered bar and frozen evaluation rows, but its headline still called the plan unfit. Kimi independently rebuilt the frozen rows from the pinned development data, got exactly the same file and hash, and reran the five small position-mechanism tests successfully. The plan is now fit as a configuration for a later experiment. This change does not run that experiment, show that the mechanism works, finish the roadmap item, or permit GPU use.
