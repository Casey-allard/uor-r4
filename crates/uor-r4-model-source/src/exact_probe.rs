//! Durable admission evidence for the exact multicore teacher probe.

use crate::{
    exact_backend_report, ExactBackendReport, ExactForwardPlan, TeacherExecutionSnapshot,
    UOR_MATMUL_REVISION,
};
use serde::{Deserialize, Serialize};

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn normalized_report_parent(path: &std::path::Path) -> &std::path::Path {
    path.parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| std::path::Path::new("."))
}

/// Resolve one direct-tuner path exactly as a repository-root invocation.
///
/// Cargo executes a package unit test with that package as its working
/// directory, even when the user invoked Cargo from the workspace root. The
/// BDD owner and documented commands instead interpret relative fixture/report
/// paths at the repository root. Deriving that root from this crate's manifest
/// keeps both entry points on the same files. Empty paths remain empty so their
/// caller can publish the intended typed refusal.
#[cfg(all(test, not(target_arch = "wasm32")))]
pub(crate) fn resolve_direct_probe_path(path: std::path::PathBuf) -> std::path::PathBuf {
    if path.as_os_str().is_empty() || path.is_absolute() {
        return path;
    }
    let crate_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = crate_dir
        .parent()
        .and_then(std::path::Path::parent)
        .expect("model-source crate belongs to the workspace crates directory");
    workspace_root.join(path)
}

/// Versioned JSON schema written by the exact multicore admission probe.
pub const EXACT_MULTICORE_PROBE_SCHEMA: &str = "uor-r4.exact-multicore-probe/2";
/// Admission deadline for the cheap probe, including fixture load.
///
/// Work that is already inside a non-cancellable fixture load or exact forward
/// is allowed to finish, but no later forward is admitted and a run at or past
/// the deadline cannot qualify.
pub const EXACT_MULTICORE_PROBE_WALL_CEILING_SECONDS: u64 = 3_600;
/// Machine-readable deadline semantics bound into every qualified report.
pub const EXACT_MULTICORE_PROBE_DEADLINE_POLICY: &str = "NO_NEW_EXACT_FORWARD_AFTER_DEADLINE; ACTIVE_FIXTURE_LOAD_OR_EXACT_FORWARD_FINISHES_THEN_ABORTS; NEVER_QUALIFY_AT_OR_AFTER_DEADLINE";
/// Deterministic adaptive selection rule bound into every report.
pub const EXACT_MULTICORE_PROBE_SELECTION_POLICY: &str =
    "MIN_SAFETY_ADJUSTED_PROJECTED_SECONDS; TIE_LOWER_WORKER_COUNT";
/// Registered pinned transcript work exposed by the eight prompt fixtures.
pub const EXACT_MULTICORE_PROBE_REGISTERED_TRANSCRIPT_FORWARDS: usize = 36;
/// Registered S8 transcript batch widths in canonical scheduler order.
pub const EXACT_MULTICORE_PROBE_REGISTERED_TRANSCRIPT_BATCH_WIDTHS: [usize; 6] = [8, 8, 8, 7, 4, 1];
/// Registered maximum optimized continuation per cloned lane.
pub const EXACT_MULTICORE_PROBE_REGISTERED_GENERATION_TOKENS: usize = 8;
/// Registered independent cloned continuation lanes.
pub const EXACT_MULTICORE_PROBE_REGISTERED_GENERATION_LANES: usize = 8;
/// Registered maximum zero-based private-state position. Canonical prompt
/// lengths are `[6, 7, 6, 5, 4, 5, 6, 5]`; retaining each distinct final
/// teacher-forced prefix makes the longest prefix occupy positions `0..=5`,
/// followed by eight continuation positions `6..=13`.
pub const EXACT_MULTICORE_PROBE_REGISTERED_MAX_SEQUENCE_POSITION: usize = 13;
/// Registered private-state capacity covering positions `0..=13`.
pub const EXACT_MULTICORE_PROBE_REGISTERED_STATE_SEQUENCE_CAPACITY: usize = 14;
/// Registered empirical context-length boundary for projection admission.
pub const EXACT_MULTICORE_PROBE_CONTEXT_ASSUMPTION: &str = "Empirical Criterion (not Guarantee): every timed probe repetition uses the declared bounded per-state horizon covering the longest registered retained prefix plus the maximum continuation; aggregate transcript work is scheduled separately in registered S8 batches; the optimized suite reuses cloned transcript states and charges no duplicate prefill or warm-up forwards; the registered 1.25 factor is an additional projection margin";

/// Complete settled generation captured by the schema-2 production loader.
///
/// The logical names are stable report keys and the relative paths are the
/// exact layout consumed by `src/release_bundle_loader.rs`. A teacher-free
/// AVAILABLE token binds every one of these bytes; the direct tuner rehashes
/// the same set immediately before teacher access.
pub const PRODUCTION_ADMISSION_COMPONENTS: [(&str, &str); 15] = [
    ("release_manifest", "release-bundle.json"),
    ("graph", "graph/score.r4g1"),
    ("sections_absent_graph", "graph/score_sections_absent.r4g1"),
    ("label_shuffled_graph", "graph/score_label_shuffled.r4g1"),
    ("signature_artifact", "tless_artifacts.bin"),
    ("tla_comparator_store", "tless_store.bin"),
    ("tokenizer", "tokenizer.bin"),
    ("score_report", "graph/score_report.json"),
    ("compile_report", "graph-cover/cover_report.json"),
    (
        "deployed_quality_report",
        "graph/deployed_quality_report.json",
    ),
    ("cross_surface_parity", "graph/cross_surface_parity.json"),
    ("witness_replay", "graph/witness_replay.json"),
    ("corpus_meta", "corpus.meta"),
    ("corpus_records", "corpus.records"),
    ("tokenizer_adapter", "tokenizer_adapter.json"),
];

/// Recompute the complete schema-2 production-generation identity map.
///
/// Missing components are `UNAVAILABLE`; unreadable, non-regular, or symlinked
/// components are `FAILED`. The production loader has the same regular-file
/// boundary, so this helper cannot bless bytes that loader would not capture.
#[cfg(not(target_arch = "wasm32"))]
pub fn production_admission_component_cids(
    bundle_dir: impl AsRef<std::path::Path>,
) -> Result<std::collections::BTreeMap<String, String>, crate::SourceUnavailable> {
    use std::io::Read;

    let bundle_dir = bundle_dir.as_ref();
    let mut cids = std::collections::BTreeMap::new();
    for (name, relative) in PRODUCTION_ADMISSION_COMPONENTS {
        let path = bundle_dir.join(relative);
        let metadata = std::fs::symlink_metadata(&path).map_err(|error| {
            if error.kind() == std::io::ErrorKind::NotFound {
                crate::SourceUnavailable::new(format!(
                    "UNAVAILABLE: required production component {} is absent: {error}",
                    path.display()
                ))
            } else {
                crate::SourceUnavailable::new(format!(
                    "FAILED: inspect required production component {}: {error}",
                    path.display()
                ))
            }
        })?;
        if !metadata.file_type().is_file() {
            return Err(crate::SourceUnavailable::new(format!(
                "FAILED: required production component {} is not a regular non-symlink file",
                path.display()
            )));
        }
        let mut file = std::fs::File::open(&path).map_err(|error| {
            crate::SourceUnavailable::new(format!(
                "FAILED: open required production component {}: {error}",
                path.display()
            ))
        })?;
        let mut hasher = blake3::Hasher::new();
        let mut buffer = [0u8; 64 * 1024];
        loop {
            let read = file.read(&mut buffer).map_err(|error| {
                crate::SourceUnavailable::new(format!(
                    "FAILED: read required production component {}: {error}",
                    path.display()
                ))
            })?;
            if read == 0 {
                break;
            }
            hasher.update(&buffer[..read]);
        }
        cids.insert(
            name.to_owned(),
            format!("blake3:{}", hasher.finalize().to_hex()),
        );
    }
    Ok(cids)
}

#[cfg(all(test, not(target_arch = "wasm32")))]
pub(crate) const TEACHER_FREE_PREFLIGHT_SCHEMA: &str = "uor-r4.teacher-parity-preflight/1";

/// Current teacher-free prerequisite evidence consumed by the direct ignored
/// tuner before it is allowed to open teacher weights.
#[cfg(all(test, not(target_arch = "wasm32")))]
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct TeacherFreePreflightAdmission {
    pub report_cid: String,
    pub selected_source_dir: String,
    pub selected_bundle_dir: String,
    pub compiled_input_cids: std::collections::BTreeMap<String, String>,
    pub production_admission_cids: std::collections::BTreeMap<String, String>,
}

/// Typed reason that a direct live tuner cannot consume teacher-free evidence.
#[cfg(all(test, not(target_arch = "wasm32")))]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum TeacherFreePreflightAdmissionError {
    Unavailable(String),
    Refused(String),
    Failed(String),
}

#[cfg(all(test, not(target_arch = "wasm32")))]
impl std::fmt::Display for TeacherFreePreflightAdmissionError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let reason = match self {
            Self::Unavailable(reason) | Self::Refused(reason) | Self::Failed(reason) => reason,
        };
        formatter.write_str(reason)
    }
}

/// Capture and semantically verify the exact schema-2 generation that the
/// direct tuner is about to use as its teacher-free admission authority.
///
/// Digest agreement alone is insufficient: mutually inconsistent bytes can
/// all match a forged AVAILABLE report. The production API owns the portable
/// envelope semantics, so the tuner invokes that same verifier over one
/// captured byte generation and returns the identities of those exact bytes.
#[cfg(all(test, not(target_arch = "wasm32")))]
fn verify_production_envelope_semantics(
    bundle_dir: &std::path::Path,
) -> Result<std::collections::BTreeMap<String, String>, TeacherFreePreflightAdmissionError> {
    use TeacherFreePreflightAdmissionError as Error;

    let mut components = std::collections::BTreeMap::<String, Vec<u8>>::new();
    for (name, relative) in PRODUCTION_ADMISSION_COMPONENTS {
        let path = bundle_dir.join(relative);
        let metadata = std::fs::symlink_metadata(&path).map_err(|error| {
            if error.kind() == std::io::ErrorKind::NotFound {
                Error::Unavailable(format!(
                    "UNAVAILABLE: required production component {} is absent: {error}",
                    path.display()
                ))
            } else {
                Error::Failed(format!(
                    "FAILED: inspect required production component {}: {error}",
                    path.display()
                ))
            }
        })?;
        if !metadata.file_type().is_file() {
            return Err(Error::Failed(format!(
                "FAILED: required production component {} is not a regular non-symlink file",
                path.display()
            )));
        }
        let bytes = std::fs::read(&path).map_err(|error| {
            Error::Failed(format!(
                "FAILED: capture required production component {}: {error}",
                path.display()
            ))
        })?;
        components.insert(name.to_owned(), bytes);
    }

    let component = |name: &str| {
        components
            .get(name)
            .map(Vec::as_slice)
            .ok_or_else(|| Error::Failed(format!("FAILED: internal component map omitted {name}")))
    };
    uor_r4_api::verify_production_envelope(uor_r4_api::ProductionEnvelopeParts {
        graph: component("graph")?,
        sections_absent_graph: component("sections_absent_graph")?,
        label_shuffled_graph: component("label_shuffled_graph")?,
        signature_artifact: component("signature_artifact")?,
        tla_comparator_store: component("tla_comparator_store")?,
        tokenizer: component("tokenizer")?,
        score_report: component("score_report")?,
        compile_report: component("compile_report")?,
        deployed_quality_report: component("deployed_quality_report")?,
        cross_surface_parity: component("cross_surface_parity")?,
        witness_replay: component("witness_replay")?,
        corpus_meta: component("corpus_meta")?,
        corpus_records: component("corpus_records")?,
        tokenizer_adapter: component("tokenizer_adapter")?,
        release_manifest: component("release_manifest")?,
    })
    .map_err(|error| {
        Error::Failed(format!(
            "FAILED: schema-2 production envelope semantic verification: {error}"
        ))
    })?;

    Ok(components
        .into_iter()
        .map(|(name, bytes)| (name, format!("blake3:{}", blake3::hash(&bytes).to_hex())))
        .collect())
}

/// Validate the atomically published teacher-free prerequisite against the
/// exact source/bundle selection and the current compiled input bytes.
///
/// This intentionally lives in the test-only certifier surface: it authorizes
/// the direct ignored tuner, not deployed serving. Every compiled input is
/// rehashed, so a copied or stale AVAILABLE report cannot spend teacher work.
#[cfg(all(test, not(target_arch = "wasm32")))]
pub(crate) fn validate_teacher_free_preflight(
    report_path: &std::path::Path,
    source_dir: &std::path::Path,
    bundle_dir: &std::path::Path,
) -> Result<TeacherFreePreflightAdmission, TeacherFreePreflightAdmissionError> {
    use std::io::Read;
    use TeacherFreePreflightAdmissionError as Error;

    fn canonical(
        path: &std::path::Path,
        label: &str,
    ) -> Result<std::path::PathBuf, TeacherFreePreflightAdmissionError> {
        std::fs::canonicalize(path).map_err(|error| {
            TeacherFreePreflightAdmissionError::Unavailable(format!(
                "UNAVAILABLE: resolve {label} {}: {error}",
                path.display()
            ))
        })
    }

    fn file_cid(path: &std::path::Path) -> Result<String, TeacherFreePreflightAdmissionError> {
        let mut file = std::fs::File::open(path).map_err(|error| {
            TeacherFreePreflightAdmissionError::Unavailable(format!(
                "UNAVAILABLE: open teacher-free prerequisite {}: {error}",
                path.display()
            ))
        })?;
        let mut hasher = blake3::Hasher::new();
        let mut buffer = [0u8; 64 * 1024];
        loop {
            let read = file.read(&mut buffer).map_err(|error| {
                TeacherFreePreflightAdmissionError::Unavailable(format!(
                    "UNAVAILABLE: read teacher-free prerequisite {}: {error}",
                    path.display()
                ))
            })?;
            if read == 0 {
                break;
            }
            hasher.update(&buffer[..read]);
        }
        Ok(format!("blake3:{}", hasher.finalize().to_hex()))
    }

    let report_bytes = std::fs::read(report_path).map_err(|error| {
        Error::Unavailable(format!(
            "UNAVAILABLE: read teacher-free preflight {}: {error}",
            report_path.display()
        ))
    })?;
    let report: serde_json::Value = serde_json::from_slice(&report_bytes).map_err(|error| {
        Error::Failed(format!(
            "FAILED: parse teacher-free preflight {}: {error}",
            report_path.display()
        ))
    })?;
    let object = report.as_object().ok_or_else(|| {
        Error::Failed("FAILED: teacher-free preflight root is not an object".to_owned())
    })?;
    if object.get("schema").and_then(serde_json::Value::as_str)
        != Some(TEACHER_FREE_PREFLIGHT_SCHEMA)
    {
        return Err(Error::Failed(
            "FAILED: teacher-free preflight schema is absent or unsupported".to_owned(),
        ));
    }
    let current_contract_cid = exact_executor_contract_cid();
    if object
        .get("authorizing_contract_cid")
        .and_then(serde_json::Value::as_str)
        != Some(current_contract_cid.as_str())
    {
        return Err(Error::Refused(
            "NOT_RUN / REFUSED: teacher-free preflight was not emitted by the current authorizing contract"
                .to_owned(),
        ));
    }
    let status = object
        .get("status")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| Error::Failed("FAILED: teacher-free preflight omitted status".to_owned()))?;
    if status != "AVAILABLE" {
        let reason = object
            .get("reason")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("teacher-free prerequisite did not qualify");
        return Err(Error::Refused(format!(
            "NOT_RUN / REFUSED: teacher-free preflight status is {status}: {reason}"
        )));
    }
    if object
        .get("teacher_source_opened")
        .and_then(serde_json::Value::as_bool)
        != Some(false)
        || object
            .get("teacher_forwards")
            .and_then(serde_json::Value::as_u64)
            != Some(0)
    {
        return Err(Error::Failed(
            "FAILED: teacher-free preflight does not prove zero teacher work".to_owned(),
        ));
    }

    let report_path = canonical(report_path, "teacher-free preflight report")?;
    let source_dir = canonical(source_dir, "selected teacher source")?;
    let bundle_dir = canonical(bundle_dir, "selected compiled bundle")?;
    for (field, expected) in [
        ("report_path", report_path.as_path()),
        ("selected_source_dir", source_dir.as_path()),
        ("selected_bundle_dir", bundle_dir.as_path()),
    ] {
        let recorded = object
            .get(field)
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| {
                Error::Failed(format!("FAILED: teacher-free preflight omitted {field}"))
            })?;
        let recorded = canonical(std::path::Path::new(recorded), field)?;
        if recorded != expected {
            return Err(Error::Refused(format!(
                "NOT_RUN / REFUSED: teacher-free preflight {field} does not match the current selection"
            )));
        }
    }

    let inputs = object
        .get("inputs")
        .and_then(serde_json::Value::as_object)
        .ok_or_else(|| Error::Failed("FAILED: teacher-free preflight omitted inputs".to_owned()))?;
    let mut compiled_input_cids = std::collections::BTreeMap::new();
    for (name, relative) in [
        ("tokenizer", "tokenizer.bin"),
        ("legacy_artifact", "tless_artifacts.bin"),
        ("legacy_store", "tless_store.bin"),
        ("graph", "graph/score.r4g1"),
        ("graph_report", "graph/score_report.json"),
    ] {
        let recorded = inputs
            .get(name)
            .and_then(serde_json::Value::as_str)
            .filter(|cid| cid.starts_with("blake3:"))
            .ok_or_else(|| {
                Error::Failed(format!(
                    "FAILED: teacher-free preflight omitted the content identity for {name}"
                ))
            })?;
        let current = file_cid(&bundle_dir.join(relative))?;
        if current != recorded {
            return Err(Error::Refused(format!(
                "NOT_RUN / REFUSED: teacher-free preflight {name} identity is stale"
            )));
        }
        compiled_input_cids.insert(name.to_owned(), current);
    }

    let recorded_production = object
        .get("production_admission")
        .and_then(serde_json::Value::as_object)
        .ok_or_else(|| {
            Error::Failed("FAILED: teacher-free preflight omitted production_admission".to_owned())
        })?;
    let current_production =
        production_admission_component_cids(&bundle_dir).map_err(|reason| {
            let reason = reason.reason;
            if reason.starts_with("UNAVAILABLE:") {
                Error::Unavailable(reason)
            } else {
                Error::Failed(reason)
            }
        })?;
    if recorded_production.len() != current_production.len() {
        return Err(Error::Refused(
            "NOT_RUN / REFUSED: teacher-free preflight production generation is incomplete"
                .to_owned(),
        ));
    }
    for (name, current) in &current_production {
        let recorded = recorded_production
            .get(name)
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| {
                Error::Failed(format!(
                    "FAILED: teacher-free preflight omitted production component {name}"
                ))
            })?;
        if recorded != current {
            return Err(Error::Refused(format!(
                "NOT_RUN / REFUSED: teacher-free preflight production component {name} is stale"
            )));
        }
    }
    let semantically_verified_production = verify_production_envelope_semantics(&bundle_dir)?;
    if semantically_verified_production != current_production {
        return Err(Error::Refused(
            "NOT_RUN / REFUSED: production generation changed during schema-2 semantic verification"
                .to_owned(),
        ));
    }

    Ok(TeacherFreePreflightAdmission {
        report_cid: format!("blake3:{}", blake3::hash(&report_bytes).to_hex()),
        selected_source_dir: source_dir.display().to_string(),
        selected_bundle_dir: bundle_dir.display().to_string(),
        compiled_input_cids,
        production_admission_cids: current_production,
    })
}

