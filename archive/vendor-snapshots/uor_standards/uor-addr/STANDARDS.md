# Authoritative source references

UOR-ADDR's realizations each cite the authoritative source for the
standard the realization conforms to. This document is the single
index; the individual modules carry the same citation inline in
their docstrings.

## Common architectural layer

| Concern | Authoritative source |
|---|---|
| UOR-ADDR architecture | [`ARCHITECTURE.md`](ARCHITECTURE.md) |
| UOR-Framework wiki (normative substrate) | <https://github.com/UOR-Foundation/UOR-Framework/wiki> |
| ADR-001 (typed-iso surface) | <https://github.com/UOR-Foundation/UOR-Framework/wiki/ADR-001> |
| ADR-017 (canonical UOR-address mapping) | <https://github.com/UOR-Foundation/UOR-Framework/wiki/ADR-017> |
| ADR-020 (`PrismModel<H, B, A, R, C>`) | <https://github.com/UOR-Foundation/UOR-Framework/wiki/ADR-020> |
| ADR-023 (typed-iso input shape) | <https://github.com/UOR-Foundation/UOR-Framework/wiki/ADR-023> |
| ADR-031 (standard-library Layer-3 sub-crate) | <https://github.com/UOR-Foundation/UOR-Framework/wiki/ADR-031> |
| ADR-035 (canonical k-invariants branch) | <https://github.com/UOR-Foundation/UOR-Framework/wiki/ADR-035> |
| ADR-036 (`ResolverCategory` enumeration) | <https://github.com/UOR-Foundation/UOR-Framework/wiki/ADR-036> |
| ADR-037 (`HostBounds` capacity profile) | <https://github.com/UOR-Foundation/UOR-Framework/wiki/ADR-037> |
| ADR-046 (resolver-body discipline) | <https://github.com/UOR-Foundation/UOR-Framework/wiki/ADR-046> |
| ADR-047 (σ-Projection Hardening Principle) | <https://github.com/UOR-Foundation/UOR-Framework/wiki/ADR-047> |
| ADR-048 (`TypedCommitment` cost-model surface) | <https://github.com/UOR-Foundation/UOR-Framework/wiki/ADR-048> |
| ADR-049 (`axis::cryptanalyze` witness) | <https://github.com/UOR-Foundation/UOR-Framework/wiki/ADR-049> |
| ADR-054 (fold-fusion principle) | <https://github.com/UOR-Foundation/UOR-Framework/wiki/ADR-054> |
| ADR-057 (bounded recursive structural typing) | <https://github.com/UOR-Foundation/UOR-Framework/wiki/ADR-057> |
| ADR-059 (Atlas vertex-degree partition / S₄ orbit) | <https://github.com/UOR-Foundation/UOR-Framework/wiki/ADR-059> |
| ADR-060 (unbounded source-polymorphic carrier) | <https://github.com/UOR-Foundation/UOR-Framework/wiki/ADR-060> |
| ADR-061 (categorical composition on the E₈ Atlas image) | <https://github.com/UOR-Foundation/UOR-Framework/wiki/ADR-061> |
| Amendment 43 (ring element canonical bytes) | <https://github.com/UOR-Foundation/UOR-Framework/wiki/Amendment-43> |
| TC-05 (replay round-trip) | <https://github.com/UOR-Foundation/UOR-Framework/wiki/TC-05> |
| FIPS 180-4 (SHA-256) | <https://nvlpubs.nist.gov/nistpubs/FIPS/NIST.FIPS.180-4.pdf> |

## Hash σ-axes (`uor_addr::hash`)

Every realization derives its κ-label `<algorithm>:<lowercase-hex>` through
a selectable σ-axis. `address()` binds SHA-256 (the default);
`address_blake3` / `address_sha3_256` / `address_keccak256` bind the other
admissible 32-byte axes. Each is validated against vectors imported from its
authoritative source (`tests/hash_kat.rs`).

