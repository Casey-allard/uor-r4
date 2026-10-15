# #578 — feat(#510): R5 tranche — graph-cli option parsers to SourceUnavailable (graph-cli 24->16)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-10T14:39:47Z
- **Updated:** 2026-10-09T13:09:03Z
- **Closed:** 2026-08-10T14:58:58Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/578
- **Merged:** 2026-08-10T14:58:58Z
- **Merge commit:** a10d97da1e62c4dd1c01988f70805600ac0f528c
- **Base / Head:** main / UOR-Foundation:r5/graph-cli-option-parsers
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## R5 tranche — graph-cli option parsers → `SourceUnavailable`

Second `lib.rs` cluster in the #510 R5 rearchitecture. Converts all eight CLI
option/spec parsers to the sanctioned `SourceUnavailable` surface, matching the
`runtime_corpus` / `cover_sweep` precedent (host-supplied argument input →
`SourceUnavailable`, free-form reason):

`parse_recorded_compile_options`, `parse_observe_options`,
`parse_observe_text_options`, `parse_cover_options`, `parse_score_options`,
`parse_compile_options`, `parse_evaluate_report_options`, `parse_skeleton`.

Each is called by its `*_command` via `parse_x(args)?`; those commands still
return `Result<(), String>` this tranche, so the call sites map back via
`.map_err(|e| e.to_string())`. In-file only, no root-package impact. The one unit
test that compared against `Err(String)` now checks `error.reason`.

### Audit
`xtask audit-limits`: graph-cli **24 → 16** (`lib.rs` 23 → 15). Remaining: the
command entrypoints + dispatcher.

Gauntlet: build / fmt --check / clippy --all-targets / `cargo test -p uor-r4-graph-cli`
(24 pass) clean; root builds.

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

