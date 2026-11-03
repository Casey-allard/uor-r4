# REPORT: geometric-image-input, deepseek-4

Lab: deepseek. Item:
[geometric-image-input](../../../../roadmap/items/geometric-image-input.md)
(#3619). Order: [ORDER.md](ORDER.md). Base: `8fb9f3719`.

## What did not work, first

- PR #3628 merged at `5a7a5a09b` while two post-review record corrections were
  still local. The implementation was on `main`, but its report still retained
  the older 451-test count and described a clean first gate attempt.
- The final reviewer's first full gate run at that source head hit the known
  fake-`gh` `ETXTBSY` in `loop_drive`; 27 of 28 gate tests passed. The immediate
  rerun passed all 11 gates.
- The repair worktree then repeated the same known fake-`gh` `ETXTBSY` on its
  first gate run before the immediate rerun passed all 11 gates. Both attempts
  are retained.
- The historical 451-test receipt was correct for its earlier source tree and
  is preserved unchanged. Current `main` added one integer test.

## What changed

This repair carries the two already-reviewed record corrections onto current
`main` through a new ORDER:

- add an exact merged-head integer receipt for 452 passed and 1 ignored;
- update the active REPORT and M4 notebook count;
- retain both the failed gate attempt and the passing immediate rerun;
- update the evidence checksum manifest without changing any image source.

## Measurements

At merged image source head `5a7a5a09b`, the integer library ran 453 tests:
452 passed, 1 ignored, 0 failed. The earlier 451-pass receipt remains intact.

The image implementation's earlier functional evidence is unchanged:

- 17 focused image/API/composer tests;
- 266 Studio tests across 31 files;
- 14 native loopback tests;
- real native browser flow with actual request-body checks for a live
  image-only request and reloaded-history resend;
- approximately 31 prompt / 6 completion / 37 total tokens on the retained
  final browser reading;
- 11 loop gates on the immediate rerun after the retained `ETXTBSY` flake.

## Checks run at the head

- Evidence checksum verification.
- `uor-harness loop report --check`.
- `uor-harness loop pr-check --local`.
- `uor-harness loop gates --full`: passed all 11 gates on the immediate rerun
  after the retained `ETXTBSY` flake.

The repair gate attempts are retained in
[evidence/loop-gates.txt](evidence/loop-gates.txt).

## Skills used

No available Codex skill applied. This is a record-only repair with no tool or
workflow skill dependency.

## Still open

Visual understanding, semantic image answers, useful multimodal conversation,
M4 and M7 remain open. Exact-head review and protected merge are pending at
this report revision. This repair changes records only.

## Plain summary

The image input is already on `main`. This small follow-up makes the retained
records agree with current `main` and preserves both the known harness flake
and its green rerun.
