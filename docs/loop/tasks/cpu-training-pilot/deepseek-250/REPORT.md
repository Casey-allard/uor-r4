# REPORT: cpu-training-pilot, deepseek-250

Lab: deepseek. Order: [ORDER.md](ORDER.md). Parent: [deepseek-249](../deepseek-249/REPORT.md).

## What did not work, first

- The final documentation commit changed the task-tree byte total after the
  ledger's first update. The third review caught the 250-byte undercount and
  the timestamp that preceded that change.

## What changed

- Recomputed the complete `deepseek-247` through `deepseek-250` task tree plus
  the resource ledger after all final documentation existed.
- Updated `updated_utc` and free space from the same final measurement.
- No executable, manifest, evidence report or accepted checkpoint result
  changed.

## Measurements

- Final task-plus-ledger bytes: 49783.
- Existing temporary input footprint remains 556,424,899 bytes.

## Checks run at the head

- Final report policy, local policy and all 11 full gates passed at this head.

## Skills used

No system-provided Codex skill applied.

## Still open

One final read-only review is owed.

## Plain summary

This closes the final ledger after all task records and measurements exist.
