# κ-addressed region objects

Issue #263 adds an I/O-side export and resolver layer for the graded TLS1
store. The existing monolithic `Store` and prediction kernel remain the
reference implementation; this layer makes each `(depth, prefix)` entry a
portable object.

## Object and manifest identities

Each region object is encoded as deterministic CBOR:

```text
[schema, depth, prefix_bytes, [[token_id, evidence_count], ...]]
```

The token distribution is emitted in ascending token-ID order. Its κ-label is
the pinned `uor-addr` CBOR realization on the BLAKE3 axis. A manifest contains
the sorted `(depth, prefix, κ, byte_length)` references and has its own κ-label
over a skeleton that excludes the self-reference.

`ModelStore` uses the existing local CAS layout for these labels. Writes emit
canonical bytes; reads reject malformed, non-canonical, or κ-mismatched
objects. The `RegionResolver` trait makes a local CAS resolver interchangeable
with a future network resolver.

## Prediction boundary

`predict_witness_with_resolver` walks the manifest's deepest available prefix,
fetches only the selected region object, and applies the same deterministic
token argmax rule as the monolithic TLS1 path. The conformance test starts with
only the manifest and resolver-backed objects and asserts byte-for-byte
equivalent prediction witnesses. No graph-runtime kernel or TLS1 wire format
changes are required.
