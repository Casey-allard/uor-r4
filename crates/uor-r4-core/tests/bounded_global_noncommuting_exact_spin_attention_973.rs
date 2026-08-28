//! Frozen noncommuting bounded-global exact-spin successor probe for #973.
//!
//! This remains one bounded synthetic, construction-bound, same-address-reuse
//! witness over fixed #953 support. It does not establish semantic placement,
//! distinct-address sharing, a neighbor field, broad global attention, or any
//! downstream correctness, reasoning, performance, or release claim.

use std::collections::BTreeSet;
use std::sync::atomic::{AtomicUsize, Ordering};

use serde::{Deserialize, Serialize};

use uor_r4_core::bounded_global_exact_spin_attention::{
    BoundedGlobalExactSpinCandidateEvidence, BoundedGlobalExactSpinCost,
    BoundedGlobalExactSpinForbiddenReads, BoundedGlobalNoncommutingExactSpinR4V2,
    BoundedGlobalNoncommutingPopulationAudit, MatchedBoundedGlobalExactSpinPrediction,
    MatchedBoundedGlobalNoncommutingPairPrediction, BOUNDED_GLOBAL_EXACT_SPIN_CANDIDATES,
    BOUNDED_GLOBAL_EXACT_SPIN_CLASSES, BOUNDED_GLOBAL_EXACT_SPIN_ENTRIES,
    BOUNDED_GLOBAL_EXACT_SPIN_REUSE_HITS, MAX_BOUNDED_GLOBAL_EXACT_SPIN_OPERATOR_BYTES,
    MAX_BOUNDED_GLOBAL_EXACT_SPIN_QUERY_BYTES,
};
use uor_r4_core::canonical_lexical_ingestion::{
    canonical_lexical_piece_bytes, CanonicalRouteArtifact, GlobalExactSpinSnapshotView,
};
use uor_r4_core::prime_route_geometric_attention::H4S3AngularShell;
use uor_r4_core::source_free_table::{
    d3_is_held_out, BackoffOrder, ContinuationStop, MultiscaleCountRadiusR4V1, SourceDocument,
    SourceFreeTable, MAX_CONTINUATION_UNITS,
};

const OBSERVED_PROMPT: &[u8] = b"The bounded global code is";
const ID_53_SNAPSHOT: [&[u8]; 4] = [b"Lena", b"Lena", b"helix", b"prism"];
const ID_54_SNAPSHOT: [&[u8]; 4] = [b"Lena", b"helix", b"Lena", b"prism"];
const DUPLICATE_POOL: [&[u8]; 11] = [
    b".", b"Lena", b"Pavel", b"The", b"bound", b"bounded", b"class", b"code", b"global", b"is",
    b"the",
];
const TARGET_COMMITMENT: &str =
    "blake3:b7340c776e005c32316de793b332e3f218b1fad757c77044b0fa2e70fc308354";
const TARGET_PREIMAGE_BYTES: usize = 210;
const POSITIVE_TERMINAL: &str =
    "RETAIN_BOUNDED_GLOBAL_NONCOMMUTING_EXACT_SPIN_ATTENTION_CONTINUE_CORPUS_INDUCTION";

static TARGET_PREIMAGE_LOADS: AtomicUsize = AtomicUsize::new(0);

fn construction_documents() -> Vec<SourceDocument> {
    vec![
        SourceDocument::new(
            "51",
            b"Lena bound the helix class.\n\nThe bounded global code is bronze.".to_vec(),
        ),
        SourceDocument::new(
            "52",
            b"Pavel bound the prism class.\n\nThe bounded global code is teal.".to_vec(),
        ),
    ]
}

fn snapshot(units: [&[u8]; 4]) -> Vec<Vec<u8>> {
    units.into_iter().map(<[u8]>::to_vec).collect()
}

fn decoded(table: &SourceFreeTable, token: u32) -> Vec<u8> {
    table.decode_tokens(&[token]).unwrap()
}

fn decoded_support(table: &SourceFreeTable, tokens: &[u32]) -> Vec<Vec<u8>> {
    tokens.iter().map(|token| decoded(table, *token)).collect()
}

fn contains_bytes(haystack: &[u8], needle: &[u8]) -> bool {
    !needle.is_empty()
        && haystack
            .windows(needle.len())
            .any(|window| window == needle)
}

fn direct_cid(bytes: &[u8]) -> String {
    format!("blake3:{}", blake3::hash(bytes).to_hex())
}

fn source_function<'a>(source: &'a str, name: &str) -> &'a str {
    let needle = format!("fn {name}(");
    let start = source.find(&needle).unwrap();
    let opening = start + source[start..].find('{').unwrap();
    let mut depth = 0_u32;
    for (offset, byte) in source.as_bytes()[opening..].iter().enumerate() {
        match byte {
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return &source[start..=opening + offset];
                }
            }
            _ => {}
        }
    }
    panic!("unterminated source function {name}")
}

fn exact_score_firewall_accepts(source: &str) -> bool {
    const FORBIDDEN: [&str; 25] = [
        "target",
        "future_unit",
        "teacher",
        "provider",
        "corpus",
        "partition",
        "payload",
        "address",
        "prime",
        "digest",
        "class_kappa",
        "ordinal",
        "spin_sector",
        "adjacent",
        "candidate_token",
        "candidate_hex",
        "construction_identity",
        "prompt_identity",
        "table_identity",
        "base_evidence",
        "lower_state",
        "declared_work",
        "global_summary",
        "hierarchy_h4",
        "result_cid",
    ];
    FORBIDDEN.iter().all(|needle| !source.contains(needle))
}

fn assert_typed_exact_score_firewall() {
    let source = include_str!("../src/bounded_global_exact_spin_attention.rs");
    let kernel = [
        "candidate_relative_exact_cost",
        "exact_cost",
        "select_exact_costs",
        "unique_exact_cost_winner",
    ]
    .into_iter()
    .map(|name| source_function(source, name))
    .collect::<Vec<_>>()
    .join("\n");
    assert!(exact_score_firewall_accepts(&kernel));

    let injected_forbidden_read = format!("{kernel}\nlet payload_score_reads = 1_u64;");
    assert!(!exact_score_firewall_accepts(&injected_forbidden_read));
}

fn reveal_target_preimage_once() -> &'static [u8] {
    assert_eq!(TARGET_PREIMAGE_LOADS.fetch_add(1, Ordering::SeqCst), 0);
    br#"{"schema":1,"domain":"uor-r4.bounded-global-noncommuting-exact-spin-decoded-targets/1","rows":[{"document_id":"53","continuation_hex":"207465616c2e"},{"document_id":"54","continuation_hex":"2062726f6e7a652e"}]}"#
}

