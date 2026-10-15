# #235 — docs(readme): align legacy benchmark workflow with the current CLI

- **Type:** Pull request
- **State:** closed
- **Author:** @maurathat
- **Created:** 2026-07-27T03:36:43Z
- **Updated:** 2026-07-28T02:41:39Z
- **Closed:** 2026-07-28T02:41:39Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/235
- **Merged:** 2026-07-28T02:41:39Z
- **Merge commit:** 7ebb653546e5a69f1bf04693b9dda72ef0c2015d
- **Base / Head:** main / maurathat:issue-233-legacy-pipeline-docs
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Closes #233.

Docs-only. Updates the "Legacy benchmark and certification workflow" section of README.md to match the CLI at HEAD:

- Removes `compile` and `store` from the legacy chain — bare `compile` resolves to the HF graph-compiler path and fails without `--model`/`--source`; bare `store` panics (`crates/uor-r4-graph-cli/src/lib.rs:1710`, "run compile first"). A note now says both belong to the graph-compiler path.
- Documents the current chain (`gen → certify → compare → compare-report`), with `certify` performing compile, store, certificate, and census internally — matching the repo's own `setup` output.
- Records that `gen` output is not byte-reproducible across machines/eras (e.g. 754 stories / 30,036 held-out vs the certified 757 / 30,192), which bounds exact reproduction of downstream figures.

Local gates per AGENTS.md, all clean at `df4facd` on macOS 26.5.1 arm64 / rustc 1.95.0: `cargo fmt --check`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, `cargo test --workspace` (0 failures), and both no_std ladder checks for `uor-r4-graph-format`.

Process note: filed from a fork — outside contributors can't self-assign, so the AGENTS.md self-assign step is covered by this PR reference instead.

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

