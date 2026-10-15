# #768 — fix(cli): simplify plain text corpus input to single-submit prompt line

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-17T17:31:24Z
- **Updated:** 2026-10-09T14:17:01Z
- **Closed:** 2026-08-17T17:52:47Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/768
- **Merged:** 2026-08-17T17:52:47Z
- **Merge commit:** 3304e65456b3cfae5607171ee9d362f1ae5e7322
- **Base / Head:** main / UOR-Foundation:fix/cli-corpus-paste-single-line
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Summary

Fixes a latent inconsistency in the interactive CLI's "paste plain text
content" flow (`src/chat.rs`, option 2 of the corpus-add menu): the prompt
text already reads as a single-line prompt (`corpus-text > `), but the
code behind it still waited for a multi-line, `END`-terminated block
before submitting anything. A user typing one line and pressing enter
would see nothing happen.

Recovered from a small local branch that predated recent large-scale
rewrites of `src/server.rs`/`src/chat.rs` (module went from ~1.7k to
~12.9k lines since). Cherry-picked onto current `main` and hand-resolved
the one conflict; verified the target code path is otherwise unchanged
on `main` and this fix is not already covered elsewhere. Two sibling
fixes recovered from the same local batch (`src/server.rs` fallback-cascade
gating and teacher-oracle reload matching) turned out to already be
superseded by more advanced, already-merged implementations (PR #223's
`TierResult` cascade semantics and the `#654` phase G `/uor/v1/reload`
reservation-based reload) and were not carried forward.

## Test plan

- [x] `cargo build --offline` -- clean
- [x] `cargo fmt --check` -- clean
- [x] `cargo clippy --bin r4 --offline -- -D warnings` -- clean

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