fn parse_committed_targets(bytes: &[u8]) -> Result<DecodedTargets, String> {
    if bytes.len() != TARGET_PREIMAGE_BYTES || direct_cid(bytes) != TARGET_COMMITMENT {
        return Err("decoded-target commitment mismatch".to_owned());
    }
    let targets = serde_json::from_slice::<DecodedTargets>(bytes)
        .map_err(|error| format!("decoded-target JSON: {error}"))?;
    let replay = serde_json::to_vec(&targets)
        .map_err(|error| format!("decoded-target replay JSON: {error}"))?;
    if replay != bytes
        || targets.schema != 1
        || targets.domain != "uor-r4.bounded-global-noncommuting-exact-spin-decoded-targets/1"
        || targets.rows.len() != 2
        || targets.rows[0].document_id != "53"
        || targets.rows[1].document_id != "54"
        || targets
            .rows
            .iter()
            .any(|target| hex::decode(&target.continuation_hex).is_err())
    {
        return Err("decoded-target preimage is noncanonical or malformed".to_owned());
    }
    Ok(targets)
}

fn cost(
    angular_shell: H4S3AngularShell,
    fiber_distance_q29: u64,
    torsion_distance_q29: u64,
) -> BoundedGlobalExactSpinCost {
    BoundedGlobalExactSpinCost {
        angular_shell,
        fiber_distance_q29,
        torsion_distance_q29,
    }
}

fn tamper_bound_value(bytes: &[u8], needle: &[u8]) -> Vec<u8> {
    let offset = bytes
        .windows(needle.len())
        .position(|window| window == needle)
        .unwrap();
    let mut tampered = bytes.to_vec();
    tampered[offset + needle.len() / 2] ^= 1;
    tampered
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct DecodedTargets {
    schema: u32,
    domain: String,
    rows: Vec<DecodedTarget>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct DecodedTarget {
    document_id: String,
    continuation_hex: String,
}

#[derive(Serialize)]
struct TargetFreeCensus<'a> {
    schema: u32,
    domain: &'static str,
    decoded_target_commitment: &'static str,
    table_cid: String,
    base_overlay_cid: String,
    operator_cid: String,
    codec_kappa: String,
    vocabulary_kappa: String,
    route_manifest_kappa: String,
    spin_map_kappa: String,
    chart_profile_kappa: String,
    grammar_kappa: String,
    routing_policy_kappa: String,
    population_policy_kappa: String,
    h4_root_table_kappa: String,
    h4_multiplication_table_kappa: String,
    base_lower_artifact_manifest_kappa: String,
    snapshot_artifact_manifest_kappas: [String; 2],
    population_audit_cid: String,
    population_audit: &'a BoundedGlobalNoncommutingPopulationAudit,
    snapshots: [&'a GlobalExactSpinSnapshotView; 2],
    pair: &'a MatchedBoundedGlobalNoncommutingPairPrediction,
    held_out_document_ids: [&'static str; 2],
    prompt_cid: String,
    target_preimage_loads_observed: usize,
}

#[derive(Serialize)]
struct DecodedSmoke<'a> {
    schema: u32,
    domain: &'static str,
    operator_cid: String,
    decoded_target_commitment: &'static str,
    target_preimage_loads_observed: usize,
    rows: Vec<DecodedCase<'a>>,
    real_correct: u32,
    identity_disabled_correct: u32,
    class_operator_permuted_correct: u32,
    support_reversed_real_correct: u32,
    support_mismatches: u32,
    work_mismatches: u32,
    period_and_eos_terminations: u32,
    terminal: &'static str,
}

#[derive(Serialize)]
struct DecodedCase<'a> {
    document_id: &'static str,
    target_hex: String,
    continuation: &'a uor_r4_core::bounded_global_exact_spin_attention::MatchedBoundedGlobalExactSpinContinuation,
}

fn assert_snapshot_view(
    view: &GlobalExactSpinSnapshotView,
    expected: [&[u8]; 4],
    duplicate: &[u8],
) {
    assert_eq!(view.entries.len(), BOUNDED_GLOBAL_EXACT_SPIN_ENTRIES);
    assert_eq!(
        view.entries
            .iter()
            .map(|entry| entry.ordinal)
            .collect::<Vec<_>>(),
        vec![0, 1, 2, 3]
    );
    assert_eq!(
        view.entries
            .iter()
            .map(|entry| entry.payload_bytes.as_slice())
            .collect::<Vec<_>>(),
        expected
    );
    assert_eq!(view.global_epoch, view.snapshot_kappa);
    assert!(view.source_artifact_manifest_kappa.starts_with("blake3:"));
    assert!(view.snapshot_summary_kappa.starts_with("blake3:"));
    assert!(view.global_root_kappa.starts_with("blake3:"));
    assert!(view.global_exact_chain_kappa.starts_with("blake3:"));

    let repeated = view
        .entries
        .iter()
        .filter(|entry| entry.payload_bytes == duplicate)
        .collect::<Vec<_>>();
    assert_eq!(repeated.len(), 2);
    assert_ne!(repeated[0].ordinal, repeated[1].ordinal);
    assert_ne!(repeated[0].entry_kappa, repeated[1].entry_kappa);
    assert_eq!(repeated[0].address_index, repeated[1].address_index);
    assert_eq!(repeated[0].address_kappa, repeated[1].address_kappa);
    assert_eq!(repeated[0].payload_cid, repeated[1].payload_cid);
    assert_eq!(repeated[0].payload_bytes, repeated[1].payload_bytes);
    assert_eq!(repeated[0].spin, repeated[1].spin);
    assert_eq!(
        repeated[0].shared_class_kappa,
        repeated[1].shared_class_kappa
    );
    assert_eq!(
        view.entries
            .iter()
            .map(|entry| entry.shared_class_kappa.as_str())
            .collect::<BTreeSet<_>>()
            .len(),
        BOUNDED_GLOBAL_EXACT_SPIN_CLASSES
    );
}

fn candidate_for_anchor<'a>(
    prediction: &'a MatchedBoundedGlobalExactSpinPrediction,
    anchor: &[u8],
) -> &'a BoundedGlobalExactSpinCandidateEvidence {
    let anchor_hex = hex::encode(anchor);
    prediction
        .candidate_evidence
        .iter()
        .find(|candidate| candidate.prototype_anchor_hex == anchor_hex)
        .unwrap()
}

fn assert_candidate_result_permutation(prediction: &MatchedBoundedGlobalExactSpinPrediction) {
    let mut real = prediction
        .candidate_evidence
        .iter()
        .map(|candidate| candidate.real_class_result_cid.clone())
        .collect::<Vec<_>>();
    let mut permuted = prediction
        .candidate_evidence
        .iter()
        .map(|candidate| candidate.permuted_class_result_cid.clone())
        .collect::<Vec<_>>();
    real.sort();
    permuted.sort();
    assert_eq!(real, permuted);
}

