//! **`uor-addr-wasm` — WASM Component Model bindings for `uor-addr`**.
//!
//! Generates a Component-Model component from the WIT interface at
//! `wit/uor-addr.wit` via the `wit-bindgen` macro. Exports:
//!
//! - one `*-address` function per UOR-ADDR realization (κ-label only),
//! - one `*-address-with-witness` function per realization (returning
//!   an opaque `grounded` resource that carries the ψ-pipeline
//!   derivation),
//! - the `grounded` resource type with three methods:
//!   `kappa-label`, `content-fingerprint`, `verify`.
//!
//! # Polyglot consumption
//!
//! This crate is pure compute — it imports nothing from the host. Build
//! it for `wasm32-unknown-unknown` and componentize the resulting core
//! module (`jco new`, or `wasm-tools component new`) to obtain a
//! **zero-import Component Model component**. Prefer this over
//! `wasm32-wasip2`: the wasip2 target links std's WASI runtime
//! (`cli`/`io`/`exit`/`environment`) into the component even though it
//! is never called, which forces every host to provision WASI 0.2 and —
//! for the JS path — pins jco's Node-only `preview2-shim`, breaking
//! browser / Deno / Bun / Workers / bundler use.
//!
//! The zero-import component is consumable from:
//!
//! - **JS / TS** via `jco transpile` → npm-publishable bindings that run
//!   in any JS environment (see `bindings/npm/scripts/build.mjs`).
//! - **Python** via `wasmtime-py` (once it adds Component Model
//!   support; until then Python uses the C ABI path).
//! - **Go** via `wasmtime-go`.
//! - **.NET** via `Wasmtime.NET`.
//! - **Ruby / Java / C#** via their respective wasmtime bindings.
//!
//! All host paths produce the **same 71-byte κ-label byte-for-byte**
//! as the Rust + C ABI paths.
//!
//! # TC-05 replay across the wasm boundary
//!
//! `grounded.verify()` re-certifies the witness's owned replay
//! `Trace<256>` through `prism::replay::certify_from_trace` (ADR-060,
//! via `uor_addr::AddressWitness::verify`), returning the re-derived
//! κ-label. The verifier path does **not** re-invoke the canonical
//! SHA-256 hash axis (TC-05 / QS-05 — see CL-R\* in CONFORMANCE.md). The
//! resource carries the in-process owned witness; cross-process replay
//! requires re-minting at the verifier side (a deliberate constraint of
//! the Component Model resource lifecycle).
//!
//! # Allocator
//!
//! The WIT Component Model represents `list<u8>` and `string` as
//! heap-allocated Rust types in the binding layer (`Vec<u8>` and
//! `String`). Wasm runtimes ship an allocator; the binding turns on
//! the `alloc` feature of `uor-addr` accordingly. The underlying
//! ψ-pipeline remains no_alloc — only the host-input / host-output
//! marshalling at the Component Model boundary allocates.

// `uor-addr-wasm` targets `wasm32-unknown-unknown` (componentized
// post-build). Outside `wasm32`, the crate compiles to an empty
// `rlib`/`cdylib` so the workspace builds without a wasm toolchain
// everywhere. The Component Model symbol exports only link on `wasm32`.

#![cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]

#[cfg(target_arch = "wasm32")]
mod component {
    use std::string::ToString;
    use std::vec::Vec;

    // `wit-bindgen` generates the Component Model glue from the WIT
    // file at compile time. `world: "uor-addr"` matches the world
    // declared in `wit/uor-addr.wit`. The `grounded` resource is
    // bound to `GroundedImpl` through the `Guest::Grounded` associated
    // type below — wit-bindgen 0.34 uses the associated-type pattern
    // (not the `with:` remap) for resources declared in the world's
    // exported interfaces.
    wit_bindgen::generate!({
        world: "uor-addr",
        path: "wit/uor-addr.wit",
        generate_all,
    });

    use exports::uor::addr::kappa::{
        AddressError, Grounded, Guest, GuestGrounded, HashAlgorithm, KappaLabel, VerifyError,
    };

    /// Component-Model export root.
    pub struct UorAddrComponent;

    // ─── grounded resource: the foreign-managed witness handle ────────

    /// The Rust-side state behind the WIT `resource grounded`. Holds
    /// the `AddressOutcome` (which carries the sealed
    /// `Grounded<AddressLabel>` and the 71-byte κ-label) for the
    /// lifetime of the resource handle.
    ///
    /// wit-bindgen wraps this in a `Grounded` handle that the host
    /// consumes via the WIT method exports; the host's `own grounded`
    /// drop triggers `Drop` here, releasing the Rust state.
    /// Width-erased owned outcome — lets one `grounded` resource carry a
    /// κ-label of any admissible σ-axis width (71 / 73 / 74).
    enum AnyOutcome {
        W71(uor_addr::AddressOutcome<71>),
        W73(uor_addr::AddressOutcome<73>),
        W74(uor_addr::AddressOutcome<74>),
        W512(uor_addr::AddressOutcome<135, 64>),
    }

    impl From<uor_addr::AddressOutcome<71>> for AnyOutcome {
        fn from(o: uor_addr::AddressOutcome<71>) -> Self {
            Self::W71(o)
        }
    }
    impl From<uor_addr::AddressOutcome<73>> for AnyOutcome {
        fn from(o: uor_addr::AddressOutcome<73>) -> Self {
            Self::W73(o)
        }
    }
    impl From<uor_addr::AddressOutcome<74>> for AnyOutcome {
        fn from(o: uor_addr::AddressOutcome<74>) -> Self {
            Self::W74(o)
        }
    }
    impl From<uor_addr::AddressOutcome<135, 64>> for AnyOutcome {
        fn from(o: uor_addr::AddressOutcome<135, 64>) -> Self {
            Self::W512(o)
        }
    }

    impl AnyOutcome {
        fn label(&self) -> String {
            match self {
                Self::W71(o) => o.address.as_str().to_string(),
                Self::W73(o) => o.address.as_str().to_string(),
                Self::W74(o) => o.address.as_str().to_string(),
                Self::W512(o) => o.address.as_str().to_string(),
            }
        }
        fn fingerprint(&self) -> Vec<u8> {
            match self {
                Self::W71(o) => o.witness.content_fingerprint().to_vec(),
                Self::W73(o) => o.witness.content_fingerprint().to_vec(),
                Self::W74(o) => o.witness.content_fingerprint().to_vec(),
                Self::W512(o) => o.witness.content_fingerprint().to_vec(),
            }
        }
        fn verify(&self) -> Result<String, uor_addr::VerifyError> {
            match self {
                Self::W71(o) => o.witness.verify().map(|l| l.as_str().to_string()),
                Self::W73(o) => o.witness.verify().map(|l| l.as_str().to_string()),
                Self::W74(o) => o.witness.verify().map(|l| l.as_str().to_string()),
                Self::W512(o) => o.witness.verify().map(|l| l.as_str().to_string()),
            }
        }
    }

    pub struct GroundedImpl {
        outcome: AnyOutcome,
    }

    impl GuestGrounded for GroundedImpl {
        fn kappa_label(&self) -> KappaLabel {
            self.outcome.label()
        }

        fn content_fingerprint(&self) -> Vec<u8> {
            // ADR-060: the witness owns its 32-byte σ-projection fingerprint.
            self.outcome.fingerprint()
        }

        fn verify(&self) -> Result<KappaLabel, VerifyError> {
            // ADR-060: `verify()` re-certifies the owned replay `Trace<256>`
            // through `prism::replay::certify_from_trace` (SHA-256 is *not*
            // re-invoked) and confirms the re-derived fingerprint matches
            // (QS-05 replay equivalence; CL-R* in CONFORMANCE.md), returning
            // the recovered κ-label.
            self.outcome.verify().map_err(map_verify_error)
        }
    }

