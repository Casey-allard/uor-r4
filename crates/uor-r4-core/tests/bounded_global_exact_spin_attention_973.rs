//! Frozen bounded-global exact-spin probe for #973.
//!
//! The target-free Gate 0 fixture preserves a valid repeated-class census but
//! falsifies the proposed order separation: the swapped exact helix/prism
//! states commute to one identical complete fold and cannot support opposite
//! roles. It does not open decoded targets or establish global attention.

use std::sync::atomic::{AtomicUsize, Ordering};

use serde::Serialize;

use uor_r4_core::bounded_global_exact_spin_attention::{
    BoundedGlobalExactSpinCandidateEvidence, BoundedGlobalExactSpinHierarchyAudit,
    BoundedGlobalExactSpinR4V1, MatchedBoundedGlobalExactSpinPrediction,
    BOUNDED_GLOBAL_EXACT_SPIN_CANDIDATES, BOUNDED_GLOBAL_EXACT_SPIN_CLASSES,
    BOUNDED_GLOBAL_EXACT_SPIN_ENTRIES, BOUNDED_GLOBAL_EXACT_SPIN_REUSE_HITS,
    MAX_BOUNDED_GLOBAL_EXACT_SPIN_OPERATOR_BYTES, MAX_BOUNDED_GLOBAL_EXACT_SPIN_QUERY_BYTES,
};
use uor_r4_core::canonical_lexical_ingestion::{
    canonical_lexical_piece_bytes, CanonicalRouteArtifact, GlobalExactSpinSnapshotView,
};
use uor_r4_core::source_free_table::{
    d3_is_held_out, BackoffOrder, MultiscaleCountRadiusR4V1, SourceDocument, SourceFreeTable,
};

const OBSERVED_PROMPT: &[u8] = b"The bounded global code is";
const ID_49_SNAPSHOT: [&[u8]; 4] = [b"Pavel", b"Pavel", b"helix", b"prism"];
const ID_50_SNAPSHOT: [&[u8]; 4] = [b"Pavel", b"Pavel", b"prism", b"helix"];
const TARGET_COMMITMENT: &str =
    "blake3:00fef74baee2785059d7be20a20fb07f9c5f5b18ae085c20503d3080019e13b1";
const NEGATIVE_TERMINAL: &str =
    "RETAIN_CONVERSATION_ONLY_REDESIGN_BOUNDED_GLOBAL_EXACT_SPIN_RELATION";

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
    h4_root_table_kappa: String,
    h4_multiplication_table_kappa: String,
    base_lower_artifact_manifest_kappa: String,
    hierarchy: &'a BoundedGlobalExactSpinHierarchyAudit,
    snapshots: [&'a GlobalExactSpinSnapshotView; 2],
    cases: Vec<TargetFreeCase<'a>>,
    target_preimage_loads_observed: usize,
    terminal: &'static str,
}

#[derive(Serialize)]
struct TargetFreeCase<'a> {
    partition_id: &'static str,
    prompt_cid: String,
    prediction: &'a MatchedBoundedGlobalExactSpinPrediction,
}

fn assert_snapshot_view(view: &GlobalExactSpinSnapshotView, expected: [&[u8]; 4]) {
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

    let repeated = &view.entries[..2];
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
            .collect::<std::collections::BTreeSet<_>>()
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
    assert_eq!(prediction.forbidden_reads.total(), 0);
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
    assert!(prediction
        .class_evaluations
        .iter()
        .all(|class| class.cold_recomputation_equal && class.result_cid == class.cold_result_cid));

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
}

