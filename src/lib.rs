//! # UOR-aligned R⁴ Tangent Space Router — facade crate
//!
//! One dependency for library users. Transformerless inference is a core R⁴
//! capability; routing, witnessed inference, and application use cases share
//! this public surface:
//!
//! - [`uor_r4_core`]: R⁴ mathematics and the integrated transformerless
//!   compiler/runtime — zeta-zero embeddings, table-native inference,
//!   certification, tokenization, and source-model adapters.
//! - [`uor_r4_router`]: the engine plus its UOR witness layer —
//!   [`UorR4Router`] state, manifold indexing, geometric generation, the
//!   routing axis (`R4Axis`), shapes, and the `UorR4RouterModel` PrismModel
//!   (wasm-bindgen surface included).
//! - [`transformerless`]: R⁴'s integrated multiplication-free, table-native
//!   local inference implementation.
//! - [`tless_uor`]: R4's UOR addressing and `Grounded` witness surface for
//!   transformerless inference.
//!
//! Every path the binary and previous consumers used is preserved at the
//! crate root; [`prelude`] is the ergonomic one-import surface.

pub use uor_r4_core;
pub use uor_r4_core::transformerless;
pub use uor_r4_router;

pub use uor_r4_core::*;
pub use uor_r4_router::*;

pub mod tless_uor;

/// #839 phase 1 (RF-30): the shared typed selective-prediction surface
/// vocabulary — deliberately ungated so the WASM boundary and the native
/// server read identical labels.
pub mod selective;

#[cfg(not(target_arch = "wasm32"))]
pub mod chat;
#[cfg(not(target_arch = "wasm32"))]
pub mod model;
#[cfg(not(target_arch = "wasm32"))]
pub mod r4g1;
/// #655-D2: `release-bundle.json`'s shared filename constant is `pub`
/// (rather than `pub(crate)`) because the CLI packaging command in the
/// `r4` binary crate (`src/main.rs`) needs to write to the exact path
/// this module's own loader reads from.
#[cfg(not(target_arch = "wasm32"))]
pub mod release_bundle_loader;
/// #655-D2: `pub` (rather than `pub(crate)`) so the `r4` binary crate's
/// `package-release-bundle` CLI command can call
/// [`release_bundle_packager::package_release_bundle`].
#[cfg(not(target_arch = "wasm32"))]
pub mod release_bundle_packager;
/// #741: the explicit, verified release-bundle fetch (`r4
/// install-release`) — downloads a published GitHub Release's bundle
/// assets and installs them only after every declared component digest
/// matches (`docs/RELEASE_PIPELINE.md`).
#[cfg(not(target_arch = "wasm32"))]
pub mod release_install;
#[cfg(not(target_arch = "wasm32"))]
pub mod telemetry;

/// Native HTTP server and terminal chat application.
///
/// The binary is intentionally a tiny wrapper around [`server::run`], which
/// keeps the complete application available for embedding and testing.
#[cfg(not(target_arch = "wasm32"))]
pub mod server;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn generate_r4g1_response(prompt: &str, max_tokens: usize) -> Option<String> {
    tless_uor::generate_r4g1_response(prompt, max_tokens)
}

/// #839 phase 1 (RF-30): the typed selective-prediction boundary export
/// (spec §5, WASM row) — always a typed JSON value with the canonical
/// labels, never a trap; see [`tless_uor::typed_r4g1_response`]. The legacy
/// `Option<String>` export above is retained unchanged.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn typed_r4g1_response(prompt: &str, max_tokens: usize) -> String {
    tless_uor::typed_r4g1_response(prompt, max_tokens)
}

/// #790 item 5: install a graph and its exact tokenizer into the wasm
/// runtime so the dashboard's r4g1/transformerless selections can
/// actually serve through [`generate_r4g1_response`] in static mode.
/// Previously that export was unreachable — no installer was exported
/// and neither frontend assigned the `wasm_module` global it is gated
/// on, so those selections silently took the geometric fallback. Throws
/// the installer's typed refusal (CID mismatch, malformed bytes) without
/// replacing a previously active bundle.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn set_r4g1_bundle(graph: Vec<u8>, tokenizer: Vec<u8>) -> Result<(), JsValue> {
    tless_uor::set_r4g1_bundle(graph, tokenizer).map_err(|error| JsValue::from_str(&error))
}

/// The one-import surface for library users.
pub mod prelude {
    pub use crate::tless_uor;
    pub use uor_r4_core::transformerless::runtime::OpKernel;
    pub use uor_r4_core::{R4Vector, ALPHA_4, ALPHA_5};
    pub use uor_r4_router::{
        GeometricResponse, R4HostBounds, R4RoutingInput, R4RoutingOutput, RoutingData, UorR4Router,
        UorR4RouterModel, ACTIVE_ROUTER,
    };
}

/// Fold sequence of bytes into 256D Bott Periodic Fock state matrix.
pub fn cd_space_fold(text: &str) -> [i16; 256] {
    use uor_r4_core::transformerless::bott_fock::BottFockContextStore;
    use uor_r4_core::transformerless::cd_space::{
        CayleyDicksonVector, ComplexNumber, Octonion, Quaternion,
    };
    let mut store = BottFockContextStore::new();
    for &byte in text.as_bytes() {
        let oct = Octonion::imaginary((byte % 7 + 1) as usize);
        let vec = CayleyDicksonVector::embed(
            &oct,
            &Quaternion::default(),
            &ComplexNumber::default(),
            0.0,
            0.0,
        );
        let mut token = [0i16; 16];
        for (t, &v) in token.iter_mut().zip(&vec.components) {
            *t = (v * 1000.0) as i16;
        }
        store.append_token(&token);
    }
    *store.state()
}

#[cfg(test)]
mod facade_smoke_tests {
    #[test]
    fn reexport_paths_resolve() {
        let _ = core::any::type_name::<crate::UorR4Router>();
        let _ = core::any::type_name::<crate::RoutingData>();
        let _ = core::any::type_name::<crate::R4Vector>();
        let _ = core::any::type_name::<crate::UorR4RouterModel>();
        let _ = core::any::type_name::<crate::tless_uor::UorTlessModel>();
        let _ = core::any::type_name::<crate::prelude::UorR4Router>();
        const { assert!(crate::prelude::ALPHA_4 > 0.0) };
    }

    #[test]
    fn native_inference_has_no_external_provider_client() {
        let manifest = include_str!("../Cargo.toml");
        let runtime_sources = concat!(
            include_str!("chat.rs"),
            include_str!("server.rs"),
            include_str!("main.rs"),
        );
        for dependency in ["reqwest", "ureq", "ollama-rs", "async-openai", "anthropic"] {
            assert!(
                !manifest.contains(dependency),
                "external provider dependency is forbidden: {dependency}"
            );
        }
        for forbidden in [
            "TcpStream::connect(",
            "api.openai.com",
            "api.anthropic.com",
            "Command::new(\"ollama\")",
            "Command::new(\"llama-cli\")",
        ] {
            assert!(
                !runtime_sources.contains(forbidden),
                "external inference path is forbidden: {forbidden}"
            );
        }
    }
}
