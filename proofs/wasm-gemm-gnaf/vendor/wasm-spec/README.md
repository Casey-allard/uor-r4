# vendor/wasm-spec/

SPEC §4 pins WebAssembly Core to the official `wg-3.0` source at commit
`9d36019973201a19f9c9ebb0f10828b2fe2374aa`, and SPEC §5 requires the exact files
named by the conformance map to be vendored here.

**Not yet vendored.** Release gate step 1 fails until the pinned tree is present
and every file digest is recomputed from content — SPEC §5 requires offline
verification to recompute pins from bytes rather than trust a checksum string.
