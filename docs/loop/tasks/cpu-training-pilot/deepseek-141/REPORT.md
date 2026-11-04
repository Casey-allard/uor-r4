# REPORT: content-readout serving scope, deepseek-141

Lab: deepseek. Item:
[cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506).
Order: [ORDER.md](ORDER.md). Base: `70f45367d`.

## What did not work

- **The serving unit exceeded its item scope at its base.** The base item
  authorized `uor-r4-cpu-train` but none of the integer serving files,
  integration state or content-serving evidence tree used by deepseek-140.
- **Changing the item scope in the consuming branch did not fix delivery.**
  Scope is evaluated from the branch base, so a branch cannot widen the
  permission it is already using.

## What changed

The item now explicitly permits the exact integer-serving, CLI, loopback,
session-codec, integration-record and content-serving evidence paths required
by the separate deepseek-140 unit. Every previous scope entry is retained.

This change does not alter runtime code, tests, artifact formats, evidence,
status, bars or holds.

## Measurements

This code-neutral prerequisite ran no model, build or test workload. The
roadmap scope retained all 13 base entries and added 13 focused entries. The
branch has four changed paths: the roadmap item, this unit's ORDER and REPORT,
and the branch claim.

## Checks run at the head

- Full `uor-harness check`: **194 rules, 0 errors, 161 repository-wide
  warnings**; no warning attaches to the scope path itself.
- `git diff --check` passed.
- The diff contains only the roadmap item, this task's ORDER/REPORT and the
  branch claim.

## Skills used

No system-provided Codex skill applies.

## Still open

- Complete this branch's own exact-head review, protected merge and
  fresh-`main` verification before rebasing deepseek-140.

## Plain summary

This prerequisite makes the content-readout serving work legally in scope
without changing model behavior or any planned result.
