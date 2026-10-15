# #248 — Honest cascade: typed abstention for non-graph tiers, unified endpoint semantics, functional /engine selection

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-07-28T03:52:59Z
- **Updated:** 2026-07-28T18:48:01Z
- **Closed:** 2026-07-28T18:48:01Z
- **Labels:** bug
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/248
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

### Summary
Three related honesty defects in the serving cascade, found in the architecture audit:

1. **Only R4G1 can abstain.** The D4 policy (`uor-r4-api/src/engine.rs`: `Novel` → widen-once → `Abstain`) is a real prospective gate — but once the cascade falls through to transformerless/teacher/geometric tiers, every tier always emits. The geometric tier's only "I don't know" is the hardcoded string `"Manifold resonance too sparse for synthesis."` (`server.rs:2299-2306`) — a de-facto abstain that isn't typed, logged, or policy-visible. Give the non-graph tiers a typed abstain (resonance-sparsity / validator-based), so "no engine could answer" is a declared outcome rather than the least-bad string.
2. **The two HTTP endpoints disagree on abstain semantics.** `/v1/chat/completions` (server.rs:1785+) cascades past an R4G1 abstain; `/api/chat` (server.rs:2017+) treats abstain as final, with a comment claiming abstention "never falls through" — true for one endpoint only. Pick one semantic, document it, implement it in both.
3. **`/engine` selection is a no-op on the main path.** The CLI persists the choice to `.uor-models/last_engine.txt`, but `/v1/chat/completions` runs the fixed cascade regardless. Honor the selection or remove the command.

Also: `FallbackRouter` (`uor-r4-router/src/fallback.rs`, `EngineStatus::{Success,UnmappedRegion,Pathological,Failed}`) is a clean abstraction exercised only by its own tests while `server.rs` reimplements the cascade inline twice. Adopt it as the single cascade implementation (both endpoints call it) — do not delete it.

### DoD
- One documented cascade semantic, shared by both endpoints (single implementation).
- Every tier can decline; declined-by-all is a typed, logged outcome.
- `/engine` either functional end-to-end or removed from the CLI.


## Comments (1)

### comment 5107996691 — 2026-07-28T18:14:00Z — Casey-allard

Taking this (assigned). Plan, branch `issue-248-honest-cascade`:

1. **Single cascade implementation**: adopt `FallbackRouter` (`uor-r4-router/src/fallback.rs`, currently exercised only by its own tests) as the one place the engine chain lives; both `/api/chat` and `/v1/chat/completions` call it. Typed per-tier outcomes (`Success` / `UnmappedRegion` / `Pathological` / `Failed` / new `Abstained`).
2. **One documented abstain semantic** (decision recorded in the code doc): R4G1 abstention cascades to the next tier *with the abstention recorded in the response audit* — the "declared outcome" property is preserved by reporting, not by refusing to try other engines. (This matches PR #223's intent and makes the two endpoints agree; if review prefers the /api/chat semantic instead, it is a one-line policy flip in one place — that is the point of centralizing.)
3. **Typed abstention for the non-graph tiers**: the geometric tier's "Manifold resonance too sparse for synthesis." string becomes a typed `Abstained` outcome; validator rejections (`is_usable_generated_text`) become typed `Pathological` per tier. Declined-by-all is a first-class terminal outcome with every tier's reason in the audit.
4. **`/engine` honored**: explicit selection pins the cascade to the chosen engine (single-tier cascade + honest failure if it declines) instead of being ignored.
5. Tests: cascade-order unit tests against a mock tier trait; endpoint-semantic agreement test; declined-by-all shape; engine-pinning test.

Runtime kernel untouched; server/router crates only.



---
_Comment/review ids in this file: 5107996691_

