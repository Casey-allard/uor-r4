# #373 — feat(cli): #364 track B — revive the cover sweep (orphaned dead code) with emission-shrinkage axes

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-03T14:43:13Z
- **Updated:** 2026-10-09T13:09:11Z
- **Closed:** 2026-08-03T15:03:09Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/373
- **Merged:** 2026-08-03T15:03:08Z
- **Merge commit:** c1fa2fcff99e43a095ba169cc48af3ecb311c093
- **Base / Head:** main / UOR-Foundation:track-b-sweep-shrinkage
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## #364 Track B: revive the cover sweep — it was orphaned dead code — and add the shrinkage axes

**Discovery:** `crates/uor-r4-core/src/transformerless/cover_sweep.rs` was **orphaned** — no `mod` declaration anywhere in the crate, and its `use super::score` / `super::cover` imports cannot resolve in `uor-r4-core` since the crate restructuring moved those modules into `uor-r4-graph-certify` / `uor-r4-graph-compiler`. The "cover fineness sweep harness" (#70) has been dead code on main: nothing compiled it, its tests never ran, and #361/#362-era edits kept its text current without it ever building. `trace_lane.rs` in the same directory is orphaned the same way — left untouched here, flagged for a maintainer decision (delete, or revive it the same way).

**The move:** relocated to `uor-r4-graph-cli`, whose existing aliases (`uor_r4_graph_certify as score`, `induction as cover`, core's `compiler`/`runtime`) are exactly the names the module already uses. Wired as the `cover-sweep` subcommand. One real signature drift fixed (`build_edges` now takes the story map). The module's own tests compile and pass **for the first time**.

**The #364 axes (sweep schema 1 → 2):** `--emission-selection` / `--emission-shrinkage` flow into the swept `ScoreConfig`; the report's scorer block records them plus `context_order` / `context_entries` (#372's knobs). Default invocation reproduces schema-1 rows exactly, modulo the added config fields.

**Why:** the #364 verdict — contrast weighting reaches parity, cover specialization (1.41×) is the ceiling — rests on measurements at the default 42-region cover only. Granularity × contrast-shrinkage has never been measured; this makes that a one-command fixture-scale run, attributable end-to-end with #371's report fields.

Gates from cleaned targets, real exit codes: fmt, clippy `--all-targets --all-features`, full workspace tests.

Refs #364, #362, #70. Third PR of the dual-track series; the sweep run + trigram A/B measurements follow on the fixture corpus.

🤖 Generated with [Claude Code](https://claude.com/claude-code)

https://claude.ai/code/session_018UJdEQFdckSK2et9G44jsm


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

