# #935 — docs: audit canonical quality baseline (#934)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-25T01:47:02Z
- **Updated:** 2026-10-09T14:17:18Z
- **Closed:** 2026-08-25T02:09:50Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/935
- **Merged:** 2026-08-25T02:09:50Z
- **Merge commit:** c29fa0e003aaa2176d7b4024508540d883490141
- **Base / Head:** main / UOR-Foundation:issue-934-canonical-quality-baseline-audit
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Outcome

- Establishes one authoritative threshold genealogy: the 29.7% fixed floor is scoped to pinned or legacy reports, while the canonical broad profile is relative to its same-report TLA row.
- Binds the canonical 72,130-position census and diagnoses Rule 1+2 at 17,595/72,130 (24.393%) versus TLA at 20,284/72,130 (28.121%), a 2,689-position deficit.
- Adds an append-only research ledger, source-to-admission reachability map, exact candidate/status headroom, and a five-step bounded remediation order.
- Adds a versioned machine-readable diagnostic and fail-closed validator with planted negative controls and a live local-bundle identity check.
- Reconciles README, ROADMAP, RESEARCH, BASELINE, and the authoritative S0 plan. The dependency order remains #934 -> #933 -> #932.

## Current reproduction

The teacher-free #908 reference/off-serving harness was rerun on the bound broad-clean corpus and reproduced its committed result exactly in 128.4 s: base 19,372, skip 21,424, shuffled null 2,281, and result CID `blake3:e32e4e33d70f342ae3c0913ba00d9aef0cf789b539b9e1b658a9366c51402a26`. This remains `R4Engine` reference evidence, not canonical `R4G1Runtime` serving evidence.

## Verification

- `cargo test --workspace --no-fail-fast --offline`
- `cargo clippy --workspace --all-targets --all-features --offline -- -D warnings`
- `cargo fmt --all -- --check`
- both graph-format `no_std` configurations
- focused #934 tests plus the explicit live 360M binding test
- non-vacuous canonical kappa reproduction against `/tmp/ref/out/model.bin`
- wasm32 library check
- claim-wording and repository conformance registers
- `cargo deny check`

## Claim boundary

This is a teacher-free certifier and reachability audit. It does not rerun canonical Gate C, a teacher forward, or observation/corpus generation; each is explicitly recorded as `NOT_RUN`. It changes no production behavior and lowers no gate. #933 owns normative runtime reachability and admission; #932 remains downstream BDD observability and exact-parallel infrastructure.

Closes #934


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

