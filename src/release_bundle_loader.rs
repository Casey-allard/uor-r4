//! #655-C1c: an additive, best-effort verification layer for a resolved
//! R4G1 bundle's optional `release-bundle.json` sidecar
//! (`docs/serving_shared_loader_655_c1.md`, #655-C0/C1a/C1b).
//!
//! This module does the filesystem I/O and digesting that
//! `uor_r4_api::release_bundle` deliberately does not (that crate is
//! schema/validation only, no I/O). [`verify_release_bundle_sidecar`]
//! never changes which bundle loads: a missing sidecar, a parse or
//! structural-validation failure, or a digest mismatch against the
//! bundle's actual files on disk all resolve to `None`, exactly as if no
//! sidecar existed. Per the design doc's Q3, `None` is the common case
//! today -- no shipped bundle produces a sidecar yet (#655-D is still
//! open).
//!
//! Scope: this slice verifies exactly the two components the caller
//! already has resolved file paths for -- `components.graph` against the
//! bundle's graph file, and `components.signature_artifact` against its
//! teacher-companion file. `score_report`, `compile_report`, and
//! `tokenizer` are checked for structural well-formedness (digest shape)
//! by [`ReleaseBundleManifest::validate`] but not re-hashed against bytes
//! on disk here: this resolution has no established path convention for
//! locating those files independently of a packaging step that doesn't
//! exist yet. Closing that gap is future work (#655-C1d) once #655-D
//! settles a real bundle directory layout.

use std::path::Path;

use uor_r4_api::ReleaseBundleManifest;

/// Sidecar file name a packaged bundle directory may place next to its
/// R4G1 artifacts. Nothing writes this file yet (#655-D is still open);
/// reading it here is purely additive.
pub(crate) const RELEASE_BUNDLE_SIDECAR_FILE_NAME: &str = "release-bundle.json";

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
    if !file_matches_digest(teacher_path, &manifest.components.signature_artifact) {
        return None;
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
        BundleAbi, BundleCapability, BundleComponentDigests, UorMatmulProvenance,
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
                signature_artifact: teacher_digest,
                tokenizer: None,
                score_report: PLACEHOLDER_DIGEST.to_string(),
                compile_report: PLACEHOLDER_DIGEST.to_string(),
            },
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
        dir
    }

    #[test]
    fn absent_sidecar_is_none() {
        let dir = scratch_dir("absent");
        let graph = write(&dir, "graph", b"graph bytes");
        let teacher = write(&dir, "teacher", b"teacher bytes");
        assert!(verify_release_bundle_sidecar(&dir, &graph, &teacher).is_none());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn matching_digests_verify() {
        let dir = scratch_dir("match");
        let graph = write(&dir, "graph", b"graph bytes");
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
        let graph = write(&dir, "graph", b"graph bytes");
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
        let graph = write(&dir, "graph", b"graph bytes");
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
        let graph = write(&dir, "graph", b"graph bytes");
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
        let graph = write(&dir, "graph", b"graph bytes");
        let teacher = write(&dir, "teacher", b"teacher bytes");
        std::fs::write(dir.join(RELEASE_BUNDLE_SIDECAR_FILE_NAME), b"not json")
            .expect("write sidecar");
        assert!(verify_release_bundle_sidecar(&dir, &graph, &teacher).is_none());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn missing_referenced_file_is_none() {
        let dir = scratch_dir("missing-file");
        let graph = write(&dir, "graph", b"graph bytes");
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
