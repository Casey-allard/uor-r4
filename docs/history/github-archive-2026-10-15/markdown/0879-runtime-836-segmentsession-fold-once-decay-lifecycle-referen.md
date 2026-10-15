# #879 — runtime(#836): SegmentSession fold-once+decay lifecycle, reference-equivalent (3b foundation)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-20T21:38:58Z
- **Updated:** 2026-08-21T01:56:22Z
- **Closed:** 2026-08-20T22:00:52Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/879
- **Merged:** 2026-08-20T22:00:51Z
- **Merge commit:** 0d4b5e6c2563cd1410aef21fe6c47044256057fc
- **Base / Head:** main / UOR-Foundation:issue-836-pstate-engine
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Problem and outcome

Increment 3b-foundation of #836. The session-stateful channel Casey selected: `SegmentSession<CAP>` in `uor-r4-graph-runtime::runtime_state` wraps `SegmentRing` (#878) in the #835 reference lifecycle — the initial fold Φ₀ records the **whole prompt** into the ring (no decay), then each generation transition decays the lane; a candidate present in the ring receives the descriptor `boost`.

This is the primitive the R4Engine wiring consumes: the caller (engine) constructs an active session from the artifact's PSTATE descriptor, primes it once with the full prompt, and reads contributions per step. An **inactive** session (no PSTATE) makes every fold a no-op and every contribution zero → absent-section identity.

## Scope

- `crates/uor-r4-graph-runtime/src/runtime_state.rs` — `SegmentSession<CAP>` (`inactive`/`active`, `fold_prompt`, `fold_token`, `step`, `contribution`, `reset`, `is_active`, `live_slots`) + two tests.

## Verification (all green)

- `segment_session_matches_reference_lifecycle` — differential vs a `Vec` oracle running the identical fold-then-decay lifecycle: contributions match **immediately after Φ₀ and through 24 generation steps**, for decay shifts {0,1,2}, across the candidate space.
- `inactive_segment_session_is_identity` — an inactive session is inert: folds no-op, ring empty, every contribution zero.
- `cargo test -p uor-r4-graph-runtime`, `clippy -D warnings`, `xtask validate` (every gate incl. the P-4 forbidden-construct scan). `forbid(unsafe_code)` preserved; no_std, allocation-free.

## Compatibility / conformance

Additive, dormant. `CONFORMANCE.md` unchanged — no serving capability claimed (the capability + `model/ids.toml` row → Gherkin → CONFORMANCE land when the scorer is wired onto the request path and the causal gate re-clears). No P-4 / allocation / no_std obligation weakened.

## Closure

**References #836** — the session-stateful lifecycle primitive. Next: wire it into `R4Engine` (read the PSTATE descriptor at load, caller-owned session primed with the full prompt, segment-adjusted candidate scoring, absent-section identity) with witness attribution, then the causal-gate re-run → PROMOTE/REVISE/RETIRE.

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

