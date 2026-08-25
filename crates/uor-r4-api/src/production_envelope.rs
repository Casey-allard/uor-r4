//! Filesystem-free schema-2 production-envelope admission.
//!
//! Native startup and browser startup must make the same decision from the
//! same bytes. This module owns that portable decision: component CIDs,
//! tokenizer adapter identity, canonical held-out population, deployed-quality
//! bindings, and the token-free-D4 plus R4G1Runtime serving composition are all
//! checked before a generation can become active.

use uor_r4_core::transformerless::{compiler, hf_bpe::TokenizerAdapter, runtime};
use uor_r4_graph_format::{GraphView, SectionId};
use uor_r4_model_source::SourceUnavailable;

use crate::deployed_quality::{
    derive_deployed_quality_bindings, DeployedQualityBindingMaterial, DeployedQualityBindings,
    DeployedQualityReport, WitnessReplayEvidence, LABEL_SHUFFLED_CONTROL_ID,
    SECTIONS_ABSENT_COMPARATOR_VERSION, TLA_COMPARATOR_VERSION,
};
use crate::engine::{AbiVersion, EngineParts};
use crate::release_bundle::{BundleAbi, ReleaseBundleManifest};
use crate::serving::{validate_production_serving_parts, ProductionServingParts};
use crate::witness_replay::{
    parse_and_validate_normative_witness_replay, NormativeWitnessReplayMaterial,
    NormativeWitnessReplaySpec, DEFAULT_NORMATIVE_WITNESS_SAMPLE,
};

/// Exact bytes that make up one settled schema-2 production generation.
///
/// No path or ambient process state participates in verification. Callers may
/// capture these bytes from guarded native file handles or fetch them into a
/// browser, then pass the same immutable generation here.
#[derive(Debug, Clone, Copy)]
pub struct ProductionEnvelopeParts<'a> {
    pub graph: &'a [u8],
    pub sections_absent_graph: &'a [u8],
    pub label_shuffled_graph: &'a [u8],
    pub signature_artifact: &'a [u8],
    pub tla_comparator_store: &'a [u8],
    pub tokenizer: &'a [u8],
    pub score_report: &'a [u8],
    pub compile_report: &'a [u8],
    pub deployed_quality_report: &'a [u8],
    pub cross_surface_parity: &'a [u8],
    pub witness_replay: &'a [u8],
    pub corpus_meta: &'a [u8],
    pub corpus_records: &'a [u8],
    pub tokenizer_adapter: &'a [u8],
    pub release_manifest: &'a [u8],
}

/// Raw evidence whose contents, not merely file digests, must agree with one
/// deployed-quality report. Kept separate from [`ProductionEnvelopeParts`]
/// so native packagers can validate a staging generation before its manifest
/// has been serialized.
#[derive(Debug, Clone, Copy)]
pub struct ProductionEvidenceParts<'a> {
    pub graph: &'a [u8],
    pub sections_absent_graph: &'a [u8],
    pub label_shuffled_graph: &'a [u8],
    pub signature_artifact: &'a [u8],
    pub tla_comparator_store: &'a [u8],
    pub tokenizer: &'a [u8],
    pub score_report: &'a [u8],
    pub deployed_quality_report: &'a [u8],
    pub cross_surface_parity: &'a [u8],
    pub witness_replay: &'a [u8],
    pub corpus_meta: &'a [u8],
    pub corpus_records: &'a [u8],
}

/// Unforgeable result of verifying one complete production envelope.
///
/// The fields are deliberately private: production-serving constructors take
/// this value as a capability, so a caller cannot substitute a manifest CID
/// or a hand-built set of quality bindings after verification.
#[derive(Debug)]
pub struct VerifiedProductionEnvelope {
    manifest: ReleaseBundleManifest,
    loaded_bindings: DeployedQualityBindings,
    _verified: VerifiedEnvelopeSeal,
}

#[derive(Debug)]
struct VerifiedEnvelopeSeal;

