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
//! `model_id`, `capability`, `uor_matmul`, and `provenance_note` remain
//! caller-supplied policy. Production packaging first calls
//! [`verify_bundle_for_production_packaging`], which reads the persisted
//! `tokenizer_adapter.json` and independently reproduces every deployed-
//! quality binding. The CLI additionally resolves an explicitly selected
//! source tokenizer and requires it to equal that persisted adapter.
//!
//! #655-D2 (`src/main.rs`'s `r4 package-release-bundle` command) is the
//! caller: it resolves a real `tokenizer_adapter` from an explicit
//! `--source` HF snapshot for `InstructionChat` bundles (using
//! `uor_r4_core::transformerless::hf_bpe::resolve_source_tokenizer`,
//! outside this module's own responsibility), builds [`PackageInputs`],
//! and writes this function's returned manifest to
//! `release_bundle_loader::RELEASE_BUNDLE_SIDECAR_FILE_NAME` next to
//! `physical_root`.
//!
//! #655-D3 (this module's own test suite, see
//! `packaged_bundle_is_accepted_by_the_loaders_sidecar_verifier`) closes
//! the loop: a golden test that packages a bundle, writes the sidecar
//! exactly as #655-D2's CLI command does, and asserts
//! `release_bundle_loader::verify_release_bundle_sidecar` accepts it --
//! proving the two independently-tested halves actually compose.

use std::path::{Path, PathBuf};

use uor_r4_api::{
    derive_deployed_quality_bindings, parse_and_validate_normative_witness_replay,
    validate_production_evidence_links, AbiVersion, BundleAbi, BundleCapability,
    BundleComponentDigests, CompilerIdentity, CrossSurfaceParityEvidence,
    DeployedQualityBindingMaterial, DeployedQualityBindings, DeployedQualityReport,
    NormativeWitnessReplayMaterial, NormativeWitnessReplaySpec, ProductionEvidenceParts,
    ReleaseBundleManifest, SelectorIdentity, TokenizerAdapter, UorMatmulProvenance,
    DEFAULT_NORMATIVE_WITNESS_SAMPLE, RELEASE_BUNDLE_MANIFEST_SCHEMA,
};
use uor_r4_core::transformerless::compiler;

/// Standing #655 `uor-matmul` pin (`serving_655.md` project memory,
/// mirroring `docs/matrix_operation_census.md`). Bump only via the
/// project's κ/artifact-era re-pin process.
pub const UOR_MATMUL_REVISION: &str = "b13c98449948174f590e337c4dc25dfc394a07d0";

/// Relative paths within a resolved R4G1 bundle's `physical_root`, per
/// `docs/serving_release_packaging_655_d.md`'s field-to-file mapping.
const GRAPH_RELATIVE_PATH: &str = "graph/score.r4g1";
pub const SECTIONS_ABSENT_GRAPH_RELATIVE_PATH: &str = "graph/score_sections_absent.r4g1";
pub const LABEL_SHUFFLED_GRAPH_RELATIVE_PATH: &str = "graph/score_label_shuffled.r4g1";
const SIGNATURE_ARTIFACT_RELATIVE_PATH: &str = "tless_artifacts.bin";
pub const TLA_COMPARATOR_STORE_RELATIVE_PATH: &str = "tless_store.bin";
const TOKENIZER_RELATIVE_PATH: &str = "tokenizer.bin";
const SCORE_REPORT_RELATIVE_PATH: &str = "graph/score_report.json";
pub const DEPLOYED_QUALITY_REPORT_RELATIVE_PATH: &str = "graph/deployed_quality_report.json";
pub const CROSS_SURFACE_PARITY_RELATIVE_PATH: &str = "graph/cross_surface_parity.json";
pub const WITNESS_REPLAY_RELATIVE_PATH: &str = "graph/witness_replay.json";
pub const CORPUS_META_RELATIVE_PATH: &str = "corpus.meta";
pub const CORPUS_RECORDS_RELATIVE_PATH: &str = "corpus.records";
pub const TOKENIZER_ADAPTER_RELATIVE_PATH: &str = "tokenizer_adapter.json";
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
    /// Parsed from the required deployed-quality report by the CLI caller.
    pub selector: SelectorIdentity,
    /// Compiler source/configuration identity bound independently by schema 2.
    pub compiler: CompilerIdentity,
    pub provenance_note: Option<String>,
}

/// Independently reproduced admission identities for one exact bundle
/// generation. The package command obtains these from the component bytes,
/// never by copying the deployed-quality report's claimed bindings.
#[derive(Debug)]
pub struct VerifiedPackagingAdmission {
    pub bindings: DeployedQualityBindings,
    pub tokenizer_adapter: TokenizerAdapter,
}

