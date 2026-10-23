# ORDER: legal-construction-exact-solve, deepseek-33

Lab: deepseek. Item: [legal-construction-exact-solve](../../../../roadmap/items/legal-construction-exact-solve.md) (#2828). Base: `ea257ef40`. Item PR (partial: `References #2828`; the item stays `ready`).

## Goal

Carry unit 3 of the design note ([#2828 comment 6351533475](https://github.com/UOR-Foundation/uor-r4/issues/2828#issuecomment-6351533475)) as far as the instrument honestly reaches: **settle the saved-case mapping, build the exact case loader, and record what the dense exact solve actually does.** The case is not solved by this unit; the design note's unit 3 expected a saved-case optimum or an infeasibility proof, and the dense exact root LP was already measured at 72+ minutes without returning (independent log in `local/claude/legal-case/root-lp-run1.log`, 2026-10-22T22:05:45Z, pivot 650, denominators ~15.6k bits). This unit turns that into a recorded instrument blocker with a resolved input mapping, so the next unit can choose a reduced method with evidence instead of re-deriving the inputs.

## Cancel probe (run first)

Class **instrument** (host only; no GPU, no release). At the base: the design note's open point 1 is unresolved (which files are `g` and `m`), no case loader exists, and the unit-2 crate has no saved-case binary. The original source at commit `ebd6ecaa` is present in the local object store, so the mapping can be settled from code rather than inferred. Task stands.

## Files in scope

- `docs/labs/legal-construction-exact-solve/**` (loader, binary, tests, lab record)
- `docs/mechanisms/legal-construction.md` (status paragraph)
- `docs/research/notebook/legal-construction.md` (new page, one row)
- this task folder

Not in scope: the saved data itself (gitignored `local/claude/legal-case/`), GPU/FIT work, or scoring on the real panel.

## Done when

- **Mapping settled from source.** `case.rs` loads `m = [prefix, generate]` masters, `g = [prefix, generate]` gradients and the 380 `protected-margin-NNN` guard rows, with citations to `ebd6ecaa` `protected_joint_vector.rs::legal_proposals` and `protected_legal_set.rs`.
- **Loader verified.** Unit tests (shape, NaN refusal, row-length refusal) pass; the saved case loads with `n = 1,920`, `guards = 380`, `fractional = 20`, and the exact zero move is legal and guarded (min guard value `0/1`, objective `0/1`).
- **F32 term-aggregate identity.** The 31 per-term gradient files per family sum, in f32 order, bit-exactly to the matching `coupled-*-gradient.f32le` file (both families).
- **Solve attempt recorded honestly.** The exact root LP is run under a recorded cap; it does not return in 120 s at this size (and the independent dense run took 72+ minutes without returning). No optimality or infeasibility claim is made; the result is recorded as an **instrument blocker** (D22).
- **Records:** lab README section, mechanism-page status paragraph, one notebook row (`blocked`), REPORT with the exact commands and timings.
- **Checks:** solver crate tests + no-float scan pass; `loop report --check`, `loop pr-check --local`, `loop gates --full` green at the head.

## Stop and ask if

- A file outside the scope above must change.
- The next step would need GPU compute, a pre-registration, a FIT review or an owner release — this unit stops at the host instrument.
- A claimed optimum/infeasibility would depend on the dense simplex result (it does not; the run timed out).