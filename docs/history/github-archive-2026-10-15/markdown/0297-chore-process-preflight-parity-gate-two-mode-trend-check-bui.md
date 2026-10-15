# #297 — chore(process): preflight parity gate, two-mode trend check, build-isolation conventions

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-07-30T05:15:05Z
- **Updated:** 2026-10-09T14:17:23Z
- **Closed:** 2026-07-30T05:29:11Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/297
- **Merged:** 2026-07-30T05:29:11Z
- **Merge commit:** d38c740351f3726b380af12ec692f1f80bb1dd3f
- **Base / Head:** main / UOR-Foundation:process-preflight
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Summary

Ends three recurring waste classes structurally (details in the commit message and AGENTS.md):

- **`scripts/preflight.sh`** — the exact CI gate locally, one command, before every push. Includes the two checks that ejected PRs from the merge queue tonight (fmt-check, trend harness) and the clippy `--all-targets --all-features` parity gap. Worktree-local target dir by default.
- **Two-mode Gate C trend check** — regression mode when the pin is untouched; re-pin self-consistency mode when a PR re-pins (its own measured row must match the new pin, both directions). Era re-pins now travel inside the PR that prices them and are merge-order-independent. All three modes verified locally.
- **AGENTS.md process conventions** — preflight-before-push, per-worktree build isolation (three stale-binary incidents in one night), enqueue≠merged watching discipline, science-run binary freshness asserts.

Self-tested: pushed only after its own preflight ran ALL GATES GREEN (κ-reproduction included).

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

