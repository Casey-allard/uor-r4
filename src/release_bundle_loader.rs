//! Release-bundle verification for resolved R4G1 generations.
//!
//! [`verify_release_bundle_sidecar`] retains #655-C1c's advisory status/UI
//! probe. Production serving uses [`capture_production_admission`] plus
//! [`verify_production_admission`] instead: schema 2, every required byte,
//! and independently derived deployed-quality identities are mandatory.
//!
//! The historical probe is an additive, best-effort verification layer for a
//! resolved R4G1 bundle's optional `release-bundle.json` sidecar
//! (`docs/serving_shared_loader_655_c1.md`, #655-C0/C1a/C1b).
//!
//! This module does the filesystem I/O and digesting that
//! `uor_r4_api::release_bundle` deliberately does not (that crate is
//! schema/validation only, no I/O). [`verify_release_bundle_sidecar`]
//! never changes which bundle loads: a missing sidecar, a parse or
//! structural-validation failure, or a digest mismatch against the
//! bundle's actual files on disk all resolve to `None`, exactly as if no
//! sidecar existed. Sidecars now exist in the wild: `r4
//! package-release-bundle` (#655-D2) writes one, and `r4 install-release`
//! (#741) installs a hard-verified copy beside every released bundle --
//! locally compiled bundles that never ran packaging still have none,
//! and remain the `None` case.
//!
//! Schema 2 verifies every runtime/admission component against the settled
//! bundle layout, including the deployed-quality report. Missing or
//! mismatched evidence is never represented as a verified release bundle.

use std::path::Path;

use uor_r4_api::{
    verify_production_envelope, ProductionEnvelopeParts, ReleaseBundleManifest,
    VerifiedProductionEnvelope,
};

/// Sidecar file name a packaged bundle directory may place next to its
/// R4G1 artifacts. `pub` (not `pub(crate)`) since #655-D2's
/// `r4 package-release-bundle` CLI command (`src/main.rs`) writes to
/// this exact path.
pub const RELEASE_BUNDLE_SIDECAR_FILE_NAME: &str = "release-bundle.json";

/// Exact non-runtime bytes captured from the same immutable bundle
/// generation as the graph, teacher artifact, and tokenizer. Production
/// admission never reopens these paths after startup capture.
#[derive(Clone)]
pub(crate) struct CapturedProductionAdmission {
    pub(crate) release_manifest: Vec<u8>,
    pub(crate) deployed_quality_report: Vec<u8>,
    pub(crate) sections_absent_graph: Vec<u8>,
    pub(crate) label_shuffled_graph: Vec<u8>,
    pub(crate) tla_comparator_store: Vec<u8>,
    pub(crate) cross_surface_parity: Vec<u8>,
    pub(crate) witness_replay: Vec<u8>,
    pub(crate) score_report: Vec<u8>,
    pub(crate) compile_report: Vec<u8>,
    pub(crate) corpus_meta: Vec<u8>,
    pub(crate) corpus_records: Vec<u8>,
    pub(crate) tokenizer_adapter: Vec<u8>,
}

/// A schema-2 release envelope whose component bytes and independently
/// derived serving identities match the deployed-quality report.
#[derive(Debug)]
pub(crate) struct VerifiedProductionAdmission {
    pub(crate) deployed_quality_report: Vec<u8>,
    pub(crate) envelope: VerifiedProductionEnvelope,
}

impl VerifiedProductionAdmission {
    pub(crate) fn manifest(&self) -> &ReleaseBundleManifest {
        self.envelope.manifest()
    }
}

/// Resolve the settled production layout shared by packaging and serving.
/// A graph outside `<root>/graph/score.r4g1`, or a signature artifact outside
/// the same root, is research input and cannot acquire production status.
pub(crate) fn production_bundle_root<'a>(
    graph_path: &Path,
    teacher_path: &'a Path,
) -> Result<&'a Path, String> {
    let root = teacher_path.parent().ok_or_else(|| {
        format!(
            "production signature artifact {} has no bundle root",
            teacher_path.display()
        )
    })?;
    if graph_path != root.join("graph/score.r4g1")
        || teacher_path != root.join("tless_artifacts.bin")
    {
        return Err(format!(
            "production R4G1 admission requires graph/score.r4g1 and tless_artifacts.bin under one settled bundle root; got {} and {}",
            graph_path.display(),
            teacher_path.display()
        ));
    }
    Ok(root)
}

