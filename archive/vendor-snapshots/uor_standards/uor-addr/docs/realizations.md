# Picking a realization

Each realization plugs an existing canonical-form specification into
UOR-ADDR's ψ-pipeline. The choice is determined by what your data
already is, not by what you want it to be.

## Decision matrix

| Your data… | Use | Why |
|---|---|---|
| JSON / JSON-LD payloads | [`uor_addr::json`](../crates/uor-addr/src/json/) | Canonical form per RFC 8785 JCS (industry standard for canonical JSON, used by JOSE / COSE / signed JWTs). |
| S-expressions / SPKI-style data | [`uor_addr::sexp`](../crates/uor-addr/src/sexp/) | Rivest 1997 canonical form (the same form RFC 2693 SPKI uses). |
| XML documents (no namespaces, no DTDs) | [`uor_addr::xml`](../crates/uor-addr/src/xml/) | Subset of W3C XML-C14N 1.1. |
| ASN.1 structures (X.509, PKCS#7, CMS, SPKI) | [`uor_addr::asn1`](../crates/uor-addr/src/asn1/) | DER per ITU-T X.690 — the standard for digital-certificate and signed-payload encoding. |
| Ring elements from the UOR-Framework algebra | [`uor_addr::ring`](../crates/uor-addr/src/ring/) | Amendment 43 §2 canonical bytes — UOR-Framework's typed-input layout. |
| AST of a code module | [`uor_addr::codemodule`](../crates/uor-addr/src/codemodule/) | CCMAS canonical AST grammar — UOR-native, no upstream language standard exists. |
| Photo metadata | [`uor_addr::schema::photo`](../crates/uor-addr/src/schema/photo.rs) | Imports schema.org/Photograph (extending ImageObject → MediaObject → CreativeWork). |
| Article / document metadata | [`uor_addr::schema::document`](../crates/uor-addr/src/schema/document.rs) | Imports schema.org/Article + 14 published subtypes. |
| Signed-software attestation | [`uor_addr::schema::codemodule_signed`](../crates/uor-addr/src/schema/codemodule_signed.rs) | Imports in-toto Statement v1 (used by sigstore, SLSA). |
| JSON with K-bit storage admission | [`uor_addr::variant::storage`](../crates/uor-addr/src/variant/storage.rs) | Same κ-label, plus a `TypedCommitment` cost-model surface for storage-tier admission. |
| JSON with signature commitment | [`uor_addr::variant::signed`](../crates/uor-addr/src/variant/signed.rs) | Same κ-label, plus a `TypedCommitment` for signature-shape admission. |

## What's the difference between a format and a schema?

A **format** describes byte-level syntax: JSON, XML, ASN.1.
Format-specific realizations canonicalize bytes using the format's
published canonical-form rules.

A **schema** describes semantic structure on top of a format: a
schema.org/Photograph is a specific shape of JSON-LD. Schema-pinned
descendants validate the structure at parse time, then route to the
underlying format's canonicalizer. The κ-label produced by
`schema::photo::address(bytes)` is byte-identical to
`json::address(bytes)` for the same admitted input.

A **cost-model variant** carries a `TypedCommitment` (ADR-048) — a
typed predicate over the κ-label's digest. The κ-label is unchanged;
the commitment names a property the κ-label must satisfy (e.g. an
admission bound on a storage tier).

## Consuming UOR-ADDR from other languages

Two non-Rust distribution targets ship the same κ-label byte-for-byte:

- **C ABI** — [`uor-addr-c`](../crates/uor-addr-c/) emits a
  `staticlib` + `cdylib` plus a `cbindgen`-generated header at
  [`include/uor_addr.h`](../crates/uor-addr-c/include/uor_addr.h).
  Each realization is exposed as one `extern "C"` function
  (`uor_addr_json`, `uor_addr_sexp`, …). Builds clean on hosted
  targets and on `thumbv7em-none-eabihf` (Cortex-M4 bare-metal, no
  allocator) — the base substrate for Python (cffi), Go (cgo),
  Ruby (FFI), .NET (P/Invoke), and any embedded toolchain.

- **WASM Component Model** — [`uor-addr-wasm`](../crates/uor-addr-wasm/)
  is a `wit-bindgen`-driven component declared by
  [`wit/uor-addr.wit`](../crates/uor-addr-wasm/wit/uor-addr.wit).
  Build with `cargo build -p uor-addr-wasm --target wasm32-wasip2
  --release`; the emitted `.wasm` is consumable from JS
  (`jco transpile`), Python (`wasmtime-py`), Go (`wasmtime-go`),
  .NET (`Wasmtime.NET`), and any language with a wasm runtime.

Every binding produces the same 71-byte ASCII `sha256:<64hex>` κ-label
the Rust crate produces. The byte-identity guarantee is pinned by
the CF-C\* / CF-W\* invariant classes in
[../CONFORMANCE.md](../CONFORMANCE.md).

## What if my data doesn't fit any of these?

- If your data has a published canonical form not yet realized here
  (RDF C-14N, CBOR deterministic encoding, …), open an issue —
  adding a realization follows the pattern documented in
  [../ARCHITECTURE.md](../ARCHITECTURE.md) "Format-specific
  realizations".
- If your data is application-specific without a published canonical
  form, you must either define one (UOR-native, like
  [`codemodule`](../crates/uor-addr/src/codemodule/) does) or
  serialize through an existing format first (e.g. emit JSON bytes
  via your serializer of choice and pass them to
  `uor_addr::json::address`).

## What guarantees does picking the right realization give me?

For every shipped realization:

- **Determinism (CD-D01).** Same input bytes → same κ-label.
- **Wire-format width (CL-W01).** κ-label is exactly 71 ASCII bytes:
  `sha256:` plus 64 lowercase-hex.
- **Structural invariance.** Two inputs that are equivalent under the
  realization's canonical form share their κ-label.
- **Typed distinction.** Two inputs that are structurally distinct
  produce distinct κ-labels with the SHA-256 sensitivity bound.
- **TC-05 replay round-trip.** Every emitted Grounded value replays
  through `prism_verify::certify_from_trace` to a bit-identical
  `ContentFingerprint`.

The numbered conformance contract is in
[../CONFORMANCE.md](../CONFORMANCE.md); the per-realization test
suites under [../crates/uor-addr/tests/](../crates/uor-addr/tests/)
pin each invariant against the imported standard's published rules
(RFC 8785 Appendix B, X.690 Annex A, Rivest §6 examples, schema.org
type hierarchy, in-toto Statement v1 spec, …).
