# Showcase compatibility fixtures

These are byte/data-structure fixtures, not token/model inputs or model
instruments. The Rust tests generate source bytes, authenticate the retained
known answers, and exercise the object store. They perform no GPU or model work
and make no language-quality, learning, physical-network or energy claim.

## Pinned sources

| Source | SHA-256 |
|---|---|
| Owner-supplied `SpiralCore_Showcase_Local_App.zip` | `6399240e51ad5f5347390690ce45f460eda390bbe1d9c5e86dee0e8fd5c26754` |
| ZIP member `SpiralCore_Showcase/core.js` | `7ca3dcfb552ca6ff3fc8696734265bcd9b993aa8b8e6242984060ace8b35bc22` |
| ZIP member `SpiralCore_Showcase/transport.js` | `1d254a75dfe497f91b302ada0d581eab3407c7ea8f3994920a58d9a2652dc0fe` |
| Unmodified ZIP member `python_vectors.json`, retained here | `0930044db928f0389dfa35e603812e4c765e0c8c7ced3182f6616adf31299031` |
| Independent review's `construction.json`, retained here | `644351faf3eb22b91f742bf30bcdb650793d2ef477ddf4f5c0b2669e43c1bcce` |
| Original retained `make-fixtures.cjs` adapter | `4b68ec4fdc535445d153649881a555d73ce133ca5c08f6941c508e163d02a3e3` |

The archive was verified again when extracting the unmodified vector file. Its
name records the supplied fixture's historical provenance; no Python runtime is
used by these tests. The test executes Rust only and has no dependency on the
owner's Downloads directory or an external executable.

The retained adapter is
`docs/labs/spiralcore-native-attention-review-2026-10-14/evidence/cs/make-fixtures.cjs`.
The independently reproduced construction record is
`docs/labs/spiralcore-attention-independent-review-2026-10-14/evidence/cs/construction.json`.
The four deterministic source-byte recipes in `showcase.rs` reproduce that
adapter: SHA-256-derived unique atoms; three repeated 32-byte atoms; UTF-8 with
embedded and trailing NULs; and the empty object. Each recipe is checked against
the retained input hash before its object-store output is checked.

## Ordering and exactness

The original `transport.js::recordsFor` emits unique records in root-ordered,
depth-first, children-before-parent order. The object store's export is sorted
by record identity. The tests parse the sorted export and independently reorder
the records using the manifest's root occurrences and each parent payload's
three child identities, then compare the resulting bytes' SHA-256 with the
original retained record hash. This checks exact record compatibility without
mistaking a deliberate ordering difference for a serialization mismatch.

The supplied fold vectors independently pin roots, object SHA-256 and distinct
record counts across partial atoms and ternary carry boundaries. Their larger
reference/novel fixtures are reproduced from the recipes in the supplied
`test_core.cjs`. Partial final atoms remain zero-padded in their stored leaf
record, while `rebuild` must return exactly the original byte length.

Regenerate and verify all fixtures with:

```sh
CARGO_TARGET_DIR=local/object-store-target cargo test -p uor-r4-object-store --test showcase --offline
```

The JSON known answers are immutable inputs, not regenerated from the new
implementation. A change in expected hashes requires an explicit source-format
decision and independently generated reference evidence.
