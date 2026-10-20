# #123 — Define the formal vocabulary: definitions, objectives, guarantees, and notation

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @auser
- **Created:** 2026-07-24T05:03:33Z
- **Updated:** 2026-07-24T06:06:50Z
- **Closed:** 2026-07-24T06:05:50Z
- **Labels:** graph-compiler
- **Assignees:** auser
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/123
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Parent: #122
Source: PDF §§1, 7, 13.

Create the repository's normative mathematical vocabulary for Hologram/R4. The current notes mix architectural definitions, compiler optimization objectives, empirical certification claims, and structural runtime guarantees. Separate these classes and define the symbols that recur across the compiler, artifact, certificate, and proof model.

## Scope

- Define `S` (semantic state space), `G=(V,E)` (compiled graph), `H(x)` (overlapping holographic projections), `T:S x A -> S` (typed dynamics), reconstruction/recovery operator `R`, and the teacher/runtime distributions.
- State which quantities are compiler objectives versus runtime invariants.
- Replace overloaded uses of "intent", "semantic atom", "equivalence", and "reasoning" with precise terms or explicitly mark them as informal.
- Add a claim-status convention for structural, witnessed, empirical, assumed, and unproven statements.

## Acceptance criteria

- A versioned spec exists under `docs/` and is cross-linked from the implementation plan and proof-model README.
- Every equation in the new spec is labeled Definition, Objective, Guarantee, Assumption, or Empirical Criterion.
- The notation maps to concrete Rust modules or is explicitly marked as compiler/reference-only.
- A CI-checkable rule prevents "machine-verified" or exact-equivalence wording without a linked proof artifact or certificate.

## Comments (1)

### comment 5066683237 — 2026-07-24T06:06:50Z — auser

Done via PR #139, merge commit 624a1cfeaab3e9a78e7c724608d6ad8fe0eb1564.

DoD evidence:
- Versioned spec `docs/formal_vocabulary.md` v0.1.0, cross-linked from `docs/r4_graph_compiler_implementation_plan.md` (Phase 0) and `crates/uor-r4-proof-model/README.md`.
- Every equation in the spec is labeled Definition / Objective / Guarantee / Assumption / Empirical Criterion (§§1, 3, 4); guarantees additionally carry a claim status mapped to `ProofStatus` (§2).
- Notation (`S`, `G=(V,E)`, `H(x)`, `T`, `R`, `C`, `P_θ`, `P_G`, `D`/`ε`) maps to concrete Rust modules or is marked compiler/reference-only with the owning issue (§3).
- CI-checkable wording rule: `scripts/check_claim_wording.py` runs as a step of the `gates` job and blocks machine-verified / exact-equivalence wording without a linked proof artifact, certificate, or same-line disavowal (§2.1, §6).
- Overloaded terms ("intent", "semantic atom", "equivalence", "reasoning") replaced with precise terms or marked informal (§5).
- Gates: `cargo test --workspace --offline` (58 suites ok), clippy `-D warnings`, `cargo fmt --check`, no_std ladder — all clean locally; all five required CI checks + Kani proofs green on the PR.


---
_Comment/review ids in this file: 5066683237_

