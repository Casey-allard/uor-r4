//! Versioned, content-bound deployed-quality evidence for the normative
//! R4G1 selector (#933).
//!
//! This module is deliberately schema-only. It does not run an evaluation,
//! read bundle files, or select a token. Report producers supply the measured
//! integer counts and exact identities; production loaders call
//! [`DeployedQualityReport::validate_for_production`] with identities derived
//! independently from the bytes they loaded. Historical and off-serving JSON
//! remains inspectable only through [`parse_deployed_quality_for_research`].
//!
//! Rates use parts per million and paired integer counts. No floating-point
//! value participates in serialization or admission, so identical evidence
//! produces identical report bytes and a stable report CID.

use core::fmt;

use serde::{Deserialize, Serialize};
use uor_r4_core::transformerless::{compiler, hf_bpe::TokenizerAdapter};
use uor_r4_graph_format::{
    corpus_partition_cid, ArtifactCid, CorpusPartitionRole, GraphView, SectionId,
};
use uor_r4_model_source::SourceUnavailable;

/// Current deployed-quality report schema.
pub const DEPLOYED_QUALITY_REPORT_SCHEMA: u32 = 1;
/// The only production profile this schema currently admits.
pub const DEPLOYED_QUALITY_PROFILE_ID: &str = "r4g1-runtime-deployed-quality";
/// Version of [`DEPLOYED_QUALITY_PROFILE_ID`].
pub const DEPLOYED_QUALITY_PROFILE_VERSION: u32 = 1;
/// ADR-0001's sole normative candidate/token selector.
pub const NORMATIVE_SELECTOR_ID: &str = "R4G1Runtime";
/// Versioned semantic contract hashed into every derived selector binding.
pub const NORMATIVE_SELECTOR_SEMANTICS_VERSION: &str = "r4g1-runtime-serving-selector/1";
/// Formal-vocabulary execution scope required for production evidence.
pub const NORMATIVE_EXECUTION_SCOPE: &str = "normative-runtime";
/// Era-neutral same-position plain-TLA runtime comparator identity. The exact
/// container/store generation is bound separately by its byte-derived CID.
pub const TLA_COMPARATOR_ID: &str = "plain-tla-runtime";
/// Versioned plain-TLA comparator implementation. The production envelope
/// independently hashes this tag together with the exact graded-store bytes.
pub const TLA_COMPARATOR_VERSION: &str = "plain-tla-same-position/1";
/// Normative selector with SKMX/PSIB absent, used for RF-31's causal delta.
pub const SECTIONS_ABSENT_COMPARATOR_ID: &str = "R4G1Runtime-sections-absent";
/// Versioned sections-absent comparator implementation. Its definition CID is
/// independently reproduced from the exact planted control graph.
pub const SECTIONS_ABSENT_COMPARATOR_VERSION: &str = "r4g1-sections-absent/1";
/// The required label-shuffled conditioning-specificity falsifier.
pub const LABEL_SHUFFLED_CONTROL_ID: &str = "label-shuffled-skmx-psib";
/// RF-31's frozen +20 per-mille paired lower-bound floor, in ppm.
pub const RF31_MIN_LANE_DELTA_PPM: i64 = 20_000;
/// Canonical full-population position order bound into production evidence.
pub const FULL_POPULATION_SELECTION_ALGORITHM: &str = "ascending-certification-position/1";
/// Canonical nested sample selection. Stories and their positions are both
/// visited in recursive midpoint order, round by round; the chosen set is then
/// sorted for deterministic reduction. A larger requested prefix therefore
/// contains every position from a smaller prefix while spreading both
/// prefixes across held-out stories and within-story regions.
pub const DETERMINISTIC_SAMPLE_SELECTION_ALGORITHM: &str = "story-round-robin-midpoint-prefix/1";

/// Canonical paired-interval implementation emitted and recomputed by schema
/// version one. The fixed rational `196/100` is the predeclared normal approximation.
/// No platform floating-point operation participates in the bound.
pub const PAIRED_INTERVAL_METHOD: &str = "paired-wald-95-fixed-ppm-v1";
/// Confidence label carried by [`PAIRED_INTERVAL_METHOD`].
pub const PAIRED_INTERVAL_CONFIDENCE_PPM: u32 = 950_000;

// The integer formula below cubes the population and squares the discordant
// count. Restricting it to u32-sized populations keeps every intermediate in
// u128 while exceeding any currently representable recorded-corpus census.
const MAX_INTERVAL_POPULATION: u64 = u32::MAX as u64;

const RATE_SCALE_PPM: u128 = 1_000_000;

const NORMATIVE_SELECTOR_SEMANTICS: &[u8] = b"r4g1-runtime-serving-selector/1\n\
candidate-authority=R4G1Runtime\n\
policy-authority=token-free-D4\n\
selection=ranked-candidate-top1\n\
lane=SKMX-then-PSIB\n";
const NORMATIVE_DECODE_IMPLEMENTATION: &str = "normative-ranked-candidate-argmax/1";
const NORMATIVE_DECODE_CONFIGURATION: &[u8] = b"normative-ranked-candidate-argmax/1\n\
mode=greedy-top1\n\
tie-break=canonical-candidate-order\n";
const PARTITION_SPLIT_VERSION: &str = "story-disjoint-80-20/1";

/// Report a field-specific failure at the sanctioned host-ingestion boundary.
fn binding_error(field: &'static str, reason: impl Into<String>) -> SourceUnavailable {
    SourceUnavailable::new(format!(
        "deployed-quality binding {field}: {}",
        reason.into()
    ))
}

/// Select one nested, story-distributed prefix from a canonical certification
/// population. This function owns the sample semantics used by evaluators,
/// witness generation, report construction, and binding validation.
///
/// `population` must be strictly increasing and every absolute position must
/// index `story_by_position`. The returned positions are strictly increasing;
/// selection order itself is committed by
/// [`DETERMINISTIC_SAMPLE_SELECTION_ALGORITHM`].
pub fn deterministic_story_sample(
    story_by_position: &[u32],
    population: &[u64],
    requested: usize,
) -> Result<Vec<u64>, SourceUnavailable> {
    if requested == 0 {
        return Err(binding_error("evaluated_positions", "sample size is zero"));
    }
    validate_positions("full_population_positions", population, false)?;
    for &position in population {
        let index = usize::try_from(position).map_err(|_| {
            binding_error(
                "full_population_positions",
                "a corpus position does not fit usize",
            )
        })?;
        if index >= story_by_position.len() {
            return Err(binding_error(
                "full_population_positions",
                format!(
                    "position {position} is outside story map length {}",
                    story_by_position.len()
                ),
            ));
        }
    }
    let target = requested.min(population.len());
    if target == population.len() {
        return Ok(population.to_vec());
    }

    // The certification partition is in corpus order, so equal story IDs are
    // contiguous. Preserve that canonical story order without a map whose
    // iteration semantics could become part of the evidence contract.
    let mut stories: Vec<&[u64]> = Vec::new();
    let mut seen_story_ids = std::collections::BTreeSet::new();
    let mut start = 0usize;
    while start < population.len() {
        let absolute = usize::try_from(population[start]).map_err(|_| {
            binding_error(
                "full_population_positions",
                "a corpus position does not fit usize",
            )
        })?;
        let story = story_by_position[absolute];
        if !seen_story_ids.insert(story) {
            return Err(binding_error(
                "full_population_positions",
                format!("story {story} appears in more than one disjoint corpus segment"),
            ));
        }
        let mut end = start + 1;
        while end < population.len() {
            let absolute = usize::try_from(population[end]).map_err(|_| {
                binding_error(
                    "full_population_positions",
                    "a corpus position does not fit usize",
                )
            })?;
            if story_by_position[absolute] != story {
                break;
            }
            end += 1;
        }
        stories.push(&population[start..end]);
        start = end;
    }

    fn midpoint_indices(len: usize) -> Vec<usize> {
        let mut order = Vec::with_capacity(len);
        let mut ranges = std::collections::VecDeque::from([(0usize, len)]);
        while let Some((start, end)) = ranges.pop_front() {
            if start >= end {
                continue;
            }
            let midpoint = start + (end - start - 1) / 2;
            order.push(midpoint);
            if start < midpoint {
                ranges.push_back((start, midpoint));
            }
            if midpoint + 1 < end {
                ranges.push_back((midpoint + 1, end));
            }
        }
        order
    }

    let story_order = midpoint_indices(stories.len());
    let per_story: Vec<Vec<u64>> = stories
        .into_iter()
        .map(|story| {
            midpoint_indices(story.len())
                .into_iter()
                .map(|index| story[index])
                .collect()
        })
        .collect();
    let mut selected = Vec::with_capacity(target);
    let mut round = 0usize;
    while selected.len() < target {
        let before = selected.len();
        for &story_index in &story_order {
            if let Some(&position) = per_story[story_index].get(round) {
                selected.push(position);
                if selected.len() == target {
                    break;
                }
            }
        }
        if selected.len() == before {
            return Err(binding_error(
                "evaluated_positions",
                "story-distributed selector exhausted before filling the sample",
            ));
        }
        round += 1;
    }
    selected.sort_unstable();
    Ok(selected)
}