/// Capture every byte needed to derive production identities. This is the
/// path-based convenience seam; startup's guarded explicit-artifact lane may
/// construct the same value from already-open regular file handles.
pub(crate) fn capture_production_admission(
    root: &Path,
) -> Result<CapturedProductionAdmission, String> {
    Ok(CapturedProductionAdmission {
        release_manifest: read_required_regular(&root.join(RELEASE_BUNDLE_SIDECAR_FILE_NAME))?,
        deployed_quality_report: read_required_regular(
            &root.join("graph/deployed_quality_report.json"),
        )?,
        sections_absent_graph: read_required_regular(
            &root.join("graph/score_sections_absent.r4g1"),
        )?,
        label_shuffled_graph: read_required_regular(&root.join("graph/score_label_shuffled.r4g1"))?,
        tla_comparator_store: read_required_regular(&root.join("tless_store.bin"))?,
        cross_surface_parity: read_required_regular(&root.join("graph/cross_surface_parity.json"))?,
        witness_replay: read_required_regular(&root.join("graph/witness_replay.json"))?,
        score_report: read_required_regular(&root.join("graph/score_report.json"))?,
        compile_report: read_required_regular(&root.join("graph-cover/cover_report.json"))?,
        corpus_meta: read_required_regular(&root.join("corpus.meta"))?,
        corpus_records: read_required_regular(&root.join("corpus.records"))?,
        tokenizer_adapter: read_required_regular(&root.join("tokenizer_adapter.json"))?,
    })
}

/// Verify one already-captured production generation. No pathname is opened
/// here: graph, teacher, tokenizer, corpus, reports, and manifest are the
/// exact byte slices selected by the caller's startup authority.
pub(crate) fn verify_production_admission(
    graph: &[u8],
    teacher: &[u8],
    tokenizer: Option<&[u8]>,
    captured: &CapturedProductionAdmission,
) -> Result<VerifiedProductionAdmission, String> {
    let tokenizer = tokenizer.unwrap_or(&[]);
    let verified = verify_production_envelope(ProductionEnvelopeParts {
        graph,
        sections_absent_graph: &captured.sections_absent_graph,
        label_shuffled_graph: &captured.label_shuffled_graph,
        signature_artifact: teacher,
        tla_comparator_store: &captured.tla_comparator_store,
        tokenizer,
        score_report: &captured.score_report,
        compile_report: &captured.compile_report,
        deployed_quality_report: &captured.deployed_quality_report,
        cross_surface_parity: &captured.cross_surface_parity,
        witness_replay: &captured.witness_replay,
        corpus_meta: &captured.corpus_meta,
        corpus_records: &captured.corpus_records,
        tokenizer_adapter: &captured.tokenizer_adapter,
        release_manifest: &captured.release_manifest,
    })
    .map_err(|error| error.to_string())?;
    Ok(VerifiedProductionAdmission {
        deployed_quality_report: captured.deployed_quality_report.clone(),
        envelope: verified,
    })
}

fn read_required_regular(path: &Path) -> Result<Vec<u8>, String> {
    let metadata = std::fs::symlink_metadata(path)
        .map_err(|error| format!("required production component {}: {error}", path.display()))?;
    if !metadata.file_type().is_file() {
        return Err(format!(
            "required production component {} is not a regular non-symlink file",
            path.display()
        ));
    }
    std::fs::read(path)
        .map_err(|error| format!("required production component {}: {error}", path.display()))
}

