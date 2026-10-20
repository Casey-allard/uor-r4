# #542 — feat(#510): R5 tranche 5b — graph-compiler induction cover ops to total

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-09T12:32:12Z
- **Updated:** 2026-10-09T13:09:01Z
- **Closed:** 2026-08-09T12:50:33Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/542
- **Merged:** 2026-08-09T12:50:33Z
- **Merge commit:** e5ab9a1b870efb72ce33732ee785948616b62840
- **Base / Head:** main / UOR-Foundation:r5/gc-induction
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Second `uor-r4-graph-compiler` tranche. **82 → 80.** Two standalone cover-induction entry points go total; `emit_r4g1` stays `Result` for now (it wraps `routing::synthesize_routing_program`, part of the still-unconverted executor cluster).

## What changed
- `build_observations_with_threads` → `Vec<Observation>` (was `Result<_,String>`). Its only failure was a worker-thread panic, which now propagates as a panic — a panicked worker is a defect, not a reportable limitation (R5), matching the core runtime thread-worker pattern.
- `induce_cover` → `Option<InducedCover>` (None = no train observations).

Callers adapted (signatures ripple across crates; their own surfaces unchanged): graph-compiler `lib.rs` + graph-cli (`lib.rs`, `cover_sweep.rs`) still return `Result<_,String>`, so `induce_cover` callers map `None` via `.ok_or_else(...)?`; `build_observations_with_threads` callers drop `?`/`.expect(...)` (graph-cli + the graph-compiler/graph-certify test suites). `induce_cover` callers using `.expect()/.unwrap()` are unchanged (both work on `Option`).

## Verification
- `cargo build --workspace --all-targets` clean
- `cargo build -p uor-r4-graph-compiler --target wasm32-unknown-unknown` clean (TRAP A — no host I/O / `SourceUnavailable`)
- induction unit + integration tests pass; `clippy -D warnings` clean (graph-compiler + graph-cli); `cargo fmt --check` clean
- `audit-deferral` (R4) passes; `audit-limits` graph-compiler **82 → 80**

🤖 Generated with [Claude Code](https://claude.com/claude-code)

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

