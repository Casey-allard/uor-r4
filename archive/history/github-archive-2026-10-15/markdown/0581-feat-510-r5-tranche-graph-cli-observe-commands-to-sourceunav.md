# #581 — feat(#510): R5 tranche — graph-cli observe commands to SourceUnavailable (graph-cli 11->9)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-10T16:18:55Z
- **Updated:** 2026-10-09T13:09:03Z
- **Closed:** 2026-08-10T16:37:02Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/581
- **Merged:** 2026-08-10T16:37:02Z
- **Merge commit:** aa46c66ebf8f5f43b55cc05669c9958d74cac60b
- **Base / Head:** main / UOR-Foundation:r5/graph-cli-observe-commands
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## R5 tranche — graph-cli observe commands → SourceUnavailable

Converts the two teacher-observation commands (`observe`, `observe-text`) to the
sanctioned `SourceUnavailable` surface. Both are host commands: they load the
teacher (HF / legacy checkpoint), export the tokenizer, run the sharded
observation, and write the merged corpus — every failure is a host-resource
condition. Their option parsers are already `SourceUnavailable`, so parse
propagates via `?`; `io::Error` sites (`create_dir_all`, `write`) use `?` via
`From`; the remaining `Display` errors map through `SourceUnavailable::new`. The
two `run()` dispatch arms map back via `to_string`.

Internal-only (not called from the root package).

audit-limits: graph-cli **11 → 9**. Gauntlet green (build/fmt/clippy/test + root).

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