| Axis (`AddrHash`) | κ-label prefix | Authoritative source |
|---|---|---|
| `Sha256Hasher` | `sha256` | NIST FIPS 180-4 — <https://nvlpubs.nist.gov/nistpubs/FIPS/NIST.FIPS.180-4.pdf> |
| `Sha3_256Hasher` | `sha3-256` | NIST FIPS 202 — <https://nvlpubs.nist.gov/nistpubs/FIPS/NIST.FIPS.202.pdf> |
| `Keccak256Hasher` | `keccak256` | Keccak SHA-3 submission (pre-FIPS padding) — <https://keccak.team/files/Keccak-submission-3.pdf> |
| `Blake3Hasher` | `blake3` | BLAKE3 specification + reference vectors — <https://github.com/BLAKE3-team/BLAKE3-specs> |
| `Sha512Hasher` | `sha512` | NIST FIPS 180-4 §6.4 — <https://nvlpubs.nist.gov/nistpubs/FIPS/NIST.FIPS.180-4.pdf> |

The 64-byte `Sha512Hasher` (a `Hasher<64>`) is bound with the
`AddrBounds64` capacity profile and yields a 135-byte κ-label; it became
admissible once foundation 0.5.2 generalized the resolver tower over the
fingerprint-width const generic (prism 0.3.3 / foundation 0.5.2).

## Format-specific realizations

### CBOR realization (`uor_addr::cbor`)

| Concern | Authoritative source |
|---|---|
| CBOR data model + encoding | RFC 8949 — <https://www.rfc-editor.org/rfc/rfc8949> |
| Canonical form (Deterministic Encoding) | RFC 8949 §4.2 — <https://www.rfc-editor.org/rfc/rfc8949#section-4.2> |
| Preferred (shortest) serialization | RFC 8949 §4.1 / §4.2.2 |
| Encoding examples (test vectors) | RFC 8949 Appendix A — <https://www.rfc-editor.org/rfc/rfc8949#appendix-A> |

Conformance: the RFC 8949 §4.2 deterministic-encoding rules (shortest
integer/float heads, definite lengths, bytewise-sorted map keys, canonical
NaN) validated against Appendix A vectors. See
[crates/uor-addr/tests/cbor_rfc8949.rs](crates/uor-addr/tests/cbor_rfc8949.rs).

### JSON realization (`uor_addr::json`)

| Concern | Authoritative source |
|---|---|
| JSON syntax | RFC 8259 — <https://datatracker.ietf.org/doc/rfc8259/> |
| Canonical form (JCS) | RFC 8785 — <https://datatracker.ietf.org/doc/rfc8785/> |
| Unicode normalization (NFC) | UAX #15 — <https://www.unicode.org/reports/tr15/> |
| ECMA-262 numeric serialization | <https://datatracker.ietf.org/doc/html/rfc8785#section-3.2.2.3> |
| `mcp.uor.foundation/tools/encode_address` reference | <https://mcp.uor.foundation/tools/encode_address> |

Conformance corpus: 12-fixture `mcp.uor.foundation/tools/encode_address`
baseline, the 8-fixture Maura Clark reference baseline, the
JCS-RFC8785 published test vectors. See
[crates/uor-addr/tests/byte_identity.rs](crates/uor-addr/tests/byte_identity.rs),
[crates/uor-addr/tests/conformance.rs](crates/uor-addr/tests/conformance.rs),
[crates/uor-addr/tests/cross_validation.rs](crates/uor-addr/tests/cross_validation.rs).

### S-expression realization (`uor_addr::sexp`)

| Concern | Authoritative source |
|---|---|
| Canonical S-expressions (Rivest, 1997) | <https://people.csail.mit.edu/rivest/Sexp.txt> |
| I-D form (draft-rivest-sexp-00) | <https://datatracker.ietf.org/doc/html/draft-rivest-sexp-00> |
| RFC 2693 §3 ("Canonical S-Expressions") | <https://datatracker.ietf.org/doc/html/rfc2693#section-3> |
| SPKI test vectors (RFC 2693 §11) | <https://datatracker.ietf.org/doc/html/rfc2693#section-11> |

