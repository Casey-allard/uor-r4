# #780 — feat(#655-E1): add dormant EngineProfile type + persisted-preference reader

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-18T03:26:05Z
- **Updated:** 2026-08-18T03:48:44Z
- **Closed:** 2026-08-18T03:46:59Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/780
- **Merged:** 2026-08-18T03:46:59Z
- **Merge commit:** 0343d54672c23f0df290f6084e05235be19da1d5
- **Base / Head:** main / UOR-Foundation:feat/655-e1-profile-type
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Summary

Implements #655-E1: the dormant, additive infrastructure for the
Production/Experimental engine-profile split designed in #655-E0
(docs/serving_engine_profiles_655_e.md, merged in #779).

- `EngineProfile` enum (`Production` / `Experimental`) with
  `from_persisted(raw: Option<&str>) -> Self`, which fails safe to
  `Production` for anything other than an exact, trimmed `"experimental"`
  match.
- `engine_profile_preference()`, reading `.uor-models/engine_profile.txt`,
  mirroring the shape of the existing `persisted_engine_preference()`.

Both new items carry `#[allow(dead_code)] // #655-E1: dormant until
#655-E2 wires it into run_serving_cascade`, matching this file's
established per-item dead-code-allow convention. **Nothing calls these
yet** — `run_serving_cascade` and default tier-reachability are
byte-for-byte unchanged by this PR.

Per E0's own design doc, #655-E2 (actually wiring `EngineProfile` into
`run_serving_cascade` so `Production` restricts which tiers are
reachable) is explicitly out of scope here and will not be started
without a separate Casey check-in first, since it changes default
runtime behavior in a way that sits close to #655-F's sign-off-gated
territory.

## Test plan

- [x] `cargo fmt` — clean, no changes needed
- [x] `cargo clippy -p uor-r4-wasm-router --all-targets --all-features -- -D warnings` — clean
- [x] `cargo test -p uor-r4-wasm-router --lib -- server::` — 182 passed, 0 failed
- [x] `cargo test -p uor-r4-wasm-router --lib -- engine_profile` — new test passes
- [x] `cargo build -p uor-r4-wasm-router --lib --target wasm32-unknown-unknown` — clean
- [x] `cargo test --workspace` — full suite green
- [x] R1-R6 register conformance (check-model, repo-conformance, audit-deferral, audit-limits, cargo deny check)

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