/// Why [`package_release_bundle`] could not build a manifest.
#[derive(Debug)]
pub enum PackageBundleError {
    /// A required component/evidence file could not be read (missing,
    /// permission denied, not a regular file, etc). The structural manifest
    /// helper still permits an absent tokenizer; strict production packaging
    /// requires tokenizer, corpus, adapter, and deployed-quality bytes.
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

/// Reproduce and validate the complete production envelope before a release
/// manifest is written. The compiler revision is external release authority;
/// every other identity comes from the actual graph, artifact, corpus,
/// tokenizer, adapter, score, cover, and held-out-position bytes under
/// `physical_root`.
pub fn verify_bundle_for_production_packaging(
    physical_root: &Path,
    compiler_revision: &str,
) -> Result<VerifiedPackagingAdmission, PackageBundleError> {
    let graph = read_required(physical_root, GRAPH_RELATIVE_PATH)?;
    let sections_absent_graph = read_required(physical_root, SECTIONS_ABSENT_GRAPH_RELATIVE_PATH)?;
    let label_shuffled_graph = read_required(physical_root, LABEL_SHUFFLED_GRAPH_RELATIVE_PATH)?;
    let signature_artifact = read_required(physical_root, SIGNATURE_ARTIFACT_RELATIVE_PATH)?;
    let tla_comparator_store = read_required(physical_root, TLA_COMPARATOR_STORE_RELATIVE_PATH)?;
    let tokenizer = read_required(physical_root, TOKENIZER_RELATIVE_PATH)?;
    let score_report = read_required(physical_root, SCORE_REPORT_RELATIVE_PATH)?;
    let compile_report = read_required(physical_root, COMPILE_REPORT_RELATIVE_PATH)?;
    let deployed_quality_report =
        read_required(physical_root, DEPLOYED_QUALITY_REPORT_RELATIVE_PATH)?;
    let cross_surface_parity = read_required(physical_root, CROSS_SURFACE_PARITY_RELATIVE_PATH)?;
    let witness_replay = read_required(physical_root, WITNESS_REPLAY_RELATIVE_PATH)?;
    let corpus_meta = read_required(physical_root, CORPUS_META_RELATIVE_PATH)?;
    let corpus_records = read_required(physical_root, CORPUS_RECORDS_RELATIVE_PATH)?;
    let tokenizer_adapter_bytes = read_required(physical_root, TOKENIZER_ADAPTER_RELATIVE_PATH)?;

    let corpus =
        compiler::load_corpus_bytes(&corpus_meta, &corpus_records, None).ok_or_else(|| {
            PackageBundleError::Invalid(
                "corpus.meta/corpus.records do not parse as one completed corpus".to_owned(),
            )
        })?;
    let (_, certification_positions) = compiler::split_positions(&corpus);
    let certification_positions: Vec<u64> = certification_positions
        .into_iter()
        .map(|position| {
            u64::try_from(position).map_err(|_| {
                PackageBundleError::Invalid(
                    "certification corpus position does not fit the u64 wire identity".to_owned(),
                )
            })
        })
        .collect::<Result<_, _>>()?;
    let bindings = derive_deployed_quality_bindings(DeployedQualityBindingMaterial {
        graph: &graph,
        teacher_artifact: &signature_artifact,
        corpus_meta: &corpus_meta,
        corpus_records: &corpus_records,
        tokenizer: &tokenizer,
        tokenizer_adapter: &tokenizer_adapter_bytes,
        score_report: &score_report,
        compile_report: &compile_report,
        compiler_revision,
        full_population_positions: &certification_positions,
        evaluated_positions: &certification_positions,
    })
    .map_err(|error| PackageBundleError::Invalid(error.to_string()))?;
    let report: DeployedQualityReport =
        serde_json::from_slice(&deployed_quality_report).map_err(|error| {
            PackageBundleError::Invalid(format!("invalid deployed-quality report: {error}"))
        })?;
    if let Some(error) = report.validate_for_production(&bindings) {
        return Err(PackageBundleError::Invalid(format!(
            "deployed-quality report does not match independently reproduced bundle identities: {error}"
        )));
    }
    CrossSurfaceParityEvidence::parse_and_validate_for_production_bundle(
        &cross_surface_parity,
        &graph,
        &signature_artifact,
        &tokenizer,
        &score_report,
    )
    .map_err(|error| {
        PackageBundleError::Invalid(format!("invalid cross-surface parity evidence: {error}"))
    })?;
    parse_and_validate_normative_witness_replay(
        &witness_replay,
        NormativeWitnessReplaySpec {
            material: NormativeWitnessReplayMaterial {
                graph: &graph,
                signature_artifact: &signature_artifact,
                tokenizer: &tokenizer,
                score_report: Some(&score_report),
                corpus_meta: &corpus_meta,
                corpus_records: &corpus_records,
            },
            evaluated_positions: &certification_positions,
            sample_size: DEFAULT_NORMATIVE_WITNESS_SAMPLE,
        },
    )
    .map_err(|error| {
        PackageBundleError::Invalid(format!("invalid normative witness replay: {error}"))
    })?;
    validate_production_evidence_links(
        ProductionEvidenceParts {
            graph: &graph,
            sections_absent_graph: &sections_absent_graph,
            label_shuffled_graph: &label_shuffled_graph,
            signature_artifact: &signature_artifact,
            tla_comparator_store: &tla_comparator_store,
            tokenizer: &tokenizer,
            score_report: &score_report,
            deployed_quality_report: &deployed_quality_report,
            cross_surface_parity: &cross_surface_parity,
            witness_replay: &witness_replay,
            corpus_meta: &corpus_meta,
            corpus_records: &corpus_records,
        },
        &certification_positions,
    )
    .map_err(|error| PackageBundleError::Invalid(error.to_string()))?;
    let tokenizer_adapter: TokenizerAdapter = serde_json::from_slice(&tokenizer_adapter_bytes)
        .map_err(|error| {
            PackageBundleError::Invalid(format!("invalid tokenizer_adapter.json: {error}"))
        })?;

    Ok(VerifiedPackagingAdmission {
        bindings,
        tokenizer_adapter,
    })
}

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
    let sections_absent_graph = read_required(physical_root, SECTIONS_ABSENT_GRAPH_RELATIVE_PATH)?;
    let label_shuffled_graph = read_required(physical_root, LABEL_SHUFFLED_GRAPH_RELATIVE_PATH)?;
    let signature_artifact = read_required(physical_root, SIGNATURE_ARTIFACT_RELATIVE_PATH)?;
    let tla_comparator_store = read_required(physical_root, TLA_COMPARATOR_STORE_RELATIVE_PATH)?;
    let score_report = read_required(physical_root, SCORE_REPORT_RELATIVE_PATH)?;
    let deployed_quality_report =
        read_required(physical_root, DEPLOYED_QUALITY_REPORT_RELATIVE_PATH)?;
    let cross_surface_parity = read_required(physical_root, CROSS_SURFACE_PARITY_RELATIVE_PATH)?;
    let witness_replay = read_required(physical_root, WITNESS_REPLAY_RELATIVE_PATH)?;
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
            sections_absent_graph: Some(digest(&sections_absent_graph)),
            label_shuffled_graph: Some(digest(&label_shuffled_graph)),
            signature_artifact: digest(&signature_artifact),
            tla_comparator_store: Some(digest(&tla_comparator_store)),
            tokenizer: tokenizer.as_deref().map(digest),
            score_report: digest(&score_report),
            compile_report: digest(&compile_report),
            deployed_quality_report: Some(digest(&deployed_quality_report)),
            cross_surface_parity: Some(digest(&cross_surface_parity)),
            witness_replay: Some(digest(&witness_replay)),
        },
        selector: Some(inputs.selector),
        compiler: Some(inputs.compiler),
        tokenizer_adapter: inputs.tokenizer_adapter,
        provenance_note: inputs.provenance_note,
    };

    match manifest.validate() {
        Some(reason) => Err(PackageBundleError::Invalid(reason)),
        None => Ok(manifest),
    }
}

