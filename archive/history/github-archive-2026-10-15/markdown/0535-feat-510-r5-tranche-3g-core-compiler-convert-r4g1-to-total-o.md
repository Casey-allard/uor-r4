# #535 — feat(#510): R5 tranche 3g — core compiler + convert_r4g1 to total ops

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-09T08:25:00Z
- **Updated:** 2026-10-09T13:08:54Z
- **Closed:** 2026-08-09T08:37:22Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/535
- **Merged:** 2026-08-09T08:37:22Z
- **Merge commit:** 86c8bc66dfdfcafa16e4e64389bdd202a81092d2
- **Base / Head:** main / UOR-Foundation:feat/510-r5-t3g-compiler
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Continues **core** (issue #510): the recorded-corpus compile entry points and the R4G1 converter.

- `RecordedRepresentation::from_corpus` → `Option<Self>`, `compile_recorded` → `Option<Compiled>`. A recorded corpus either compiles or it does not — a zero/oversized vocabulary or corpus arrays disagreeing with the record count yield `None`.
- `convert_r4g1::convert` → `Option<(Vec<u8>, ConversionReport)>`. Malformed token codes, an over-u32 vocabulary, wrong-shape class-signature books, or a serialization failure yield `None`.

Callers: the graph-cli recorded-compile and R4G1-convert commands map `None` to their own `String` context via `ok_or_else`; tests use `.expect`/`.unwrap`.

`audit-limits`: core `compiler.rs` and `convert_r4g1.rs` report zero unsanctioned errors. Build / clippy (`-D warnings`) / fmt clean; all workspace test targets compile.

Note: two categories remain in core for follow-up tranches — the serde `serialize`/`deserialize` helpers (`Result<S::Ok, S::Error>`, a serde trait contract the gate should carve out) and the legacy `scenarios.rs` `io::Result` I/O.

🤖 Generated with [Claude Code](https://claude.com/claude-code)

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