/// Content identity of the exact executor, probe contract, and teacher call
/// surface compiled into this crate.
///
/// Binding admission to source bytes prevents evidence from an older scheduler
/// or teacher implementation from admitting a newly built full run.
fn exact_executor_contract_components() -> Vec<(&'static str, &'static [u8])> {
    macro_rules! workspace_component {
        ($path:literal) => {
            (
                $path,
                include_bytes!(concat!("../../../", $path)).as_slice(),
            )
        };
    }

    vec![
        ("schema", EXACT_MULTICORE_PROBE_SCHEMA.as_bytes()),
        ("uor-matmul-revision", UOR_MATMUL_REVISION.as_bytes()),
        workspace_component!("Cargo.toml"),
        workspace_component!("Cargo.lock"),
        workspace_component!("rust-toolchain.toml"),
        workspace_component!(".cargo/config.toml"),
        workspace_component!("tests/bdd.rs"),
        workspace_component!("tests/support/parity_observability.rs"),
        workspace_component!("features/suites/teacher_parity_benchmarks.feature"),
        workspace_component!("model/ids.toml"),
        workspace_component!("src/lib.rs"),
        workspace_component!("src/r4g1.rs"),
        workspace_component!("src/release_bundle_loader.rs"),
        workspace_component!("crates/uor-r4-api/Cargo.toml"),
        workspace_component!("crates/uor-r4-api/src/capability_suite.rs"),
        workspace_component!("crates/uor-r4-api/src/compile.rs"),
        workspace_component!("crates/uor-r4-api/src/deployed_quality.rs"),
        workspace_component!("crates/uor-r4-api/src/engine.rs"),
        workspace_component!("crates/uor-r4-api/src/lib.rs"),
        workspace_component!("crates/uor-r4-api/src/production_envelope.rs"),
        workspace_component!("crates/uor-r4-api/src/release_bundle.rs"),
        workspace_component!("crates/uor-r4-api/src/serving.rs"),
        workspace_component!("crates/uor-r4-api/src/serving_eval.rs"),
        workspace_component!("crates/uor-r4-api/src/witness_replay.rs"),
        workspace_component!("crates/uor-r4-core/Cargo.toml"),
        workspace_component!("crates/uor-r4-core/src/bin/r4-group-geometry-export.rs"),
        workspace_component!("crates/uor-r4-core/src/bin/r4-h4-spin-frame-export.rs"),
        workspace_component!("crates/uor-r4-core/src/bounded_global_exact_spin_attention.rs"),
        workspace_component!("crates/uor-r4-core/src/canonical_lexical_ingestion.rs"),
        workspace_component!("crates/uor-r4-core/src/cayley_dickson.rs"),
        workspace_component!("crates/uor-r4-core/src/construction_causal_return_attention.rs"),
        workspace_component!("crates/uor-r4-core/src/conversation_entity_spin_path_attention.rs"),
        workspace_component!("crates/uor-r4-core/src/corpus_induced_spin_placement.rs"),
        workspace_component!("crates/uor-r4-core/src/direct_causal_geometric_attention.rs"),
        workspace_component!("crates/uor-r4-core/src/geometric_gated_delta_retention.rs"),
        workspace_component!("crates/uor-r4-core/src/helm_d_r4_attention.rs"),
        workspace_component!("crates/uor-r4-core/src/h4_spin_frame_sidecar.rs"),
        workspace_component!("crates/uor-r4-core/src/higher_scope_geometric_attention.rs"),
        workspace_component!("crates/uor-r4-core/src/lib.rs"),
        workspace_component!("crates/uor-r4-core/src/local_geometric_generation.rs"),
        workspace_component!("crates/uor-r4-core/src/paragraph_entity_spin_path_attention.rs"),
        workspace_component!("crates/uor-r4-core/src/prime_route_attention.rs"),
        workspace_component!("crates/uor-r4-core/src/prime_route_geometric_attention.rs"),
        workspace_component!("crates/uor-r4-core/src/r4_group_addressed_retention.rs"),
        workspace_component!("crates/uor-r4-core/src/r4_softmax_trace_state_student.rs"),
        workspace_component!("crates/uor-r4-core/src/r4_softmax_trace_student.rs"),
        workspace_component!("crates/uor-r4-core/src/recursive_geometric_attention.rs"),
        workspace_component!("crates/uor-r4-core/src/semantic/manifest.rs"),
        workspace_component!("crates/uor-r4-core/src/semantic/merkle.rs"),
        workspace_component!("crates/uor-r4-core/src/semantic/mod.rs"),
        workspace_component!("crates/uor-r4-core/src/semantic/reasoning.rs"),
        workspace_component!("crates/uor-r4-core/src/semantic/reference.rs"),
        workspace_component!("crates/uor-r4-core/src/semantic/vsa.rs"),
        workspace_component!("crates/uor-r4-core/src/source_free_table.rs"),
        workspace_component!("crates/uor-r4-core/src/spiralcore_operator.rs"),
        workspace_component!("crates/uor-r4-core/src/transformerless/bott_fock.rs"),
        workspace_component!("crates/uor-r4-core/src/transformerless/cd_space.rs"),
        workspace_component!("crates/uor-r4-core/src/transformerless/code_sidecar.rs"),
        workspace_component!("crates/uor-r4-core/src/transformerless/compiler.rs"),
        workspace_component!("crates/uor-r4-core/src/transformerless/convert_r4g1.rs"),
        workspace_component!("crates/uor-r4-core/src/transformerless/endomorphism.rs"),
        workspace_component!("crates/uor-r4-core/src/transformerless/graph_patch.rs"),
        workspace_component!("crates/uor-r4-core/src/transformerless/hf_bpe.rs"),
        workspace_component!("crates/uor-r4-core/src/transformerless/lie_jordan.rs"),
        workspace_component!("crates/uor-r4-core/src/transformerless/mod.rs"),
        workspace_component!("crates/uor-r4-core/src/transformerless/reference_state.rs"),
        workspace_component!("crates/uor-r4-core/src/transformerless/region_store.rs"),
        workspace_component!("crates/uor-r4-core/src/transformerless/resolution_status.rs"),
        workspace_component!("crates/uor-r4-core/src/transformerless/runtime.rs"),
        workspace_component!("crates/uor-r4-core/src/transformerless/scenarios.rs"),
        workspace_component!("crates/uor-r4-core/src/transformerless/score_q.rs"),
        workspace_component!("crates/uor-r4-core/src/transformerless/sentencepiece.rs"),
        workspace_component!("crates/uor-r4-core/src/transformerless/simd.rs"),
        workspace_component!("crates/uor-r4-core/src/transformerless/source_scan.rs"),
        workspace_component!("crates/uor-r4-core/src/transformerless/transitions.rs"),
        workspace_component!("crates/uor-r4-core/src/zeta_projection.rs"),
        workspace_component!("crates/uor-r4-core/src/zeta_zeros.rs"),
        workspace_component!("crates/uor-r4-graph-compiler/Cargo.toml"),
        workspace_component!("crates/uor-r4-graph-compiler/src/behavioral_probes.rs"),
        workspace_component!("crates/uor-r4-graph-compiler/src/compositional_planning.rs"),
        workspace_component!("crates/uor-r4-graph-compiler/src/dependency_audit.rs"),
        workspace_component!("crates/uor-r4-graph-compiler/src/executor.rs"),
        workspace_component!("crates/uor-r4-graph-compiler/src/future_state_planner.rs"),
        workspace_component!("crates/uor-r4-graph-compiler/src/graph.rs"),
        workspace_component!("crates/uor-r4-graph-compiler/src/induction.rs"),
        workspace_component!("crates/uor-r4-graph-compiler/src/jobs_config.rs"),
        workspace_component!("crates/uor-r4-graph-compiler/src/lib.rs"),
        workspace_component!("crates/uor-r4-graph-compiler/src/lower_semantic_regions.rs"),
        workspace_component!("crates/uor-r4-graph-compiler/src/memory_budget.rs"),
        workspace_component!("crates/uor-r4-graph-compiler/src/monograph.rs"),
        workspace_component!("crates/uor-r4-graph-compiler/src/observation.rs"),
        workspace_component!("crates/uor-r4-graph-compiler/src/observation_shards.rs"),
        workspace_component!("crates/uor-r4-graph-compiler/src/observation_text.rs"),
        workspace_component!("crates/uor-r4-graph-compiler/src/pack.rs"),
        workspace_component!("crates/uor-r4-graph-compiler/src/patch_induction.rs"),
        workspace_component!("crates/uor-r4-graph-compiler/src/perturbation.rs"),
        workspace_component!("crates/uor-r4-graph-compiler/src/probability_calibration.rs"),
        workspace_component!("crates/uor-r4-graph-compiler/src/quantum_cover.rs"),
        workspace_component!("crates/uor-r4-graph-compiler/src/rate_distortion_compression.rs"),
        workspace_component!("crates/uor-r4-graph-compiler/src/recorded_corpus.rs"),
        workspace_component!("crates/uor-r4-graph-compiler/src/reference_compiler_ir.rs"),
        workspace_component!("crates/uor-r4-graph-compiler/src/reproducibility.rs"),
        workspace_component!("crates/uor-r4-graph-compiler/src/residual.rs"),
        workspace_component!("crates/uor-r4-graph-compiler/src/route_fit.rs"),
        workspace_component!("crates/uor-r4-graph-compiler/src/routing.rs"),
        workspace_component!("crates/uor-r4-graph-compiler/src/segment_fit.rs"),
        workspace_component!("crates/uor-r4-graph-compiler/src/semantic_emission_decoupling.rs"),
        workspace_component!("crates/uor-r4-graph-compiler/src/semantic_state.rs"),
        workspace_component!("crates/uor-r4-graph-compiler/src/semantic_transitions.rs"),
        workspace_component!("crates/uor-r4-graph-compiler/src/skipmix_fit.rs"),
        workspace_component!("crates/uor-r4-graph-compiler/src/stage_dag.rs"),
        workspace_component!("crates/uor-r4-graph-compiler/src/trace_profile.rs"),
        workspace_component!("crates/uor-r4-graph-format/Cargo.toml"),
        workspace_component!("crates/uor-r4-graph-format/src/code.rs"),
        workspace_component!("crates/uor-r4-graph-format/src/error.rs"),
        workspace_component!("crates/uor-r4-graph-format/src/fmm.rs"),
        workspace_component!("crates/uor-r4-graph-format/src/fwda.rs"),
        workspace_component!("crates/uor-r4-graph-format/src/head.rs"),
        workspace_component!("crates/uor-r4-graph-format/src/header.rs"),
        workspace_component!("crates/uor-r4-graph-format/src/inference_contract.rs"),
        workspace_component!("crates/uor-r4-graph-format/src/invariant_ownership.rs"),
        workspace_component!("crates/uor-r4-graph-format/src/lib.rs"),
        workspace_component!("crates/uor-r4-graph-format/src/msa_selector.rs"),
        workspace_component!("crates/uor-r4-graph-format/src/ngram.rs"),
        workspace_component!("crates/uor-r4-graph-format/src/plan.rs"),
        workspace_component!("crates/uor-r4-graph-format/src/plan_sections.rs"),
        workspace_component!("crates/uor-r4-graph-format/src/prov.rs"),
        workspace_component!("crates/uor-r4-graph-format/src/pstate.rs"),
        workspace_component!("crates/uor-r4-graph-format/src/r4g1.rs"),
        workspace_component!("crates/uor-r4-graph-format/src/records.rs"),
        workspace_component!("crates/uor-r4-graph-format/src/rout.rs"),
        workspace_component!("crates/uor-r4-graph-format/src/route_attention.rs"),
        workspace_component!("crates/uor-r4-graph-format/src/sanctioned.rs"),
        workspace_component!("crates/uor-r4-graph-format/src/scoring_semantics.rs"),
        workspace_component!("crates/uor-r4-graph-format/src/ser.rs"),
        workspace_component!("crates/uor-r4-graph-format/src/skipmix.rs"),
        workspace_component!("crates/uor-r4-graph-format/src/stage2.rs"),
        workspace_component!("crates/uor-r4-graph-format/src/types.rs"),
        workspace_component!("crates/uor-r4-graph-format/src/view.rs"),
        workspace_component!("crates/uor-r4-graph-runtime/Cargo.toml"),
        workspace_component!("crates/uor-r4-graph-runtime/src/engine.rs"),
        workspace_component!("crates/uor-r4-graph-runtime/src/lib.rs"),
        workspace_component!("crates/uor-r4-graph-runtime/src/msa_selector.rs"),
        workspace_component!("crates/uor-r4-graph-runtime/src/packed_kernels.rs"),
        workspace_component!("crates/uor-r4-graph-runtime/src/patch_chain.rs"),
        workspace_component!("crates/uor-r4-graph-runtime/src/plan.rs"),
        workspace_component!("crates/uor-r4-graph-runtime/src/route_attention.rs"),
        workspace_component!("crates/uor-r4-graph-runtime/src/routing.rs"),
        workspace_component!("crates/uor-r4-graph-runtime/src/runtime_state.rs"),
        workspace_component!("crates/uor-r4-graph-runtime/src/scoring.rs"),
        workspace_component!("crates/uor-r4-graph-runtime/src/status.rs"),
        workspace_component!("crates/uor-r4-graph-runtime/src/vp_tree.rs"),
        workspace_component!("crates/uor-r4-graph-certify/Cargo.toml"),
        workspace_component!("crates/uor-r4-graph-certify/src/anti_degeneracy.rs"),
        workspace_component!("crates/uor-r4-graph-certify/src/certificate.rs"),
        workspace_component!("crates/uor-r4-graph-certify/src/certify.rs"),
        workspace_component!("crates/uor-r4-graph-certify/src/compare.rs"),
        workspace_component!("crates/uor-r4-graph-certify/src/compiler_scaling.rs"),
        workspace_component!(
            "crates/uor-r4-graph-certify/src/corpus_signed_transport_attention.rs"
        ),
        workspace_component!("crates/uor-r4-graph-certify/src/fairness_provenance.rs"),
        workspace_component!("crates/uor-r4-graph-certify/src/fmm.rs"),
        workspace_component!("crates/uor-r4-graph-certify/src/frame_consistency.rs"),
        workspace_component!("crates/uor-r4-graph-certify/src/holographic_encoding.rs"),
        workspace_component!("crates/uor-r4-graph-certify/src/lib.rs"),
        workspace_component!("crates/uor-r4-graph-certify/src/long_context.rs"),
        workspace_component!("crates/uor-r4-graph-certify/src/msa_ab_harness.rs"),
        workspace_component!("crates/uor-r4-graph-certify/src/msa_selector.rs"),
        workspace_component!("crates/uor-r4-graph-certify/src/octeract.rs"),
        workspace_component!("crates/uor-r4-graph-certify/src/octeract_trace_screen.rs"),
        workspace_component!("crates/uor-r4-graph-certify/src/patch_lifecycle.rs"),
        workspace_component!("crates/uor-r4-graph-certify/src/performance_certificate.rs"),
        workspace_component!("crates/uor-r4-graph-certify/src/predictive_sufficiency.rs"),
        workspace_component!("crates/uor-r4-graph-certify/src/prime_route_worker_canary.rs"),
        workspace_component!("crates/uor-r4-graph-certify/src/route_attention.rs"),
        workspace_component!("crates/uor-r4-graph-certify/src/route_cost.rs"),
        workspace_component!("crates/uor-r4-graph-certify/src/route_fit_report.rs"),
        workspace_component!("crates/uor-r4-graph-certify/src/score.rs"),
        workspace_component!("crates/uor-r4-graph-certify/src/score_runtime.rs"),
        workspace_component!("crates/uor-r4-graph-certify/src/shortlist_evaluator.rs"),
        workspace_component!("crates/uor-r4-graph-certify/src/target_operator_certificate.rs"),
        workspace_component!("crates/uor-r4-model-source/Cargo.toml"),
        workspace_component!("crates/uor-r4-model-source/src/attention.rs"),
        workspace_component!("crates/uor-r4-model-source/src/conformance.rs"),
        workspace_component!("crates/uor-r4-model-source/src/dense.rs"),
        workspace_component!("crates/uor-r4-model-source/src/exact_executor.rs"),
        workspace_component!("crates/uor-r4-model-source/src/exact_probe.rs"),
        workspace_component!("crates/uor-r4-model-source/src/geometric_decoder.rs"),
        workspace_component!("crates/uor-r4-model-source/src/geometric_training.rs"),
        workspace_component!("crates/uor-r4-model-source/src/geometry.rs"),
        workspace_component!("crates/uor-r4-model-source/src/gpt2.rs"),
        workspace_component!("crates/uor-r4-model-source/src/lib.rs"),
        workspace_component!("crates/uor-r4-model-source/src/observation_blas_exception.rs"),
        workspace_component!("crates/uor-r4-model-source/src/progress.rs"),
        workspace_component!("crates/uor-r4-model-source/src/teacher.rs"),
        workspace_component!("third_party/arrayref/Cargo.toml"),
        workspace_component!("third_party/arrayref/src/lib.rs"),
    ]
}

