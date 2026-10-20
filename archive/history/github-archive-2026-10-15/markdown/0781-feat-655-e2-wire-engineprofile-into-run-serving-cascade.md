# #781 — feat(#655-E2): wire EngineProfile into run_serving_cascade

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-18T04:19:44Z
- **Updated:** 2026-08-18T04:40:15Z
- **Closed:** 2026-08-18T04:39:30Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/781
- **Merged:** 2026-08-18T04:39:30Z
- **Merge commit:** cd6bce6d5d66b04249e62d2f73585d17489973be
- **Base / Head:** main / UOR-Foundation:feat/655-e2-profile-enforcement
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Summary

Implements #655-E2: wires the `EngineProfile` type (added dormant in
#655-E1, PR #780) into `run_serving_cascade` so the `Production` profile
designed in #655-E0 (docs/serving_engine_profiles_655_e.md) actually
restricts tier reachability. Casey explicitly confirmed this specific
slice should proceed (it changes default runtime behavior, unlike E0/E1
which were dormant/additive).

- `run_serving_cascade` gains a `profile: EngineProfile` parameter.
  Under `Production`, only `TIER_R4G1` is ever admitted to the tier
  list, regardless of what `pinned` names. Under `Experimental` the
  function is byte-for-byte unchanged from before this PR.
- New pure helpers (`profile_restricts_to_r4g1`, `tier_admitted`,
  `tier_for_engine_name`) carry the actual admission logic and are
  directly unit tested, without needing to construct a real serving/
  router/tless triple.
- **Persisted (non-explicit) preference for a disallowed tier under
  `production`: silently inert** — r4g1 still runs normally.
- **Explicit per-request `engine` naming a disallowed tier under
  `production`: typed decline**, not a silent r4g1 substitution. Both
  HTTP call sites (`generate_serving_completion` — `/v1/chat/completions`
  and `/v1/responses` — and the native `/api/chat` handler) check the
  raw request field before `resolve_pinned_tier`'s persisted-fallback
  collapses the distinction, and short-circuit to
  `production_profile_decline`, reusing the existing `declined_by_all`
  JSON contract (no new terminal state).
- Removed E1's now-unnecessary `#[allow(dead_code)]` annotations.

These are exactly the three decisions #655-E0 already recorded (see the
doc's "Enforcement point" / "explicit vs persisted" sections) — this PR
implements them, it doesn't make new product calls.

## Test plan

- [x] `cargo fmt` — clean
- [x] `cargo clippy -p uor-r4-wasm-router --all-targets --all-features -- -D warnings` — clean
- [x] `cargo test -p uor-r4-wasm-router --lib -- server::` — 187 passed (5 new), 0 failed
- [x] New tests: `tier_for_engine_name_recognizes_exactly_the_pinnable_tiers`, `profile_restricts_to_r4g1_true_only_for_production`, `tier_admitted_restricts_every_tier_but_r4g1_under_production` (proves the silently-inert persisted-preference case), `tier_admitted_matches_pre_e2_include_closure_under_experimental` (proves zero behavior change for experimental), `production_profile_decline_names_the_disabled_tier_and_declines`
- [x] `cargo build -p uor-r4-wasm-router --lib --target wasm32-unknown-unknown` — clean
- [x] `cargo test --workspace` — full suite green
- [x] R1-R6 register conformance

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