/// Raw byte material from one loaded generation. With the exception of the
/// source revision (which is not encoded in R4G1 v0), every binding is derived
/// from these bytes rather than accepted from a report.
#[derive(Debug, Clone, Copy)]
pub struct DeployedQualityBindingMaterial<'a> {
    pub graph: &'a [u8],
    pub teacher_artifact: &'a [u8],
    pub corpus_meta: &'a [u8],
    pub corpus_records: &'a [u8],
    pub tokenizer: &'a [u8],
    pub tokenizer_adapter: &'a [u8],
    pub score_report: &'a [u8],
    pub compile_report: &'a [u8],
    pub compiler_revision: &'a str,
    /// Absolute corpus indices in canonical certification-population order.
    pub full_population_positions: &'a [u64],
    /// Absolute corpus indices evaluated by this report, in canonical order.
    pub evaluated_positions: &'a [u64],
}

/// A typed deployed-quality validation failure. The public validation surface
/// returns `Option<Self>` rather than a custom-error `Result` to follow the
/// shipped API's R5 convention while still allowing callers to distinguish
/// schema, structure, admissibility, and binding failures.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeployedQualityValidationError {
    UnsupportedSchema {
        found: u32,
        supported: u32,
    },
    Structural {
        field: &'static str,
        reason: String,
    },
    NotProductionAdmissible {
        reason: String,
    },
    IdentityMismatch {
        field: &'static str,
        report: String,
        loaded: String,
    },
}

impl fmt::Display for DeployedQualityValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedSchema { found, supported } => write!(
                f,
                "unsupported deployed-quality schema {found} (this build reads {supported})"
            ),
            Self::Structural { field, reason } => {
                write!(f, "deployed-quality field {field}: {reason}")
            }
            Self::NotProductionAdmissible { reason } => {
                write!(
                    f,
                    "deployed-quality report is not production-admissible: {reason}"
                )
            }
            Self::IdentityMismatch {
                field,
                report,
                loaded,
            } => write!(
                f,
                "deployed-quality identity mismatch for {field}: report {report}, loaded {loaded}"
            ),
        }
    }
}

impl std::error::Error for DeployedQualityValidationError {}

fn structural(field: &'static str, reason: impl Into<String>) -> DeployedQualityValidationError {
    DeployedQualityValidationError::Structural {
        field,
        reason: reason.into(),
    }
}

/// Whether `value` is a workspace-standard `blake3:<64 hex>` CID.
pub fn is_blake3_cid(value: &str) -> bool {
    value
        .strip_prefix("blake3:")
        .is_some_and(|hex| hex.len() == 64 && hex.chars().all(|c| c.is_ascii_hexdigit()))
}

fn is_canonical_blake3_cid(value: &str) -> bool {
    value.strip_prefix("blake3:").is_some_and(|hex| {
        hex.len() == 64
            && hex
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    })
}

fn validate_cid(field: &'static str, value: &str) -> Option<DeployedQualityValidationError> {
    (!is_blake3_cid(value)).then(|| structural(field, format!("{value:?} is not a blake3 CID")))
}

fn validate_nonempty(field: &'static str, value: &str) -> Option<DeployedQualityValidationError> {
    value
        .trim()
        .is_empty()
        .then(|| structural(field, "is empty"))
}

/// Profile identity: report semantics and formal execution scope.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct QualityProfileIdentity {
    pub id: String,
    pub version: u32,
    pub execution_scope: String,
}

/// Candidate/token selector identity and its versioned semantics artifact.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SelectorIdentity {
    pub id: String,
    pub semantics_version: String,
    pub semantics_cid: String,
}

impl SelectorIdentity {
    fn validate_shape(&self) -> Option<DeployedQualityValidationError> {
        validate_nonempty("bindings.selector.id", &self.id)
            .or_else(|| {
                validate_nonempty(
                    "bindings.selector.semantics_version",
                    &self.semantics_version,
                )
            })
            .or_else(|| validate_cid("bindings.selector.semantics_cid", &self.semantics_cid))
    }

    /// Validate the identity required by ADR-0001 for production selection.
    pub fn validate_normative(&self) -> Option<DeployedQualityValidationError> {
        self.validate_shape().or_else(|| {
            (self.id != NORMATIVE_SELECTOR_ID).then(|| {
                DeployedQualityValidationError::NotProductionAdmissible {
                    reason: format!(
                        "selector {:?} is not the normative selector {:?}",
                        self.id, NORMATIVE_SELECTOR_ID
                    ),
                }
            })
        })
    }
}

/// Byte and representation identities for a graph or teacher artifact.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ArtifactIdentity {
    pub bytes_cid: String,
    pub artifact_kappa: String,
}

impl ArtifactIdentity {
    fn validate(&self, prefix: &'static str) -> Option<DeployedQualityValidationError> {
        let bytes_field = match prefix {
            "graph" => "bindings.graph.bytes_cid",
            _ => "bindings.teacher_artifact.bytes_cid",
        };
        let kappa_field = match prefix {
            "graph" => "bindings.graph.artifact_kappa",
            _ => "bindings.teacher_artifact.artifact_kappa",
        };
        validate_cid(bytes_field, &self.bytes_cid)
            .or_else(|| validate_cid(kappa_field, &self.artifact_kappa))
    }
}

/// Exact recorded-corpus byte identities.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CorpusIdentity {
    pub meta_cid: String,
    pub records_cid: String,
    pub stream_cid: String,
}

impl CorpusIdentity {
    fn validate(&self) -> Option<DeployedQualityValidationError> {
        validate_cid("bindings.corpus.meta_cid", &self.meta_cid)
            .or_else(|| validate_cid("bindings.corpus.records_cid", &self.records_cid))
            .or_else(|| validate_cid("bindings.corpus.stream_cid", &self.stream_cid))
    }
}

/// Construction/certification split and exact evaluated-position identity.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PartitionIdentity {
    pub manifest_cid: String,
    pub construction_cid: String,
    pub certification_cid: String,
    pub evaluated_positions_cid: String,
    pub split_version: String,
}

impl PartitionIdentity {
    fn validate(&self) -> Option<DeployedQualityValidationError> {
        validate_cid("bindings.partition.manifest_cid", &self.manifest_cid)
            .or_else(|| {
                validate_cid(
                    "bindings.partition.construction_cid",
                    &self.construction_cid,
                )
            })
            .or_else(|| {
                validate_cid(
                    "bindings.partition.certification_cid",
                    &self.certification_cid,
                )
            })
            .or_else(|| {
                validate_cid(
                    "bindings.partition.evaluated_positions_cid",
                    &self.evaluated_positions_cid,
                )
            })
            .or_else(|| validate_nonempty("bindings.partition.split_version", &self.split_version))
    }
}

/// Tokenizer bytes plus the registered adapter/configuration identity.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct QualityTokenizerIdentity {
    pub bytes_cid: String,
    pub adapter_id: String,
    pub adapter_version: String,
    pub adapter_config_cid: String,
}

impl QualityTokenizerIdentity {
    fn validate(&self) -> Option<DeployedQualityValidationError> {
        validate_cid("bindings.tokenizer.bytes_cid", &self.bytes_cid)
            .or_else(|| validate_nonempty("bindings.tokenizer.adapter_id", &self.adapter_id))
            .or_else(|| {
                validate_nonempty("bindings.tokenizer.adapter_version", &self.adapter_version)
            })
            .or_else(|| {
                validate_cid(
                    "bindings.tokenizer.adapter_config_cid",
                    &self.adapter_config_cid,
                )
            })
    }
}

/// Compiler source revision and exact graph-producing configuration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CompilerIdentity {
    pub revision: String,
    pub configuration_cid: String,
}

impl CompilerIdentity {
    fn validate(&self) -> Option<DeployedQualityValidationError> {
        if self.revision.len() != 40
            || !self
                .revision
                .chars()
                .all(|character| character.is_ascii_hexdigit())
        {
            return Some(structural(
                "bindings.compiler.revision",
                format!("{:?} is not a 40-character git revision", self.revision),
            ));
        }
        validate_cid(
            "bindings.compiler.configuration_cid",
            &self.configuration_cid,
        )
    }
}

/// One active R4G1 section and its exact bytes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ActiveSectionIdentity {
    pub id: String,
    pub cid: String,
}

/// Ordered identity of every section active for the measured selector.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ActiveSectionSetIdentity {
    pub set_cid: String,
    pub sections: Vec<ActiveSectionIdentity>,
}

impl ActiveSectionSetIdentity {
    fn validate(&self) -> Option<DeployedQualityValidationError> {
        if let Some(error) = validate_cid("bindings.active_sections.set_cid", &self.set_cid) {
            return Some(error);
        }
        if self.sections.is_empty() {
            return Some(structural(
                "bindings.active_sections.sections",
                "contains no active section",
            ));
        }
        let mut previous: Option<&str> = None;
        for section in &self.sections {
            if section.id.trim().is_empty() {
                return Some(structural(
                    "bindings.active_sections.sections.id",
                    "is empty",
                ));
            }
            if previous.is_some_and(|id| id >= section.id.as_str()) {
                return Some(structural(
                    "bindings.active_sections.sections",
                    "must be strictly sorted by section id with no duplicates",
                ));
            }
            if let Some(error) = validate_cid("bindings.active_sections.sections.cid", &section.cid)
            {
                return Some(error);
            }
            previous = Some(&section.id);
        }
        None
    }

