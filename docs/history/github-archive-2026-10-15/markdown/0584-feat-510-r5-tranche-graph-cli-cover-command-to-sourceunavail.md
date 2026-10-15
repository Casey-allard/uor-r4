# #584 — feat(#510): R5 tranche — graph-cli cover_command to SourceUnavailable (graph-cli 7->6)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-10T17:47:04Z
- **Updated:** 2026-10-09T13:09:02Z
- **Closed:** 2026-08-10T18:06:30Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/584
- **Merged:** 2026-08-10T18:06:30Z
- **Merge commit:** 9280cd3e76b6c2440beb7f2d78ff7339b27e2c9d
- **Base / Head:** main / UOR-Foundation:r5/graph-cli-cover-command
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## R5 tranche — graph-cli cover_command → SourceUnavailable

First of the root-facing graph-cli commands. Converts `cover_command` (the cover
induction + R4G1 emission pipeline) to the sanctioned `SourceUnavailable`
surface. Host command: reads the teacher container + corpus streams, induces the
cover, emits the scored artifact, writes the report. Parser already
`SourceUnavailable` so parse propagates via `?`; io/serde use `?`/`From`;
path-context and load failures map through `SourceUnavailable::new(format!)`. The
vocab-overflow (`token_codes/STAGES` exceeding `u32`) is an impossible-in-practice
defect on a validated artifact → `.expect()`.

Root touch: `src/server.rs::compile_r4g1_bundle` calls `cover_command` and returns
`Result<_, String>`, so its call site maps via `to_string`. The `run()` cover
dispatch arm maps back via `to_string` (`run()` itself is converted in its own
final cluster).

audit-limits: graph-cli **7 → 6**. Gauntlet green (graph-cli + root + api build,
fmt/clippy/test).

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

