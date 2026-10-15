# #58 — CI workflow: fmt/clippy/nextest/fuzz/rebuild/κ gates

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @auser
- **Created:** 2026-07-22T05:56:30Z
- **Updated:** 2026-07-22T18:04:17Z
- **Closed:** 2026-07-22T18:04:17Z
- **Labels:** graph-compiler
- **Milestone:** Phase 1: Packed graph format and trusted parser
- **Assignees:** auser
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/58
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Plan §13 and the Phase 1 exit require a real CI workflow; today only the Pages deploy runs, which is why the κ baseline broke twice (stale pin, then platform-sensitive TLA5 pin merged ungated). Build .github/workflows/ci.yml: cargo fmt --check, cargo clippy --workspace --all-targets --all-features -- -D warnings, cargo nextest run --workspace + cargo test --doc, allocation-freedom suite (pattern: tests/allocation_census.rs), kernel source-scan + op census, deterministic rebuild (extends #21), compiler T-invariance (T=1 vs T=4, plan §4.1), fuzz smoke (crates/uor-r4-graph-format/tests/fuzz_smoke.rs), dependency/license audit. Pinned-runner policy per D7: benchmark/κ gates may need the pinned dev machine or a fixed runner image. Plan: docs/r4_graph_compiler_implementation_plan.md §13.

## Comments (1)

### comment 5049671490 — 2026-07-22T18:04:17Z — auser

Done — CI workflow live and green (run 29944762857, all three jobs success).

Landed: .github/workflows/ci.yml per plan §13 — gates job (cargo fmt --check; clippy --workspace --all-targets --all-features -- -D warnings; nextest run --workspace incl. allocation-freedom census, op-census/kernel-scan invariants, deterministic-rebuild; cargo test --doc; graph-format no_std ladder), cargo audit (rustsec/audit-check), and fuzz smoke (nightly libFuzzer, 60s x 2 targets + the stable deterministic smoke in nextest). Path dependencies (uor-addr/UOR-Framework/prism) are checked out from their public upstream repos at pinned revisions recorded in the workflow env.

Notes/deviations: (1) kappa-reproduction runs with the cached stories15M checkpoint but is continue-on-error on Linux — the baseline is macOS-pinned per D2 until the canonical deterministic compile mode lands (Phase 2); it skips/fails-soft by design. (2) T-invariance (T=1 vs T=4) is a no-op until the compiler parallelizes (plan §4.1); the deterministic double-compile in deterministic_rebuild_test covers the current single-threaded case. (3) License checking (cargo deny) not included — cargo audit covers advisories; add deny.toml in a follow-up if wanted. (4) First-run fix: taiki-e/install-action has no cargo-fuzz mapping; installed via cargo install --locked instead (5ef214b).


---
_Comment/review ids in this file: 5049671490_