/// Look for, parse, and verify a `release-bundle.json` sidecar next to a
/// resolved bundle's `physical_root`. Returns `Some` only when the
/// sidecar exists, parses, passes [`ReleaseBundleManifest::validate`],
/// and its declared `components.graph` / `components.signature_artifact`
/// digests match the actual bytes at `graph_path` / `teacher_path`.
/// Returns `None` for every other case, including an I/O error reading
/// either file -- this check is advisory and must never turn into a load
/// failure for a bundle that would otherwise serve today.
pub(crate) fn verify_release_bundle_sidecar(
    physical_root: &Path,
    graph_path: &Path,
    teacher_path: &Path,
) -> Option<ReleaseBundleManifest> {
    let sidecar_path = physical_root.join(RELEASE_BUNDLE_SIDECAR_FILE_NAME);
    let bytes = std::fs::read(sidecar_path).ok()?;
    let manifest: ReleaseBundleManifest = serde_json::from_slice(&bytes).ok()?;
    if manifest.validate().is_some() {
        return None;
    }
    if !file_matches_digest(graph_path, &manifest.components.graph) {
        return None;
    }
    let sections_absent = manifest.components.sections_absent_graph.as_deref()?;
    if !file_matches_digest(
        &physical_root.join("graph/score_sections_absent.r4g1"),
        sections_absent,
    ) {
        return None;
    }
    let label_shuffled = manifest.components.label_shuffled_graph.as_deref()?;
    if !file_matches_digest(
        &physical_root.join("graph/score_label_shuffled.r4g1"),
        label_shuffled,
    ) {
        return None;
    }
    if !file_matches_digest(teacher_path, &manifest.components.signature_artifact) {
        return None;
    }
    let tla_store = manifest.components.tla_comparator_store.as_deref()?;
    if !file_matches_digest(&physical_root.join("tless_store.bin"), tla_store) {
        return None;
    }
    if !file_matches_digest(
        &physical_root.join("graph/score_report.json"),
        &manifest.components.score_report,
    ) {
        return None;
    }
    if !file_matches_digest(
        &physical_root.join("graph-cover/cover_report.json"),
        &manifest.components.compile_report,
    ) {
        return None;
    }
    let deployed_quality = manifest.components.deployed_quality_report.as_deref()?;
    if !file_matches_digest(
        &physical_root.join("graph/deployed_quality_report.json"),
        deployed_quality,
    ) {
        return None;
    }
    let cross_surface = manifest.components.cross_surface_parity.as_deref()?;
    if !file_matches_digest(
        &physical_root.join("graph/cross_surface_parity.json"),
        cross_surface,
    ) {
        return None;
    }
    let witness = manifest.components.witness_replay.as_deref()?;
    if !file_matches_digest(&physical_root.join("graph/witness_replay.json"), witness) {
        return None;
    }
    match manifest.components.tokenizer.as_deref() {
        Some(digest) if !file_matches_digest(&physical_root.join("tokenizer.bin"), digest) => {
            return None;
        }
        None if physical_root.join("tokenizer.bin").is_file() => return None,
        _ => {}
    }
    Some(manifest)
}

fn file_matches_digest(path: &Path, declared: &str) -> bool {
    let Ok(bytes) = std::fs::read(path) else {
        return false;
    };
    format!("blake3:{}", blake3::hash(&bytes).to_hex()) == declared
}

#[cfg(test)]
mod tests {
    use super::*;
    use uor_r4_api::{
        BundleAbi, BundleCapability, BundleComponentDigests, SelectorIdentity, UorMatmulProvenance,
        RELEASE_BUNDLE_MANIFEST_SCHEMA,
    };
    use uor_r4_core::transformerless::hf_bpe::TokenizerAdapter;

    const VALID_REV: &str = "b13c98449948174f590e337c4dc25dfc394a07d0";
    const PLACEHOLDER_DIGEST: &str =
        "blake3:0000000000000000000000000000000000000000000000000000000000000001";

    fn digest_of(bytes: &[u8]) -> String {
        format!("blake3:{}", blake3::hash(bytes).to_hex())
    }

    fn write(dir: &Path, name: &str, bytes: &[u8]) -> std::path::PathBuf {
        let path = dir.join(name);
        std::fs::write(&path, bytes).expect("write fixture file");
        path
    }