impl VerifiedProductionEnvelope {
    /// Schema-2 manifest whose exact bytes participated in verification.
    pub fn manifest(&self) -> &ReleaseBundleManifest {
        &self.manifest
    }

    /// Independently reproduced identities retained for serving admission.
    pub fn loaded_bindings(&self) -> &DeployedQualityBindings {
        &self.loaded_bindings
    }
}

/// Verify one immutable schema-2 generation without filesystem access.
pub fn verify_production_envelope(
    parts: ProductionEnvelopeParts<'_>,
) -> Result<VerifiedProductionEnvelope, SourceUnavailable> {
    let manifest: ReleaseBundleManifest = serde_json::from_slice(parts.release_manifest)
        .map_err(|error| unavailable(format!("invalid production release-bundle.json: {error}")))?;
    if let Some(reason) = manifest.validate() {
        return Err(unavailable(format!(
            "production release-bundle.json is not schema-2 admissible: {reason}"
        )));
    }

    let expected_abi = BundleAbi::from(AbiVersion::current());
    if manifest.abi != expected_abi {
        return Err(unavailable(format!(
            "production release ABI {:?} does not equal this runtime ABI {:?}",
            manifest.abi, expected_abi
        )));
    }

    require_digest("graph/score.r4g1", parts.graph, &manifest.components.graph)?;
    require_schema_two_digest(
        "graph/score_sections_absent.r4g1",
        parts.sections_absent_graph,
        manifest.components.sections_absent_graph.as_deref(),
    )?;
    require_schema_two_digest(
        "graph/score_label_shuffled.r4g1",
        parts.label_shuffled_graph,
        manifest.components.label_shuffled_graph.as_deref(),
    )?;
    require_digest(
        "tless_artifacts.bin",
        parts.signature_artifact,
        &manifest.components.signature_artifact,
    )?;
    require_schema_two_digest(
        "tless_store.bin",
        parts.tla_comparator_store,
        manifest.components.tla_comparator_store.as_deref(),
    )?;
    require_digest(
        "graph/score_report.json",
        parts.score_report,
        &manifest.components.score_report,
    )?;
    require_digest(
        "graph-cover/cover_report.json",
        parts.compile_report,
        &manifest.components.compile_report,
    )?;
    let declared_quality_cid = manifest
        .components
        .deployed_quality_report
        .as_deref()
        .ok_or_else(|| unavailable("schema-2 manifest omitted deployed-quality report CID"))?;
    require_digest(
        "graph/deployed_quality_report.json",
        parts.deployed_quality_report,
        declared_quality_cid,
    )?;
    require_schema_two_digest(
        "graph/cross_surface_parity.json",
        parts.cross_surface_parity,
        manifest.components.cross_surface_parity.as_deref(),
    )?;
    require_schema_two_digest(
        "graph/witness_replay.json",
        parts.witness_replay,
        manifest.components.witness_replay.as_deref(),
    )?;
    let declared_tokenizer_cid = manifest
        .components
        .tokenizer
        .as_deref()
        .ok_or_else(|| unavailable("schema-2 production admission requires tokenizer.bin"))?;
    require_digest("tokenizer.bin", parts.tokenizer, declared_tokenizer_cid)?;

    serde_json::from_slice::<serde_json::Value>(parts.score_report)
        .map_err(|error| unavailable(format!("invalid graph/score_report.json: {error}")))?;
    serde_json::from_slice::<serde_json::Value>(parts.compile_report)
        .map_err(|error| unavailable(format!("invalid graph-cover/cover_report.json: {error}")))?;

    let adapter: TokenizerAdapter = serde_json::from_slice(parts.tokenizer_adapter)
        .map_err(|error| unavailable(format!("invalid tokenizer_adapter.json: {error}")))?;
    if adapter != manifest.tokenizer_adapter {
        return Err(unavailable(
            "tokenizer_adapter.json does not equal the schema-2 release manifest adapter",
        ));
    }
    let reproduced_adapter_digest = adapter.declared_digest();
    if adapter.adapter_digest != reproduced_adapter_digest {
        return Err(unavailable(format!(
            "tokenizer_adapter.json declares digest {}, but its fields reproduce {reproduced_adapter_digest}",
            adapter.adapter_digest
        )));
    }

    let corpus = compiler::load_corpus_bytes(parts.corpus_meta, parts.corpus_records, None)
        .ok_or_else(|| unavailable("corpus.meta/corpus.records do not parse as one corpus"))?;
    let (_, held_out) = compiler::split_positions(&corpus);
    let certification_positions = held_out
        .into_iter()
        .map(|position| {
            u64::try_from(position).map_err(|_| {
                unavailable("certification corpus position does not fit the u64 wire identity")
            })
        })
        .collect::<Result<Vec<_>, SourceUnavailable>>()?;

    let manifest_selector = manifest
        .selector
        .as_ref()
        .ok_or_else(|| unavailable("schema-2 release manifest omitted selector identity"))?;
    let manifest_compiler = manifest
        .compiler
        .as_ref()
        .ok_or_else(|| unavailable("schema-2 release manifest omitted compiler identity"))?;
    let loaded_bindings = derive_deployed_quality_bindings(DeployedQualityBindingMaterial {
        graph: parts.graph,
        teacher_artifact: parts.signature_artifact,
        corpus_meta: parts.corpus_meta,
        corpus_records: parts.corpus_records,
        tokenizer: parts.tokenizer,
        tokenizer_adapter: parts.tokenizer_adapter,
        score_report: parts.score_report,
        compile_report: parts.compile_report,
        compiler_revision: &manifest_compiler.revision,
        full_population_positions: &certification_positions,
        evaluated_positions: &certification_positions,
    })
    .map_err(|error| unavailable(error.to_string()))?;
    if manifest_selector != &loaded_bindings.selector {
        return Err(unavailable(format!(
            "schema-2 manifest selector {:?} does not equal the selector derived by this runtime {:?}",
            manifest_selector, loaded_bindings.selector
        )));
    }
    if manifest_compiler != &loaded_bindings.compiler {
        return Err(unavailable(format!(
            "schema-2 manifest compiler identity {:?} does not equal the captured compiler/configuration identity {:?}",
            manifest_compiler, loaded_bindings.compiler
        )));
    }

    let verified = VerifiedProductionEnvelope {
        manifest,
        loaded_bindings,
        _verified: VerifiedEnvelopeSeal,
    };
    validate_production_serving_parts(&ProductionServingParts {
        engine: EngineParts {
            graph: parts.graph,
            signature_artifact: parts.signature_artifact,
            tokenizer: Some(parts.tokenizer),
            score_report: Some(parts.score_report),
        },
        deployed_quality_report: parts.deployed_quality_report,
        verified_envelope: &verified,
    })?;
    validate_production_evidence_links(
        ProductionEvidenceParts {
            graph: parts.graph,
            sections_absent_graph: parts.sections_absent_graph,
            label_shuffled_graph: parts.label_shuffled_graph,
            signature_artifact: parts.signature_artifact,
            tla_comparator_store: parts.tla_comparator_store,
            tokenizer: parts.tokenizer,
            score_report: parts.score_report,
            deployed_quality_report: parts.deployed_quality_report,
            cross_surface_parity: parts.cross_surface_parity,
            witness_replay: parts.witness_replay,
            corpus_meta: parts.corpus_meta,
            corpus_records: parts.corpus_records,
        },
        &certification_positions,
    )?;

    Ok(verified)
}

