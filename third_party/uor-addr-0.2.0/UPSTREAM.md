# Upstream source

UOR Foundation uor-addr 0.2.0, exact Git revision `165b51e3e2113ee5d032730cde709335d4fe9b60`, Apache-2.0. LICENSE copied unchanged. The exact original package tree is preserved at historical/upstream/crates/uor-addr; its original workspace manifest is historical/upstream/Cargo.toml.

Active NFC source and generated UCD 15.1.0 tables are copied byte-for-byte. JSON tag constants and the parser/emitter through nibble_hex are extracted from src/json/value.rs; parser arithmetic, ordering, string escapes, NFC and number emission are unchanged. Only the error carrier/import plumbing is local.

The active address function reproduces the upstream json::address_blake3 content label: canonicalize input, unkeyed BLAKE3 over the complete canonical bytes, lowercase digest with blake3: prefix. The upstream Prism Blake3Hasher uses exactly blake3::Hasher::new/update/finalize; the upstream hash_kat fixture explicitly binds this pipeline to that canonical-byte hash. This narrow facade returns the content label only and does not fabricate a Prism/TC-05 witness. Numerical/model/pipeline APIs are preserved but excluded, and there are no Prism dependencies. gguf/onnx requests fail at compile time.

No compiled/model/quality qualification is implied by preservation or metadata checks.
