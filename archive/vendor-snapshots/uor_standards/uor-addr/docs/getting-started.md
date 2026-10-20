# Getting started

This walks the three-step path from raw bytes to a κ-label, then shows
how κ-labels compose with the framework's structural guarantees.

## 1. Pick a realization

Each realization handles a specific data format. The choice is fixed
by what your data already is; pick the row that matches the format you
have on the wire.

```rust
use uor_addr::json::address as json_address;
use uor_addr::sexp::address as sexp_address;
use uor_addr::xml::address as xml_address;
use uor_addr::asn1::address as asn1_address;
use uor_addr::ring::address as ring_address;
use uor_addr::codemodule::address as codemodule_address;
```

For schema-typed data (photo metadata, articles, signed-software
attestations) prefer the schema-pinned descendants — they add
admission predicates without changing the κ-label.

```rust
use uor_addr::schema::photo::address as photo_address;
use uor_addr::schema::document::address as document_address;
use uor_addr::schema::codemodule_signed::address as signed_address;
```

See [realizations.md](realizations.md) for the full decision matrix.

## 2. Mint a κ-label

The `address` function takes raw bytes and returns an outcome carrying
the wire-format κ-label plus an owned `AddressWitness` for downstream
verification.

```rust
let outcome = uor_addr::json::address(br#"{"foo": "bar"}"#).unwrap();
println!("{}", outcome.address);
// sha256:7a38bf81f383f69433ad6e900d35b3e2385593f76a7b7ab5d4355b8ba41ee24b
```

The κ-label is **deterministic** — feed the same bytes twice, get the
same label. It's also **invariant** under the format's canonical-form
rules:

```rust
// JSON: whitespace, key order, NFC vs NFD all collapse.
let a = uor_addr::json::address(br#"{"a":1,"b":2}"#).unwrap().address;
let b = uor_addr::json::address(br#"{ "b" : 2 , "a" : 1 }"#).unwrap().address;
assert_eq!(a, b);
```

But it **distinguishes** typed values that look similar but mean
different things:

```rust
let int = uor_addr::json::address(b"42").unwrap().address;
let str = uor_addr::json::address(br#""42""#).unwrap().address;
assert_ne!(int, str);
```

## 3. Verify a κ-label without re-hashing

Every `address()` call also emits an **owned** `AddressWitness` holding
the replayable trace plus the σ-projection fingerprint. Downstream
consumers call `witness.verify()`, which re-certifies through
`prism::replay::certify_from_trace` — the verifier sees the trace, not
the original input, and does not invoke SHA-256 again.

```rust
let outcome = uor_addr::json::address(br#"{"foo": "bar"}"#).unwrap();
let witness = outcome.witness;
assert_eq!(witness.kappa_label(), outcome.address);
// verify() replays the trace and re-confirms the fingerprint,
// returning the same κ-label without re-hashing.
assert_eq!(witness.verify().unwrap(), outcome.address);
// The trace replay path is exercised by `tests/replay.rs`.
```

## Embedded / `no_std`

The crate is `no_std` by default, and the ψ-pipeline itself never
touches an allocator. The realizations whose canonical form is produced
without heap — **ring, sexp, asn1, codemodule** — are `no_alloc`: their
input flows as an `Inline`, `Stream`, or `Borrowed`-over-input carrier.
Build for bare-metal Cortex-M4:

```bash
rustup target add thumbv7em-none-eabihf
cargo build -p uor-addr --no-default-features --target thumbv7em-none-eabihf
```

That `--no-default-features` build exposes the `no_alloc` realizations'
κ-label functions only. The realizations that must materialize a
canonical form on the heap — **json, xml, the `schema::*` descendants,
gguf, onnx** (object-key / attribute sorting, or the materialized
flat skeleton) — gate their `address()` / `canonicalize()` entry points
behind the `alloc` feature. Enabling `alloc` (or `std`) is otherwise
purely additive and never changes any κ-label byte-for-byte (CB-A03 +
CB-A04 in [../CONFORMANCE.md](../CONFORMANCE.md)).

## Consuming from non-Rust callers

Two FFI distribution targets ship the same κ-label byte-for-byte:

- **C / embedded** — [`uor-addr-c`](../crates/uor-addr-c/) emits a
  `staticlib` + `cdylib` plus a `cbindgen`-generated header. Each
  realization is exposed as one `extern "C"` function. Builds for
  hosted targets and for `thumbv7em-none-eabihf` (Cortex-M4
  bare-metal, no allocator).

  ```c
  #include "uor_addr.h"

  uint8_t  out[UOR_ADDR_LABEL_BYTES];
  size_t   written = 0;
  int32_t  rc = uor_addr_json(
      (const uint8_t *)"{\"foo\":\"bar\"}", 13,
      out, sizeof(out), &written);
  /* rc == UOR_ADDR_OK; out[..71] is the ASCII κ-label */
  ```

- **WASM Component Model** —
  [`uor-addr-wasm`](../crates/uor-addr-wasm/) is a `wit-bindgen`
  component declared by
  [`wit/uor-addr.wit`](../crates/uor-addr-wasm/wit/uor-addr.wit).
  Build with `cargo build -p uor-addr-wasm --target wasm32-wasip2
  --release`; consume the resulting `.wasm` from JS / Python / Go /
  .NET / Ruby / Java / C# via their respective wasmtime bindings.

## Where to next?

- Pick the right realization: [realizations.md](realizations.md).
- See the full architectural picture: [../ARCHITECTURE.md](../ARCHITECTURE.md).
- Run every example: `just examples`.
- Reproduce the V&V gate: [../VERIFICATION.md](../VERIFICATION.md).
- Mint κ-labels from C: [../crates/uor-addr-c/README.md](../crates/uor-addr-c/README.md).
- Build the WASM component: [../crates/uor-addr-wasm/README.md](../crates/uor-addr-wasm/README.md).
