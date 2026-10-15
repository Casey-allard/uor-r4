# #101 — test(scorer): pin honest interaction-residuals evaluation harness (#69)

- **Type:** Pull request
- **State:** closed
- **Author:** @auser
- **Created:** 2026-07-23T15:34:44Z
- **Updated:** 2026-10-09T14:17:16Z
- **Closed:** 2026-07-23T15:45:30Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/101
- **Merged:** 2026-07-23T15:45:30Z
- **Merge commit:** f989575375250108127bdeead3111f9a7e2d2b16
- **Base / Head:** main / UOR-Foundation:issue-69-evaluation-harness
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Refs #69

## Why this harness exists

PR #99 closed the #69 evaluation with a "0.0000 bits, 0.0000% agreement" zero-delta result — but review commit `662865f` (unmerged) flags the metric behind it as **fake**: the scored residual was never honestly wired into the candidate distribution, so the zero deltas proved nothing either way.

This PR lands the honest measurement as a pinned, reproducible test: `crates/uor-r4-core/tests/interaction_residuals.rs`. It scores the 16 strongest co-activation region pairs (from E_o neighbor edges, ranked by joint-observation mass) with quantized log-probability interaction residuals ΔI(pair,v) = ln P(v|pair) − ln P(v|p1) − ln P(v|p2) + ln P(v|parent), side-by-side against the chain-telescoped baseline on the D3 held-out split.

## Verdict: EVALUATED, NOT JUSTIFIED

All 16 candidate pairs make held-out scoring **worse**:

- **Rule 1 (precedence):** +0.64 … +2.25 bits/token worse across candidates.
- **Rule 1+2 (compiler):** bits worse by +0.02 … +0.29 **and** top-1 agreement drops up to −0.53pp — breaking the EXCT argmax-identity the deployed gate relies on (#68).
- Decision threshold (issue #69 DoD): ≥0.05 bits/token improvement without agreement loss. **No candidate clears it.**

Anchors reproduced and asserted by the test:

| Metric | Value |
| :--- | :--- |
| Artifact κ | `blake3:de04eec8be0ce001c1493acee1b28f83976a74f85519855e1f23e8676d713704` |
| Rule 1 agreement / bits | 1.2185% / 17.7299 |
| Rule 1+2 agreement / bits | 31.7086% / 9.8612 |
| TLA3 baseline agreement / bits | 31.71% / 11.8781 |

## Interpretation

The interaction residual double-counts information the EXCT refinement chain already carries: conditioning on a region *pair* re-attributes probability mass that chain telescoping (#64) has already assigned, so the candidate distribution moves *away* from the teacher. Explicit overlap nodes are rejected for artifact storage on this distribution — a measured negative result.

## Note on main's current state

Main currently carries #99's evaluation with the metric flagged as fake in the unmerged review commit `662865f` (which also removes #80's scoring variants). This harness supersedes that measurement as the pinned provenance for the #69 rejection. Whether to land `662865f`'s revert of the in-tree metric/variants is a separate cleanup question, tracked in a follow-up issue.

Run: `cargo test -p uor-r4-core --release --test interaction_residuals -- --ignored --nocapture` (~19s; requires `/tmp/ref/out/model.bin`).


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

