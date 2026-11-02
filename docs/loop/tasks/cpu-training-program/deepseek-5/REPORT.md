# REPORT: restore required preparation-sheet blocks, deepseek-5

Lab: deepseek. Item: [cpu-training-program](../../../../roadmap/items/cpu-training-program.md) (#3507). Base: `374c7bdd3`. Supersedes closed PR #3548, whose content was approved but could not satisfy the ORDER-first commit rule after its base task had already merged.

## What did not work, first

- **A merged Kimi performance sheet omitted six blocks required by `prep.sheet_shape`.** The repository-wide harness therefore reported one error despite the sheet's measurements and FIT records being complete.
- **The first repair branch did not add its own ORDER.** The content was reviewed and correct, but the harness correctly refused to treat an already-merged ORDER from another PR as this unit's first commit.

## What changed

- Added `Base artifact`, `Data split`, `Steps`, `Stop rule`, `Cost` and `Minimum test power` to the existing sheet.
- Derived every added statement only from the sheet's existing workload, pinned source heads, 504-row correctness gate, V/K grid, 240-record measurement contract, hardware scopes, cost cap and rejected-run record.
- Preserved every existing claim, pin, failed attempt, FIT record and NOT_RUN boundary.
- Added no code, GPU run or performance claim.

## Measurements

- No compute ran.
- The existing rejected and final-run records are unchanged.

## Checks run at the head

- `uor-harness check`: no errors; only pre-existing warnings remain.
- `cargo run -q --locked -p xtask -- roadmap check`: PASS.
- `cargo run -q --locked -p xtask -- roadmap render --check`: PASS.
- `git diff --check`: PASS.

## Skills used

No repository skill was invoked. This is a documentation-shape repair needed to restore the repository-wide harness.

## Still open

Nothing in this documentation-only unit.

## Plain summary

The existing Kimi performance sheet now satisfies the required D22 block shape without changing any of its evidence or claims.