pub fn exact_executor_contract_cid() -> String {
    let mut hasher = blake3::Hasher::new();
    for (label, bytes) in exact_executor_contract_components() {
        hasher.update(&u64::try_from(label.len()).unwrap_or(u64::MAX).to_le_bytes());
        hasher.update(label.as_bytes());
        hasher.update(&u64::try_from(bytes.len()).unwrap_or(u64::MAX).to_le_bytes());
        hasher.update(bytes);
    }
    for (label, value) in [
        ("target-arch", std::env::consts::ARCH),
        ("target-os", std::env::consts::OS),
        ("target-family", std::env::consts::FAMILY),
        (
            "target-endian",
            if cfg!(target_endian = "little") {
                "little"
            } else {
                "big"
            },
        ),
        (
            "target-pointer-width",
            if cfg!(target_pointer_width = "64") {
                "64"
            } else if cfg!(target_pointer_width = "32") {
                "32"
            } else {
                "other"
            },
        ),
        (
            "debug-assertions",
            if cfg!(debug_assertions) { "on" } else { "off" },
        ),
        (
            "observation-blas-exception",
            if cfg!(feature = "observation-blas-exception") {
                "on"
            } else {
                "off"
            },
        ),
    ] {
        hasher.update(&u64::try_from(label.len()).unwrap_or(u64::MAX).to_le_bytes());
        hasher.update(label.as_bytes());
        hasher.update(&u64::try_from(value.len()).unwrap_or(u64::MAX).to_le_bytes());
        hasher.update(value.as_bytes());
    }
    #[cfg(target_arch = "aarch64")]
    for (feature, enabled) in [
        ("aes", cfg!(target_feature = "aes")),
        ("crc", cfg!(target_feature = "crc")),
        ("dotprod", cfg!(target_feature = "dotprod")),
        ("fp16", cfg!(target_feature = "fp16")),
        ("neon", cfg!(target_feature = "neon")),
        ("sve", cfg!(target_feature = "sve")),
    ] {
        hasher.update(feature.as_bytes());
        hasher.update(&[u8::from(enabled)]);
    }
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    for (feature, enabled) in [
        ("avx", cfg!(target_feature = "avx")),
        ("avx2", cfg!(target_feature = "avx2")),
        ("fma", cfg!(target_feature = "fma")),
        ("sse2", cfg!(target_feature = "sse2")),
        ("sse4.1", cfg!(target_feature = "sse4.1")),
    ] {
        hasher.update(feature.as_bytes());
        hasher.update(&[u8::from(enabled)]);
    }
    format!("blake3:{}", hasher.finalize().to_hex())
}

/// Content-bound identity of the live teacher fixture used by a probe.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ExactMulticoreProbeSource {
    /// Loader κ over the validated Safetensors shard bytes.
    pub model_kappa: String,
    /// BLAKE3 CID of the exact `config.json` bytes.
    pub config_cid: String,
    /// Total validated Safetensors bytes.
    pub source_bytes: u64,
}

/// Host identity and capacity that bound a probe result.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ExactMulticoreProbeHost {
    /// Rust target architecture.
    pub target_arch: String,
    /// Rust target operating system.
    pub target_os: String,
    /// Capacity returned by `std::thread::available_parallelism()`.
    pub available_parallelism: usize,
    /// Operating-system version, when the host exposes it safely.
    pub operating_system_version: Option<String>,
    /// CPU model string, when the host exposes it safely.
    pub cpu_model: Option<String>,
    /// Physical CPU core count, when the host exposes it safely.
    pub physical_core_count: Option<usize>,
    /// Performance-cluster logical cores, when the host distinguishes them.
    pub performance_logical_core_count: Option<usize>,
    /// Efficiency-cluster logical cores, when the host distinguishes them.
    pub efficiency_logical_core_count: Option<usize>,
    /// Explicit reason for every unavailable topology field.
    pub topology_unavailable_reason: Option<String>,
}

/// Discover the host identity used to bind exact probe admission.
#[cfg(not(target_arch = "wasm32"))]
pub fn exact_probe_host_identity() -> ExactMulticoreProbeHost {
    let available_parallelism = std::thread::available_parallelism().map_or(1, usize::from);
    #[cfg(target_os = "macos")]
    {
        fn sysctl(name: &str) -> Result<String, crate::SourceUnavailable> {
            let output = std::process::Command::new("/usr/sbin/sysctl")
                .args(["-n", name])
                .output()
                .map_err(|error| {
                    crate::SourceUnavailable::new(format!("sysctl {name}: {error}"))
                })?;
            if !output.status.success() {
                return Err(crate::SourceUnavailable::new(format!(
                    "sysctl {name} exited {}",
                    output.status
                )));
            }
            String::from_utf8(output.stdout)
                .map(|value| value.trim().to_owned())
                .map_err(|error| {
                    crate::SourceUnavailable::new(format!(
                        "sysctl {name} returned non-UTF-8 output: {error}"
                    ))
                })
        }

        fn nonempty(name: &str, unavailable: &mut Vec<String>) -> Option<String> {
            match sysctl(name) {
                Ok(value) if !value.is_empty() => Some(value),
                Ok(_) => {
                    unavailable.push(format!("{name} returned an empty value"));
                    None
                }
                Err(reason) => {
                    unavailable.push(reason.to_string());
                    None
                }
            }
        }

        fn positive_usize(name: &str, unavailable: &mut Vec<String>) -> Option<usize> {
            match sysctl(name).and_then(|raw| {
                raw.parse::<usize>().map_err(|error| {
                    crate::SourceUnavailable::new(format!("{name} returned {raw:?}: {error}"))
                })
            }) {
                Ok(value) if value > 0 => Some(value),
                Ok(_) => {
                    unavailable.push(format!("{name} reported zero"));
                    None
                }
                Err(reason) => {
                    unavailable.push(reason.to_string());
                    None
                }
            }
        }

        let mut unavailable = Vec::new();
        let operating_system_version = nonempty("kern.osproductversion", &mut unavailable);
        let cpu_model = nonempty("machdep.cpu.brand_string", &mut unavailable)
            .or_else(|| nonempty("hw.model", &mut unavailable));
        let physical_core_count = positive_usize("hw.physicalcpu", &mut unavailable);
        let performance_logical_core_count =
            positive_usize("hw.perflevel0.logicalcpu", &mut unavailable);
        let efficiency_logical_core_count =
            positive_usize("hw.perflevel1.logicalcpu", &mut unavailable);
        ExactMulticoreProbeHost {
            target_arch: std::env::consts::ARCH.to_owned(),
            target_os: std::env::consts::OS.to_owned(),
            available_parallelism,
            operating_system_version,
            cpu_model,
            physical_core_count,
            performance_logical_core_count,
            efficiency_logical_core_count,
            topology_unavailable_reason: (!unavailable.is_empty()).then(|| unavailable.join("; ")),
        }
    }
    #[cfg(not(target_os = "macos"))]
    {
        ExactMulticoreProbeHost {
            target_arch: std::env::consts::ARCH.to_owned(),
            target_os: std::env::consts::OS.to_owned(),
            available_parallelism,
            operating_system_version: None,
            cpu_model: None,
            physical_core_count: None,
            performance_logical_core_count: None,
            efficiency_logical_core_count: None,
            topology_unavailable_reason: Some(format!(
                "safe exact-probe topology discovery is not implemented for {}",
                std::env::consts::OS
            )),
        }
    }
}

/// Portable counterpart for a target without hosted topology discovery.
#[cfg(target_arch = "wasm32")]
pub fn exact_probe_host_identity() -> ExactMulticoreProbeHost {
    ExactMulticoreProbeHost {
        target_arch: std::env::consts::ARCH.to_owned(),
        target_os: std::env::consts::OS.to_owned(),
        available_parallelism: 1,
        operating_system_version: None,
        cpu_model: None,
        physical_core_count: None,
        performance_logical_core_count: None,
        efficiency_logical_core_count: None,
        topology_unavailable_reason: Some(
            "host topology discovery is unavailable on wasm32".to_owned(),
        ),
    }
}

/// Per-configuration CPU/RSS evidence.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ExactMulticoreProbeResources {
    /// `AVAILABLE`, `PARTIAL`, or `UNAVAILABLE`.
    pub status: String,
    /// `EXACT_FORWARD_INTERVALS_ONLY` or `FULL_PROBE_WALL`.
    pub measurement_scope: String,
    /// Process CPU time at configuration start.
    pub cpu_time_start_seconds: Option<f64>,
    /// Process CPU time at configuration completion.
    pub cpu_time_end_seconds: Option<f64>,
    /// CPU seconds consumed by this configuration.
    pub cpu_time_consumed_seconds: Option<f64>,
    /// Resident bytes at configuration completion.
    pub current_rss_bytes: Option<u64>,
    /// Largest resident-set sample observed during the configuration.
    pub max_sampled_rss_bytes: Option<u64>,
    /// True OS-maintained peak resident bytes, when safely exposed.
    pub peak_rss_bytes: Option<u64>,
    /// Mean CPU core-equivalents (`CPU seconds / wall seconds`).
    pub mean_cpu_core_equivalents: Option<f64>,
    /// Mean process CPU percentage (`100 * core-equivalents`).
    pub mean_cpu_percent: Option<f64>,
    /// Required explanation for unavailable fields.
    pub reason: Option<String>,
}

/// Redundant completeness accounting for every content-bound output trace.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ExactMulticoreProbeTraceShape {
    /// Forward positions retained in the trace.
    pub positions: usize,
    /// Independent state records retained at every position.
    pub streams_per_position: usize,
    /// Private KV/attention position capacity allocated per state.
    pub sequence_capacity: usize,
    /// Total position/stream state records.
    pub state_records: u64,
    /// Full raw-logit words retained per state record.
    pub logits_per_state: usize,
    /// Total raw-logit words retained across the trace.
    pub logit_words: u64,
    /// Total raw-logit bytes retained across the trace.
    pub logit_bytes: u64,
    /// Persistent state words retained per state record.
    pub persistent_state_words_per_state: usize,
    /// Total persistent state words retained across the trace.
    pub persistent_state_words: u64,
    /// One canonical greedy token retained per state record.
    pub greedy_tokens: u64,
    /// Canonical top-k width retained per state record.
    pub top_k: usize,
    /// Total canonical top-k token ids retained across the trace.
    pub top_tokens: u64,
}

impl ExactMulticoreProbeResources {
    /// Explicitly unavailable portable process-resource evidence.
    pub fn unavailable(reason: impl Into<String>) -> Self {
        Self {
            status: "UNAVAILABLE".to_owned(),
            measurement_scope: "UNAVAILABLE".to_owned(),
            cpu_time_start_seconds: None,
            cpu_time_end_seconds: None,
            cpu_time_consumed_seconds: None,
            current_rss_bytes: None,
            max_sampled_rss_bytes: None,
            peak_rss_bytes: None,
            mean_cpu_core_equivalents: None,
            mean_cpu_percent: None,
            reason: Some(reason.into()),
        }
    }
}

/// Cheap executor/backend initialization excluded from candidate timing.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ExactMulticoreProbePrestart {
    /// Wall time spent waking the pool and exercising a tiny exact GEMM.
    pub elapsed_seconds: f64,
    /// Dedicated workers observed through the pool-wide barrier.
    pub workers_observed: usize,
    /// Shared-weight width exercised by the tiny exact GEMM.
    pub batch_width: usize,
    /// Whether the known one-hot product returned every expected output bit.
    pub backend_exercised: bool,
    /// Retained model/executor workspace capacity prepared for this candidate.
    pub workspace_capacity_bytes: u64,
    /// Retained buffers whose capacity grew during excluded preparation.
    pub workspace_growth_events: u64,
    /// Actual `Vec` capacity bytes added during excluded preparation.
    pub workspace_growth_bytes: u64,
    /// Must remain true: prestart is not part of the projected forward rate.
    pub excluded_from_measurement: bool,
}

