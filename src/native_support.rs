//! Native support surface of the facade crate (legacy-crate repair, 19 Oct 2026).
//!
//! The historical `lib.rs` and the `r4` binary (`main.rs`) serve R4G1 through
//! the retired `transformerless::{compiler, runtime}` (normalized-exponential
//! teacher sampling, dense dot-argmax assignment), compile the `r4_*`
//! teacher-trace and reference-generation experiments on the R4/Spin
//! normalized-exponential executor in `uor-r4-model-source`, and run a dense ReLU
//! relation head
//! (`r4_source_relation_head`). Those paths are banned in every compiled module
//! (D23/D25/D29/D30) and stay preserved in place as uncompiled evidence; the
//! R4G1 and wasm-router mechanisms stay PARKED (D22, D36). This surface
//! re-exports the current core and router and compiles only the modules that
//! need none of them. No Cargo feature re-enables the historical surface.

pub use uor_r4_core;
pub use uor_r4_core::transformerless;
pub use uor_r4_router;

pub use uor_r4_core::*;
pub use uor_r4_router::*;

pub mod r4_source_span_pointer;
pub mod telemetry;

/// The one-import surface for library users.
pub mod prelude {
    pub use uor_r4_router::R4HostBounds;
}

/// Static checks carried over from the historical `lib.rs` facade tests: they
/// read source text and need no excluded module.
#[cfg(test)]
mod facade_smoke_tests {
    #[test]
    fn reexport_paths_resolve() {
        let _ = core::any::type_name::<crate::R4HostBounds>();
        let _ = core::any::type_name::<crate::TokenHistorySignature>();
        let _ = core::any::type_name::<crate::prelude::R4HostBounds>();
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

    #[test]
    fn active_studio_worker_has_no_implicit_model_or_provider_fallback() {
        let worker = include_str!("../web/studio/assets/js/uor_model_worker.js");
        assert!(!worker.contains("native_geometric_init(new Uint8Array())"));
        assert!(!worker.contains("UorR4Router"));
        assert!(!worker.contains("@huggingface/transformers"));
        assert!(!worker.contains("computeHopfTelemetry"));
    }
}
