# #186 — feat(compiler): add compiler memory-budget and backpressure model (#169)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-07-25T02:52:21Z
- **Updated:** 2026-10-09T14:17:11Z
- **Closed:** 2026-07-25T03:32:08Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/186
- **Merged:** 2026-07-25T03:32:08Z
- **Merge commit:** 58f209c7680413a6cf574d2fef08e3cc7ef01480
- **Base / Head:** main / UOR-Foundation:issue-169-compiler-memory-budget
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Summary
Implements Issue #169: Compiler memory-budget and backpressure model for multicore compilation.

- **Normative Specification ([](file:///Users/casey.allard/uor-r4/docs/compiler_memory_budget.md))**: Defines concurrency-aware memory budget formula ($\text{PeakRSS} \le \text{Budget}$), per-stage estimates, worker scratch reuse, bounded queue backpressure, typed `BudgetExceeded`/`BudgetTooSmall` errors, and CPU-only memory rules.
- **Memory Budget Module ([)**: Implements `StageMemoryEstimate`, `CompilerMemoryBudget`, `InFlightBackpressureLimiter`, and `MemoryBudgetError`.
- **Stage DAG Extension ([)**: Attaches `memory_estimate()` to `StageNode`.
- **Proof Model Verification ([)**: Adds `verify_compiler_memory_budget_compliance`.
- **BDD Suite & Vocabulary**: BDD feature suite (`features/suites/compiler_memory_budget.feature`) and step definitions in `tests/bdd.rs`; updated `docs/formal_vocabulary.md` to v0.1.10.

Closes #169.

## Timeline (3 entries; chronological: conversation comments, reviews, review comments)

### comment 5076539624 — 2026-07-25T02:52:25Z — chatgpt-codex-connector[bot]

You have reached your Codex usage limits for code reviews. You can see your limits in the [Codex usage dashboard](https://chatgpt.com/codex/cloud/settings/usage).

### comment 5076539734 — 2026-07-25T02:52:27Z — Casey-allard

@copilot please review this PR

### comment 5076556122 — 2026-07-25T02:57:34Z — Copilot


> @copilot please review this PR

Reviewed the PR. All tests pass (18 unit tests in `uor-r4-graph-compiler`, 5 proof-model tests, 87 BDD scenarios / 292 steps — all green). Code review and CodeQL are clean.

One defect found and fixed in commit `fix(docs): restore missing Changelog section header in formal_vocabulary.md`: the `## Changelog` section header was accidentally dropped from `docs/formal_vocabulary.md` when inserting the v0.1.10 and v0.1.9 entries, leaving the bullet list stranded under `## 6. Enforcement`. Restored the header.

Everything else looks correct:
- `CompilerMemoryBudget::derive` correctly enforces the minimum threshold and the concurrency-aware `max_in_flight_tasks` calculation is sound.
- `InFlightBackpressureLimiter` uses a proper CAS loop (`compare_exchange_weak`) with `Acquire`/`Release` ordering and a `Drop`-based RAII guard for safe slot release.
- `StageNode::memory_estimate()` delegates cleanly to the four concurrency classes.
- `verify_compiler_memory_budget_compliance` in the proof model exercises all three sub-checks (derivation, rejection, backpressure cap) without panicking.


---
_Comment/review ids in this file: 5076539624, 5076539734, 5076556122_

