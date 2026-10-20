# #880 — runtime(#836): wire the segment lane into the deployed R4Engine (gated, identity-safe)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-20T22:34:39Z
- **Updated:** 2026-08-21T01:56:22Z
- **Closed:** 2026-08-20T22:56:31Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/880
- **Merged:** 2026-08-20T22:56:31Z
- **Merge commit:** 68f346a613c13b4713cc9b0a895bd228184978da
- **Base / Head:** main / UOR-Foundation:issue-836-engine-wire
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Problem and outcome

Increment 3b of #836 — the #835 segment lane reaches the **real serving path** (`uor-r4-api::engine::R4Engine`), behind the PSTATE gate, with **absent-section identity** as the load-bearing safety property. This is the increment the earlier dormant primitives (#876/#877/#878/#879) were built for.

## What changed

- **`R4Engine::load`** reads the optional PSTATE segment-lane descriptor (`decay_shift`, `base_w`, `boost`) exactly as it reads other optional sections. Absent PSTATE — **every artifact today** — yields `None`, and the scorer is byte-identical to before. A descriptor whose ring capacity exceeds the engine's fixed caller-owned state **fails safe** to `None` (the lane stays inert) rather than truncating the reference semantics.
- **`R4Engine::segment_session()`** builds the caller-owned `SegmentSession` — active from the descriptor, inactive when absent. Per the session-stateful decision, the caller primes it once with the whole prompt (`fold_prompt`) and decays per step, so whole-prompt content reaches the scorer **without changing the bounded-window prediction interface**.
- **`R4Engine::predict_decision_candidates_with_segment(window, candidates, session)`** — byte-identical to the base decision when the session is inactive; when active, the served token becomes the **segment-adjusted argmax** over the decided candidate list (each candidate gains the decode-independent `boost` when present in the content ring, same canonical tie-break). An **abstention is never overridden** into a served token; the served token is always one of the decided candidates, **never fabricated**.
- **`segment_adjusted_token()`** — the pure, allocation-free, **P-4** re-rank (saturating add + compare only).

## Non-goals (this increment)

- Witness attribution for the segment-promoted token, and the causal-gate re-run / **PROMOTE** verdict (increment 4). **No quality claim is merged with the implementation.**
- **No `model/ids.toml` capability row yet**: the lane is inert on every existing artifact (dormant); the serving capability + regenerated CONFORMANCE land with the causal re-clear.

## Verification (all green)

- **Unit** (`engine.rs`): `segment_adjusted_token` promotes a boosted candidate over a higher-scored one, keeps the base winner when inactive, and tie-breaks by lower id.
- **Integration** (`tests/pstate_engine_836.rs`, the real synthetic engine): a no-PSTATE artifact yields an **inactive** session; an inactive session is **byte-identical** to the base decision across windows; an active session **runs on the real engine** and preserves the serving invariants (abstention never overridden, served token never fabricated).
- `cargo fmt --check`; `cargo clippy --workspace --all-targets --all-features -D warnings`; `cargo run -p xtask -- validate` — **every gate incl. the P-4 forbidden-construct scan and register/conformance**; `CONFORMANCE.md` unchanged. `forbid(unsafe_code)` preserved.

## Compatibility

`crates/uor-r4-api/Cargo.toml`: `uor-r4-graph-runtime` moved from dev- to normal dependency (the deployed engine now consumes `SegmentSession`). Additive and dormant — no existing serving behavior changes, because no artifact carries a PSTATE section.

## Closure

**References #836** — the deployed segment-lane scorer, gated and identity-safe. Next (increment 4): witness attribution + the frozen causal benchmark on the packed production path → `PROMOTE` / `REVISE` / `RETIRE` verdict with ledger and claim updates.

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

