# #868 — cargo-deny advisories red on main: yanked arrayref (transitive via pinned uor-addr blake3 1.5.x)

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-08-20T13:13:24Z
- **Updated:** 2026-08-20T13:47:06Z
- **Closed:** 2026-08-20T13:47:06Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/868
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Summary

`cargo deny check advisories` is red on `main`: the whole `arrayref` 0.3.x line
(0.3.5–0.3.9) was **yanked** upstream. It surfaced in the merge-queue
`register conformance (R1..R6)` job (`advisories FAILED, bans ok`) — non-blocking
(register-conformance is not a required branch-protection status) but it leaves the
gate red on every PR.

## Root cause

`arrayref` is pulled **only transitively**, by `blake3` 1.5.x. blake3 cannot be
upgraded past 1.5.x in this graph: the pinned `uor-addr` rev
(`165b51e3…`)'s `uor-prism-crypto v0.4.0` requires `blake3 >=1.5, <1.6`, and every
blake3 1.5.x still bundles `arrayref` (blake3 drops it only at >= 1.6). Verified:
bumping the workspace blake3 pin to 1.8.7 fails resolution against
`uor-prism-crypto`'s `<1.6`; bumping to the latest allowed (1.5.5) keeps `arrayref`.

So the only true root fix is a **`uor-addr` pin bump** to a rev whose
`uor-prism-crypto` uses a blake3 that no longer depends on `arrayref` — a separate
ecosystem decision.

## Fix (this issue)

A targeted, documented `deny.toml` `[advisories].ignore` for `arrayref@0.3.9`,
naming the crate and the reason there is no upgrade (per the file's own convention).
Not a vulnerability — a maintainer yank, no RUSTSEC advisory. `bans` stays green.

## Removal condition

Delete the ignore the moment `uor-addr` moves to a blake3 (>= 1.6) that drops
`arrayref`. Tracked here.


## Comments (0)

_No comments._

---
_Comment/review ids in this file: _

