# REPORT: cpu-training-pilot, deepseek-258

Lab: deepseek. Order: [ORDER.md](ORDER.md). Parent:
[deepseek-257](../deepseek-257/REPORT.md).

## What did not work, first

- The fourth fresh exact-head review found no code or behavior defect. It
  withheld approval because the final deepseek-257 size commit added one byte
  to that REPORT after the resource ledger measured the retained text.
- The ledger declared `82,835` bytes while the exact tree totalled `82,836`.
  A submission receipt that differs from its files is not complete, even when
  the difference is one byte.

## What changed

- Updated the deepseek-257 report to record the fourth review and this repair.
- Recomputed the retained-text receipt at this unit's final head:
  deepseek-254 through deepseek-258 plus the resource ledger total
  `87536` bytes.
- Corrected only `storage.task_and_ledger_bytes` and the ledger timestamp in
  `resource-ledger.json`. No Rust source, checkpoint, evidence report,
  executable, notebook row or scientific claim changed.
- Size relative to the resource estimate is negligible; `pr-check` supplies
  the exact base-to-head line counts below.

## Measurements

- Exact task-folder files plus the resource ledger at the deepseek-258 head:
  `87536` bytes. Deepseek-259 later adds real-checkpoint evidence and records a
  new final receipt.
- Focused binary tests: eight remain defined; this record-only repair does not
  rebuild or execute them.
- Evidence hashes and release executable identity are unchanged from
  deepseek-254 through deepseek-257.
- `pr-check` size: tests 0 files `+0/-0`; code 2 files `+937/-109`; docs 12
  files `+719/-0`; generated 0 files `+0/-0`; other 4 files `+1864/-0`.
- External GPU cost remains `$0`; no pod, store upload or paid API was used.

## Checks run at the head

- Exact task-plus-ledger byte arithmetic, JSON parsing and `git diff --check`:
  passed.
- `loop report --check --order .../deepseek-258/ORDER.md` and
  `loop pr-check --local`: passed. The only warnings were five pre-existing
  FILEMAP advisory rows.
- `loop gates --full`: all 11 gates passed.
- A fifth fresh exact-head review is requested after the final receipt commit.

## Skills used

No system-provided Codex skill applied.

## Still open

The real held checkpoints were `NOT_RUN` at the original 06:07Z probe and were
audited after Claude-334 published by deepseek-259. This receipt repair adds
no B2, carry, capability, export or serving evidence.

## Plain summary

The audit code and scientific scope did not change. This unit makes the final
resource ledger agree exactly with the files committed beside it.