Conformance corpus: [crates/uor-addr/tests/sexp_conformance.rs](crates/uor-addr/tests/sexp_conformance.rs).

### XML realization (`uor_addr::xml`)

| Concern | Authoritative source |
|---|---|
| Canonical XML 1.1 | W3C REC-xml-c14n11 — <https://www.w3.org/TR/xml-c14n11/> |
| XML 1.0 base syntax | W3C REC-xml — <https://www.w3.org/TR/xml/> |

Conformance corpus: covered in
[crates/uor-addr/tests/all_realizations.rs](crates/uor-addr/tests/all_realizations.rs)
plus the [`uor_addr::xml::value::tests`](crates/uor-addr/src/xml/value.rs)
unit-test suite (lexicographic attribute ordering per §1.1 rule 3,
CDATA-to-Text expansion, attribute-value and text-content escape
rules per §1.1 rules 4–5, idempotence).

This realization implements a **subset** of XML-C14N 1.1 over the
typed `XmlValue` grammar's five cases (Element, Attribute, Text,
CDATA, ProcessingInstruction). Out-of-scope rules (namespace prefix
rewriting, DTD-internal entity resolution, document-level
processing instructions outside the root) are documented in the
[`uor_addr::xml`](crates/uor-addr/src/xml/mod.rs) module docstring
— they apply to deserialization from arbitrary XML 1.0 documents,
not to typed-input pipelines.

### ASN.1 realization (`uor_addr::asn1`)

| Concern | Authoritative source |
|---|---|
| ITU-T X.690 (BER / CER / DER) | <https://www.itu.int/rec/T-REC-X.690> |
| ITU-T X.680 (ASN.1 abstract notation) | <https://www.itu.int/rec/T-REC-X.680> |

Conformance corpus: [`uor_addr::asn1::value::tests`](crates/uor-addr/src/asn1/value.rs)
unit tests pin X.690 §8.2.2 / §8.3 / §8.8 / §10.1 / §11.1
encoding rules (canonical Boolean, minimum-octets Integer, Null
zero-length, no long-form length under 128, no indefinite length).
Cross-realization coverage in
[crates/uor-addr/tests/all_realizations.rs](crates/uor-addr/tests/all_realizations.rs).

Supported universal-tag cases: Boolean, Integer, OctetString,
Null, Sequence. Additional tags (BitString, ObjectIdentifier,
PrintableString, IA5String, UTCTime, GeneralizedTime, Set, …)
extend the typed-input shape per X.690 / X.680; their encoding
follows the same DER discipline this module pins.

### Ring realization (`uor_addr::ring`)

| Concern | Authoritative source |
|---|---|
| Amendment 43 §2 canonical-bytes layout | <https://github.com/UOR-Foundation/UOR-Framework/wiki/Amendment-43> |
| ADR-039 ring-algebra surface | <https://github.com/UOR-Foundation/UOR-Framework/wiki/ADR-039> |

Conformance corpus: [`uor_addr::ring::value::tests`](crates/uor-addr/src/ring/value.rs)
pins the `header(k) || le_bytes(x, k+1)` layout, Witt-level bound
enforcement, and the canonicalizer's identity property (Amendment
43 pins canonical bytes at construction). Cross-realization
coverage in
[crates/uor-addr/tests/all_realizations.rs](crates/uor-addr/tests/all_realizations.rs).

### Code-module AST realization (`uor_addr::codemodule`)

| Concern | Authoritative source |
|---|---|
| Canonical Code-Module AST Serialization (CCMAS) | [`uor_addr::codemodule`](crates/uor-addr/src/codemodule/mod.rs) module docstring (normative inline) |
| Underlying canonical S-expression form | Rivest 1997 — <https://people.csail.mit.edu/rivest/Sexp.txt> |