/// One fixed-worker adaptive candidate result.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ExactMulticoreProbeRun {
    /// Dedicated worker count.
    pub workers: usize,
    /// Independent sequence states advanced together through shared weights.
    pub batch_width: usize,
    /// Cheap pool/backend initialization, never a model forward.
    pub prestart: ExactMulticoreProbePrestart,
    /// Wall time for the fixed all-stream workload.
    pub elapsed_ms: u64,
    /// Exact wall seconds used to derive rates and CPU core-equivalents.
    pub elapsed_seconds: f64,
    /// Aggregate logical stream-forwards per second.
    pub aggregate_forwards_per_second: f64,
    /// Rate divided by the four-worker candidate rate (diagnostic only).
    pub relative_throughput_vs_worker4: f64,
    /// Whether all persistent output/state bits equal the first candidate.
    pub equal_to_reference: bool,
    /// Canonical BLAKE3 identity of every tested position and sequence state.
    pub output_trace_cid: String,
    /// Redundant trace-completeness counters bound by admission.
    pub trace_shape: ExactMulticoreProbeTraceShape,
    /// Owner-computed counter plan for one shared-weight batch forward.
    pub forward_plan: ExactForwardPlan,
    /// Whether peak row tasks reached the effective worker bound.
    pub all_workers_active: bool,
    /// Whether the full independent-stream cohort was observed in flight.
    pub all_streams_active: bool,
    /// Raw full-suite projection at this rate.
    pub raw_projected_suite_seconds: f64,
    /// Safety-adjusted full-suite projection at this rate.
    pub safety_adjusted_projected_suite_seconds: f64,
    /// Exact execution counters after the workload.
    pub snapshot: TeacherExecutionSnapshot,
    /// CPU/RSS evidence or an explicit unavailability reason.
    pub resources: ExactMulticoreProbeResources,
}

/// Fastest measured exact candidate selected for the full suite.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ExactMulticoreProbeSelection {
    /// Worker count.
    pub workers: usize,
    /// Output-row tiles requested per worker.
    pub tiles_per_worker: usize,
    /// Aggregate logical stream-forwards per second.
    pub aggregate_forwards_per_second: f64,
    /// Raw projected exact-teacher wall time for the optimized suite.
    pub raw_projected_suite_seconds: f64,
    /// Safety-adjusted projected exact-teacher wall time.
    pub safety_adjusted_projected_suite_seconds: f64,
}

/// Owner-computed exact counter plan for one fixed worker configuration.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ExactMulticoreProbeWorkerPlan {
    pub workers: usize,
    pub forward_plan: ExactForwardPlan,
}

/// Config-only expectation geometry computed without opening model weights.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExactMulticoreProbeExpectationShapes {
    pub forward_plans: Vec<ExactMulticoreProbeWorkerPlan>,
    pub trace_shape: ExactMulticoreProbeTraceShape,
}

/// Configured/default live teacher work projected by the probe.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ExactMulticoreProbeWork {
    /// Exact teacher forwards used to build the shared transcript templates.
    pub transcript_logical_forwards: usize,
    /// Maximum continuation forwards measured per cloned lane.
    pub generation_tokens_per_lane: usize,
    /// Independent cloned continuation lanes.
    pub generation_lanes: usize,
    /// Derived optimized logical forward count; no duplicated prefill/warm-up.
    pub logical_forwards: usize,
    /// Physical shared-weight transcript forwards under the registered S8
    /// scheduler (six at the pinned 36-forward cap).
    pub transcript_physical_batches: usize,
    /// Physical S8 continuation forwards (one per continuation token step).
    pub generation_physical_batches: usize,
    /// Derived physical forwards used for conservative projection.
    pub physical_batches: usize,
    /// Maximum zero-based private-state position exercised by the suite.
    pub max_sequence_position: usize,
    /// Allocated private-state capacity covering that maximum position.
    pub state_sequence_capacity: usize,
}

impl ExactMulticoreProbeWork {
    /// Recompute the declared logical work with saturating arithmetic.
    pub fn derived_logical_forwards(&self) -> usize {
        self.transcript_logical_forwards.saturating_add(
            self.generation_lanes
                .saturating_mul(self.generation_tokens_per_lane),
        )
    }

    /// Recompute registered transcript batches for the configured logical cap.
    pub fn derived_transcript_physical_batches(&self) -> usize {
        let mut remaining = self.transcript_logical_forwards;
        let mut batches = 0usize;
        for width in EXACT_MULTICORE_PROBE_REGISTERED_TRANSCRIPT_BATCH_WIDTHS {
            if remaining == 0 {
                break;
            }
            remaining = remaining.saturating_sub(width);
            batches = batches.saturating_add(1);
        }
        batches
    }

    /// Recompute the optimized suite's physical shared-weight forwards.
    pub fn derived_physical_batches(&self) -> usize {
        self.derived_transcript_physical_batches()
            .saturating_add(self.generation_tokens_per_lane)
    }

    /// Conservative bounded context exercised by the cheap probe.
    ///
    /// Return the explicitly bound maximum private-state horizon.
    pub fn derived_probe_context_ceiling_tokens(&self) -> usize {
        self.state_sequence_capacity
    }

    /// Whether this is the complete registered work shape authorized to admit
    /// the optimized live suite. Smaller operator caps remain useful as
    /// diagnostics but can never publish a binding qualified verdict.
    pub(crate) fn is_registered_binding_work(&self) -> bool {
        self.transcript_logical_forwards == EXACT_MULTICORE_PROBE_REGISTERED_TRANSCRIPT_FORWARDS
            && self.generation_tokens_per_lane == EXACT_MULTICORE_PROBE_REGISTERED_GENERATION_TOKENS
            && self.generation_lanes == EXACT_MULTICORE_PROBE_REGISTERED_GENERATION_LANES
            && self.logical_forwards
                == EXACT_MULTICORE_PROBE_REGISTERED_TRANSCRIPT_FORWARDS.saturating_add(
                    EXACT_MULTICORE_PROBE_REGISTERED_GENERATION_LANES
                        .saturating_mul(EXACT_MULTICORE_PROBE_REGISTERED_GENERATION_TOKENS),
                )
            && self.transcript_physical_batches
                == EXACT_MULTICORE_PROBE_REGISTERED_TRANSCRIPT_BATCH_WIDTHS.len()
            && self.generation_physical_batches
                == EXACT_MULTICORE_PROBE_REGISTERED_GENERATION_TOKENS
            && self.physical_batches
                == EXACT_MULTICORE_PROBE_REGISTERED_TRANSCRIPT_BATCH_WIDTHS
                    .len()
                    .saturating_add(EXACT_MULTICORE_PROBE_REGISTERED_GENERATION_TOKENS)
            && self.max_sequence_position == EXACT_MULTICORE_PROBE_REGISTERED_MAX_SEQUENCE_POSITION
            && self.state_sequence_capacity
                == EXACT_MULTICORE_PROBE_REGISTERED_STATE_SEQUENCE_CAPACITY
    }
}

/// Binding admission status.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ExactMulticoreProbeStatus {
    /// All predeclared gates qualified the full run.
    Qualified,
    /// At least one gate refused the full run.
    RefuseFullRun,
}

/// Machine-readable full-run admission verdict.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ExactMulticoreProbeVerdict {
    /// Qualified/refused status.
    pub status: ExactMulticoreProbeStatus,
    /// Adaptive rule used to choose the measured exact configuration.
    pub selection_policy: String,
    /// Operator-configured wall limit.
    pub configured_max_wall_seconds: u64,
    /// Binding ceiling (never greater than eight hours).
    pub qualification_wall_seconds: u64,
    /// Redundant fail-closed boolean for simple consumers.
    pub qualifies_full_run: bool,
}

/// Content binding for the finalized sibling JSONL event artifact.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ExactMulticoreProbeEventsBinding {
    /// Sibling file name derived from the canonical report path.
    pub file_name: String,
    /// BLAKE3 identity of every finalized JSONL byte.
    pub content_cid: String,
    /// Exact finalized JSONL byte count.
    pub byte_len: u64,
    /// Exact nonempty JSONL record count.
    pub record_count: u64,
    /// One-based record number of the required last `FINAL` event.
    pub final_record_number: u64,
    /// Required terminal event name (`FINAL`).
    pub final_event: String,
    /// Terminal status copied from the binding verdict.
    pub final_status: ExactMulticoreProbeStatus,
    /// Terminal qualification flag copied from the binding verdict.
    pub final_qualifies_full_run: bool,
    /// CID of canonical report fields with this binding replaced by its
    /// deterministic pending placeholder, avoiding a cyclic hash.
    pub report_body_cid: String,
}

impl ExactMulticoreProbeEventsBinding {
    #[cfg(not(target_arch = "wasm32"))]
    fn pending(file_name: impl Into<String>, status: ExactMulticoreProbeStatus) -> Self {
        Self {
            file_name: file_name.into(),
            content_cid: "PENDING".to_owned(),
            byte_len: 0,
            record_count: 0,
            final_record_number: 0,
            final_event: "PENDING".to_owned(),
            final_status: status,
            final_qualifies_full_run: false,
            report_body_cid: "PENDING".to_owned(),
        }
    }
}

/// Durable exact multicore probe report consumed by live-suite admission.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ExactMulticoreProbeReport {
    /// [`EXACT_MULTICORE_PROBE_SCHEMA`].
    pub schema: String,
    /// Content identity returned by [`exact_executor_contract_cid`].
    pub executor_contract_cid: String,
    /// Bound teacher source identity.
    pub source: ExactMulticoreProbeSource,
    /// Bound host identity/capacity.
    pub host: ExactMulticoreProbeHost,
    /// Exact arithmetic and observable kernel provenance.
    pub backend: ExactBackendReport,
    /// Positions run by every worker configuration.
    pub probe_positions: usize,
    /// Configured maximum context shape represented by the timed probe.
    pub probe_context_ceiling_tokens: usize,
    /// Exact full-context prefix index timed by every measured repetition.
    pub probe_position_indices: Vec<usize>,
    /// Independent streams used by every worker configuration.
    pub probe_streams: usize,
    /// Binding cheap-probe admission deadline.
    pub probe_wall_ceiling_seconds: u64,
    /// Exact semantics for an operation already active at the deadline.
    pub probe_deadline_policy: String,
    /// Finalized content identity and terminal record of the sibling JSONL.
    pub events: ExactMulticoreProbeEventsBinding,
    /// Observed probe wall time including fixture load.
    pub probe_elapsed_seconds: f64,
    /// Fixed-worker measurements (four workers and all available workers,
    /// deduplicated when the host exposes exactly four).
    pub runs: Vec<ExactMulticoreProbeRun>,
    /// Worker count of the first candidate whose exact trace is the reference.
    pub reference_workers: usize,
    /// Fastest observed qualified candidate.
    pub selected_best_config: ExactMulticoreProbeSelection,
    /// Exact worker/tile configuration the full suite will execute.
    pub configured_execution: ExactMulticoreProbeSelection,
    /// Aggregate bit-equality verdict.
    pub exact_equality: bool,
    /// Aggregate worker-participation diagnostic.
    pub all_workers_active: bool,
    /// Aggregate independent-stream participation verdict.
    pub all_streams_active: bool,
    /// Configured/default suite work.
    pub configured_suite_work: ExactMulticoreProbeWork,
    /// Selected-candidate projection before safety margin.
    pub raw_projected_suite_seconds: f64,
    /// Multiplicative projection safety margin.
    pub projection_safety_factor: f64,
    /// Explicit formal-vocabulary boundary for context-length projection.
    pub projection_context_assumption: String,
    /// Selected-candidate projection after safety margin.
    pub safety_adjusted_projected_suite_seconds: f64,
    /// Binding admission verdict.
    pub binding_verdict: ExactMulticoreProbeVerdict,
    /// Overall CPU/RSS evidence or explicit unavailability.
    pub resources: ExactMulticoreProbeResources,
}

/// Live state a consumer requires a durable report to match.
#[derive(Clone, Debug, PartialEq)]
pub struct ExactMulticoreProbeExpectation {
    /// Current executor/build contract identity.
    pub executor_contract_cid: String,
    /// Current live source identity.
    pub source: ExactMulticoreProbeSource,
    /// Current host identity and capacity.
    pub host: ExactMulticoreProbeHost,
    /// Exact full-suite work budget.
    pub configured_suite_work: ExactMulticoreProbeWork,
    /// Owner-computed counter plans for every measured worker configuration.
    pub forward_plans: Vec<ExactMulticoreProbeWorkerPlan>,
    /// Expected complete trace dimensions for the cheap probe.
    pub trace_shape: ExactMulticoreProbeTraceShape,
    /// Output-row tiles per worker the admitted full suite will actually use.
    pub tiles_per_worker: usize,
    /// Full-suite wall ceiling configured by the operator.
    pub configured_max_wall_seconds: u64,
}

/// Focused reason durable evidence cannot admit live teacher work.
#[derive(Clone, Debug, PartialEq)]
pub enum ExactMulticoreProbeValidationError {
    SchemaMismatch,
    ExecutorContractMismatch,
    SourceIdentityMismatch,
    HostIdentityMismatch,
    HostTopologyUnavailable,
    BudgetMismatch(&'static str),
    BackendMismatch,
    MissingWorkerConfiguration(usize),
    ExactEqualityNotEstablished,
    WorkerParticipationNotEstablished,
    BatchedExecutionNotEstablished,
    ResourceEvidenceUnavailable,
    EventsEvidenceUnavailable,
    EventsEvidenceMismatch(&'static str),
    ProbeWallCeilingExceeded,
    ProjectionExceedsLimit,
    NumericEvidenceMismatch(&'static str),
    VerdictNotQualified,
}

impl std::fmt::Display for ExactMulticoreProbeValidationError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "exact multicore probe admission refused: {self:?}"
        )
    }
}

impl std::error::Error for ExactMulticoreProbeValidationError {}

