# #877 — format(#836): complete PSTATE descriptor + compiler-packing round-trip (increment 2)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-20T20:37:39Z
- **Updated:** 2026-08-21T01:56:22Z
- **Closed:** 2026-08-20T21:14:38Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/877
- **Merged:** 2026-08-20T21:14:38Z
- **Merge commit:** 45ac56289e4583a99342e978fdb51e432aac3d5b
- **Base / Head:** main / UOR-Foundation:issue-836-pstate-packing
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Problem and outcome

Increment 2 of #836. #876 landed the optional `PSTATE` R4G1 section; two things follow here:

1. **Complete the descriptor.** The #835 segment lane is *prompt-derived* (fold prompt content into a bounded decaying ring; boost present candidates), so what a compiler packs is the lane **descriptor**, not a learned table. This extends the PSTATE header (24 → 40 bytes) with the fields the deployed R4Engine needs to reconstruct the lane with caller-owned fixed-capacity state — `ring_capacity`, `base_w`, `boost` — plus semantic validation for them (`capacity >= 1`, `decay_shift < 32`) with planted zero-capacity parse teeth. This is safe format completion: **no shipped artifact writes PSTATE yet** (the section merged minutes earlier and nothing produces it), so finishing the never-yet-written format before first use is a no-op for every existing bundle.

2. **Prove compiler packing end-to-end.** The "compiler packing and a borrowed/zero-copy view" scope item, exercised on the real artifact container: a `SegmentLaneDescriptor` packs into a PSTATE section, survives a full `ArtifactBuilder` → `GraphView` round-trip, and reads back bit-for-bit through `GraphView::pstate_table`.

## Scope

- `crates/uor-r4-graph-format/src/pstate.rs` — 40-byte descriptor header; `SegmentLaneDescriptor` builder input; `ring_capacity()` / `base_w()` / `boost()` accessors; semantic descriptor validation.
- `crates/uor-r4-graph-format/tests/stage1.rs` — two artifact-level tests (below).

## Tests and verification (all green)

- `pstate_section_round_trips_through_artifact` — pack descriptor + residual table, attach via `add_section`, `GraphView::parse`, assert the descriptor (capacity, decay, base_w, boost, lane_kind) and rows round-trip; assert **absent-section identity** (dropping PSTATE leaves HEAD/NODE/ROUT/EMIT byte-identical and `pstate_table` returns `None`).
- `pstate_malformed_section_fails_closed` — a corrupt (zero-capacity) descriptor **fails closed** at `pstate_table()` with a typed error while the optional-section container still parses.
- Plus the module's 5 descriptor tests (round-trip/determinism, corrupt magic/version/reserved/offset, invalid-descriptor teeth, non-canonical builder rejection, empty-section validity).
- `cargo test -p uor-r4-graph-format`, no_std ladder (`--no-default-features` +`--features alloc`), `clippy -D warnings`, `fmt --check`, `cargo check --workspace`, wasm target check, `xtask validate` (every gate passed; CONFORMANCE unchanged). `forbid(unsafe_code)` preserved.

## Non-goals (this increment)

- The deployed scorer still does **not** consume PSTATE (the hot-path transition + contributions + witnesses are the next increment, where the serving capability + its `model/ids.toml` row → Gherkin → CONFORMANCE land).
- No quality claim. No serving-behavior change.

## Compatibility and migration

Purely additive and **non-serving**. Absent-section identity is re-proven at the artifact level. The header extension touches a format nothing yet writes, so no historical bundle is affected; `PSTATE_VERSION` stays 1.

## Conformance / claim boundary

No capability added; `CONFORMANCE.md` unchanged. Binds existing RF-21/22. No claim change.

## Closure

**References #836** — increment 2 (complete descriptor + compiler-packing round-trip). Next: the normative R4Engine hot-path scorer + witnesses (P-4/no_std/allocation), then the causal-gate re-run on the packed path with the PROMOTE/REVISE/RETIRE verdict.

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