The CCMAS grammar extends Rivest canonical S-expressions with
AST-shaped term constructors (`(3:mod …)`, `(3:fun …)`,
`(3:type …)`, `(3:const …)`, atom literals/identifiers). The
canonical-form output is byte-identical to
[`crate::sexp::canonicalize`] applied to the CCMAS surface AST.

Conformance corpus:
[`uor_addr::codemodule::value::tests`](crates/uor-addr/src/codemodule/value.rs)
pins the grammar's round-trip property and the CCMAS-as-Rivest-subset
relation.

## Schema-pinned descendants

UOR-ADDR's schema-import discipline: well-known kinds and types
map to **existing standards** rather than UOR-native inventions.
The shipped schemas therefore import from published taxonomies.

### Photo schema (`uor_addr::schema::photo`)

Schema-pinned descendant of [`uor_addr::json`]. **Imports
schema.org's `Photograph` type.**

| Concern | Authoritative source |
|---|---|
| schema.org Photograph | <https://schema.org/Photograph> |
| schema.org ImageObject (parent) | <https://schema.org/ImageObject> |
| schema.org MediaObject (parent) | <https://schema.org/MediaObject> |
| schema.org CreativeWork (parent) | <https://schema.org/CreativeWork> |
| JSON-LD 1.1 | <https://www.w3.org/TR/json-ld11/> |

Required `@type = "Photograph"`, `@context ∈ {https://schema.org, http://schema.org}`,
`contentUrl`, `creator`. Conformance corpus:
[`crates/uor-addr/src/schema/photo.rs`](crates/uor-addr/src/schema/photo.rs).

### Document schema (`uor_addr::schema::document`)

Schema-pinned descendant of [`uor_addr::json`]. **Imports
schema.org's `Article` type** (extending `CreativeWork`).

| Concern | Authoritative source |
|---|---|
| schema.org Article | <https://schema.org/Article> |
| schema.org Article subtypes (NewsArticle, ScholarlyArticle, BlogPosting, …) | <https://schema.org/Article#subtypes> |
| schema.org CreativeWork (parent) | <https://schema.org/CreativeWork> |

Required `@context = schema.org`, `@type ∈ {Article, NewsArticle, Report,
ScholarlyArticle, SocialMediaPosting, TechArticle, BlogPosting, …}` (15
admissible subtypes), `headline`, `author`, `datePublished`.
Conformance corpus:
[`crates/uor-addr/src/schema/document.rs`](crates/uor-addr/src/schema/document.rs).

### Signed code-module schema (`uor_addr::schema::codemodule_signed`)

Schema-pinned descendant of [`uor_addr::json`]. **Imports the
industry-standard in-toto Statement v1 attestation envelope** used
by sigstore, SLSA, and the broader software-supply-chain
attestation ecosystem.

| Concern | Authoritative source |
|---|---|
| in-toto Statement v1 | <https://in-toto.io/Statement/v1> |
| in-toto Attestation Framework v1 | <https://github.com/in-toto/attestation/blob/main/spec/v1/README.md> |
| in-toto Statement v1 spec | <https://github.com/in-toto/attestation/blob/main/spec/v1/statement.md> |
| SLSA Provenance v1 (common predicate) | <https://slsa.dev/spec/v1.0/provenance> |
| sigstore signature spec | <https://docs.sigstore.dev/cosign/signature_specification/> |

Required `_type = "https://in-toto.io/Statement/v1"`, non-empty
`subject[]` with `name` + `digest.sha256` (64 lowercase-hex chars),
`predicateType`, `predicate` (object). Conformance corpus:
[`crates/uor-addr/src/schema/codemodule_signed.rs`](crates/uor-addr/src/schema/codemodule_signed.rs).

## Cost-model-bearing variants

### Storage variant (`uor_addr::variant::storage`)

