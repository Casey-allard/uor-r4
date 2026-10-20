# #613 — perf(cover-sweep): checkpoint and resume completed points

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @auser
- **Created:** 2026-08-10T22:45:29Z
- **Updated:** 2026-08-13T16:34:10Z
- **Closed:** 2026-08-13T16:34:10Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/613
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Context

A long sweep currently loses all progress if the process, machine, or input environment fails after several points. This makes long-running campaigns expensive even when the per-point implementation is correct.

## Proposal

Persist each completed point's artifact metadata, metrics, and report row atomically, together with a manifest containing:

- input κs
- scorer configuration
- cover configuration
- source/revision identifier
- sampling and skipped-arm settings
- schema version

Add a resume mode that validates the manifest and reuses only compatible completed points.

## Acceptance criteria

- A sweep can resume after interruption without rerunning valid completed points.
- Partial files cannot be mistaken for completed results.
- Any input, scorer, schema, or cover-config mismatch invalidates the affected checkpoint.
- Resumed output is byte-identical to a clean serial run with the same inputs.
- The final report is emitted only after all required points complete.
- The default behavior remains safe for existing callers.


## Comments (0)

_No comments._

---
_Comment/review ids in this file: _

