//! #655-D1: an additive, pure helper that builds a `ReleaseBundleManifest`
//! by digesting an already-compiled R4G1 bundle's files on disk
//! (`docs/serving_release_packaging_655_d.md`, #655-D0).
//!
//! [`package_release_bundle`] reads and blake3-hashes the real component
//! files the existing CLI `compile` / `transformerless cover` /
//! `transformerless score` pipeline already produces inside a bundle's
//! `physical_root` -- it runs no compile/cover/score stage itself and
//! does not route through `uor_r4_api::compile` (see the design doc for
//! why that path's divergent `work_dir` layout makes it unsuitable here).
//! It is pure construction: it does not write a `release-bundle.json`
//! sidecar to disk (that is #655-D2, which will call this helper and
//! `std::fs::write` its result next to `physical_root`, giving
//! `release_bundle_loader::verify_release_bundle_sidecar` (#655-C1c) its
//! first real input).
//!
//! `model_id`, `capability`, `uor_matmul`, `tokenizer_adapter`, and
//! `provenance_note` are caller-supplied ([`PackageInputs`]), not derived
//! from `physical_root` -- none of them has an on-disk producer today.
//! This includes `tokenizer_adapter`, which the D0 design doc did not
//! resolve: a bundle's `tokenizer.bin` is a different, byte-oriented
//! format from the HF `tokenizer.json` a `TokenizerAdapter` is derived
//! from (`uor_r4_core::transformerless::hf_bpe`), and deriving one
//! requires the original Hugging Face source snapshot, which does not
//! live in a compiled bundle's `physical_root`.
//!
//! #655-D2 (`src/main.rs`'s `r4 package-release-bundle` command) is the
//! caller: it resolves a real `tokenizer_adapter` from an explicit
//! `--source` HF snapshot for `InstructionChat` bundles (using
//! `uor_r4_core::transformerless::hf_bpe::resolve_source_tokenizer`,
//! outside this module's own responsibility), builds [`PackageInputs`],
//! and writes this function's returned manifest to
//! `release_bundle_loader::RELEASE_BUNDLE_SIDECAR_FILE_NAME` next to
//! `physical_root`.

use std::path::{Path, PathBuf};

use uor_r4_api::{
    AbiVersion, BundleAbi, BundleCapability, BundleComponentDigests, ReleaseBundleManifest,
    TokenizerAdapter, UorMatmulProvenance, RELEASE_BUNDLE_MANIFEST_SCHEMA,
};

/// Standing #655 `uor-matmul` pin (`serving_655.md` project memory,
/// mirroring `docs/matrix_operation_census.md`). Bump only via the
/// project's κ/artifact-era re-pin process.
pub const UOR_MATMUL_REVISION: &str = "b13c98449948174f590e337c4dc25dfc394a07d0";

/// Relative paths within a resolved R4G1 bundle's `physical_root`, per
/// `docs/serving_release_packaging_655_d.md`'s field-to-file mapping.
const GRAPH_RELATIVE_PATH: &str = "graph/score.r4g1";
const SIGNATURE_ARTIFACT_RELATIVE_PATH: &str = "tless_artifacts.bin";
const TOKENIZER_RELATIVE_PATH: &str = "tokenizer.bin";
const SCORE_REPORT_RELATIVE_PATH: &str = "graph/score_report.json";
/// `components.compile_report` maps to the cover stage's report, not a
/// literal `compile_report.json` -- see the design doc's "one ambiguous
/// mapping" section for why those are two different files.
const COMPILE_REPORT_RELATIVE_PATH: &str = "graph-cover/cover_report.json";

/// Caller-supplied policy `package_release_bundle` does not derive from
/// `physical_root` -- see the module docs for why each field has no
/// on-disk producer today.
pub struct PackageInputs {
    pub model_id: String,
    pub capability: BundleCapability,
    pub uor_matmul: UorMatmulProvenance,
    pub tokenizer_adapter: TokenizerAdapter,
    pub provenance_note: Option<String>,
}

