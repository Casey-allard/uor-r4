# #612 — perf(cover-sweep): run independent points with bounded deterministic parallelism

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @auser
- **Created:** 2026-08-10T22:45:26Z
- **Updated:** 2026-08-13T14:35:30Z
- **Closed:** 2026-08-13T14:35:30Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/612
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Context

The nine sweep points are independent, but `run_sweep` evaluates them serially. This leaves multicore CPU capacity unused during cover induction, compilation, and scoring.

Unbounded parallelism is unsafe because each point can hold large intermediate tables and Gate C already uses Rayon internally.

## Proposal

Evaluate points concurrently with a small configurable worker limit, then collect results in canonical grid order. Avoid nested Rayon oversubscription and make the memory tradeoff visible.

Possible controls:

- a `cover-sweep --jobs` option, or
- a dedicated environment variable with a documented default

## Acceptance criteria

- Two or more points can run concurrently when configured.
- Results, report ordering, and artifact bytes are identical to serial execution.
- Peak memory is bounded by the configured worker count.
- Nested thread-pool oversubscription is avoided or documented.
- A benchmark records speedup and peak RSS at 1, 2, and a host-appropriate worker count.
- Serial mode remains available for reproducibility/debugging.


## Comments (0)

_No comments._

---
_Comment/review ids in this file: _

