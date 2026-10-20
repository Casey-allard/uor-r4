# #563 — feat(#510): R5 tranche — graph-certify Smoothing::parse to Option (certify 22->21)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-10T00:09:26Z
- **Updated:** 2026-10-09T13:09:04Z
- **Closed:** 2026-08-10T00:28:27Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/563
- **Merged:** 2026-08-10T00:28:27Z
- **Merge commit:** 1c38234b797fc1c5acd443ff52829f610d01357a
- **Base / Head:** main / UOR-Foundation:r5/smoothing-parse-option
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## R5 tranche (#510): graph-certify `Smoothing::parse` → `Option`

`Smoothing::parse` parses a `--smoothing` flag value; a parse failure is invalid
user input, not one of the sanctioned conditions. Converts it from
`Result<Smoothing, String>` to **`Option<Smoothing>`** (`None` = not a valid
smoothing spec: unknown rule, non-numeric δ, or δ outside the finite `(0, 1]`
range).

The single caller (graph-cli `parse_score_options`) supplies the user-facing
message via `.ok_or_else(...)`, keeping its own `Result<_, String>` CLI surface
until graph-cli's own R5 tranche.

### Audit
`graph-certify` audit-limits: **22 → 21** (score.rs 6 → 5). No sanctioned error
types introduced — pure removal of a `Result<_, String>`.

### Gauntlet
`cargo fmt --check`, `cargo build --workspace --all-targets`, `clippy` (certify +
cli, all-targets), and both crates' lib test suites all green.

### Remaining graph-certify (after this)
5 sites in `score.rs` (the gate-C cluster + `recover_from_artifact`) and 16 in
`score_runtime.rs` — all coupled to the `GraphScorer` methods, so they land after
`score_runtime.rs` converts.

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