/// Why [`package_release_bundle`] could not build a manifest.
#[derive(Debug)]
pub enum PackageBundleError {
    /// A required component file could not be read (missing, permission
    /// denied, not a regular file, etc). The tokenizer is the one
    /// optional component (`BundleComponentDigests::tokenizer`) -- its
    /// absence is not this error, only `graph` / `signature_artifact` /
    /// `score_report` / `compile_report` are required.
    MissingRequiredFile {
        path: PathBuf,
        source: std::io::Error,
    },
    /// The manifest built from real digests failed its own structural
    /// `ReleaseBundleManifest::validate` (e.g. an empty caller-supplied
    /// `model_id`, or `InstructionChat` capability with an empty
    /// `tokenizer_adapter.family`).
    Invalid(String),
}

impl std::fmt::Display for PackageBundleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PackageBundleError::MissingRequiredFile { path, source } => write!(
                f,
                "required bundle component {} could not be read: {source}",
                path.display()
            ),
            PackageBundleError::Invalid(reason) => {
                write!(f, "packaged manifest failed validation: {reason}")
            }
        }
    }
}

impl std::error::Error for PackageBundleError {}

/// Build a [`ReleaseBundleManifest`] for the real R4G1 bundle at
/// `physical_root` by blake3-digesting its already-compiled component
/// files. Pure and read-only: does not write `release-bundle.json`
/// (#655-D2) and runs no compile/cover/score stage. The returned
/// manifest has already passed [`ReleaseBundleManifest::validate`] --
/// `Ok` never carries a structurally invalid manifest.
pub fn package_release_bundle(
    physical_root: &Path,
    inputs: PackageInputs,
) -> Result<ReleaseBundleManifest, PackageBundleError> {
    let graph = read_required(physical_root, GRAPH_RELATIVE_PATH)?;
    let signature_artifact = read_required(physical_root, SIGNATURE_ARTIFACT_RELATIVE_PATH)?;
    let score_report = read_required(physical_root, SCORE_REPORT_RELATIVE_PATH)?;
    let compile_report = read_required(physical_root, COMPILE_REPORT_RELATIVE_PATH)?;
    let tokenizer = read_optional(physical_root, TOKENIZER_RELATIVE_PATH)?;

    let manifest = ReleaseBundleManifest {
        schema: RELEASE_BUNDLE_MANIFEST_SCHEMA,
        model_id: inputs.model_id,
        capability: inputs.capability,
        abi: BundleAbi::from(AbiVersion::current()),
        uor_matmul: inputs.uor_matmul,
        components: BundleComponentDigests {
            graph: digest(&graph),
            signature_artifact: digest(&signature_artifact),
            tokenizer: tokenizer.as_deref().map(digest),
            score_report: digest(&score_report),
            compile_report: digest(&compile_report),
        },
        tokenizer_adapter: inputs.tokenizer_adapter,
        provenance_note: inputs.provenance_note,
    };

    match manifest.validate() {
        Some(reason) => Err(PackageBundleError::Invalid(reason)),
        None => Ok(manifest),
    }
}

fn read_required(physical_root: &Path, relative: &str) -> Result<Vec<u8>, PackageBundleError> {
    let path = physical_root.join(relative);
    std::fs::read(&path).map_err(|source| PackageBundleError::MissingRequiredFile { path, source })
}

fn read_optional(
    physical_root: &Path,
    relative: &str,
) -> Result<Option<Vec<u8>>, PackageBundleError> {
    let path = physical_root.join(relative);
    match std::fs::read(&path) {
        Ok(bytes) => Ok(Some(bytes)),
        Err(source) if source.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(source) => Err(PackageBundleError::MissingRequiredFile { path, source }),
    }
}

fn digest(bytes: &[u8]) -> String {
    format!("blake3:{}", blake3::hash(bytes).to_hex())
}

#[cfg(test)]
mod tests {
    use super::*;

    const VALID_REV: &str = UOR_MATMUL_REVISION;

    fn scratch_dir(label: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "uor-r4-release-bundle-packager-{label}-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("scratch directory");
        dir
    }