impl ExactMulticoreProbeReport {
    /// Validate report fields without treating the report alone as admission
    /// evidence. Live callers must use [`Self::validate_for_with_events`].
    pub fn validate_for(
        &self,
        expected: &ExactMulticoreProbeExpectation,
    ) -> Result<(), ExactMulticoreProbeValidationError> {
        use ExactMulticoreProbeValidationError as Error;

        fn same_f64(left: f64, right: f64) -> bool {
            left.to_bits() == right.to_bits()
        }

        fn validate_resources_if_available(
            resources: &ExactMulticoreProbeResources,
            elapsed_seconds: f64,
            expected_scope: &str,
        ) -> Result<(), ExactMulticoreProbeValidationError> {
            if !elapsed_seconds.is_finite() || elapsed_seconds <= 0.0 {
                return Err(Error::NumericEvidenceMismatch("resource_elapsed"));
            }
            if resources.status == "UNAVAILABLE" {
                if resources.measurement_scope != "UNAVAILABLE"
                    || resources
                        .reason
                        .as_deref()
                        .map(str::is_empty)
                        .unwrap_or(true)
                    || resources.cpu_time_start_seconds.is_some()
                    || resources.cpu_time_end_seconds.is_some()
                    || resources.cpu_time_consumed_seconds.is_some()
                    || resources.current_rss_bytes.is_some()
                    || resources.max_sampled_rss_bytes.is_some()
                    || resources.peak_rss_bytes.is_some()
                    || resources.mean_cpu_core_equivalents.is_some()
                    || resources.mean_cpu_percent.is_some()
                {
                    return Err(Error::NumericEvidenceMismatch("resource_unavailable"));
                }
                return Ok(());
            }
            let (
                Some(cpu_start),
                Some(cpu_end),
                Some(cpu_consumed),
                Some(current_rss),
                Some(max_sampled_rss),
                Some(mean_cores),
                Some(mean_percent),
            ) = (
                resources.cpu_time_start_seconds,
                resources.cpu_time_end_seconds,
                resources.cpu_time_consumed_seconds,
                resources.current_rss_bytes,
                resources.max_sampled_rss_bytes,
                resources.mean_cpu_core_equivalents,
                resources.mean_cpu_percent,
            )
            else {
                return Err(Error::NumericEvidenceMismatch("resource_evidence"));
            };
            if !matches!(resources.status.as_str(), "AVAILABLE" | "PARTIAL")
                || resources.measurement_scope != expected_scope
                || !cpu_start.is_finite()
                || !cpu_end.is_finite()
                || !cpu_consumed.is_finite()
                || cpu_start < 0.0
                || cpu_end < cpu_start
                || current_rss == 0
                || max_sampled_rss < current_rss
                || !mean_cores.is_finite()
                || mean_cores < 0.0
                || !mean_percent.is_finite()
                || mean_percent < 0.0
                || cpu_consumed > cpu_end - cpu_start + f64::EPSILON
                || !same_f64(mean_cores, cpu_consumed / elapsed_seconds)
                || !same_f64(mean_percent, mean_cores * 100.0)
            {
                return Err(Error::NumericEvidenceMismatch("resource_evidence"));
            }
            if resources.peak_rss_bytes.is_none()
                && (resources.status != "PARTIAL" || resources.reason.is_none())
            {
                return Err(Error::NumericEvidenceMismatch("resource_peak_status"));
            }
            if let Some(peak_rss) = resources.peak_rss_bytes {
                if peak_rss < max_sampled_rss {
                    return Err(Error::NumericEvidenceMismatch("resource_peak_rss"));
                }
            }
            Ok(())
        }

        if self.schema != EXACT_MULTICORE_PROBE_SCHEMA {
            return Err(Error::SchemaMismatch);
        }
        if self.executor_contract_cid != expected.executor_contract_cid
            || self.executor_contract_cid != exact_executor_contract_cid()
        {
            return Err(Error::ExecutorContractMismatch);
        }
        if self.source != expected.source {
            return Err(Error::SourceIdentityMismatch);
        }
        if self.host != expected.host {
            return Err(Error::HostIdentityMismatch);
        }
        if self.host.cpu_model.is_none() || self.host.physical_core_count.is_none() {
            return Err(Error::HostTopologyUnavailable);
        }
        if self.host.available_parallelism < 4 {
            return Err(Error::BudgetMismatch("minimum_adaptive_host"));
        }
        if self.configured_suite_work != expected.configured_suite_work {
            return Err(Error::BudgetMismatch("configured_suite_work"));
        }
        if self.configured_suite_work.logical_forwards
            != self.configured_suite_work.derived_logical_forwards()
        {
            return Err(Error::BudgetMismatch("logical_forwards"));
        }
        if self.configured_suite_work.transcript_physical_batches
            != self
                .configured_suite_work
                .derived_transcript_physical_batches()
            || self.configured_suite_work.generation_physical_batches
                != self.configured_suite_work.generation_tokens_per_lane
            || self.configured_suite_work.physical_batches
                != self.configured_suite_work.derived_physical_batches()
        {
            return Err(Error::BudgetMismatch("physical_batches"));
        }
        if !self.configured_suite_work.is_registered_binding_work() {
            return Err(Error::BudgetMismatch("optimized_suite_work"));
        }
        if self.probe_streams != 8 || self.probe_positions == 0 {
            return Err(Error::BudgetMismatch("probe_shape"));
        }
        // Aggregate teacher-forced work and one private state's bounded
        // sequence horizon are deliberately independent. The caller computes
        // the latter through the config-only trace-shape owner; never recover
        // it by treating all logical transcript positions as one sequence.
        let context_ceiling = expected.trace_shape.sequence_capacity;
        if context_ceiling == 0
            || self.probe_context_ceiling_tokens != context_ceiling
            || context_ceiling
                < self
                    .configured_suite_work
                    .derived_probe_context_ceiling_tokens()
            || self.probe_position_indices.len() != self.probe_positions
            || self
                .probe_position_indices
                .iter()
                .any(|&position| position != context_ceiling - 1)
        {
            return Err(Error::BudgetMismatch("probe_context_shape"));
        }
        if self.probe_deadline_policy != EXACT_MULTICORE_PROBE_DEADLINE_POLICY {
            return Err(Error::BudgetMismatch("probe_deadline_policy"));
        }
        if self.probe_wall_ceiling_seconds != EXACT_MULTICORE_PROBE_WALL_CEILING_SECONDS
            || !self.probe_elapsed_seconds.is_finite()
            || self.probe_elapsed_seconds <= 0.0
            || self.probe_elapsed_seconds >= self.probe_wall_ceiling_seconds as f64
        {
            return Err(Error::ProbeWallCeilingExceeded);
        }
        if self.projection_context_assumption != EXACT_MULTICORE_PROBE_CONTEXT_ASSUMPTION {
            return Err(Error::BudgetMismatch("projection_context_assumption"));
        }
        let current_backend = exact_backend_report();
        if self.backend != current_backend
            || current_backend.arithmetic_owner != "uor-matmul exact GEMM"
            || !current_backend.std_runtime_detection_enabled
            || current_backend.uor_matmul_revision != UOR_MATMUL_REVISION
            || current_backend.target_arch != self.host.target_arch
            || current_backend.target_os != self.host.target_os
        {
            return Err(Error::BackendMismatch);
        }

        let mut required_workers = vec![4usize, self.host.available_parallelism];
        required_workers.sort_unstable();
        required_workers.dedup();
        let mut planned_workers: Vec<usize> = expected
            .forward_plans
            .iter()
            .map(|plan| plan.workers)
            .collect();
        planned_workers.sort_unstable();
        planned_workers.dedup();
        if planned_workers != required_workers
            || expected.forward_plans.len() != required_workers.len()
        {
            return Err(Error::BudgetMismatch("candidate_worker_plans"));
        }
        if self.runs.len() != required_workers.len() {
            for workers in &required_workers {
                if !self.runs.iter().any(|run| run.workers == *workers) {
                    return Err(Error::MissingWorkerConfiguration(*workers));
                }
            }
            return Err(Error::NumericEvidenceMismatch("candidate_count"));
        }
        if self.reference_workers != self.runs[0].workers
            || !required_workers.contains(&self.reference_workers)
        {
            return Err(Error::NumericEvidenceMismatch("reference_workers"));
        }
        for workers in &required_workers {
            if self
                .runs
                .iter()
                .filter(|run| run.workers == *workers)
                .count()
                != 1
            {
                return Err(Error::MissingWorkerConfiguration(*workers));
            }
        }
        let worker4 = self
            .runs
            .iter()
            .find(|run| run.workers == 4)
            .ok_or(Error::MissingWorkerConfiguration(4))?;
        if !worker4.aggregate_forwards_per_second.is_finite()
            || worker4.aggregate_forwards_per_second <= 0.0
        {
            return Err(Error::NumericEvidenceMismatch("worker4_rate"));
        }

        let physical_batches = self.configured_suite_work.physical_batches as f64;
        let measured_forwards =
            self.probe_streams
                .checked_mul(self.probe_positions)
                .ok_or(Error::NumericEvidenceMismatch("probe_forward_count"))? as f64;
        let trace_records = u64::try_from(self.probe_streams)
            .unwrap_or(u64::MAX)
            .saturating_mul(u64::try_from(self.probe_positions).unwrap_or(u64::MAX));
        if expected.trace_shape.positions != self.probe_positions
            || expected.trace_shape.streams_per_position != self.probe_streams
            || expected.trace_shape.sequence_capacity != self.probe_context_ceiling_tokens
            || expected.trace_shape.state_records != trace_records
            || expected.trace_shape.logit_words
                != trace_records.saturating_mul(
                    u64::try_from(expected.trace_shape.logits_per_state).unwrap_or(u64::MAX),
                )
            || expected.trace_shape.logit_bytes
                != expected.trace_shape.logit_words.saturating_mul(4)
            || expected.trace_shape.persistent_state_words
                != trace_records.saturating_mul(
                    u64::try_from(expected.trace_shape.persistent_state_words_per_state)
                        .unwrap_or(u64::MAX),
                )
            || expected.trace_shape.greedy_tokens != trace_records
            || expected.trace_shape.top_tokens
                != trace_records
                    .saturating_mul(u64::try_from(expected.trace_shape.top_k).unwrap_or(u64::MAX))
        {
            return Err(Error::NumericEvidenceMismatch("trace_completeness"));
        }
        for run in &self.runs {
            if run.workers == 0
                || run.batch_width != self.probe_streams
                || self
                    .runs
                    .iter()
                    .filter(|candidate| candidate.workers == run.workers)
                    .count()
                    != 1
                || !run.aggregate_forwards_per_second.is_finite()
                || run.aggregate_forwards_per_second <= 0.0
                || !run.elapsed_seconds.is_finite()
                || run.elapsed_seconds <= 0.0
                || !run.prestart.elapsed_seconds.is_finite()
                || run.prestart.elapsed_seconds < 0.0
                || run.prestart.workers_observed != run.workers
                || run.prestart.batch_width != self.probe_streams
                || !run.prestart.backend_exercised
                || run.prestart.workspace_capacity_bytes == 0
                || run.prestart.workspace_growth_events == 0
                || run.prestart.workspace_growth_bytes == 0
                || !run.prestart.excluded_from_measurement
            {
                return Err(Error::NumericEvidenceMismatch("run_rate"));
            }
            let measured_rate = measured_forwards / run.elapsed_seconds;
            let relative_to_worker4 =
                run.aggregate_forwards_per_second / worker4.aggregate_forwards_per_second;
            let raw_projection =
                physical_batches * run.elapsed_seconds / self.probe_positions as f64;
            let adjusted_projection = raw_projection * self.projection_safety_factor;
            if !same_f64(run.aggregate_forwards_per_second, measured_rate)
                || !same_f64(run.relative_throughput_vs_worker4, relative_to_worker4)
                || !same_f64(run.raw_projected_suite_seconds, raw_projection)
                || !same_f64(
                    run.safety_adjusted_projected_suite_seconds,
                    adjusted_projection,
                )
            {
                return Err(Error::NumericEvidenceMismatch("run_arithmetic"));
            }
            let Some(expected_plan) = expected
                .forward_plans
                .iter()
                .find(|plan| plan.workers == run.workers)
            else {
                return Err(Error::MissingWorkerConfiguration(run.workers));
            };
            if expected
                .forward_plans
                .iter()
                .filter(|plan| plan.workers == run.workers)
                .count()
                != 1
                || run.forward_plan != expected_plan.forward_plan
                || run.trace_shape != expected.trace_shape
                || run.trace_shape.positions != self.probe_positions
                || run.trace_shape.streams_per_position != self.probe_streams
            {
                return Err(Error::NumericEvidenceMismatch("trace_or_plan_shape"));
            }
            let positions_u64 = u64::try_from(self.probe_positions).unwrap_or(u64::MAX);
            let expected_matrix_calls = run.forward_plan.matrix_calls.saturating_mul(positions_u64);
            let expected_tiles = run.forward_plan.row_tiles.saturating_mul(positions_u64);
            let expected_cells = run.forward_plan.output_cells.saturating_mul(positions_u64);
            let expected_terms = run.forward_plan.scalar_terms.saturating_mul(positions_u64);
            if run.snapshot.requested_workers != run.workers
                || run.snapshot.effective_workers != run.workers
                || run.snapshot.active_workers != 0
                || run.snapshot.max_active_workers > run.workers
                || run.snapshot.forward_max_active_workers > run.snapshot.max_active_workers
                || run.snapshot.forward_calls != positions_u64
                || run.snapshot.matrix_calls != expected_matrix_calls
                || run.snapshot.batched_matrix_calls != expected_matrix_calls
                || run.snapshot.max_matrix_batch_width != self.probe_streams
                || run.snapshot.tiles_completed != expected_tiles
                || run.snapshot.output_cells_completed != expected_cells
                || run.snapshot.scalar_terms_completed != expected_terms
                || run.snapshot.multiworker_forward_calls != positions_u64
                || !(2..=run.workers).contains(&run.snapshot.forward_max_active_workers)
                || run.snapshot.workspace_growth_events != 0
                || run.snapshot.workspace_growth_bytes != 0
                || run.snapshot.observer_epoch == 0
            {
                return Err(Error::BatchedExecutionNotEstablished);
            }
            validate_resources_if_available(
                &run.resources,
                run.elapsed_seconds,
                "EXACT_FORWARD_INTERVALS_ONLY",
            )?;
            if !run.output_trace_cid.starts_with("blake3:")
                || run.equal_to_reference != (run.output_trace_cid == self.runs[0].output_trace_cid)
            {
                return Err(Error::NumericEvidenceMismatch("output_trace_cid"));
            }
            let observed_all_workers = run.snapshot.requested_workers == run.workers
                && run.snapshot.effective_workers == run.workers
                && run.snapshot.active_workers == 0
                && run.snapshot.max_active_workers == run.workers;
            if run.all_workers_active != observed_all_workers {
                return Err(Error::NumericEvidenceMismatch("run_participation"));
            }
            let expected_stream_steps = self.probe_streams.saturating_mul(self.probe_positions);
            let observed_all_streams = run.snapshot.active_streams == 0
                && run.snapshot.max_active_streams == run.batch_width
                && run.snapshot.streams_started
                    == u64::try_from(expected_stream_steps).unwrap_or(u64::MAX)
                && run.snapshot.streams_completed
                    == u64::try_from(expected_stream_steps).unwrap_or(u64::MAX);
            if run.all_streams_active != observed_all_streams {
                return Err(Error::NumericEvidenceMismatch("run_stream_participation"));
            }
        }

        for workers in &required_workers {
            let Some(run) = self.runs.iter().find(|run| run.workers == *workers) else {
                return Err(Error::MissingWorkerConfiguration(*workers));
            };
            if !run.equal_to_reference {
                return Err(Error::ExactEqualityNotEstablished);
            }
            if !run.all_streams_active {
                return Err(Error::WorkerParticipationNotEstablished);
            }
        }

        let aggregate_equality = self.runs.iter().all(|run| run.equal_to_reference);
        let aggregate_participation = self.runs.iter().all(|run| run.all_workers_active);
        let aggregate_stream_participation = self.runs.iter().all(|run| run.all_streams_active);
        if self.exact_equality != aggregate_equality {
            return Err(Error::NumericEvidenceMismatch("aggregate_equality"));
        }
        if self.all_workers_active != aggregate_participation {
            return Err(Error::NumericEvidenceMismatch("aggregate_participation"));
        }
        if self.all_streams_active != aggregate_stream_participation {
            return Err(Error::NumericEvidenceMismatch(
                "aggregate_stream_participation",
            ));
        }
        if !self.exact_equality {
            return Err(Error::ExactEqualityNotEstablished);
        }
        if !self.all_streams_active {
            return Err(Error::WorkerParticipationNotEstablished);
        }

        let selected_run = self
            .runs
            .iter()
            .min_by(|left, right| {
                left.safety_adjusted_projected_suite_seconds
                    .total_cmp(&right.safety_adjusted_projected_suite_seconds)
                    .then_with(|| left.workers.cmp(&right.workers))
            })
            .ok_or(Error::NumericEvidenceMismatch("selected_run"))?;
        if self.configured_execution.tiles_per_worker != expected.tiles_per_worker
            || self.selected_best_config.tiles_per_worker != expected.tiles_per_worker
        {
            return Err(Error::BudgetMismatch("tiles_per_worker"));
        }
        let expected_selection = ExactMulticoreProbeSelection {
            workers: selected_run.workers,
            tiles_per_worker: expected.tiles_per_worker,
            aggregate_forwards_per_second: selected_run.aggregate_forwards_per_second,
            raw_projected_suite_seconds: selected_run.raw_projected_suite_seconds,
            safety_adjusted_projected_suite_seconds: selected_run
                .safety_adjusted_projected_suite_seconds,
        };
        if self.selected_best_config != expected_selection
            || self.configured_execution != expected_selection
        {
            return Err(Error::NumericEvidenceMismatch("selected_best_config"));
        }

        if !self.projection_safety_factor.is_finite() || self.projection_safety_factor < 1.25 {
            return Err(Error::ProjectionExceedsLimit);
        }
        if !same_f64(
            self.raw_projected_suite_seconds,
            selected_run.raw_projected_suite_seconds,
        ) || !same_f64(
            self.safety_adjusted_projected_suite_seconds,
            selected_run.safety_adjusted_projected_suite_seconds,
        ) {
            return Err(Error::NumericEvidenceMismatch("configured_projection"));
        }
        if self.binding_verdict.selection_policy != EXACT_MULTICORE_PROBE_SELECTION_POLICY {
            return Err(Error::BudgetMismatch("selection_policy"));
        }
        validate_resources_if_available(
            &self.resources,
            self.probe_elapsed_seconds,
            "FULL_PROBE_WALL",
        )?;
        let qualification_wall_seconds = expected.configured_max_wall_seconds.min(28_800);
        if self.binding_verdict.configured_max_wall_seconds != expected.configured_max_wall_seconds
            || self.binding_verdict.qualification_wall_seconds != qualification_wall_seconds
            || qualification_wall_seconds == 0
            || !self.safety_adjusted_projected_suite_seconds.is_finite()
            || self.safety_adjusted_projected_suite_seconds >= qualification_wall_seconds as f64
        {
            return Err(Error::ProjectionExceedsLimit);
        }

        let recomputed_qualification = self.exact_equality
            && self.all_streams_active
            && self.safety_adjusted_projected_suite_seconds < qualification_wall_seconds as f64;
        if self.binding_verdict.qualifies_full_run != recomputed_qualification
            || (self.binding_verdict.status == ExactMulticoreProbeStatus::Qualified)
                != recomputed_qualification
        {
            return Err(Error::NumericEvidenceMismatch("binding_verdict"));
        }
        if self.binding_verdict.status != ExactMulticoreProbeStatus::Qualified
            || !self.binding_verdict.qualifies_full_run
        {
            return Err(Error::VerdictNotQualified);
        }
        Ok(())
    }

