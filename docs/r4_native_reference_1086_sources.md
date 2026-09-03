# Original-source and ownership audit for the #1086 native reference contract

**2026-09-03 — `SOURCE_INSPECTED_NOT_EXECUTED`.** This audit is based on
`UOR-Foundation/uor-r4` revision
`eade29f4b78435e9857936786426bb34e596b301`. It informs the export/loader
specification after the accepted [#1094 raw-text comparison](r4_retained_comparison_1094.md).
It adds no native implementation, artifact, model observation, mathematical
proof or execution authorization. Upstream sources are design evidence only.

The public project index was queried for typed identity, UOR source audit,
NEMESIS/W33, carrying substrates, `kappa_from_value` and `compose_model`.
The original `kappa-registry` and `hologram` source records were retrieved,
alongside the earlier [#1085 source audit](integration/clause-segmentation-1085-sources.md).
Their source IDs, revisions and body hashes are recorded in the
[source manifest](r4_native_reference_1086_source_manifest.json). Indexed
historical decisions are retrieval aids; the current revision and live issue
graph govern task selection.

The pinned original files were read directly from the retained public source
cache and checked against their Git-tree blob identities. The NEMESIS PDF was
rehashed and its retained, independently hashed page-marked extraction was read
on pages 1–3. Five small license/workspace files were freshly fetched at the
same pinned revisions and checked against the retained trees. The manifest
contains **24 original-file identities**, inspected locations, byte lengths,
SHA256 digests and Git blob IDs; it does not contain copied upstream source.
No upstream code or witness was executed, and no dependency pin was changed.

## Original research relevant to this boundary

| Original source | Inspected support | Consequence for #1086 and limit |
|---|---|---|
| [NEMESIS carrying report](https://github.com/markrnd87-cmd/NEMESIS-Theory/blob/0d106967843c2c96477cf3e57aeff213e7db1c97/Technical%20Report_%20Integration%20of%20Hypercomplex%20Geometries%20as%20UOR%20Structure%20Carrying%20Substrates.pdf), Mark / NEMESIS 3D Studio, pp. 1–3; revision `0d106967843c2c96477cf3e57aeff213e7db1c97`; SHA256 `697d48b70a1499a1fd70d8f1a4c285606a198a3831250425ae11439f37b395cc` | Separates a map of state spaces, preservation of transitions and interpretation by declared primitives. | Name the exact decoded state and operation semantics that the exported bytes are intended to preserve. These proposed carrying criteria do not establish a bijection, commuting implementation, CPU cost bound, zero error or energy claim for the accepted reader/core. The report's stronger assertions are not adopted as proof or native behavior evidence. |
| [W33 runtime](https://github.com/wilcompute/W33-Theory/blob/5674aa2e74c7de81864556b1bbc90aa9a1b1bd3d/analysis/w33_fractal_microvm_runtime.py#L38), lines 38–58 and 314–329; revision `5674aa2e74c7de81864556b1bbc90aa9a1b1bd3d`; SHA256 `875b53408cc5312b60b5a6254dbac80a9a1324c89cdf24936488d7a4744e90ca` | Checks an explicit instruction vocabulary, serializes objects with a declared JSON convention, and stores payloads under digests without replacing existing entries. | Explicit admission, canonical byte production and retained receipts are useful boundary patterns. This microVM supplies no reader/core operator implementation, tensor codec or native parity evidence. Its content-store convention does not prove loader correctness or complete provenance elsewhere. |
| [UOR-ADDR JSON realization](https://github.com/UOR-Foundation/uor-addr/blob/165b51e3e2113ee5d032730cde709335d4fe9b60/crates/uor-addr/src/json/mod.rs#L23), lines 1–36 and 57–67; revision `165b51e3e2113ee5d032730cde709335d4fe9b60`; SHA256 `39bc9fcb5f83ef2408e752a1eb775c984f40068791119d38df8acd39b14289f1` | Its declared realization combines JCS-RFC8785 with Unicode NFC and offers explicit hash-axis entry points. | A canonical manifest label and a digest of its originally supplied bytes have different meanings. State the realization and version before claiming identity; canonicalization must not silently alter the raw-text bytes or spans carried from #1094. |
| [UOR-ADDR composition](https://github.com/UOR-Foundation/uor-addr/blob/165b51e3e2113ee5d032730cde709335d4fe9b60/crates/uor-addr/src/composition/canonicalize.rs#L68), lines 68–118; same revision; SHA256 `d9032fc9bc95a4f86ddbb8c0db3753865ee6de36a8f915fc446597243b8a6d89` | Checks digest-axis compatibility and realizes a commutative binary G2 product by lexicographic operand ordering. | Sorting unqualified component digests loses argument position. Preserve tensor names, model roles, ordered axes, clause/token order and topology inside the hashed representation. No composition or geometry theorem for this model is supplied. |

## Identity must state what was hashed

**Definition recommendation.** An identity record needs a kind, hash axis,
schema/realization version, complete framing rule, byte length where applicable
and digest. The following identities answer different questions and must remain
separately typed:

| Identity | Required content and meaning |
|---|---|
| Artifact bytes | The exact native container bytes under its explicitly defined digest coverage. A source checkpoint digest remains a separate provenance field. If the container contains a digest of itself, define the excluded or zeroed region; do not inherit another format's rule accidentally. |
| Decoded model state | Ordered records carrying component role, tensor/buffer name, dtype, rank, ordered dimensions, byte order and exact payload. Include behavior-relevant nonparameter buffers and fixed control/configuration identities. Equal raw float bytes, numeric equality and equality after canonical normalization are different predicates. |
| Codec realization | Container/schema and codec versions, tensor ordering, alignment, byte order, shape/layout convention, and the treatment of exceptional float encodings. A changed lossless codec may produce different artifact bytes while preserving decoded state. No quantization, rounding or learned transformation is implied by export. |
| Derivation provenance | Source implementation revision, source checkpoint/state identities, exporter identity, ordered dependencies and complete scalar-parameter framing. This identifies how a candidate was produced; it does not substitute for hashing the produced bytes. |
| Execution/reference evidence | Comparator and native implementation identities, arithmetic contract, exact selected input identity, output/intermediate receipts and result. Neither container integrity nor model-state identity alone establishes output preservation. |

The distinction is visible in original implementations:

- [`kappa_from_bytes` and `kappa_from_value`](https://github.com/UOR-Foundation/kappa-registry/blob/2af86560a177fc9651b6c0e92e7974140ed77dd5/crates/kappa-core/src/kappa/compute.rs#L127)
  hash opaque bytes and dCBOR-encoded values respectively. They cannot be
  exchanged merely because both return a label. The original source was
  retrieved from the index as
  `kb:6b8e284c02461ae77686686561ce481cea3ff3017b300af2c9d17f6da1108430`
  and checked against the pinned local file.
- The [`uor-matmul` codec manifest](https://github.com/UOR-Foundation/uor-matmul/blob/3cc5882f210667f9ac00fd8c02c5b5957b493f5d/crates/uor-matmul-codec/src/kappa.rs#L28)
  binds its schema and code/codebook digests; its comments explicitly separate
  equal decodes from identical artifact encodings. This is a design reference
  at an upstream revision, **not** an adoption or upgrade. R4's dependency
  remains `b13c98449948174f590e337c4dc25dfc394a07d0`.
- [`hologram::derive_label`](https://github.com/Hologram-Technologies/hologram/blob/94ecb886811115491a77c8229e494965bea03fc2/crates/hologram-archive/src/address.rs#L88)
  hashes an operation, parameters and ordered operand labels without hashing
  result bytes. Its validity as a memo key depends on deterministic operation
  semantics and complete framing. A #1086 record must bind those semantics and
  frame variable lengths/counts explicitly instead of importing a general
  result-identity claim. The original source was retrieved as
  `kb:69220a35a42f227a256c00ae70ad723f7b29880e61221eb8e876031a08176f02`.
- The archive's [`compose_model`](https://github.com/Hologram-Technologies/hologram/blob/94ecb886811115491a77c8229e494965bea03fc2/crates/hologram-archive/src/address.rs#L150)
  performs an unsorted left fold. Binary commutativity alone does not establish
  associativity or invariance under arbitrary permutations of a longer fold.
  [`hologram-ai`](https://github.com/Hologram-Technologies/hologram-ai/blob/d5337ec2b3289fc8462abac2e627d165342079b2/crates/hologram-ai/src/address.rs#L117)
  explicitly sorts first to define an order-independent component collection.
  Neither convention alone identifies the ordered wiring of this reader/core.

These are source observations and specification requirements. Digest matching
is an integrity comparison under the declared hash convention, not a
mathematical injectivity proof or proof of neural behavior.

## Rust ownership and format boundary

At the R4 revision named above, the
[`uor-r4-api` README](https://github.com/UOR-Foundation/uor-r4/blob/eade29f4b78435e9857936786426bb34e596b301/crates/uor-r4-api/README.md#L3)
explicitly identifies that crate as the preserved teacher-compiled R4G1 facade.
[`EngineParts`](https://github.com/UOR-Foundation/uor-r4/blob/eade29f4b78435e9857936786426bb34e596b301/crates/uor-r4-api/src/engine.rs#L256)
contains a graph, teacher signature artifact, optional tokenizer and score
report. [`R4Engine::load`](https://github.com/UOR-Foundation/uor-r4/blob/eade29f4b78435e9857936786426bb34e596b301/crates/uor-r4-api/src/engine.rs#L2148)
uses graph structural validation and CID verification before constructing its
scorer. That is a useful loading pattern, not an implementation of the
accepted #1077 reader/#1073 core/#1079 R4 transport path.

The [graph-format crate](https://github.com/UOR-Foundation/uor-r4/blob/eade29f4b78435e9857936786426bb34e596b301/crates/uor-r4-graph-format/src/lib.rs#L1)
owns graph-specific sections and a specific hash coverage rule, including
`artifact_bytes[56..total_len]`. The accepted learned state has no established
mapping to those sections. Reusing the `R4G1` magic, `GraphView` or its
`ArtifactCid` meaning would require a separate versioned correspondence
decision; this specification supplies none.

**Design recommendation.** Give the native reference path a separately named
module and artifact type under the current core/reference implementation
boundary. Keep caller-owned bytes and validation before state construction;
return focused load errors. Let the exporter and file access remain outside
that bytes-only library boundary. Use a distinct schema/magic and explicit
reference arithmetic identity. Do not route the new path through the legacy
teacher graph engine or advertise it as the replacement serving facade.
Reference floating-point execution, if selected by the contract, does not
qualify the separate multiplication-free deployed kernel. The final module
name and format details belong to the #1086 specification, not this audit.

## Licensing, proof and retained negative results

The pinned NEMESIS inventory contains no license; authorship and original links
are retained, and no material was vendored. W33's actual LICENSE is MIT.
UOR-ADDR's actual LICENSE and workspace declaration are Apache-2.0. Hologram
has MIT and Apache license files matching its dual declaration. The inspected
`uor-matmul` LICENSE is MIT while its Cargo workspace declares Apache-2.0;
that discrepancy remains recorded. `kappa-registry` declares
`MIT OR Apache-2.0` in Cargo but has no root license file in the inspected tree.
This audit copies none of their code and does not select a new dependency.

The accepted #1094 result remains bounded raw-text preservation on the fixed
grammar and already-observed semantic groups. Its consumed envelope is not
reopened. Its original `UNAVAILABLE_REFERENCE_REPLAY` preparation remains
historical evidence. #1079 remains `LANGUAGE_R4_PRESERVED_CONTROL_WEAK`, and
#1082 remains `TOKEN_EXPOSURE_DESCRIPTIVE_COMPLETE`; neither is reinterpreted
as a causal segmentation diagnosis. NEMESIS/W33/UOR source inspection supplies
no missing causal intervention or generalization evidence.

**Status separation.** Source and byte-identity inspection are complete for
this audit. Native export, native load, reference/native comparisons, fresh
native replay and kernel qualification are **`NOT_RUN`**. There is no new proof
artifact. No model checkpoint, runtime asset or withheld payload was opened,
and no upstream witness, model computation, build or dependency setup ran.
