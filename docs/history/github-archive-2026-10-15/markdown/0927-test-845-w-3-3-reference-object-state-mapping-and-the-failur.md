# #927 — test(#845): W(3,3) reference object, state mapping, and the failure-surface probe

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-22T19:47:33Z
- **Updated:** 2026-08-22T21:05:05Z
- **Closed:** 2026-08-22T20:09:53Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/927
- **Merged:** 2026-08-22T20:09:53Z
- **Merge commit:** 7133c8831433c1d3a0dcaa50f319ed6e32d11b9e
- **Base / Head:** main / UOR-Foundation:issue-845-mapping-instruments
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Problem and outcome

References #845 (S4 item C, increment 2 of 4 — mapping and instruments).

The frozen design contract (`docs/w33_geometry_qualification_spec_845.md`, PR #926) pins the normative W(3,3) as Definitions and requires the binding failure-surface probe to ship as a repository instrument. This PR delivers both as certifier-instrument test code: the reference object with its invariants machine-checked as tests, the state mapping mu, and the probe with its non-vacuity gates.

## What this PR contains

- `crates/uor-r4-graph-certify/tests/support/w33.rs` (new, shared support module) — the symplectic quadrangle over GF(3) exactly as the spec pins it: the 40 canonical points in lex order, the pinned alternating form, collinearity distance d_W, phase phi, the 40x40 tables, the 40 totally isotropic lines, the state mapping mu (total, content-derived, 9-periodic), the pinned relabel symplectomorphism, and the adversarial phase permutation.
- `crates/uor-r4-graph-certify/tests/w33_mapping_845.rs` (new) — 9 tests asserting the frozen Definitions hold of the constructed object: 40 canonical points; the collinearity graph is SRG(40, 12, 2, 4) checked over every pair; 40 lines, 4 points each, each point on 4 lines, pairwise-collinear; the generalized-quadrangle axiom at every (point, line) pair; table/definition agreement with d_W symmetry, diameter 2, phi antisymmetry, and phi = 0 exactly on collinear-or-equal pairs; mu totality, surjectivity, 9-periodicity, and residue-purity over a signed lattice; readout fibers partitioning the 81 residue classes (3 at the basepoint, 2 elsewhere); the relabel control a non-identity collinearity automorphism; the phase permutation adversarial but metric-safe.
- `crates/uor-r4-graph-certify/tests/geometry_probe_845.rs` (new) — the binding cheap instrument (already run 2026-08-22; its record is in Amendment A2 and the spec §6): the bounded-breadth-first failure surface over five families x H in {1, 2, 4, 8, 12, 16} at the frozen budget plus the frontier/expansion ladder at H = 8, with all four #843 nulls measured alongside and the A2(b) admissibility judge. The full grid is `#[ignore]`d (a measurement, not a gate); two default tests assert the instrument packs-and-plans and that the admissibility judge can fire and can fail.

## Verification

- fmt clean; clippy `--workspace --all-targets --all-features -D warnings` clean; wasm lib build clean.
- `cargo test --workspace --no-fail-fast --offline` green (worktree, teacher-gated suites skip as expected).
- `xtask validate` (R1/R4/R5/gnaf), `cargo deny check bans`, and the claim-wording gate all pass; `audit-limits` re-run immediately before push.
- Test code only in `crates/uor-r4-graph-certify/tests/`; no shipped-crate source changes.

## Conformance, compatibility, claim boundary

Certifier-instrument / off-serving-path additions only; no production format, runtime, or dependency changes; CONFORMANCE.md untouched; no new RF id. These tests verify reference mathematics and instrument non-vacuity — they license no semantic-usefulness, activation, or empirical-superiority claim (those are increments 3–4 under the pre-registered statistics).

## Intentionally excluded

The ordering-arm skeleton, controls, and byte-parity audit (increment 3); all measurements beyond the recorded probe (increment 4).


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

