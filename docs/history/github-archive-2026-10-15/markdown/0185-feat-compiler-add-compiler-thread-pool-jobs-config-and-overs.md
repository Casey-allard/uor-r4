# #185 — feat(compiler): add compiler thread-pool, jobs config, and oversubscription policy (#168)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-07-25T02:23:51Z
- **Updated:** 2026-10-09T14:16:54Z
- **Closed:** 2026-07-25T02:50:32Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/185
- **Merged:** 2026-07-25T02:50:32Z
- **Merge commit:** 177053dee23f041feabb460c1226144939e6bb97
- **Base / Head:** main / UOR-Foundation:issue-168-compiler-jobs-config
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Summary
Implements Issue #168: Compiler thread-pool, jobs configuration, and oversubscription policy.

- **Normative Specification ([](file:///Users/casey.allard/uor-r4/docs/compiler_concurrency_config.md))**: Defines precedence resolution (`CLI > env > default`), default thread count policy (`min(available_logical_cpus(), 8)`), dedicated custom named Rayon thread-pool ownership (`r4-compile-<n>`), and oversubscription policies.
- **Jobs Configuration & Pool Builder ()**: Implements `CompilerJobsConfig`, `JobsConfigSource`, `JobsConfigError`, and `build_dedicated_thread_pool()`.
- **Proof Model Verification ()**: Adds `verify_compiler_jobs_config_compliance`.
- **BDD Suite & Vocabulary**: BDD feature suite (`features/suites/compiler_jobs_config.feature`) and step definitions in `tests/bdd.rs`; updated `docs/formal_vocabulary.md` to v0.1.9.

Closes #168.

## Timeline (3 entries; chronological: conversation comments, reviews, review comments)

### comment 5076409248 — 2026-07-25T02:23:55Z — chatgpt-codex-connector[bot]

You have reached your Codex usage limits for code reviews. You can see your limits in the [Codex usage dashboard](https://chatgpt.com/codex/cloud/settings/usage).

### comment 5076409395 — 2026-07-25T02:23:57Z — Casey-allard

@copilot please review this PR

### comment 5076424204 — 2026-07-25T02:27:51Z — Copilot


> @copilot please review this PR

Reviewed. All tests pass (18 graph-compiler, 5 proof-model, 8 observe), clippy is clean, formatting is clean, and the claim-wording gate passes.

**Overall**: Clean, well-structured implementation. A few observations worth noting:

**1. Integration gap (expected but worth tracking)**
`CompilerJobsConfig` and `build_dedicated_thread_pool` are not yet wired into the actual `compile()` pipeline (`lib.rs`) or `parse_options()`. The `--jobs` CLI flag is not parsed and `R4_COMPILER_THREADS` is not read at startup. This PR establishes the infrastructure (types, resolution logic, pool builder) but the compile path still ignores the config entirely. If this is intentional deferral, it should be tracked (e.g. as a sub-task of #169's memory-budget integration).

**2. `build_dedicated_thread_pool` returns `String` as error**
The other public error surface uses focused enums (`JobsConfigError`), but the pool builder uses `map_err(|e| format!(...))`. This is pragmatic given `rayon::ThreadPoolBuildError`'s trait limitations, but it breaks the error-handling contract at the library boundary. Consider a `JobsConfigError::ThreadPoolBuildFailed(String)` variant so callers can match without string parsing.

**3. BDD coverage gap**
The feature file covers CLI precedence, env fallback, and zero-rejection but misses the `InvalidJobCount` path (e.g. `R4_COMPILER_THREADS=abc`). That case is covered in unit tests, so it's not a gap in correctness, but adding it to the BDD suite would complete scenario parity.

**4. `verify_compiler_jobs_config_compliance` error kind**
The function maps precedence assertion failures to `ProofValidationError::NondeterministicOutput`. Precedence logic failures aren't nondeterminism — a `VerificationConstraintViolated` or similar variant would be more semantically accurate if one exists or is added later.


---
_Comment/review ids in this file: 5076409248, 5076409395, 5076424204_

