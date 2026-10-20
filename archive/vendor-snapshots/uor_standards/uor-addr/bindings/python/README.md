# uor-addr (Python)

Python bindings for [`uor-addr`](https://github.com/UOR-Foundation/uor-addr) — typed content-addressing producing deterministic `sha256:<64hex>` κ-labels from JSON, S-expressions, XML, ASN.1 DER, schema.org, in-toto, and more.

Wraps the [`uor-addr-c`](https://github.com/UOR-Foundation/uor-addr/tree/main/crates/uor-addr-c) C ABI dynamic library via `ctypes` (stdlib — **no external dependencies**). The produced κ-label is **byte-for-byte identical** to the Rust crate's output.

## Install

```bash
pip install uor-addr
```

Requires Python 3.10+. Wheels ship per-platform; sdist falls back to building from the workspace.

## Quickstart

```python
from uor_addr import kappa

label = kappa.json_address(b'{"foo":"bar"}')
print(label)
# sha256:7a38bf81f383f69433ad6e900d35b3e2385593f76a7b7ab5d4355b8ba41ee24b
```

## API

Nine `*_address` methods on the `kappa` singleton, one per realization. Each takes `bytes`-like and returns the 71-byte ASCII `sha256:<64-lowercase-hex>` κ-label as `str`. Failures raise `uor_addr.AddressError` carrying one of three `kind` tags (`'invalid-input'` / `'too-large'` / `'pipeline-failure'`).

| Method | Realization | Imported spec |
|---|---|---|
| `kappa.json_address` | JSON | RFC 8259 + RFC 8785 JCS + UAX #15 NFC |
| `kappa.sexp_address` | S-expressions | Rivest 1997 canonical form |
| `kappa.xml_address` | XML | W3C XML-C14N 1.1 (subset) |
| `kappa.asn1_address` | ASN.1 | ITU-T X.690 DER |
| `kappa.ring_address` | Ring elements | UOR-Framework Amendment 43 §2 |
| `kappa.codemodule_address` | Code-module AST | CCMAS |
| `kappa.schema_photo_address` | schema.org/Photograph | schema.org/Photograph |
| `kappa.schema_document_address` | schema.org/Article (+ subtypes) | schema.org/Article |
| `kappa.schema_codemodule_signed_address` | in-toto Statement v1 | in-toto Statement v1 |

## Determinism + canonical-form invariance

```python
from uor_addr import kappa

# JSON: whitespace, key order, NFC vs NFD all collapse.
a = kappa.json_address(b'{"a":1,"b":2}')
b = kappa.json_address(b'{ "b" : 2 , "a" : 1 }')
assert a == b

# But typed values that look similar produce distinct κ-labels.
int_label = kappa.json_address(b"42")
str_label = kappa.json_address(b'"42"')
assert int_label != str_label
```

## TC-05 replay across the FFI boundary

Each `*_address_with_witness` method returns a `Grounded` handle carrying the ψ-pipeline's emitted derivation. Calling `grounded.verify()` replays the derivation through `prism_verify::certify_from_trace` and returns the recovered κ-label **without re-invoking SHA-256**. The verifier reads the trace events the source pipeline emitted and re-packages the certified output (QS-05 replay equivalence; CL-R\* in [CONFORMANCE.md](https://github.com/UOR-Foundation/uor-addr/blob/main/CONFORMANCE.md)).

```python
from uor_addr import kappa

with kappa.json_address_with_witness(b'{"foo":"bar"}') as grounded:
    print(grounded.kappa_label())
    # sha256:7a38bf81f383f69433ad6e900d35b3e2385593f76a7b7ab5d4355b8ba41ee24b

    assert grounded.verify() == grounded.kappa_label()
    # TC-05 round-trip; SHA-256 was not re-invoked.

    fingerprint: bytes = grounded.content_fingerprint()
    # 32-byte content fingerprint (distinct from the κ-label hex
    # suffix — it is prism's content-address of the Grounded's full
    # state, used for replay verification).
```

The `Grounded` handle is freed on `__exit__` of the `with` block; otherwise it is freed at garbage-collection time. Cross-process attestation is not supported (the underlying `Trace<256>` constructor is `pub(crate)` in `uor-foundation`); for that, persist the κ-label itself and re-mint at the verifier side.

## Why C ABI rather than WASM?

The `@uor-foundation/uor-addr` npm package wraps the WASM Component Model artifact via [`jco`](https://github.com/bytecodealliance/jco). Python's [`wasmtime-py`](https://pypi.org/project/wasmtime/) does not yet expose the Component Model API (24.x ships core-module support only); pivoting to wasm would require either waiting for upstream or pulling in a less-mature runtime. The C ABI path is faster (native code), more compact (no wasm runtime in the wheel), and produces the same κ-label byte-for-byte. The `uor-addr-c` native library is bundled per-platform in the wheel.

## Byte identity with the Rust crate

The κ-label this package produces is **byte-for-byte identical** to `uor_addr::<realization>::address(input).address` from the Rust crate (and to `@uor-foundation/uor-addr` npm's output). Cross-validation is pinned by the **CF-C\*** invariant class in [CONFORMANCE.md](https://github.com/UOR-Foundation/uor-addr/blob/main/CONFORMANCE.md).

## License

Apache-2.0.