    fn map_verify_error(e: uor_addr::VerifyError) -> VerifyError {
        // Both are defensive — unreachable for a witness this component
        // minted. Map to the closest existing WIT verify-error variants.
        match e {
            uor_addr::VerifyError::ReplayFailed => VerifyError::EmptyTrace,
            uor_addr::VerifyError::FingerprintMismatch => VerifyError::OutOfOrderEvent,
        }
    }

    // ─── Helpers to factor out the κ-label-only path ────────────────

    // Under ADR-060 every realization's `AddressFailure` is the uniform
    // two-variant `{ Invalid*, PipelineFailure }` (no `TooLarge` — inputs
    // are unbounded). `$err_ty` is retained for the `PipelineFailure`
    // path; `$invalid` names the realization's parse-rejection variant.
    macro_rules! map_addr {
        ($result:expr, $err_ty:path, $invalid:path) => {
            match $result {
                Ok(outcome) => Ok(outcome.address.as_str().to_string()),
                Err($invalid) => Err(AddressError::InvalidInput),
                Err(<$err_ty>::PipelineFailure) => Err(AddressError::PipelineFailure),
            }
        };
    }

    macro_rules! map_witness {
        ($result:expr, $err_ty:path, $invalid:path) => {
            match $result {
                Ok(outcome) => Ok(Grounded::new(GroundedImpl {
                    outcome: outcome.into(),
                })),
                Err($invalid) => Err(AddressError::InvalidInput),
                Err(<$err_ty>::PipelineFailure) => Err(AddressError::PipelineFailure),
            }
        };
    }

    // ─── κ-label composition helpers (ADR-061) ─────────────────────
    //
    // Operands arrive as κ-label strings; each axis fixes the
    // `KappaLabel<N>` width. A parse failure or a `CompositionFailure`
    // other than `PipelineFailure` (malformed operand / σ-axis mismatch)
    // maps to `invalid-input`; the WIT `address-error` surface is
    // unchanged.
    fn map_compose_failure(e: uor_addr::composition::CompositionFailure) -> AddressError {
        match e {
            uor_addr::composition::CompositionFailure::PipelineFailure => {
                AddressError::PipelineFailure
            }
            _ => AddressError::InvalidInput,
        }
    }

    /// Unary composition → κ-label.
    fn compose1_label<const N: usize, const FP: usize>(
        operand: &str,
        f: impl Fn(
            &uor_addr::KappaLabel<N>,
        )
            -> Result<uor_addr::AddressOutcome<N, FP>, uor_addr::composition::CompositionFailure>,
    ) -> Result<KappaLabel, AddressError> {
        let l = uor_addr::KappaLabel::<N>::from_bytes(operand.as_bytes())
            .map_err(|_| AddressError::InvalidInput)?;
        f(&l)
            .map(|o| o.address.as_str().to_string())
            .map_err(map_compose_failure)
    }

    /// Unary composition → verifiable witness.
    fn compose1_witness<const N: usize, const FP: usize>(
        operand: &str,
        f: impl Fn(
            &uor_addr::KappaLabel<N>,
        )
            -> Result<uor_addr::AddressOutcome<N, FP>, uor_addr::composition::CompositionFailure>,
    ) -> Result<Grounded, AddressError>
    where
        AnyOutcome: From<uor_addr::AddressOutcome<N, FP>>,
    {
        let l = uor_addr::KappaLabel::<N>::from_bytes(operand.as_bytes())
            .map_err(|_| AddressError::InvalidInput)?;
        f(&l)
            .map(|o| Grounded::new(GroundedImpl { outcome: o.into() }))
            .map_err(map_compose_failure)
    }

    /// Binary (CS-G2) composition → κ-label.
    fn compose2_label<const N: usize, const FP: usize>(
        left: &str,
        right: &str,
        f: impl Fn(
            &uor_addr::KappaLabel<N>,
            &uor_addr::KappaLabel<N>,
        )
            -> Result<uor_addr::AddressOutcome<N, FP>, uor_addr::composition::CompositionFailure>,
    ) -> Result<KappaLabel, AddressError> {
        let la = uor_addr::KappaLabel::<N>::from_bytes(left.as_bytes())
            .map_err(|_| AddressError::InvalidInput)?;
        let ra = uor_addr::KappaLabel::<N>::from_bytes(right.as_bytes())
            .map_err(|_| AddressError::InvalidInput)?;
        f(&la, &ra)
            .map(|o| o.address.as_str().to_string())
            .map_err(map_compose_failure)
    }

    /// Binary (CS-G2) composition → verifiable witness.
    fn compose2_witness<const N: usize, const FP: usize>(
        left: &str,
        right: &str,
        f: impl Fn(
            &uor_addr::KappaLabel<N>,
            &uor_addr::KappaLabel<N>,
        )
            -> Result<uor_addr::AddressOutcome<N, FP>, uor_addr::composition::CompositionFailure>,
    ) -> Result<Grounded, AddressError>
    where
        AnyOutcome: From<uor_addr::AddressOutcome<N, FP>>,
    {
        let la = uor_addr::KappaLabel::<N>::from_bytes(left.as_bytes())
            .map_err(|_| AddressError::InvalidInput)?;
        let ra = uor_addr::KappaLabel::<N>::from_bytes(right.as_bytes())
            .map_err(|_| AddressError::InvalidInput)?;
        f(&la, &ra)
            .map(|o| Grounded::new(GroundedImpl { outcome: o.into() }))
            .map_err(map_compose_failure)
    }

    impl Guest for UorAddrComponent {
        type Grounded = GroundedImpl;

        // ─── κ-label-only entry points ──────────────────────────────

        fn json_address(input: Vec<u8>) -> Result<KappaLabel, AddressError> {
            map_addr!(
                uor_addr::json::address(&input),
                uor_addr::json::AddressFailure,
                uor_addr::json::AddressFailure::InvalidJson
            )
        }

        fn sexp_address(input: Vec<u8>) -> Result<KappaLabel, AddressError> {
            map_addr!(
                uor_addr::sexp::address(&input),
                uor_addr::sexp::AddressFailure,
                uor_addr::sexp::AddressFailure::InvalidSExpr
            )
        }

        fn xml_address(input: Vec<u8>) -> Result<KappaLabel, AddressError> {
            map_addr!(
                uor_addr::xml::address(&input),
                uor_addr::xml::AddressFailure,
                uor_addr::xml::AddressFailure::InvalidXml
            )
        }

        fn asn1_address(input: Vec<u8>) -> Result<KappaLabel, AddressError> {
            map_addr!(
                uor_addr::asn1::address(&input),
                uor_addr::asn1::AddressFailure,
                uor_addr::asn1::AddressFailure::InvalidDer
            )
        }

        fn ring_address(input: Vec<u8>) -> Result<KappaLabel, AddressError> {
            map_addr!(
                uor_addr::ring::address(&input),
                uor_addr::ring::AddressFailure,
                uor_addr::ring::AddressFailure::InvalidRingElement
            )
        }

        fn codemodule_address(input: Vec<u8>) -> Result<KappaLabel, AddressError> {
            map_addr!(
                uor_addr::codemodule::address(&input),
                uor_addr::codemodule::AddressFailure,
                uor_addr::codemodule::AddressFailure::InvalidAst
            )
        }

        fn cbor_address(input: Vec<u8>) -> Result<KappaLabel, AddressError> {
            map_addr!(
                uor_addr::cbor::address(&input),
                uor_addr::cbor::AddressFailure,
                uor_addr::cbor::AddressFailure::InvalidCbor
            )
        }

        fn schema_photo_address(input: Vec<u8>) -> Result<KappaLabel, AddressError> {
            map_addr!(
                uor_addr::schema::photo::address(&input),
                uor_addr::schema::photo::AddressFailure,
                uor_addr::schema::photo::AddressFailure::SchemaViolation
            )
        }

