# #575 — feat(#510): R5 tranche — graph-cli runtime-corpus + recommend-scale to SourceUnavailable (graph-cli 42->34)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-10T12:49:06Z
- **Updated:** 2026-10-09T13:09:01Z
- **Closed:** 2026-08-10T13:08:06Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/575
- **Merged:** 2026-08-10T13:08:06Z
- **Merge commit:** 8893a9e2229f42da8eec783e8d0ddaee99636a18
- **Base / Head:** main / UOR-Foundation:r5/graph-cli-aux-commands-source-unavailable
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## R5 tranche — graph-cli `runtime-corpus` + `recommend-scale` → `SourceUnavailable`

First graph-cli PR in the #510 R5 rearchitecture. Converts the two self-contained
command modules so their host-source I/O surfaces the sanctioned `SourceUnavailable`
error instead of bare `String`.

Both are host-only paths that read teacher configs / seed corpora / artifact bytes
from disk. Every failure mode is a host-resource condition (missing file, unreadable
path, malformed input the caller supplied), so `SourceUnavailable` (free-form reason)
is the correct sanctioned surface — matching the `api/compile.rs` precedent.

### Changes
- `runtime_corpus.rs`: `parse_options`, `file_kappa`, `append_runtime_record`, `run`
  → `Result<_, SourceUnavailable>`; bare `io::Error` sites propagate via `?`
  (`From<io::Error>`), `serde_json` via `From<serde_json::Error>`.
- `recommend_scale.rs`: `config_from_hf`, `run` → `Result<_, SourceUnavailable>`.
- `lib.rs`: the two dispatch arms map `SourceUnavailable` back to the `String`
  surface the top-level `run()` still uses.

### Audit
`xtask audit-limits`: graph-cli **42 → 34**. `runtime_corpus.rs` and
`recommend_scale.rs` are now at 0. Remaining in graph-cli: `lib.rs` (26),
`cover_sweep.rs` (7), `convert_r4g1.rs` (1) — follow-up PRs.

Gauntlet: build / fmt --check / clippy --all-targets / `cargo test -p uor-r4-graph-cli`
(24 pass) all clean; root `uor-r4-wasm-router` builds.

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