#[test]
fn target_free_global_census_rejects_the_commuting_exact_spin_relation() {
    assert_eq!(TARGET_PREIMAGE_LOADS.load(Ordering::SeqCst), 0);

    let construction = construction_documents();
    assert_eq!(construction.len(), 2);
    assert!(construction
        .iter()
        .all(|document| !d3_is_held_out(&document.id)));
    assert!(d3_is_held_out("49"));
    assert!(d3_is_held_out("50"));
    let held_out = [
        SourceDocument::new("49", OBSERVED_PROMPT.to_vec()),
        SourceDocument::new("50", OBSERVED_PROMPT.to_vec()),
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

    let left_snapshot = snapshot(ID_49_SNAPSHOT);
    let right_snapshot = snapshot(ID_50_SNAPSHOT);
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
        BoundedGlobalExactSpinR4V1::compile(&table, &base_overlay, &construction).unwrap();
    let table_bytes = table.to_bytes();
    let base_overlay_bytes = base_overlay.to_bytes();
    let operator_bytes = operator.to_bytes().unwrap();
    assert!(operator_bytes.len() <= MAX_BOUNDED_GLOBAL_EXACT_SPIN_OPERATOR_BYTES);
    let operator_cid = operator.artifact_cid().unwrap();

    let table = SourceFreeTable::from_bytes(&table_bytes).unwrap();
    let base_overlay = MultiscaleCountRadiusR4V1::from_bytes(&table, &base_overlay_bytes).unwrap();
    let oversized_operator = vec![0_u8; MAX_BOUNDED_GLOBAL_EXACT_SPIN_OPERATOR_BYTES + 1];
    assert!(BoundedGlobalExactSpinR4V1::from_bytes(
        &table,
        &base_overlay,
        &construction,
        &oversized_operator,
    )
    .is_err());
    let mut tampered_operator = operator_bytes.clone();
    *tampered_operator.last_mut().unwrap() ^= 1;
    assert!(BoundedGlobalExactSpinR4V1::from_bytes(
        &table,
        &base_overlay,
        &construction,
        &tampered_operator,
    )
    .is_err());
    let operator = BoundedGlobalExactSpinR4V1::from_bytes(
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
    for prototype in &prototypes {
        assert!(prototype.anchor_address_kappa.starts_with("blake3:"));
        assert!(prototype.anchor_payload_cid.starts_with("blake3:"));
        assert!(prototype.anchor_class_kappa.starts_with("blake3:"));
        assert!(prototype
            .anchor_state
            .h4_coordinate
            .scaled_zphi_quaternion
            .iter()
            .all(|coordinate| coordinate[1] == 0));
    }

    let base_artifact = operator.build_query_artifact(OBSERVED_PROMPT).unwrap();
    assert!(operator.build_query_artifact(b"wrong prompt").is_err());
    assert!(operator
        .build_snapshot_artifact(
            OBSERVED_PROMPT,
            &[b"Pavel".to_vec(), b"helix".to_vec(), b"prism".to_vec()],
        )
        .is_err());
    let left_artifact = operator
        .build_snapshot_artifact(OBSERVED_PROMPT, &left_snapshot)
        .unwrap();
    let right_artifact = operator
        .build_snapshot_artifact(OBSERVED_PROMPT, &right_snapshot)
        .unwrap();
    let base_artifact_bytes = base_artifact.canonical_bytes().unwrap();
    let left_artifact_bytes = left_artifact.canonical_bytes().unwrap();
    let right_artifact_bytes = right_artifact.canonical_bytes().unwrap();
    let base_manifest = base_artifact.manifest_kappa().to_owned();
    let left_manifest = left_artifact.manifest_kappa().to_owned();
    let right_manifest = right_artifact.manifest_kappa().to_owned();
    assert_ne!(base_manifest, left_manifest);
    assert_ne!(base_manifest, right_manifest);
    assert_ne!(left_manifest, right_manifest);
    let left_view = left_artifact.global_exact_spin_snapshot_view().unwrap();
    let right_view = right_artifact.global_exact_spin_snapshot_view().unwrap();
    assert_eq!(
        left_artifact.canonical_bytes().unwrap(),
        left_artifact_bytes
    );
    assert_eq!(
        right_artifact.canonical_bytes().unwrap(),
        right_artifact_bytes
    );
    assert_eq!(left_artifact.manifest_kappa(), left_manifest);
    assert_eq!(right_artifact.manifest_kappa(), right_manifest);
    assert_snapshot_view(&left_view, ID_49_SNAPSHOT);
    assert_snapshot_view(&right_view, ID_50_SNAPSHOT);
    assert_eq!(left_view.source_artifact_manifest_kappa, left_manifest);
    assert_eq!(right_view.source_artifact_manifest_kappa, right_manifest);
    assert_ne!(left_view.snapshot_kappa, right_view.snapshot_kappa);
    assert_ne!(left_view.global_root_kappa, right_view.global_root_kappa);

    let hierarchy = operator
        .audit_hierarchy_pair(&left_artifact, &right_artifact)
        .unwrap();
    assert_eq!(hierarchy.levels.len(), 7);
    assert!(hierarchy.lower_through_conversation_ordered_state_equal);
    assert!(hierarchy.global_identity_distinct);
    assert!(hierarchy.global_ordered_state_distinct);
    assert_ne!(
        hierarchy.left_global_snapshot_kappa,
        hierarchy.right_global_snapshot_kappa
    );
    for level in &hierarchy.levels[..6] {
        assert!(level.ordered_state_equal);
        assert_eq!(level.left_state, level.right_state);
    }
    assert_eq!(hierarchy.levels[6].level, "global");
    assert!(!hierarchy.levels[6].identity_equal);
    assert!(!hierarchy.levels[6].ordered_state_equal);

    let left = operator
        .predict_matched(
            &table,
            &base_overlay,
            &base_artifact,
            &left_artifact,
            OBSERVED_PROMPT,
        )
        .unwrap();
    let right = operator
        .predict_matched(
            &table,
            &base_overlay,
            &base_artifact,
            &right_artifact,
            OBSERVED_PROMPT,
        )
        .unwrap();
    assert_common_prediction(&table, &left);
    assert_common_prediction(&table, &right);
    assert_eq!(left.local, right.local);
    assert_eq!(left.real.work, right.real.work);
    assert_eq!(left.real.support_tokens, right.real.support_tokens);
    assert_ne!(left.global_epoch, right.global_epoch);
    assert_ne!(left.global_root_kappa, right.global_root_kappa);
    assert_eq!(left.global_result, right.global_result);
    assert_eq!(left.base_lower_artifact_manifest_kappa, base_manifest);
    assert_eq!(right.base_lower_artifact_manifest_kappa, base_manifest);
    assert_eq!(left.source_snapshot_artifact_manifest_kappa, left_manifest);
    assert_eq!(
        right.source_snapshot_artifact_manifest_kappa,
        right_manifest
    );
    assert_eq!(left.operator_cid, operator_cid);
    assert_eq!(right.operator_cid, operator_cid);
    assert_eq!(left.spin_map_kappa, operator.spin_map_kappa());
    assert_eq!(right.spin_map_kappa, operator.spin_map_kappa());
    assert_eq!(left.chart_profile_kappa, operator.chart_profile_kappa());
    assert_eq!(right.chart_profile_kappa, operator.chart_profile_kappa());

    let identity_coordinate = [[2, 0], [0, 0], [0, 0], [0, 0]];
    let minus_one_coordinate = [[-2, 0], [0, 0], [0, 0], [0, 0]];
    assert_eq!(
        left.fold_steps[0]
            .before
            .h4_coordinate
            .scaled_zphi_quaternion,
        identity_coordinate
    );
    assert_eq!(
        right.fold_steps[0]
            .before
            .h4_coordinate
            .scaled_zphi_quaternion,
        identity_coordinate
    );
    assert_eq!(left.fold_steps[0].before.fiber_q29, 0);
    assert_eq!(left.fold_steps[0].before.torsion_q29, 0);
    assert_eq!(
        left.global_result.h4_coordinate.scaled_zphi_quaternion,
        minus_one_coordinate
    );
    assert_eq!(
        right.global_result.h4_coordinate.scaled_zphi_quaternion,
        minus_one_coordinate
    );
    assert_eq!(left.global_result.fiber_q29, 110_444_176);
    assert_eq!(right.global_result.fiber_q29, 110_444_176);
    assert_eq!(left.global_result.torsion_q29, -10_509_096);
    assert_eq!(right.global_result.torsion_q29, -10_509_096);
    assert_eq!(left.fold_steps.last().unwrap().after, left.global_result);
    assert_eq!(right.fold_steps.last().unwrap().after, right.global_result);
    assert_eq!(left.fold_steps[1].after, right.fold_steps[1].after);
    assert_ne!(left.fold_steps[2].after, right.fold_steps[2].after);
    assert_eq!(left.fold_steps[3].after, right.fold_steps[3].after);

    let left_helix_entry = left
        .snapshot_entries
        .iter()
        .find(|entry| entry.payload_hex == hex::encode(b"helix"))
        .unwrap();
    let right_helix_entry = right
        .snapshot_entries
        .iter()
        .find(|entry| entry.payload_hex == hex::encode(b"helix"))
        .unwrap();
    let left_prism_entry = left
        .snapshot_entries
        .iter()
        .find(|entry| entry.payload_hex == hex::encode(b"prism"))
        .unwrap();
    let right_prism_entry = right
        .snapshot_entries
        .iter()
        .find(|entry| entry.payload_hex == hex::encode(b"prism"))
        .unwrap();
    assert_eq!(
        left_helix_entry.mapped_state,
        right_helix_entry.mapped_state
    );
    assert_eq!(
        left_prism_entry.mapped_state,
        right_prism_entry.mapped_state
    );
    assert_ne!(left_helix_entry.mapped_state, left_prism_entry.mapped_state);

    let left_helix = candidate_for_anchor(&left, b"helix");
    let left_prism = candidate_for_anchor(&left, b"prism");
    let right_helix = candidate_for_anchor(&right, b"helix");
    let right_prism = candidate_for_anchor(&right, b"prism");
    assert_eq!(left.real.minimum_cost, Some(left_helix.real_measured_cost));
    assert_eq!(
        right.real.minimum_cost,
        Some(right_helix.real_measured_cost)
    );
    assert_eq!(
        left_helix.real_measured_cost,
        right_helix.real_measured_cost
    );
    assert_eq!(
        left_prism.real_measured_cost,
        right_prism.real_measured_cost
    );
    assert_ne!(left_helix.real_measured_cost, left_prism.real_measured_cost);
    assert_eq!(decoded(&table, left.real.token), b" bronze");
    assert_eq!(decoded(&table, right.real.token), b" bronze");
    assert_eq!(decoded(&table, left.identity_disabled.token), b" bronze");
    assert_eq!(decoded(&table, right.identity_disabled.token), b" bronze");
    assert_eq!(
        decoded(&table, left.class_operator_permuted.token),
        b" teal"
    );
    assert_eq!(
        decoded(&table, right.class_operator_permuted.token),
        b" teal"
    );
    assert_eq!(left.real.token, right.real.token);
    assert_eq!(
        left.class_operator_permuted.token,
        right.class_operator_permuted.token
    );
    let mut left_real_results = left
        .candidate_evidence
        .iter()
        .map(|candidate| candidate.real_class_result_cid.clone())
        .collect::<Vec<_>>();
    let mut left_permuted_results = left
        .candidate_evidence
        .iter()
        .map(|candidate| candidate.permuted_class_result_cid.clone())
        .collect::<Vec<_>>();
    left_real_results.sort();
    left_permuted_results.sort();
    assert_eq!(left_real_results, left_permuted_results);
    let mut right_real_results = right
        .candidate_evidence
        .iter()
        .map(|candidate| candidate.real_class_result_cid.clone())
        .collect::<Vec<_>>();
    let mut right_permuted_results = right
        .candidate_evidence
        .iter()
        .map(|candidate| candidate.permuted_class_result_cid.clone())
        .collect::<Vec<_>>();
    right_real_results.sort();
    right_permuted_results.sort();
    assert_eq!(right_real_results, right_permuted_results);

    assert_eq!(TARGET_PREIMAGE_LOADS.load(Ordering::SeqCst), 0);
    let target_free_census = TargetFreeCensus {
        schema: 1,
        domain: "uor-r4.bounded-global-exact-spin-target-free-census/1",
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
        h4_root_table_kappa: operator.h4_root_table_kappa().to_owned(),
        h4_multiplication_table_kappa: operator.h4_multiplication_table_kappa().to_owned(),
        base_lower_artifact_manifest_kappa: base_manifest.clone(),
        hierarchy: &hierarchy,
        snapshots: [&left_view, &right_view],
        cases: vec![
            TargetFreeCase {
                partition_id: "49",
                prompt_cid: direct_cid(OBSERVED_PROMPT),
                prediction: &left,
            },
            TargetFreeCase {
                partition_id: "50",
                prompt_cid: direct_cid(OBSERVED_PROMPT),
                prediction: &right,
            },
        ],
        target_preimage_loads_observed: TARGET_PREIMAGE_LOADS.load(Ordering::SeqCst),
        terminal: NEGATIVE_TERMINAL,
    };
    let target_free_bytes = serde_json::to_vec(&target_free_census).unwrap();
    let target_free_cid = direct_cid(&target_free_bytes);

    let base_artifact_replay =
        CanonicalRouteArtifact::decode_canonical(&base_artifact_bytes).unwrap();
    let left_artifact_replay =
        CanonicalRouteArtifact::decode_canonical(&left_artifact_bytes).unwrap();
    let right_artifact_replay =
        CanonicalRouteArtifact::decode_canonical(&right_artifact_bytes).unwrap();
    assert_eq!(
        base_artifact_replay.canonical_bytes().unwrap(),
        base_artifact_bytes
    );
    assert_eq!(base_artifact_replay.manifest_kappa(), base_manifest);
    assert_eq!(
        left_artifact_replay.canonical_bytes().unwrap(),
        left_artifact_bytes
    );
    assert_eq!(
        right_artifact_replay.canonical_bytes().unwrap(),
        right_artifact_bytes
    );
    let left_view_replay = left_artifact_replay
        .global_exact_spin_snapshot_view()
        .unwrap();
    let right_view_replay = right_artifact_replay
        .global_exact_spin_snapshot_view()
        .unwrap();
    assert_eq!(left_view_replay, left_view);
    assert_eq!(right_view_replay, right_view);
    let hierarchy_replay = operator
        .audit_hierarchy_pair(&left_artifact_replay, &right_artifact_replay)
        .unwrap();
    let left_replay = operator
        .predict_matched(
            &table,
            &base_overlay,
            &base_artifact_replay,
            &left_artifact_replay,
            OBSERVED_PROMPT,
        )
        .unwrap();
    let right_replay = operator
        .predict_matched(
            &table,
            &base_overlay,
            &base_artifact_replay,
            &right_artifact_replay,
            OBSERVED_PROMPT,
        )
        .unwrap();
    assert_eq!(hierarchy_replay, hierarchy);
    assert_eq!(left_replay, left);
    assert_eq!(right_replay, right);
    let replay_target_free_census = TargetFreeCensus {
        schema: 1,
        domain: "uor-r4.bounded-global-exact-spin-target-free-census/1",
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
        h4_root_table_kappa: operator.h4_root_table_kappa().to_owned(),
        h4_multiplication_table_kappa: operator.h4_multiplication_table_kappa().to_owned(),
        base_lower_artifact_manifest_kappa: base_manifest.clone(),
        hierarchy: &hierarchy_replay,
        snapshots: [&left_view_replay, &right_view_replay],
        cases: vec![
            TargetFreeCase {
                partition_id: "49",
                prompt_cid: direct_cid(OBSERVED_PROMPT),
                prediction: &left_replay,
            },
            TargetFreeCase {
                partition_id: "50",
                prompt_cid: direct_cid(OBSERVED_PROMPT),
                prediction: &right_replay,
            },
        ],
        target_preimage_loads_observed: TARGET_PREIMAGE_LOADS.load(Ordering::SeqCst),
        terminal: NEGATIVE_TERMINAL,
    };
    let replay_target_free_bytes = serde_json::to_vec(&replay_target_free_census).unwrap();
    assert_eq!(replay_target_free_bytes, target_free_bytes);
    assert_eq!(direct_cid(&replay_target_free_bytes), target_free_cid);
    assert_eq!(TARGET_PREIMAGE_LOADS.load(Ordering::SeqCst), 0);

    assert_eq!(operator.to_bytes().unwrap(), operator_bytes);
    assert_eq!(TARGET_PREIMAGE_LOADS.load(Ordering::SeqCst), 0);

    println!(
        "table_cid={}\nbase_overlay_cid={}\noperator_bytes={}\noperator_cid={operator_cid}\ncodec_kappa={}\nvocabulary_kappa={}\nroute_manifest_kappa={}\nspin_map_kappa={}\nchart_profile_kappa={}\ngrammar_kappa={}\nrouting_policy_kappa={}\nh4_root_table_kappa={}\nh4_multiplication_table_kappa={}\nleft_global_epoch={}\nright_global_epoch={}\ntarget_commitment={TARGET_COMMITMENT}\ntarget_free_census_cid={target_free_cid}\nleft_global_fold=-1\nright_global_fold=-1\nreal_roles=helix/helix\nclass_operator_permuted_roles=prism/prism\nsupport_mismatches=0\nwork_mismatches=0\ntarget_preimage_loads=0\nterminal={NEGATIVE_TERMINAL}",
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
        operator.h4_root_table_kappa(),
        operator.h4_multiplication_table_kappa(),
        left.global_epoch,
        right.global_epoch,
    );
}