fn assert_common_prediction(
    table: &SourceFreeTable,
    prediction: &MatchedBoundedGlobalExactSpinPrediction,
) {
    assert_eq!(prediction.local.order, BackoffOrder::Trigram);
    assert_eq!(prediction.local.max_count, 1);
    assert!(prediction.local.geometry_reachable);
    assert_eq!(
        decoded_support(table, &prediction.local.max_count_tie_tokens),
        vec![b" bronze".to_vec(), b" teal".to_vec()]
    );
    assert_eq!(
        prediction.local.baseline_support_tokens,
        prediction.local.geometric_support_tokens
    );
    assert_eq!(
        prediction.local.max_count_tie_tokens,
        prediction.local.baseline_support_tokens
    );
    assert_eq!(
        prediction.local.baseline_work,
        prediction.local.geometric_work
    );
    assert_eq!(decoded(table, prediction.local.geometric_token), b" bronze");
    assert_eq!(
        prediction.snapshot_entries.len(),
        BOUNDED_GLOBAL_EXACT_SPIN_ENTRIES
    );
    assert_eq!(
        prediction.fold_steps.len(),
        BOUNDED_GLOBAL_EXACT_SPIN_ENTRIES
    );
    assert_eq!(
        prediction.class_evaluations.len(),
        BOUNDED_GLOBAL_EXACT_SPIN_CLASSES
    );
    assert_eq!(
        prediction.candidate_evidence.len(),
        BOUNDED_GLOBAL_EXACT_SPIN_CANDIDATES
    );
    assert!(prediction.support_reversal_invariant);
    assert_eq!(
        prediction.support_reversed_real_token,
        prediction.real.token
    );
    assert!(prediction.coherent_relabel_equivariant);
    assert!(prediction.support_matched);
    assert!(prediction.work_matched);
    assert_eq!(prediction.operator_abstention, None);
    assert_eq!(
        prediction.forbidden_reads,
        BoundedGlobalExactSpinForbiddenReads::default()
    );
    assert_eq!(prediction.real.work, prediction.identity_disabled.work);
    assert_eq!(
        prediction.real.work,
        prediction.class_operator_permuted.work
    );
    assert_eq!(
        prediction.real.support_tokens,
        prediction.identity_disabled.support_tokens
    );
    assert_eq!(
        prediction.real.support_tokens,
        prediction.class_operator_permuted.support_tokens
    );
    assert!(prediction.real.unique_minimum.is_some());
    assert!(prediction.class_operator_permuted.unique_minimum.is_some());
    assert_eq!(prediction.identity_disabled.unique_minimum, None);
    assert_eq!(prediction.identity_disabled.minimum_cost, None);
    assert_eq!(
        prediction.identity_disabled.token,
        prediction.local.geometric_token
    );

    let work = prediction.real.work;
    assert_eq!(work.snapshot_entry_reads, 4);
    assert_eq!(work.exact_class_comparisons, 4);
    assert_eq!(work.unique_class_evaluations, 3);
    assert_eq!(work.class_reuse_hits, BOUNDED_GLOBAL_EXACT_SPIN_REUSE_HITS);
    assert_eq!(work.class_result_applications, 4);
    assert_eq!(work.h4_product_table_reads, 10);
    assert_eq!(work.h4_inverse_table_reads, 6);
    assert_eq!(work.phase_additions, 20);
    assert_eq!(work.phase_distance_reads, 12);
    assert_eq!(work.angular_shell_reads, 6);
    assert_eq!(work.candidate_class_lookups, 2);
    assert_eq!(work.cost_comparisons, 2);
    assert_eq!(work.final_choice_operations, 1);

    assert_eq!(
        prediction
            .class_evaluations
            .iter()
            .map(|class| class.evaluation_count)
            .sum::<u64>(),
        3
    );
    assert_eq!(
        prediction
            .class_evaluations
            .iter()
            .map(|class| class.reference_count)
            .sum::<u64>(),
        4
    );
    assert_eq!(
        prediction
            .class_evaluations
            .iter()
            .map(|class| class.reuse_count)
            .sum::<u64>(),
        BOUNDED_GLOBAL_EXACT_SPIN_REUSE_HITS
    );
    assert!(prediction.class_evaluations.iter().all(|class| {
        class.cold_recomputation_equal && class.result_cid == class.cold_result_cid
    }));
    let repeated = prediction
        .class_evaluations
        .iter()
        .find(|class| class.reference_count == 2)
        .unwrap();
    assert_eq!(repeated.reference_entry_kappas.len(), 2);
    assert_ne!(
        repeated.reference_entry_kappas[0],
        repeated.reference_entry_kappas[1]
    );
    assert_eq!(repeated.evaluation_count, 1);
    assert_eq!(repeated.reuse_count, 1);

    let helix = prediction
        .snapshot_entries
        .iter()
        .find(|entry| entry.payload_hex == hex::encode(b"helix"))
        .unwrap();
    let prism = prediction
        .snapshot_entries
        .iter()
        .find(|entry| entry.payload_hex == hex::encode(b"prism"))
        .unwrap();
    assert_ne!(helix.shared_class_kappa, prism.shared_class_kappa);
    assert_eq!(helix.diagnostic_sector, prism.diagnostic_sector);
    assert!(prediction.snapshot_entries.iter().all(|entry| entry
        .mapped_state
        .h4_coordinate
        .scaled_zphi_quaternion
        .iter()
        .all(|coordinate| coordinate[1] == 0)));

    for candidate in &prediction.candidate_evidence {
        assert_eq!(candidate.count, 1);
        assert_eq!(
            candidate.real_ranking_cost,
            Some(candidate.real_measured_cost)
        );
        assert_eq!(candidate.identity_disabled_ranking_cost, None);
        assert_eq!(
            candidate.permuted_ranking_cost,
            Some(candidate.permuted_measured_cost)
        );
        assert!(candidate.real_class_result_cid.starts_with("blake3:"));
        assert!(candidate.permuted_class_result_cid.starts_with("blake3:"));
        assert!(candidate.candidate_state_kappa.starts_with("blake3:"));
    }
    assert_candidate_result_permutation(prediction);
}

