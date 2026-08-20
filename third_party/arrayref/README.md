# Vendored `arrayref` 0.3.9

This is an in-tree, byte-identical copy of the crates.io release
[`arrayref` 0.3.9](https://crates.io/crates/arrayref/0.3.9), used via a
`[patch.crates-io]` entry in the workspace root `Cargo.toml`.

## Why it is here

`arrayref` is pulled only transitively, by `blake3` 1.5.x (`blake3` declares
`arrayref = "0.3.5"`). The entire `arrayref` 0.3.x line (0.3.5–0.3.9) was
**yanked** upstream — a maintainer yank, **not** a security advisory (there is
no RUSTSEC entry; `cargo deny` reports it as `yanked`, not `vulnerability`).

`blake3` cannot move off 1.5.x in this graph: the pinned `uor-addr` rev's
`uor-prism-crypto v0.4.0` requires `blake3 >=1.5, <1.6`, and every `blake3`
1.5.x still bundles `arrayref` (`blake3` drops it only at >= 1.6). So the graph
is stuck on a yanked-but-benign crate for reasons entirely outside this repo.

Rather than suppress the advisory with a `deny.toml` ignore (which outlives the
dependency and trains readers to skip the place a real warning would appear),
we **own the code**: the exact 0.3.9 bytes are vendored here and sourced from
the local path. `cargo deny`'s yanked check only applies to registry-sourced
crates, so the advisory genuinely goes away — nothing is suppressed — and the
bytes `blake3` links are unchanged, so κ / CID behaviour is bit-identical.

## Provenance (verify before trusting)

    src/lib.rs  sha256 = b74872c9bb2b836132817e024a3f9205f83a6864de1a9bfb46acc1bfbbc1873a
    LICENSE     sha256 = 1bc7e6f475b3ec99b7e2643411950ae2368c250dd4c5c325f80f9811362a94a1

Both match crates.io `arrayref` 0.3.9 exactly. `Cargo.toml` here is trimmed to a
library-only build (no dev-dependencies, no examples); `src/lib.rs` is verbatim,
including its `#[cfg(test)]` module (not compiled in dependency builds).

## Removal condition

Delete this directory and the `arrayref` line from the root `[patch.crates-io]`
(and the `third_party/arrayref` entry from `[workspace].exclude`) the moment the
`uor-addr` / `uor-prism-crypto` chain moves to a `blake3` (>= 1.6) that no longer
depends on `arrayref`. Tracked in issue #868.
