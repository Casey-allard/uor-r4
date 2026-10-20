# #611 — perf(cover-sweep): add a minimal Gate C evaluator for sweep metrics

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @auser
- **Created:** 2026-08-10T22:45:21Z
- **Updated:** 2026-08-13T13:47:08Z
- **Closed:** 2026-08-13T13:47:08Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/611
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Context

The cover sweep consumes only a small subset of `evaluate_gate_c`:

- Rule 1 reconstruction metrics
- Rule 1+2 agreement
- the TLA3 baseline
- the null metrics required by the selected harness

The full Gate C evaluator also builds and scores multiple experimental arms that are not reported by `cover_sweep`. Those extra arms multiply per-position work for every one of the nine points.

## Proposal

Add a sweep-specific evaluator that computes exactly the metrics required by the sweep report and null-arm harness. Keep the full evaluator unchanged for certification and research reports.

Validate the slim evaluator against the full evaluator on a pinned fixture before using it for production sweeps.

## Acceptance criteria

- The sweep-specific evaluator reports every field currently consumed by `SweepRow`, `SweepReport`, and the null-arm harness.
- On the fixture, required metrics match the full evaluator within the existing deterministic floating-point contract.
- The full Gate C path remains available and unchanged for certification.
- Benchmarks show a material reduction in per-point wall time and allocations.
- The distinction between sampled decision runs and full census runs remains explicit.


## Comments (0)

_No comments._

---
_Comment/review ids in this file: _