fn require_schema_two_digest(
    label: &str,
    bytes: &[u8],
    declared: Option<&str>,
) -> Result<(), SourceUnavailable> {
    let declared = declared.ok_or_else(|| {
        unavailable(format!(
            "schema-2 production manifest omitted required component {label}"
        ))
    })?;
    require_digest(label, bytes, declared)
}

pub fn validate_production_evidence_links(
    parts: ProductionEvidenceParts<'_>,
    certification_positions: &[u64],
) -> Result<(), SourceUnavailable> {
    validate_control_graphs(
        parts.graph,
        parts.sections_absent_graph,
        parts.label_shuffled_graph,
    )?;
    let report: DeployedQualityReport = serde_json::from_slice(parts.deployed_quality_report)
        .map_err(|error| unavailable(format!("invalid deployed-quality report: {error}")))?;
    let measurements = report.evaluation.measurements.as_ref().ok_or_else(|| {
        unavailable("production deployed-quality report omitted measured evidence")
    })?;

    if runtime::parse_store(parts.tla_comparator_store).is_none() {
        return Err(unavailable(
            "tless_store.bin is not a valid TLS1 plain-TLA comparator store",
        ));
    }
    let expected_tla = tagged_cid(
        b"r4-deployed-quality-tla-comparator/1",
        &[
            TLA_COMPARATOR_VERSION.as_bytes(),
            parts.tla_comparator_store,
        ],
    );
    if measurements.versus_tla.comparator.definition_cid != expected_tla {
        return Err(unavailable(
            "deployed-quality report is not bound to tless_store.bin",
        ));
    }

    let expected_absent = tagged_cid(
        b"r4-deployed-quality-sections-absent-comparator/1",
        &[
            SECTIONS_ABSENT_COMPARATOR_VERSION.as_bytes(),
            parts.sections_absent_graph,
        ],
    );
    if measurements
        .versus_sections_absent
        .comparator
        .definition_cid
        != expected_absent
    {
        return Err(unavailable(
            "deployed-quality report is not bound to graph/score_sections_absent.r4g1",
        ));
    }

    let label_control = report
        .negative_controls
        .iter()
        .find(|control| control.id == LABEL_SHUFFLED_CONTROL_ID)
        .ok_or_else(|| unavailable("deployed-quality report omitted label-shuffled control"))?;
    const SHUFFLED_VERSION: &str = "train-target-rotation-half-plus-one/1";
    let expected_shuffled = tagged_cid(
        b"r4-deployed-quality-label-shuffled-control/1",
        &[
            SHUFFLED_VERSION.as_bytes(),
            parts.sections_absent_graph,
            parts.label_shuffled_graph,
            report.bindings.partition.manifest_cid.as_bytes(),
        ],
    );
    if label_control.identity_cid != expected_shuffled {
        return Err(unavailable(
            "deployed-quality report is not bound to graph/score_label_shuffled.r4g1",
        ));
    }

    let cross =
        crate::serving::CrossSurfaceParityEvidence::parse_and_validate_for_production_bundle(
            parts.cross_surface_parity,
            parts.graph,
            parts.signature_artifact,
            parts.tokenizer,
            parts.score_report,
        )?;
    let expected_combined_checks = cross
        .checks
        .checked_add(measurements.internal_base_control_checks)
        .ok_or_else(|| unavailable("combined cross-surface check count overflow"))?;
    let expected_combined_mismatches = cross
        .mismatches
        .checked_add(measurements.internal_base_control_mismatches)
        .ok_or_else(|| unavailable("combined cross-surface mismatch count overflow"))?;
    if measurements.cross_surface_checks != expected_combined_checks
        || measurements.cross_surface_mismatches != expected_combined_mismatches
    {
        return Err(unavailable(
            "combined cross-surface counts do not equal raw external plus declared internal evidence",
        ));
    }
    if measurements.internal_base_control_checks != report.evaluation.evaluated_positions
        || measurements.internal_base_control_mismatches != 0
    {
        return Err(unavailable(format!(
            "full-census sections-absent identity covered {}/{} evaluated positions with {} mismatches",
            measurements.internal_base_control_checks,
            report.evaluation.evaluated_positions,
            measurements.internal_base_control_mismatches
        )));
    }
    let expected_cross = tagged_cid(
        b"r4-deployed-quality-cross-surface-evidence/1",
        &[
            &cross.checks.to_le_bytes(),
            &cross.mismatches.to_le_bytes(),
            &measurements.internal_base_control_checks.to_le_bytes(),
            &measurements.internal_base_control_mismatches.to_le_bytes(),
            cross.graph_cid.as_bytes(),
            cross.signature_artifact_cid.as_bytes(),
            parts.cross_surface_parity,
        ],
    );
    if measurements.cross_surface_evidence_cid != expected_cross {
        return Err(unavailable(
            "deployed-quality report is not transitively bound to graph/cross_surface_parity.json",
        ));
    }

    validate_bound_witness_replay(
        parts.witness_replay,
        NormativeWitnessReplayMaterial {
            graph: parts.graph,
            signature_artifact: parts.signature_artifact,
            tokenizer: parts.tokenizer,
            score_report: Some(parts.score_report),
            corpus_meta: parts.corpus_meta,
            corpus_records: parts.corpus_records,
        },
        certification_positions,
        &report.witness_replay,
    )?;
    Ok(())
}