    pub fn contains(&self, id: &str) -> bool {
        self.sections
            .binary_search_by_key(&id, |section| section.id.as_str())
            .is_ok()
    }
}

/// Decode policy is separate from selector identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DecodeMode {
    GreedyTop1,
    SeededSample,
    Beam,
}

/// Exact decode implementation/configuration used by the evaluation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DecodeIdentity {
    pub mode: DecodeMode,
    pub implementation: String,
    pub configuration_cid: String,
}

impl DecodeIdentity {
    fn validate(&self) -> Option<DeployedQualityValidationError> {
        validate_nonempty("bindings.decode.implementation", &self.implementation)
            .or_else(|| validate_cid("bindings.decode.configuration_cid", &self.configuration_cid))
    }
}

/// How positions were selected from the certification population.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PositionSelectionMode {
    FullPopulation,
    DeterministicSample,
}

/// Seed and algorithm identity for population/sample selection.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SeedIdentity {
    pub mode: PositionSelectionMode,
    pub algorithm: String,
    pub seed: u64,
    pub selection_cid: String,
}

impl SeedIdentity {
    fn validate(&self) -> Option<DeployedQualityValidationError> {
        validate_nonempty("bindings.seed.algorithm", &self.algorithm)
            .or_else(|| validate_cid("bindings.seed.selection_cid", &self.selection_cid))
    }
}

/// Every independently recomputable identity a production loader must bind.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DeployedQualityBindings {
    pub selector: SelectorIdentity,
    pub graph: ArtifactIdentity,
    pub teacher_artifact: ArtifactIdentity,
    pub corpus: CorpusIdentity,
    pub partition: PartitionIdentity,
    pub tokenizer: QualityTokenizerIdentity,
    pub compiler: CompilerIdentity,
    pub serving_configuration_cid: String,
    pub active_sections: ActiveSectionSetIdentity,
    pub decode: DecodeIdentity,
    pub seed: SeedIdentity,
}

impl DeployedQualityBindings {
    fn validate_shape(&self) -> Option<DeployedQualityValidationError> {
        self.selector
            .validate_shape()
            .or_else(|| self.graph.validate("graph"))
            .or_else(|| self.teacher_artifact.validate("teacher_artifact"))
            .or_else(|| self.corpus.validate())
            .or_else(|| self.partition.validate())
            .or_else(|| self.tokenizer.validate())
            .or_else(|| self.compiler.validate())
            .or_else(|| {
                validate_cid(
                    "bindings.serving_configuration_cid",
                    &self.serving_configuration_cid,
                )
            })
            .or_else(|| self.active_sections.validate())
            .or_else(|| self.decode.validate())
            .or_else(|| self.seed.validate())
    }
}

/// Derive the production/research binding envelope from independently loaded
/// bytes. Producers and loaders call this same function; a report therefore
/// cannot choose a second spelling for a CID or position selection.
pub fn derive_deployed_quality_bindings(
    material: DeployedQualityBindingMaterial<'_>,
) -> Result<DeployedQualityBindings, SourceUnavailable> {
    validate_json_bytes("score_report", material.score_report)?;
    validate_json_bytes("compile_report", material.compile_report)?;
    if material.tokenizer.is_empty() {
        return Err(binding_error("tokenizer", "tokenizer bytes are empty"));
    }
    if material.corpus_meta.is_empty() || material.corpus_records.is_empty() {
        return Err(binding_error(
            "corpus",
            "metadata and records must both be present and nonempty",
        ));
    }
    if material.compiler_revision.len() != 40
        || !material
            .compiler_revision
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit())
    {
        return Err(binding_error(
            "compiler_revision",
            "must be a full 40-character hexadecimal revision",
        ));
    }
    validate_positions(
        "full_population_positions",
        material.full_population_positions,
        false,
    )?;
    validate_positions("evaluated_positions", material.evaluated_positions, false)?;
    if !material.evaluated_positions.iter().all(|position| {
        material
            .full_population_positions
            .binary_search(position)
            .is_ok()
    }) {
        return Err(binding_error(
            "evaluated_positions",
            "contains a position outside the full certification population",
        ));
    }

    let view = GraphView::parse(material.graph)
        .map_err(|error| binding_error("graph", format!("R4G1 parse failed: {error}")))?;
    view.verify_cids()
        .map_err(|error| binding_error("graph", format!("R4G1 CID check failed: {error}")))?;
    let head = view
        .head()
        .ok_or_else(|| binding_error("graph.HEAD", "validated graph has no HEAD section"))?;
    let corpus = compiler::load_corpus_bytes(material.corpus_meta, material.corpus_records, None)
        .ok_or_else(|| {
        binding_error("corpus", "metadata/records do not form a valid corpus")
    })?;
    let (construction_positions, certification_positions) = compiler::split_positions(&corpus);
    let construction_positions: Vec<u64> = construction_positions
        .into_iter()
        .map(|position| position as u64)
        .collect();
    let certification_positions: Vec<u64> = certification_positions
        .into_iter()
        .map(|position| position as u64)
        .collect();
    if material.full_population_positions != certification_positions.as_slice() {
        return Err(binding_error(
            "full_population_positions",
            "does not exactly equal the canonical certification partition encoded by the corpus",
        ));
    }
    let expected_construction_cid = corpus_partition_cid(
        material.corpus_meta,
        material.corpus_records,
        CorpusPartitionRole::Construction,
        &construction_positions,
    );
    let expected_certification_cid = corpus_partition_cid(
        material.corpus_meta,
        material.corpus_records,
        CorpusPartitionRole::Certification,
        &certification_positions,
    );
    if head.corpus_construction_cid() != expected_construction_cid {
        return Err(binding_error(
            "graph.HEAD.corpus_construction_cid",
            "does not bind the exact corpus construction positions",
        ));
    }
    if head.corpus_certification_cid() != expected_certification_cid {
        return Err(binding_error(
            "graph.HEAD.corpus_certification_cid",
            "does not bind the exact corpus certification positions",
        ));
    }
    let teacher = compiler::parse_artifacts(material.teacher_artifact).ok_or_else(|| {
        binding_error("teacher_artifact", "not a supported TLA artifact container")
    })?;
    let teacher_bytes_cid = bytes_cid(material.teacher_artifact);
    require_head_cid(
        "graph.HEAD.teacher_cid",
        head.teacher_cid(),
        &teacher_bytes_cid,
    )?;

    let adapter: TokenizerAdapter =
        serde_json::from_slice(material.tokenizer_adapter).map_err(|error| {
            binding_error(
                "tokenizer_adapter",
                format!("invalid tokenizer adapter JSON: {error}"),
            )
        })?;
    if adapter.family.trim().is_empty() || adapter.version == 0 {
        return Err(binding_error(
            "tokenizer_adapter",
            "adapter family is empty or version is zero",
        ));
    }
    if !is_canonical_blake3_cid(&adapter.tokenizer_cid) || cid_is_zero(&adapter.tokenizer_cid) {
        return Err(binding_error(
            "tokenizer_adapter.tokenizer_cid",
            "source tokenizer definition CID is malformed or zero",
        ));
    }
    let declared_adapter_digest = adapter.declared_digest();
    if adapter.adapter_digest != declared_adapter_digest {
        return Err(binding_error(
            "tokenizer_adapter.adapter_digest",
            format!(
                "record declares {}, canonical bytes hash to {declared_adapter_digest}",
                adapter.adapter_digest
            ),
        ));
    }
    // The adapter record binds the raw source definition (for example,
    // tokenizer.json), while HEAD binds the exported runtime tokenizer.bin
    // bytes consumed by the serving engine. They are deliberately distinct
    // identities and must not be conflated.
    let deployed_tokenizer_cid = bytes_cid(material.tokenizer);
    require_head_cid(
        "graph.HEAD.tokenizer_cid",
        head.tokenizer_cid(),
        &deployed_tokenizer_cid,
    )?;

    let construction_cid = head_cid_string(head.corpus_construction_cid());
    let certification_cid = head_cid_string(head.corpus_certification_cid());
    let compiler_version_cid = head_cid_string(head.compiler_version_cid());
    for (field, cid) in [
        ("graph.HEAD.corpus_construction_cid", &construction_cid),
        ("graph.HEAD.corpus_certification_cid", &certification_cid),
        ("graph.HEAD.compiler_version_cid", &compiler_version_cid),
    ] {
        if cid_is_zero(cid) {
            return Err(binding_error(field, "zero CID is unavailable evidence"));
        }
    }

    let stream_cid = corpus_stream_cid(material.corpus_meta, material.corpus_records);
    let population_positions_cid = positions_cid(material.full_population_positions);
    let evaluated_positions_cid = positions_cid(material.evaluated_positions);
    let partition_manifest_cid = tagged_cid(
        b"r4-deployed-quality-partition-manifest/1",
        &[
            PARTITION_SPLIT_VERSION.as_bytes(),
            stream_cid.as_bytes(),
            construction_cid.as_bytes(),
            certification_cid.as_bytes(),
            population_positions_cid.as_bytes(),
        ],
    );

    let address = uor_r4_graph_format::r4g1::address(material.graph).ok_or_else(|| {
        binding_error(
            "graph.artifact_kappa",
            "validated graph has no canonical R4G1 realization",
        )
    })?;
    let graph_representation_cid =
        tagged_cid(b"r4g1-realization-blake3/1", &[address.skeleton.as_slice()]);
    let active_sections = active_section_identity(&view);
    let compiler_configuration_cid = tagged_cid(
        b"r4-deployed-quality-compiler-configuration/1",
        &[
            material.score_report,
            material.compile_report,
            compiler_version_cid.as_bytes(),
        ],
    );
    let selector_semantics_cid = bytes_cid(NORMATIVE_SELECTOR_SEMANTICS);
    let decode_configuration_cid = bytes_cid(NORMATIVE_DECODE_CONFIGURATION);
    let serving_configuration_cid = tagged_cid(
        b"r4-deployed-quality-serving-configuration/1",
        &[
            material.score_report,
            NORMATIVE_SELECTOR_SEMANTICS,
            NORMATIVE_DECODE_CONFIGURATION,
        ],
    );
    let selection_mode = if material.evaluated_positions == material.full_population_positions {
        PositionSelectionMode::FullPopulation
    } else {
        let expected = deterministic_story_sample(
            &corpus.story,
            material.full_population_positions,
            material.evaluated_positions.len(),
        )?;
        if material.evaluated_positions != expected.as_slice() {
            return Err(binding_error(
                "evaluated_positions",
                format!(
                    "does not equal the canonical {} selection of size {}",
                    DETERMINISTIC_SAMPLE_SELECTION_ALGORITHM,
                    material.evaluated_positions.len()
                ),
            ));
        }
        PositionSelectionMode::DeterministicSample
    };

    Ok(DeployedQualityBindings {
        selector: SelectorIdentity {
            id: NORMATIVE_SELECTOR_ID.to_string(),
            semantics_version: NORMATIVE_SELECTOR_SEMANTICS_VERSION.to_string(),
            semantics_cid: selector_semantics_cid,
        },
        graph: ArtifactIdentity {
            bytes_cid: bytes_cid(material.graph),
            artifact_kappa: graph_representation_cid,
        },
        teacher_artifact: ArtifactIdentity {
            bytes_cid: teacher_bytes_cid,
            artifact_kappa: compiler::artifact_kappa(&teacher),
        },
        corpus: CorpusIdentity {
            meta_cid: bytes_cid(material.corpus_meta),
            records_cid: bytes_cid(material.corpus_records),
            stream_cid,
        },
        partition: PartitionIdentity {
            manifest_cid: partition_manifest_cid,
            construction_cid,
            certification_cid,
            evaluated_positions_cid: evaluated_positions_cid.clone(),
            split_version: PARTITION_SPLIT_VERSION.to_string(),
        },
        tokenizer: QualityTokenizerIdentity {
            bytes_cid: bytes_cid(material.tokenizer),
            adapter_id: adapter.family,
            adapter_version: adapter.version.to_string(),
            adapter_config_cid: declared_adapter_digest,
        },
        compiler: CompilerIdentity {
            revision: material.compiler_revision.to_ascii_lowercase(),
            configuration_cid: compiler_configuration_cid,
        },
        serving_configuration_cid,
        active_sections,
        decode: DecodeIdentity {
            mode: DecodeMode::GreedyTop1,
            implementation: NORMATIVE_DECODE_IMPLEMENTATION.to_string(),
            configuration_cid: decode_configuration_cid,
        },
        seed: SeedIdentity {
            mode: selection_mode,
            algorithm: match selection_mode {
                PositionSelectionMode::FullPopulation => {
                    FULL_POPULATION_SELECTION_ALGORITHM.to_string()
                }
                PositionSelectionMode::DeterministicSample => {
                    DETERMINISTIC_SAMPLE_SELECTION_ALGORITHM.to_string()
                }
            },
            seed: 0,
            selection_cid: evaluated_positions_cid,
        },
    })
}