| Concern | Authoritative source |
|---|---|
| ADR-048 typed-commitment surface | <https://github.com/UOR-Foundation/UOR-Framework/wiki/ADR-048> |
| ADR-047 U6 bandwidth-additivity | <https://github.com/UOR-Foundation/UOR-Framework/wiki/ADR-047> |
| QS-06 storage-tier admission exemplar | <https://github.com/UOR-Foundation/UOR-Framework/wiki/QS-06> |

Binds `C = AndCommitment<EmptyCommitment, SingletonCommitment<LexicographicLessEqThreshold>>`.
Conformance:
[crates/uor-addr/tests/variant_storage.rs](crates/uor-addr/tests/variant_storage.rs).

### Signed variant (`uor_addr::variant::signed`)

| Concern | Authoritative source |
|---|---|
| ADR-048 typed-commitment surface | <https://github.com/UOR-Foundation/UOR-Framework/wiki/ADR-048> |
| ADR-049 `axis::cryptanalyze` witness | <https://github.com/UOR-Foundation/UOR-Framework/wiki/ADR-049> |

Binds `C = SingletonCommitment<UltrametricCloseTo<2>>`. The
architectural commitment per ARCHITECTURE.md is a
`SignatureCommitmentPredicate`; the foundation's
`ObservablePredicate` trait is sealed, so this variant binds the
closest standing predicate from `prism::pipeline`'s published
roster (the 2-adic ultrametric proximity predicate) that fits
the signature-admission-shape semantics per ADR-049. When
`prism::pipeline` publishes a `SignatureCommitmentPredicate`
primitive, this module retargets without changing the architectural
surface.

Conformance:
[`uor_addr::variant::signed::tests`](crates/uor-addr/src/variant/signed.rs)
plus
[crates/uor-addr/tests/all_realizations.rs](crates/uor-addr/tests/all_realizations.rs).

## Categorical composition (`uor_addr::composition`)

The five categorical operations on the Atlas image inside E₈ per
ADR-061. The framework (ADR-061 §(3)) names each operation's algebraic
structure; the realization (per CA-5) commits the specific byte-level
canonicalize discipline that implements it. Each composes operand
κ-labels into a new κ-label on the same σ-axis (CA-3 σ-axis
homogeneity).

| Operation | Algebraic structure (ADR-061 §(3) / ADR-059) | Realization commitment (canonical form) |
|---|---|---|
| CS-G2 `compose_g2_product` | commutative binary product | lex-min-first concatenation `lo ‖ hi` (2N bytes) |
| CS-F4 `compose_f4_quotient` | 2-element ± involution equivalence | lex-min of `{raw, ~raw}`, re-emitted as `<axis>:<hex>` (N bytes) |
| CS-E6 `compose_e6_filtration` | 2-class degree partition, 8:1 population (ADR-059 64:8) | `[degree_tag] ‖ operand`, tag = `first_byte mod 9 → {0x05,0x06}` (N+1 bytes) |
| CS-E7 `compose_e7_augmentation` | 24-element S₄ quarter-permutation orbit | lex-min of the 24-element orbit over 4 digest quarters (N bytes) |
| CS-E8 `compose_e8_embedding` | identity (each operand its own class) | identity on canonical-form bytes; distinguished by realization-IRI provenance (N bytes) |

| Concern | Authoritative source |
|---|---|
| ADR-061 categorical composition | <https://github.com/UOR-Foundation/UOR-Framework/wiki/ADR-061> |
| ADR-059 vertex-degree partition + S₄ orbit | <https://github.com/UOR-Foundation/UOR-Framework/wiki/ADR-059> |

Conformance:
[crates/uor-addr/tests/composition.rs](crates/uor-addr/tests/composition.rs);
example [crates/uor-addr/examples/composition.rs](crates/uor-addr/examples/composition.rs);
Lean width / orbit / partition / commutativity proofs in
[uor-addr-lean/UorAddr/CompositionLaws.lean](uor-addr-lean/UorAddr/CompositionLaws.lean).
Exposed across every binding: C
(`uor_addr_compose_{g2,f4,e6,e7,e8}[_with_witness]`), Python
(`kappa.compose_*`), and the WASM Component Model / npm
(`compose-g2` … / `composeG2` …).

