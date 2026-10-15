# #771 — chore: add scripts/fetch_simple_wiki_20k.py (#531 corpus-scale fetch utility)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-17T20:09:45Z
- **Updated:** 2026-08-17T20:32:56Z
- **Closed:** 2026-08-17T20:30:33Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/771
- **Merged:** 2026-08-17T20:30:33Z
- **Merge commit:** cc7cb42c31e5cd98c0d1719ac899c0f1c8949884
- **Base / Head:** main / UOR-Foundation:chore/add-fetch-simple-wiki-script
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## What

Adds `scripts/fetch_simple_wiki_20k.py`, a corpus-fetch utility (extends
`scripts/fetch_d3_corpus.py`'s 3000-article D3 sample to a larger `TARGET`
so a teacher observe run can pass the ~1-2M-record coverage knee). Found
untracked in the base checkout while auditing stale state left over from
an earlier session (see the handoff brief for #653/#655 work). References
the now-closed #531 corpus-scale sweep.

## Why committing rather than discarding

It's complete, working, and was already used locally to produce an
`obs-text/` observation corpus still present in the checkout — real
unshipped work product, not scratch. Same resumable-fetch pattern as the
existing `scripts/fetch_d3_corpus.py`.

## Verification

- `python3 -m py_compile scripts/fetch_simple_wiki_20k.py` — clean
- `cargo fmt --check` — clean (no Rust changed)
- `cargo build --workspace` — clean
- `cargo clippy --workspace --all-targets --all-features --offline -- -D warnings` — clean
- `cargo test --workspace --offline` — all pass, 0 failures

No behavior change to any shipped crate.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