/// Exact CID of a canonical absolute-position list. This is public so an
/// evaluator can bind rows before it constructs the rest of the report.
pub fn deployed_quality_positions_cid(positions: &[u64]) -> String {
    positions_cid(positions)
}

fn validate_json_bytes(field: &'static str, bytes: &[u8]) -> Result<(), SourceUnavailable> {
    serde_json::from_slice::<serde_json::Value>(bytes)
        .map(|_| ())
        .map_err(|error| binding_error(field, format!("invalid JSON: {error}")))
}

fn validate_positions(
    field: &'static str,
    positions: &[u64],
    allow_empty: bool,
) -> Result<(), SourceUnavailable> {
    if positions.is_empty() && !allow_empty {
        return Err(binding_error(field, "position list is empty"));
    }
    if positions.windows(2).any(|pair| pair[0] >= pair[1]) {
        return Err(binding_error(
            field,
            "positions must be strictly increasing with no duplicates",
        ));
    }
    Ok(())
}

fn bytes_cid(bytes: &[u8]) -> String {
    format!("blake3:{}", blake3::hash(bytes).to_hex())
}

fn corpus_stream_cid(meta: &[u8], records: &[u8]) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(meta);
    hasher.update(records);
    format!("blake3:{}", hasher.finalize().to_hex())
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

fn positions_cid(positions: &[u64]) -> String {
    let mut bytes = Vec::with_capacity(8 + positions.len() * 8);
    bytes.extend_from_slice(&(positions.len() as u64).to_le_bytes());
    for position in positions {
        bytes.extend_from_slice(&position.to_le_bytes());
    }
    tagged_cid(b"r4-deployed-quality-positions/1", &[&bytes])
}

fn head_cid_string(cid: ArtifactCid) -> String {
    let mut hex = String::with_capacity(64);
    for byte in cid.0 {
        use fmt::Write as _;
        let _ = write!(hex, "{byte:02x}");
    }
    format!("blake3:{hex}")
}

fn cid_is_zero(cid: &str) -> bool {
    cid.strip_prefix("blake3:")
        .is_some_and(|hex| hex.bytes().all(|byte| byte == b'0'))
}

fn require_head_cid(
    field: &'static str,
    head: ArtifactCid,
    actual: &str,
) -> Result<(), SourceUnavailable> {
    let declared = head_cid_string(head);
    if cid_is_zero(&declared) {
        return Err(binding_error(field, "zero CID is unavailable evidence"));
    }
    if declared != actual {
        return Err(binding_error(
            field,
            format!("HEAD declares {declared}, loaded bytes identify as {actual}"),
        ));
    }
    Ok(())
}

fn active_section_identity(view: &GraphView<'_>) -> ActiveSectionSetIdentity {
    let mut sections = Vec::with_capacity(view.sections().len());
    for section in view.sections() {
        let name = section_name(section.id);
        let cid = tagged_cid(
            b"r4g1-active-section/1",
            &[
                &section.id.raw().to_le_bytes(),
                &section.flags.to_le_bytes(),
                section.payload,
            ],
        );
        sections.push(ActiveSectionIdentity { id: name, cid });
    }
    sections.sort_by(|left, right| left.id.cmp(&right.id));
    let mut set_bytes = Vec::new();
    for section in &sections {
        set_bytes.extend_from_slice(&(section.id.len() as u64).to_le_bytes());
        set_bytes.extend_from_slice(section.id.as_bytes());
        set_bytes.extend_from_slice(&(section.cid.len() as u64).to_le_bytes());
        set_bytes.extend_from_slice(section.cid.as_bytes());
    }
    ActiveSectionSetIdentity {
        set_cid: tagged_cid(b"r4g1-active-section-set/1", &[&set_bytes]),
        sections,
    }
}

fn section_name(id: SectionId) -> String {
    let known = [
        (SectionId::HEAD, "HEAD"),
        (SectionId::CODE, "CODE"),
        (SectionId::NODE, "NODE"),
        (SectionId::EDGE, "EDGE"),
        (SectionId::ROUT, "ROUT"),
        (SectionId::EMIT, "EMIT"),
        (SectionId::EXCT, "EXCT"),
        (SectionId::PROV, "PROV"),
        (SectionId::CERT, "CERT"),
        (SectionId::PTCH, "PTCH"),
        (SectionId::SECT, "SECT"),
        (SectionId::RTNX, "RTNX"),
        (SectionId::NGRAM, "NGRAM"),
        (SectionId::FWDA, "FWDA"),
        (SectionId::FMM, "FMM"),
        (SectionId::PSTATE, "PSTATE"),
        (SectionId::SKMX, "SKMX"),
        (SectionId::PSIB, "PSIB"),
        (SectionId::PSCH, "PSCH"),
        (SectionId::PTRN, "PTRN"),
        (SectionId::PGOL, "PGOL"),
        (SectionId::PWIT, "PWIT"),
    ];
    known
        .iter()
        .find_map(|(known_id, name)| (*known_id == id).then(|| (*name).to_string()))
        .unwrap_or_else(|| format!("0x{:08x}", id.raw()))
}

