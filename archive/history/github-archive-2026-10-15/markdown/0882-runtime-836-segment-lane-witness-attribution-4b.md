# #882 — runtime(#836): segment-lane witness attribution (4b)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-20T23:31:53Z
- **Updated:** 2026-08-21T01:56:23Z
- **Closed:** 2026-08-21T00:39:10Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/882
- **Merged:** 2026-08-21T00:39:10Z
- **Merge commit:** d61d1f22e3107d1c926a3899b95ed01e42411e2a
- **Base / Head:** main / UOR-Foundation:issue-836-witness
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Problem and outcome

Increment 4b of #836 — witness attribution, so a segment-promoted served token is **provable**. Extends the deployed witness data model and the deployed segment path; no quality claim (the verdict is 4c).

## What changed

- **`InferenceWitness`** gains `segment_lane: Option<SegmentLaneWitness>` with `#[serde(default, skip_serializing_if = "Option::is_none")]`. Absent for every artifact without a PSTATE section, so **pre-#836 witnesses serialize/deserialize unchanged** — the witness-level absent-section identity. `SegmentLaneWitness` records the **promoted token**, the **base-scorer token** it displaced, and the raw **`ScoreQ` boost**.
- **`R4Engine::predict_decision_candidates_with_segment_witness(...)`** — the deployed segment path, additionally returning `Some(attribution)` **exactly when** the lane promoted a different served token; `None` otherwise (inactive lane, abstention, or no change). The served token and the attribution's `promoted_token` agree by construction.
- **`segment_lane_attribution()`** — pure helper (reuses `segment_adjusted_token`).

## Verification (all green)

- `segment_lane_attribution_records_only_real_promotions` — inactive lane / unchanged winner → `None`; a real promotion → attribution with the right promoted/base tokens and boost.
- `inference_witness_segment_lane_serde_is_backward_compatible` — a witness carrying `segment_lane` round-trips, **and a pre-#836 witness JSON (no `segment_lane`) still deserializes → `None`**.
- `segment_witness_method_is_identity_when_inactive` — on the real engine, the witness method returns the base decision and `None` attribution when inactive.
- `cargo fmt --check`; `cargo clippy --workspace --all-targets --all-features -D warnings`; `cargo run -p xtask -- validate` (every gate incl. the **P-4 scan** + register/conformance; `CONFORMANCE.md` unchanged). `forbid(unsafe_code)` preserved.

## Closure

**References #836** — witnesses now attribute the segment promotion. Next (4c, the finale): compile a real teacher-grounded bundle with the lane, run the frozen causal benchmark on the packed path, and record the **PROMOTE / REVISE / RETIRE** verdict + `model/ids.toml` capability row → Gherkin → CONFORMANCE regen. A run contract precedes that hours-long run.

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

