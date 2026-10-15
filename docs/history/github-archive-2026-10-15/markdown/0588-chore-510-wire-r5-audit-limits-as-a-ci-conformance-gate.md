# #588 — chore(#510): wire R5 audit-limits as a CI conformance gate

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-10T19:36:38Z
- **Updated:** 2026-10-09T13:09:04Z
- **Closed:** 2026-08-10T19:55:42Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/588
- **Merged:** 2026-08-10T19:55:42Z
- **Merge commit:** 4c983fd268b475e0ea862fbc35df2d561d93e1af
- **Base / Head:** main / UOR-Foundation:r5/ci-audit-limits-gate
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## chore(#510): wire R5 audit-limits as a CI conformance gate

The R5 error-surface migration is complete — graph-cli hit 0 in #587, so every
shipped crate's `Result` now names only the four sanctioned error types
(`NotAProduct` / `ObservedBound` / `KappaError` / `SourceUnavailable`).

This adds an **R5 audit-limits** step to the `register-conformance` job, right
after `audit-deferral`:

```yaml
      - name: R5 audit-limits (every shipped Result names only sanctioned error types)
        run: cargo run -q -p xtask -- audit-limits
```

`xtask audit-limits` already exits nonzero on any finding (`main.rs` maps the gate
`Err` to `ExitCode::FAILURE`), so this fails CI on any reintroduced unsanctioned
`Result`. The job comment + name are updated to reflect that R5 is now enforced.

> **Note:** promoting `register-conformance` to a GitHub **required** status check
> is a repo-admin branch-protection setting — the workflow file can't self-promote.
> Until that toggle is flipped, this gate runs and reports on every PR but won't
> block merge on its own.

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