    /// Fail-closed admission validation of both the typed report and its
    /// finalized sibling JSONL artifact.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn validate_for_with_events(
        &self,
        report_path: impl AsRef<std::path::Path>,
        expected: &ExactMulticoreProbeExpectation,
    ) -> Result<(), ExactMulticoreProbeValidationError> {
        use ExactMulticoreProbeValidationError as Error;

        self.validate_for(expected)?;
        let report_path = report_path.as_ref();
        let expected_file_name = events_file_name_for_report(report_path);
        if self.events.file_name != expected_file_name {
            return Err(Error::EventsEvidenceMismatch("events_file_name"));
        }
        let events_path = normalized_report_parent(report_path).join(&expected_file_name);
        let bytes = std::fs::read(events_path).map_err(|_| Error::EventsEvidenceUnavailable)?;
        let report_body_cid = self
            .report_body_cid()
            .map_err(|_| Error::EventsEvidenceMismatch("report_body_cid"))?;
        let observed = finalized_events_binding(
            expected_file_name,
            &bytes,
            &report_body_cid,
            self.binding_verdict.status,
            self.binding_verdict.qualifies_full_run,
        )?;
        if self.events != observed {
            return Err(Error::EventsEvidenceMismatch("events_content_binding"));
        }
        Ok(())
    }

    /// Portable targets cannot admit a host-qualified probe without the
    /// required local event artifact.
    #[cfg(target_arch = "wasm32")]
    pub fn validate_for_with_events(
        &self,
        _report_path: impl AsRef<std::path::Path>,
        _expected: &ExactMulticoreProbeExpectation,
    ) -> Result<(), ExactMulticoreProbeValidationError> {
        Err(ExactMulticoreProbeValidationError::EventsEvidenceUnavailable)
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn report_body_cid(&self) -> Result<String, crate::SourceUnavailable> {
        let mut body = self.clone();
        body.events = ExactMulticoreProbeEventsBinding::pending(
            self.events.file_name.clone(),
            self.binding_verdict.status,
        );
        let mut bytes = serde_json::to_vec_pretty(&body)?;
        bytes.push(b'\n');
        Ok(format!("blake3:{}", blake3::hash(&bytes).to_hex()))
    }

    /// Atomically publish a flushed report in the destination directory.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn write_atomic(
        &self,
        path: impl AsRef<std::path::Path>,
    ) -> Result<(), crate::SourceUnavailable> {
        let path = path.as_ref();
        let bytes = self.canonical_bytes()?;
        write_atomic_bytes(path, &bytes)
    }

    /// Commit a canonical report only after its required `FINAL` event has
    /// been durably written by `emit_final`.
    ///
    /// The callback receives the CID of canonical report fields excluding the
    /// cyclic event binding. If it fails, this method never touches the report
    /// path, so an earlier RUNNING or non-PASS state cannot become qualified.
    #[cfg(all(test, not(target_arch = "wasm32")))]
    pub(crate) fn write_after_durable_final(
        &mut self,
        report_path: impl AsRef<std::path::Path>,
        events_path: impl AsRef<std::path::Path>,
        emit_final: impl FnOnce(&str, u64) -> std::io::Result<()>,
    ) -> std::io::Result<String> {
        let report_path = report_path.as_ref();
        let events_path = events_path.as_ref();
        let events_file_name = events_file_name_for_report(report_path);
        if events_path.file_name() != Some(std::ffi::OsStr::new(&events_file_name)) {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "exact probe events path does not match report-derived sibling name",
            ));
        }
        self.events = ExactMulticoreProbeEventsBinding::pending(
            events_file_name.clone(),
            self.binding_verdict.status,
        );
        let report_body_cid = self
            .report_body_cid()
            .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidData, error))?;
        let prefix_bytes = std::fs::read(events_path)?;
        if !prefix_bytes.is_empty() && prefix_bytes.last() != Some(&b'\n') {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "exact probe JSONL prefix is not newline terminated",
            ));
        }
        let final_record_number =
            u64::try_from(prefix_bytes.iter().filter(|&&byte| byte == b'\n').count())
                .unwrap_or(u64::MAX)
                .checked_add(1)
                .ok_or_else(|| {
                    std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "exact probe JSONL record count overflow",
                    )
                })?;
        emit_final(&report_body_cid, final_record_number)?;
        let events_bytes = std::fs::read(events_path)?;
        self.events = finalized_events_binding(
            events_file_name,
            &events_bytes,
            &report_body_cid,
            self.binding_verdict.status,
            self.binding_verdict.qualifies_full_run,
        )
        .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidData, error))?;
        let bytes = self
            .canonical_bytes()
            .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidData, error))?;
        let cid = format!("blake3:{}", blake3::hash(&bytes).to_hex());
        write_atomic_bytes(report_path, &bytes).map_err(std::io::Error::other)?;
        Ok(cid)
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn canonical_bytes(&self) -> Result<Vec<u8>, crate::SourceUnavailable> {
        let mut bytes = serde_json::to_vec_pretty(self)?;
        bytes.push(b'\n');
        Ok(bytes)
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn events_file_name_for_report(report_path: &std::path::Path) -> String {
    let stem = report_path
        .file_stem()
        .and_then(std::ffi::OsStr::to_str)
        .unwrap_or("exact-multicore-probe");
    format!("{stem}.events.jsonl")
}

#[cfg(not(target_arch = "wasm32"))]
fn finalized_events_binding(
    file_name: String,
    bytes: &[u8],
    expected_report_body_cid: &str,
    expected_status: ExactMulticoreProbeStatus,
    expected_qualifies_full_run: bool,
) -> Result<ExactMulticoreProbeEventsBinding, ExactMulticoreProbeValidationError> {
    use ExactMulticoreProbeValidationError as Error;

    if bytes.is_empty() || bytes.last() != Some(&b'\n') {
        return Err(Error::EventsEvidenceMismatch("events_termination"));
    }
    let mut record_count = 0u64;
    let mut final_record = None;
    for line in bytes[..bytes.len() - 1].split(|byte| *byte == b'\n') {
        if line.is_empty() {
            return Err(Error::EventsEvidenceMismatch("events_empty_record"));
        }
        let record: serde_json::Value = serde_json::from_slice(line)
            .map_err(|_| Error::EventsEvidenceMismatch("events_jsonl"))?;
        record_count = record_count
            .checked_add(1)
            .ok_or(Error::EventsEvidenceMismatch("events_record_count"))?;
        final_record = Some(record);
    }
    let final_record = final_record.ok_or(Error::EventsEvidenceMismatch("events_final"))?;
    let expected_status = match expected_status {
        ExactMulticoreProbeStatus::Qualified => "QUALIFIED",
        ExactMulticoreProbeStatus::RefuseFullRun => "REFUSE_FULL_RUN",
    };
    if final_record
        .get("schema")
        .and_then(serde_json::Value::as_str)
        != Some(EXACT_MULTICORE_PROBE_SCHEMA)
        || final_record
            .get("record")
            .and_then(serde_json::Value::as_str)
            != Some("EXACT_MULTICORE_PROBE")
        || final_record
            .get("event")
            .and_then(serde_json::Value::as_str)
            != Some("FINAL")
        || final_record
            .get("status")
            .and_then(serde_json::Value::as_str)
            != Some(expected_status)
        || final_record
            .get("qualifies_full_run")
            .and_then(serde_json::Value::as_bool)
            != Some(expected_qualifies_full_run)
        || final_record
            .get("report_body_cid")
            .and_then(serde_json::Value::as_str)
            != Some(expected_report_body_cid)
        || final_record
            .get("sequence")
            .and_then(serde_json::Value::as_u64)
            != Some(record_count)
    {
        return Err(Error::EventsEvidenceMismatch("events_final"));
    }
    Ok(ExactMulticoreProbeEventsBinding {
        file_name,
        content_cid: format!("blake3:{}", blake3::hash(bytes).to_hex()),
        byte_len: u64::try_from(bytes.len()).unwrap_or(u64::MAX),
        record_count,
        final_record_number: record_count,
        final_event: "FINAL".to_owned(),
        final_status: match expected_status {
            "QUALIFIED" => ExactMulticoreProbeStatus::Qualified,
            _ => ExactMulticoreProbeStatus::RefuseFullRun,
        },
        final_qualifies_full_run: expected_qualifies_full_run,
        report_body_cid: expected_report_body_cid.to_owned(),
    })
}

#[cfg(not(target_arch = "wasm32"))]
fn write_atomic_bytes(
    path: &std::path::Path,
    bytes: &[u8],
) -> Result<(), crate::SourceUnavailable> {
    use std::io::Write;

    let parent = normalized_report_parent(path);
    std::fs::create_dir_all(parent)?;
    let file_name = path
        .file_name()
        .and_then(std::ffi::OsStr::to_str)
        .unwrap_or("exact-multicore-probe.json");
    let temporary = parent.join(format!(".{file_name}.{}.tmp", std::process::id()));
    let mut file = std::fs::File::create(&temporary)?;
    file.write_all(bytes)?;
    file.flush()?;
    file.sync_all()?;
    std::fs::rename(&temporary, path)?;
    std::fs::File::open(parent)?.sync_all()?;
    Ok(())
}

#[cfg(all(
    test,
    not(all(feature = "observation-blas-exception", target_os = "macos"))
))]
mod tests {
    use super::*;

    #[test]
    fn exact_executor_contract_binds_complete_local_source_closure() {
        fn collect_rust_sources(
            workspace_root: &std::path::Path,
            directory: &std::path::Path,
            sources: &mut std::collections::BTreeSet<String>,
        ) {
            let entries = std::fs::read_dir(directory).unwrap_or_else(|error| {
                panic!(
                    "read contract source directory {}: {error}",
                    directory.display()
                )
            });
            for entry in entries {
                let path = entry.unwrap().path();
                if path.is_dir() {
                    collect_rust_sources(workspace_root, &path, sources);
                } else if path.extension().is_some_and(|extension| extension == "rs") {
                    let relative = path.strip_prefix(workspace_root).unwrap();
                    sources.insert(relative.to_string_lossy().replace('\\', "/"));
                }
            }
        }

        let workspace_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(std::path::Path::parent)
            .unwrap();
        let components = exact_executor_contract_components();
        let labels = components
            .iter()
            .map(|(label, _)| (*label).to_owned())
            .collect::<std::collections::BTreeSet<_>>();
        assert_eq!(
            labels.len(),
            components.len(),
            "contract component labels must be unique"
        );

        let mut required = std::collections::BTreeSet::from(["src/lib.rs".to_owned()]);
        for crate_path in [
            "crates/uor-r4-api",
            "crates/uor-r4-core",
            "crates/uor-r4-graph-compiler",
            "crates/uor-r4-graph-format",
            "crates/uor-r4-graph-runtime",
            "crates/uor-r4-graph-certify",
            "crates/uor-r4-model-source",
            "third_party/arrayref",
        ] {
            required.insert(format!("{crate_path}/Cargo.toml"));
            collect_rust_sources(
                workspace_root,
                &workspace_root.join(crate_path).join("src"),
                &mut required,
            );
        }

        let missing = required.difference(&labels).collect::<Vec<_>>();
        assert!(
            missing.is_empty(),
            "exact executor contract omitted local production sources: {missing:#?}"
        );
    }

    fn resources(elapsed_seconds: f64, mean_cores: f64) -> ExactMulticoreProbeResources {
        let consumed = elapsed_seconds * mean_cores;
        let mean_cores = consumed / elapsed_seconds;
        ExactMulticoreProbeResources {
            status: "PARTIAL".to_owned(),
            measurement_scope: "EXACT_FORWARD_INTERVALS_ONLY".to_owned(),
            cpu_time_start_seconds: Some(0.0),
            cpu_time_end_seconds: Some(consumed),
            cpu_time_consumed_seconds: Some(consumed),
            current_rss_bytes: Some(100),
            max_sampled_rss_bytes: Some(120),
            peak_rss_bytes: None,
            mean_cpu_core_equivalents: Some(mean_cores),
            mean_cpu_percent: Some(mean_cores * 100.0),
            reason: Some("test OS peak RSS unavailable".to_owned()),
        }
    }

    fn work() -> ExactMulticoreProbeWork {
        let mut work = ExactMulticoreProbeWork {
            transcript_logical_forwards: 36,
            generation_tokens_per_lane: 8,
            generation_lanes: 8,
            logical_forwards: 0,
            transcript_physical_batches: 0,
            generation_physical_batches: 0,
            physical_batches: 0,
            max_sequence_position: 13,
            state_sequence_capacity: 14,
        };
        work.logical_forwards = work.derived_logical_forwards();
        work.transcript_physical_batches = work.derived_transcript_physical_batches();
        work.generation_physical_batches = work.generation_tokens_per_lane;
        work.physical_batches = work.derived_physical_batches();
        work
    }

    fn source() -> ExactMulticoreProbeSource {
        ExactMulticoreProbeSource {
            model_kappa: "blake3:model".to_owned(),
            config_cid: "blake3:config".to_owned(),
            source_bytes: 42,
        }
    }

    fn host() -> ExactMulticoreProbeHost {
        ExactMulticoreProbeHost {
            target_arch: std::env::consts::ARCH.to_owned(),
            target_os: std::env::consts::OS.to_owned(),
            available_parallelism: 8,
            operating_system_version: Some("test-os".to_owned()),
            cpu_model: Some("test-cpu".to_owned()),
            physical_core_count: Some(8),
            performance_logical_core_count: Some(4),
            efficiency_logical_core_count: Some(4),
            topology_unavailable_reason: None,
        }
    }