fn assert_population_audit(
    operator: &BoundedGlobalNoncommutingExactSpinR4V2,
    audit: &BoundedGlobalNoncommutingPopulationAudit,
) {
    assert_eq!(audit.schema, 1);
    assert_eq!(
        audit.domain,
        "uor-r4.bounded-global-noncommuting-population-audit/1"
    );
    assert_eq!(
        audit.population_policy_kappa,
        operator.population_policy_kappa()
    );
    assert_eq!(
        audit
            .duplicate_pool_hex
            .iter()
            .map(|value| hex::decode(value).unwrap())
            .collect::<Vec<_>>(),
        DUPLICATE_POOL
            .iter()
            .map(|value| value.to_vec())
            .collect::<Vec<_>>()
    );
    assert_eq!(audit.rows_examined.len(), 2);
    assert_eq!(
        hex::decode(&audit.rows_examined[0].duplicate_hex).unwrap(),
        b"."
    );
    assert!(!audit.rows_examined[0].direct_noncommutation);
    assert_eq!(audit.rows_examined[0].selected_pair_indices, None);
    assert_eq!(
        audit.rows_examined[0]
            .duplicate_state
            .h4_coordinate
            .scaled_zphi_quaternion,
        [[2, 0], [0, 0], [0, 0], [0, 0]]
    );
    assert_eq!(
        hex::decode(&audit.rows_examined[1].duplicate_hex).unwrap(),
        b"Lena"
    );
    assert!(audit.rows_examined[1].direct_noncommutation);
    assert_eq!(audit.rows_examined[1].unique_permutations, 12);
    assert_eq!(audit.rows_examined[1].selected_pair_indices, Some([0, 2]));
    assert_eq!(audit.selected_duplicate_hex, hex::encode(b"Lena"));
    assert_eq!(audit.selected_pair_indices, [0, 2]);
    assert_eq!(audit.left_snapshot_hex, ID_53_SNAPSHOT.map(hex::encode));
    assert_eq!(audit.right_snapshot_hex, ID_54_SNAPSHOT.map(hex::encode));
    assert!(audit.one_transposition);
    assert_eq!(audit.transposed_ordinals, [1, 2]);
    assert!(audit.noncommutation.products_distinct);
    assert_eq!(
        hex::decode(&audit.noncommutation.left_operand_hex).unwrap(),
        b"Lena"
    );
    assert_eq!(
        hex::decode(&audit.noncommutation.right_operand_hex).unwrap(),
        b"helix"
    );
    assert_eq!(
        audit
            .noncommutation
            .left_operand
            .h4_coordinate
            .scaled_zphi_quaternion,
        [[0, 0], [2, 0], [0, 0], [0, 0]]
    );
    assert_eq!(
        audit
            .noncommutation
            .right_operand
            .h4_coordinate
            .scaled_zphi_quaternion,
        [[1, 0], [1, 0], [1, 0], [1, 0]]
    );
    assert_eq!(
        audit
            .noncommutation
            .left_then_right
            .h4_coordinate
            .scaled_zphi_quaternion,
        [[-1, 0], [1, 0], [-1, 0], [1, 0]]
    );
    assert_eq!(
        audit
            .noncommutation
            .right_then_left
            .h4_coordinate
            .scaled_zphi_quaternion,
        [[-1, 0], [1, 0], [1, 0], [-1, 0]]
    );
    assert_ne!(
        audit.noncommutation.left_then_right,
        audit.noncommutation.right_then_left
    );
    assert_eq!(
        audit.noncommutation.left_then_right.fiber_q29,
        audit.noncommutation.right_then_left.fiber_q29
    );
    assert_eq!(
        audit.noncommutation.left_then_right.torsion_q29,
        audit.noncommutation.right_then_left.torsion_q29
    );
    assert!(audit.distinct_nonidentity_folds);
    assert!(audit.complete_phase_totals_equal);
    assert_ne!(audit.left_fold, audit.right_fold);
    assert_eq!(
        audit.left_fold.h4_coordinate.scaled_zphi_quaternion,
        [[-1, 0], [-1, 0], [-1, 0], [-1, 0]]
    );
    assert_eq!(
        audit.right_fold.h4_coordinate.scaled_zphi_quaternion,
        [[-1, 0], [-1, 0], [1, 0], [1, 0]]
    );
    for fold in [audit.left_fold, audit.right_fold] {
        assert_eq!(fold.fiber_q29, 102_410_010);
        assert_eq!(fold.torsion_q29, -9_745_662);
    }

    let left_helix = audit
        .left_candidate_costs
        .iter()
        .find(|row| row.prototype_anchor_hex == hex::encode(b"helix"))
        .unwrap();
    let left_prism = audit
        .left_candidate_costs
        .iter()
        .find(|row| row.prototype_anchor_hex == hex::encode(b"prism"))
        .unwrap();
    let right_helix = audit
        .right_candidate_costs
        .iter()
        .find(|row| row.prototype_anchor_hex == hex::encode(b"helix"))
        .unwrap();
    let right_prism = audit
        .right_candidate_costs
        .iter()
        .find(|row| row.prototype_anchor_hex == hex::encode(b"prism"))
        .unwrap();
    assert_eq!(
        left_helix.cost,
        cost(H4S3AngularShell::Antipodal, 61_239_177, 5_831_083)
    );
    assert_eq!(
        left_prism.cost,
        cost(H4S3AngularShell::Degrees120, 55_205_017, 5_262_467)
    );
    assert_eq!(
        right_helix.cost,
        cost(H4S3AngularShell::Orthogonal, 61_239_177, 5_831_083)
    );
    assert_eq!(
        right_prism.cost,
        cost(H4S3AngularShell::Degrees120, 55_205_017, 5_262_467)
    );
    assert_eq!(audit.left_winner_anchor_hex, hex::encode(b"prism"));
    assert_eq!(audit.right_winner_anchor_hex, hex::encode(b"helix"));
    assert!(audit.incompatible_unique_winners);
}

