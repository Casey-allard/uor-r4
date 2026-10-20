# #583 — feat(#510): R5 tranche — graph-cli evaluate_report to SourceUnavailable (graph-cli 8->7)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-10T17:23:01Z
- **Updated:** 2026-10-09T13:09:02Z
- **Closed:** 2026-08-10T17:42:10Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/583
- **Merged:** 2026-08-10T17:42:10Z
- **Merge commit:** 19cdf7c59839a681b98ee4399d6ec95f870f827f
- **Base / Head:** main / UOR-Foundation:r5/graph-cli-evaluate-report
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## R5 tranche — graph-cli evaluate_report → SourceUnavailable

Converts the hf `evaluate-report` command to the sanctioned `SourceUnavailable`
surface. Host command: it CID-checks the compiled bundle, loads the HF teacher,
replays the held-out split against the store, and writes the evaluation report.
Its parser + the `file_cid`/`directory_cid` helpers are already
`SourceUnavailable`, so those calls collapse to `?`; `io::Error` and
`serde_json` both have `From` so they use `?` too; the remaining literal /
`format!` / tokenizer-CID error sites map through `SourceUnavailable::new`. The
`run()` dispatch arm maps back via `to_string`.

Internal-only (dispatched from `run()`; not root-called).

audit-limits: graph-cli **8 → 7**. Gauntlet green (build/fmt/clippy/test + root).

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

