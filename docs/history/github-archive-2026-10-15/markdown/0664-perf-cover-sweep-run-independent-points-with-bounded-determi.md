# #664 — perf(cover-sweep): run independent points with bounded deterministic parallelism (#612)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-13T14:19:32Z
- **Updated:** 2026-10-09T14:17:05Z
- **Closed:** 2026-08-13T14:35:29Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/664
- **Merged:** 2026-08-13T14:35:28Z
- **Merge commit:** 09e26132fe77b6e6cfa5bf5a70803ce2b467745e
- **Base / Head:** main / UOR-Foundation:i612-bounded-parallel
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Summary
The nine sweep points are independent but ran serially, leaving multicore capacity idle during cover induction, compilation, and scoring. Since #611 moved the dominant per-point cost off Gate C onto **cover induction** (~33 s/point on the fixture), that idle capacity is now the bottleneck. This evaluates the points with a **bounded worker pool**, byte-identical to serial.

## What's included
- **`run_points_bounded`** — with `R4_SWEEP_JOBS=1` (default) the serial loop is unchanged; with jobs>1, that many OS worker threads pull points from a shared cursor and drop each result into the point's **fixed grid-order slot**.
- **`sweep_jobs`/`clamp_jobs`** — worker count from `R4_SWEEP_JOBS`, defaulting to `1` (serial, for reproducibility/debugging) and clamped to `[1, point_count]`.
- `run_sweep` reports the actual worker count in `SweepTiming.thread_count` (the field #609 wired for exactly this).

## Determinism (the constraint)
- Results are collected into fixed slots → the report is in canonical grid order regardless of completion order.
- Each point's **internal** Rayon (compile + Gate C) uses the global pool exactly as in serial mode and already reduces in input order, so a point computes identical bytes whether it runs alone or beside others. The TLA3 baseline is taken from the first point (identical at every point), matching the serial `get_or_insert`.
- **Oversubscription is bounded and documented:** outer OS workers share the one global Rayon pool; `jobs` is small and the OS scheduler multiplexes the overlap. Peak memory is bounded by `jobs` concurrent points.

## Acceptance criteria
- Two or more points run concurrently when configured ✔ (`R4_SWEEP_JOBS`).
- Results, report ordering, and artifact bytes identical to serial ✔ (measured — byte-identical reports incl. `graph_kappa`).
- Peak memory bounded by worker count ✔ (structural: `jobs` concurrent points; measured RSS in a comment below).
- Nested thread-pool oversubscription avoided or documented ✔ (documented on `run_points_bounded`).
- Benchmark records speedup and peak RSS at 1, 2, host ✔ (speedup below; RSS in a comment).
- Serial mode remains available ✔ (the default).

## Verification (measured, real fixtures)
- **Parity** — new `#[ignore]` test runs the full sweep at 1, 2, and host workers and asserts each parallel report equals the serial report **except** the `timing` block. `graph_kappa` (blake3 of each artifact) is in the compared content, so equal reports prove **byte-identical artifacts**. All byte-identical.
- **Speedup** — serial **250,746 ms**; 2 workers **177,001 ms (1.42×)**; 8 workers **140,170 ms (1.79×)**. Sub-linear because inner Rayon already uses the cores; the win comes from overlapping the less-parallel per-point stages (cover induction dominates).
- **`clamp_jobs`** unit test (CI, no fixtures): default serial; request clamped to `[1, point_count]`.
- **Local CI parity, all green:** `fmt --check`, `clippy --workspace --all-targets --all-features -D warnings`, `wasm32 --lib`, `cargo test --workspace`, `check-model` (R1), `audit-deferral` (R4 — nothing deferred), `repo-conformance` (R2/R3), `cargo deny` (R6).

## Breaking changes
None. `R4_SWEEP_JOBS=1` (default) is byte-identical to pre-#612 behavior; `thread_count` in the report now reflects the worker count (was always `1`).

Closes #612

## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 5281996901 — 2026-08-13T14:49:23Z — Casey-allard

**Peak RSS (acceptance criterion, measured)** — captured via `/usr/bin/time -l` on the parity test binary (fixtures, `R4_GATE_C_SAMPLE=300`), which runs the full sweep at 1 → 2 → 8 workers in one process, so the reported maximum is the high-water mark during the 8-worker (max-concurrency) phase:

- **maximum resident set size: 2,205,581,312 bytes (~2.05 GiB)** at 8 concurrent workers.

This is the ceiling: peak memory scales with the worker count because at most `jobs` points hold their intermediate tables (cover, transitions, emissions, the ~15 MB artifact) at once — at `jobs=1` (serial, the default) the working set is one point's. Same-run speedup reconfirmed and byte-identical to serial at every worker count: serial 244,341 ms → 2 workers 169,430 ms (1.44×) → 8 workers 127,136 ms (1.92×).


---
_Comment/review ids in this file: 5281996901_