    fn run(workers: usize, aggregate_forwards_per_second: f64) -> ExactMulticoreProbeRun {
        let elapsed_seconds = 8.0 / aggregate_forwards_per_second;
        let raw_projected_suite_seconds = work().physical_batches as f64 * elapsed_seconds;
        let forward_plan = ExactForwardPlan {
            batch_width: 8,
            matrix_calls: 2,
            row_tiles: u64::try_from(workers).unwrap(),
            worker_tasks: u64::try_from(workers).unwrap(),
            output_cells: 80,
            scalar_terms: 800,
        };
        ExactMulticoreProbeRun {
            workers,
            batch_width: 8,
            prestart: ExactMulticoreProbePrestart {
                elapsed_seconds: 0.01,
                workers_observed: workers,
                batch_width: 8,
                backend_exercised: true,
                workspace_capacity_bytes: 1_024,
                workspace_growth_events: 4,
                workspace_growth_bytes: 1_024,
                excluded_from_measurement: true,
            },
            elapsed_ms: u64::try_from((elapsed_seconds * 1_000.0) as u128).unwrap(),
            elapsed_seconds,
            aggregate_forwards_per_second,
            relative_throughput_vs_worker4: aggregate_forwards_per_second / 20.0,
            equal_to_reference: true,
            output_trace_cid: "blake3:trace".to_owned(),
            trace_shape: trace_shape(),
            forward_plan,
            all_workers_active: true,
            all_streams_active: true,
            raw_projected_suite_seconds,
            safety_adjusted_projected_suite_seconds: raw_projected_suite_seconds * 1.25,
            snapshot: TeacherExecutionSnapshot {
                observer_epoch: 1,
                requested_workers: workers,
                effective_workers: workers,
                max_active_workers: workers,
                forward_max_active_workers: workers,
                multiworker_forward_calls: 1,
                forward_calls: 1,
                streams_started: 8,
                streams_completed: 8,
                max_active_streams: 8,
                matrix_calls: 2,
                batched_matrix_calls: 2,
                max_matrix_batch_width: 8,
                tiles_completed: u64::try_from(workers).unwrap(),
                output_cells_completed: 80,
                scalar_terms_completed: 800,
                ..TeacherExecutionSnapshot::default()
            },
            resources: resources(elapsed_seconds, workers as f64 * 0.8),
        }
    }

    fn trace_shape() -> ExactMulticoreProbeTraceShape {
        ExactMulticoreProbeTraceShape {
            positions: 1,
            streams_per_position: 8,
            sequence_capacity: 14,
            state_records: 8,
            logits_per_state: 10,
            logit_words: 80,
            logit_bytes: 320,
            persistent_state_words_per_state: 20,
            persistent_state_words: 160,
            greedy_tokens: 8,
            top_k: 8,
            top_tokens: 64,
        }
    }

    fn report() -> ExactMulticoreProbeReport {
        let mut overall_resources = resources(1.0, 6.4);
        overall_resources.measurement_scope = "FULL_PROBE_WALL".to_owned();
        ExactMulticoreProbeReport {
            schema: EXACT_MULTICORE_PROBE_SCHEMA.to_owned(),
            executor_contract_cid: exact_executor_contract_cid(),
            source: source(),
            host: host(),
            backend: exact_backend_report(),
            probe_positions: 1,
            probe_context_ceiling_tokens: 14,
            probe_position_indices: vec![13],
            probe_streams: 8,
            probe_wall_ceiling_seconds: EXACT_MULTICORE_PROBE_WALL_CEILING_SECONDS,
            probe_deadline_policy: EXACT_MULTICORE_PROBE_DEADLINE_POLICY.to_owned(),
            events: ExactMulticoreProbeEventsBinding::pending(
                "probe.events.jsonl",
                ExactMulticoreProbeStatus::Qualified,
            ),
            probe_elapsed_seconds: 1.0,
            runs: vec![run(8, 40.0), run(4, 20.0)],
            reference_workers: 8,
            selected_best_config: ExactMulticoreProbeSelection {
                workers: 8,
                tiles_per_worker: 4,
                aggregate_forwards_per_second: 40.0,
                raw_projected_suite_seconds: work().physical_batches as f64 * (8.0 / 40.0),
                safety_adjusted_projected_suite_seconds: work().physical_batches as f64
                    * (8.0 / 40.0)
                    * 1.25,
            },
            configured_execution: ExactMulticoreProbeSelection {
                workers: 8,
                tiles_per_worker: 4,
                aggregate_forwards_per_second: 40.0,
                raw_projected_suite_seconds: work().physical_batches as f64 * (8.0 / 40.0),
                safety_adjusted_projected_suite_seconds: work().physical_batches as f64
                    * (8.0 / 40.0)
                    * 1.25,
            },
            exact_equality: true,
            all_workers_active: true,
            all_streams_active: true,
            configured_suite_work: work(),
            raw_projected_suite_seconds: work().physical_batches as f64 * (8.0 / 40.0),
            projection_safety_factor: 1.25,
            projection_context_assumption: EXACT_MULTICORE_PROBE_CONTEXT_ASSUMPTION.to_owned(),
            safety_adjusted_projected_suite_seconds: work().physical_batches as f64
                * (8.0 / 40.0)
                * 1.25,
            binding_verdict: ExactMulticoreProbeVerdict {
                status: ExactMulticoreProbeStatus::Qualified,
                selection_policy: EXACT_MULTICORE_PROBE_SELECTION_POLICY.to_owned(),
                configured_max_wall_seconds: 28_800,
                qualification_wall_seconds: 28_800,
                qualifies_full_run: true,
            },
            resources: overall_resources,
        }
    }

    fn expectation() -> ExactMulticoreProbeExpectation {
        ExactMulticoreProbeExpectation {
            executor_contract_cid: exact_executor_contract_cid(),
            source: source(),
            host: host(),
            configured_suite_work: work(),
            forward_plans: [4usize, 8]
                .into_iter()
                .map(|workers| ExactMulticoreProbeWorkerPlan {
                    workers,
                    forward_plan: run(workers, if workers == 4 { 20.0 } else { 40.0 }).forward_plan,
                })
                .collect(),
            trace_shape: trace_shape(),
            tiles_per_worker: 4,
            configured_max_wall_seconds: 28_800,
        }
    }

    #[test]
    fn qualified_report_validates() {
        let encoded = serde_json::to_vec(&report()).unwrap();
        let decoded: ExactMulticoreProbeReport = serde_json::from_slice(&encoded).unwrap();
        assert_eq!(
            decoded.configured_suite_work.transcript_logical_forwards,
            36
        );
        assert_eq!(decoded.configured_suite_work.logical_forwards, 100);
        assert_eq!(decoded.probe_context_ceiling_tokens, 14);
        decoded.validate_for(&expectation()).unwrap();
    }