/// Evaluation population coverage.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum EvaluationMode {
    Sample,
    FullCensus,
}

/// Evidence verdict. `Estimate` is deliberately distinct from `Pass` so a
/// sample cannot be serialized as a production verdict.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "kebab-case", deny_unknown_fields)]
pub enum QualityVerdict {
    Pass,
    Fail { reason: String },
    Estimate { decision: String },
    Unavailable { reason: String },
}

/// Typed interpretation of a sample estimate. Samples remain research-only:
/// even [`SampleDecisionKind::Proceed`] authorizes only the predeclared full
/// census, never production admission.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SampleDecisionKind {
    Proceed,
    Stop,
    Inconclusive,
}

impl SampleDecisionKind {
    /// Parse the schema-1 decision vocabulary. The one legacy phrase remains
    /// readable for checkpoint compatibility; new producers emit one of the
    /// three uppercase typed prefixes.
    pub fn parse(value: &str) -> Option<Self> {
        let value = value.trim();
        let has_reason = |prefix: &str| {
            value
                .strip_prefix(prefix)
                .is_some_and(|reason| !reason.trim().is_empty())
        };
        if value == "proceed to census" || has_reason("PROCEED:") {
            Some(Self::Proceed)
        } else if has_reason("STOP:") {
            Some(Self::Stop)
        } else if has_reason("INCONCLUSIVE:") {
            Some(Self::Inconclusive)
        } else {
            None
        }
    }
}

impl QualityVerdict {
    /// Return the typed decision carried by a sample estimate.
    pub fn sample_decision(&self) -> Option<SampleDecisionKind> {
        match self {
            Self::Estimate { decision } => SampleDecisionKind::parse(decision),
            Self::Pass | Self::Fail { .. } | Self::Unavailable { .. } => None,
        }
    }
}

/// Exact fraction plus its deterministic fixed-point presentation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExactRate {
    pub numerator: u64,
    pub denominator: u64,
    pub ppm: u32,
}

impl ExactRate {
    fn validate(
        &self,
        field: &'static str,
        expected_denominator: u64,
    ) -> Option<DeployedQualityValidationError> {
        if self.denominator == 0 || self.denominator != expected_denominator {
            return Some(structural(
                field,
                format!(
                    "denominator {} does not equal evaluated population {expected_denominator}",
                    self.denominator
                ),
            ));
        }
        if self.numerator > self.denominator {
            return Some(structural(field, "numerator exceeds denominator"));
        }
        let expected =
            ((u128::from(self.numerator) * RATE_SCALE_PPM) / u128::from(self.denominator)) as u32;
        (self.ppm != expected).then(|| {
            structural(
                field,
                format!(
                    "ppm {} does not equal exact integer rate {expected}",
                    self.ppm
                ),
            )
        })
    }
}

/// Exact signed rate (for a paired delta) plus fixed-point presentation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExactSignedRate {
    pub numerator: i64,
    pub denominator: u64,
    pub ppm: i64,
}

impl ExactSignedRate {
    fn validate(
        &self,
        field: &'static str,
        expected_numerator: i64,
        expected_denominator: u64,
    ) -> Option<DeployedQualityValidationError> {
        if self.numerator != expected_numerator || self.denominator != expected_denominator {
            return Some(structural(
                field,
                format!(
                    "fraction {}/{} does not equal paired count delta {expected_numerator}/{expected_denominator}",
                    self.numerator, self.denominator
                ),
            ));
        }
        if self.denominator == 0 {
            return Some(structural(field, "denominator is zero"));
        }
        let expected = ((i128::from(self.numerator) * RATE_SCALE_PPM as i128)
            / i128::from(self.denominator)) as i64;
        (self.ppm != expected).then(|| {
            structural(
                field,
                format!(
                    "ppm {} does not equal exact integer delta {expected}",
                    self.ppm
                ),
            )
        })
    }
}

/// Exact paired 2x2 outcome counts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PairedCounts {
    pub both_correct: u64,
    pub selector_only_correct: u64,
    pub comparator_only_correct: u64,
    pub neither_correct: u64,
}

impl PairedCounts {
    fn total(self) -> Option<u64> {
        self.both_correct
            .checked_add(self.selector_only_correct)?
            .checked_add(self.comparator_only_correct)?
            .checked_add(self.neither_correct)
    }

    fn selector_hits(self) -> Option<u64> {
        self.both_correct.checked_add(self.selector_only_correct)
    }

    fn comparator_hits(self) -> Option<u64> {
        self.both_correct.checked_add(self.comparator_only_correct)
    }

    fn signed_delta(self) -> Option<i64> {
        let selector_only = i64::try_from(self.selector_only_correct).ok()?;
        let comparator_only = i64::try_from(self.comparator_only_correct).ok()?;
        selector_only.checked_sub(comparator_only)
    }
}

/// Declared paired uncertainty in fixed-point ppm.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PairedInterval {
    pub method: String,
    pub confidence_ppm: u32,
    pub lower_delta_ppm: i64,
    pub estimate_delta_ppm: i64,
    pub upper_delta_ppm: i64,
}

impl PairedInterval {
    /// Construct the sole schema-1 paired interval from exact 2x2 counts.
    ///
    /// Let `X` be the per-position paired difference in `{-1, 0, +1}`.
    /// For `n` positions, discordant count `q`, and signed discordant
    /// difference `d`, the population-form standard error is
    /// `sqrt((q*n - d^2) / n^3)`. The outward-rounded margin is exactly
    /// `ceil((196/100) * 1_000_000 * SE)` ppm. All arithmetic is checked
    /// integer arithmetic; `None` means zero/overflowing/out-of-contract
    /// counts, never a silently approximated interval.
    pub fn from_counts(counts: PairedCounts) -> Option<Self> {
        let n = counts.total()?;
        if n == 0 || n > MAX_INTERVAL_POPULATION {
            return None;
        }
        let d = i128::from(counts.signed_delta()?);
        let q = u128::from(
            counts
                .selector_only_correct
                .checked_add(counts.comparator_only_correct)?,
        );
        let n128 = u128::from(n);
        let d_squared = d.unsigned_abs().checked_mul(d.unsigned_abs())?;
        let variance_numerator = q.checked_mul(n128)?.checked_sub(d_squared)?;

        // margin^2 >= (196^2 * 1_000_000^2 * variance_numerator)
        //             / (100^2 * n^3)
        let numerator = u128::from(196_u16)
            .checked_mul(u128::from(196_u16))?
            .checked_mul(1_000_000_u128.checked_mul(1_000_000)?)?
            .checked_mul(variance_numerator)?;
        let denominator = u128::from(100_u8)
            .checked_mul(u128::from(100_u8))?
            .checked_mul(n128.checked_mul(n128)?.checked_mul(n128)?)?;
        let squared_margin = div_ceil_u128(numerator, denominator)?;
        let margin = ceil_sqrt_u128(squared_margin);
        let estimate = ((d * RATE_SCALE_PPM as i128) / i128::from(n)) as i64;
        let margin = i64::try_from(margin).ok()?;
        Some(Self {
            method: PAIRED_INTERVAL_METHOD.to_string(),
            confidence_ppm: PAIRED_INTERVAL_CONFIDENCE_PPM,
            lower_delta_ppm: estimate.saturating_sub(margin).max(-1_000_000),
            estimate_delta_ppm: estimate,
            upper_delta_ppm: estimate.saturating_add(margin).min(1_000_000),
        })
    }

    fn validate(
        &self,
        field: &'static str,
        counts: PairedCounts,
    ) -> Option<DeployedQualityValidationError> {
        let Some(expected) = Self::from_counts(counts) else {
            return Some(structural(
                field,
                "paired counts cannot produce the schema-1 integer interval",
            ));
        };
        (self != &expected).then(|| {
            structural(
                field,
                format!(
                    "interval is not the recomputed {PAIRED_INTERVAL_METHOD}: report {self:?}, expected {expected:?}"
                ),
            )
        })
    }
}

fn div_ceil_u128(numerator: u128, denominator: u128) -> Option<u128> {
    if denominator == 0 {
        return None;
    }
    let quotient = numerator / denominator;
    let remainder = numerator % denominator;
    quotient.checked_add(u128::from(remainder != 0))
}

fn ceil_sqrt_u128(value: u128) -> u128 {
    if value < 2 {
        return value;
    }
    // Integer Newton iteration converges from above. Returning the converged
    // ceiling keeps the interval outward-rounded even for non-squares.
    let mut high = 1_u128 << ((128 - value.leading_zeros() as usize).div_ceil(2));
    loop {
        let next = (high + value / high) / 2;
        if next >= high {
            break;
        }
        high = next;
    }
    if high.checked_mul(high).is_some_and(|square| square < value) {
        high + 1
    } else {
        high
    }
}

/// Comparator definition and the exact positions shared with the selector.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ComparatorIdentity {
    pub id: String,
    pub version: String,
    pub definition_cid: String,
    pub positions_cid: String,
}