        fn schema_document_address(input: Vec<u8>) -> Result<KappaLabel, AddressError> {
            map_addr!(
                uor_addr::schema::document::address(&input),
                uor_addr::schema::document::AddressFailure,
                uor_addr::schema::document::AddressFailure::SchemaViolation
            )
        }

        fn schema_codemodule_signed_address(input: Vec<u8>) -> Result<KappaLabel, AddressError> {
            map_addr!(
                uor_addr::schema::codemodule_signed::address(&input),
                uor_addr::schema::codemodule_signed::AddressFailure,
                uor_addr::schema::codemodule_signed::AddressFailure::SchemaViolation
            )
        }

        // ─── Witness-bearing entry points ───────────────────────────

        fn json_address_with_witness(input: Vec<u8>) -> Result<Grounded, AddressError> {
            map_witness!(
                uor_addr::json::address(&input),
                uor_addr::json::AddressFailure,
                uor_addr::json::AddressFailure::InvalidJson
            )
        }

        fn sexp_address_with_witness(input: Vec<u8>) -> Result<Grounded, AddressError> {
            map_witness!(
                uor_addr::sexp::address(&input),
                uor_addr::sexp::AddressFailure,
                uor_addr::sexp::AddressFailure::InvalidSExpr
            )
        }

        fn xml_address_with_witness(input: Vec<u8>) -> Result<Grounded, AddressError> {
            map_witness!(
                uor_addr::xml::address(&input),
                uor_addr::xml::AddressFailure,
                uor_addr::xml::AddressFailure::InvalidXml
            )
        }

        fn asn1_address_with_witness(input: Vec<u8>) -> Result<Grounded, AddressError> {
            map_witness!(
                uor_addr::asn1::address(&input),
                uor_addr::asn1::AddressFailure,
                uor_addr::asn1::AddressFailure::InvalidDer
            )
        }

        fn ring_address_with_witness(input: Vec<u8>) -> Result<Grounded, AddressError> {
            map_witness!(
                uor_addr::ring::address(&input),
                uor_addr::ring::AddressFailure,
                uor_addr::ring::AddressFailure::InvalidRingElement
            )
        }

        fn codemodule_address_with_witness(input: Vec<u8>) -> Result<Grounded, AddressError> {
            map_witness!(
                uor_addr::codemodule::address(&input),
                uor_addr::codemodule::AddressFailure,
                uor_addr::codemodule::AddressFailure::InvalidAst
            )
        }

        fn cbor_address_with_witness(input: Vec<u8>) -> Result<Grounded, AddressError> {
            map_witness!(
                uor_addr::cbor::address(&input),
                uor_addr::cbor::AddressFailure,
                uor_addr::cbor::AddressFailure::InvalidCbor
            )
        }

        fn schema_photo_address_with_witness(input: Vec<u8>) -> Result<Grounded, AddressError> {
            map_witness!(
                uor_addr::schema::photo::address(&input),
                uor_addr::schema::photo::AddressFailure,
                uor_addr::schema::photo::AddressFailure::SchemaViolation
            )
        }

        fn schema_document_address_with_witness(input: Vec<u8>) -> Result<Grounded, AddressError> {
            map_witness!(
                uor_addr::schema::document::address(&input),
                uor_addr::schema::document::AddressFailure,
                uor_addr::schema::document::AddressFailure::SchemaViolation
            )
        }

        fn schema_codemodule_signed_address_with_witness(
            input: Vec<u8>,
        ) -> Result<Grounded, AddressError> {
            map_witness!(
                uor_addr::schema::codemodule_signed::address(&input),
                uor_addr::schema::codemodule_signed::AddressFailure,
                uor_addr::schema::codemodule_signed::AddressFailure::SchemaViolation
            )
        }

        // ── GGUF v3 realization ──

        fn gguf_address(input: Vec<u8>) -> Result<KappaLabel, AddressError> {
            map_addr!(
                uor_addr::gguf::address(&input),
                uor_addr::gguf::AddressFailure,
                uor_addr::gguf::AddressFailure::InvalidGguf
            )
        }

        fn gguf_address_with_witness(input: Vec<u8>) -> Result<Grounded, AddressError> {
            map_witness!(
                uor_addr::gguf::address(&input),
                uor_addr::gguf::AddressFailure,
                uor_addr::gguf::AddressFailure::InvalidGguf
            )
        }

        // ── ONNX IR v13 realization ──

        fn onnx_address(input: Vec<u8>) -> Result<KappaLabel, AddressError> {
            map_addr!(
                uor_addr::onnx::address(&input),
                uor_addr::onnx::AddressFailure,
                uor_addr::onnx::AddressFailure::InvalidOnnx
            )
        }

        fn onnx_address_with_witness(input: Vec<u8>) -> Result<Grounded, AddressError> {
            map_witness!(
                uor_addr::onnx::address(&input),
                uor_addr::onnx::AddressFailure,
                uor_addr::onnx::AddressFailure::InvalidOnnx
            )
        }

        // ─── σ-axis-selecting entry points ──────────────────────────

        fn json_address_with_hash(
            input: Vec<u8>,
            algo: HashAlgorithm,
        ) -> Result<KappaLabel, AddressError> {
            match algo {
                HashAlgorithm::Sha256 => map_addr!(
                    uor_addr::json::address(&input),
                    uor_addr::json::AddressFailure,
                    uor_addr::json::AddressFailure::InvalidJson
                ),
                HashAlgorithm::Blake3 => map_addr!(
                    uor_addr::json::address_blake3(&input),
                    uor_addr::json::AddressFailure,
                    uor_addr::json::AddressFailure::InvalidJson
                ),
                HashAlgorithm::Sha3256 => map_addr!(
                    uor_addr::json::address_sha3_256(&input),
                    uor_addr::json::AddressFailure,
                    uor_addr::json::AddressFailure::InvalidJson
                ),
                HashAlgorithm::Keccak256 => map_addr!(
                    uor_addr::json::address_keccak256(&input),
                    uor_addr::json::AddressFailure,
                    uor_addr::json::AddressFailure::InvalidJson
                ),
                HashAlgorithm::Sha512 => map_addr!(
                    uor_addr::json::address_sha512(&input),
                    uor_addr::json::AddressFailure,
                    uor_addr::json::AddressFailure::InvalidJson
                ),
            }
        }

        fn json_address_with_witness_hash(
            input: Vec<u8>,
            algo: HashAlgorithm,
        ) -> Result<Grounded, AddressError> {
            match algo {
                HashAlgorithm::Sha256 => map_witness!(
                    uor_addr::json::address(&input),
                    uor_addr::json::AddressFailure,
                    uor_addr::json::AddressFailure::InvalidJson
                ),
                HashAlgorithm::Blake3 => map_witness!(
                    uor_addr::json::address_blake3(&input),
                    uor_addr::json::AddressFailure,
                    uor_addr::json::AddressFailure::InvalidJson
                ),
                HashAlgorithm::Sha3256 => map_witness!(
                    uor_addr::json::address_sha3_256(&input),
                    uor_addr::json::AddressFailure,
                    uor_addr::json::AddressFailure::InvalidJson
                ),
                HashAlgorithm::Keccak256 => map_witness!(
                    uor_addr::json::address_keccak256(&input),
                    uor_addr::json::AddressFailure,
                    uor_addr::json::AddressFailure::InvalidJson
                ),
                HashAlgorithm::Sha512 => map_witness!(
                    uor_addr::json::address_sha512(&input),
                    uor_addr::json::AddressFailure,
                    uor_addr::json::AddressFailure::InvalidJson
                ),
            }
        }

