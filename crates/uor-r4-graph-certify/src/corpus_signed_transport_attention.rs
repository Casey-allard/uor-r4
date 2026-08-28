//! Fail-closed pre-geometry feasibility certificate for issue #986.
//!
//! This module deliberately has no placement, diffusion, Gate 0, calibration,
//! label, or route-map API. It records the observed corpus/codec/split state,
//! revalidates the live exact SpiralCore control, and stops at the absent
//! complete same-frame lexical `O(x)` binding.

use serde::Serialize;
use uor_r4_core::spiralcore_operator::{
    cl06_finite_composition_table, validate_spiralcore_v63_operator, CANONICAL_SIX_PRIME_VALUES,
    CHART_TRANSPORT_STATUS, CL06_FINITE_COMPOSITION_DOMAIN,
    CL06_FINITE_COMPOSITION_KAPPA_REFERENCE, CL06_FINITE_GROUP_ORDER, OPERATOR_SEMANTIC_STATUS,
    SIX_PRIME_CHART_DOMAIN, SPIRALCORE_OPERATOR_DOMAIN, SPIRALCORE_OPERATOR_KAPPA_REFERENCE,
    SPIRALCORE_V63_REFERENCE_SHA256,
};

pub const PREFLIGHT_SCHEMA: u32 = 1;
pub const PREFLIGHT_DOMAIN: &str = "uor-r4.corpus-signed-transport-pregeometry/1";
pub const FROZEN_POLICY: &str = "CorpusSignedTransportV1";
pub const UNAVAILABLE_FRAME_OR_POPULATION: &str = "UNAVAILABLE_FRAME_OR_POPULATION";
pub const TERMINAL_PRECEDENCE: [&str; 5] = [
    "INVALID_CONTRACT",
    UNAVAILABLE_FRAME_OR_POPULATION,
    "PROCEED_TO_I1_WITH_CORPUS_SIGNED_TRANSPORT_ATTENTION",
    "RETAIN_GEOMETRY_AS_TRANSPORT_ADVANCE_TABLE_VALUE_QUALIFIER",
    "REDESIGN_CORPUS_OBJECTIVE_OR_PLACEMENT",
];

const REQUIRED_CALIBRATION_PAIRS: u32 = 16;
const REQUIRED_CALIBRATION_DECISIONS: u32 = 32;
const REQUIRED_SEALED_TEST_PAIRS: u32 = 32;
const REQUIRED_SEALED_TEST_DECISIONS: u32 = 64;
const LEXICAL_BINDING_STATUS: &str = "ABSENT_NO_VERIFIABLE_COMPLETE_SAME_FRAME_LEXICAL_O_X_SURFACE";

/// Label-free facts observed before any geometric work is allowed.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize)]
pub struct ObservedCorpusCodecSplitReadiness {
    pub audited_main_commit: String,
    pub raw_manifest_bytes_cid: Option<String>,
    pub declared_corpus_bytes_cid: Option<String>,
    pub reproduced_corpus_bytes_cid: Option<String>,
    pub codec_cid: Option<String>,
    pub split_commitment_cid: Option<String>,
    pub source_document_count: u64,
    pub construction_document_count: u64,
    pub held_out_document_count: u64,
    pub construction_lexical_route_count: u64,
    pub calibration_pair_count: u32,
    pub calibration_decision_count: u32,
    pub sealed_test_pair_count: u32,
    pub sealed_test_decision_count: u32,
    pub source_free_observation_corpus: bool,
    pub canonical_codec_reproduces: bool,
    pub document_cid_partition_verified: bool,
    pub anti_recall_disjointness_verified: bool,
    pub natural_candidate_rule_frozen: bool,
    pub pair_selection_key_frozen: bool,
}

/// Live exact-algebra finding, separated from the missing lexical binding.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LiveSpiralCorePrerequisite {
    pub reference_sha256: &'static str,
    pub operator_domain: &'static str,
    pub operator_kappa_reference: &'static str,
    pub observed_operator_kappa: Option<String>,
    pub composition_domain: &'static str,
    pub composition_kappa_reference: &'static str,
    pub observed_composition_kappa: Option<String>,
    pub finite_group_order: u64,
    pub composition_entries: u64,
    pub associativity_checks: u64,
    pub two_sided_inverses: u64,
    pub noncommuting_ordered_pairs: u64,
    pub exact_operator_and_table_reproduce: bool,
    pub six_prime_chart_domain: &'static str,
    pub canonical_six_prime_values: [u32; 6],
    pub chart_transport_status: &'static str,
    pub operator_semantic_status: &'static str,
    pub cross_chart_transport_established: bool,
    pub lexical_binding_status: &'static str,
    pub lexical_binding_manifest_cid: Option<String>,
    pub verified_bound_lexical_route_count: u64,
    pub complete_same_frame_lexical_operator_binding: bool,
    pub compiler_query_frame_identity_verified: bool,
}

