# #660 — perf(cover-sweep): attribute wall time across the 9-point graph compile (#609)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-13T11:57:25Z
- **Updated:** 2026-10-09T14:17:05Z
- **Closed:** 2026-08-13T12:17:55Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/660
- **Merged:** 2026-08-13T12:17:55Z
- **Merge commit:** 7f4e43f73320c96f12831581ba2b1d96959a1228
- **Base / Head:** main / UOR-Foundation:i609-sweep-timing
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Summary
Adds a `timing` block to `cover_sweep.json` (schema **3 → 4**) that attributes wall-clock time across the nine-point cover sweep. This identifies the dominant cost stage per point and gives a **sequential baseline** a future parallel sweep can be compared against.

## What's included
- **`PointTiming`** — per-point elapsed ms for the seven pipeline stages (`cover_induction`, `recall_and_edges`, `transitions`, `context_and_forward_rows`, `emissions`, `r4g1_emission`, `gate_c`), the point `total_ms`, and `dominant_stage` (max-wall-time stage; ties break to the **earliest** stage in pipeline order).
- **`SweepTiming`** — run `total_ms`, `thread_count` (`1` — the sweep is sequential today), `gate_c_sample` (records `R4_GATE_C_SAMPLE` so a *sampled* Gate C isn't mistaken for a fast one), observation counts, and the per-point vector.
- `run_point` wraps each existing stage in an `Instant` read; the `?` early-returns are preserved.

## Instrumentation is pure measurement
No artifact bytes and no score values change — each stage is timed by wrapping it in `Instant::now()` / elapsed read, with no reordering or logic change. The schema-3 row fields are byte-identical with and without the timing block.

## Verification (measured)
- `cargo test -p uor-r4-graph-cli cover_sweep`: **4 passed**, incl. two new timing tests — dominant-stage reporting + serialization shape, and the tie-break-to-first-stage rule.
- Local CI parity, all green: `fmt --check`, `clippy --workspace --all-targets --all-features -D warnings`, `wasm32 --lib`, `cargo test --workspace`, `check-model` (R1), `audit-deferral` (R4 — nothing deferred), `repo-conformance` (R2/R3), `cargo deny` (R6).

## Breaking changes
None. Schema consumers pinned to 3 should bump to 4; the existing row fields are unchanged and the block is purely additive.

## Follow-up
None required. `thread_count` is wired so a parallel sweep can populate it without a further schema bump.

Closes #609

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