        fn sexp_address_with_hash(
            input: Vec<u8>,
            algo: HashAlgorithm,
        ) -> Result<KappaLabel, AddressError> {
            match algo {
                HashAlgorithm::Sha256 => map_addr!(
                    uor_addr::sexp::address(&input),
                    uor_addr::sexp::AddressFailure,
                    uor_addr::sexp::AddressFailure::InvalidSExpr
                ),
                HashAlgorithm::Blake3 => map_addr!(
                    uor_addr::sexp::address_blake3(&input),
                    uor_addr::sexp::AddressFailure,
                    uor_addr::sexp::AddressFailure::InvalidSExpr
                ),
                HashAlgorithm::Sha3256 => map_addr!(
                    uor_addr::sexp::address_sha3_256(&input),
                    uor_addr::sexp::AddressFailure,
                    uor_addr::sexp::AddressFailure::InvalidSExpr
                ),
                HashAlgorithm::Keccak256 => map_addr!(
                    uor_addr::sexp::address_keccak256(&input),
                    uor_addr::sexp::AddressFailure,
                    uor_addr::sexp::AddressFailure::InvalidSExpr
                ),
                HashAlgorithm::Sha512 => map_addr!(
                    uor_addr::sexp::address_sha512(&input),
                    uor_addr::sexp::AddressFailure,
                    uor_addr::sexp::AddressFailure::InvalidSExpr
                ),
            }
        }

        fn sexp_address_with_witness_hash(
            input: Vec<u8>,
            algo: HashAlgorithm,
        ) -> Result<Grounded, AddressError> {
            match algo {
                HashAlgorithm::Sha256 => map_witness!(
                    uor_addr::sexp::address(&input),
                    uor_addr::sexp::AddressFailure,
                    uor_addr::sexp::AddressFailure::InvalidSExpr
                ),
                HashAlgorithm::Blake3 => map_witness!(
                    uor_addr::sexp::address_blake3(&input),
                    uor_addr::sexp::AddressFailure,
                    uor_addr::sexp::AddressFailure::InvalidSExpr
                ),
                HashAlgorithm::Sha3256 => map_witness!(
                    uor_addr::sexp::address_sha3_256(&input),
                    uor_addr::sexp::AddressFailure,
                    uor_addr::sexp::AddressFailure::InvalidSExpr
                ),
                HashAlgorithm::Keccak256 => map_witness!(
                    uor_addr::sexp::address_keccak256(&input),
                    uor_addr::sexp::AddressFailure,
                    uor_addr::sexp::AddressFailure::InvalidSExpr
                ),
                HashAlgorithm::Sha512 => map_witness!(
                    uor_addr::sexp::address_sha512(&input),
                    uor_addr::sexp::AddressFailure,
                    uor_addr::sexp::AddressFailure::InvalidSExpr
                ),
            }
        }

        fn xml_address_with_hash(
            input: Vec<u8>,
            algo: HashAlgorithm,
        ) -> Result<KappaLabel, AddressError> {
            match algo {
                HashAlgorithm::Sha256 => map_addr!(
                    uor_addr::xml::address(&input),
                    uor_addr::xml::AddressFailure,
                    uor_addr::xml::AddressFailure::InvalidXml
                ),
                HashAlgorithm::Blake3 => map_addr!(
                    uor_addr::xml::address_blake3(&input),
                    uor_addr::xml::AddressFailure,
                    uor_addr::xml::AddressFailure::InvalidXml
                ),
                HashAlgorithm::Sha3256 => map_addr!(
                    uor_addr::xml::address_sha3_256(&input),
                    uor_addr::xml::AddressFailure,
                    uor_addr::xml::AddressFailure::InvalidXml
                ),
                HashAlgorithm::Keccak256 => map_addr!(
                    uor_addr::xml::address_keccak256(&input),
                    uor_addr::xml::AddressFailure,
                    uor_addr::xml::AddressFailure::InvalidXml
                ),
                HashAlgorithm::Sha512 => map_addr!(
                    uor_addr::xml::address_sha512(&input),
                    uor_addr::xml::AddressFailure,
                    uor_addr::xml::AddressFailure::InvalidXml
                ),
            }
        }

        fn xml_address_with_witness_hash(
            input: Vec<u8>,
            algo: HashAlgorithm,
        ) -> Result<Grounded, AddressError> {
            match algo {
                HashAlgorithm::Sha256 => map_witness!(
                    uor_addr::xml::address(&input),
                    uor_addr::xml::AddressFailure,
                    uor_addr::xml::AddressFailure::InvalidXml
                ),
                HashAlgorithm::Blake3 => map_witness!(
                    uor_addr::xml::address_blake3(&input),
                    uor_addr::xml::AddressFailure,
                    uor_addr::xml::AddressFailure::InvalidXml
                ),
                HashAlgorithm::Sha3256 => map_witness!(
                    uor_addr::xml::address_sha3_256(&input),
                    uor_addr::xml::AddressFailure,
                    uor_addr::xml::AddressFailure::InvalidXml
                ),
                HashAlgorithm::Keccak256 => map_witness!(
                    uor_addr::xml::address_keccak256(&input),
                    uor_addr::xml::AddressFailure,
                    uor_addr::xml::AddressFailure::InvalidXml
                ),
                HashAlgorithm::Sha512 => map_witness!(
                    uor_addr::xml::address_sha512(&input),
                    uor_addr::xml::AddressFailure,
                    uor_addr::xml::AddressFailure::InvalidXml
                ),
            }
        }

        fn asn1_address_with_hash(
            input: Vec<u8>,
            algo: HashAlgorithm,
        ) -> Result<KappaLabel, AddressError> {
            match algo {
                HashAlgorithm::Sha256 => map_addr!(
                    uor_addr::asn1::address(&input),
                    uor_addr::asn1::AddressFailure,
                    uor_addr::asn1::AddressFailure::InvalidDer
                ),
                HashAlgorithm::Blake3 => map_addr!(
                    uor_addr::asn1::address_blake3(&input),
                    uor_addr::asn1::AddressFailure,
                    uor_addr::asn1::AddressFailure::InvalidDer
                ),
                HashAlgorithm::Sha3256 => map_addr!(
                    uor_addr::asn1::address_sha3_256(&input),
                    uor_addr::asn1::AddressFailure,
                    uor_addr::asn1::AddressFailure::InvalidDer
                ),
                HashAlgorithm::Keccak256 => map_addr!(
                    uor_addr::asn1::address_keccak256(&input),
                    uor_addr::asn1::AddressFailure,
                    uor_addr::asn1::AddressFailure::InvalidDer
                ),
                HashAlgorithm::Sha512 => map_addr!(
                    uor_addr::asn1::address_sha512(&input),
                    uor_addr::asn1::AddressFailure,
                    uor_addr::asn1::AddressFailure::InvalidDer
                ),
            }
        }

        fn asn1_address_with_witness_hash(
            input: Vec<u8>,
            algo: HashAlgorithm,
        ) -> Result<Grounded, AddressError> {
            match algo {
                HashAlgorithm::Sha256 => map_witness!(
                    uor_addr::asn1::address(&input),
                    uor_addr::asn1::AddressFailure,
                    uor_addr::asn1::AddressFailure::InvalidDer
                ),
                HashAlgorithm::Blake3 => map_witness!(
                    uor_addr::asn1::address_blake3(&input),
                    uor_addr::asn1::AddressFailure,
                    uor_addr::asn1::AddressFailure::InvalidDer
                ),
                HashAlgorithm::Sha3256 => map_witness!(
                    uor_addr::asn1::address_sha3_256(&input),
                    uor_addr::asn1::AddressFailure,
                    uor_addr::asn1::AddressFailure::InvalidDer
                ),
                HashAlgorithm::Keccak256 => map_witness!(
                    uor_addr::asn1::address_keccak256(&input),
                    uor_addr::asn1::AddressFailure,
                    uor_addr::asn1::AddressFailure::InvalidDer
                ),
                HashAlgorithm::Sha512 => map_witness!(
                    uor_addr::asn1::address_sha512(&input),
                    uor_addr::asn1::AddressFailure,
                    uor_addr::asn1::AddressFailure::InvalidDer
                ),
            }
        }

