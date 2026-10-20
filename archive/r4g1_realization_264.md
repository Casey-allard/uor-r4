# R4G1 representation realization

Issue #264 adds the first in-repository realization of an R4G1 artifact for
`uor-addr`. It separates two identities that serve different purposes:

- `GraphView::verify_cids()` continues to verify the exact bytes mapped by a
  runtime. Those wire CIDs are integrity checks and intentionally remain
  sensitive to layout and padding.
- `uor_r4_graph_format::r4g1::address()` produces the representation-level
  κ-label used in reports, manifests, and serving attestations. It ignores
  offsets, alignment padding, and the two wire CID fields.

The realization is a two-level Merkle skeleton encoded as deterministic CBOR
and addressed through the pinned `uor-addr` BLAKE3 axis:

```text
section skeleton := [version, "r4g1", section_id, section_flags, payload]
section κ       := uor-addr/cbor/blake3(section skeleton)

artifact skeleton := [version, "r4g1",
                      [[section_id, section_flags, section κ], ...]]
artifact κ       := uor-addr/cbor/blake3(artifact skeleton)
```

Sections are consumed in R4G1's canonical section-ID order. The section
address is therefore independently usable for future immutable graph patches
and region-object manifests, while the artifact address is stable across
equivalent container reserializations. A changed section payload or section
flag changes both its own κ-label and the artifact κ-label.

The implementation is allocation-gated with the graph-format crate's
`alloc` feature; the no-std parser and runtime feature ladder do not pull the
realization dependency into their zero-allocation build.

The ignored `uor_standards/` checkout is local reference material rather than
a repository input. The implementation is pinned to the same `uor-addr`
revision declared in the workspace manifest; upstream conformance vectors can
be added there when the standards repository accepts the realization.