/// Validate the complete canonical witness stream at the final production
/// trust boundary. Digest and aggregate equality are insufficient here: every
/// schema-2 candidate, including its SKMX/PSIB provenance and lane
/// attribution, must replay through a fresh normative engine over the exact
/// certification positions and immutable generation bytes.
fn validate_bound_witness_replay(
    bytes: &[u8],
    material: NormativeWitnessReplayMaterial<'_>,
    certification_positions: &[u64],
    expected: &WitnessReplayEvidence,
) -> Result<(), SourceUnavailable> {
    let witness = parse_and_validate_normative_witness_replay(
        bytes,
        NormativeWitnessReplaySpec {
            material,
            evaluated_positions: certification_positions,
            sample_size: DEFAULT_NORMATIVE_WITNESS_SAMPLE,
        },
    )
    .map_err(|error| unavailable(format!("production witness replay failed: {error}")))?;
    if witness.sample_positions_cid != expected.sample_cid
        || witness.requested != expected.requested
        || witness.replayed != expected.replayed
        || witness.failures != expected.failures
    {
        return Err(unavailable(
            "deployed-quality report and graph/witness_replay.json do not describe the same replay evidence",
        ));
    }
    Ok(())
}

fn validate_control_graphs(
    main: &[u8],
    sections_absent: &[u8],
    label_shuffled: &[u8],
) -> Result<(), SourceUnavailable> {
    let main = parse_control_graph("main", main)?;
    let absent = parse_control_graph("sections-absent", sections_absent)?;
    let shuffled = parse_control_graph("label-shuffled", label_shuffled)?;
    if main.section(SectionId::SKMX).is_none() || main.section(SectionId::PSIB).is_none() {
        return Err(unavailable("main graph lacks required SKMX/PSIB sections"));
    }
    if absent.section(SectionId::SKMX).is_some() || absent.section(SectionId::PSIB).is_some() {
        return Err(unavailable(
            "sections-absent control still contains SKMX or PSIB",
        ));
    }
    if shuffled.section(SectionId::SKMX).is_none() || shuffled.section(SectionId::PSIB).is_none() {
        return Err(unavailable("label-shuffled control lacks SKMX or PSIB"));
    }
    require_non_lane_identity(&main, &absent, "sections-absent")?;
    require_non_lane_identity(&main, &shuffled, "label-shuffled")?;
    if [SectionId::SKMX, SectionId::PSIB]
        .iter()
        .all(|&id| main.section(id) == shuffled.section(id))
    {
        return Err(unavailable(
            "label-shuffled control is byte-identical to the main lane",
        ));
    }
    Ok(())
}