    #[test]
    fn four_core_host_deduplicates_the_adaptive_candidates() {
        let mut report = report();
        report.host.available_parallelism = 4;
        report.host.physical_core_count = Some(4);
        report.host.performance_logical_core_count = Some(4);
        report.host.efficiency_logical_core_count = None;
        report.runs = vec![run(4, 20.0)];
        report.reference_workers = 4;
        let selection = ExactMulticoreProbeSelection {
            workers: 4,
            tiles_per_worker: 4,
            aggregate_forwards_per_second: 20.0,
            raw_projected_suite_seconds: work().physical_batches as f64 * (8.0 / 20.0),
            safety_adjusted_projected_suite_seconds: work().physical_batches as f64
                * (8.0 / 20.0)
                * 1.25,
        };
        report.selected_best_config = selection.clone();
        report.configured_execution = selection;
        report.raw_projected_suite_seconds = work().physical_batches as f64 * (8.0 / 20.0);
        report.safety_adjusted_projected_suite_seconds = report.raw_projected_suite_seconds * 1.25;
        let mut expected = expectation();
        expected.host = report.host.clone();
        expected.forward_plans = vec![ExactMulticoreProbeWorkerPlan {
            workers: 4,
            forward_plan: run(4, 20.0).forward_plan,
        }];
        report.validate_for(&expected).unwrap();
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn relative_report_path_uses_current_directory_parent() {
        assert_eq!(
            normalized_report_parent(std::path::Path::new("probe.json")),
            std::path::Path::new(".")
        );
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn direct_tuner_relative_paths_resolve_from_workspace_root() {
        let workspace_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(std::path::Path::parent)
            .unwrap();
        for relative in [
            "target/teacher-parity/exact-multicore-probe.json",
            "target/teacher-parity/teacher-free-preflight.json",
            ".uor-models/sources/smollm2-135m-instruct",
            ".uor-models/compiled/smollm2-135m-instruct",
        ] {
            assert_eq!(
                resolve_direct_probe_path(std::path::PathBuf::from(relative)),
                workspace_root.join(relative)
            );
        }
        let absolute = std::env::temp_dir().join("absolute-direct-probe-path");
        assert_eq!(resolve_direct_probe_path(absolute.clone()), absolute);
        assert!(resolve_direct_probe_path(std::path::PathBuf::new())
            .as_os_str()
            .is_empty());
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn teacher_free_preflight_fixture(
        label: &str,
    ) -> (
        std::path::PathBuf,
        std::path::PathBuf,
        std::path::PathBuf,
        std::path::PathBuf,
    ) {
        let directory = std::env::temp_dir().join(format!(
            "uor-r4-teacher-free-preflight-{}-{label}",
            std::process::id()
        ));
        if directory.exists() {
            std::fs::remove_dir_all(&directory).unwrap();
        }
        let source = directory.join("source");
        let bundle = directory.join("bundle");
        std::fs::create_dir_all(source.as_path()).unwrap();
        std::fs::create_dir_all(bundle.join("graph")).unwrap();
        let files = [
            ("tokenizer", "tokenizer.bin", b"tokenizer".as_slice()),
            (
                "legacy_artifact",
                "tless_artifacts.bin",
                b"artifact".as_slice(),
            ),
            ("legacy_store", "tless_store.bin", b"store".as_slice()),
            ("graph", "graph/score.r4g1", b"graph".as_slice()),
            (
                "graph_report",
                "graph/score_report.json",
                b"report".as_slice(),
            ),
        ];
        let mut inputs = serde_json::Map::new();
        for (name, relative, bytes) in files {
            if let Some(parent) = bundle.join(relative).parent() {
                std::fs::create_dir_all(parent).unwrap();
            }
            std::fs::write(bundle.join(relative), bytes).unwrap();
            inputs.insert(
                name.to_owned(),
                serde_json::Value::String(format!("blake3:{}", blake3::hash(bytes).to_hex())),
            );
        }
        let mut production_admission = serde_json::Map::new();
        for (name, relative) in PRODUCTION_ADMISSION_COMPONENTS {
            let path = bundle.join(relative);
            if !path.exists() {
                if let Some(parent) = path.parent() {
                    std::fs::create_dir_all(parent).unwrap();
                }
                std::fs::write(&path, format!("production component {name}\n")).unwrap();
            }
            let bytes = std::fs::read(&path).unwrap();
            production_admission.insert(
                name.to_owned(),
                serde_json::Value::String(format!("blake3:{}", blake3::hash(&bytes).to_hex())),
            );
        }
        let report_path = directory.join("preflight.json");
        let report = serde_json::json!({
            "schema": TEACHER_FREE_PREFLIGHT_SCHEMA,
            "authorizing_contract_cid": exact_executor_contract_cid(),
            "status": "AVAILABLE",
            "teacher_source_opened": false,
            "teacher_forwards": 0,
            "report_path": report_path.display().to_string(),
            "selected_source_dir": source.display().to_string(),
            "selected_bundle_dir": bundle.display().to_string(),
            "inputs": inputs,
            "production_admission": production_admission,
        });
        std::fs::write(&report_path, serde_json::to_vec_pretty(&report).unwrap()).unwrap();
        (directory, report_path, source, bundle)
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn digest_consistent_but_semantically_invalid_preflight_never_authorizes_tuner() {
        let (directory, report_path, source, bundle) =
            teacher_free_preflight_fixture("semantic-invalid");
        let result = validate_teacher_free_preflight(&report_path, &source, &bundle);
        assert!(matches!(
            result,
            Err(TeacherFreePreflightAdmissionError::Failed(reason))
                if reason.contains("schema-2 production envelope semantic verification")
        ));
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn planted_stale_compiled_input_refuses_direct_tuner() {
        let (directory, report_path, source, bundle) = teacher_free_preflight_fixture("stale");
        std::fs::write(bundle.join("graph/score.r4g1"), b"changed graph").unwrap();
        let result = validate_teacher_free_preflight(&report_path, &source, &bundle);
        assert!(matches!(
            result,
            Err(TeacherFreePreflightAdmissionError::Refused(reason))
                if reason.contains("graph identity is stale")
        ));
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn planted_stale_production_component_refuses_direct_tuner() {
        let (directory, report_path, source, bundle) =
            teacher_free_preflight_fixture("stale-production");
        std::fs::write(
            bundle.join("graph/deployed_quality_report.json"),
            b"changed deployed quality",
        )
        .unwrap();
        let result = validate_teacher_free_preflight(&report_path, &source, &bundle);
        assert!(matches!(
            result,
            Err(TeacherFreePreflightAdmissionError::Refused(reason))
                if reason.contains("deployed_quality_report is stale")
        ));
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn planted_missing_release_manifest_never_authorizes_direct_tuner() {
        let (directory, report_path, source, bundle) =
            teacher_free_preflight_fixture("missing-release-manifest");
        std::fs::remove_file(bundle.join("release-bundle.json")).unwrap();
        let result = validate_teacher_free_preflight(&report_path, &source, &bundle);
        assert!(matches!(
            result,
            Err(TeacherFreePreflightAdmissionError::Unavailable(reason))
                if reason.contains("release-bundle.json is absent")
        ));
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[cfg(all(not(target_arch = "wasm32"), unix))]
    #[test]
    fn planted_symlinked_production_component_never_authorizes_direct_tuner() {
        use std::os::unix::fs::symlink;

        let (directory, report_path, source, bundle) =
            teacher_free_preflight_fixture("symlinked-production");
        let component = bundle.join("graph/deployed_quality_report.json");
        let target = bundle.join("graph/deployed_quality_report-target.json");
        std::fs::rename(&component, &target).unwrap();
        symlink(&target, &component).unwrap();
        let result = validate_teacher_free_preflight(&report_path, &source, &bundle);
        assert!(matches!(
            result,
            Err(TeacherFreePreflightAdmissionError::Failed(reason))
                if reason.contains("not a regular non-symlink file")
        ));
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn planted_nonavailable_preflight_refuses_direct_tuner() {
        let (directory, report_path, source, bundle) = teacher_free_preflight_fixture("refused");
        let mut report: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&report_path).unwrap()).unwrap();
        report["status"] = serde_json::Value::String("FAILED".to_owned());
        report["reason"] = serde_json::Value::String("planted compiled gate failure".to_owned());
        std::fs::write(&report_path, serde_json::to_vec_pretty(&report).unwrap()).unwrap();
        let result = validate_teacher_free_preflight(&report_path, &source, &bundle);
        assert!(matches!(
            result,
            Err(TeacherFreePreflightAdmissionError::Refused(reason))
                if reason.contains("planted compiled gate failure")
        ));
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn planted_stale_preflight_contract_refuses_direct_tuner() {
        let (directory, report_path, source, bundle) = teacher_free_preflight_fixture("contract");
        let mut report: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&report_path).unwrap()).unwrap();
        report["authorizing_contract_cid"] =
            serde_json::Value::String("blake3:stale-contract".to_owned());
        std::fs::write(&report_path, serde_json::to_vec_pretty(&report).unwrap()).unwrap();
        let result = validate_teacher_free_preflight(&report_path, &source, &bundle);
        assert!(matches!(
            result,
            Err(TeacherFreePreflightAdmissionError::Refused(reason))
                if reason.contains("current authorizing contract")
        ));
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn planted_final_event_failure_cannot_publish_qualified_report() {
        let path = std::env::temp_dir().join(format!(
            "uor-r4-exact-probe-finalization-{}.json",
            std::process::id()
        ));
        let events_path = normalized_report_parent(&path).join(events_file_name_for_report(&path));
        let running = b"{\"status\":\"NOT_QUALIFIED\",\"event\":\"RUNNING\"}\n";
        std::fs::write(&path, running).unwrap();

        let mut report = report();
        let result = report.write_after_durable_final(
            &path,
            &events_path,
            |_report_body_cid, _final_record_number| {
                Err(std::io::Error::other("planted FINAL event failure"))
            },
        );

        assert!(result.is_err());
        assert_eq!(std::fs::read(&path).unwrap(), running);
        std::fs::remove_file(path).unwrap();
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn finalized_fixture(
        label: &str,
    ) -> (
        std::path::PathBuf,
        std::path::PathBuf,
        ExactMulticoreProbeReport,
    ) {
        use std::io::Write;

        let directory = std::env::temp_dir().join(format!(
            "uor-r4-exact-probe-events-{}-{label}",
            std::process::id()
        ));
        std::fs::create_dir_all(&directory).unwrap();
        let report_path = directory.join("probe.json");
        let events_path = directory.join("probe.events.jsonl");
        let _ = std::fs::remove_file(&report_path);
        let _ = std::fs::remove_file(&events_path);
        std::fs::File::create(&events_path).unwrap();

        let mut report = report();
        let status = report.binding_verdict.status;
        let qualifies = report.binding_verdict.qualifies_full_run;
        report
            .write_after_durable_final(
                &report_path,
                &events_path,
                |report_body_cid, final_record_number| {
                    let record = serde_json::json!({
                        "schema": EXACT_MULTICORE_PROBE_SCHEMA,
                        "record": "EXACT_MULTICORE_PROBE",
                        "event": "FINAL",
                        "sequence": final_record_number,
                        "status": status,
                        "qualifies_full_run": qualifies,
                        "report_body_cid": report_body_cid,
                    });
                    let mut file = std::fs::OpenOptions::new()
                        .append(true)
                        .open(&events_path)?;
                    serde_json::to_writer(&mut file, &record).map_err(|error| {
                        std::io::Error::new(std::io::ErrorKind::InvalidData, error)
                    })?;
                    file.write_all(b"\n")?;
                    file.flush()?;
                    file.sync_all()
                },
            )
            .unwrap();
        (report_path, events_path, report)
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn remove_finalized_fixture(report_path: &std::path::Path, events_path: &std::path::Path) {
        if report_path.is_file() {
            std::fs::remove_file(report_path).unwrap();
        }
        if events_path.is_file() {
            std::fs::remove_file(events_path).unwrap();
        }
        std::fs::remove_dir(report_path.parent().unwrap()).unwrap();
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn finalized_events_and_report_validate_together() {
        let (report_path, events_path, report) = finalized_fixture("qualified");
        report
            .validate_for_with_events(&report_path, &expectation())
            .unwrap();
        remove_finalized_fixture(&report_path, &events_path);
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn planted_deleted_events_refuse_admission() {
        let (report_path, events_path, report) = finalized_fixture("deleted");
        std::fs::remove_file(&events_path).unwrap();
        assert_eq!(
            report.validate_for_with_events(&report_path, &expectation()),
            Err(ExactMulticoreProbeValidationError::EventsEvidenceUnavailable)
        );
        remove_finalized_fixture(&report_path, &events_path);
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn planted_truncated_events_refuse_admission() {
        let (report_path, events_path, report) = finalized_fixture("truncated");
        let mut bytes = std::fs::read(&events_path).unwrap();
        assert_eq!(bytes.pop(), Some(b'\n'));
        std::fs::write(&events_path, bytes).unwrap();
        assert_eq!(
            report.validate_for_with_events(&report_path, &expectation()),
            Err(ExactMulticoreProbeValidationError::EventsEvidenceMismatch(
                "events_termination"
            ))
        );
        remove_finalized_fixture(&report_path, &events_path);
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn planted_tampered_events_refuse_admission() {
        let (report_path, events_path, report) = finalized_fixture("tampered");
        let contents = std::fs::read_to_string(&events_path).unwrap();
        let tampered = contents.replacen("\"FINAL\"", "\"FINAX\"", 1);
        assert_ne!(tampered, contents);
        std::fs::write(&events_path, tampered).unwrap();
        assert_eq!(
            report.validate_for_with_events(&report_path, &expectation()),
            Err(ExactMulticoreProbeValidationError::EventsEvidenceMismatch(
                "events_final"
            ))
        );
        remove_finalized_fixture(&report_path, &events_path);
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn macos_host_identity_binds_model_and_topology() {
        let host = exact_probe_host_identity();
        assert!(host
            .cpu_model
            .as_ref()
            .is_some_and(|model| !model.is_empty()));
        assert!(host.physical_core_count.is_some_and(|cores| cores > 0));
        assert!(host.available_parallelism > 0);
    }

    #[test]
    fn planted_equality_negative_refuses_admission() {
        let mut report = report();
        report.exact_equality = false;
        report.runs[1].equal_to_reference = false;
        report.runs[1].output_trace_cid = "blake3:different".to_owned();
        assert_eq!(
            report.validate_for(&expectation()),
            Err(ExactMulticoreProbeValidationError::ExactEqualityNotEstablished)
        );
    }

    #[test]
    fn partial_worker_high_water_is_diagnostic_when_multicore_execution_is_established() {
        let mut report = report();
        report.all_workers_active = false;
        report.runs[0].all_workers_active = false;
        report.runs[0].snapshot.max_active_workers = report.runs[0].workers.saturating_sub(1);
        report.runs[0].snapshot.forward_max_active_workers =
            report.runs[0].snapshot.max_active_workers;
        report
            .validate_for(&expectation())
            .expect("full worker occupancy is diagnostic once exact multicore execution is bound");
    }

    #[test]
    fn planted_nonterminal_worker_counter_refuses_admission() {
        let mut report = report();
        report.runs[0].snapshot.active_workers = 1;
        assert_eq!(
            report.validate_for(&expectation()),
            Err(ExactMulticoreProbeValidationError::BatchedExecutionNotEstablished)
        );
    }

    #[test]
    fn planted_genuine_serial_fallback_refuses_admission() {
        let mut report = report();
        report.runs[0].snapshot.forward_max_active_workers = 1;
        report.runs[0].snapshot.multiworker_forward_calls = 0;
        assert_eq!(
            report.validate_for(&expectation()),
            Err(ExactMulticoreProbeValidationError::BatchedExecutionNotEstablished)
        );
    }

    #[test]
    fn planted_stream_participation_negative_refuses_admission() {
        let mut report = report();
        report.all_streams_active = false;
        report.runs[0].all_streams_active = false;
        report.runs[0].snapshot.max_active_streams = 1;
        assert_eq!(
            report.validate_for(&expectation()),
            Err(ExactMulticoreProbeValidationError::WorkerParticipationNotEstablished)
        );
    }

    #[test]
    fn planted_missing_candidate_refuses_admission() {
        let mut report = report();
        report.runs.retain(|run| run.workers != 4);
        assert_eq!(
            report.validate_for(&expectation()),
            Err(ExactMulticoreProbeValidationError::MissingWorkerConfiguration(4))
        );
    }

    #[test]
    fn planted_wrong_selection_refuses_admission() {
        let mut report = report();
        let slower = &report.runs[1];
        let wrong = ExactMulticoreProbeSelection {
            workers: slower.workers,
            tiles_per_worker: 4,
            aggregate_forwards_per_second: slower.aggregate_forwards_per_second,
            raw_projected_suite_seconds: slower.raw_projected_suite_seconds,
            safety_adjusted_projected_suite_seconds: slower.safety_adjusted_projected_suite_seconds,
        };
        report.selected_best_config = wrong.clone();
        report.configured_execution = wrong;
        assert_eq!(
            report.validate_for(&expectation()),
            Err(ExactMulticoreProbeValidationError::NumericEvidenceMismatch(
                "selected_best_config"
            ))
        );
    }

    #[test]
    fn adaptive_selection_can_choose_four_workers_when_it_is_faster() {
        let mut report = report();
        report.runs[1] = run(4, 50.0);
        report.runs[0].relative_throughput_vs_worker4 = 40.0 / 50.0;
        report.runs[1].relative_throughput_vs_worker4 = 1.0;
        let selected_run = &report.runs[1];
        let raw_projection = selected_run.raw_projected_suite_seconds;
        let adjusted_projection = selected_run.safety_adjusted_projected_suite_seconds;
        let selection = ExactMulticoreProbeSelection {
            workers: 4,
            tiles_per_worker: 4,
            aggregate_forwards_per_second: selected_run.aggregate_forwards_per_second,
            raw_projected_suite_seconds: raw_projection,
            safety_adjusted_projected_suite_seconds: adjusted_projection,
        };
        report.selected_best_config = selection.clone();
        report.configured_execution = selection;
        report.raw_projected_suite_seconds = raw_projection;
        report.safety_adjusted_projected_suite_seconds = adjusted_projection;
        report.validate_for(&expectation()).unwrap();
    }

    #[test]
    fn planted_projection_negative_refuses_admission() {
        let mut report = report();
        let exact_boundary = 3;
        report.binding_verdict.configured_max_wall_seconds = exact_boundary;
        report.binding_verdict.qualification_wall_seconds = exact_boundary;
        let mut expected = expectation();
        expected.configured_max_wall_seconds = exact_boundary;
        assert_eq!(
            report.validate_for(&expected),
            Err(ExactMulticoreProbeValidationError::ProjectionExceedsLimit)
        );
    }

    #[test]
    fn planted_context_horizon_tamper_refuses_admission() {
        let mut report = report();
        report.probe_context_ceiling_tokens = 36;
        report.probe_position_indices.fill(35);
        assert_eq!(
            report.validate_for(&expectation()),
            Err(ExactMulticoreProbeValidationError::BudgetMismatch(
                "probe_context_shape"
            ))
        );
    }

    #[test]
    fn planted_source_and_budget_negatives_refuse_admission() {
        let mut wrong_source = report();
        wrong_source.source.config_cid = "blake3:wrong".to_owned();
        assert_eq!(
            wrong_source.validate_for(&expectation()),
            Err(ExactMulticoreProbeValidationError::SourceIdentityMismatch)
        );

        let mut wrong_budget = report();
        wrong_budget
            .configured_suite_work
            .generation_tokens_per_lane += 1;
        assert_eq!(
            wrong_budget.validate_for(&expectation()),
            Err(ExactMulticoreProbeValidationError::BudgetMismatch(
                "configured_suite_work"
            ))
        );
    }

    #[test]
    fn planted_reduced_but_consistent_work_cannot_authorize_the_full_suite() {
        let mut reduced = work();
        reduced.transcript_logical_forwards -= 1;
        reduced.logical_forwards = reduced.derived_logical_forwards();
        reduced.transcript_physical_batches = reduced.derived_transcript_physical_batches();
        reduced.physical_batches = reduced.derived_physical_batches();
        assert!(!reduced.is_registered_binding_work());

        let mut report = report();
        report.configured_suite_work = reduced.clone();
        let mut expected = expectation();
        expected.configured_suite_work = reduced;
        assert_eq!(
            report.validate_for(&expected),
            Err(ExactMulticoreProbeValidationError::BudgetMismatch(
                "optimized_suite_work"
            ))
        );
    }

    #[test]
    fn planted_stale_executor_build_refuses_admission() {
        let mut stale = report();
        stale.executor_contract_cid = "blake3:stale-build".to_owned();
        assert_eq!(
            stale.validate_for(&expectation()),
            Err(ExactMulticoreProbeValidationError::ExecutorContractMismatch)
        );
    }

    #[test]
    fn planted_backend_inventory_tamper_refuses_admission() {
        let mut stale = report();
        stale
            .backend
            .available_backends
            .push("fabricated-backend".to_owned());
        assert_eq!(
            stale.validate_for(&expectation()),
            Err(ExactMulticoreProbeValidationError::BackendMismatch)
        );
    }

    #[test]
    fn optimized_work_omits_duplicate_prefill_and_warmup() {
        let bounded = work();
        assert_eq!(bounded.logical_forwards, 36 + 8 * 8);
        assert_eq!(bounded.transcript_physical_batches, 6);
        assert_eq!(bounded.generation_physical_batches, 8);
        assert_eq!(bounded.physical_batches, 14);
        assert_eq!(bounded.derived_probe_context_ceiling_tokens(), 14);
        assert!(bounded.is_registered_binding_work());
    }

    #[test]
    fn planted_redundant_arithmetic_tamper_refuses_admission() {
        let mut report = report();
        report.runs[0].relative_throughput_vs_worker4 = 4.01;
        assert_eq!(
            report.validate_for(&expectation()),
            Err(ExactMulticoreProbeValidationError::NumericEvidenceMismatch(
                "run_arithmetic"
            ))
        );
    }

    #[test]
    fn resource_unavailability_is_truthful_diagnostic_evidence() {
        let mut report = report();
        report.runs[0].resources = ExactMulticoreProbeResources::unavailable("planted gap");
        report.validate_for(&expectation()).unwrap();
    }

    #[test]
    fn planted_serial_fallback_counter_refuses_admission() {
        let mut report = report();
        report.runs[0].snapshot.batched_matrix_calls = 0;
        assert_eq!(
            report.validate_for(&expectation()),
            Err(ExactMulticoreProbeValidationError::BatchedExecutionNotEstablished)
        );
    }

    #[test]
    fn planted_truncated_trace_refuses_admission() {
        let mut truncated_lane = report();
        truncated_lane.runs[0].trace_shape.streams_per_position = 7;
        assert_eq!(
            truncated_lane.validate_for(&expectation()),
            Err(ExactMulticoreProbeValidationError::NumericEvidenceMismatch(
                "trace_or_plan_shape"
            ))
        );

        let mut truncated_position = report();
        truncated_position.runs[0].trace_shape.positions = 0;
        assert_eq!(
            truncated_position.validate_for(&expectation()),
            Err(ExactMulticoreProbeValidationError::NumericEvidenceMismatch(
                "trace_or_plan_shape"
            ))
        );
    }

    #[test]
    fn planted_generation_lane_mismatch_refuses_admission() {
        let mut report = report();
        report.configured_suite_work.generation_lanes = 7;
        report.configured_suite_work.logical_forwards =
            report.configured_suite_work.derived_logical_forwards();
        let mut expected = expectation();
        expected.configured_suite_work = report.configured_suite_work.clone();
        assert_eq!(
            report.validate_for(&expected),
            Err(ExactMulticoreProbeValidationError::BudgetMismatch(
                "optimized_suite_work"
            ))
        );
    }

    #[test]
    fn planted_prestart_tamper_refuses_admission() {
        let mut backend_tamper = report();
        backend_tamper.runs[0].prestart.backend_exercised = false;
        assert_eq!(
            backend_tamper.validate_for(&expectation()),
            Err(ExactMulticoreProbeValidationError::NumericEvidenceMismatch(
                "run_rate"
            ))
        );

        let mut missing_workspace = report();
        missing_workspace.runs[0].prestart.workspace_capacity_bytes = 0;
        assert_eq!(
            missing_workspace.validate_for(&expectation()),
            Err(ExactMulticoreProbeValidationError::NumericEvidenceMismatch(
                "run_rate"
            ))
        );

        let mut timed_growth = report();
        timed_growth.runs[0].snapshot.workspace_growth_events = 1;
        timed_growth.runs[0].snapshot.workspace_growth_bytes = 4;
        assert_eq!(
            timed_growth.validate_for(&expectation()),
            Err(ExactMulticoreProbeValidationError::BatchedExecutionNotEstablished)
        );
    }

    #[test]
    fn planted_host_below_four_workers_refuses_admission() {
        let mut report = report();
        report.host.available_parallelism = 3;
        let mut expected = expectation();
        expected.host = report.host.clone();
        assert_eq!(
            report.validate_for(&expected),
            Err(ExactMulticoreProbeValidationError::BudgetMismatch(
                "minimum_adaptive_host"
            ))
        );
    }

    #[test]
    fn planted_probe_wall_overrun_refuses_admission() {
        let mut report = report();
        report.probe_elapsed_seconds = 3_601.0;
        assert_eq!(
            report.validate_for(&expectation()),
            Err(ExactMulticoreProbeValidationError::ProbeWallCeilingExceeded)
        );
    }

    #[test]
    fn planted_deadline_policy_tamper_refuses_admission() {
        let mut report = report();
        report.probe_deadline_policy = "HARD_KILL_IN_FLIGHT_FORWARD".to_owned();
        assert_eq!(
            report.validate_for(&expectation()),
            Err(ExactMulticoreProbeValidationError::BudgetMismatch(
                "probe_deadline_policy"
            ))
        );
    }
}
