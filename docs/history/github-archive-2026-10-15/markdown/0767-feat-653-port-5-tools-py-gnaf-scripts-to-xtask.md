# #767 — feat(#653): port 5 Tools/*.py GNAF scripts to xtask

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-17T16:43:19Z
- **Updated:** 2026-08-17T18:08:11Z
- **Closed:** 2026-08-17T17:49:46Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/767
- **Merged:** 2026-08-17T17:49:46Z
- **Merge commit:** bc1f14411db3de853bc30c432904428a8f83da78
- **Base / Head:** main / UOR-Foundation:i653-tools-xtask
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Summary

`docs/gnaf_integration_653.md` (#653 phase-2 follow-up) recommends porting
each `Tools/*.py` check to an r4-native equivalent rather than vendoring
Python into the Rust workspace. This PR ports the 5 of 10 `Tools/*.py`
scripts that do not invoke `lean`/`lake` (directly or transitively) to
`cargo xtask` subcommands:

- `gnaf-firewall` -- SPEC 10.1 dependency firewall (`Tools/firewall.py`)
- `gnaf-root` -- SPEC 5 layer ownership + root-import regeneration (`Tools/root.py`)
- `gnaf-scan` -- SPEC 19 forbidden-construct scan (`Tools/scan.py`)
- `gnaf-release-path` -- SPEC 19/6.3 no-`noncomputable`-on-release-path check (`Tools/releasepath.py`)
- `gnaf-manifest` -- SPEC 4/5 MANIFEST.json identity-stage generator/checker (`Tools/manifest.py`)

The remaining 4 (`axioms.py`, `required.py`, `gate.py`, `mutation.py`) all
invoke the Lean toolchain and are out of scope for this port.

Each port was verified byte-for-byte against its Python counterpart on the
real vendored `proofs/wasm-gemm-gnaf` tree, not just against synthetic
fixtures.

### `gnaf-firewall`, `gnaf-root`, `gnaf-scan`

All three currently pass against the real tree and are wired into
`cargo xtask validate`.

### `gnaf-release-path`

Currently **fails** against the real tree -- and so does
`Tools/releasepath.py` itself, at the identical two locations
(`WasmGemmGnaf/Artifact/Release.lean:446` and `:456`). This is a known,
pre-existing gap gated on outstanding WGG-GO-1, not a bug in the port.
Deliberately **not** wired into `validate` (which must stay green);
documented in both the module doc and a `validate()` comment.

### `gnaf-manifest`

Needs byte-for-byte parity with Python's `json.dumps(sort_keys=True,
separators=(",", ":"))` (compact/canonical, for hashing) and
`json.dumps(indent=2)` (pretty, for the on-disk file), including Python's
default `ensure_ascii=True` escaping. `serde_json`'s serializer only
preserves key ordering, not `ensure_ascii` escaping, so this hand-rolls a
minimal canonical + pretty JSON writer instead. Verified via a cross-tool
round trip against a scratch copy of the real tree: Python-written
`MANIFEST.json` checked by `cargo xtask gnaf-manifest` agrees at all three
identity hashes (135 source files); Rust-written `MANIFEST.json` checked
by `python3 Tools/manifest.py --check` also agrees.

Like `gnaf-release-path`, this is **not** wired into `validate`: the real
committed `MANIFEST.json` is currently stale (predates recent tree
changes) per both the Python and Rust checkers. Regenerating it is out of
scope for this port and left to a follow-up.

## Test plan

- [x] `cargo test -p xtask gnaf` -- 23/23 pass
- [x] `cargo xtask validate` -- passes (`gnaf-firewall`, `gnaf-root`,
      `gnaf-scan` wired in; `gnaf-release-path`/`gnaf-manifest`
      deliberately excluded, both documented)
- [x] `cargo fmt --check` -- clean
- [x] `cargo clippy --workspace --all-targets --all-features --offline -- -D warnings` -- clean
- [x] `cargo test --workspace --offline` -- 1452 passed, 0 failed
- [x] Each of the 5 ports cross-checked against its `Tools/*.py`
      counterpart on the real vendored tree

Generated with local engineering work performed during a GitHub outage,
now being synced back now that GitHub is reachable again.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