#[test]
fn noncommuting_global_relation_is_load_bearing_before_the_committed_decode() {
    assert_eq!(TARGET_PREIMAGE_LOADS.load(Ordering::SeqCst), 0);
    assert_typed_exact_score_firewall();

    let construction = construction_documents();
    assert_eq!(construction.len(), 2);
    assert!(construction
        .iter()
        .all(|document| !d3_is_held_out(&document.id)));
    assert!(d3_is_held_out("53"));
    assert!(d3_is_held_out("54"));
    let held_out = [
        SourceDocument::new("53", OBSERVED_PROMPT.to_vec()),
        SourceDocument::new("54", OBSERVED_PROMPT.to_vec()),
    ];
    for document in &held_out {
        assert!(construction.iter().all(|source| {
            source.id != document.id && source.text_cid() != document.text_cid()
        }));
        assert_eq!(document.text, OBSERVED_PROMPT);
        assert!(!contains_bytes(&document.text, b" bronze"));
        assert!(!contains_bytes(&document.text, b" teal"));
    }
    assert_eq!(held_out[0].text_cid(), held_out[1].text_cid());
    assert!(OBSERVED_PROMPT.len() <= MAX_BOUNDED_GLOBAL_EXACT_SPIN_QUERY_BYTES);
    let prompt_pieces = canonical_lexical_piece_bytes(OBSERVED_PROMPT).unwrap();
    assert!(!prompt_pieces
        .iter()
        .any(|piece| piece == b"bronze" || piece == b"teal"));

    let left_snapshot = snapshot(ID_53_SNAPSHOT);
    let right_snapshot = snapshot(ID_54_SNAPSHOT);
    let mut left_multiset = left_snapshot.clone();
    let mut right_multiset = right_snapshot.clone();
    left_multiset.sort();
    right_multiset.sort();
    assert_eq!(left_multiset, right_multiset);
    assert_ne!(left_snapshot, right_snapshot);
    for units in [&left_snapshot, &right_snapshot] {
        assert_eq!(units.len(), BOUNDED_GLOBAL_EXACT_SPIN_ENTRIES);
        assert!(units
            .iter()
            .all(|unit| unit != b"bronze" && unit != b"teal"));
    }

    let table = SourceFreeTable::compile(&construction).unwrap();
    let base_overlay = MultiscaleCountRadiusR4V1::compile(&table).unwrap();
    let operator =
        BoundedGlobalNoncommutingExactSpinR4V2::compile(&table, &base_overlay, &construction)
            .unwrap();
    let table_bytes = table.to_bytes();
    let base_overlay_bytes = base_overlay.to_bytes();
    let operator_bytes = operator.to_bytes().unwrap();
    assert!(operator_bytes.len() <= MAX_BOUNDED_GLOBAL_EXACT_SPIN_OPERATOR_BYTES);
    let operator_wire: serde_json::Value = serde_json::from_slice(&operator_bytes[8..]).unwrap();
    assert!(operator_wire.get("held_out_document_ids").is_none());
    assert!(operator_wire.get("decoded_targets").is_none());
    assert!(operator_wire.get("decoded_target_commitment").is_none());
    assert!(!contains_bytes(
        &operator_bytes,
        TARGET_COMMITMENT.as_bytes()
    ));
    let operator_cid = operator.artifact_cid().unwrap();

    let table = SourceFreeTable::from_bytes(&table_bytes).unwrap();
    let base_overlay = MultiscaleCountRadiusR4V1::from_bytes(&table, &base_overlay_bytes).unwrap();
    let operator = BoundedGlobalNoncommutingExactSpinR4V2::from_bytes(
        &table,
        &base_overlay,
        &construction,
        &operator_bytes,
    )
    .unwrap();
    assert_eq!(table.to_bytes(), table_bytes);
    assert_eq!(base_overlay.to_bytes(), base_overlay_bytes);
    assert_eq!(operator.to_bytes().unwrap(), operator_bytes);
    assert_eq!(operator.artifact_cid().unwrap(), operator_cid);
    assert_eq!(operator.table_artifact_cid(), table.artifact_cid());
    assert_eq!(
        operator.base_overlay_artifact_cid(),
        base_overlay.artifact_cid()
    );

    let prototypes = operator.prototype_traces().unwrap();
    assert_eq!(prototypes.len(), BOUNDED_GLOBAL_EXACT_SPIN_CANDIDATES);
    assert_eq!(
        prototypes
            .iter()
            .map(|prototype| hex::decode(&prototype.candidate_hex).unwrap())
            .collect::<Vec<_>>(),
        vec![b" bronze".to_vec(), b" teal".to_vec()]
    );
    assert_eq!(
        prototypes
            .iter()
            .map(|prototype| hex::decode(&prototype.anchor_hex).unwrap())
            .collect::<Vec<_>>(),
        vec![b"helix".to_vec(), b"prism".to_vec()]
    );
    assert_ne!(
        prototypes[0].anchor_class_kappa,
        prototypes[1].anchor_class_kappa
    );

    let population_audit = operator.population_audit().unwrap();
    assert_population_audit(&operator, &population_audit);
    let population_audit_bytes = serde_json::to_vec(&population_audit).unwrap();
    let population_audit_cid = direct_cid(&population_audit_bytes);
    let population_audit_replay = operator.population_audit().unwrap();
    assert_eq!(population_audit_replay, population_audit);
    assert_eq!(
        serde_json::to_vec(&population_audit_replay).unwrap(),
        population_audit_bytes
    );

    let [base_carrier, left_carrier, right_carrier] = std::thread::scope(|scope| {
        [
            scope.spawn(|| {
                let artifact = operator.build_query_artifact(OBSERVED_PROMPT).unwrap();
                let bytes = artifact.canonical_bytes().unwrap();
                (artifact, bytes)
            }),
            scope.spawn(|| {
                let artifact = operator
                    .build_snapshot_artifact(OBSERVED_PROMPT, &left_snapshot)
                    .unwrap();
                let bytes = artifact.canonical_bytes().unwrap();
                (artifact, bytes)
            }),
            scope.spawn(|| {
                let artifact = operator
                    .build_snapshot_artifact(OBSERVED_PROMPT, &right_snapshot)
                    .unwrap();
                let bytes = artifact.canonical_bytes().unwrap();
                (artifact, bytes)
            }),
        ]
        .map(|handle| handle.join().unwrap())
    });
    let (base_artifact, base_artifact_bytes) = base_carrier;
    let (left_artifact, left_artifact_bytes) = left_carrier;
    let (right_artifact, right_artifact_bytes) = right_carrier;
    let base_manifest = base_artifact.manifest_kappa().to_owned();
    let left_manifest = left_artifact.manifest_kappa().to_owned();
    let right_manifest = right_artifact.manifest_kappa().to_owned();
    assert_ne!(base_manifest, left_manifest);
    assert_ne!(base_manifest, right_manifest);
    assert_ne!(left_manifest, right_manifest);
    let left_view = left_artifact.global_exact_spin_snapshot_view().unwrap();
    let right_view = right_artifact.global_exact_spin_snapshot_view().unwrap();
    assert_snapshot_view(&left_view, ID_53_SNAPSHOT, b"Lena");
    assert_snapshot_view(&right_view, ID_54_SNAPSHOT, b"Lena");
    assert_eq!(left_view.source_artifact_manifest_kappa, left_manifest);
    assert_eq!(right_view.source_artifact_manifest_kappa, right_manifest);
    assert_ne!(left_view.snapshot_kappa, right_view.snapshot_kappa);
    assert_ne!(left_view.global_root_kappa, right_view.global_root_kappa);

    let pair = operator
        .predict_pair_matched(
            &table,
            &base_overlay,
            &base_artifact,
            &left_artifact,
            &right_artifact,
            OBSERVED_PROMPT,
        )
        .unwrap();
    assert_eq!(pair.population_audit, population_audit);
    assert!(pair.exact_fold_distinct);
    assert!(pair.real_winners_incompatible);
    assert!(pair.permuted_winners_incompatible);
    assert!(pair.common_lower_artifact);
    assert!(pair.support_matched_between_cases);
    assert!(pair.work_matched_between_cases);
    assert_common_prediction(&table, &pair.left);
    assert_common_prediction(&table, &pair.right);
    assert_eq!(pair.left.local, pair.right.local);
    assert_eq!(pair.left.real.work, pair.right.real.work);
    assert_eq!(
        pair.left.real.support_tokens,
        pair.right.real.support_tokens
    );
    assert_eq!(pair.left.base_lower_artifact_manifest_kappa, base_manifest);
    assert_eq!(pair.right.base_lower_artifact_manifest_kappa, base_manifest);
    assert_eq!(
        pair.left.source_snapshot_artifact_manifest_kappa,
        left_manifest
    );
    assert_eq!(
        pair.right.source_snapshot_artifact_manifest_kappa,
        right_manifest
    );
    assert_ne!(pair.left.global_epoch, pair.right.global_epoch);
    assert_ne!(pair.left.global_root_kappa, pair.right.global_root_kappa);
    assert_eq!(pair.left.global_result, population_audit.left_fold);
    assert_eq!(pair.right.global_result, population_audit.right_fold);
    assert_ne!(pair.left.global_result, pair.right.global_result);
    assert_eq!(pair.left.operator_cid, operator_cid);
    assert_eq!(pair.right.operator_cid, operator_cid);
    assert_eq!(pair.left.spin_map_kappa, operator.spin_map_kappa());
    assert_eq!(pair.right.spin_map_kappa, operator.spin_map_kappa());
    assert_eq!(
        pair.left.chart_profile_kappa,
        operator.chart_profile_kappa()
    );
    assert_eq!(
        pair.right.chart_profile_kappa,
        operator.chart_profile_kappa()
    );

    let left_helix = candidate_for_anchor(&pair.left, b"helix");
    let left_prism = candidate_for_anchor(&pair.left, b"prism");
    let right_helix = candidate_for_anchor(&pair.right, b"helix");
    let right_prism = candidate_for_anchor(&pair.right, b"prism");
    assert_eq!(
        left_helix.real_measured_cost,
        cost(H4S3AngularShell::Antipodal, 61_239_177, 5_831_083)
    );
    assert_eq!(
        left_prism.real_measured_cost,
        cost(H4S3AngularShell::Degrees120, 55_205_017, 5_262_467)
    );
    assert_eq!(
        right_helix.real_measured_cost,
        cost(H4S3AngularShell::Orthogonal, 61_239_177, 5_831_083)
    );
    assert_eq!(
        right_prism.real_measured_cost,
        cost(H4S3AngularShell::Degrees120, 55_205_017, 5_262_467)
    );
    assert_eq!(
        pair.left.real.minimum_cost,
        Some(left_prism.real_measured_cost)
    );
    assert_eq!(
        pair.right.real.minimum_cost,
        Some(right_helix.real_measured_cost)
    );
    assert_eq!(decoded(&table, pair.left.real.token), b" teal");
    assert_eq!(decoded(&table, pair.right.real.token), b" bronze");
    assert_eq!(
        decoded(&table, pair.left.identity_disabled.token),
        b" bronze"
    );
    assert_eq!(
        decoded(&table, pair.right.identity_disabled.token),
        b" bronze"
    );
    assert_eq!(
        decoded(&table, pair.left.class_operator_permuted.token),
        b" bronze"
    );
    assert_eq!(
        decoded(&table, pair.right.class_operator_permuted.token),
        b" teal"
    );
    assert_ne!(pair.left.real.token, pair.right.real.token);
    assert_ne!(
        pair.left.class_operator_permuted.token,
        pair.right.class_operator_permuted.token
    );

    assert_eq!(TARGET_PREIMAGE_LOADS.load(Ordering::SeqCst), 0);
    let target_free_census = TargetFreeCensus {
        schema: 1,
        domain: "uor-r4.bounded-global-noncommuting-exact-spin-target-free-census/1",
        decoded_target_commitment: TARGET_COMMITMENT,
        table_cid: table.artifact_cid(),
        base_overlay_cid: base_overlay.artifact_cid(),
        operator_cid: operator_cid.clone(),
        codec_kappa: operator.codec_kappa().to_owned(),
        vocabulary_kappa: operator.vocabulary_kappa().to_owned(),
        route_manifest_kappa: operator.route_manifest_kappa().to_owned(),
        spin_map_kappa: operator.spin_map_kappa().to_owned(),
        chart_profile_kappa: operator.chart_profile_kappa().to_owned(),
        grammar_kappa: operator.grammar_kappa().to_owned(),
        routing_policy_kappa: operator.routing_policy_kappa().to_owned(),
        population_policy_kappa: operator.population_policy_kappa().to_owned(),
        h4_root_table_kappa: operator.h4_root_table_kappa().to_owned(),
        h4_multiplication_table_kappa: operator.h4_multiplication_table_kappa().to_owned(),
        base_lower_artifact_manifest_kappa: base_manifest.clone(),
        snapshot_artifact_manifest_kappas: [left_manifest.clone(), right_manifest.clone()],
        population_audit_cid: population_audit_cid.clone(),
        population_audit: &population_audit,
        snapshots: [&left_view, &right_view],
        pair: &pair,
        held_out_document_ids: ["53", "54"],
        prompt_cid: direct_cid(OBSERVED_PROMPT),
        target_preimage_loads_observed: TARGET_PREIMAGE_LOADS.load(Ordering::SeqCst),
    };
    let target_free_bytes = serde_json::to_vec(&target_free_census).unwrap();
    let target_free_cid = direct_cid(&target_free_bytes);

    let [base_artifact_replay, left_artifact_replay, right_artifact_replay] =
        std::thread::scope(|scope| {
            [
                scope.spawn(|| CanonicalRouteArtifact::decode_canonical(&base_artifact_bytes)),
                scope.spawn(|| CanonicalRouteArtifact::decode_canonical(&left_artifact_bytes)),
                scope.spawn(|| CanonicalRouteArtifact::decode_canonical(&right_artifact_bytes)),
            ]
            .map(|handle| handle.join().unwrap().unwrap())
        });
    let left_view_replay = left_artifact_replay
        .global_exact_spin_snapshot_view()
        .unwrap();
    let right_view_replay = right_artifact_replay
        .global_exact_spin_snapshot_view()
        .unwrap();
    assert_eq!(left_view_replay, left_view);
    assert_eq!(right_view_replay, right_view);
    let pair_replay = operator
        .predict_pair_matched(
            &table,
            &base_overlay,
            &base_artifact_replay,
            &left_artifact_replay,
            &right_artifact_replay,
            OBSERVED_PROMPT,
        )
        .unwrap();
    assert_eq!(pair_replay, pair);
    let replay_population_audit = operator.population_audit().unwrap();
    assert_eq!(
        serde_json::to_vec(&replay_population_audit).unwrap(),
        population_audit_bytes
    );
    let replay_target_free_census = TargetFreeCensus {
        schema: 1,
        domain: "uor-r4.bounded-global-noncommuting-exact-spin-target-free-census/1",
        decoded_target_commitment: TARGET_COMMITMENT,
        table_cid: table.artifact_cid(),
        base_overlay_cid: base_overlay.artifact_cid(),
        operator_cid: operator_cid.clone(),
        codec_kappa: operator.codec_kappa().to_owned(),
        vocabulary_kappa: operator.vocabulary_kappa().to_owned(),
        route_manifest_kappa: operator.route_manifest_kappa().to_owned(),
        spin_map_kappa: operator.spin_map_kappa().to_owned(),
        chart_profile_kappa: operator.chart_profile_kappa().to_owned(),
        grammar_kappa: operator.grammar_kappa().to_owned(),
        routing_policy_kappa: operator.routing_policy_kappa().to_owned(),
        population_policy_kappa: operator.population_policy_kappa().to_owned(),
        h4_root_table_kappa: operator.h4_root_table_kappa().to_owned(),
        h4_multiplication_table_kappa: operator.h4_multiplication_table_kappa().to_owned(),
        base_lower_artifact_manifest_kappa: base_manifest.clone(),
        snapshot_artifact_manifest_kappas: [left_manifest.clone(), right_manifest.clone()],
        population_audit_cid: population_audit_cid.clone(),
        population_audit: &replay_population_audit,
        snapshots: [&left_view_replay, &right_view_replay],
        pair: &pair_replay,
        held_out_document_ids: ["53", "54"],
        prompt_cid: direct_cid(OBSERVED_PROMPT),
        target_preimage_loads_observed: TARGET_PREIMAGE_LOADS.load(Ordering::SeqCst),
    };
    let replay_target_free_bytes = serde_json::to_vec(&replay_target_free_census).unwrap();
    assert_eq!(replay_target_free_bytes, target_free_bytes);
    assert_eq!(direct_cid(&replay_target_free_bytes), target_free_cid);
    assert_eq!(operator.to_bytes().unwrap(), operator_bytes);
    assert_eq!(TARGET_PREIMAGE_LOADS.load(Ordering::SeqCst), 0);

    let oversized_operator = vec![0_u8; MAX_BOUNDED_GLOBAL_EXACT_SPIN_OPERATOR_BYTES + 1];
    assert!(BoundedGlobalNoncommutingExactSpinR4V2::from_bytes(
        &table,
        &base_overlay,
        &construction,
        &oversized_operator,
    )
    .is_err());
    let tampered_operator = tamper_bound_value(
        &tamper_bound_value(
            &tamper_bound_value(
                &tamper_bound_value(&operator_bytes, operator.spin_map_kappa().as_bytes()),
                operator.h4_root_table_kappa().as_bytes(),
            ),
            operator.population_policy_kappa().as_bytes(),
        ),
        b"4c656e61",
    );
    let drifted_construction = vec![
        SourceDocument::new(
            "51",
            b"Lena bound the helix class.\n\nThe bounded global code is brass.".to_vec(),
        ),
        construction[1].clone(),
    ];
    let unrelated = vec![
        SourceDocument::new("51", b"alpha beta gamma.".to_vec()),
        SourceDocument::new("52", b"delta epsilon zeta.".to_vec()),
    ];
    let drifted_table = SourceFreeTable::compile(&unrelated).unwrap();
    let drifted_overlay = MultiscaleCountRadiusR4V1::compile(&drifted_table).unwrap();
    let binding_rejections = std::thread::scope(|scope| {
        let tampered = scope.spawn(|| {
            BoundedGlobalNoncommutingExactSpinR4V2::from_bytes(
                &table,
                &base_overlay,
                &construction,
                &tampered_operator,
            )
            .is_err()
        });
        let construction_drift = scope.spawn(|| {
            BoundedGlobalNoncommutingExactSpinR4V2::from_bytes(
                &table,
                &base_overlay,
                &drifted_construction,
                &operator_bytes,
            )
            .is_err()
        });
        let table_drift = scope.spawn(|| {
            BoundedGlobalNoncommutingExactSpinR4V2::from_bytes(
                &drifted_table,
                &drifted_overlay,
                &construction,
                &operator_bytes,
            )
            .is_err()
        });
        let overlay_drift = scope.spawn(|| {
            BoundedGlobalNoncommutingExactSpinR4V2::from_bytes(
                &table,
                &drifted_overlay,
                &construction,
                &operator_bytes,
            )
            .is_err()
        });
        [
            tampered.join().unwrap(),
            construction_drift.join().unwrap(),
            table_drift.join().unwrap(),
            overlay_drift.join().unwrap(),
        ]
    });
    assert!(binding_rejections.into_iter().all(|rejected| rejected));
    assert!(operator.build_query_artifact(b"wrong prompt").is_err());
    assert!(operator
        .build_query_artifact(&vec![b'x'; MAX_BOUNDED_GLOBAL_EXACT_SPIN_QUERY_BYTES + 1])
        .is_err());
    assert!(operator
        .build_snapshot_artifact(
            OBSERVED_PROMPT,
            &[b"Lena".to_vec(), b"helix".to_vec(), b"prism".to_vec()],
        )
        .is_err());
    assert!(operator
        .build_snapshot_artifact(
            OBSERVED_PROMPT,
            &snapshot([b"Lena", b"Lena", b"prism", b"helix"]),
        )
        .is_err());
    let carrier_rejections = std::thread::scope(|scope| {
        let reversed = scope.spawn(|| {
            operator
                .predict_pair_matched(
                    &table,
                    &base_overlay,
                    &base_artifact,
                    &right_artifact,
                    &left_artifact,
                    OBSERVED_PROMPT,
                )
                .is_err()
        });
        let aliased = scope.spawn(|| {
            operator
                .predict_pair_matched(
                    &table,
                    &base_overlay,
                    &base_artifact,
                    &left_artifact,
                    &left_artifact,
                    OBSERVED_PROMPT,
                )
                .is_err()
        });
        let wrong_lower = scope.spawn(|| {
            operator
                .predict_pair_matched(
                    &table,
                    &base_overlay,
                    &left_artifact,
                    &left_artifact,
                    &right_artifact,
                    OBSERVED_PROMPT,
                )
                .is_err()
        });
        [
            reversed.join().unwrap(),
            aliased.join().unwrap(),
            wrong_lower.join().unwrap(),
        ]
    });
    assert!(carrier_rejections.into_iter().all(|rejected| rejected));
    let mut tampered_carrier = left_artifact_bytes.clone();
    *tampered_carrier.last_mut().unwrap() ^= 1;
    assert!(CanonicalRouteArtifact::decode_canonical(&tampered_carrier).is_err());
    for invalid_bound in [0, MAX_CONTINUATION_UNITS + 1] {
        assert!(operator
            .continue_pair_matched(
                &table,
                &base_overlay,
                &base_artifact,
                &left_artifact,
                &right_artifact,
                OBSERVED_PROMPT,
                invalid_bound,
            )
            .is_err());
    }
    let operator_ref = &operator;
    let table_ref = &table;
    let base_overlay_ref = &base_overlay;
    let base_artifact_ref = &base_artifact;
    let left_artifact_ref = &left_artifact;
    let right_artifact_ref = &right_artifact;
    let truncation_rejections = std::thread::scope(|scope| {
        [1, 2]
            .map(|invalid_bound| {
                scope.spawn(move || {
                    operator_ref
                        .continue_pair_matched(
                            table_ref,
                            base_overlay_ref,
                            base_artifact_ref,
                            left_artifact_ref,
                            right_artifact_ref,
                            OBSERVED_PROMPT,
                            invalid_bound,
                        )
                        .is_err()
                })
            })
            .map(|handle| handle.join().unwrap())
    });
    assert!(truncation_rejections.into_iter().all(|rejected| rejected));
    assert_eq!(TARGET_PREIMAGE_LOADS.load(Ordering::SeqCst), 0);

    let target_preimage = reveal_target_preimage_once();
    assert_eq!(TARGET_PREIMAGE_LOADS.load(Ordering::SeqCst), 1);
    assert_eq!(target_preimage.len(), TARGET_PREIMAGE_BYTES);
    assert_eq!(direct_cid(target_preimage), TARGET_COMMITMENT);
    let sealed_targets = parse_committed_targets(target_preimage).unwrap();
    assert_eq!(
        serde_json::to_vec(&sealed_targets).unwrap(),
        target_preimage
    );
    let decoded_targets = sealed_targets
        .rows
        .iter()
        .map(|target| hex::decode(&target.continuation_hex).unwrap())
        .collect::<Vec<_>>();
    assert_eq!(
        decoded_targets,
        vec![b" teal.".to_vec(), b" bronze.".to_vec()]
    );
    let mut tampered_target = target_preimage.to_vec();
    *tampered_target.last_mut().unwrap() ^= 1;
    assert!(parse_committed_targets(&tampered_target).is_err());
    let mut target_with_newline = target_preimage.to_vec();
    target_with_newline.push(b'\n');
    assert!(parse_committed_targets(&target_with_newline).is_err());
    let mut target_with_unknown = target_preimage[..target_preimage.len() - 1].to_vec();
    target_with_unknown.extend_from_slice(br#","unknown":0}"#);
    assert!(serde_json::from_slice::<DecodedTargets>(&target_with_unknown).is_err());

    let continuation = operator
        .continue_pair_matched(
            &table,
            &base_overlay,
            &base_artifact_replay,
            &left_artifact_replay,
            &right_artifact_replay,
            OBSERVED_PROMPT,
            3,
        )
        .unwrap();
    assert_eq!(continuation.first_pair, pair);
    assert_eq!(
        continuation.first_pair.left,
        continuation.left.first_decision
    );
    assert_eq!(
        continuation.first_pair.right,
        continuation.right.first_decision
    );
    let cases = [&continuation.left, &continuation.right];
    let mut real_correct = 0_u32;
    let mut identity_disabled_correct = 0_u32;
    let mut class_operator_permuted_correct = 0_u32;
    let mut support_reversed_real_correct = 0_u32;
    let mut period_and_eos_terminations = 0_u32;
    for (index, case) in cases.iter().enumerate() {
        let target = &decoded_targets[index];
        real_correct += u32::from(case.real.decoded == *target);
        identity_disabled_correct += u32::from(case.identity_disabled.decoded == *target);
        class_operator_permuted_correct +=
            u32::from(case.class_operator_permuted.decoded == *target);
        support_reversed_real_correct += u32::from(
            case.first_decision.support_reversal_invariant
                && case.first_decision.support_reversed_real_token
                    == case.first_decision.real.token
                && case.real.decoded == *target,
        );
        for arm in [
            &case.real,
            &case.identity_disabled,
            &case.class_operator_permuted,
        ] {
            assert_eq!(arm.stop, ContinuationStop::EndOfDocument);
            assert_eq!(arm.tokens.len(), 2);
            assert_ne!(arm.tokens[0], arm.tokens[1]);
            assert_eq!(decoded(&table, arm.tokens[1]), b".");
            assert!(arm.decoded.ends_with(b"."));
            period_and_eos_terminations += 1;
        }
        assert!(case.first_decision.support_matched);
        assert!(case.first_decision.work_matched);
        assert_eq!(case.first_decision.forbidden_reads.total(), 0);
    }
    assert_eq!(continuation.left.real.decoded, b" teal.");
    assert_eq!(continuation.right.real.decoded, b" bronze.");
    assert_eq!(continuation.left.identity_disabled.decoded, b" bronze.");
    assert_eq!(continuation.right.identity_disabled.decoded, b" bronze.");
    assert_eq!(
        continuation.left.class_operator_permuted.decoded,
        b" bronze."
    );
    assert_eq!(
        continuation.right.class_operator_permuted.decoded,
        b" teal."
    );
    assert_eq!(real_correct, 2);
    assert_eq!(identity_disabled_correct, 1);
    assert_eq!(class_operator_permuted_correct, 0);
    assert_eq!(support_reversed_real_correct, 2);
    assert_eq!(period_and_eos_terminations, 6);

    let decoded_smoke = DecodedSmoke {
        schema: 1,
        domain: "uor-r4.bounded-global-noncommuting-exact-spin-decoded-smoke/1",
        operator_cid: operator_cid.clone(),
        decoded_target_commitment: TARGET_COMMITMENT,
        target_preimage_loads_observed: TARGET_PREIMAGE_LOADS.load(Ordering::SeqCst),
        rows: vec![
            DecodedCase {
                document_id: "53",
                target_hex: sealed_targets.rows[0].continuation_hex.clone(),
                continuation: &continuation.left,
            },
            DecodedCase {
                document_id: "54",
                target_hex: sealed_targets.rows[1].continuation_hex.clone(),
                continuation: &continuation.right,
            },
        ],
        real_correct,
        identity_disabled_correct,
        class_operator_permuted_correct,
        support_reversed_real_correct,
        support_mismatches: 0,
        work_mismatches: 0,
        period_and_eos_terminations,
        terminal: POSITIVE_TERMINAL,
    };
    let decoded_smoke_bytes = serde_json::to_vec(&decoded_smoke).unwrap();
    let decoded_smoke_cid = direct_cid(&decoded_smoke_bytes);
    let continuation_replay = operator
        .continue_pair_matched(
            &table,
            &base_overlay,
            &base_artifact_replay,
            &left_artifact_replay,
            &right_artifact_replay,
            OBSERVED_PROMPT,
            3,
        )
        .unwrap();
    assert_eq!(continuation_replay, continuation);
    let replay_decoded_smoke = DecodedSmoke {
        schema: 1,
        domain: "uor-r4.bounded-global-noncommuting-exact-spin-decoded-smoke/1",
        operator_cid: operator_cid.clone(),
        decoded_target_commitment: TARGET_COMMITMENT,
        target_preimage_loads_observed: TARGET_PREIMAGE_LOADS.load(Ordering::SeqCst),
        rows: vec![
            DecodedCase {
                document_id: "53",
                target_hex: sealed_targets.rows[0].continuation_hex.clone(),
                continuation: &continuation_replay.left,
            },
            DecodedCase {
                document_id: "54",
                target_hex: sealed_targets.rows[1].continuation_hex.clone(),
                continuation: &continuation_replay.right,
            },
        ],
        real_correct,
        identity_disabled_correct,
        class_operator_permuted_correct,
        support_reversed_real_correct,
        support_mismatches: 0,
        work_mismatches: 0,
        period_and_eos_terminations,
        terminal: POSITIVE_TERMINAL,
    };
    assert_eq!(
        serde_json::to_vec(&replay_decoded_smoke).unwrap(),
        decoded_smoke_bytes
    );
    assert_eq!(TARGET_PREIMAGE_LOADS.load(Ordering::SeqCst), 1);

    println!(
        "table_cid={}\nbase_overlay_cid={}\noperator_bytes={}\noperator_cid={operator_cid}\npopulation_audit_cid={population_audit_cid}\ntarget_free_census_cid={target_free_cid}\ndecoded_smoke_cid={decoded_smoke_cid}\ncodec_kappa={}\nvocabulary_kappa={}\nroute_manifest_kappa={}\nspin_map_kappa={}\nchart_profile_kappa={}\ngrammar_kappa={}\nrouting_policy_kappa={}\npopulation_policy_kappa={}\nh4_root_table_kappa={}\nh4_multiplication_table_kappa={}\nleft_global_epoch={}\nright_global_epoch={}\ntarget_commitment={TARGET_COMMITMENT}\ntarget_preimage_loads=1\nreal=2/2\nidentity_disabled=1/2\nclass_operator_permuted=0/2\nsupport_reversed_real=2/2\nperiod_and_eos_terminations=6/6\nsupport_mismatches=0\nwork_mismatches=0\nterminal={POSITIVE_TERMINAL}",
        table.artifact_cid(),
        base_overlay.artifact_cid(),
        operator_bytes.len(),
        operator.codec_kappa(),
        operator.vocabulary_kappa(),
        operator.route_manifest_kappa(),
        operator.spin_map_kappa(),
        operator.chart_profile_kappa(),
        operator.grammar_kappa(),
        operator.routing_policy_kappa(),
        operator.population_policy_kappa(),
        operator.h4_root_table_kappa(),
        operator.h4_multiplication_table_kappa(),
        pair.left.global_epoch,
        pair.right.global_epoch,
    );
}
