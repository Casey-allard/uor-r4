# #610 — perf(cover-sweep): cache cover-independent scoring preparation across points

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @auser
- **Created:** 2026-08-10T22:45:17Z
- **Updated:** 2026-08-13T12:50:51Z
- **Closed:** 2026-08-13T12:50:51Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/610
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Context

`run_point` currently rebuilds several values for every cover configuration even though they depend only on the corpus, teacher artifact, store, and scorer configuration—not on the induced cover.

Likely reusable work includes:

- context rows
- forward-anchor rows
- corpus code derivations and prepared assignment tables
- other Gate C corpus-side tables that are invariant across cover points

The nine-point sweep therefore pays repeated whole-corpus preparation costs.

## Proposal

Introduce a prepared, immutable scoring context in `SweepInputs` or a dedicated type. Build invariant data once, then pass references into each point's cover-dependent compilation and evaluation.

Keep cover-dependent work—induction, regions, transitions, emissions, and artifact emission—per point.

## Acceptance criteria

- Shared preparation runs once per sweep, not once per point.
- Results and emitted artifact bytes remain identical to the current implementation.
- κ/configuration validation prevents reuse across incompatible inputs.
- Memory ownership is explicit and bounded; no per-point mutation can corrupt shared state.
- A regression test proves schema/report metric parity on the fixture.


## Comments (0)

_No comments._

---
_Comment/review ids in this file: _

