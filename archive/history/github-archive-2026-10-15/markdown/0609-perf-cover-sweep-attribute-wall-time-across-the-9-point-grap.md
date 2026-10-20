# #609 — perf(cover-sweep): attribute wall time across the 9-point graph compile

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @auser
- **Created:** 2026-08-10T22:45:15Z
- **Updated:** 2026-08-13T12:17:57Z
- **Closed:** 2026-08-13T12:17:57Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/609
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Context

The cover sweep in `crates/uor-r4-graph-cli/src/cover_sweep.rs` runs nine independent points, but the current progress output only identifies the point and reports its final metrics. Gate C has internal phase timing, while the surrounding work is not attributed.

When a run takes hours or days, we cannot tell whether the dominant cost is cover induction, held-out recall, edge construction, transitions, context rows, emissions, artifact emission, or Gate C.

## Proposal

Add stable per-stage wall-clock instrumentation around each `run_point` stage and include the timing in the console output and/or a machine-readable report.

At minimum measure:

- cover induction
- reference/held-out recall and edge construction
- transitions
- context and forward rows
- emissions
- R4G1 emission
- Gate C

Record the point label, corpus sizes, thread count, `R4_GATE_C_SAMPLE`, and skipped arms alongside the timings.

## Acceptance criteria

- Timing identifies the dominant stage for every sweep point.
- Timing instrumentation does not change artifact bytes or score values.
- Output is usable for comparing sequential and parallel implementations.
- A test covers the report serialization shape.


## Comments (0)

_No comments._

---
_Comment/review ids in this file: _

