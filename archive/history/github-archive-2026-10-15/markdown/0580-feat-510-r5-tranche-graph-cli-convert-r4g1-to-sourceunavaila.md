# #580 — feat(#510): R5 tranche — graph-cli convert_r4g1 to SourceUnavailable/panic (graph-cli 12->11)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-10T15:42:28Z
- **Updated:** 2026-10-09T13:09:02Z
- **Closed:** 2026-08-10T16:01:45Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/580
- **Merged:** 2026-08-10T16:01:45Z
- **Merge commit:** 35634bc881f5b8644535c1dda73a3389efaf6208
- **Base / Head:** main / UOR-Foundation:r5/graph-cli-convert-r4g1
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## R5 tranche — graph-cli convert_r4g1 → SourceUnavailable / panic

Converts the `convert-r4g1` command (TLA + TLS1 → R4G1) to the sanctioned surface.

- **Host input path → `SourceUnavailable`**: CLI arg parse, `fs::read` of the
  artifact/store containers, `parse_artifacts` / `parse_store` rejection,
  calibration `read_to_string` + `serde_json` parse, the `convert()` `None`, and
  the `fs::write` of the output.
- **Self-produced-bytes validation → panic**: the two "fail closed" checks on the
  converter's OWN output (`GraphView::parse` + `verify_cids`) are defects if they
  ever fire, not host conditions, so they now `.expect()`.

`lib.rs` dispatch arm maps `SourceUnavailable` back via `to_string`. `convert_r4g1.rs`
is now at 0.

audit-limits: graph-cli **12 → 11**. Gauntlet green (build/fmt/clippy/test + root).

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