    fn manifest_for(graph_digest: String, teacher_digest: String) -> ReleaseBundleManifest {
        ReleaseBundleManifest {
            schema: RELEASE_BUNDLE_MANIFEST_SCHEMA,
            model_id: "r4".to_string(),
            capability: BundleCapability::InstructionChat,
            abi: BundleAbi {
                format_major: 1,
                format_minor: 0,
                contract_major: 1,
                contract_minor: 0,
                contract_patch: 0,
                api_crate_version: "0.1.0".to_string(),
            },
            uor_matmul: UorMatmulProvenance {
                rev: VALID_REV.to_string(),
                operation_profile: "exact-gemm-float".to_string(),
                license: "MIT".to_string(),
                source_digest: None,
            },
            components: BundleComponentDigests {
                graph: graph_digest,
                sections_absent_graph: Some(digest_of(b"absent")),
                label_shuffled_graph: Some(digest_of(b"shuffled")),
                signature_artifact: teacher_digest,
                tla_comparator_store: Some(digest_of(b"tla-store")),
                tokenizer: None,
                score_report: digest_of(b"score"),
                compile_report: digest_of(b"cover"),
                deployed_quality_report: Some(digest_of(b"quality")),
                cross_surface_parity: Some(digest_of(b"cross")),
                witness_replay: Some(digest_of(b"witness")),
            },
            selector: Some(SelectorIdentity {
                id: uor_r4_api::NORMATIVE_SELECTOR_ID.to_string(),
                semantics_version: "1".to_string(),
                semantics_cid: PLACEHOLDER_DIGEST.to_string(),
            }),
            compiler: Some(uor_r4_api::CompilerIdentity {
                revision: VALID_REV.to_string(),
                configuration_cid: PLACEHOLDER_DIGEST.to_string(),
            }),
            tokenizer_adapter: TokenizerAdapter {
                family: "hf-byte-bpe".to_string(),
                ..Default::default()
            },
            provenance_note: None,
        }
    }

    fn scratch_dir(label: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "uor-r4-release-bundle-loader-{label}-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("scratch directory");
        write_bound_components(&dir);
        dir
    }

    fn write_bound_components(dir: &Path) {
        for (relative, bytes) in [
            ("graph/score_sections_absent.r4g1", b"absent".as_slice()),
            ("graph/score_label_shuffled.r4g1", b"shuffled".as_slice()),
            ("tless_store.bin", b"tla-store".as_slice()),
            ("graph/score_report.json", b"score".as_slice()),
            ("graph-cover/cover_report.json", b"cover".as_slice()),
            ("graph/deployed_quality_report.json", b"quality".as_slice()),
            ("graph/cross_surface_parity.json", b"cross".as_slice()),
            ("graph/witness_replay.json", b"witness".as_slice()),
        ] {
            let path = dir.join(relative);
            std::fs::create_dir_all(path.parent().expect("component parent"))
                .expect("create component parent");
            std::fs::write(path, bytes).expect("write bound component");
        }
    }