    fn write(dir: &Path, relative: &str, bytes: &[u8]) {
        let path = dir.join(relative);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).expect("create parent dir");
        }
        std::fs::write(&path, bytes).expect("write fixture file");
    }

    /// Writes every required (and the optional tokenizer) component into
    /// `dir`, mirroring the real CLI pipeline's layout
    /// (`docs/serving_release_packaging_655_d.md`).
    fn write_full_bundle(dir: &Path) {
        write(dir, GRAPH_RELATIVE_PATH, b"graph bytes");
        write(dir, SIGNATURE_ARTIFACT_RELATIVE_PATH, b"signature bytes");
        write(dir, TOKENIZER_RELATIVE_PATH, b"tokenizer bytes");
        write(dir, SCORE_REPORT_RELATIVE_PATH, b"{\"score\":true}");
        write(dir, COMPILE_REPORT_RELATIVE_PATH, b"{\"cover\":true}");
    }

    fn valid_inputs() -> PackageInputs {
        PackageInputs {
            model_id: "r4".to_string(),
            capability: BundleCapability::Continuation,
            uor_matmul: UorMatmulProvenance {
                rev: VALID_REV.to_string(),
                operation_profile: "exact-gemm-float".to_string(),
                license: "MIT".to_string(),
                source_digest: None,
            },
            tokenizer_adapter: TokenizerAdapter::default(),
            provenance_note: Some("packaged in a unit test".to_string()),
        }
    }

    #[test]
    fn builds_a_valid_manifest_from_real_files_on_disk() {
        let dir = scratch_dir("full-bundle");
        write_full_bundle(&dir);
        let manifest = package_release_bundle(&dir, valid_inputs()).expect("full bundle packages");
        assert_eq!(manifest.model_id, "r4");
        assert_eq!(manifest.components.graph, digest(b"graph bytes"));
        assert_eq!(
            manifest.components.signature_artifact,
            digest(b"signature bytes")
        );
        assert_eq!(
            manifest.components.tokenizer.as_deref(),
            Some(digest(b"tokenizer bytes").as_str())
        );
        assert_eq!(
            manifest.components.score_report,
            digest(b"{\"score\":true}")
        );
        assert_eq!(
            manifest.components.compile_report,
            digest(b"{\"cover\":true}")
        );
        assert_eq!(manifest.abi, BundleAbi::from(AbiVersion::current()));
        assert_eq!(manifest.validate(), None, "returned manifest is valid");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn absent_tokenizer_packages_with_no_tokenizer_digest() {
        let dir = scratch_dir("no-tokenizer");
        write(&dir, GRAPH_RELATIVE_PATH, b"graph bytes");
        write(&dir, SIGNATURE_ARTIFACT_RELATIVE_PATH, b"signature bytes");
        write(&dir, SCORE_REPORT_RELATIVE_PATH, b"{}");
        write(&dir, COMPILE_REPORT_RELATIVE_PATH, b"{}");
        let manifest = package_release_bundle(&dir, valid_inputs())
            .expect("bundle without a tokenizer still packages");
        assert_eq!(manifest.components.tokenizer, None);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn missing_graph_file_is_missing_required_file_error() {
        let dir = scratch_dir("missing-graph");
        write(&dir, SIGNATURE_ARTIFACT_RELATIVE_PATH, b"signature bytes");
        write(&dir, SCORE_REPORT_RELATIVE_PATH, b"{}");
        write(&dir, COMPILE_REPORT_RELATIVE_PATH, b"{}");
        let error = package_release_bundle(&dir, valid_inputs())
            .expect_err("missing graph file must error");
        assert!(
            matches!(error, PackageBundleError::MissingRequiredFile { .. }),
            "{error}"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn missing_signature_artifact_is_missing_required_file_error() {
        let dir = scratch_dir("missing-signature");
        write(&dir, GRAPH_RELATIVE_PATH, b"graph bytes");
        write(&dir, SCORE_REPORT_RELATIVE_PATH, b"{}");
        write(&dir, COMPILE_REPORT_RELATIVE_PATH, b"{}");
        let error = package_release_bundle(&dir, valid_inputs())
            .expect_err("missing signature artifact must error");
        assert!(
            matches!(error, PackageBundleError::MissingRequiredFile { .. }),
            "{error}"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn missing_score_report_is_missing_required_file_error() {
        let dir = scratch_dir("missing-score-report");
        write(&dir, GRAPH_RELATIVE_PATH, b"graph bytes");
        write(&dir, SIGNATURE_ARTIFACT_RELATIVE_PATH, b"signature bytes");
        write(&dir, COMPILE_REPORT_RELATIVE_PATH, b"{}");
        let error = package_release_bundle(&dir, valid_inputs())
            .expect_err("missing score report must error");
        assert!(
            matches!(error, PackageBundleError::MissingRequiredFile { .. }),
            "{error}"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn missing_compile_report_is_missing_required_file_error() {
        let dir = scratch_dir("missing-compile-report");
        write(&dir, GRAPH_RELATIVE_PATH, b"graph bytes");
        write(&dir, SIGNATURE_ARTIFACT_RELATIVE_PATH, b"signature bytes");
        write(&dir, SCORE_REPORT_RELATIVE_PATH, b"{}");
        let error = package_release_bundle(&dir, valid_inputs())
            .expect_err("missing compile report (graph-cover/cover_report.json) must error");
        assert!(
            matches!(error, PackageBundleError::MissingRequiredFile { .. }),
            "{error}"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn invalid_caller_input_surfaces_as_invalid_error() {
        let dir = scratch_dir("invalid-input");
        write_full_bundle(&dir);
        let mut inputs = valid_inputs();
        inputs.model_id = String::new();
        let error =
            package_release_bundle(&dir, inputs).expect_err("empty model_id must fail validate");
        match error {
            PackageBundleError::Invalid(reason) => {
                assert!(reason.contains("model_id"), "reason was: {reason}")
            }
            other => panic!("expected Invalid, got {other}"),
        }
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn instruction_chat_without_a_real_tokenizer_family_is_invalid() {
        // Documents the gap the module docs describe: this helper cannot
        // derive a real TokenizerAdapter from physical_root alone, so a
        // caller claiming InstructionChat with the default (empty-family)
        // adapter is correctly rejected by validate(), not silently
        // packaged as if it were a real instruction-tuned bundle.
        let dir = scratch_dir("instruction-chat-no-adapter");
        write_full_bundle(&dir);
        let mut inputs = valid_inputs();
        inputs.capability = BundleCapability::InstructionChat;
        let error = package_release_bundle(&dir, inputs)
            .expect_err("InstructionChat with an empty tokenizer_adapter.family must fail");
        match error {
            PackageBundleError::Invalid(reason) => {
                assert!(reason.contains("tokenizer_adapter"), "reason was: {reason}")
            }
            other => panic!("expected Invalid, got {other}"),
        }
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// #655-D1/D3 preview: confirms `package_release_bundle` works
    /// end-to-end against a real local compiled bundle directory, not
    /// just a synthetic fixture. `#[ignore]`d by default and
    /// environment-gated (mirrors `crates/uor-r4-api/tests/api.rs`'s own
    /// `UOR_R4_API_E2E_SOURCE` convention) so CI never depends on a
    /// multi-GB local `.uor-models/compiled/` store this repository does
    /// not check in (`.gitignore:29`), and so this test does not hardcode
    /// one developer's machine path. To run:
    ///
    /// ```sh
    /// UOR_R4_RELEASE_BUNDLE_PATH=.uor-models/compiled/smollm2-135m-instruct \
    ///   cargo test -p uor-r4-wasm-router --lib -- --ignored packages_the_real_local_bundle
    /// ```
    ///
    /// Capability is `Continuation`: see this test module's own
    /// `instruction_chat_without_a_real_tokenizer_family_is_invalid` for
    /// why `InstructionChat` would correctly fail here.
    #[test]
    #[ignore = "requires UOR_R4_RELEASE_BUNDLE_PATH pointed at a real local compiled bundle"]
    fn packages_the_real_local_bundle() {
        let Ok(path) = std::env::var("UOR_R4_RELEASE_BUNDLE_PATH") else {
            eprintln!("UOR_R4_RELEASE_BUNDLE_PATH not set; skipping");
            return;
        };
        let manifest = package_release_bundle(Path::new(&path), valid_inputs())
            .expect("real local bundle packages successfully");
        assert_eq!(manifest.model_id, "r4");
        assert!(
            manifest.components.tokenizer.is_some(),
            "the real local bundle has a tokenizer.bin"
        );
        assert_eq!(manifest.validate(), None);
    }
}