/// Canonical, label-free prerequisite certificate for the current repository
/// surface. Its only emitted terminal is the frozen pre-sealed failure branch.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CorpusSignedTransportPregeometryCertificate {
    pub schema: u32,
    pub domain: &'static str,
    pub policy: &'static str,
    pub terminal_precedence: [&'static str; 5],
    pub required_calibration_pairs: u32,
    pub required_calibration_decisions: u32,
    pub required_sealed_test_pairs: u32,
    pub required_sealed_test_decisions: u32,
    pub observed: ObservedCorpusCodecSplitReadiness,
    pub audited_main_commit_is_canonical: bool,
    pub raw_manifest_bytes_cid_is_canonical: bool,
    pub corpus_bytes_cid_reproduces: bool,
    pub source_partition_counts_reproduce: bool,
    pub codec_cid_is_canonical: bool,
    pub split_commitment_is_canonical: bool,
    pub exact_population_available: bool,
    pub spiralcore: LiveSpiralCorePrerequisite,
    pub placement_started: bool,
    pub diffusion_started: bool,
    pub gate0_started: bool,
    pub calibration_started: bool,
    pub sealed_label_path_started: bool,
    pub terminal: &'static str,
    pub unavailable_reasons: Vec<&'static str>,
}

impl CorpusSignedTransportPregeometryCertificate {
    /// Schema-fixed JSON with declaration-order fields and deterministic
    /// escaping. All identity-bearing inputs are also reported with their
    /// canonical-syntax verdicts.
    pub fn canonical_json_bytes(&self) -> Vec<u8> {
        let mut json = JsonObject::new();
        json.u64("schema", u64::from(self.schema));
        json.string("domain", self.domain);
        json.string("policy", self.policy);
        json.strings("terminal_precedence", &self.terminal_precedence);
        json.u64(
            "required_calibration_pairs",
            u64::from(self.required_calibration_pairs),
        );
        json.u64(
            "required_calibration_decisions",
            u64::from(self.required_calibration_decisions),
        );
        json.u64(
            "required_sealed_test_pairs",
            u64::from(self.required_sealed_test_pairs),
        );
        json.u64(
            "required_sealed_test_decisions",
            u64::from(self.required_sealed_test_decisions),
        );
        json.raw("observed", &observed_json(&self.observed));
        json.boolean(
            "audited_main_commit_is_canonical",
            self.audited_main_commit_is_canonical,
        );
        json.boolean(
            "raw_manifest_bytes_cid_is_canonical",
            self.raw_manifest_bytes_cid_is_canonical,
        );
        json.boolean(
            "corpus_bytes_cid_reproduces",
            self.corpus_bytes_cid_reproduces,
        );
        json.boolean(
            "source_partition_counts_reproduce",
            self.source_partition_counts_reproduce,
        );
        json.boolean("codec_cid_is_canonical", self.codec_cid_is_canonical);
        json.boolean(
            "split_commitment_is_canonical",
            self.split_commitment_is_canonical,
        );
        json.boolean(
            "exact_population_available",
            self.exact_population_available,
        );
        json.raw("spiralcore", &spiralcore_json(&self.spiralcore));
        json.boolean("placement_started", self.placement_started);
        json.boolean("diffusion_started", self.diffusion_started);
        json.boolean("gate0_started", self.gate0_started);
        json.boolean("calibration_started", self.calibration_started);
        json.boolean("sealed_label_path_started", self.sealed_label_path_started);
        json.string("terminal", self.terminal);
        json.strings("unavailable_reasons", &self.unavailable_reasons);
        json.finish().into_bytes()
    }

    pub fn cid(&self) -> String {
        format!(
            "blake3:{}",
            blake3::hash(&self.canonical_json_bytes()).to_hex()
        )
    }
}

