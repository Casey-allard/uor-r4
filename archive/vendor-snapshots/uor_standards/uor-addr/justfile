# `just vv` is the normative V&V acceptance gate.
# See VERIFICATION.md for the full axis-by-axis mapping.

set shell := ["bash", "-cu"]

default: vv

# ──────────────────────────────────────────────────────────────────────────
# Acceptance gate
# ──────────────────────────────────────────────────────────────────────────

# Full V&V — every axis required for merge. Halts on the first failure.
vv: fmt-check lint test embedded wasm conformance analysis replay examples doc-check verify version-sync

# Fast CI subset — no Lean, no live network. Use when iterating locally.
ci: fmt-check lint test embedded wasm

# ──────────────────────────────────────────────────────────────────────────
# Individual axes
# ──────────────────────────────────────────────────────────────────────────

# Axis 1 — format check.
fmt-check:
	cargo fmt --all -- --check

# Axis 2 — clippy with -D warnings.
lint:
	cargo clippy --workspace --all-targets -- -D warnings

# Axis 3 — workspace unit + integration tests.
# Excludes `uor-addr-wasm`: its `wit-bindgen`-generated Component Model
# symbols are valid for `wasm32-wasip2` (verified by `just wasm`) but
# the same crate-type=cdylib does not link cleanly on hosted ELF
# targets. Hosted Rust tests run for `uor-addr` + `uor-addr-c`.
test:
	cargo test --workspace --exclude uor-addr-wasm

# Axis 3b — `uor-addr-c` no_std embedded build proof
# (Cortex-M4 / thumbv7em-none-eabihf).
embedded:
	cargo build -p uor-addr-c --no-default-features --target thumbv7em-none-eabihf

# Axis 3c — `uor-addr-wasm` WASM Component Model build proof
# (wasm32-wasip2, polyglot consumption via jco / wasmtime / etc.).
wasm:
	cargo build -p uor-addr-wasm --target wasm32-wasip2 --release

# Axis 4 — conformance suite (release). Each shipped realization
# has a dedicated published-spec test vector suite plus the
# cross-realization integration test.
conformance:
	# Common architectural surface
	cargo test -p uor-addr --release --test common_surface
	# JSON realization — RFC 8259 + RFC 8785 JCS + UAX #15 NFC
	cargo test -p uor-addr --release --test conformance
	cargo test -p uor-addr --release --test jcs_rfc8785
	# S-expression — Rivest 1997
	cargo test -p uor-addr --release --test sexp_conformance
	cargo test -p uor-addr --release --test sexp_rivest_examples
	# XML — W3C XML-C14N 1.1
	cargo test -p uor-addr --release --test xml_c14n_1_1
	# ASN.1 — ITU-T X.690 DER
	cargo test -p uor-addr --release --test asn1_x690_der
	# Ring — UOR-Framework Amendment 43 §2
	cargo test -p uor-addr --release --test ring_amendment_43
	# Code-module AST — CCMAS canonical
	cargo test -p uor-addr --release --test codemodule_ccmas
	# Schema-pinned descendants — schema.org + in-toto
	cargo test -p uor-addr --release --test schema_org_conformance
	cargo test -p uor-addr --release --test in_toto_statement_v1
	# Cost-model variants
	cargo test -p uor-addr --release --test variant_storage
	# GGUF v3 — flat Merkle skeleton (canonical-gguf.py byte-identity + invariants)
	cargo test -p uor-addr --release --test gguf_conformance --test gguf_byte_identity
	# ONNX — flat skeleton (canonical-onnx.py byte-identity + invariants)
	cargo test -p uor-addr --release --test onnx_conformance --test onnx_byte_identity
	# ADR-060 streaming / bounded-carrier proof (64 MiB synthetic tensors)
	cargo test -p uor-addr --release --test streaming
	# Arbitrary-hash σ-axes (authoritative FIPS/Keccak/BLAKE3 KATs) + CBOR
	# (RFC 8949 §4.2 / Appendix A).
	cargo test -p uor-addr --release --test hash_kat
	cargo test -p uor-addr --release --test cbor_rfc8949
	cargo test -p uor-addr --release --test composition
	# Cross-realization
	cargo test -p uor-addr --release --test all_realizations

# Axis 5 — analysis suite (release, large samples).
analysis:
	cargo test -p uor-addr --release --test analysis

# Axis 6 — TC-05 replay round-trip via `prism_verify::certify_from_trace`.
replay:
	cargo test -p uor-addr --release --test replay

# Axis 7 — runnable use-case examples. Each example panics on a failed
# invariant; passing requires every example to exit cleanly.
examples:
	# Common architectural surface
	cargo run -p uor-addr --example common_surface
	# JSON realization
	cargo run -p uor-addr --example address_value
	cargo run -p uor-addr --example dedupe_cache
	cargo run -p uor-addr --example typed_distinction
	cargo run -p uor-addr --example replay_verification
	# Other format-specific realizations
	cargo run -p uor-addr --example sexp_address
	cargo run -p uor-addr --example cbor_address
	cargo run -p uor-addr --example xml_realization
	cargo run -p uor-addr --example asn1_realization
	cargo run -p uor-addr --example ring_realization
	cargo run -p uor-addr --example codemodule_realization
	# Schema-pinned descendants
	cargo run -p uor-addr --example photo_schema
	cargo run -p uor-addr --example document_schema
	cargo run -p uor-addr --example codemodule_signed_schema
	# Cost-model variants
	cargo run -p uor-addr --example storage_variant
	cargo run -p uor-addr --example signed_variant
	# Model-file realizations (real-world: registry / provenance)
	cargo run -p uor-addr --example gguf_model_registry
	cargo run -p uor-addr --example onnx_provenance
	# Cross-realization showcase
	cargo run -p uor-addr --example composition
	cargo run -p uor-addr --example multi_realization

# Axis 8 — rustdoc with broken-intra-doc-links denied.
doc-check:
	RUSTDOCFLAGS="-D rustdoc::broken-intra-doc-links" cargo doc --workspace --no-deps

# Axis 9 — Lean proofs (lake build).
verify:
	cd uor-addr-lean && lake build

# Axis 11 — single-source-of-truth version sync. Cargo.toml's
# [workspace.package].version is the master; sync-versions.py
# propagates it to bindings/npm + bindings/python; --check refuses
# drift. CI runs this on every PR.
version-sync:
	python3 tools/sync-versions.py --check

# Axis 10 — live cross-validation. Gated; opt in via UOR_ADDR_LIVE=1.
# Includes the spec-side Python encoders (CN-GGUF / CN-ONNX) and the
# pinned external real-model V&V (CM-EXT) — the latter downloads ~635 MB
# of real GGUF / ONNX models (cached under tests/fixtures/models/) and the
# 531 MB GGUF exercises the streaming / bounded-carrier path.
cn:
	UOR_ADDR_LIVE=1 cargo test -p uor-addr --release --test cross_validation -- --ignored
	UOR_ADDR_LIVE=1 cargo test -p uor-addr --release --features gguf,onnx --test gguf_cross_validation --test onnx_cross_validation -- --ignored
	UOR_ADDR_LIVE=1 cargo test -p uor-addr --release --features gguf,onnx --test external_models -- --ignored

# ──────────────────────────────────────────────────────────────────────────
# Build / clean / repl conveniences
# ──────────────────────────────────────────────────────────────────────────

build:
	cargo build --workspace

build-release:
	cargo build --workspace --release

clean:
	cargo clean && cd uor-addr-lean && lake clean

doc:
	cargo doc --workspace --no-deps --open

fmt:
	cargo fmt --all
