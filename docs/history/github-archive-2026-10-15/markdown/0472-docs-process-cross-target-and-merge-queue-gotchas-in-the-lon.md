# #472 — docs(process): cross-target and merge-queue gotchas in the long-run discipline

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-07T02:46:15Z
- **Updated:** 2026-10-09T14:17:06Z
- **Closed:** 2026-08-07T04:54:14Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/472
- **Merged:** 2026-08-07T04:54:14Z
- **Merge commit:** 341f63f334f79cb52fb72c5821022baed0c121e8
- **Base / Head:** main / UOR-Foundation:i460-disc2
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

References #460. Two gaps found the expensive way while shipping #470.

**Cross-target.** `cargo clippy --workspace --all-targets` does not build other *targets* — the merge queue builds wasm, so PR checks were green and the queue build failed on `corpus_codes_cached` not existing under `wasm32`. Any `uor-r4-core` change now needs `cargo check --target wasm32-unknown-unknown -p uor-r4-wasm-router --lib` before shipping, and a filesystem-touching helper gated `#[cfg(not(target_arch = "wasm32"))]` should get a wasm counterpart rather than making every caller cfg-aware.

**Merge-queue silence.** `gh pr merge` printing nothing usually means the PR queued, not that it failed — verify with `git ls-remote origin 'refs/heads/gh-readonly-queue/*'` or by re-reading the PR state, and never reach for `--admin` on a shared repo.

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