/// Revalidate the current prerequisite surface and emit its fail-closed
/// certificate. Summary data cannot establish a route-by-route `O(x)` map, so
/// the current version always records zero verified bindings rather than
/// accepting a caller assertion or synthesizing a mapping.
pub fn certify_corpus_signed_transport_pregeometry(
    observed: ObservedCorpusCodecSplitReadiness,
) -> CorpusSignedTransportPregeometryCertificate {
    let audited_main_commit_is_canonical = is_git_revision(&observed.audited_main_commit);
    let raw_manifest_bytes_cid_is_canonical = observed
        .raw_manifest_bytes_cid
        .as_deref()
        .is_some_and(is_blake3_cid);
    let corpus_bytes_cid_reproduces = observed
        .declared_corpus_bytes_cid
        .as_deref()
        .is_some_and(is_blake3_cid)
        && observed.declared_corpus_bytes_cid == observed.reproduced_corpus_bytes_cid;
    let source_partition_counts_reproduce = observed.source_document_count > 0
        && observed
            .construction_document_count
            .checked_add(observed.held_out_document_count)
            == Some(observed.source_document_count);
    let codec_cid_is_canonical = observed.codec_cid.as_deref().is_some_and(is_blake3_cid);
    let split_commitment_is_canonical = observed
        .split_commitment_cid
        .as_deref()
        .is_some_and(is_blake3_cid);
    let exact_population_available = audited_main_commit_is_canonical
        && raw_manifest_bytes_cid_is_canonical
        && corpus_bytes_cid_reproduces
        && source_partition_counts_reproduce
        && codec_cid_is_canonical
        && split_commitment_is_canonical
        && observed.construction_lexical_route_count > 0
        && observed.calibration_pair_count == REQUIRED_CALIBRATION_PAIRS
        && observed.calibration_decision_count == REQUIRED_CALIBRATION_DECISIONS
        && observed.sealed_test_pair_count == REQUIRED_SEALED_TEST_PAIRS
        && observed.sealed_test_decision_count == REQUIRED_SEALED_TEST_DECISIONS
        && observed.source_free_observation_corpus
        && observed.canonical_codec_reproduces
        && observed.document_cid_partition_verified
        && observed.anti_recall_disjointness_verified
        && observed.natural_candidate_rule_frozen
        && observed.pair_selection_key_frozen;

    let operator = validate_spiralcore_v63_operator().ok();
    let composition = cl06_finite_composition_table()
        .and_then(|table| table.validate())
        .ok();
    let exact_operator_and_table_reproduce = operator.as_ref().is_some_and(|value| {
        value.operator_kappa == SPIRALCORE_OPERATOR_KAPPA_REFERENCE
            && value.finite_group_size == CL06_FINITE_GROUP_ORDER
    }) && composition.as_ref().is_some_and(|value| {
        value.operator_kappa == SPIRALCORE_OPERATOR_KAPPA_REFERENCE
            && value.composition_kappa == CL06_FINITE_COMPOSITION_KAPPA_REFERENCE
            && value.unique_states == CL06_FINITE_GROUP_ORDER
            && value.composition_entries == CL06_FINITE_GROUP_ORDER * CL06_FINITE_GROUP_ORDER
            && value.associativity_checks
                == CL06_FINITE_GROUP_ORDER * CL06_FINITE_GROUP_ORDER * CL06_FINITE_GROUP_ORDER
            && value.two_sided_inverses == CL06_FINITE_GROUP_ORDER
            && value.noncommuting_ordered_pairs > 0
    });
    let spiralcore = LiveSpiralCorePrerequisite {
        reference_sha256: SPIRALCORE_V63_REFERENCE_SHA256,
        operator_domain: SPIRALCORE_OPERATOR_DOMAIN,
        operator_kappa_reference: SPIRALCORE_OPERATOR_KAPPA_REFERENCE,
        observed_operator_kappa: operator.as_ref().map(|value| value.operator_kappa.clone()),
        composition_domain: CL06_FINITE_COMPOSITION_DOMAIN,
        composition_kappa_reference: CL06_FINITE_COMPOSITION_KAPPA_REFERENCE,
        observed_composition_kappa: composition
            .as_ref()
            .map(|value| value.composition_kappa.clone()),
        finite_group_order: operator
            .as_ref()
            .map_or(0, |value| value.finite_group_size as u64),
        composition_entries: composition
            .as_ref()
            .map_or(0, |value| value.composition_entries as u64),
        associativity_checks: composition
            .as_ref()
            .map_or(0, |value| value.associativity_checks as u64),
        two_sided_inverses: composition
            .as_ref()
            .map_or(0, |value| value.two_sided_inverses as u64),
        noncommuting_ordered_pairs: composition
            .as_ref()
            .map_or(0, |value| value.noncommuting_ordered_pairs as u64),
        exact_operator_and_table_reproduce,
        six_prime_chart_domain: SIX_PRIME_CHART_DOMAIN,
        canonical_six_prime_values: CANONICAL_SIX_PRIME_VALUES,
        chart_transport_status: CHART_TRANSPORT_STATUS,
        operator_semantic_status: OPERATOR_SEMANTIC_STATUS,
        cross_chart_transport_established: CHART_TRANSPORT_STATUS == "ESTABLISHED",
        lexical_binding_status: LEXICAL_BINDING_STATUS,
        lexical_binding_manifest_cid: None,
        verified_bound_lexical_route_count: 0,
        complete_same_frame_lexical_operator_binding: false,
        compiler_query_frame_identity_verified: false,
    };

    let mut unavailable_reasons = Vec::new();
    if !audited_main_commit_is_canonical {
        unavailable_reasons.push("AUDITED_MAIN_COMMIT_UNBOUND");
    }
    if !exact_population_available {
        unavailable_reasons.push("EXACT_CORPUS_CODEC_SPLIT_POPULATION_UNAVAILABLE");
    }
    if !exact_operator_and_table_reproduce {
        unavailable_reasons.push("EXACT_SPIRALCORE_OPERATOR_UNAVAILABLE");
    }
    unavailable_reasons.push("COMPLETE_SAME_FRAME_LEXICAL_O_X_BINDING_UNAVAILABLE");

    CorpusSignedTransportPregeometryCertificate {
        schema: PREFLIGHT_SCHEMA,
        domain: PREFLIGHT_DOMAIN,
        policy: FROZEN_POLICY,
        terminal_precedence: TERMINAL_PRECEDENCE,
        required_calibration_pairs: REQUIRED_CALIBRATION_PAIRS,
        required_calibration_decisions: REQUIRED_CALIBRATION_DECISIONS,
        required_sealed_test_pairs: REQUIRED_SEALED_TEST_PAIRS,
        required_sealed_test_decisions: REQUIRED_SEALED_TEST_DECISIONS,
        observed,
        audited_main_commit_is_canonical,
        raw_manifest_bytes_cid_is_canonical,
        corpus_bytes_cid_reproduces,
        source_partition_counts_reproduce,
        codec_cid_is_canonical,
        split_commitment_is_canonical,
        exact_population_available,
        spiralcore,
        placement_started: false,
        diffusion_started: false,
        gate0_started: false,
        calibration_started: false,
        sealed_label_path_started: false,
        terminal: UNAVAILABLE_FRAME_OR_POPULATION,
        unavailable_reasons,
    }
}

