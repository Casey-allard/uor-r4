# #619 — build(#618): collapse duplicate uor-foundation/-sdk builds via patch.crates-io

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-11T22:52:25Z
- **Updated:** 2026-10-09T14:17:06Z
- **Closed:** 2026-08-11T23:11:58Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/619
- **Merged:** 2026-08-11T23:11:58Z
- **Merge commit:** eab191eb31863b4530cd2938022c4ec511efdad7
- **Base / Head:** main / UOR-Foundation:i618-registry-dedupe
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Closes #618. Sub-issue of #589 (ecosystem rebase, step 1: uor-framework).

## What

Adds a `[patch.crates-io]` section mapping `uor-foundation` and `uor-foundation-sdk` onto the already-pinned Framework rev `51c0138`. `Cargo.lock` collapses from **two copies of each** (our git pins + the registry copies pulled transitively via uor-addr → uor-prism) to **one**, unifying type identity across uor-addr, uor-prism, and r4. Two files touched: `Cargo.toml` (+14 comment-heavy lines), `Cargo.lock` (−37/+17).

## Why not "switch to registry 0.5.2" (the issue's original plan)

`uor-foundation-verify` is published only in the Framework git workspace, is a live dev-dep (`src/tless_uor.rs::grounded_mints_and_replays`), and `verify_trace` consumes types minted by our `uor_foundation` — registry-foundation + git-verify would split type identity and break the test build. Direction flipped, goal unchanged; recorded on #618. Safe because the pinned rev's foundation crates are code-identical to published 0.5.2 (README-only delta; audit on #589).

## Breaking changes

None. No API, no runtime code, no version change — same rev, fewer builds of it. Router runtime remains zero-matmul; deployed-kernel invariant untouched.

## Verification (all run on this branch, exit codes read bare)

1. `grep -c 'name = "uor-foundation"' Cargo.lock` → **1** (was 2); same for `-sdk` → **1** (was 2)
2. `cargo fmt --check` → 0
3. `cargo check -p uor-r4-graph-format --no-default-features` and `--features alloc` → 0, 0
4. `cargo clippy --workspace --all-targets --all-features -- -D warnings` → 0
5. `cargo test --workspace` → 0 (120 suite blocks, zero failures — includes the `uor_foundation_verify` replay test and the allocation census)

## Maintenance note

The `[patch.crates-io]` revs must move together with the `[dependencies]` git pins — the Cargo.toml comment says so at the patch site. When Framework publishes verify to crates.io, the whole section can be replaced by plain registry deps (that would be the original #618 plan; re-evaluate at the next framework version bump).


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

