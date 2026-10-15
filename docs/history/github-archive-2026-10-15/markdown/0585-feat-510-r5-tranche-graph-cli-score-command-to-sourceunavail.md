# #585 — feat(#510): R5 tranche — graph-cli score_command to SourceUnavailable (graph-cli 6->5)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-10T18:17:18Z
- **Updated:** 2026-10-09T13:09:03Z
- **Closed:** 2026-08-10T18:36:29Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/585
- **Merged:** 2026-08-10T18:36:29Z
- **Merge commit:** e4c487dc656df574ef29d2955001d48c8caa554f
- **Base / Head:** main / UOR-Foundation:r5/graph-cli-score-command
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## R5 tranche — graph-cli score_command → SourceUnavailable

Converts `score_command` (the scoring + Gate C evaluation pipeline) to the
sanctioned `SourceUnavailable` surface. Host command: reads teacher + corpus (+
optional cached cover + stories index), compiles transitions/emissions, emits the
scored R4G1, runs Gate C, writes the report. Parser already `SourceUnavailable`
so parse propagates via `?`; `Display` errors (`StoryIndex::load`,
`recover_from_artifact`, io, serde) map via `SourceUnavailable::new`; path/load
failures via `SourceUnavailable::new(format!)`. Two defects panic: vocab-overflow
(`token_codes/STAGES > u32`) and "cannot address emitted R4G1 artifact"
(addressing the command's OWN just-emitted bytes) — both `.expect()`.

Root touch: `src/server.rs::compile_r4g1_bundle` score call maps via `to_string`;
`api/compile.rs` calls `score_command` via `format!()` (Display) and still builds.
`run()` score arm maps via `to_string`.

audit-limits: graph-cli **6 → 5**. Gauntlet green (graph-cli + root + api build,
fmt/clippy/test).

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

