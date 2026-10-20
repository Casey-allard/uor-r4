# #102 — Main carries #99's fake interaction-residual metric and #80's invariant-breaking variants (unmerged revert 662865f)

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @auser
- **Created:** 2026-07-23T15:36:00Z
- **Updated:** 2026-07-23T16:33:34Z
- **Closed:** 2026-07-23T16:33:01Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/102
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Problem

Main currently carries two pieces of scorer surface that an unmerged review commit flags as incorrect:

1. **The #99 interaction-residuals Gate C metric** (`rule12_interaction_residuals` in `evaluate_gate_c`) — reported "0.0000 bits, 0.0000% agreement" zero deltas because the residual was never honestly wired into the candidate distribution. Review commit `662865f` on the dangling branch `origin/issue-69-interaction-residuals` calls this a *fake metric*.
2. **The #80 scoring variants** (`--scoring-variant` CLI, `ScoringVariant::CloudSizeNormalized` / `MarginWeighted`) — flagged by the same review as invariant-breaking.

Commit `662865f` ("revert(scorer): remove invariant-breaking scoring variants and fake interaction-residual metric (#99 review)") removes both but was never merged.

## Evidence

- #99's zero-delta result is superseded by the honest pinned harness in #101 (`crates/uor-r4-core/tests/interaction_residuals.rs`), which shows all 16 candidate pairs actively *degrade* held-out scoring (Rule 1: +0.64…+2.25 bits/token; Rule 1+2: +0.02…+0.29 bits and up to −0.53pp agreement).
- The in-tree metric remaining in `score.rs` / `score_runtime.rs` misrepresents that result as "no effect".

## Definition of Done

- [ ] Decide: land the `662865f` revert (adapted to current main), or justify keeping the variants with corrected wiring.
- [ ] If reverting: `rule12_interaction_residuals`, `ScoringVariant`, and the `--scoring-variant` CLI removed; `cargo clippy --workspace --all-targets --all-features -- -D warnings` and `cargo test --workspace` green; no Gate C trend-alarm regression.
- [ ] PR merged via merge queue; #80 cross-referenced so its record reflects the revert.

## Acceptance Criteria

- `grep -r "ScoringVariant\|rule12_interaction_residuals" crates/` returns no hits (or the kept surface is documented with an honest measurement).
- The pinned #101 harness still passes unchanged.

Refs #69, #80, #99, #101


## Comments (1)

### comment 5060907181 — 2026-07-23T16:33:32Z — auser

## Done — decision: remove (revert path), PR #114

**DoD:**
- [x] Decided: removal, adapted to current main (the `662865f` revert's first half — fake metric + `--scoring-variant` CLI — had already landed via `610466b` / #100; this closes out the remaining `ScoringVariant` machinery)
- [x] `rule12_interaction_residuals`, `ScoringVariant`, and the `--scoring-variant` CLI: none remain in-tree; `score_variant.rs` deleted, scorer plumbing removed, Gate C report schema 5 → 6
- [x] Gates green: `cargo clippy --workspace --all-targets --all-features -- -D warnings` clean, `cargo fmt --check` clean, `cargo test --workspace --offline` green, Gate C trend alarm ok (rule12 0.3171/9.8612, baseline 11.8781 — unchanged)
- [x] PR via merge queue; #80 cross-referenced

**Acceptance criteria:**
- `grep -r "ScoringVariant\|rule12_interaction_residuals\|score_variant\|cloud_size_normalized\|margin_weighted" crates/ src/ tests/` → no code hits (only the schema-history comment documenting schema 5)
- The pinned #101 harness untouched and still passes
- Deciding evidence: variant rows bit-identical to `rule12_precedence` on the fixture (top-1 0.317086, 9.8612 bits) — structural under ExactContext dominance, so the columns measured nothing at two extra scorer constructions per evaluation

Closing.



---
_Comment/review ids in this file: 5060907181_

