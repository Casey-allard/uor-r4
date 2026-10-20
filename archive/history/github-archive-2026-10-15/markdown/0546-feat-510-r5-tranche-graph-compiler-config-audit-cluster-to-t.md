# #546 — feat(#510): R5 tranche — graph-compiler config/audit cluster to total (56→50)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-09T15:05:17Z
- **Updated:** 2026-10-09T14:17:25Z
- **Closed:** 2026-08-09T15:21:31Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/546
- **Merged:** 2026-08-09T15:21:31Z
- **Merge commit:** 45f5da324fe71bdc2347ed888a3a61e08a2a52b7
- **Base / Head:** main / UOR-Foundation:r5/gc-config
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

R5 rearchitecture (#510): convert the compiler config/audit surface so `Result` names only sanctioned types. The fallible-but-not-error cases become the absence of a product.

## Modules converted
- **jobs_config.rs** — `CompilerJobsConfig::resolve` → `Option<Self>` (None on 0-jobs or unparseable env); `build_dedicated_thread_pool` → `ThreadPool` or **panic** (host-resource construction defect). `JobsConfigError` removed.
- **memory_budget.rs** — `CompilerMemoryBudget::derive` → `Option<Self>` (None below minimum); `InFlightBackpressureLimiter::try_acquire` → `Option<BackpressureGuard>` (None at capacity). `MemoryBudgetError` removed (incl. unused `BudgetExceeded`).
- **dependency_audit.rs** — `audit_lockfile_contents` / `audit_workspace_features` → `Option<usize>` (None on a forbidden crate/feature). Audit error removed.

## Ripple (same PR)
- **proof-model structural_guarantees.rs** — the three `verify_compiler_*` fns use let-Some-else / `.is_none()` instead of `matches!`/`Err` checks.
- **graph-compiler lib.rs** `compile()` — `resolve()?` adapts via `ok_or_else`; the dedicated pool is now infallible.
- **tests/bdd.rs** — World fields to `Option<Option<_>>`; steps assert `.is_none()`.

## Verification
audit-limits graph-compiler **56 → 50**. 86 lib tests, BDD 100/100, proof-model 24 tests, wasm32 + clippy + fmt clean, audit-deferral clean.

Part of the #510 bottom-up sweep (graph-compiler leaf first).

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

