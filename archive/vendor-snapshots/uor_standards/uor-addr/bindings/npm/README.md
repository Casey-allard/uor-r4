# @uor-foundation/uor-addr

JavaScript / TypeScript bindings for [`uor-addr`](https://github.com/UOR-Foundation/uor-addr) — typed content-addressing producing deterministic `sha256:<64hex>` κ-labels from JSON, S-expressions, XML, ASN.1 DER, schema.org, in-toto, and more.

Wraps the [`uor-addr-wasm`](https://github.com/UOR-Foundation/uor-addr/tree/main/crates/uor-addr-wasm) WASM Component Model artifact via [`jco`](https://github.com/bytecodealliance/jco). The produced κ-label is **byte-for-byte identical** to the Rust crate's output.

## Install

```bash
npm install @uor-foundation/uor-addr
```

Runs anywhere a WebAssembly engine is available — Node, Deno, Bun, Cloudflare Workers, and the browser. The package is a single self-contained ES module: the WebAssembly is inlined, so there are no `node:` imports, no `fetch`, and no sidecar `.wasm` asset to resolve.

## Quickstart

```typescript
import { kappa } from "@uor-foundation/uor-addr";

const label = kappa.jsonAddress(new TextEncoder().encode('{"foo":"bar"}'));
console.log(label);
// sha256:7a38bf81f383f69433ad6e900d35b3e2385593f76a7b7ab5d4355b8ba41ee24b
```

## API

Eleven `*-address` functions, one per realization (json, sexp, xml, asn1, ring, codemodule, the three `schema-*` descendants, gguf, onnx). Each takes a `Uint8Array` and returns a 71-byte ASCII string of the form `sha256:<64-lowercase-hex>`. Failures throw with a wasm-runtime error carrying the realization's `address-error` variant — `invalid-input` or `pipeline-failure` (`too-large` is reserved and never thrown under ADR-060: inputs are unbounded).

| Function | Realization | Imported spec |
|---|---|---|
| `kappa.jsonAddress` | JSON | RFC 8259 + RFC 8785 JCS + UAX #15 NFC |
| `kappa.sexpAddress` | S-expressions | Rivest 1997 canonical form |
| `kappa.xmlAddress` | XML | W3C XML-C14N 1.1 (subset) |
| `kappa.asn1Address` | ASN.1 | ITU-T X.690 DER |
| `kappa.ringAddress` | Ring elements | UOR-Framework Amendment 43 §2 |
| `kappa.codemoduleAddress` | Code-module AST | CCMAS |
| `kappa.schemaPhotoAddress` | schema.org/Photograph | schema.org/Photograph |
| `kappa.schemaDocumentAddress` | schema.org/Article (+ subtypes) | schema.org/Article |
| `kappa.schemaCodemoduleSignedAddress` | in-toto Statement v1 | in-toto Statement v1 |

## Determinism + canonical-form invariance

The κ-label is **deterministic** — the same input bytes always produce the same label. It is also **invariant** under each format's canonical-form rules:

```typescript
const enc = new TextEncoder();

// JSON: whitespace, key order, NFC vs NFD all collapse.
const a = kappa.jsonAddress(enc.encode('{"a":1,"b":2}'));
const b = kappa.jsonAddress(enc.encode('{ "b" : 2 , "a" : 1 }'));
console.assert(a === b);

// But it DISTINGUISHES typed values that look similar.
const intLabel = kappa.jsonAddress(enc.encode("42"));
const strLabel = kappa.jsonAddress(enc.encode('"42"'));
console.assert(intLabel !== strLabel);
```

## TC-05 replay across the wasm boundary

Each `*AddressWithWitness` function returns a `Grounded` resource carrying the ψ-pipeline's emitted derivation. Calling `grounded.verify()` replays the derivation through `prism_verify::certify_from_trace` and returns the recovered κ-label **without re-invoking SHA-256**. The verifier reads the trace events the source pipeline emitted and re-packages the certified output (QS-05 replay equivalence; CL-R\* in [CONFORMANCE.md](https://github.com/UOR-Foundation/uor-addr/blob/main/CONFORMANCE.md)).

```typescript
import { kappa } from "@uor-foundation/uor-addr";

const grounded = kappa.jsonAddressWithWitness(
  new TextEncoder().encode('{"foo":"bar"}'),
);

console.log(grounded.kappaLabel());
// sha256:7a38bf81f383f69433ad6e900d35b3e2385593f76a7b7ab5d4355b8ba41ee24b

console.log(grounded.verify() === grounded.kappaLabel());
// true — TC-05 round-trip; SHA-256 was not re-invoked.

const fp: Uint8Array = grounded.contentFingerprint();
// 32-byte content fingerprint (distinct from the κ-label hex suffix —
// it is prism's content-address of the Grounded's full state).
```

The `Grounded` resource is managed by the Component Model runtime; let it go out of scope to release the underlying handle. Cross-process attestation is not supported (the underlying `Trace<256>` constructor is `pub(crate)` in `uor-foundation`); for that, persist the κ-label itself and re-mint at the verifier side.

## Byte identity with the Rust crate

The κ-label this package produces is **byte-for-byte identical** to `uor_addr::<realization>::address(input).address` from the [Rust crate](https://crates.io/crates/uor-addr). Cross-validation is pinned by the **CF-W\*** invariant class in [CONFORMANCE.md](https://github.com/UOR-Foundation/uor-addr/blob/main/CONFORMANCE.md).

## Building from source

```bash
# From the workspace root — build the zero-import core module:
cargo build -p uor-addr-wasm --target wasm32-unknown-unknown --release

# Then in this directory:
cd bindings/npm
npm install
npm run build      # componentizes + inlines the wasm into one ES module
npm test           # smoke-tests every realization
```

## License

Apache-2.0. See [LICENSE](LICENSE).
