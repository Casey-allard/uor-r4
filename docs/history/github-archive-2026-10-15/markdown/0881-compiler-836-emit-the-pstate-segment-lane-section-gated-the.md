# #881 — compiler(#836): emit the PSTATE segment-lane section (gated) — the lane goes live

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-20T23:18:42Z
- **Updated:** 2026-08-21T01:56:22Z
- **Closed:** 2026-08-20T23:40:12Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/881
- **Merged:** 2026-08-20T23:40:12Z
- **Merge commit:** 632e43d120de848de4bf8c7f68bb2ff093e6f228
- **Base / Head:** main / UOR-Foundation:issue-836-compile-pstate
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Problem and outcome

Increment 4a of #836 — the enabler that makes the deployed segment lane **non-inert**. #880 wired the lane into `R4Engine`, but nothing produced a PSTATE-bearing bundle, so no real request was ever re-ranked. This teaches the compiler to emit the section.

## What changed

- **`convert_r4g1::convert_with_segment_lane(...)`** — like `convert()`, plus an optional `PSTATE` section built from a `SegmentLaneDescriptor`. The prompt-derived segment lane ships a **config-only** descriptor (capacity / decay / score constants; empty residual table — there is no learned table). `convert()` is unchanged and delegates to a shared inner fn with no lane, so **every existing call site and every no-lane bundle is byte-identical** (absent-section identity), and bytes stay deterministic.

## Verification (all green)

Real compile → load path, in `uor-r4-api/tests/pstate_engine_836.rs`:

- **`compiler_emits_pstate_and_engine_activates_the_lane`** — a bundle compiled with a descriptor carries a round-tripping PSTATE section **and** `R4Engine::segment_session()` is **active** on it. The milestone: the lane is no longer inert on a real compiled bundle.
- **`segment_lane_emission_is_deterministic_and_section_preserving`** — identical inputs → identical bytes; emitting PSTATE leaves `HEAD/NODE/EDGE/ROUT/EMIT/EXCT` byte-identical, and the no-lane bundle carries no PSTATE.
- `cargo fmt --check`; `cargo clippy --workspace --all-targets --all-features -D warnings`; `cargo run -p xtask -- validate` — every gate incl. the **P-4 forbidden-construct scan** and register/conformance; `CONFORMANCE.md` unchanged. `forbid(unsafe_code)` preserved.

## Non-goals

- **No quality claim / no `model/ids.toml` capability row yet.** The descriptor emitted here is *representative*; the **final selected descriptor** and the **PROMOTE / REVISE / RETIRE** verdict come from the causal re-run on the packed production path (increment 4c). No CLI flag yet — the causal harness calls `convert_with_segment_lane` directly.

## Closure

**References #836** — the compiler now emits the lane. Next: witness attribution for the segment-promoted token (4b), then the frozen causal benchmark on the packed path + the verdict, capability row, and CONFORMANCE regen (4c).

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

