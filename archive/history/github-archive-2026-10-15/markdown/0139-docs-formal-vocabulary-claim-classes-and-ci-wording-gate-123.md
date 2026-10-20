# #139 — docs: formal vocabulary, claim classes, and CI wording gate (#123)

- **Type:** Pull request
- **State:** closed
- **Author:** @auser
- **Created:** 2026-07-24T05:55:12Z
- **Updated:** 2026-10-09T14:17:10Z
- **Closed:** 2026-07-24T06:05:48Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/139
- **Merged:** 2026-07-24T06:05:48Z
- **Merge commit:** 624a1cfeaab3e9a78e7c724608d6ad8fe0eb1564
- **Base / Head:** main / UOR-Foundation:issue-123-formal-vocabulary
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Closes #123. Parent tracker: #122. Source: PDF §§1, 7, 13.

## What lands

- **`docs/formal_vocabulary.md` v0.1.0** — the normative mathematical vocabulary:
  - Claim classes: every equation/statement is labeled **Definition**, **Objective**, **Guarantee**, **Assumption**, or **Empirical Criterion**.
  - Claim statuses: **Structural / Witnessed / Empirical / Assumed / Unproven**, mapped normatively to the proof-model `ProofStatus` enum.
  - Core notation with Rust bindings: `S`, `G=(V,E)`, `H(x)`, `T: S × A → S`, `R`, `C: Θ → G`, `P_θ`, `P_G`, `D(·,·)`/`ε` — each mapped to a concrete module or marked compiler/reference-only with the owning issue (#124/#126/#127/#129/#131).
  - Objectives vs. runtime invariants kept on separate tables; guarantees cite their proof-matrix evidence and status.
  - Term discipline: "intent" → future-state optimization; "semantic atom" prohibited (semantic region); "equivalence" only qualified (byte reproducibility vs. empirical behavioral equivalence); "reasoning" informal, replaced by typed state transitions / graph navigation / bounded planning.
- **`scripts/check_claim_wording.py`** — CI-checkable wording rule: blocks "machine-verified" / "machine verified" / "exact equivalence" / "exact teacher equivalence" / "provably equivalent" in `docs/**` and crate READMEs unless the line links a proof artifact/certificate or carries a same-line disavowal. Wired into the `gates` job in `.github/workflows/ci.yml`.
- Cross-links from `docs/r4_graph_compiler_implementation_plan.md` (Phase 0) and `crates/uor-r4-proof-model/README.md`; convention recorded in `AGENTS.md`.

## Acceptance criteria

- [x] Versioned spec under `docs/`, cross-linked from the implementation plan and proof-model README.
- [x] Every equation labeled Definition / Objective / Guarantee / Assumption / Empirical Criterion.
- [x] Notation maps to concrete Rust modules or is marked compiler/reference-only.
- [x] CI-checkable rule prevents machine-verified / exact-equivalence wording without a linked proof artifact or certificate.

## Local gates

- `cargo test --workspace --offline` — 58 suites ok
- `cargo clippy --workspace --all-targets --all-features --offline -- -D warnings` — clean
- `cargo fmt --check` — clean
- no_std ladder (both `uor-r4-graph-format` checks) — clean
- `python3 scripts/check_claim_wording.py` — passes

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