## GGUF realization (`uor_addr::gguf`)

| Concern | Authoritative source |
|---|---|
| GGUF v3 binary format | https://github.com/ggml-org/ggml/blob/master/docs/gguf.md |
| Reference C++ header | https://github.com/ggml-org/ggml/blob/master/include/gguf.h |
| Reference Python tooling | https://github.com/ggml-org/llama.cpp/tree/master/gguf-py |
| GGML type enum (per-dtype IDs) | https://github.com/ggml-org/ggml/blob/master/include/ggml.h |
| Tensor element-type alphabet | `prism::tensor::dtype` (uor-prism-tensor 0.2.0) |
| Canonical form encoder (executable spec) | `tools/canonical-gguf.py` |
| SHA-256 σ-projection | FIPS 180-4 |

The κ-label is SHA-256 over the **full flat Merkle skeleton**
(`CANONICAL_FORM_VERSION = 2`): the header (`magic ‖ version ‖
tensor_count ‖ kv_count ‖ alignment`), then the metadata KVs sorted by
key bytes, then the tensor-info records sorted by name bytes. Each
variable-length leaf — a string, an array payload, a tensor's data
region — is replaced by its streamed SHA-256 digest, so the skeleton's
size grows only with the KV / tensor counts (never with model size)
while binding every weight byte. Under ADR-060 the full skeleton flows
through the pipeline as a `Borrowed` carrier and ψ₉ folds it; there is
no two-level commitment and no count / width cap. The exact layout is in
the [`gguf::value`](crates/uor-addr/src/gguf/value.rs) module header.
`tools/canonical-gguf.py` is the executable form of this
canonicalization; CL-GGUF asserts byte-identity against it.

## ONNX realization (`uor_addr::onnx`)

| Concern | Authoritative source |
|---|---|
| ONNX protobuf schema | https://github.com/onnx/onnx/blob/main/onnx/onnx.proto |
| ONNX IR specification | https://github.com/onnx/onnx/blob/main/docs/IR.md |
| ONNX versioning | https://github.com/onnx/onnx/blob/main/docs/Versioning.md |
| Protobuf v3 wire format | https://protobuf.dev/programming-guides/encoding/ |
| Tensor element-type alphabet | `prism::tensor::dtype` (uor-prism-tensor 0.2.0) |
| Canonical form encoder (executable spec) | `tools/canonical-onnx.py` |
| SHA-256 σ-projection | FIPS 180-4 |

Any known IR revision is admitted — `ir_version` in
`1..=ONNX_IR_VERSION_MAX` (= 13, the latest in `onnx.proto`; real-world
exports are predominantly IR 6–10). The skeleton is IR-version-agnostic
and binds the `ir_version` value, so distinct revisions of the same
logical model address distinctly.

The κ-label is SHA-256 over the **full flat skeleton**:
`LE_i64(ir_version)`, then the opset imports sorted by `(domain,
version)`, then the graph emitted recursively, then the model metadata.
The graph orders nodes by Kahn topological sort with lexicographic
`(name, op_type, domain)` tie-break, sorts initializers / IO by name,
reduces typed-data fields to the canonical `raw_data` layout, and
recurses into `GRAPH` / `GRAPHS` subgraphs inline (depth-bounded by a
stack-safety guard, `ONNX_SUBGRAPH_DEPTH_MAX = 64`). Variable-length
leaves (tensor data, strings, opaque sub-message payloads) are replaced
by their `sha256(...)` digest. Under ADR-060 the full skeleton flows
through the pipeline as a `Borrowed` carrier and ψ₉ folds it; there is
no two-level commitment and no count cap. The exact layout is in the
[`onnx::value`](crates/uor-addr/src/onnx/value.rs) module header.
`tools/canonical-onnx.py` is the executable form; CL-ONNX asserts
byte-identity against it.