        fn ring_address_with_hash(
            input: Vec<u8>,
            algo: HashAlgorithm,
        ) -> Result<KappaLabel, AddressError> {
            match algo {
                HashAlgorithm::Sha256 => map_addr!(
                    uor_addr::ring::address(&input),
                    uor_addr::ring::AddressFailure,
                    uor_addr::ring::AddressFailure::InvalidRingElement
                ),
                HashAlgorithm::Blake3 => map_addr!(
                    uor_addr::ring::address_blake3(&input),
                    uor_addr::ring::AddressFailure,
                    uor_addr::ring::AddressFailure::InvalidRingElement
                ),
                HashAlgorithm::Sha3256 => map_addr!(
                    uor_addr::ring::address_sha3_256(&input),
                    uor_addr::ring::AddressFailure,
                    uor_addr::ring::AddressFailure::InvalidRingElement
                ),
                HashAlgorithm::Keccak256 => map_addr!(
                    uor_addr::ring::address_keccak256(&input),
                    uor_addr::ring::AddressFailure,
                    uor_addr::ring::AddressFailure::InvalidRingElement
                ),
                HashAlgorithm::Sha512 => map_addr!(
                    uor_addr::ring::address_sha512(&input),
                    uor_addr::ring::AddressFailure,
                    uor_addr::ring::AddressFailure::InvalidRingElement
                ),
            }
        }

        fn ring_address_with_witness_hash(
            input: Vec<u8>,
            algo: HashAlgorithm,
        ) -> Result<Grounded, AddressError> {
            match algo {
                HashAlgorithm::Sha256 => map_witness!(
                    uor_addr::ring::address(&input),
                    uor_addr::ring::AddressFailure,
                    uor_addr::ring::AddressFailure::InvalidRingElement
                ),
                HashAlgorithm::Blake3 => map_witness!(
                    uor_addr::ring::address_blake3(&input),
                    uor_addr::ring::AddressFailure,
                    uor_addr::ring::AddressFailure::InvalidRingElement
                ),
                HashAlgorithm::Sha3256 => map_witness!(
                    uor_addr::ring::address_sha3_256(&input),
                    uor_addr::ring::AddressFailure,
                    uor_addr::ring::AddressFailure::InvalidRingElement
                ),
                HashAlgorithm::Keccak256 => map_witness!(
                    uor_addr::ring::address_keccak256(&input),
                    uor_addr::ring::AddressFailure,
                    uor_addr::ring::AddressFailure::InvalidRingElement
                ),
                HashAlgorithm::Sha512 => map_witness!(
                    uor_addr::ring::address_sha512(&input),
                    uor_addr::ring::AddressFailure,
                    uor_addr::ring::AddressFailure::InvalidRingElement
                ),
            }
        }

        fn codemodule_address_with_hash(
            input: Vec<u8>,
            algo: HashAlgorithm,
        ) -> Result<KappaLabel, AddressError> {
            match algo {
                HashAlgorithm::Sha256 => map_addr!(
                    uor_addr::codemodule::address(&input),
                    uor_addr::codemodule::AddressFailure,
                    uor_addr::codemodule::AddressFailure::InvalidAst
                ),
                HashAlgorithm::Blake3 => map_addr!(
                    uor_addr::codemodule::address_blake3(&input),
                    uor_addr::codemodule::AddressFailure,
                    uor_addr::codemodule::AddressFailure::InvalidAst
                ),
                HashAlgorithm::Sha3256 => map_addr!(
                    uor_addr::codemodule::address_sha3_256(&input),
                    uor_addr::codemodule::AddressFailure,
                    uor_addr::codemodule::AddressFailure::InvalidAst
                ),
                HashAlgorithm::Keccak256 => map_addr!(
                    uor_addr::codemodule::address_keccak256(&input),
                    uor_addr::codemodule::AddressFailure,
                    uor_addr::codemodule::AddressFailure::InvalidAst
                ),
                HashAlgorithm::Sha512 => map_addr!(
                    uor_addr::codemodule::address_sha512(&input),
                    uor_addr::codemodule::AddressFailure,
                    uor_addr::codemodule::AddressFailure::InvalidAst
                ),
            }
        }

        fn codemodule_address_with_witness_hash(
            input: Vec<u8>,
            algo: HashAlgorithm,
        ) -> Result<Grounded, AddressError> {
            match algo {
                HashAlgorithm::Sha256 => map_witness!(
                    uor_addr::codemodule::address(&input),
                    uor_addr::codemodule::AddressFailure,
                    uor_addr::codemodule::AddressFailure::InvalidAst
                ),
                HashAlgorithm::Blake3 => map_witness!(
                    uor_addr::codemodule::address_blake3(&input),
                    uor_addr::codemodule::AddressFailure,
                    uor_addr::codemodule::AddressFailure::InvalidAst
                ),
                HashAlgorithm::Sha3256 => map_witness!(
                    uor_addr::codemodule::address_sha3_256(&input),
                    uor_addr::codemodule::AddressFailure,
                    uor_addr::codemodule::AddressFailure::InvalidAst
                ),
                HashAlgorithm::Keccak256 => map_witness!(
                    uor_addr::codemodule::address_keccak256(&input),
                    uor_addr::codemodule::AddressFailure,
                    uor_addr::codemodule::AddressFailure::InvalidAst
                ),
                HashAlgorithm::Sha512 => map_witness!(
                    uor_addr::codemodule::address_sha512(&input),
                    uor_addr::codemodule::AddressFailure,
                    uor_addr::codemodule::AddressFailure::InvalidAst
                ),
            }
        }

        fn cbor_address_with_hash(
            input: Vec<u8>,
            algo: HashAlgorithm,
        ) -> Result<KappaLabel, AddressError> {
            match algo {
                HashAlgorithm::Sha256 => map_addr!(
                    uor_addr::cbor::address(&input),
                    uor_addr::cbor::AddressFailure,
                    uor_addr::cbor::AddressFailure::InvalidCbor
                ),
                HashAlgorithm::Blake3 => map_addr!(
                    uor_addr::cbor::address_blake3(&input),
                    uor_addr::cbor::AddressFailure,
                    uor_addr::cbor::AddressFailure::InvalidCbor
                ),
                HashAlgorithm::Sha3256 => map_addr!(
                    uor_addr::cbor::address_sha3_256(&input),
                    uor_addr::cbor::AddressFailure,
                    uor_addr::cbor::AddressFailure::InvalidCbor
                ),
                HashAlgorithm::Keccak256 => map_addr!(
                    uor_addr::cbor::address_keccak256(&input),
                    uor_addr::cbor::AddressFailure,
                    uor_addr::cbor::AddressFailure::InvalidCbor
                ),
                HashAlgorithm::Sha512 => map_addr!(
                    uor_addr::cbor::address_sha512(&input),
                    uor_addr::cbor::AddressFailure,
                    uor_addr::cbor::AddressFailure::InvalidCbor
                ),
            }
        }

        fn cbor_address_with_witness_hash(
            input: Vec<u8>,
            algo: HashAlgorithm,
        ) -> Result<Grounded, AddressError> {
            match algo {
                HashAlgorithm::Sha256 => map_witness!(
                    uor_addr::cbor::address(&input),
                    uor_addr::cbor::AddressFailure,
                    uor_addr::cbor::AddressFailure::InvalidCbor
                ),
                HashAlgorithm::Blake3 => map_witness!(
                    uor_addr::cbor::address_blake3(&input),
                    uor_addr::cbor::AddressFailure,
                    uor_addr::cbor::AddressFailure::InvalidCbor
                ),
                HashAlgorithm::Sha3256 => map_witness!(
                    uor_addr::cbor::address_sha3_256(&input),
                    uor_addr::cbor::AddressFailure,
                    uor_addr::cbor::AddressFailure::InvalidCbor
                ),
                HashAlgorithm::Keccak256 => map_witness!(
                    uor_addr::cbor::address_keccak256(&input),
                    uor_addr::cbor::AddressFailure,
                    uor_addr::cbor::AddressFailure::InvalidCbor
                ),
                HashAlgorithm::Sha512 => map_witness!(
                    uor_addr::cbor::address_sha512(&input),
                    uor_addr::cbor::AddressFailure,
                    uor_addr::cbor::AddressFailure::InvalidCbor
                ),
            }
        }