fn parse_control_graph<'a>(
    label: &str,
    bytes: &'a [u8],
) -> Result<GraphView<'a>, SourceUnavailable> {
    let view = GraphView::parse(bytes)
        .map_err(|error| unavailable(format!("{label} graph parse failed: {error}")))?;
    view.verify_cids()
        .map_err(|error| unavailable(format!("{label} graph CID check failed: {error}")))?;
    Ok(view)
}

fn require_non_lane_identity(
    main: &GraphView<'_>,
    control: &GraphView<'_>,
    label: &str,
) -> Result<(), SourceUnavailable> {
    for section in main.sections() {
        if section.id == SectionId::SKMX || section.id == SectionId::PSIB {
            continue;
        }
        let Some(other) = control
            .sections()
            .find(|candidate| candidate.id == section.id)
        else {
            return Err(unavailable(format!(
                "{label} control is missing non-lane section 0x{:08x}",
                section.id.raw()
            )));
        };
        if other.flags != section.flags || other.payload != section.payload {
            return Err(unavailable(format!(
                "{label} control changes non-lane section 0x{:08x}",
                section.id.raw()
            )));
        }
    }
    for section in control.sections() {
        if section.id != SectionId::SKMX
            && section.id != SectionId::PSIB
            && main.section(section.id).is_none()
        {
            return Err(unavailable(format!(
                "{label} control adds non-lane section 0x{:08x}",
                section.id.raw()
            )));
        }
    }
    Ok(())
}