fn is_blake3_cid(value: &str) -> bool {
    value.strip_prefix("blake3:").is_some_and(|hex| {
        hex.len() == 64
            && hex
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    })
}

fn is_git_revision(value: &str) -> bool {
    matches!(value.len(), 40 | 64)
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn observed_json(value: &ObservedCorpusCodecSplitReadiness) -> String {
    let mut json = JsonObject::new();
    json.string("audited_main_commit", &value.audited_main_commit);
    json.optional_string(
        "raw_manifest_bytes_cid",
        value.raw_manifest_bytes_cid.as_deref(),
    );
    json.optional_string(
        "declared_corpus_bytes_cid",
        value.declared_corpus_bytes_cid.as_deref(),
    );
    json.optional_string(
        "reproduced_corpus_bytes_cid",
        value.reproduced_corpus_bytes_cid.as_deref(),
    );
    json.optional_string("codec_cid", value.codec_cid.as_deref());
    json.optional_string(
        "split_commitment_cid",
        value.split_commitment_cid.as_deref(),
    );
    json.u64("source_document_count", value.source_document_count);
    json.u64(
        "construction_document_count",
        value.construction_document_count,
    );
    json.u64("held_out_document_count", value.held_out_document_count);
    json.u64(
        "construction_lexical_route_count",
        value.construction_lexical_route_count,
    );
    json.u64(
        "calibration_pair_count",
        u64::from(value.calibration_pair_count),
    );
    json.u64(
        "calibration_decision_count",
        u64::from(value.calibration_decision_count),
    );
    json.u64(
        "sealed_test_pair_count",
        u64::from(value.sealed_test_pair_count),
    );
    json.u64(
        "sealed_test_decision_count",
        u64::from(value.sealed_test_decision_count),
    );
    json.boolean(
        "source_free_observation_corpus",
        value.source_free_observation_corpus,
    );
    json.boolean(
        "canonical_codec_reproduces",
        value.canonical_codec_reproduces,
    );
    json.boolean(
        "document_cid_partition_verified",
        value.document_cid_partition_verified,
    );
    json.boolean(
        "anti_recall_disjointness_verified",
        value.anti_recall_disjointness_verified,
    );
    json.boolean(
        "natural_candidate_rule_frozen",
        value.natural_candidate_rule_frozen,
    );
    json.boolean("pair_selection_key_frozen", value.pair_selection_key_frozen);
    json.finish()
}

fn spiralcore_json(value: &LiveSpiralCorePrerequisite) -> String {
    let mut json = JsonObject::new();
    json.string("reference_sha256", value.reference_sha256);
    json.string("operator_domain", value.operator_domain);
    json.string("operator_kappa_reference", value.operator_kappa_reference);
    json.optional_string(
        "observed_operator_kappa",
        value.observed_operator_kappa.as_deref(),
    );
    json.string("composition_domain", value.composition_domain);
    json.string(
        "composition_kappa_reference",
        value.composition_kappa_reference,
    );
    json.optional_string(
        "observed_composition_kappa",
        value.observed_composition_kappa.as_deref(),
    );
    json.u64("finite_group_order", value.finite_group_order);
    json.u64("composition_entries", value.composition_entries);
    json.u64("associativity_checks", value.associativity_checks);
    json.u64("two_sided_inverses", value.two_sided_inverses);
    json.u64(
        "noncommuting_ordered_pairs",
        value.noncommuting_ordered_pairs,
    );
    json.boolean(
        "exact_operator_and_table_reproduce",
        value.exact_operator_and_table_reproduce,
    );
    json.string("six_prime_chart_domain", value.six_prime_chart_domain);
    json.u32s(
        "canonical_six_prime_values",
        &value.canonical_six_prime_values,
    );
    json.string("chart_transport_status", value.chart_transport_status);
    json.string("operator_semantic_status", value.operator_semantic_status);
    json.boolean(
        "cross_chart_transport_established",
        value.cross_chart_transport_established,
    );
    json.string("lexical_binding_status", value.lexical_binding_status);
    json.optional_string(
        "lexical_binding_manifest_cid",
        value.lexical_binding_manifest_cid.as_deref(),
    );
    json.u64(
        "verified_bound_lexical_route_count",
        value.verified_bound_lexical_route_count,
    );
    json.boolean(
        "complete_same_frame_lexical_operator_binding",
        value.complete_same_frame_lexical_operator_binding,
    );
    json.boolean(
        "compiler_query_frame_identity_verified",
        value.compiler_query_frame_identity_verified,
    );
    json.finish()
}

struct JsonObject {
    text: String,
    first: bool,
}

impl JsonObject {
    fn new() -> Self {
        Self {
            text: "{".to_owned(),
            first: true,
        }
    }

    fn key(&mut self, key: &str) {
        if !self.first {
            self.text.push(',');
        }
        self.first = false;
        push_json_string(&mut self.text, key);
        self.text.push(':');
    }

    fn string(&mut self, key: &str, value: &str) {
        self.key(key);
        push_json_string(&mut self.text, value);
    }

    fn optional_string(&mut self, key: &str, value: Option<&str>) {
        self.key(key);
        if let Some(value) = value {
            push_json_string(&mut self.text, value);
        } else {
            self.text.push_str("null");
        }
    }

    fn boolean(&mut self, key: &str, value: bool) {
        self.key(key);
        self.text.push_str(if value { "true" } else { "false" });
    }

    fn u64(&mut self, key: &str, value: u64) {
        self.key(key);
        self.text.push_str(&value.to_string());
    }

    fn strings(&mut self, key: &str, values: &[&str]) {
        self.key(key);
        self.text.push('[');
        for (index, value) in values.iter().enumerate() {
            if index > 0 {
                self.text.push(',');
            }
            push_json_string(&mut self.text, value);
        }
        self.text.push(']');
    }

    fn u32s(&mut self, key: &str, values: &[u32]) {
        self.key(key);
        self.text.push('[');
        for (index, value) in values.iter().enumerate() {
            if index > 0 {
                self.text.push(',');
            }
            self.text.push_str(&value.to_string());
        }
        self.text.push(']');
    }

    fn raw(&mut self, key: &str, value: &str) {
        self.key(key);
        self.text.push_str(value);
    }

    fn finish(mut self) -> String {
        self.text.push('}');
        self.text
    }
}

fn push_json_string(output: &mut String, value: &str) {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    output.push('"');
    for character in value.chars() {
        match character {
            '"' => output.push_str("\\\""),
            '\\' => output.push_str("\\\\"),
            '\u{08}' => output.push_str("\\b"),
            '\u{0c}' => output.push_str("\\f"),
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            '\t' => output.push_str("\\t"),
            character if character <= '\u{1f}' => {
                let value = character as usize;
                output.push_str("\\u00");
                output.push(char::from(HEX[(value >> 4) & 0x0f]));
                output.push(char::from(HEX[value & 0x0f]));
            }
            character => output.push(character),
        }
    }
    output.push('"');
}