        fn schema_photo_address_with_hash(
            input: Vec<u8>,
            algo: HashAlgorithm,
        ) -> Result<KappaLabel, AddressError> {
            match algo {
                HashAlgorithm::Sha256 => map_addr!(
                    uor_addr::schema::photo::address(&input),
                    uor_addr::schema::photo::AddressFailure,
                    uor_addr::schema::photo::AddressFailure::SchemaViolation
                ),
                HashAlgorithm::Blake3 => map_addr!(
                    uor_addr::schema::photo::address_blake3(&input),
                    uor_addr::schema::photo::AddressFailure,
                    uor_addr::schema::photo::AddressFailure::SchemaViolation
                ),
                HashAlgorithm::Sha3256 => map_addr!(
                    uor_addr::schema::photo::address_sha3_256(&input),
                    uor_addr::schema::photo::AddressFailure,
                    uor_addr::schema::photo::AddressFailure::SchemaViolation
                ),
                HashAlgorithm::Keccak256 => map_addr!(
                    uor_addr::schema::photo::address_keccak256(&input),
                    uor_addr::schema::photo::AddressFailure,
                    uor_addr::schema::photo::AddressFailure::SchemaViolation
                ),
                HashAlgorithm::Sha512 => map_addr!(
                    uor_addr::schema::photo::address_sha512(&input),
                    uor_addr::schema::photo::AddressFailure,
                    uor_addr::schema::photo::AddressFailure::SchemaViolation
                ),
            }
        }

        fn schema_photo_address_with_witness_hash(
            input: Vec<u8>,
            algo: HashAlgorithm,
        ) -> Result<Grounded, AddressError> {
            match algo {
                HashAlgorithm::Sha256 => map_witness!(
                    uor_addr::schema::photo::address(&input),
                    uor_addr::schema::photo::AddressFailure,
                    uor_addr::schema::photo::AddressFailure::SchemaViolation
                ),
                HashAlgorithm::Blake3 => map_witness!(
                    uor_addr::schema::photo::address_blake3(&input),
                    uor_addr::schema::photo::AddressFailure,
                    uor_addr::schema::photo::AddressFailure::SchemaViolation
                ),
                HashAlgorithm::Sha3256 => map_witness!(
                    uor_addr::schema::photo::address_sha3_256(&input),
                    uor_addr::schema::photo::AddressFailure,
                    uor_addr::schema::photo::AddressFailure::SchemaViolation
                ),
                HashAlgorithm::Keccak256 => map_witness!(
                    uor_addr::schema::photo::address_keccak256(&input),
                    uor_addr::schema::photo::AddressFailure,
                    uor_addr::schema::photo::AddressFailure::SchemaViolation
                ),
                HashAlgorithm::Sha512 => map_witness!(
                    uor_addr::schema::photo::address_sha512(&input),
                    uor_addr::schema::photo::AddressFailure,
                    uor_addr::schema::photo::AddressFailure::SchemaViolation
                ),
            }
        }

        fn schema_document_address_with_hash(
            input: Vec<u8>,
            algo: HashAlgorithm,
        ) -> Result<KappaLabel, AddressError> {
            match algo {
                HashAlgorithm::Sha256 => map_addr!(
                    uor_addr::schema::document::address(&input),
                    uor_addr::schema::document::AddressFailure,
                    uor_addr::schema::document::AddressFailure::SchemaViolation
                ),
                HashAlgorithm::Blake3 => map_addr!(
                    uor_addr::schema::document::address_blake3(&input),
                    uor_addr::schema::document::AddressFailure,
                    uor_addr::schema::document::AddressFailure::SchemaViolation
                ),
                HashAlgorithm::Sha3256 => map_addr!(
                    uor_addr::schema::document::address_sha3_256(&input),
                    uor_addr::schema::document::AddressFailure,
                    uor_addr::schema::document::AddressFailure::SchemaViolation
                ),
                HashAlgorithm::Keccak256 => map_addr!(
                    uor_addr::schema::document::address_keccak256(&input),
                    uor_addr::schema::document::AddressFailure,
                    uor_addr::schema::document::AddressFailure::SchemaViolation
                ),
                HashAlgorithm::Sha512 => map_addr!(
                    uor_addr::schema::document::address_sha512(&input),
                    uor_addr::schema::document::AddressFailure,
                    uor_addr::schema::document::AddressFailure::SchemaViolation
                ),
            }
        }

        fn schema_document_address_with_witness_hash(
            input: Vec<u8>,
            algo: HashAlgorithm,
        ) -> Result<Grounded, AddressError> {
            match algo {
                HashAlgorithm::Sha256 => map_witness!(
                    uor_addr::schema::document::address(&input),
                    uor_addr::schema::document::AddressFailure,
                    uor_addr::schema::document::AddressFailure::SchemaViolation
                ),
                HashAlgorithm::Blake3 => map_witness!(
                    uor_addr::schema::document::address_blake3(&input),
                    uor_addr::schema::document::AddressFailure,
                    uor_addr::schema::document::AddressFailure::SchemaViolation
                ),
                HashAlgorithm::Sha3256 => map_witness!(
                    uor_addr::schema::document::address_sha3_256(&input),
                    uor_addr::schema::document::AddressFailure,
                    uor_addr::schema::document::AddressFailure::SchemaViolation
                ),
                HashAlgorithm::Keccak256 => map_witness!(
                    uor_addr::schema::document::address_keccak256(&input),
                    uor_addr::schema::document::AddressFailure,
                    uor_addr::schema::document::AddressFailure::SchemaViolation
                ),
                HashAlgorithm::Sha512 => map_witness!(
                    uor_addr::schema::document::address_sha512(&input),
                    uor_addr::schema::document::AddressFailure,
                    uor_addr::schema::document::AddressFailure::SchemaViolation
                ),
            }
        }

        fn schema_codemodule_signed_address_with_hash(
            input: Vec<u8>,
            algo: HashAlgorithm,
        ) -> Result<KappaLabel, AddressError> {
            match algo {
                HashAlgorithm::Sha256 => map_addr!(
                    uor_addr::schema::codemodule_signed::address(&input),
                    uor_addr::schema::codemodule_signed::AddressFailure,
                    uor_addr::schema::codemodule_signed::AddressFailure::SchemaViolation
                ),
                HashAlgorithm::Blake3 => map_addr!(
                    uor_addr::schema::codemodule_signed::address_blake3(&input),
                    uor_addr::schema::codemodule_signed::AddressFailure,
                    uor_addr::schema::codemodule_signed::AddressFailure::SchemaViolation
                ),
                HashAlgorithm::Sha3256 => map_addr!(
                    uor_addr::schema::codemodule_signed::address_sha3_256(&input),
                    uor_addr::schema::codemodule_signed::AddressFailure,
                    uor_addr::schema::codemodule_signed::AddressFailure::SchemaViolation
                ),
                HashAlgorithm::Keccak256 => map_addr!(
                    uor_addr::schema::codemodule_signed::address_keccak256(&input),
                    uor_addr::schema::codemodule_signed::AddressFailure,
                    uor_addr::schema::codemodule_signed::AddressFailure::SchemaViolation
                ),
                HashAlgorithm::Sha512 => map_addr!(
                    uor_addr::schema::codemodule_signed::address_sha512(&input),
                    uor_addr::schema::codemodule_signed::AddressFailure,
                    uor_addr::schema::codemodule_signed::AddressFailure::SchemaViolation
                ),
            }
        }

