# #577 — feat(#510): R5 tranche — graph-cli host-CID helpers to SourceUnavailable (graph-cli 27->24)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-10T13:49:59Z
- **Updated:** 2026-10-09T13:09:01Z
- **Closed:** 2026-08-10T14:10:05Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/577
- **Merged:** 2026-08-10T14:10:04Z
- **Merge commit:** c6662e99811002e8c723476f13dfef6e4684ca0b
- **Base / Head:** main / UOR-Foundation:r5/graph-cli-cid-helpers
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## R5 tranche — graph-cli host-CID helpers → `SourceUnavailable`

First `lib.rs` cluster in the #510 R5 rearchitecture (graph-cli is the last crate).

The host content-address helpers `file_cid`, `collect_file_entries`, and
`directory_cid` read files/directories from disk, so their error surface becomes
the sanctioned `SourceUnavailable` (`io::Error` via `From`, path context via
`SourceUnavailable::new`).

These are internal-only leaves. The sole out-of-cluster caller, `evaluate_report`
(still `String` this tranche), maps them back via `.map_err(|e| e.to_string())`.
No root-package impact.

### Audit
`xtask audit-limits`: graph-cli **27 → 24** (`lib.rs` 26 → 23).
Remaining in graph-cli: `lib.rs` (23), `convert_r4g1.rs` (1).

Gauntlet: build / fmt --check / clippy --all-targets / `cargo test -p uor-r4-graph-cli`
(24 pass) clean; root `uor-r4-wasm-router` builds.

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

