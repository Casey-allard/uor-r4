# #870 — chore(#868): vendor arrayref 0.3.9 in-tree to clear the yanked advisory at the source

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-20T14:35:45Z
- **Updated:** 2026-10-09T14:17:18Z
- **Closed:** 2026-08-20T14:58:08Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/870
- **Merged:** 2026-08-20T14:58:08Z
- **Merge commit:** 0cc3d05da7c8ec921339da98874d14256b7421cc
- **Base / Head:** main / UOR-Foundation:issue-868-vendor-arrayref
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Problem

`cargo deny check advisories` went red on `main`: the whole `arrayref` 0.3.x line
(0.3.5–0.3.9) was **yanked** upstream. It's a maintainer yank, not a RUSTSEC
advisory, and `arrayref` is pulled **only transitively** by `blake3` 1.5.x. #869
landed an interim `deny.toml` ignore to clear the red; this PR replaces that
suppression with a resolution we own.

## Why we can't just upgrade

`blake3` can't move past 1.5.x in this graph: the pinned `uor-addr` rev's
`uor-prism-crypto v0.4.0` requires `blake3 >=1.5, <1.6`, and every `blake3` 1.5.x
still bundles `arrayref` (dropped only at >= 1.6). No in-repo bump helps (verified:
1.8.7 fails the `<1.6` cap; 1.5.5 keeps `arrayref`). The only true root fix is an
upstream `uor-prism-crypto`/`uor-addr` bump — which we are now treating as
indefinitely unavailable, so we resolve it ourselves.

## Fix — vendor, don't suppress

Vendor the **exact** `arrayref` 0.3.9 bytes into `third_party/arrayref` and source
them via `[patch.crates-io]`. cargo-deny's yanked check only applies to
**registry-sourced** crates, so pointing `arrayref` at a local path makes the
advisory **genuinely disappear** — the `deny.toml` ignore is removed, not relied
on. The bytes `blake3` links are byte-identical (`src/lib.rs` sha256
`b74872c9bb2b836132817e024a3f9205f83a6864de1a9bfb46acc1bfbbc1873a`), so κ / CID
output is unchanged: this is a re-sourcing, not a code change.

- `third_party/arrayref/`: verbatim 0.3.9 `src/lib.rs` + `LICENSE`, a library-only
  manifest, and a `README.md` documenting provenance (sha256s) and the removal
  condition. Excluded from the workspace, so it is not linted/tested as a member.
- `Cargo.toml`: `[patch.crates-io] arrayref = { path = "third_party/arrayref" }`
  plus a `[workspace].exclude` entry.
- `Cargo.lock`: `arrayref` re-sourced from the path (registry `source`/`checksum`
  gone).
- `deny.toml`: `[advisories].ignore` back to `[]` (the file's canonical
  "no ignores" state), with a breadcrumb to the vendored crate.

## Verification (local)

- `cargo deny check` → **advisories, bans, licenses, sources all ok** with **no
  ignore**.
- `cargo fmt --check`; `cargo clippy --workspace --all-targets --all-features --
  -D warnings`; `cargo build --target wasm32-unknown-unknown --lib` → green.
- Register conformance **R1–R6** green (`repo-conformance`, `xtask validate`,
  `cargo deny check bans`).
- κ / CID determinism, deterministic-rebuild, allocation-census, and fuzz-smoke
  tests green; `blake3` κ verified identical through the patched crate.
- Full `cargo test --workspace` re-run clean; the merge-queue ladder runs it
  authoritatively.

## Removal condition

Delete `third_party/arrayref`, the `[patch.crates-io] arrayref` entry, and the
`[workspace].exclude` entry the moment `uor-addr`/`uor-prism-crypto` move to a
`blake3` (>= 1.6) that no longer depends on `arrayref`.

Supersedes the interim ignore from #869. Refs #868.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