        fn schema_codemodule_signed_address_with_witness_hash(
            input: Vec<u8>,
            algo: HashAlgorithm,
        ) -> Result<Grounded, AddressError> {
            match algo {
                HashAlgorithm::Sha256 => map_witness!(
                    uor_addr::schema::codemodule_signed::address(&input),
                    uor_addr::schema::codemodule_signed::AddressFailure,
                    uor_addr::schema::codemodule_signed::AddressFailure::SchemaViolation
                ),
                HashAlgorithm::Blake3 => map_witness!(
                    uor_addr::schema::codemodule_signed::address_blake3(&input),
                    uor_addr::schema::codemodule_signed::AddressFailure,
                    uor_addr::schema::codemodule_signed::AddressFailure::SchemaViolation
                ),
                HashAlgorithm::Sha3256 => map_witness!(
                    uor_addr::schema::codemodule_signed::address_sha3_256(&input),
                    uor_addr::schema::codemodule_signed::AddressFailure,
                    uor_addr::schema::codemodule_signed::AddressFailure::SchemaViolation
                ),
                HashAlgorithm::Keccak256 => map_witness!(
                    uor_addr::schema::codemodule_signed::address_keccak256(&input),
                    uor_addr::schema::codemodule_signed::AddressFailure,
                    uor_addr::schema::codemodule_signed::AddressFailure::SchemaViolation
                ),
                HashAlgorithm::Sha512 => map_witness!(
                    uor_addr::schema::codemodule_signed::address_sha512(&input),
                    uor_addr::schema::codemodule_signed::AddressFailure,
                    uor_addr::schema::codemodule_signed::AddressFailure::SchemaViolation
                ),
            }
        }

        fn gguf_address_with_hash(
            input: Vec<u8>,
            algo: HashAlgorithm,
        ) -> Result<KappaLabel, AddressError> {
            match algo {
                HashAlgorithm::Sha256 => map_addr!(
                    uor_addr::gguf::address(&input),
                    uor_addr::gguf::AddressFailure,
                    uor_addr::gguf::AddressFailure::InvalidGguf
                ),
                HashAlgorithm::Blake3 => map_addr!(
                    uor_addr::gguf::address_blake3(&input),
                    uor_addr::gguf::AddressFailure,
                    uor_addr::gguf::AddressFailure::InvalidGguf
                ),
                HashAlgorithm::Sha3256 => map_addr!(
                    uor_addr::gguf::address_sha3_256(&input),
                    uor_addr::gguf::AddressFailure,
                    uor_addr::gguf::AddressFailure::InvalidGguf
                ),
                HashAlgorithm::Keccak256 => map_addr!(
                    uor_addr::gguf::address_keccak256(&input),
                    uor_addr::gguf::AddressFailure,
                    uor_addr::gguf::AddressFailure::InvalidGguf
                ),
                HashAlgorithm::Sha512 => map_addr!(
                    uor_addr::gguf::address_sha512(&input),
                    uor_addr::gguf::AddressFailure,
                    uor_addr::gguf::AddressFailure::InvalidGguf
                ),
            }
        }

        fn gguf_address_with_witness_hash(
            input: Vec<u8>,
            algo: HashAlgorithm,
        ) -> Result<Grounded, AddressError> {
            match algo {
                HashAlgorithm::Sha256 => map_witness!(
                    uor_addr::gguf::address(&input),
                    uor_addr::gguf::AddressFailure,
                    uor_addr::gguf::AddressFailure::InvalidGguf
                ),
                HashAlgorithm::Blake3 => map_witness!(
                    uor_addr::gguf::address_blake3(&input),
                    uor_addr::gguf::AddressFailure,
                    uor_addr::gguf::AddressFailure::InvalidGguf
                ),
                HashAlgorithm::Sha3256 => map_witness!(
                    uor_addr::gguf::address_sha3_256(&input),
                    uor_addr::gguf::AddressFailure,
                    uor_addr::gguf::AddressFailure::InvalidGguf
                ),
                HashAlgorithm::Keccak256 => map_witness!(
                    uor_addr::gguf::address_keccak256(&input),
                    uor_addr::gguf::AddressFailure,
                    uor_addr::gguf::AddressFailure::InvalidGguf
                ),
                HashAlgorithm::Sha512 => map_witness!(
                    uor_addr::gguf::address_sha512(&input),
                    uor_addr::gguf::AddressFailure,
                    uor_addr::gguf::AddressFailure::InvalidGguf
                ),
            }
        }

        fn onnx_address_with_hash(
            input: Vec<u8>,
            algo: HashAlgorithm,
        ) -> Result<KappaLabel, AddressError> {
            match algo {
                HashAlgorithm::Sha256 => map_addr!(
                    uor_addr::onnx::address(&input),
                    uor_addr::onnx::AddressFailure,
                    uor_addr::onnx::AddressFailure::InvalidOnnx
                ),
                HashAlgorithm::Blake3 => map_addr!(
                    uor_addr::onnx::address_blake3(&input),
                    uor_addr::onnx::AddressFailure,
                    uor_addr::onnx::AddressFailure::InvalidOnnx
                ),
                HashAlgorithm::Sha3256 => map_addr!(
                    uor_addr::onnx::address_sha3_256(&input),
                    uor_addr::onnx::AddressFailure,
                    uor_addr::onnx::AddressFailure::InvalidOnnx
                ),
                HashAlgorithm::Keccak256 => map_addr!(
                    uor_addr::onnx::address_keccak256(&input),
                    uor_addr::onnx::AddressFailure,
                    uor_addr::onnx::AddressFailure::InvalidOnnx
                ),
                HashAlgorithm::Sha512 => map_addr!(
                    uor_addr::onnx::address_sha512(&input),
                    uor_addr::onnx::AddressFailure,
                    uor_addr::onnx::AddressFailure::InvalidOnnx
                ),
            }
        }

        fn onnx_address_with_witness_hash(
            input: Vec<u8>,
            algo: HashAlgorithm,
        ) -> Result<Grounded, AddressError> {
            match algo {
                HashAlgorithm::Sha256 => map_witness!(
                    uor_addr::onnx::address(&input),
                    uor_addr::onnx::AddressFailure,
                    uor_addr::onnx::AddressFailure::InvalidOnnx
                ),
                HashAlgorithm::Blake3 => map_witness!(
                    uor_addr::onnx::address_blake3(&input),
                    uor_addr::onnx::AddressFailure,
                    uor_addr::onnx::AddressFailure::InvalidOnnx
                ),
                HashAlgorithm::Sha3256 => map_witness!(
                    uor_addr::onnx::address_sha3_256(&input),
                    uor_addr::onnx::AddressFailure,
                    uor_addr::onnx::AddressFailure::InvalidOnnx
                ),
                HashAlgorithm::Keccak256 => map_witness!(
                    uor_addr::onnx::address_keccak256(&input),
                    uor_addr::onnx::AddressFailure,
                    uor_addr::onnx::AddressFailure::InvalidOnnx
                ),
                HashAlgorithm::Sha512 => map_witness!(
                    uor_addr::onnx::address_sha512(&input),
                    uor_addr::onnx::AddressFailure,
                    uor_addr::onnx::AddressFailure::InvalidOnnx
                ),
            }
        }

        // ─── κ-label composition (ADR-061) ──────────────────────────
        // CS-G2 (binary commutative product) + four unary endomorphisms.