impl ComparatorIdentity {
    fn validate(&self) -> Option<DeployedQualityValidationError> {
        validate_nonempty("comparison.comparator.id", &self.id)
            .or_else(|| validate_nonempty("comparison.comparator.version", &self.version))
            .or_else(|| validate_cid("comparison.comparator.definition_cid", &self.definition_cid))
            .or_else(|| validate_cid("comparison.comparator.positions_cid", &self.positions_cid))
    }
}

/// One same-position paired comparison against the normative selector.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PairedComparison {
    pub comparator: ComparatorIdentity,
    pub counts: PairedCounts,
    pub selector_rate: ExactRate,
    pub comparator_rate: ExactRate,
    pub delta: ExactSignedRate,
    pub interval: PairedInterval,
}

impl PairedComparison {
    fn validate(
        &self,
        field: &'static str,
        evaluated: u64,
        positions_cid: &str,
    ) -> Option<DeployedQualityValidationError> {
        if let Some(error) = self.comparator.validate() {
            return Some(error);
        }
        if self.comparator.positions_cid != positions_cid {
            return Some(DeployedQualityValidationError::IdentityMismatch {
                field: "comparison.comparator.positions_cid",
                report: self.comparator.positions_cid.clone(),
                loaded: positions_cid.to_string(),
            });
        }
        let Some(total) = self.counts.total() else {
            return Some(structural(field, "paired counts overflow u64"));
        };
        if total != evaluated {
            return Some(structural(
                field,
                format!("paired counts total {total} does not equal evaluated {evaluated}"),
            ));
        }
        let Some(selector_hits) = self.counts.selector_hits() else {
            return Some(structural(field, "selector hit count overflows u64"));
        };
        let Some(comparator_hits) = self.counts.comparator_hits() else {
            return Some(structural(field, "comparator hit count overflows u64"));
        };
        if let Some(error) = self.selector_rate.validate(field, evaluated) {
            return Some(error);
        }
        if self.selector_rate.numerator != selector_hits {
            return Some(structural(
                field,
                "selector rate numerator does not equal paired selector hits",
            ));
        }
        if let Some(error) = self.comparator_rate.validate(field, evaluated) {
            return Some(error);
        }
        if self.comparator_rate.numerator != comparator_hits {
            return Some(structural(
                field,
                "comparator rate numerator does not equal paired comparator hits",
            ));
        }
        let selector_only = i64::try_from(self.counts.selector_only_correct)
            .map_err(|_| ())
            .ok();
        let comparator_only = i64::try_from(self.counts.comparator_only_correct)
            .map_err(|_| ())
            .ok();
        let (Some(selector_only), Some(comparator_only)) = (selector_only, comparator_only) else {
            return Some(structural(field, "paired discordant count exceeds i64"));
        };
        let expected_delta = selector_only - comparator_only;
        if let Some(error) = self.delta.validate(field, expected_delta, evaluated) {
            return Some(error);
        }
        self.interval.validate(field, self.counts)
    }
}

/// The two promotion comparisons and cross-surface reachability census.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct QualityMeasurements {
    pub versus_tla: PairedComparison,
    pub versus_sections_absent: PairedComparison,
    /// Same-position comparisons between the main runtime's internally
    /// lane-disabled winner and the independently emitted sections-absent
    /// graph. Production requires one check for every evaluated position.
    pub internal_base_control_checks: u64,
    pub internal_base_control_mismatches: u64,
    /// Combined external cross-surface plus internal absent-identity counts.
    pub cross_surface_checks: u64,
    pub cross_surface_mismatches: u64,
    /// CID of the durable raw parity evidence from which the two counts were
    /// reduced. Counts without this artifact identity are structurally
    /// unavailable rather than self-attesting.
    pub cross_surface_evidence_cid: String,
}

impl QualityMeasurements {
    fn validate(
        &self,
        evaluated: u64,
        positions_cid: &str,
    ) -> Option<DeployedQualityValidationError> {
        self.versus_tla
            .validate("measurements.versus_tla", evaluated, positions_cid)
            .or_else(|| {
                self.versus_sections_absent.validate(
                    "measurements.versus_sections_absent",
                    evaluated,
                    positions_cid,
                )
            })
            .or_else(|| {
                (self.versus_tla.selector_rate != self.versus_sections_absent.selector_rate).then(
                    || {
                        structural(
                            "measurements",
                            "selector rates differ across same-position comparisons",
                        )
                    },
                )
            })
            .or_else(|| {
                (self.internal_base_control_mismatches > self.internal_base_control_checks
                    || self.internal_base_control_checks > evaluated)
                    .then(|| {
                        structural(
                            "measurements.internal_base_control_checks",
                            "internal absent-identity counts exceed their evaluated bounds",
                        )
                    })
            })
            .or_else(|| {
                (self.cross_surface_checks < self.internal_base_control_checks
                    || self.cross_surface_mismatches < self.internal_base_control_mismatches)
                    .then(|| {
                        structural(
                            "measurements.cross_surface_checks",
                            "combined counts are below declared internal absent-identity counts",
                        )
                    })
            })
            .or_else(|| {
                (self.cross_surface_mismatches > self.cross_surface_checks).then(|| {
                    structural(
                        "measurements.cross_surface_mismatches",
                        "exceeds cross_surface_checks",
                    )
                })
            })
            .or_else(|| {
                validate_cid(
                    "measurements.cross_surface_evidence_cid",
                    &self.cross_surface_evidence_cid,
                )
            })
    }
}

/// Witness replay evidence for a deterministic report sample.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WitnessReplayEvidence {
    pub sample_cid: String,
    pub requested: u64,
    pub replayed: u64,
    pub failures: u64,
}

impl WitnessReplayEvidence {
    fn validate(&self, evaluated: u64) -> Option<DeployedQualityValidationError> {
        validate_cid("witness_replay.sample_cid", &self.sample_cid).or_else(|| {
            (self.replayed > self.requested
                || self.failures > self.replayed
                || self.requested > evaluated)
                .then(|| {
                    structural(
                        "witness_replay",
                        "requires failures <= replayed <= requested <= evaluated",
                    )
                })
        })
    }
}

/// Whether a declared falsifier had teeth.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum NegativeControlVerdict {
    Passed,
    Failed,
    Unavailable,
}

/// One content-bound negative control. A measured paired comparison is
/// required for production admission; research records may retain a failed or
/// unavailable control without upgrading it to PASS.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NegativeControlEvidence {
    pub id: String,
    pub identity_cid: String,
    pub verdict: NegativeControlVerdict,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub comparison: Option<PairedComparison>,
}

impl NegativeControlEvidence {
    fn validate(
        &self,
        evaluated: u64,
        positions_cid: &str,
    ) -> Option<DeployedQualityValidationError> {
        validate_nonempty("negative_controls.id", &self.id)
            .or_else(|| validate_cid("negative_controls.identity_cid", &self.identity_cid))
            .or_else(|| {
                self.comparison.as_ref().and_then(|comparison| {
                    comparison.validate("negative_controls.comparison", evaluated, positions_cid)
                })
            })
    }
}

/// Evaluation extent, verdict, and optional measured comparisons.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EvaluationEvidence {
    pub mode: EvaluationMode,
    pub population_size: u64,
    pub evaluated_positions: u64,
    pub verdict: QualityVerdict,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub measurements: Option<QualityMeasurements>,
}

impl EvaluationEvidence {
    fn validate(&self) -> Option<DeployedQualityValidationError> {
        if self.population_size == 0 {
            return Some(structural("evaluation.population_size", "is zero"));
        }
        if self.evaluated_positions > self.population_size {
            return Some(structural(
                "evaluation.evaluated_positions",
                "exceeds population_size",
            ));
        }
        match (&self.mode, &self.verdict) {
            (EvaluationMode::Sample, QualityVerdict::Estimate { decision }) => {
                if decision.trim().is_empty() {
                    return Some(structural("evaluation.verdict.decision", "is empty"));
                }
                if SampleDecisionKind::parse(decision).is_none() {
                    return Some(structural(
                        "evaluation.verdict.decision",
                        "must use the typed PROCEED, STOP, or INCONCLUSIVE vocabulary",
                    ));
                }
                if self.evaluated_positions == 0 || self.evaluated_positions >= self.population_size
                {
                    return Some(structural(
                        "evaluation",
                        "a sample requires 0 < evaluated_positions < population_size",
                    ));
                }
            }
            (EvaluationMode::Sample, QualityVerdict::Unavailable { reason })
            | (EvaluationMode::FullCensus, QualityVerdict::Unavailable { reason }) => {
                if reason.trim().is_empty() {
                    return Some(structural("evaluation.verdict.reason", "is empty"));
                }
            }
            (EvaluationMode::FullCensus, QualityVerdict::Pass)
            | (EvaluationMode::FullCensus, QualityVerdict::Fail { .. }) => {
                if self.evaluated_positions != self.population_size {
                    return Some(structural(
                        "evaluation",
                        "a measured full census must evaluate the entire population",
                    ));
                }
                if let QualityVerdict::Fail { reason } = &self.verdict {
                    if reason.trim().is_empty() {
                        return Some(structural("evaluation.verdict.reason", "is empty"));
                    }
                }
            }
            (EvaluationMode::Sample, _) => {
                return Some(structural(
                    "evaluation.verdict",
                    "sample mode may only be estimate or unavailable",
                ));
            }
            (EvaluationMode::FullCensus, QualityVerdict::Estimate { .. }) => {
                return Some(structural(
                    "evaluation.verdict",
                    "full-census mode may not carry an estimate verdict",
                ));
            }
        }
        let unavailable = matches!(self.verdict, QualityVerdict::Unavailable { .. });
        if unavailable && self.measurements.is_some() {
            return Some(structural(
                "evaluation.measurements",
                "must be absent when evidence is unavailable",
            ));
        }
        if !unavailable && self.measurements.is_none() {
            return Some(structural(
                "evaluation.measurements",
                "is required for measured or estimated evidence",
            ));
        }
        None
    }
}