fn tagged_cid(tag: &[u8], parts: &[&[u8]]) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(&(tag.len() as u64).to_le_bytes());
    hasher.update(tag);
    for part in parts {
        hasher.update(&(part.len() as u64).to_le_bytes());
        hasher.update(part);
    }
    format!("blake3:{}", hasher.finalize().to_hex())
}

fn require_digest(label: &str, bytes: &[u8], declared: &str) -> Result<(), SourceUnavailable> {
    let actual = format!("blake3:{}", blake3::hash(bytes).to_hex());
    if actual == declared {
        Ok(())
    } else {
        Err(unavailable(format!(
            "production component {label} digest mismatch: manifest declares {declared}, captured bytes hash to {actual}"
        )))
    }
}

fn unavailable(reason: impl Into<String>) -> SourceUnavailable {
    SourceUnavailable::new(reason.into())
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use uor_r4_core::transformerless::compiler::STAGES;
    use uor_r4_core::transformerless::{convert_r4g1, runtime};

    use super::*;
    use crate::serving::{NormativeServingDecision, NormativeServingEngine};
    use crate::witness_replay::{
        produce_normative_witness_replay, NormativeWitnessCandidateSource,
    };

    #[test]
    fn malformed_manifest_is_refused_before_any_component_can_activate() {
        let empty = &[];
        let error = verify_production_envelope(ProductionEnvelopeParts {
            graph: empty,
            sections_absent_graph: empty,
            label_shuffled_graph: empty,
            signature_artifact: empty,
            tla_comparator_store: empty,
            tokenizer: empty,
            score_report: empty,
            compile_report: empty,
            deployed_quality_report: empty,
            cross_surface_parity: empty,
            witness_replay: empty,
            corpus_meta: empty,
            corpus_records: empty,
            tokenizer_adapter: empty,
            release_manifest: br#"{}"#,
        })
        .expect_err("an incomplete envelope must fail closed");
        assert!(error.to_string().contains("release-bundle.json"));
    }

    struct SyntheticWitnessMaterial {
        graph: Vec<u8>,
        teacher: Vec<u8>,
        tokenizer: Vec<u8>,
        score_report: Vec<u8>,
        corpus_meta: Vec<u8>,
        corpus_records: Vec<u8>,
        served_position: u64,
    }

    fn synthetic_witness_material() -> SyntheticWitnessMaterial {
        let teacher = std::fs::read(format!(
            "{}/../uor-r4-core/tests/fixtures/tless_artifacts.bin",
            env!("CARGO_MANIFEST_DIR")
        ))
        .expect("teacher artifact fixture");
        let artifacts = compiler::parse_artifacts(&teacher).expect("teacher artifact parses");
        let mut store: runtime::Store = (0..=STAGES).map(|_| BTreeMap::new()).collect();
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
        let graph = convert_r4g1::convert(&teacher, &artifacts, &store, &store_bytes, None)
            .expect("convert synthetic graph")
            .0;

        let next = [3u16, 1, 4, 3, 1, 4, 7, 5, 8, 3, 1, 4, 7, 5, 8, 3];
        let mut corpus_meta = Vec::with_capacity(25);
        corpus_meta.extend_from_slice(&(next.len() as u64).to_le_bytes());
        corpus_meta.extend_from_slice(&1u64.to_le_bytes());
        corpus_meta.extend_from_slice(&0u64.to_le_bytes());
        corpus_meta.push(1);
        let mut corpus_records = Vec::with_capacity(next.len() * 16);
        for token in next {
            corpus_records.extend_from_slice(&0u32.to_le_bytes());
            corpus_records.extend_from_slice(&token.to_le_bytes());
            corpus_records.extend_from_slice(&token.to_le_bytes());
            corpus_records.extend_from_slice(&(-0.1f32).to_le_bytes());
        }
        let tokenizer = [b"<unk>".as_slice(), b"<s>", b"</s>", b" ", b"a"]
            .into_iter()
            .fold(Vec::new(), |mut bytes, token| {
                bytes.extend_from_slice(&(token.len() as i32).to_le_bytes());
                bytes.extend_from_slice(token);
                bytes
            });
        let score_report = b"{}".to_vec();
        let corpus = compiler::load_corpus_bytes(&corpus_meta, &corpus_records, None)
            .expect("synthetic corpus parses");
        let mut engine = NormativeServingEngine::load_for_research(EngineParts {
            graph: &graph,
            signature_artifact: &teacher,
            tokenizer: Some(&tokenizer),
            score_report: Some(&score_report),
        })
        .expect("synthetic serving engine loads");
        let served_position = (0..corpus.n)
            .find(|&position| {
                engine.reset_policy_state();
                let window = uor_r4_graph_compiler::induction::context_window(&corpus, position);
                matches!(
                    engine.predict(&window).expect("synthetic decision"),
                    NormativeServingDecision::Serve(_)
                )
            })
            .expect("synthetic corpus has a served position") as u64;
        SyntheticWitnessMaterial {
            graph,
            teacher,
            tokenizer,
            score_report,
            corpus_meta,
            corpus_records,
            served_position,
        }
    }

    #[test]
    fn production_boundary_replays_candidate_provenance_instead_of_trusting_records() {
        let fixture = synthetic_witness_material();
        let evaluated = [fixture.served_position];
        let material = NormativeWitnessReplayMaterial {
            graph: &fixture.graph,
            signature_artifact: &fixture.teacher,
            tokenizer: &fixture.tokenizer,
            score_report: Some(&fixture.score_report),
            corpus_meta: &fixture.corpus_meta,
            corpus_records: &fixture.corpus_records,
        };
        let spec = NormativeWitnessReplaySpec {
            material,
            evaluated_positions: &evaluated,
            sample_size: DEFAULT_NORMATIVE_WITNESS_SAMPLE,
        };
        let artifact = produce_normative_witness_replay(spec).expect("produce witness replay");
        let expected = WitnessReplayEvidence {
            sample_cid: artifact.sample_positions_cid.clone(),
            requested: artifact.requested,
            replayed: artifact.replayed,
            failures: artifact.failures,
        };
        let canonical = artifact
            .deterministic_json_bytes()
            .expect("canonical witness bytes");
        validate_bound_witness_replay(&canonical, material, &evaluated, &expected)
            .expect("canonical witness replays at the production boundary");

        for (skmx_contributed, psib_contributed) in [(true, false), (false, true)] {
            let mut planted = artifact.clone();
            let candidate = planted.records[0]
                .candidate
                .as_mut()
                .expect("served record has a candidate");
            assert_eq!(candidate.source, NormativeWitnessCandidateSource::Base);
            candidate.source = NormativeWitnessCandidateSource::Skipmix;
            candidate.skmx_contributed = skmx_contributed;
            candidate.psib_contributed = psib_contributed;
            let planted = planted
                .deterministic_json_bytes()
                .expect("canonical planted witness bytes");
            let error = validate_bound_witness_replay(&planted, material, &evaluated, &expected)
                .expect_err("forged candidate provenance must fail independent replay");
            assert!(error
                .to_string()
                .contains("production witness replay failed"));
        }
    }
}