        fn compose_g2(
            left: KappaLabel,
            right: KappaLabel,
            algo: HashAlgorithm,
        ) -> Result<KappaLabel, AddressError> {
            use uor_addr::composition as c;
            match algo {
                HashAlgorithm::Sha256 => compose2_label(&left, &right, c::compose_g2_product),
                HashAlgorithm::Blake3 => {
                    compose2_label(&left, &right, c::compose_g2_product_blake3)
                }
                HashAlgorithm::Sha3256 => {
                    compose2_label(&left, &right, c::compose_g2_product_sha3_256)
                }
                HashAlgorithm::Keccak256 => {
                    compose2_label(&left, &right, c::compose_g2_product_keccak256)
                }
                HashAlgorithm::Sha512 => {
                    compose2_label(&left, &right, c::compose_g2_product_sha512)
                }
            }
        }

        fn compose_g2_with_witness(
            left: KappaLabel,
            right: KappaLabel,
            algo: HashAlgorithm,
        ) -> Result<Grounded, AddressError> {
            use uor_addr::composition as c;
            match algo {
                HashAlgorithm::Sha256 => compose2_witness(&left, &right, c::compose_g2_product),
                HashAlgorithm::Blake3 => {
                    compose2_witness(&left, &right, c::compose_g2_product_blake3)
                }
                HashAlgorithm::Sha3256 => {
                    compose2_witness(&left, &right, c::compose_g2_product_sha3_256)
                }
                HashAlgorithm::Keccak256 => {
                    compose2_witness(&left, &right, c::compose_g2_product_keccak256)
                }
                HashAlgorithm::Sha512 => {
                    compose2_witness(&left, &right, c::compose_g2_product_sha512)
                }
            }
        }

        fn compose_f4(
            operand: KappaLabel,
            algo: HashAlgorithm,
        ) -> Result<KappaLabel, AddressError> {
            use uor_addr::composition as c;
            match algo {
                HashAlgorithm::Sha256 => compose1_label(&operand, c::compose_f4_quotient),
                HashAlgorithm::Blake3 => compose1_label(&operand, c::compose_f4_quotient_blake3),
                HashAlgorithm::Sha3256 => compose1_label(&operand, c::compose_f4_quotient_sha3_256),
                HashAlgorithm::Keccak256 => {
                    compose1_label(&operand, c::compose_f4_quotient_keccak256)
                }
                HashAlgorithm::Sha512 => compose1_label(&operand, c::compose_f4_quotient_sha512),
            }
        }

        fn compose_f4_with_witness(
            operand: KappaLabel,
            algo: HashAlgorithm,
        ) -> Result<Grounded, AddressError> {
            use uor_addr::composition as c;
            match algo {
                HashAlgorithm::Sha256 => compose1_witness(&operand, c::compose_f4_quotient),
                HashAlgorithm::Blake3 => compose1_witness(&operand, c::compose_f4_quotient_blake3),
                HashAlgorithm::Sha3256 => {
                    compose1_witness(&operand, c::compose_f4_quotient_sha3_256)
                }
                HashAlgorithm::Keccak256 => {
                    compose1_witness(&operand, c::compose_f4_quotient_keccak256)
                }
                HashAlgorithm::Sha512 => compose1_witness(&operand, c::compose_f4_quotient_sha512),
            }
        }

        fn compose_e6(
            operand: KappaLabel,
            algo: HashAlgorithm,
        ) -> Result<KappaLabel, AddressError> {
            use uor_addr::composition as c;
            match algo {
                HashAlgorithm::Sha256 => compose1_label(&operand, c::compose_e6_filtration),
                HashAlgorithm::Blake3 => compose1_label(&operand, c::compose_e6_filtration_blake3),
                HashAlgorithm::Sha3256 => {
                    compose1_label(&operand, c::compose_e6_filtration_sha3_256)
                }
                HashAlgorithm::Keccak256 => {
                    compose1_label(&operand, c::compose_e6_filtration_keccak256)
                }
                HashAlgorithm::Sha512 => compose1_label(&operand, c::compose_e6_filtration_sha512),
            }
        }

        fn compose_e6_with_witness(
            operand: KappaLabel,
            algo: HashAlgorithm,
        ) -> Result<Grounded, AddressError> {
            use uor_addr::composition as c;
            match algo {
                HashAlgorithm::Sha256 => compose1_witness(&operand, c::compose_e6_filtration),
                HashAlgorithm::Blake3 => {
                    compose1_witness(&operand, c::compose_e6_filtration_blake3)
                }
                HashAlgorithm::Sha3256 => {
                    compose1_witness(&operand, c::compose_e6_filtration_sha3_256)
                }
                HashAlgorithm::Keccak256 => {
                    compose1_witness(&operand, c::compose_e6_filtration_keccak256)
                }
                HashAlgorithm::Sha512 => {
                    compose1_witness(&operand, c::compose_e6_filtration_sha512)
                }
            }
        }

        fn compose_e7(
            operand: KappaLabel,
            algo: HashAlgorithm,
        ) -> Result<KappaLabel, AddressError> {
            use uor_addr::composition as c;
            match algo {
                HashAlgorithm::Sha256 => compose1_label(&operand, c::compose_e7_augmentation),
                HashAlgorithm::Blake3 => {
                    compose1_label(&operand, c::compose_e7_augmentation_blake3)
                }
                HashAlgorithm::Sha3256 => {
                    compose1_label(&operand, c::compose_e7_augmentation_sha3_256)
                }
                HashAlgorithm::Keccak256 => {
                    compose1_label(&operand, c::compose_e7_augmentation_keccak256)
                }
                HashAlgorithm::Sha512 => {
                    compose1_label(&operand, c::compose_e7_augmentation_sha512)
                }
            }
        }

        fn compose_e7_with_witness(
            operand: KappaLabel,
            algo: HashAlgorithm,
        ) -> Result<Grounded, AddressError> {
            use uor_addr::composition as c;
            match algo {
                HashAlgorithm::Sha256 => compose1_witness(&operand, c::compose_e7_augmentation),
                HashAlgorithm::Blake3 => {
                    compose1_witness(&operand, c::compose_e7_augmentation_blake3)
                }
                HashAlgorithm::Sha3256 => {
                    compose1_witness(&operand, c::compose_e7_augmentation_sha3_256)
                }
                HashAlgorithm::Keccak256 => {
                    compose1_witness(&operand, c::compose_e7_augmentation_keccak256)
                }
                HashAlgorithm::Sha512 => {
                    compose1_witness(&operand, c::compose_e7_augmentation_sha512)
                }
            }
        }

        fn compose_e8(
            operand: KappaLabel,
            algo: HashAlgorithm,
        ) -> Result<KappaLabel, AddressError> {
            use uor_addr::composition as c;
            match algo {
                HashAlgorithm::Sha256 => compose1_label(&operand, c::compose_e8_embedding),
                HashAlgorithm::Blake3 => compose1_label(&operand, c::compose_e8_embedding_blake3),
                HashAlgorithm::Sha3256 => {
                    compose1_label(&operand, c::compose_e8_embedding_sha3_256)
                }
                HashAlgorithm::Keccak256 => {
                    compose1_label(&operand, c::compose_e8_embedding_keccak256)
                }
                HashAlgorithm::Sha512 => compose1_label(&operand, c::compose_e8_embedding_sha512),
            }
        }

        fn compose_e8_with_witness(
            operand: KappaLabel,
            algo: HashAlgorithm,
        ) -> Result<Grounded, AddressError> {
            use uor_addr::composition as c;
            match algo {
                HashAlgorithm::Sha256 => compose1_witness(&operand, c::compose_e8_embedding),
                HashAlgorithm::Blake3 => compose1_witness(&operand, c::compose_e8_embedding_blake3),
                HashAlgorithm::Sha3256 => {
                    compose1_witness(&operand, c::compose_e8_embedding_sha3_256)
                }
                HashAlgorithm::Keccak256 => {
                    compose1_witness(&operand, c::compose_e8_embedding_keccak256)
                }
                HashAlgorithm::Sha512 => compose1_witness(&operand, c::compose_e8_embedding_sha512),
            }
        }
    }

    export!(UorAddrComponent);
}