/// Production packaging boundary. Unlike the historical manifest-only
/// helper, this requires every caller-supplied admission identity to equal the
/// independently reproduced bundle/report envelope before returning bytes
/// that may be published.
pub fn package_verified_release_bundle(
    physical_root: &Path,
    inputs: PackageInputs,
    compiler_revision: &str,
) -> Result<ReleaseBundleManifest, PackageBundleError> {
    let admission = verify_bundle_for_production_packaging(physical_root, compiler_revision)?;
    if inputs.tokenizer_adapter != admission.tokenizer_adapter {
        return Err(PackageBundleError::Invalid(
            "manifest tokenizer adapter does not equal tokenizer_adapter.json".to_owned(),
        ));
    }
    if inputs.selector != admission.bindings.selector {
        return Err(PackageBundleError::Invalid(
            "manifest selector does not equal independently reproduced selector".to_owned(),
        ));
    }
    if inputs.compiler != admission.bindings.compiler {
        return Err(PackageBundleError::Invalid(
            "manifest compiler does not equal independently reproduced compiler identity"
                .to_owned(),
        ));
    }
    let manifest = package_release_bundle(physical_root, inputs)?;
    let final_admission = verify_bundle_for_production_packaging(physical_root, compiler_revision)?;
    if final_admission.bindings != admission.bindings
        || final_admission.tokenizer_adapter != admission.tokenizer_adapter
    {
        return Err(PackageBundleError::Invalid(
            "bundle generation changed while production packaging was in progress".to_owned(),
        ));
    }
    let current_graph = read_required(physical_root, GRAPH_RELATIVE_PATH)?;
    let current_sections_absent =
        read_required(physical_root, SECTIONS_ABSENT_GRAPH_RELATIVE_PATH)?;
    let current_label_shuffled = read_required(physical_root, LABEL_SHUFFLED_GRAPH_RELATIVE_PATH)?;
    let current_teacher = read_required(physical_root, SIGNATURE_ARTIFACT_RELATIVE_PATH)?;
    let current_tla_store = read_required(physical_root, TLA_COMPARATOR_STORE_RELATIVE_PATH)?;
    let current_score = read_required(physical_root, SCORE_REPORT_RELATIVE_PATH)?;
    let current_compile = read_required(physical_root, COMPILE_REPORT_RELATIVE_PATH)?;
    let current_quality = read_required(physical_root, DEPLOYED_QUALITY_REPORT_RELATIVE_PATH)?;
    let current_cross = read_required(physical_root, CROSS_SURFACE_PARITY_RELATIVE_PATH)?;
    let current_witness = read_required(physical_root, WITNESS_REPLAY_RELATIVE_PATH)?;
    let current_tokenizer = read_required(physical_root, TOKENIZER_RELATIVE_PATH)?;
    if manifest.components.graph != digest(&current_graph)
        || manifest.components.sections_absent_graph.as_deref()
            != Some(digest(&current_sections_absent).as_str())
        || manifest.components.label_shuffled_graph.as_deref()
            != Some(digest(&current_label_shuffled).as_str())
        || manifest.components.signature_artifact != digest(&current_teacher)
        || manifest.components.tla_comparator_store.as_deref()
            != Some(digest(&current_tla_store).as_str())
        || manifest.components.score_report != digest(&current_score)
        || manifest.components.compile_report != digest(&current_compile)
        || manifest.components.deployed_quality_report.as_deref()
            != Some(digest(&current_quality).as_str())
        || manifest.components.cross_surface_parity.as_deref()
            != Some(digest(&current_cross).as_str())
        || manifest.components.witness_replay.as_deref() != Some(digest(&current_witness).as_str())
        || manifest.components.tokenizer.as_deref() != Some(digest(&current_tokenizer).as_str())
        || manifest.selector.as_ref() != Some(&final_admission.bindings.selector)
        || manifest.compiler.as_ref() != Some(&final_admission.bindings.compiler)
        || manifest.tokenizer_adapter != final_admission.tokenizer_adapter
    {
        return Err(PackageBundleError::Invalid(
            "constructed release manifest diverges from the verified production envelope"
                .to_owned(),
        ));
    }
    Ok(manifest)
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
pub(crate) mod tests {
    use super::*;
    use std::collections::BTreeMap;

    use uor_r4_api::{
        produce_normative_witness_replay, ComparatorIdentity, CrossSurfaceDisposition,
        CrossSurfaceParityEvidenceBuilder, CrossSurfaceParityObservation, EngineParts,
        EvaluationEvidence, EvaluationMode, ExactRate, ExactSignedRate, NegativeControlEvidence,
        NegativeControlVerdict, NormativeServingDecision, NormativeServingEngine,
        NormativeWitnessReplayMaterial, NormativeWitnessReplaySpec, PairedComparison, PairedCounts,
        PairedInterval, QualityMeasurements, QualityProfileIdentity, QualityVerdict,
        WitnessReplayEvidence, DEPLOYED_QUALITY_PROFILE_ID, DEPLOYED_QUALITY_PROFILE_VERSION,
        DEPLOYED_QUALITY_REPORT_SCHEMA, LABEL_SHUFFLED_CONTROL_ID, NORMATIVE_EXECUTION_SCOPE,
        SECTIONS_ABSENT_COMPARATOR_ID, SECTIONS_ABSENT_COMPARATOR_VERSION, TLA_COMPARATOR_ID,
        TLA_COMPARATOR_VERSION,
    };
    use uor_r4_core::transformerless::{convert_r4g1, runtime};
    use uor_r4_graph_format::{ArtifactBuilder, GraphView, SectionId};

    const VALID_REV: &str = UOR_MATMUL_REVISION;
    const PLACEHOLDER_DIGEST: &str =
        "blake3:0000000000000000000000000000000000000000000000000000000000000001";
    pub(crate) const PRODUCTION_COMPILER_REVISION: &str =
        "0123456789abcdef0123456789abcdef01234567";

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
        write(
            dir,
            SECTIONS_ABSENT_GRAPH_RELATIVE_PATH,
            b"sections absent graph bytes",
        );
        write(
            dir,
            LABEL_SHUFFLED_GRAPH_RELATIVE_PATH,
            b"label shuffled graph bytes",
        );
        write(dir, SIGNATURE_ARTIFACT_RELATIVE_PATH, b"signature bytes");
        write(dir, TLA_COMPARATOR_STORE_RELATIVE_PATH, b"store bytes");
        write(dir, TOKENIZER_RELATIVE_PATH, b"tokenizer bytes");
        write(dir, SCORE_REPORT_RELATIVE_PATH, b"{\"score\":true}");
        write(
            dir,
            DEPLOYED_QUALITY_REPORT_RELATIVE_PATH,
            b"{\"schema\":1}",
        );
        write(
            dir,
            CROSS_SURFACE_PARITY_RELATIVE_PATH,
            b"{\"cross_surface\":true}",
        );
        write(dir, WITNESS_REPLAY_RELATIVE_PATH, b"{\"witness\":true}");
        write(dir, COMPILE_REPORT_RELATIVE_PATH, b"{\"cover\":true}");
    }

    fn completed_corpus() -> (Vec<u8>, Vec<u8>) {
        const HELD_OUT: usize = 1_000;
        let n = HELD_OUT + 1;
        let mut meta = Vec::with_capacity(25);
        meta.extend_from_slice(&(n as u64).to_le_bytes());
        meta.extend_from_slice(&2u64.to_le_bytes());
        meta.extend_from_slice(&7u64.to_le_bytes());
        meta.push(1);
        let mut records = vec![0u8; n * 48];
        for index in 0..n {
            let offset = index * 48;
            let story = u32::from(index != 0);
            records[offset..offset + 4].copy_from_slice(&story.to_le_bytes());
            records[offset + 4..offset + 8].copy_from_slice(&1u32.to_le_bytes());
            records[offset + 8..offset + 12].copy_from_slice(&1u32.to_le_bytes());
            records[offset + 12..offset + 16].copy_from_slice(&2u32.to_le_bytes());
            records[offset + 16..offset + 20].copy_from_slice(&3u32.to_le_bytes());
            records[offset + 20..offset + 24].copy_from_slice(&100u32.to_le_bytes());
            let span = index.saturating_sub(1) as u32;
            records[offset + 32..offset + 36].copy_from_slice(&span.to_le_bytes());
            records[offset + 36..offset + 40]
                .copy_from_slice(&span.saturating_add(1).to_le_bytes());
            records[offset + 40..offset + 44].copy_from_slice(&u32::MAX.to_le_bytes());
            records[offset + 44..offset + 48].copy_from_slice(&u32::MAX.to_le_bytes());
        }
        (meta, records)
    }

    fn bind_graph_head_and_lanes(
        base: &[u8],
        teacher: &[u8],
        tokenizer: &[u8],
        corpus_meta: &[u8],
        corpus_records: &[u8],
    ) -> Vec<u8> {
        let view = GraphView::parse(base).expect("base graph parses");
        let mut builder = ArtifactBuilder::new(view.header().alignment_log2);
        for section in view.sections() {
            assert!(section.id != SectionId::SKMX && section.id != SectionId::PSIB);
            if section.id == SectionId::HEAD {
                let mut head = section.payload.to_vec();
                head[0..32].copy_from_slice(blake3::hash(teacher).as_bytes());
                head[32..64].copy_from_slice(blake3::hash(tokenizer).as_bytes());
                let corpus = compiler::load_corpus_bytes(corpus_meta, corpus_records, None)
                    .expect("fixture corpus parses");
                let (construction, certification) = compiler::split_positions(&corpus);
                let construction: Vec<u64> = construction
                    .into_iter()
                    .map(|position| position as u64)
                    .collect();
                let certification: Vec<u64> = certification
                    .into_iter()
                    .map(|position| position as u64)
                    .collect();
                let construction_cid = uor_r4_graph_format::corpus_partition_cid(
                    corpus_meta,
                    corpus_records,
                    uor_r4_graph_format::CorpusPartitionRole::Construction,
                    &construction,
                );
                let certification_cid = uor_r4_graph_format::corpus_partition_cid(
                    corpus_meta,
                    corpus_records,
                    uor_r4_graph_format::CorpusPartitionRole::Certification,
                    &certification,
                );
                head[64..96].copy_from_slice(&construction_cid.0);
                head[96..128].copy_from_slice(&certification_cid.0);
                head[148..180]
                    .copy_from_slice(blake3::hash(b"fixture-compiler-version").as_bytes());
                builder.add_section(section.id, section.flags, &head);
            } else {
                builder.add_section(section.id, section.flags, section.payload);
            }
        }
        let skmx =
            uor_r4_graph_format::build_skipmix_table(&[(5u32, 5u32, vec![(42u32, 1_000i32)])])
                .expect("fixture SKMX");
        let psib = uor_r4_graph_format::build_psi_bag_table(&[(5u32, vec![(43u32, 500i32)])])
            .expect("fixture PSIB");
        builder.add_section(SectionId::SKMX, 0, &skmx);
        builder.add_section(SectionId::PSIB, 0, &psib);
        builder.build().expect("production fixture graph")
    }

    fn graph_without_lanes(graph: &[u8]) -> Vec<u8> {
        let view = GraphView::parse(graph).expect("fixture graph parses");
        let mut builder = ArtifactBuilder::new(view.header().alignment_log2);
        for section in view.sections() {
            if section.id != SectionId::SKMX && section.id != SectionId::PSIB {
                builder.add_section(section.id, section.flags, section.payload);
            }
        }
        builder.build().expect("sections-absent graph")
    }

    fn graph_with_shuffled_lane(graph: &[u8]) -> Vec<u8> {
        let view = GraphView::parse(graph).expect("fixture graph parses");
        let mut builder = ArtifactBuilder::new(view.header().alignment_log2);
        for section in view.sections() {
            if section.id == SectionId::SKMX {
                let shuffled = uor_r4_graph_format::build_skipmix_table(&[(
                    5u32,
                    5u32,
                    vec![(44u32, 1_000i32)],
                )])
                .expect("shuffled fixture SKMX");
                builder.add_section(section.id, section.flags, &shuffled);
            } else {
                builder.add_section(section.id, section.flags, section.payload);
            }
        }
        builder.build().expect("label-shuffled graph")
    }

    fn report_tagged_cid(tag: &[u8], parts: &[&[u8]]) -> String {
        let mut hasher = blake3::Hasher::new();
        hasher.update(&(tag.len() as u64).to_le_bytes());
        hasher.update(tag);
        for part in parts {
            hasher.update(&(part.len() as u64).to_le_bytes());
            hasher.update(part);
        }
        format!("blake3:{}", hasher.finalize().to_hex())
    }

    fn disposition_and_token(
        decision: NormativeServingDecision,
    ) -> (CrossSurfaceDisposition, Option<u32>) {
        match decision {
            NormativeServingDecision::Serve(outcome) => {
                (CrossSurfaceDisposition::Serve, Some(outcome.token))
            }
            NormativeServingDecision::Abstain(_) => (CrossSurfaceDisposition::Abstain, None),
            NormativeServingDecision::Decline(_) => (CrossSurfaceDisposition::Decline, None),
        }
    }

    fn sampled_disposition_and_token(
        decision: NormativeServingDecision,
        seed: u32,
    ) -> (CrossSurfaceDisposition, Option<u32>) {
        match decision {
            NormativeServingDecision::Serve(outcome) => {
                let mut rng = runtime::SampleRng::new(seed);
                (
                    CrossSurfaceDisposition::Serve,
                    Some(outcome.select_sampled_token(&[], &mut rng)),
                )
            }
            NormativeServingDecision::Abstain(_) => (CrossSurfaceDisposition::Abstain, None),
            NormativeServingDecision::Decline(_) => (CrossSurfaceDisposition::Decline, None),
        }
    }

    fn rate(numerator: u64, denominator: u64) -> ExactRate {
        ExactRate {
            numerator,
            denominator,
            ppm: ((u128::from(numerator) * 1_000_000) / u128::from(denominator)) as u32,
        }
    }

    fn signed_rate(numerator: i64, denominator: u64) -> ExactSignedRate {
        ExactSignedRate {
            numerator,
            denominator,
            ppm: ((i128::from(numerator) * 1_000_000) / i128::from(denominator)) as i64,
        }
    }

    fn comparison(
        comparator: ComparatorIdentity,
        both_correct: u64,
        selector_only_correct: u64,
        comparator_only_correct: u64,
        neither_correct: u64,
    ) -> PairedComparison {
        let counts = PairedCounts {
            both_correct,
            selector_only_correct,
            comparator_only_correct,
            neither_correct,
        };
        let denominator =
            both_correct + selector_only_correct + comparator_only_correct + neither_correct;
        PairedComparison {
            comparator,
            counts,
            selector_rate: rate(both_correct + selector_only_correct, denominator),
            comparator_rate: rate(both_correct + comparator_only_correct, denominator),
            delta: signed_rate(
                selector_only_correct as i64 - comparator_only_correct as i64,
                denominator,
            ),
            interval: PairedInterval::from_counts(counts).expect("fixture paired interval"),
        }
    }

    fn production_report(
        bindings: DeployedQualityBindings,
        tla_comparator_store: &[u8],
        sections_absent_graph: &[u8],
        label_shuffled_graph: &[u8],
        cross_surface_parity: &[u8],
        cross_surface: &CrossSurfaceParityEvidence,
        witness: &uor_r4_api::NormativeWitnessReplayArtifact,
    ) -> DeployedQualityReport {
        let positions_cid = bindings.partition.evaluated_positions_cid.clone();
        let tla_comparator = ComparatorIdentity {
            id: TLA_COMPARATOR_ID.to_owned(),
            version: TLA_COMPARATOR_VERSION.to_owned(),
            definition_cid: report_tagged_cid(
                b"r4-deployed-quality-tla-comparator/1",
                &[TLA_COMPARATOR_VERSION.as_bytes(), tla_comparator_store],
            ),
            positions_cid: positions_cid.clone(),
        };
        let absent_definition_cid = report_tagged_cid(
            b"r4-deployed-quality-sections-absent-comparator/1",
            &[
                SECTIONS_ABSENT_COMPARATOR_VERSION.as_bytes(),
                sections_absent_graph,
            ],
        );
        let sections_absent_comparator = ComparatorIdentity {
            id: SECTIONS_ABSENT_COMPARATOR_ID.to_owned(),
            version: SECTIONS_ABSENT_COMPARATOR_VERSION.to_owned(),
            definition_cid: absent_definition_cid,
            positions_cid: positions_cid.clone(),
        };
        let label_identity_cid = report_tagged_cid(
            b"r4-deployed-quality-label-shuffled-control/1",
            &[
                b"train-target-rotation-half-plus-one/1",
                sections_absent_graph,
                label_shuffled_graph,
                bindings.partition.manifest_cid.as_bytes(),
            ],
        );
        let cross_surface_evidence_cid = report_tagged_cid(
            b"r4-deployed-quality-cross-surface-evidence/1",
            &[
                &cross_surface.checks.to_le_bytes(),
                &cross_surface.mismatches.to_le_bytes(),
                &1_000u64.to_le_bytes(),
                &0u64.to_le_bytes(),
                cross_surface.graph_cid.as_bytes(),
                cross_surface.signature_artifact_cid.as_bytes(),
                cross_surface_parity,
            ],
        );
        DeployedQualityReport {
            schema: DEPLOYED_QUALITY_REPORT_SCHEMA,
            profile: QualityProfileIdentity {
                id: DEPLOYED_QUALITY_PROFILE_ID.to_owned(),
                version: DEPLOYED_QUALITY_PROFILE_VERSION,
                execution_scope: NORMATIVE_EXECUTION_SCOPE.to_owned(),
            },
            bindings,
            evaluation: EvaluationEvidence {
                mode: EvaluationMode::FullCensus,
                population_size: 1_000,
                evaluated_positions: 1_000,
                verdict: QualityVerdict::Pass,
                measurements: Some(QualityMeasurements {
                    versus_tla: comparison(tla_comparator, 250, 100, 50, 600),
                    versus_sections_absent: comparison(
                        sections_absent_comparator.clone(),
                        250,
                        100,
                        30,
                        620,
                    ),
                    internal_base_control_checks: 1_000,
                    internal_base_control_mismatches: 0,
                    cross_surface_checks: cross_surface
                        .checks
                        .checked_add(1_000)
                        .expect("fixture cross-surface count"),
                    cross_surface_mismatches: cross_surface.mismatches,
                    cross_surface_evidence_cid,
                }),
            },
            witness_replay: WitnessReplayEvidence {
                sample_cid: witness.sample_positions_cid.clone(),
                requested: witness.requested,
                replayed: witness.replayed,
                failures: witness.failures,
            },
            negative_controls: vec![NegativeControlEvidence {
                id: LABEL_SHUFFLED_CONTROL_ID.to_owned(),
                identity_cid: label_identity_cid,
                verdict: NegativeControlVerdict::Passed,
                comparison: Some(comparison(sections_absent_comparator, 250, 20, 30, 700)),
            }],
        }
    }

    /// Write a small, fully production-admissible generation for root-crate
    /// release tests. Unlike the historical manifest-only fixtures, every
    /// report identity is reproduced from these exact bytes.
    pub(crate) fn write_production_bundle(dir: &Path) -> VerifiedPackagingAdmission {
        let teacher = std::fs::read(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("crates/uor-r4-core/tests/fixtures/tless_artifacts.bin"),
        )
        .expect("checked-in teacher fixture");
        let mut tokenizer = Vec::new();
        for piece in [b" ".as_slice(), b"a".as_slice()] {
            tokenizer.extend_from_slice(&(piece.len() as i32).to_le_bytes());
            tokenizer.extend_from_slice(piece);
        }
        let (corpus_meta, corpus_records) = completed_corpus();
        let artifacts = compiler::parse_artifacts(&teacher).expect("teacher fixture parses");
        let mut store: runtime::Store = (0..=compiler::STAGES).map(|_| BTreeMap::new()).collect();
        for (index, code) in [
            [3, 1, 4, 1],
            [3, 1, 4, 2],
            [3, 5, 9, 2],
            [7, 5, 9, 2],
            [7, 5, 8, 2],
            [11, 5, 8, 7],
        ]
        .iter()
        .enumerate()
        {
            runtime::add_evidence(&mut store, code, (index + 1) as u32, 1);
        }
        let store_bytes = runtime::store_bytes(&store);
        let base = convert_r4g1::convert(&teacher, &artifacts, &store, &store_bytes, None)
            .expect("base graph conversion")
            .0;
        let graph =
            bind_graph_head_and_lanes(&base, &teacher, &tokenizer, &corpus_meta, &corpus_records);
        let sections_absent_graph = graph_without_lanes(&graph);
        let label_shuffled_graph = graph_with_shuffled_lane(&graph);
        let tokenizer_cid = digest(b"distinct raw tokenizer definition fixture");
        assert_ne!(
            tokenizer_cid,
            digest(&tokenizer),
            "source tokenizer definition and exported runtime bytes are separate identities"
        );
        let mut adapter = TokenizerAdapter {
            family: TokenizerAdapter::HF_BYTE_BPE_FAMILY.to_owned(),
            version: TokenizerAdapter::HF_BYTE_BPE_VERSION,
            tokenizer_cid,
            ..Default::default()
        };
        adapter.adapter_digest = adapter.declared_digest();
        let adapter_bytes = serde_json::to_vec_pretty(&adapter).expect("adapter JSON");
        let score_report = b"{\"config\":{}}\n";
        let compile_report = b"{\"cover\":\"fixture\"}\n";

        write(dir, GRAPH_RELATIVE_PATH, &graph);
        write(
            dir,
            SECTIONS_ABSENT_GRAPH_RELATIVE_PATH,
            &sections_absent_graph,
        );
        write(
            dir,
            LABEL_SHUFFLED_GRAPH_RELATIVE_PATH,
            &label_shuffled_graph,
        );
        write(dir, SIGNATURE_ARTIFACT_RELATIVE_PATH, &teacher);
        write(dir, TLA_COMPARATOR_STORE_RELATIVE_PATH, &store_bytes);
        write(dir, TOKENIZER_RELATIVE_PATH, &tokenizer);
        write(dir, SCORE_REPORT_RELATIVE_PATH, score_report);
        write(dir, COMPILE_REPORT_RELATIVE_PATH, compile_report);
        write(dir, CORPUS_META_RELATIVE_PATH, &corpus_meta);
        write(dir, CORPUS_RECORDS_RELATIVE_PATH, &corpus_records);
        write(dir, TOKENIZER_ADAPTER_RELATIVE_PATH, &adapter_bytes);

        let corpus = compiler::load_corpus_bytes(&corpus_meta, &corpus_records, None)
            .expect("fixture corpus parses");
        let (_, held_out) = compiler::split_positions(&corpus);
        let held_out: Vec<u64> = held_out
            .into_iter()
            .map(|position| position as u64)
            .collect();
        let bindings = derive_deployed_quality_bindings(DeployedQualityBindingMaterial {
            graph: &graph,
            teacher_artifact: &teacher,
            corpus_meta: &corpus_meta,
            corpus_records: &corpus_records,
            tokenizer: &tokenizer,
            tokenizer_adapter: &adapter_bytes,
            score_report,
            compile_report,
            compiler_revision: PRODUCTION_COMPILER_REVISION,
            full_population_positions: &held_out,
            evaluated_positions: &held_out,
        })
        .expect("fixture bindings derive");
        let context = [5u32, 5u32];
        let mut engine = NormativeServingEngine::load_for_research(EngineParts {
            graph: &graph,
            signature_artifact: &teacher,
            tokenizer: Some(&tokenizer),
            score_report: Some(score_report),
        })
        .expect("fixture normative engine loads");
        let decision = engine.predict(&context).expect("fixture decision");
        let NormativeServingDecision::Serve(direct_serve) = decision else {
            panic!("fixture context must serve")
        };
        assert!(
            direct_serve.lane_reachable,
            "fixture context must exercise the planted serving lane"
        );
        let (greedy_disposition, greedy_token) = disposition_and_token(decision);
        let (sampled_disposition, sampled_token) = sampled_disposition_and_token(decision, 42);
        let session_signature = uor_r4_router::session_signature_from_tokens(&context);
        engine.reset_policy_state();
        let session_decision = engine
            .predict_with_session_signature(&context, Some(&session_signature))
            .expect("fixture session decision");
        let NormativeServingDecision::Serve(session_serve) = session_decision else {
            panic!("fixture session context must serve")
        };
        assert!(
            session_serve.lane_reachable,
            "fixture session context must exercise the planted serving lane"
        );
        let (session_greedy_disposition, session_greedy_token) =
            disposition_and_token(session_decision);
        let (session_sampled_disposition, session_sampled_token) =
            sampled_disposition_and_token(session_decision, 42);
        let mut cross_builder = CrossSurfaceParityEvidenceBuilder::new_for_bundle(
            &graph,
            &teacher,
            Some(&tokenizer),
            Some(score_report),
        );
        for (surface, policy, authoritative, authoritative_token, disposition, signature) in [
            (
                "direct-api",
                "greedy",
                decision,
                greedy_token,
                greedy_disposition,
                None,
            ),
            (
                "direct-api",
                "default-sampled-seed-42",
                decision,
                sampled_token,
                sampled_disposition,
                None,
            ),
            (
                "r4g1-state-native-host-adapter",
                "greedy",
                decision,
                greedy_token,
                greedy_disposition,
                None,
            ),
            (
                "r4g1-state-native-host-adapter",
                "default-sampled-seed-42",
                decision,
                sampled_token,
                sampled_disposition,
                None,
            ),
            (
                "direct-api-session-bound",
                "beam-first-step",
                session_decision,
                session_greedy_token,
                session_greedy_disposition,
                Some(session_signature.as_slice()),
            ),
            (
                "direct-api-session-bound",
                "default-sampled-seed-42",
                session_decision,
                session_sampled_token,
                session_sampled_disposition,
                Some(session_signature.as_slice()),
            ),
            (
                "cli-chat-shared-production-step",
                "beam-first-step",
                session_decision,
                session_greedy_token,
                session_greedy_disposition,
                Some(session_signature.as_slice()),
            ),
            (
                "cli-chat-shared-production-step",
                "default-sampled-seed-42",
                session_decision,
                session_sampled_token,
                session_sampled_disposition,
                Some(session_signature.as_slice()),
            ),
        ] {
            cross_builder
                .record(CrossSurfaceParityObservation {
                    surface,
                    decode_policy: policy,
                    context_tokens: &context,
                    session_signature: signature,
                    authoritative,
                    authoritative_token,
                    observed_disposition: disposition,
                    observed_token: authoritative_token,
                    observed_candidates: match authoritative {
                        NormativeServingDecision::Serve(outcome) => Some(outcome.candidates),
                        NormativeServingDecision::Abstain(_)
                        | NormativeServingDecision::Decline(_) => None,
                    },
                })
                .expect("fixture cross-surface observation");
        }
        let cross_surface = cross_builder.finish().expect("fixture parity evidence");
        let cross_surface_bytes = cross_surface
            .deterministic_json_bytes()
            .expect("fixture parity JSON");
        write(
            dir,
            CROSS_SURFACE_PARITY_RELATIVE_PATH,
            &cross_surface_bytes,
        );

        let witness = produce_normative_witness_replay(NormativeWitnessReplaySpec {
            material: NormativeWitnessReplayMaterial {
                graph: &graph,
                signature_artifact: &teacher,
                tokenizer: &tokenizer,
                score_report: Some(score_report),
                corpus_meta: &corpus_meta,
                corpus_records: &corpus_records,
            },
            evaluated_positions: &held_out,
            sample_size: DEFAULT_NORMATIVE_WITNESS_SAMPLE,
        })
        .expect("fixture witness replay");
        write(
            dir,
            WITNESS_REPLAY_RELATIVE_PATH,
            &witness
                .deterministic_json_bytes()
                .expect("fixture witness JSON"),
        );

        let report = production_report(
            bindings,
            &store_bytes,
            &sections_absent_graph,
            &label_shuffled_graph,
            &cross_surface_bytes,
            &cross_surface,
            &witness,
        );
        write(
            dir,
            DEPLOYED_QUALITY_REPORT_RELATIVE_PATH,
            &report.deterministic_json_bytes().expect("report bytes"),
        );
        verify_bundle_for_production_packaging(dir, PRODUCTION_COMPILER_REVISION)
            .expect("production fixture verifies")
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
            selector: SelectorIdentity {
                id: uor_r4_api::NORMATIVE_SELECTOR_ID.to_string(),
                semantics_version: "1".to_string(),
                semantics_cid: PLACEHOLDER_DIGEST.to_string(),
            },
            compiler: CompilerIdentity {
                revision: VALID_REV.to_string(),
                configuration_cid: PLACEHOLDER_DIGEST.to_string(),
            },
            provenance_note: Some("packaged in a unit test".to_string()),
        }
    }

    fn production_inputs(admission: &VerifiedPackagingAdmission) -> PackageInputs {
        let mut inputs = valid_inputs();
        inputs.capability = BundleCapability::InstructionChat;
        inputs.tokenizer_adapter = admission.tokenizer_adapter.clone();
        inputs.selector = admission.bindings.selector.clone();
        inputs.compiler = admission.bindings.compiler.clone();
        inputs
    }

    #[test]
    fn production_packaging_reproduces_bindings_from_complete_bundle_bytes() {
        let dir = scratch_dir("production-admission");
        let admission = write_production_bundle(&dir);
        let manifest = package_verified_release_bundle(
            &dir,
            production_inputs(&admission),
            PRODUCTION_COMPILER_REVISION,
        )
        .expect("verified production bundle packages");
        assert_eq!(
            manifest.selector.as_ref(),
            Some(&admission.bindings.selector)
        );
        assert_eq!(
            manifest.compiler.as_ref(),
            Some(&admission.bindings.compiler)
        );
        assert_eq!(manifest.tokenizer_adapter, admission.tokenizer_adapter);

        let wrong_revision = "fedcba9876543210fedcba9876543210fedcba98";
        let error = verify_bundle_for_production_packaging(&dir, wrong_revision)
            .expect_err("report cannot choose the external compiler revision");
        assert!(error.to_string().contains("compiler"), "{error}");

        let cross_path = dir.join(CROSS_SURFACE_PARITY_RELATIVE_PATH);
        let original_cross_bytes = std::fs::read(&cross_path).expect("fixture parity evidence");
        let mut incomplete_cross: CrossSurfaceParityEvidence =
            serde_json::from_slice(&original_cross_bytes).expect("fixture parity parses");
        incomplete_cross
            .records
            .pop()
            .expect("eight canonical rows");
        incomplete_cross.checks = incomplete_cross.records.len() as u64;
        incomplete_cross.mismatches = incomplete_cross
            .records
            .iter()
            .filter(|record| !record.matched)
            .count() as u64;
        write(
            &dir,
            CROSS_SURFACE_PARITY_RELATIVE_PATH,
            &incomplete_cross
                .deterministic_json_bytes()
                .expect("incomplete canonical parity JSON"),
        );
        let error = verify_bundle_for_production_packaging(&dir, PRODUCTION_COMPILER_REVISION)
            .expect_err("one matching row cannot stand in for the eight-row production inventory");
        assert!(error.to_string().contains("exactly 8 checks"), "{error}");
        write(
            &dir,
            CROSS_SURFACE_PARITY_RELATIVE_PATH,
            &original_cross_bytes,
        );

        let report_path = dir.join(DEPLOYED_QUALITY_REPORT_RELATIVE_PATH);
        let original_report_bytes = std::fs::read(&report_path).expect("fixture quality report");
        let original_report: DeployedQualityReport =
            serde_json::from_slice(&original_report_bytes).expect("fixture report parses");

        let raw_cross: CrossSurfaceParityEvidence =
            serde_json::from_slice(&original_cross_bytes).expect("fixture parity parses");
        let mut planted = original_report.clone();
        let measurements = planted
            .evaluation
            .measurements
            .as_mut()
            .expect("production measurements");
        measurements.internal_base_control_checks = 0;
        measurements.internal_base_control_mismatches = 0;
        measurements.cross_surface_checks = raw_cross.checks;
        measurements.cross_surface_mismatches = raw_cross.mismatches;
        measurements.cross_surface_evidence_cid = report_tagged_cid(
            b"r4-deployed-quality-cross-surface-evidence/1",
            &[
                &raw_cross.checks.to_le_bytes(),
                &raw_cross.mismatches.to_le_bytes(),
                &0u64.to_le_bytes(),
                &0u64.to_le_bytes(),
                raw_cross.graph_cid.as_bytes(),
                raw_cross.signature_artifact_cid.as_bytes(),
                &original_cross_bytes,
            ],
        );
        write(
            &dir,
            DEPLOYED_QUALITY_REPORT_RELATIVE_PATH,
            &planted.deterministic_json_bytes().expect("planted report"),
        );
        let error = verify_bundle_for_production_packaging(&dir, PRODUCTION_COMPILER_REVISION)
            .expect_err("external rows cannot replace census-wide absent identity checks");
        assert!(
            error.to_string().contains("sections-absent identity"),
            "{error}"
        );

        let mut planted = original_report.clone();
        planted
            .evaluation
            .measurements
            .as_mut()
            .expect("production measurements")
            .cross_surface_evidence_cid = PLACEHOLDER_DIGEST.to_owned();
        write(
            &dir,
            DEPLOYED_QUALITY_REPORT_RELATIVE_PATH,
            &planted.deterministic_json_bytes().expect("planted report"),
        );
        let error = verify_bundle_for_production_packaging(&dir, PRODUCTION_COMPILER_REVISION)
            .expect_err("report cannot replace the raw cross-surface evidence identity");
        assert!(
            error.to_string().contains("cross_surface_parity"),
            "{error}"
        );

        let mut planted = original_report.clone();
        planted.witness_replay.sample_cid = PLACEHOLDER_DIGEST.to_owned();
        write(
            &dir,
            DEPLOYED_QUALITY_REPORT_RELATIVE_PATH,
            &planted.deterministic_json_bytes().expect("planted report"),
        );
        let error = verify_bundle_for_production_packaging(&dir, PRODUCTION_COMPILER_REVISION)
            .expect_err("report cannot replace the raw witness identity");
        assert!(error.to_string().contains("witness_replay"), "{error}");

        let mut planted = original_report;
        planted.negative_controls[0].identity_cid = PLACEHOLDER_DIGEST.to_owned();
        write(
            &dir,
            DEPLOYED_QUALITY_REPORT_RELATIVE_PATH,
            &planted.deterministic_json_bytes().expect("planted report"),
        );
        let error = verify_bundle_for_production_packaging(&dir, PRODUCTION_COMPILER_REVISION)
            .expect_err("report cannot replace the raw control graph identity");
        assert!(
            error.to_string().contains("score_label_shuffled"),
            "{error}"
        );

        write(
            &dir,
            DEPLOYED_QUALITY_REPORT_RELATIVE_PATH,
            &original_report_bytes,
        );

        let mut records =
            std::fs::read(dir.join(CORPUS_RECORDS_RELATIVE_PATH)).expect("fixture corpus records");
        records[4] ^= 1;
        write(&dir, CORPUS_RECORDS_RELATIVE_PATH, &records);
        let error = verify_bundle_for_production_packaging(&dir, PRODUCTION_COMPILER_REVISION)
            .expect_err("corpus mutation must invalidate the report binding");
        assert!(
            error
                .to_string()
                .contains("graph.HEAD.corpus_construction_cid"),
            "{error}"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn production_packaging_rejects_comparator_substitution() {
        let dir = scratch_dir("comparator-substitution");
        let admission = write_production_bundle(&dir);
        let report_path = dir.join(DEPLOYED_QUALITY_REPORT_RELATIVE_PATH);
        let store_path = dir.join(TLA_COMPARATOR_STORE_RELATIVE_PATH);
        let original_store = std::fs::read(&store_path).expect("fixture TLA store");
        let original_report_bytes = std::fs::read(&report_path).expect("fixture quality report");
        let original_report: DeployedQualityReport =
            serde_json::from_slice(&original_report_bytes).expect("fixture report parses");

        let write_planted_report = |report: &DeployedQualityReport| {
            write(
                &dir,
                DEPLOYED_QUALITY_REPORT_RELATIVE_PATH,
                &report.deterministic_json_bytes().expect("planted report"),
            );
        };

        let mut planted = original_report.clone();
        planted
            .evaluation
            .measurements
            .as_mut()
            .expect("production measurements")
            .versus_tla
            .comparator
            .version = "plain-tla-same-position/substituted".to_owned();
        write_planted_report(&planted);
        let error = verify_bundle_for_production_packaging(&dir, PRODUCTION_COMPILER_REVISION)
            .expect_err("an unrecognized TLA comparator version must fail closed");
        assert!(
            error.to_string().contains("TLA comparison identifies"),
            "{error}"
        );

        let mut planted = original_report.clone();
        planted
            .evaluation
            .measurements
            .as_mut()
            .expect("production measurements")
            .versus_tla
            .comparator
            .definition_cid = PLACEHOLDER_DIGEST.to_owned();
        write_planted_report(&planted);
        let error = verify_bundle_for_production_packaging(&dir, PRODUCTION_COMPILER_REVISION)
            .expect_err("a substituted TLA comparator definition must fail closed");
        assert!(error.to_string().contains("tless_store.bin"), "{error}");

        let mut malformed_store = original_store.clone();
        malformed_store[0] ^= 1;
        let mut rebound = original_report.clone();
        rebound
            .evaluation
            .measurements
            .as_mut()
            .expect("production measurements")
            .versus_tla
            .comparator
            .definition_cid = report_tagged_cid(
            b"r4-deployed-quality-tla-comparator/1",
            &[TLA_COMPARATOR_VERSION.as_bytes(), &malformed_store],
        );
        write_planted_report(&rebound);
        write(&dir, TLA_COMPARATOR_STORE_RELATIVE_PATH, &malformed_store);
        let structurally_rebound_manifest =
            package_release_bundle(&dir, production_inputs(&admission))
                .expect("a structural manifest can be rebound to the malformed inputs");
        assert_eq!(
            structurally_rebound_manifest
                .components
                .tla_comparator_store
                .as_deref(),
            Some(digest(&malformed_store).as_str())
        );
        let error = verify_bundle_for_production_packaging(&dir, PRODUCTION_COMPILER_REVISION)
            .expect_err("a malformed but content-rebound TLA store must fail closed");
        assert!(error.to_string().contains("not a valid TLS1"), "{error}");

        write(
            &dir,
            DEPLOYED_QUALITY_REPORT_RELATIVE_PATH,
            &original_report_bytes,
        );
        write(&dir, TLA_COMPARATOR_STORE_RELATIVE_PATH, &original_store);
        let mut planted = original_report.clone();
        planted
            .evaluation
            .measurements
            .as_mut()
            .expect("production measurements")
            .versus_sections_absent
            .comparator
            .positions_cid = PLACEHOLDER_DIGEST.to_owned();
        write_planted_report(&planted);
        let error = verify_bundle_for_production_packaging(&dir, PRODUCTION_COMPILER_REVISION)
            .expect_err("a comparator over different positions must fail closed");
        assert!(
            error
                .to_string()
                .contains("comparison.comparator.positions_cid"),
            "{error}"
        );

        let mut planted = original_report.clone();
        planted.negative_controls[0]
            .comparison
            .as_mut()
            .expect("label-shuffled comparison")
            .comparator
            .version = "r4g1-sections-absent/substituted".to_owned();
        write_planted_report(&planted);
        let error = verify_bundle_for_production_packaging(&dir, PRODUCTION_COMPILER_REVISION)
            .expect_err("a nested control comparator substitution must fail closed");
        assert!(
            error
                .to_string()
                .contains("exact validated sections-absent comparator"),
            "{error}"
        );

        write(
            &dir,
            DEPLOYED_QUALITY_REPORT_RELATIVE_PATH,
            &original_report_bytes,
        );
        let mut substituted_store = original_store;
        let last = substituted_store
            .len()
            .checked_sub(1)
            .expect("fixture TLA store is nonempty");
        substituted_store[last] ^= 1;
        write(&dir, TLA_COMPARATOR_STORE_RELATIVE_PATH, &substituted_store);
        let error = verify_bundle_for_production_packaging(&dir, PRODUCTION_COMPILER_REVISION)
            .expect_err("changed TLA store bytes must invalidate comparator admission");
        assert!(error.to_string().contains("tless_store.bin"), "{error}");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn builds_a_valid_manifest_from_real_files_on_disk() {
        let dir = scratch_dir("full-bundle");
        write_full_bundle(&dir);
        let manifest = package_release_bundle(&dir, valid_inputs()).expect("full bundle packages");
        assert_eq!(manifest.model_id, "r4");
        assert_eq!(manifest.components.graph, digest(b"graph bytes"));
        assert_eq!(
            manifest.components.sections_absent_graph.as_deref(),
            Some(digest(b"sections absent graph bytes").as_str())
        );
        assert_eq!(
            manifest.components.label_shuffled_graph.as_deref(),
            Some(digest(b"label shuffled graph bytes").as_str())
        );
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
        assert_eq!(
            manifest.components.cross_surface_parity.as_deref(),
            Some(digest(b"{\"cross_surface\":true}").as_str())
        );
        assert_eq!(
            manifest.components.witness_replay.as_deref(),
            Some(digest(b"{\"witness\":true}").as_str())
        );
        assert_eq!(manifest.abi, BundleAbi::from(AbiVersion::current()));
        assert_eq!(manifest.validate(), None, "returned manifest is valid");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn absent_tokenizer_packages_with_no_tokenizer_digest() {
        let dir = scratch_dir("no-tokenizer");
        write_full_bundle(&dir);
        std::fs::remove_file(dir.join(TOKENIZER_RELATIVE_PATH)).expect("remove tokenizer fixture");
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
    fn schema_two_packaging_requires_every_raw_evidence_artifact() {
        for relative in [
            SECTIONS_ABSENT_GRAPH_RELATIVE_PATH,
            LABEL_SHUFFLED_GRAPH_RELATIVE_PATH,
            CROSS_SURFACE_PARITY_RELATIVE_PATH,
            WITNESS_REPLAY_RELATIVE_PATH,
        ] {
            let dir = scratch_dir(&format!("missing-{}", relative.replace('/', "-")));
            write_full_bundle(&dir);
            std::fs::remove_file(dir.join(relative)).expect("remove planted required artifact");
            let error = package_release_bundle(&dir, valid_inputs())
                .expect_err("schema-2 packaging must fail closed on missing raw evidence");
            assert!(error.to_string().contains(relative), "{relative}: {error}");
            let _ = std::fs::remove_dir_all(&dir);
        }
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

    /// #655-D3: closes the loop from "D1/D2 can produce a sidecar" to
    /// "C1c's verifier actually accepts what D2 produces" -- the two
    /// halves are each independently well-tested (this module's own
    /// tests above; `release_bundle_loader`'s synthetic-fixture tests)
    /// but neither proves they compose against the exact bytes/paths
    /// the other side expects. This test writes a real
    /// `release-bundle.json` the same way #655-D2's CLI command does
    /// (`serde_json::to_vec_pretty` to
    /// `release_bundle_loader::RELEASE_BUNDLE_SIDECAR_FILE_NAME` next to
    /// `physical_root`) and asserts
    /// `release_bundle_loader::verify_release_bundle_sidecar` returns
    /// `Some` of that exact manifest, unblocking #655-C1d.
    #[test]
    fn packaged_bundle_is_accepted_by_the_loaders_sidecar_verifier() {
        use crate::release_bundle_loader::{
            verify_release_bundle_sidecar, RELEASE_BUNDLE_SIDECAR_FILE_NAME,
        };

        let dir = scratch_dir("d3-golden-round-trip");
        write_full_bundle(&dir);
        let manifest = package_release_bundle(&dir, valid_inputs()).expect("full bundle packages");
        std::fs::write(
            dir.join(RELEASE_BUNDLE_SIDECAR_FILE_NAME),
            serde_json::to_vec_pretty(&manifest).expect("serialize manifest"),
        )
        .expect("write sidecar, mirroring main.rs's package_release_bundle_command");

        let graph_path = dir.join(GRAPH_RELATIVE_PATH);
        let teacher_path = dir.join(SIGNATURE_ARTIFACT_RELATIVE_PATH);
        let verified = verify_release_bundle_sidecar(&dir, &graph_path, &teacher_path);
        assert_eq!(
            verified,
            Some(manifest),
            "the loader must accept exactly what the packager just wrote"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// #655-D3 companion: the same round trip, but for an
    /// `InstructionChat` bundle carrying a real (non-default)
    /// `tokenizer_adapter`, mirroring #655-D2's `--source` +
    /// `--tokenizer-family`/`--tokenizer-version` path rather than only
    /// the simpler `Continuation` default-adapter case above.
    #[test]
    fn instruction_chat_bundle_with_real_tokenizer_adapter_round_trips_through_the_loader() {
        use crate::release_bundle_loader::{
            verify_release_bundle_sidecar, RELEASE_BUNDLE_SIDECAR_FILE_NAME,
        };

        let dir = scratch_dir("d3-golden-instruction-chat");
        write_full_bundle(&dir);
        let mut inputs = valid_inputs();
        inputs.capability = BundleCapability::InstructionChat;
        inputs.tokenizer_adapter = TokenizerAdapter {
            family: "hf-byte-bpe".to_string(),
            ..Default::default()
        };
        let manifest =
            package_release_bundle(&dir, inputs).expect("instruction-chat bundle packages");
        std::fs::write(
            dir.join(RELEASE_BUNDLE_SIDECAR_FILE_NAME),
            serde_json::to_vec_pretty(&manifest).expect("serialize manifest"),
        )
        .expect("write sidecar, mirroring main.rs's package_release_bundle_command");

        let graph_path = dir.join(GRAPH_RELATIVE_PATH);
        let teacher_path = dir.join(SIGNATURE_ARTIFACT_RELATIVE_PATH);
        let verified = verify_release_bundle_sidecar(&dir, &graph_path, &teacher_path);
        assert_eq!(
            verified,
            Some(manifest),
            "the loader must accept an InstructionChat sidecar with a real tokenizer_adapter too"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// #655-D1 preview: confirms `package_release_bundle` works
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
