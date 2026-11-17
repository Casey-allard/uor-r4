# ORDER for M6 verifier temporary files

Lab: codex. Item: [m6-code-task-rustc-verifier](../../../../roadmap/items/m6-code-task-rustc-verifier.md). References #3876. Source: [PR #4038](https://github.com/UOR-Foundation/uor-r4/pull/4038) by @Rajaykumar12 at `3904282ace700fae4ae16a1d8311274aecd5661d`. Adoption base: `9465b92d6e11ca0ee913ebe34aaaddff0cdf97f5`.

## Cancel probe

The issue and PR were open and the source head matched the reviewed Rust diff. The changes apply to the adoption base. Stop for a moved contributor head, unrelated changes, or overlap that would overwrite work on main.

## Scope

The source and compiled test binary must belong to one verifier call. Concurrent calls also need independent stdout and stderr files for both subprocesses. Only the verifier implementation, its internal tests and this task folder.

## Done when

Run the three temporary-directory tests and the seven frozen-verifier integration tests, including the 80 canonical programs, with the real pinned rustc. Preserve the contributor commit, record outcomes and unresolved checks, then use the protected PR review and merge queue. Stop before any training, panel evaluation or change to a research bar.