/// One deterministic, identity-bound quality record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DeployedQualityReport {
    pub schema: u32,
    pub profile: QualityProfileIdentity,
    pub bindings: DeployedQualityBindings,
    pub evaluation: EvaluationEvidence,
    pub witness_replay: WitnessReplayEvidence,
    pub negative_controls: Vec<NegativeControlEvidence>,
}

impl DeployedQualityReport {
    /// Structural validation for explicit research use. This preserves
    /// off-serving selectors and negative outcomes as readable evidence but
    /// does not authorize production.
    pub fn validate_for_research(&self) -> Option<DeployedQualityValidationError> {
        if self.schema != DEPLOYED_QUALITY_REPORT_SCHEMA {
            return Some(DeployedQualityValidationError::UnsupportedSchema {
                found: self.schema,
                supported: DEPLOYED_QUALITY_REPORT_SCHEMA,
            });
        }
        if let Some(error) = validate_nonempty("profile.id", &self.profile.id)
            .or_else(|| validate_nonempty("profile.execution_scope", &self.profile.execution_scope))
            .or_else(|| self.bindings.validate_shape())
            .or_else(|| self.evaluation.validate())
        {
            return Some(error);
        }
        if let Some(measurements) = self.evaluation.measurements.as_ref() {
            if let Some(error) = measurements.validate(
                self.evaluation.evaluated_positions,
                &self.bindings.partition.evaluated_positions_cid,
            ) {
                return Some(error);
            }
        }
        if let Some(error) = self
            .witness_replay
            .validate(self.evaluation.evaluated_positions)
        {
            return Some(error);
        }
        let mut previous: Option<&str> = None;
        for control in &self.negative_controls {
            if previous.is_some_and(|id| id >= control.id.as_str()) {
                return Some(structural(
                    "negative_controls",
                    "must be strictly sorted by id with no duplicates",
                ));
            }
            if let Some(error) = control.validate(
                self.evaluation.evaluated_positions,
                &self.bindings.partition.evaluated_positions_cid,
            ) {
                return Some(error);
            }
            previous = Some(&control.id);
        }
        None
    }

    /// Fail-closed production validation. In addition to structural checks,
    /// production requires the normative profile/selector, independently
    /// loaded identities matching field-for-field, a full-census PASS, both
    /// paired lower-bound criteria, zero cross-surface/witness failures, and a
    /// non-degenerate label-shuffled control.
    pub fn validate_for_production(
        &self,
        loaded: &DeployedQualityBindings,
    ) -> Option<DeployedQualityValidationError> {
        if let Some(error) = self.validate_for_research() {
            return Some(error);
        }
        if self.profile.id != DEPLOYED_QUALITY_PROFILE_ID
            || self.profile.version != DEPLOYED_QUALITY_PROFILE_VERSION
            || self.profile.execution_scope != NORMATIVE_EXECUTION_SCOPE
        {
            return Some(DeployedQualityValidationError::NotProductionAdmissible {
                reason: format!(
                    "profile must be {DEPLOYED_QUALITY_PROFILE_ID}@{DEPLOYED_QUALITY_PROFILE_VERSION} in scope {NORMATIVE_EXECUTION_SCOPE}"
                ),
            });
        }
        if let Some(error) = self.bindings.selector.validate_normative() {
            return Some(error);
        }
        if let Some(error) = loaded.validate_shape() {
            return Some(error);
        }
        if let Some(error) = loaded.selector.validate_normative() {
            return Some(error);
        }
        macro_rules! require_binding {
            ($field:literal, $report:expr, $loaded:expr) => {
                if $report != $loaded {
                    return Some(DeployedQualityValidationError::IdentityMismatch {
                        field: $field,
                        report: format!("{:?}", $report),
                        loaded: format!("{:?}", $loaded),
                    });
                }
            };
        }
        require_binding!("selector", &self.bindings.selector, &loaded.selector);
        require_binding!("graph", &self.bindings.graph, &loaded.graph);
        require_binding!(
            "teacher_artifact",
            &self.bindings.teacher_artifact,
            &loaded.teacher_artifact
        );
        require_binding!("corpus", &self.bindings.corpus, &loaded.corpus);
        require_binding!("partition", &self.bindings.partition, &loaded.partition);
        require_binding!("tokenizer", &self.bindings.tokenizer, &loaded.tokenizer);
        require_binding!("compiler", &self.bindings.compiler, &loaded.compiler);
        require_binding!(
            "serving_configuration_cid",
            &self.bindings.serving_configuration_cid,
            &loaded.serving_configuration_cid
        );
        require_binding!(
            "active_sections",
            &self.bindings.active_sections,
            &loaded.active_sections
        );
        require_binding!("decode", &self.bindings.decode, &loaded.decode);
        require_binding!("seed", &self.bindings.seed, &loaded.seed);

        if self.bindings.decode.mode != DecodeMode::GreedyTop1 {
            return Some(DeployedQualityValidationError::NotProductionAdmissible {
                reason: "promotion evidence must measure greedy-top1 selection".to_string(),
            });
        }
        if self.bindings.seed.mode != PositionSelectionMode::FullPopulation {
            return Some(DeployedQualityValidationError::NotProductionAdmissible {
                reason: "production evidence must bind the full-population position selection"
                    .to_string(),
            });
        }
        if !self.bindings.active_sections.contains("SKMX")
            || !self.bindings.active_sections.contains("PSIB")
        {
            return Some(DeployedQualityValidationError::NotProductionAdmissible {
                reason: "RF-31 production evidence requires active SKMX and PSIB sections"
                    .to_string(),
            });
        }
        if self.evaluation.mode != EvaluationMode::FullCensus
            || !matches!(self.evaluation.verdict, QualityVerdict::Pass)
        {
            return Some(DeployedQualityValidationError::NotProductionAdmissible {
                reason: "only a full-census PASS may authorize production".to_string(),
            });
        }
        let Some(measurements) = self.evaluation.measurements.as_ref() else {
            return Some(DeployedQualityValidationError::NotProductionAdmissible {
                reason: "full-census PASS has no measurements".to_string(),
            });
        };
        if measurements.versus_tla.comparator.id != TLA_COMPARATOR_ID
            || measurements.versus_tla.comparator.version != TLA_COMPARATOR_VERSION
        {
            return Some(DeployedQualityValidationError::NotProductionAdmissible {
                reason: format!(
                    "TLA comparison identifies {:?}@{:?}, expected {:?}@{:?}",
                    measurements.versus_tla.comparator.id,
                    measurements.versus_tla.comparator.version,
                    TLA_COMPARATOR_ID,
                    TLA_COMPARATOR_VERSION
                ),
            });
        }
        if measurements.versus_tla.comparator.positions_cid
            != self.bindings.partition.evaluated_positions_cid
        {
            return Some(DeployedQualityValidationError::NotProductionAdmissible {
                reason: "TLA comparison does not use the bound evaluated-position partition"
                    .to_string(),
            });
        }
        if measurements.versus_sections_absent.comparator.id != SECTIONS_ABSENT_COMPARATOR_ID
            || measurements.versus_sections_absent.comparator.version
                != SECTIONS_ABSENT_COMPARATOR_VERSION
        {
            return Some(DeployedQualityValidationError::NotProductionAdmissible {
                reason: format!(
                    "lane comparison identifies {:?}@{:?}, expected {:?}@{:?}",
                    measurements.versus_sections_absent.comparator.id,
                    measurements.versus_sections_absent.comparator.version,
                    SECTIONS_ABSENT_COMPARATOR_ID,
                    SECTIONS_ABSENT_COMPARATOR_VERSION
                ),
            });
        }
        if measurements.versus_sections_absent.comparator.positions_cid
            != self.bindings.partition.evaluated_positions_cid
        {
            return Some(DeployedQualityValidationError::NotProductionAdmissible {
                reason:
                    "sections-absent comparison does not use the bound evaluated-position partition"
                        .to_string(),
            });
        }
        if measurements.versus_tla.interval.lower_delta_ppm < 0 {
            return Some(DeployedQualityValidationError::NotProductionAdmissible {
                reason: format!(
                    "TLA paired lower bound {} ppm is below zero",
                    measurements.versus_tla.interval.lower_delta_ppm
                ),
            });
        }
        if measurements.versus_sections_absent.interval.lower_delta_ppm < RF31_MIN_LANE_DELTA_PPM {
            return Some(DeployedQualityValidationError::NotProductionAdmissible {
                reason: format!(
                    "RF-31 paired lower bound {} ppm is below the frozen {} ppm floor",
                    measurements.versus_sections_absent.interval.lower_delta_ppm,
                    RF31_MIN_LANE_DELTA_PPM
                ),
            });
        }
        if measurements.internal_base_control_checks != self.evaluation.evaluated_positions
            || measurements.internal_base_control_mismatches != 0
        {
            return Some(DeployedQualityValidationError::NotProductionAdmissible {
                reason: format!(
                    "internal sections-absent identity evidence is incomplete or divergent: {} checks for {} evaluated positions, {} mismatches",
                    measurements.internal_base_control_checks,
                    self.evaluation.evaluated_positions,
                    measurements.internal_base_control_mismatches
                ),
            });
        }
        if measurements.cross_surface_checks == 0 || measurements.cross_surface_mismatches != 0 {
            return Some(DeployedQualityValidationError::NotProductionAdmissible {
                reason: format!(
                    "cross-surface evidence is vacuous or divergent: {} checks, {} mismatches",
                    measurements.cross_surface_checks, measurements.cross_surface_mismatches
                ),
            });
        }
        if self.witness_replay.replayed == 0 || self.witness_replay.failures != 0 {
            return Some(DeployedQualityValidationError::NotProductionAdmissible {
                reason: format!(
                    "witness replay is vacuous or failed: {} replayed, {} failures",
                    self.witness_replay.replayed, self.witness_replay.failures
                ),
            });
        }
        if self.negative_controls.is_empty() {
            return Some(DeployedQualityValidationError::NotProductionAdmissible {
                reason: "no negative control evidence is present".to_string(),
            });
        }
        let mut label_shuffled = None;
        for control in &self.negative_controls {
            if control.verdict != NegativeControlVerdict::Passed {
                return Some(DeployedQualityValidationError::NotProductionAdmissible {
                    reason: format!(
                        "negative control {:?} did not pass ({:?})",
                        control.id, control.verdict
                    ),
                });
            }
            let Some(comparison) = control.comparison.as_ref() else {
                return Some(DeployedQualityValidationError::NotProductionAdmissible {
                    reason: format!("negative control {:?} has no paired evidence", control.id),
                });
            };
            if control.id == LABEL_SHUFFLED_CONTROL_ID {
                label_shuffled = Some(comparison);
            }
        }
        let Some(label_shuffled) = label_shuffled else {
            return Some(DeployedQualityValidationError::NotProductionAdmissible {
                reason: format!(
                    "required negative control {LABEL_SHUFFLED_CONTROL_ID:?} is absent"
                ),
            });
        };
        if label_shuffled.comparator != measurements.versus_sections_absent.comparator {
            return Some(DeployedQualityValidationError::NotProductionAdmissible {
                reason: "label-shuffled control is not paired against the exact validated sections-absent comparator"
                    .to_string(),
            });
        }
        if label_shuffled.delta.numerator > 0 {
            return Some(DeployedQualityValidationError::NotProductionAdmissible {
                reason: format!(
                    "label-shuffled control has a positive paired effect ({}/{})",
                    label_shuffled.delta.numerator, label_shuffled.delta.denominator
                ),
            });
        }
        None
    }

