# ORDER: m1-w2-code-utilization, claude-89 (register the code-utilization sheet and its FIT)

Lab: claude. Item: [m1-w2-code-utilization](../../../../roadmap/items/m1-w2-code-utilization.md) (#3179). Base: `a4eaaf238`.

## Goal

The item went onto `main` in #3178. Its done-when needs a registered sheet with a non-author FIT before the diagnostic runs. This unit writes `docs/harness/prep/m1-w2-code-utilization-2026-10-25.md`, which holds:
- the question;
- the arms, with their exact integer definitions;
- the decision table and what each outcome unlocks;
- the instrument checks;
- the cost.

The FIT comes from a fresh-context reviewer (CHARTER §4.5) and is recorded in the sheet.

The sheet registers only host post-processing of the stored W2 capture. It names the GPU arms that would need a run (the full-TRAIN capture, the init snapshot) and registers neither. Nothing runs in this unit: the diagnostic itself is the next unit, which `loop next` offers once this merges.

## Cancel probe (run first)

Class: docs (host only). At the base:
- `ls docs/harness/prep/m1-w2-code-utilization*` finds nothing;
- the item reads `status: ready`, and its only dependency is `done`;
- the W2 README still pins `run-b.tar` `6aa0a766…`.

The unit stands.

## Files in scope

- `docs/harness/prep/m1-w2-code-utilization-2026-10-25.md` (new)
- this task folder

## Done when

- `uor-harness check --only prep` exits 0 and no finding names the new sheet: its shape is complete, and its last `TEST FITNESS` line reads FIT.
- The FIT is non-author: a fresh-context review whose defects are listed, with how each was resolved, in the REPORT.
- `loop report --check` and `loop pr-check --local` pass.
- A fresh-context review at the exact head counts (D66).
- The REPORT says `References #3179`: the item stays `ready` for its run.

## Stop and ask if

- The FIT finds an arm that needs a GPU or an input not in the store, and the arm cannot be dropped without changing what a band decides.
- A band would change a bar, a status or a gate. Stage T's gate is a decision review's call.
