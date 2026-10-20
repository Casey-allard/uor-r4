# uor-addr

> UOR-ADDR — the typed reference vocabulary for typed
> content-addressing across recursively-grammared formats. A
> [UOR Foundation](https://uor.foundation) standard-library Layer-3
> realization grounded against the wiki specification at
> <https://github.com/UOR-Foundation/UOR-Framework/wiki>.

A single Rust crate shipping UOR-ADDR's **common architectural surface**
plus **multiple concrete realizations** of that surface, each a
`PrismModel<HostTypes, HostBounds, Hasher, ResolverTuple,
TypedCommitment>` that derives a 71-byte `sha256:<64hex>` content
address from a typed format-specific value.

```rust
use uor_addr::json::address as json_address;
let outcome = json_address(br#"{"foo": "bar"}"#).unwrap();
// outcome.address == "sha256:7a38bf81…ee24b"

use uor_addr::sexp::address as sexp_address;
let outcome = sexp_address(b"(a b c)").unwrap();
// outcome.address == "sha256:cdd489dd…f50e"
```

## Realizations

Every realization implements the common
[`uor_addr::common::AddressInput`] trait and emits the canonical
71-byte κ-label.

- **Format-specific**: [`json`] (RFC 8785 JCS + RFC 8259 + UAX #15 NFC),
  [`sexp`] (Rivest 1997), [`xml`] (W3C XML-C14N 1.1 subset), [`asn1`]
  (ITU-T X.690 DER), [`ring`] (UOR-Framework Amendment 43 §2),
  [`codemodule`] (CCMAS canonical AST).
- **Schema-pinned descendants** that import existing standards per
  UOR's schema-import discipline: [`schema::photo`] →
  [schema.org/Photograph](https://schema.org/Photograph),
  [`schema::document`] →
  [schema.org/Article](https://schema.org/Article),
  [`schema::codemodule_signed`] →
  [in-toto Statement v1](https://in-toto.io/Statement/v1).
- **Cost-model-bearing variants** per ADR-048: [`variant::storage`]
  binds `AndCommitment<EmptyCommitment, SingletonCommitment<LexicographicLessEqThreshold>>`;
  [`variant::signed`] binds `SingletonCommitment<UltrametricCloseTo<2>>`.

Full per-realization authoritative-source index in
[STANDARDS.md](https://github.com/UOR-Foundation/uor-addr/blob/main/STANDARDS.md).

## Architectural commitments

The complete architectural specification is in
[ARCHITECTURE.md](https://github.com/UOR-Foundation/uor-addr/blob/main/ARCHITECTURE.md);
the numbered conformance contract is in
[CONFORMANCE.md](https://github.com/UOR-Foundation/uor-addr/blob/main/CONFORMANCE.md);
the V&V acceptance gate (`just vv`) is documented in
[VERIFICATION.md](https://github.com/UOR-Foundation/uor-addr/blob/main/VERIFICATION.md).

Key commitments every realization upholds:

- **ADR-035 canonical k-invariants branch** — the `address_inference`
  verb body composes ψ_1 + ψ_7 + ψ_8 + ψ_9; ψ_2..ψ_6 are off-path
  with identity-emitting resolver bodies.
- **ADR-046 canonicalization at carrier production** — the format's
  canonicalization happens at the host boundary in the input handle's
  `as_binding_value` (which yields the ADR-060 `TermValue` carrier), so
  the shared ψ-tower is format-independent and ψ_9 only folds the carrier.
- **ADR-047 σ-projection Hardening Principle** — every realization
  binds `prism::crypto::Sha256Hasher` as the canonical hash axis.
- **ADR-048 typed-commitment surface** — default `C = EmptyCommitment`;
  cost-model variants bind non-default `C` selections.
- **ADR-057 bounded recursive structural typing** — enforced by the
  recursive parsers' native-stack depth guards (`MAX_*_DEPTH`); inputs
  are otherwise unbounded (no width / count caps).
- **ADR-060 source-polymorphic value carrier** — each input handle yields
  `Inline` / `Borrowed` / `Stream` `TermValue` bytes; there is no fixed
  input buffer or size ceiling.

## Build + V&V

```bash
cargo build           # rustc >= 1.83
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
just vv               # full V&V gate
just examples         # 16 runnable comprehensive demos
```

`no_std`-compatible (`default-features = false`); only `alloc`
required. `#![forbid(unsafe_code)]` — zero unsafe blocks.

## License

Apache-2.0, matching `uor-foundation`.