    /// Stable pretty JSON bytes (field order is declaration order, all maps are
    /// excluded from the schema, and validated variable rows have a canonical
    /// strict order). A final newline is part of the byte contract.
    pub fn deterministic_json_bytes(&self) -> Result<Vec<u8>, SourceUnavailable> {
        let mut bytes = serde_json::to_vec_pretty(self).map_err(|error| {
            binding_error(
                "deployed_quality_report",
                format!("JSON serialization failed: {error}"),
            )
        })?;
        bytes.push(b'\n');
        Ok(bytes)
    }

    /// BLAKE3 CID of [`Self::deterministic_json_bytes`].
    pub fn cid(&self) -> Result<String, SourceUnavailable> {
        let bytes = self.deterministic_json_bytes()?;
        Ok(format!("blake3:{}", blake3::hash(&bytes).to_hex()))
    }
}

/// Explicit research-only parse result. Legacy/off-serving JSON is retained as
/// an untyped document with its declared schema; it can never flow through
/// [`DeployedQualityReport::validate_for_production`].
#[derive(Debug, Clone, PartialEq)]
pub enum ResearchDeployedQualityReport {
    Current(Box<DeployedQualityReport>),
    LegacyUnavailable {
        declared_schema: Option<u64>,
        document: serde_json::Value,
    },
}

/// Parse quality evidence for historical/research inspection. Production must
/// deserialize [`DeployedQualityReport`] directly and then call
/// [`DeployedQualityReport::validate_for_production`].
pub fn parse_deployed_quality_for_research(
    bytes: &[u8],
) -> Result<ResearchDeployedQualityReport, SourceUnavailable> {
    let document: serde_json::Value = serde_json::from_slice(bytes).map_err(|error| {
        binding_error("research_report", format!("invalid JSON document: {error}"))
    })?;
    let declared_schema = document.get("schema").and_then(serde_json::Value::as_u64);
    if declared_schema == Some(u64::from(DEPLOYED_QUALITY_REPORT_SCHEMA))
        && document.get("profile").is_some()
        && document.get("bindings").is_some()
    {
        let report = serde_json::from_value(document).map_err(|error| {
            binding_error(
                "research_report",
                format!("current schema decode failed: {error}"),
            )
        })?;
        Ok(ResearchDeployedQualityReport::Current(Box::new(report)))
    } else {
        Ok(ResearchDeployedQualityReport::LegacyUnavailable {
            declared_schema,
            document,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn typed_sample_decision_vocabulary_is_fail_closed() {
        assert_eq!(
            SampleDecisionKind::parse("PROCEED: full census next"),
            Some(SampleDecisionKind::Proceed)
        );
        assert_eq!(
            SampleDecisionKind::parse("STOP: upper bound misses"),
            Some(SampleDecisionKind::Stop)
        );
        assert_eq!(
            SampleDecisionKind::parse("INCONCLUSIVE: extend to 18000"),
            Some(SampleDecisionKind::Inconclusive)
        );
        assert_eq!(SampleDecisionKind::parse("STOP:"), None);
        assert_eq!(SampleDecisionKind::parse("looks promising"), None);
    }

    #[test]
    fn story_sample_rejects_zero_and_preserves_nested_prefixes() {
        let stories: Vec<u32> = (0..40).map(|position| position / 10).collect();
        let population: Vec<u64> = (0..40).collect();
        assert!(deterministic_story_sample(&stories, &population, 0).is_err());
        let small = deterministic_story_sample(&stories, &population, 8).expect("small");
        let large = deterministic_story_sample(&stories, &population, 20).expect("large");
        assert!(small
            .iter()
            .all(|position| large.binary_search(position).is_ok()));
        assert!(small.windows(2).all(|pair| pair[0] < pair[1]));
        for story in 0..4 {
            assert_eq!(
                small
                    .iter()
                    .filter(|&&position| stories[position as usize] == story)
                    .count(),
                2
            );
        }

        let noncontiguous_stories = [0, 1, 0, 1];
        assert!(
            deterministic_story_sample(&noncontiguous_stories, &[0, 1, 2, 3], 2).is_err(),
            "a story that reappears in a disjoint corpus segment must fail closed"
        );
    }

    #[test]
    fn adapter_source_cids_are_canonical_lowercase_and_nonzero() {
        let lower = format!("blake3:{}", "a1".repeat(32));
        let upper = format!("blake3:{}", "A1".repeat(32));
        let zero = format!("blake3:{}", "0".repeat(64));
        assert!(is_canonical_blake3_cid(&lower));
        assert!(!is_canonical_blake3_cid(&upper));
        assert!(is_canonical_blake3_cid(&zero));
        assert!(cid_is_zero(&zero));
    }

    #[test]
    fn deployed_quality_failures_use_the_sanctioned_field_specific_boundary() {
        let sample_error = deterministic_story_sample(&[], &[], 0)
            .expect_err("a zero-sized sample must fail closed");
        assert_eq!(
            sample_error.reason,
            "deployed-quality binding evaluated_positions: sample size is zero"
        );

        let parse_error = parse_deployed_quality_for_research(b"{")
            .expect_err("malformed research JSON must fail closed");
        assert!(
            parse_error
                .reason
                .starts_with("deployed-quality binding research_report: invalid JSON document:"),
            "{}",
            parse_error.reason
        );

        let _: fn(&DeployedQualityReport) -> Result<Vec<u8>, SourceUnavailable> =
            DeployedQualityReport::deterministic_json_bytes;
        let _: fn(&DeployedQualityReport) -> Result<String, SourceUnavailable> =
            DeployedQualityReport::cid;
        let _: fn(&[u8]) -> Result<ResearchDeployedQualityReport, SourceUnavailable> =
            parse_deployed_quality_for_research;
    }
}