    #[test]
    fn absent_sidecar_is_none() {
        let dir = scratch_dir("absent");
        let graph = write(&dir, "graph/score.r4g1", b"graph bytes");
        let teacher = write(&dir, "teacher", b"teacher bytes");
        assert!(verify_release_bundle_sidecar(&dir, &graph, &teacher).is_none());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn production_layout_rejects_standalone_paths() {
        let dir = scratch_dir("production-layout");
        let graph = dir.join("custom.r4g1");
        let teacher = dir.join("teacher.bin");
        let error = production_bundle_root(&graph, &teacher)
            .expect_err("standalone paths are research-only");
        assert!(error.contains("settled bundle root"), "{error}");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn production_verifier_rejects_legacy_schema_before_component_use() {
        let mut manifest = manifest_for(digest_of(b"graph"), digest_of(b"teacher"));
        manifest.schema = uor_r4_api::LEGACY_RELEASE_BUNDLE_MANIFEST_SCHEMA;
        manifest.components.deployed_quality_report = None;
        manifest.selector = None;
        manifest.compiler = None;
        let captured = CapturedProductionAdmission {
            release_manifest: serde_json::to_vec(&manifest).expect("serialize legacy manifest"),
            deployed_quality_report: Vec::new(),
            sections_absent_graph: Vec::new(),
            label_shuffled_graph: Vec::new(),
            tla_comparator_store: Vec::new(),
            cross_surface_parity: Vec::new(),
            witness_replay: Vec::new(),
            score_report: Vec::new(),
            compile_report: Vec::new(),
            corpus_meta: Vec::new(),
            corpus_records: Vec::new(),
            tokenizer_adapter: Vec::new(),
        };
        let error = verify_production_admission(b"graph", b"teacher", None, &captured)
            .expect_err("legacy schema cannot authorize production");
        assert!(error.contains("legacy research evidence"), "{error}");
    }

    #[test]
    fn matching_digests_verify() {
        let dir = scratch_dir("match");
        let graph = write(&dir, "graph/score.r4g1", b"graph bytes");
        let teacher = write(&dir, "teacher", b"teacher bytes");
        let manifest = manifest_for(digest_of(b"graph bytes"), digest_of(b"teacher bytes"));
        std::fs::write(
            dir.join(RELEASE_BUNDLE_SIDECAR_FILE_NAME),
            serde_json::to_vec(&manifest).expect("serialize manifest"),
        )
        .expect("write sidecar");
        let verified = verify_release_bundle_sidecar(&dir, &graph, &teacher);
        assert_eq!(verified, Some(manifest));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn mismatched_graph_digest_is_none() {
        let dir = scratch_dir("mismatch-graph");
        let graph = write(&dir, "graph/score.r4g1", b"graph bytes");
        let teacher = write(&dir, "teacher", b"teacher bytes");
        let manifest = manifest_for(digest_of(b"WRONG BYTES"), digest_of(b"teacher bytes"));
        std::fs::write(
            dir.join(RELEASE_BUNDLE_SIDECAR_FILE_NAME),
            serde_json::to_vec(&manifest).expect("serialize manifest"),
        )
        .expect("write sidecar");
        assert!(verify_release_bundle_sidecar(&dir, &graph, &teacher).is_none());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn mismatched_teacher_digest_is_none() {
        let dir = scratch_dir("mismatch-teacher");
        let graph = write(&dir, "graph/score.r4g1", b"graph bytes");
        let teacher = write(&dir, "teacher", b"teacher bytes");
        let manifest = manifest_for(digest_of(b"graph bytes"), digest_of(b"WRONG BYTES"));
        std::fs::write(
            dir.join(RELEASE_BUNDLE_SIDECAR_FILE_NAME),
            serde_json::to_vec(&manifest).expect("serialize manifest"),
        )
        .expect("write sidecar");
        assert!(verify_release_bundle_sidecar(&dir, &graph, &teacher).is_none());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn structurally_invalid_manifest_is_none() {
        let dir = scratch_dir("invalid");
        let graph = write(&dir, "graph/score.r4g1", b"graph bytes");
        let teacher = write(&dir, "teacher", b"teacher bytes");
        let mut manifest = manifest_for(digest_of(b"graph bytes"), digest_of(b"teacher bytes"));
        manifest.model_id = String::new();
        std::fs::write(
            dir.join(RELEASE_BUNDLE_SIDECAR_FILE_NAME),
            serde_json::to_vec(&manifest).expect("serialize manifest"),
        )
        .expect("write sidecar");
        assert!(verify_release_bundle_sidecar(&dir, &graph, &teacher).is_none());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn malformed_json_is_none() {
        let dir = scratch_dir("malformed");
        let graph = write(&dir, "graph/score.r4g1", b"graph bytes");
        let teacher = write(&dir, "teacher", b"teacher bytes");
        std::fs::write(dir.join(RELEASE_BUNDLE_SIDECAR_FILE_NAME), b"not json")
            .expect("write sidecar");
        assert!(verify_release_bundle_sidecar(&dir, &graph, &teacher).is_none());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn missing_referenced_file_is_none() {
        let dir = scratch_dir("missing-file");
        let graph = write(&dir, "graph/score.r4g1", b"graph bytes");
        let teacher = dir.join("teacher-does-not-exist");
        let manifest = manifest_for(digest_of(b"graph bytes"), digest_of(b"teacher bytes"));
        std::fs::write(
            dir.join(RELEASE_BUNDLE_SIDECAR_FILE_NAME),
            serde_json::to_vec(&manifest).expect("serialize manifest"),
        )
        .expect("write sidecar");
        assert!(verify_release_bundle_sidecar(&dir, &graph, &teacher).is_none());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
