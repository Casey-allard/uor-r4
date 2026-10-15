# #582 — feat(#510): R5 tranche — graph-cli compile_recorded_corpus to SourceUnavailable (graph-cli 9->8)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-10T16:52:20Z
- **Updated:** 2026-10-09T13:09:01Z
- **Closed:** 2026-08-10T17:11:14Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/582
- **Merged:** 2026-08-10T17:11:14Z
- **Merge commit:** 2b5c2fed1af9ba0f7f42280bea335fc874437875
- **Base / Head:** main / UOR-Foundation:r5/graph-cli-compile-recorded
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## R5 tranche — graph-cli compile_recorded_corpus → SourceUnavailable

Converts the recorded-corpus compile command (the observation-first, no-teacher
production path) to the sanctioned `SourceUnavailable` surface. Host command: it
reads the recorded corpus, compiles the table artifact + store, and writes the
bundle. Its parser (`parse_recorded_compile_options`) is already
`SourceUnavailable` so parse propagates via `?`; `io::Error` sites
(`create_dir_all`, `write`, `copy`) use `?` via `From`; `serde_json`
serialization maps via `SourceUnavailable::from`. The `run()` dispatch arm maps
back via `to_string`.

Internal-only (dispatched from `run()`; not called directly from the root
package).

audit-limits: graph-cli **9 → 8**. Gauntlet green (build/fmt/clippy/test + root).

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

