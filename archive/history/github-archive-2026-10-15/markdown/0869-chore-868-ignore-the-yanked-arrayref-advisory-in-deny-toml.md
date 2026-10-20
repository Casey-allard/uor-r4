# #869 — chore(#868): ignore the yanked arrayref advisory in deny.toml

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-20T13:14:11Z
- **Updated:** 2026-10-09T13:08:56Z
- **Closed:** 2026-08-20T13:47:04Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/869
- **Merged:** 2026-08-20T13:47:04Z
- **Merge commit:** 022494660634251b98f8115b4772fa15c9702509
- **Base / Head:** main / UOR-Foundation:issue-868-arrayref-yanked-advisory
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Problem

`cargo deny check advisories` is red on `main` (surfaced in the merge-queue
`register conformance` job as `advisories FAILED, bans ok`): the entire `arrayref`
0.3.x line was **yanked** upstream. Non-blocking (register-conformance isn't a
required check) but red on every PR.

## Why not just upgrade

`arrayref` is pulled only transitively by `blake3` 1.5.x, and blake3 can't move past
1.5.x here: the pinned `uor-addr` rev's `uor-prism-crypto v0.4.0` requires
`blake3 >=1.5, <1.6`, and every blake3 1.5.x still bundles `arrayref` (dropped only
at >= 1.6). I verified this — a workspace blake3 bump to 1.8.7 fails resolution
against the `<1.6` cap, and 1.5.5 (the max allowed) keeps `arrayref`. The only root
fix is a **`uor-addr` pin bump**, which is a separate ecosystem decision (tracked in
#868), so I did not make it here.

## Fix

A single targeted `deny.toml` `[advisories].ignore` for `arrayref@0.3.9`, naming the
crate and the reason there is no upgrade (per the file's own convention). It's a
maintainer yank, not a vulnerability (no RUSTSEC advisory). `bans` and `licenses`
are unaffected.

## Verification

- `cargo deny check advisories` → **advisories ok** (locally, cargo-deny 0.20.2).
- `cargo deny check bans` → **bans ok**.
- One-file change (`deny.toml`); no dependency, code, or lockfile change.

## Removal condition

Delete the ignore the moment `uor-addr` moves to a blake3 (>= 1.6) that drops
`arrayref` — tracked in #868.

Closes #868


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

