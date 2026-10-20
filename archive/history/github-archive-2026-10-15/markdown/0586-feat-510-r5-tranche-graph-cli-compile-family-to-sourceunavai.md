# #586 — feat(#510): R5 tranche — graph-cli compile family to SourceUnavailable (graph-cli 5->1)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-10T18:43:55Z
- **Updated:** 2026-10-09T13:09:01Z
- **Closed:** 2026-08-10T19:03:14Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/586
- **Merged:** 2026-08-10T19:03:14Z
- **Merge commit:** e9807540b12b67825107b72196f78114fe1aa35e
- **Base / Head:** main / UOR-Foundation:r5/graph-cli-compile-family
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## R5 tranche — graph-cli compile family → SourceUnavailable

Converts the Hugging Face compile family to the sanctioned `SourceUnavailable`
surface: `download_hf_source`, the `calculate_file_hash` closure,
`compile_hugging_face_with_progress`, and the `compile_hugging_face` wrapper. Host
path throughout (validates model/revision, runs the `hf` download subprocess,
loads the teacher, generates + compiles the corpus, writes the bundle +
manifest). `parse_compile_options` already `SourceUnavailable` so parse
propagates via `?`; `download_hf_source` + `calculate_file_hash` now propagate via
bare `?` (`SourceUnavailable` has no `From<String>`); `Display` errors map via
`SourceUnavailable::new`; the HF-load + hf-subprocess + status messages via
`SourceUnavailable::new(format!)`.

Root edits (root pkg not audited but must build): `src/main.rs`
`compile_hugging_face` call maps via `to_string`; `src/server.rs`'s compile thread
`JoinHandle` now yields `Result<(), SourceUnavailable>`, so the inner result maps
to `String` after join. `api/compile.rs` still builds. `run()` compile arm maps
via `to_string` (`run()` itself is the last remaining site).

audit-limits: graph-cli **5 → 1** (only `run()` left). Gauntlet green: graph-cli +
root + api build, fmt/clippy/test.

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

