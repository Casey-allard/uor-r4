# #1358 — Document Kimi tool use and safe local cleanup

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-23T05:43:52Z
- **Updated:** 2026-10-09T13:17:36Z
- **Closed:** 2026-09-23T05:44:50Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1358
- **Merged:** 2026-09-23T05:44:50Z
- **Merge commit:** 2fda9036e8663319e76276acb7a03fad118ce8d7
- **Base / Head:** main / UOR-Foundation:codex/kimi-tooling-storage
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The Kimi research team now has an operational map for source search, historical retrieval, primary papers, exact symbolic math, optional Lean, focused Rust/model checks, full-path cost measurements, GitHub issue/PR delivery, and local storage. It explains which local connections are live, which are stale or intentionally disabled, and how to preserve unique research while reclaiming only verified reproducible cache material.

At setup, disk had about 26 GiB free. A narrowly identified inactive OpenCode updater cache (215,185,388 bytes) was removed locally; no model, sealed evidence, worktree or Downloads material was touched. The local cleanup and credentials are outside this PR.

Validation: `git diff --check` and `python3 scripts/check_claim_wording.py` passed. This is documentation only; no model behavior claim changes.

References #820.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

