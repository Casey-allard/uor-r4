# REPORT: cpu-training-pilot, deepseek-253

Lab: deepseek. Order: [ORDER.md](ORDER.md). Parent: [deepseek-251](../deepseek-251/REPORT.md).

## What did not work, first

The second fresh review approved the parser, statistics and tests but found:

- the recorded build target had grown after the ledger measurement;
- the exact command named `target/release` while the recorded binary lived
  under `target/deepseek-carry-hold-audit/release`;
- `NOT_RUN` had no retained store inventory receipt.

## What changed

- Bound README and REPORT commands to
  `CARGO_TARGET_DIR=target/deepseek-carry-hold-audit`.
- Retained [store-runs-inventory.txt](../deepseek-251/evidence/store-runs-inventory.txt),
  which records HTTP 404 and exit 1 for the future run root.
- Refreshed the resource ledger's build-target bytes, task bytes, free space
  and timestamp after the final records existed.

## Measurements

- Store `runs/` inventory: HTTP 404, exit 1, real checkpoints `NOT_RUN`.
- Final storage values are recorded in the resource ledger.

## Checks run at the head

- Store inventory receipt retained.
- Final JSON, report policy, local policy and all 11 full gates passed.

## Skills used

No system-provided Codex skill applied.

## Still open

One final read-only review is owed.

## Plain summary

This close-out makes the future command point at the actual recorded binary and
retains proof that the real B2 run did not yet exist.
