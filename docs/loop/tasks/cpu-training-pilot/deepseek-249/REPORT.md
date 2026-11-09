# REPORT: cpu-training-pilot, deepseek-249

Lab: deepseek. Order: [ORDER.md](ORDER.md). Parent: [deepseek-247](../deepseek-247/REPORT.md).

## What did not work, first

The second fresh exact-head review confirmed all five first-round repairs but
returned CHANGES on three residual record issues:

- The README said a missing archive was retained as a source failure. Archive
  verification actually aborts before a report exists.
- The temporary input total omitted 18,191 bytes of `SHA256SUMS` and `.list`
  metadata downloaded beside the archives.
- The ledger's `updated_utc` preceded the final measurements it recorded.
- The first full-gate run hit a transient `Text file busy` race in the harness's
  fake-`gh` loop-drive test. The clean rerun at the same head passed.

## What changed

- README now distinguishes early archive failure from per-arm checkpoint source
  failure and summary regression.
- The ledger records 18,191 metadata bytes and a complete 556,424,899-byte
  run-root input footprint.
- `updated_utc` is now `2026-11-09T04:28:17Z`, after the final measurements.
- No executable, manifest, evidence report or accepted checkpoint result
  changed.

## Measurements

- Downloaded archives: 178,013,963 bytes.
- Extracted checkpoint tree: 378,392,745 bytes.
- Downloaded store metadata: 18,191 bytes.
- Complete temporary input footprint: 556,424,899 bytes.

## Checks run at the head

- README behavior checked against the archive-verification and CLI error paths.
- Ledger arithmetic: archive + extracted + metadata equals 556,424,899.
- JSON syntax, ledger arithmetic and `git diff --check`: passed.
- `loop report --check` and `loop pr-check --local`: passed.
- `loop gates --full`: all 11 gates green on the clean rerun.

## Skills used

No system-provided Codex skill applied.

## Still open

The third fresh exact-head review approved all code, evidence and reading
repairs. It found only that the ledger timestamp and task-byte total predated
the final documentation commit; [deepseek-250](../deepseek-250/REPORT.md)
re-measures them.

## Plain summary

This final record repair makes the documented archive behavior exact, accounts
for every byte downloaded into the run root and moves the ledger timestamp
after its measurements.
