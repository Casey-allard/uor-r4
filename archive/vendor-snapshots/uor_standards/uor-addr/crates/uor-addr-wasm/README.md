# uor-addr-wasm

WASM Component Model bindings for [`uor-addr`](../uor-addr) — a
language-agnostic `.wasm` component that mints the same 71-byte
`sha256:<64hex>` κ-label the Rust crate produces, byte-for-byte.

## What this crate is

A [wit-bindgen]-driven Component Model component that exports one
`*-address` function per UOR-ADDR realization. Build it once, consume
it from JS, Python, Go, .NET, Ruby, Java — any language with a Wasm
runtime.

The WIT interface lives at [`wit/uor-addr.wit`](wit/uor-addr.wit) and
declares the `uor-addr` world:

```wit
interface kappa {
    type kappa-label = string;
    variant address-error { invalid-input, too-large, pipeline-failure }
    json-address: func(input: list<u8>) -> result<kappa-label, address-error>;
    sexp-address: func(input: list<u8>) -> result<kappa-label, address-error>;
    xml-address:  func(input: list<u8>) -> result<kappa-label, address-error>;
    asn1-address: func(input: list<u8>) -> result<kappa-label, address-error>;
    ring-address: func(input: list<u8>) -> result<kappa-label, address-error>;
    codemodule-address: func(input: list<u8>) -> result<kappa-label, address-error>;
    schema-photo-address:               func(input: list<u8>) -> result<kappa-label, address-error>;
    schema-document-address:            func(input: list<u8>) -> result<kappa-label, address-error>;
    schema-codemodule-signed-address:   func(input: list<u8>) -> result<kappa-label, address-error>;
}
world uor-addr { export kappa; }
```

## Building

```bash
# Build the zero-import core module, then componentize it:
rustup target add wasm32-unknown-unknown
cargo build -p uor-addr-wasm --release --target wasm32-unknown-unknown
wasm-tools component new \
  target/wasm32-unknown-unknown/release/uor_addr_wasm.wasm \
  -o uor_addr_wasm.wasm
```

This crate imports **nothing** from the host, so the componentized
output has zero imports and needs no WASI adapter. Avoid
`wasm32-wasip2`: that target links std's WASI runtime
(`cli`/`io`/`exit`/`environment`) into the component even though it is
never called, forcing every host to provision WASI 0.2 and pinning the
JS path to jco's Node-only `preview2-shim` (breaking browser / Deno /
Bun / Workers use). `cargo component build` also works but is
unnecessary for a host-import-free guest.

The crate is **only meaningful on `wasm32-*` targets**. On host
architectures the workspace builds it as an empty rlib so
`cargo build --workspace` succeeds without a wasm toolchain everywhere;
the `wit-bindgen::generate!` invocation and the Component Model
`export!` macro are gated on `target_arch = "wasm32"`.

## Consuming the component

| Language | Mechanism |
|---|---|
| JS / TS | [`jco transpile`](https://github.com/bytecodealliance/jco) → npm-publishable bindings |
| Python | [`wasmtime-py`](https://github.com/bytecodealliance/wasmtime-py) |
| Go | [`wasmtime-go`](https://github.com/bytecodealliance/wasmtime-go) |
| .NET | [`Wasmtime.NET`](https://github.com/bytecodealliance/wasmtime-dotnet) |
| Ruby | [`wasmtime-rb`](https://github.com/bytecodealliance/wasmtime-rb) |
| Java | [`chicory`](https://github.com/dylibso/chicory) or wasmtime-java bindings |

All host paths produce the **same 71-byte κ-label byte-for-byte** as
the Rust + C ABI paths.

## Allocator

The WIT Component Model represents `list<u8>` and `string` as
heap-allocated Rust types in the binding layer (`Vec<u8>` and
`String`). Wasm runtimes ship an allocator; the binding turns on
the `alloc` feature of `uor-addr` accordingly. The underlying
ψ-pipeline remains `no_alloc` — only the host-input / host-output
marshalling at the Component Model boundary allocates.

## License

Apache-2.0 — same as [`uor-addr`](../uor-addr).

[wit-bindgen]: https://github.com/bytecodealliance/wit-bindgen
