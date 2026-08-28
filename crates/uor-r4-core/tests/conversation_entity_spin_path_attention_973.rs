//! Frozen conversation-scope probe for #973 `ConversationEntitySpinPathR4V1`.
//!
//! The fixture is synthetic. D3 separation prevents held-out conversation
//! bytes from entering fitting; it does not establish natural-distribution or
//! semantic transfer.

use serde::Serialize;

use uor_r4_core::canonical_lexical_ingestion::canonical_lexical_piece_bytes;
use uor_r4_core::conversation_entity_spin_path_attention::{
    ConversationEntityHierarchyAudit, ConversationEntitySpinPathCost,
    ConversationEntitySpinPathR4V1, MatchedConversationEntitySpinPathContinuation,
    MatchedConversationEntitySpinPathPrediction, MAX_CONVERSATION_ENTITY_BYTES,
    MAX_CONVERSATION_ENTITY_OPERATOR_BYTES, MAX_CONVERSATION_ENTITY_UNITS,
};
use uor_r4_core::higher_scope_geometric_attention::{
    PriorSentenceCountRadiusAbstention, PriorSentenceCountRadiusR4V1,
};
use uor_r4_core::prime_route_geometric_attention::H4S3AngularShell;
use uor_r4_core::source_free_table::{
    d3_is_held_out, BackoffOrder, ContinuationStop, MultiscaleCountRadiusR4V1, SourceDocument,
    SourceFreeTable, BOS_TOKEN,
};

const SPIRAL_BINDING_TURN: &[u8] =
    b"Mara carried the spiral marker. Iris carried the faceted marker.";
const FACETED_BINDING_TURN: &[u8] =
    b"Mara carried the faceted marker. Iris carried the spiral marker.";
const FOCUS_TURN: &[u8] = b"Mara opened the registry. Iris waited.";
const ACTIVE_QUERY: &[u8] = b"The active registry code is";

fn construction_documents() -> Vec<SourceDocument> {
    vec![
        SourceDocument::new(
            "27",
            b"Nora carried the spiral marker.\n\nNora opened the registry. Owen waited.\n\nThe active registry code is silver."
                .to_vec(),
        ),
        SourceDocument::new(
            "28",
            b"Owen carried the faceted marker.\n\nOwen opened the registry. Nora waited.\n\nThe active registry code is violet."
                .to_vec(),
        ),
    ]
}

fn prompt(binding_turn: &[u8]) -> Vec<u8> {
    let mut bytes = binding_turn.to_vec();
    bytes.extend_from_slice(b"\n\n");
    bytes.extend_from_slice(FOCUS_TURN);
    bytes.extend_from_slice(b"\n\n");
    bytes.extend_from_slice(ACTIVE_QUERY);
    bytes
}

fn context(table: &SourceFreeTable, binding_turn: &[u8]) -> Vec<u32> {
    let mut context = vec![BOS_TOKEN];
    context.extend(table.encode_text(&prompt(binding_turn)).unwrap());
    context
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

#[derive(Serialize)]
struct ByteIdentityWire {
    schema: u32,
    domain: &'static str,
    bytes_hex: String,
}

fn canonical_payload_cid(bytes: &[u8]) -> String {
    let identity = serde_json::to_vec(&ByteIdentityWire {
        schema: 1,
        domain: "uor-r4.canonical-byte-identity/1",
        bytes_hex: hex::encode(bytes),
    })
    .unwrap();
    uor_addr::json::address_blake3(&identity)
        .unwrap()
        .address
        .to_string()
}

#[derive(Serialize)]
struct TargetFreeCensus<'a> {
    schema: u32,
    domain: &'static str,
    table_cid: String,
    base_overlay_cid: String,
    operator_cid: String,
    codec_kappa: String,
    vocabulary_kappa: String,
    route_manifest_kappa: String,
    spin_map_kappa: String,
    grammar_kappa: String,
    routing_policy_kappa: String,
    hierarchy_audit_policy_kappa: String,
    h4_root_table_kappa: String,
    h4_multiplication_table_kappa: String,
    hierarchy: &'a ConversationEntityHierarchyAudit,
    cases: Vec<TargetFreeCase<'a>>,
    teacher_calls: u64,
    provider_calls: u64,
    source_weight_reads: u64,
    future_unit_reads: u64,
    target_reads: u64,
    partition_id_reads: u64,
    full_history_key_reads: u64,
    global_operator_reads: u64,
}

#[derive(Serialize)]
struct TargetFreeCase<'a> {
    partition_id: &'static str,
    prompt_cid: String,
    prediction: &'a MatchedConversationEntitySpinPathPrediction,
}

#[derive(Serialize)]
struct DecodedSmoke<'a> {
    schema: u32,
    domain: &'static str,
    operator_cid: String,
    cases: Vec<DecodedCase<'a>>,
    real_correct: u32,
    disabled_correct: u32,
    permuted_correct: u32,
    reversed_correct: u32,
    support_mismatches: u32,
    work_mismatches: u32,
    terminal: &'static str,
}

#[derive(Serialize)]
struct DecodedCase<'a> {
    partition_id: &'static str,
    target_hex: String,
    continuation: &'a MatchedConversationEntitySpinPathContinuation,
}

#[test]
fn target_free_conversation_census_then_sealed_decoded_controls_are_load_bearing() {
    let construction = construction_documents();
    assert!(construction
        .iter()
        .all(|document| !d3_is_held_out(&document.id)));
    assert!(construction.iter().all(|document| {
        !contains_bytes(&document.text, b"Mara") && !contains_bytes(&document.text, b"Iris")
    }));
    assert!(d3_is_held_out("45"));
    assert!(d3_is_held_out("48"));

    let held_out = [
        SourceDocument::new("45", prompt(SPIRAL_BINDING_TURN)),
        SourceDocument::new("48", prompt(FACETED_BINDING_TURN)),
    ];
    assert_ne!(held_out[0].id, held_out[1].id);
    assert_ne!(held_out[0].text_cid(), held_out[1].text_cid());
    assert_eq!(held_out[0].text.len(), held_out[1].text.len());
    assert_ne!(held_out[0].text, held_out[1].text);
    assert_eq!(
        &held_out[0].text[SPIRAL_BINDING_TURN.len() + 2..],
        &held_out[1].text[FACETED_BINDING_TURN.len() + 2..]
    );
    for document in &held_out {
        assert!(construction.iter().all(|source| {
            source.id != document.id && source.text_cid() != document.text_cid()
        }));
        assert!(!contains_bytes(&document.text, b" silver"));
        assert!(!contains_bytes(&document.text, b" violet"));
        assert!(!contains_bytes(&document.text, b"Nora"));
        assert!(!contains_bytes(&document.text, b"Owen"));
        assert!(document.text.len() <= MAX_CONVERSATION_ENTITY_BYTES);
    }
    let mut spiral_multiset = canonical_lexical_piece_bytes(&held_out[0].text).unwrap();
    let mut faceted_multiset = canonical_lexical_piece_bytes(&held_out[1].text).unwrap();
    spiral_multiset.sort();
    faceted_multiset.sort();
    assert_eq!(spiral_multiset, faceted_multiset);

    let table = SourceFreeTable::compile(&construction).unwrap();
    let base_overlay = MultiscaleCountRadiusR4V1::compile(&table).unwrap();
    let operator =
        ConversationEntitySpinPathR4V1::compile(&table, &base_overlay, &construction).unwrap();
    let table_bytes = table.to_bytes();
    let base_overlay_bytes = base_overlay.to_bytes();
    let operator_bytes = operator.to_bytes().unwrap();
    assert!(operator_bytes.len() <= MAX_CONVERSATION_ENTITY_OPERATOR_BYTES);
    let operator_cid = operator.artifact_cid().unwrap();

    let table = SourceFreeTable::from_bytes(&table_bytes).unwrap();
    let base_overlay = MultiscaleCountRadiusR4V1::from_bytes(&table, &base_overlay_bytes).unwrap();
    assert_eq!(table.to_bytes(), table_bytes);
    assert_eq!(base_overlay.to_bytes(), base_overlay_bytes);
    let reloaded = ConversationEntitySpinPathR4V1::from_bytes(
        &table,
        &base_overlay,
        &construction,
        &operator_bytes,
    )
    .unwrap();
    assert_eq!(reloaded.to_bytes().unwrap(), operator_bytes);
    assert_eq!(reloaded.artifact_cid().unwrap(), operator_cid);
    assert_eq!(reloaded.table_artifact_cid(), table.artifact_cid());
    assert_eq!(
        reloaded.base_overlay_artifact_cid(),
        base_overlay.artifact_cid()
    );

    // Target-free hierarchy isolation and candidate-relative census. Expected
    // continuations are not constructed until this report has replayed.
    let hierarchy = reloaded
        .audit_hierarchy_pair(
            SPIRAL_BINDING_TURN,
            FACETED_BINDING_TURN,
            FOCUS_TURN,
            ACTIVE_QUERY,
        )
        .unwrap();
    assert!(hierarchy.lexical_multiset_equal);
    assert!(hierarchy.lower_scope_identities_equal);
    assert!(hierarchy.lower_scope_ordered_states_equal);
    assert!(hierarchy.conversation_identity_distinct);
    assert!(hierarchy.global_identity_equal);
    assert!(hierarchy.global_ordered_state_equal);
    assert_eq!(
        hierarchy.left_global_snapshot_kappa,
        hierarchy.right_global_snapshot_kappa
    );
    assert_eq!(hierarchy.global_epoch, reloaded.global_epoch().unwrap());
    assert_eq!(hierarchy.levels.len(), 7);
    for level in &hierarchy.levels[..5] {
        assert!(level.identity_equal);
        assert!(level.ordered_state_equal);
        assert_eq!(level.left_state, level.right_state);
        assert_eq!(level.left_root_coordinate, level.right_root_coordinate);
    }
    assert_eq!(hierarchy.levels[5].level, "conversation");
    assert!(!hierarchy.levels[5].identity_equal);
    assert_eq!(hierarchy.levels[6].level, "global");
    assert!(hierarchy.levels[6].identity_equal);
    assert!(hierarchy.levels[6].ordered_state_equal);
    assert!(!hierarchy.score_input_used);
    assert_eq!(hierarchy.global_operator_reads, 0);
    assert_eq!(hierarchy.target_reads, 0);
    assert_eq!(hierarchy.partition_id_reads, 0);

    let spiral = reloaded
        .predict_matched(
            &table,
            &base_overlay,
            SPIRAL_BINDING_TURN,
            FOCUS_TURN,
            ACTIVE_QUERY,
        )
        .unwrap();
    let faceted = reloaded
        .predict_matched(
            &table,
            &base_overlay,
            FACETED_BINDING_TURN,
            FOCUS_TURN,
            ACTIVE_QUERY,
        )
        .unwrap();
    for prediction in [&spiral, &faceted] {
        assert_eq!(prediction.local.order, BackoffOrder::Trigram);
        assert_eq!(prediction.local.max_count, 1);
        assert!(prediction.local.geometry_reachable);
        assert_eq!(
            decoded_support(&table, &prediction.local.max_count_tie_tokens),
            vec![b" silver".to_vec(), b" violet".to_vec()]
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
        assert_eq!(
            decoded(&table, prediction.local.geometric_token),
            b" silver"
        );
        assert_eq!(prediction.prior_candidate_occurrences, 0);
        assert!(prediction.support_matched);
        assert!(prediction.work_matched);
        assert_eq!(prediction.operator_abstention, None);
        assert_eq!(prediction.teacher_calls, 0);
        assert_eq!(prediction.provider_calls, 0);
        assert_eq!(prediction.source_weight_reads, 0);
        assert_eq!(prediction.future_unit_reads, 0);
        assert_eq!(prediction.target_reads, 0);
        assert_eq!(prediction.partition_id_reads, 0);
        assert_eq!(prediction.full_history_key_reads, 0);
        assert_eq!(prediction.global_operator_reads, 0);
        assert_eq!(prediction.candidate_evidence.len(), 2);
        assert_eq!(prediction.real.work.completed_turn_slots_scanned, 2);
        assert_eq!(prediction.real.work.binding_fact_slots_scanned, 2);
        assert_eq!(prediction.real.work.focus_role_slots_scanned, 2);
        assert_eq!(prediction.real.work.entity_key_comparisons, 2);
        assert_eq!(prediction.real.work.descriptor_row_comparisons, 2);
        assert_eq!(prediction.real.work.stored_spin_leaf_reads, 7);
        assert_eq!(prediction.real.work.h4_product_table_reads, 9);
        assert_eq!(prediction.real.work.h4_inverse_table_reads, 2);
        assert_eq!(prediction.real.work.phase_additions, 18);
        assert_eq!(prediction.real.work.phase_distance_reads, 4);
        assert_eq!(prediction.real.work.angular_shell_reads, 2);
        assert_eq!(prediction.real.work.cost_comparisons, 2);
        assert_eq!(prediction.real.work.final_choice_operations, 1);
        assert_eq!(prediction.real.work, prediction.conversation_disabled.work);
        assert_eq!(
            prediction.real.work,
            prediction.cross_turn_binding_permuted.work
        );
        assert_eq!(prediction.real.work, prediction.binding_rows_reversed.work);
        assert_eq!(prediction.conversation_disabled.unique_minimum, None);
        assert_eq!(prediction.conversation_disabled.minimum_cost, None);
        assert_eq!(
            prediction.conversation_disabled.token,
            prediction.local.geometric_token
        );
        assert_eq!(
            prediction.binding_rows_reversed.token,
            prediction.real.token
        );
        assert_eq!(
            prediction.binding_rows_reversed.unique_minimum,
            prediction.real.unique_minimum
        );
        assert_eq!(
            prediction.binding_rows_reversed.minimum_cost,
            prediction.real.minimum_cost
        );
        assert!(prediction.real.unique_minimum.is_some());
        assert!(prediction
            .cross_turn_binding_permuted
            .unique_minimum
            .is_some());
    }
    assert_eq!(spiral.local.tie_evidence, faceted.local.tie_evidence);
    assert_eq!(spiral.local.geometric_work, faceted.local.geometric_work);
    assert_eq!(spiral.local.geometric_token, faceted.local.geometric_token);
    assert!(context(&table, SPIRAL_BINDING_TURN).len() - 1 <= MAX_CONVERSATION_ENTITY_UNITS);
    assert!(context(&table, FACETED_BINDING_TURN).len() - 1 <= MAX_CONVERSATION_ENTITY_UNITS);

    assert_ne!(spiral.real.token, faceted.real.token);
    assert_eq!(spiral.cross_turn_binding_permuted.token, faceted.real.token);
    assert_eq!(faceted.cross_turn_binding_permuted.token, spiral.real.token);
    assert_eq!(spiral.binding_rows_reversed.token, spiral.real.token);
    assert_eq!(faceted.binding_rows_reversed.token, faceted.real.token);

    let prototypes = reloaded.prototype_traces().unwrap();
    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes
            .iter()
            .map(|prototype| hex::decode(&prototype.candidate_hex).unwrap())
            .collect::<Vec<_>>(),
        vec![b" silver".to_vec(), b" violet".to_vec()]
    );
    assert_eq!(
        prototypes
            .iter()
            .map(|prototype| hex::decode(&prototype.descriptor_hex).unwrap())
            .collect::<Vec<_>>(),
        vec![b"spiral".to_vec(), b"faceted".to_vec()]
    );
    assert_ne!(prototypes[0].path_state, prototypes[1].path_state);
    for (prototype, payload) in prototypes.iter().zip([b"silver".as_slice(), b"violet"]) {
        assert_eq!(prototype.binding_leaves.len(), 4);
        assert_eq!(prototype.focus_leaves.len(), 3);
        assert_eq!(
            prototype.binding_leaves.len() + prototype.focus_leaves.len(),
            7
        );
        assert_eq!(prototype.candidate_value.payload_hex, hex::encode(payload));
        assert_eq!(
            prototype.candidate_value.payload_cid,
            canonical_payload_cid(payload)
        );
        assert!(prototype
            .candidate_value
            .address_kappa
            .starts_with("blake3:"));
        assert!(prototype.candidate_value.prime >= 2);
        assert!(!prototype.candidate_geometry_used_for_ranking);
        for leaf in prototype
            .binding_leaves
            .iter()
            .chain(&prototype.focus_leaves)
        {
            assert!(leaf.address_kappa.starts_with("blake3:"));
            assert!(leaf.payload_cid.starts_with("blake3:"));
            assert!(leaf.prime >= 2);
        }
    }

    let zero_cost = ConversationEntitySpinPathCost {
        angular_shell: H4S3AngularShell::Coincident,
        fiber_distance_q29: 0,
        torsion_distance_q29: 0,
    };
    for prediction in [&spiral, &faceted] {
        let matching = prediction
            .candidate_evidence
            .iter()
            .filter(|candidate| candidate.real.measured_cost == zero_cost)
            .collect::<Vec<_>>();
        assert_eq!(matching.len(), 1);
        assert_eq!(Some(matching[0].token), prediction.real.unique_minimum);
        assert_eq!(prediction.real.minimum_cost, Some(zero_cost));
        assert!(prediction
            .candidate_evidence
            .iter()
            .filter(|candidate| candidate.real.measured_cost != zero_cost)
            .all(|candidate| candidate.real.measured_cost > zero_cost));
        for candidate in &prediction.candidate_evidence {
            assert_eq!(
                candidate.real.ranking_cost,
                Some(candidate.real.measured_cost)
            );
            assert_eq!(candidate.conversation_disabled.ranking_cost, None);
            assert_eq!(
                candidate.cross_turn_binding_permuted.ranking_cost,
                Some(candidate.cross_turn_binding_permuted.measured_cost)
            );
            assert_eq!(
                candidate.binding_rows_reversed.ranking_cost,
                Some(candidate.binding_rows_reversed.measured_cost)
            );
            assert_eq!(candidate.binding_rows_reversed, candidate.real);
        }
    }

    let prior_copy = PriorSentenceCountRadiusR4V1::compile(&table, &base_overlay).unwrap();
    for binding_turn in [SPIRAL_BINDING_TURN, FACETED_BINDING_TURN] {
        let copy = prior_copy
            .predict_matched(&table, &base_overlay, &context(&table, binding_turn))
            .unwrap();
        assert_eq!(
            copy.operator_abstention,
            Some(PriorSentenceCountRadiusAbstention::NoPriorCandidateOccurrence)
        );
        assert_eq!(decoded(&table, copy.real.token), b" silver");
    }

    let target_free_census = TargetFreeCensus {
        schema: 1,
        domain: "uor-r4.conversation-entity-spin-path-target-free-census/1",
        table_cid: table.artifact_cid(),
        base_overlay_cid: base_overlay.artifact_cid(),
        operator_cid: operator_cid.clone(),
        codec_kappa: reloaded.codec_kappa().to_owned(),
        vocabulary_kappa: reloaded.vocabulary_kappa().to_owned(),
        route_manifest_kappa: reloaded.route_manifest_kappa().to_owned(),
        spin_map_kappa: reloaded.spin_map_kappa().to_owned(),
        grammar_kappa: reloaded.grammar_kappa().to_owned(),
        routing_policy_kappa: reloaded.routing_policy_kappa().to_owned(),
        hierarchy_audit_policy_kappa: reloaded.hierarchy_audit_policy_kappa().to_owned(),
        h4_root_table_kappa: reloaded.h4_root_table_kappa().to_owned(),
        h4_multiplication_table_kappa: reloaded.h4_multiplication_table_kappa().to_owned(),
        hierarchy: &hierarchy,
        cases: vec![
            TargetFreeCase {
                partition_id: "45",
                prompt_cid: format!("blake3:{}", blake3::hash(&held_out[0].text).to_hex()),
                prediction: &spiral,
            },
            TargetFreeCase {
                partition_id: "48",
                prompt_cid: format!("blake3:{}", blake3::hash(&held_out[1].text).to_hex()),
                prediction: &faceted,
            },
        ],
        teacher_calls: 0,
        provider_calls: 0,
        source_weight_reads: 0,
        future_unit_reads: 0,
        target_reads: 0,
        partition_id_reads: 0,
        full_history_key_reads: 0,
        global_operator_reads: 0,
    };
    let target_free_bytes = serde_json::to_vec(&target_free_census).unwrap();
    let target_free_cid = format!("blake3:{}", blake3::hash(&target_free_bytes).to_hex());

    let hierarchy_replay = reloaded
        .audit_hierarchy(
            SPIRAL_BINDING_TURN,
            FACETED_BINDING_TURN,
            FOCUS_TURN,
            ACTIVE_QUERY,
        )
        .unwrap();
    let spiral_census_replay = reloaded
        .predict_matched(
            &table,
            &base_overlay,
            SPIRAL_BINDING_TURN,
            FOCUS_TURN,
            ACTIVE_QUERY,
        )
        .unwrap();
    let faceted_census_replay = reloaded
        .predict_matched(
            &table,
            &base_overlay,
            FACETED_BINDING_TURN,
            FOCUS_TURN,
            ACTIVE_QUERY,
        )
        .unwrap();
    assert_eq!(hierarchy_replay, hierarchy);
    assert_eq!(spiral_census_replay, spiral);
    assert_eq!(faceted_census_replay, faceted);
    let replay_target_free_census = TargetFreeCensus {
        schema: 1,
        domain: "uor-r4.conversation-entity-spin-path-target-free-census/1",
        table_cid: table.artifact_cid(),
        base_overlay_cid: base_overlay.artifact_cid(),
        operator_cid: operator_cid.clone(),
        codec_kappa: reloaded.codec_kappa().to_owned(),
        vocabulary_kappa: reloaded.vocabulary_kappa().to_owned(),
        route_manifest_kappa: reloaded.route_manifest_kappa().to_owned(),
        spin_map_kappa: reloaded.spin_map_kappa().to_owned(),
        grammar_kappa: reloaded.grammar_kappa().to_owned(),
        routing_policy_kappa: reloaded.routing_policy_kappa().to_owned(),
        hierarchy_audit_policy_kappa: reloaded.hierarchy_audit_policy_kappa().to_owned(),
        h4_root_table_kappa: reloaded.h4_root_table_kappa().to_owned(),
        h4_multiplication_table_kappa: reloaded.h4_multiplication_table_kappa().to_owned(),
        hierarchy: &hierarchy_replay,
        cases: vec![
            TargetFreeCase {
                partition_id: "45",
                prompt_cid: format!("blake3:{}", blake3::hash(&held_out[0].text).to_hex()),
                prediction: &spiral_census_replay,
            },
            TargetFreeCase {
                partition_id: "48",
                prompt_cid: format!("blake3:{}", blake3::hash(&held_out[1].text).to_hex()),
                prediction: &faceted_census_replay,
            },
        ],
        teacher_calls: 0,
        provider_calls: 0,
        source_weight_reads: 0,
        future_unit_reads: 0,
        target_reads: 0,
        partition_id_reads: 0,
        full_history_key_reads: 0,
        global_operator_reads: 0,
    };
    let replay_target_free_bytes = serde_json::to_vec(&replay_target_free_census).unwrap();
    assert_eq!(replay_target_free_bytes, target_free_bytes);
    assert_eq!(
        format!(
            "blake3:{}",
            blake3::hash(&replay_target_free_bytes).to_hex()
        ),
        target_free_cid
    );

    // Publicly predeclared targets are joined only after the source-free
    // operator, hierarchy audit, predictions, and complete census replay have
    // frozen above.
    let sealed = [
        ("45", SPIRAL_BINDING_TURN, b" silver.".as_slice()),
        ("48", FACETED_BINDING_TURN, b" violet.".as_slice()),
    ];
    let spiral_continuation = reloaded
        .continue_matched(
            &table,
            &base_overlay,
            sealed[0].1,
            FOCUS_TURN,
            ACTIVE_QUERY,
            3,
        )
        .unwrap();
    let faceted_continuation = reloaded
        .continue_matched(
            &table,
            &base_overlay,
            sealed[1].1,
            FOCUS_TURN,
            ACTIVE_QUERY,
            3,
        )
        .unwrap();
    let continuations = [&spiral_continuation, &faceted_continuation];
    let mut real_correct = 0_u32;
    let mut disabled_correct = 0_u32;
    let mut permuted_correct = 0_u32;
    let mut reversed_correct = 0_u32;
    for (index, continuation) in continuations.iter().enumerate() {
        let target = sealed[index].2;
        real_correct += u32::from(continuation.real.decoded == target);
        disabled_correct += u32::from(continuation.conversation_disabled.decoded == target);
        permuted_correct += u32::from(continuation.cross_turn_binding_permuted.decoded == target);
        reversed_correct += u32::from(continuation.binding_rows_reversed.decoded == target);
        for arm in [
            &continuation.real,
            &continuation.conversation_disabled,
            &continuation.cross_turn_binding_permuted,
            &continuation.binding_rows_reversed,
        ] {
            assert_eq!(arm.stop, ContinuationStop::EndOfDocument);
            assert_eq!(arm.tokens.len(), 2);
        }
        assert!(continuation.first_decision.support_matched);
        assert!(continuation.first_decision.work_matched);
    }
    assert_eq!(spiral_continuation.real.decoded, b" silver.");
    assert_eq!(faceted_continuation.real.decoded, b" violet.");
    assert_eq!(
        spiral_continuation.conversation_disabled.decoded,
        b" silver."
    );
    assert_eq!(
        faceted_continuation.conversation_disabled.decoded,
        b" silver."
    );
    assert_eq!(
        spiral_continuation.cross_turn_binding_permuted.decoded,
        b" violet."
    );
    assert_eq!(
        faceted_continuation.cross_turn_binding_permuted.decoded,
        b" silver."
    );
    assert_eq!(
        spiral_continuation.binding_rows_reversed.decoded,
        b" silver."
    );
    assert_eq!(
        faceted_continuation.binding_rows_reversed.decoded,
        b" violet."
    );
    assert_eq!(real_correct, 2);
    assert_eq!(disabled_correct, 1);
    assert_eq!(permuted_correct, 0);
    assert_eq!(reversed_correct, 2);

    let decoded_smoke = DecodedSmoke {
        schema: 1,
        domain: "uor-r4.conversation-entity-spin-path-decoded-smoke/1",
        operator_cid: operator_cid.clone(),
        cases: vec![
            DecodedCase {
                partition_id: sealed[0].0,
                target_hex: hex::encode(sealed[0].2),
                continuation: &spiral_continuation,
            },
            DecodedCase {
                partition_id: sealed[1].0,
                target_hex: hex::encode(sealed[1].2),
                continuation: &faceted_continuation,
            },
        ],
        real_correct,
        disabled_correct,
        permuted_correct,
        reversed_correct,
        support_mismatches: 0,
        work_mismatches: 0,
        terminal: "RETAIN_CONVERSATION_ENTITY_SPIN_PATH_ATTENTION_CONTINUE_BOUNDED_GLOBAL",
    };
    let decoded_bytes = serde_json::to_vec(&decoded_smoke).unwrap();
    let decoded_cid = format!("blake3:{}", blake3::hash(&decoded_bytes).to_hex());

    let spiral_replay = reloaded
        .continue_matched(
            &table,
            &base_overlay,
            SPIRAL_BINDING_TURN,
            FOCUS_TURN,
            ACTIVE_QUERY,
            3,
        )
        .unwrap();
    let faceted_replay = reloaded
        .continue_matched(
            &table,
            &base_overlay,
            FACETED_BINDING_TURN,
            FOCUS_TURN,
            ACTIVE_QUERY,
            3,
        )
        .unwrap();
    assert_eq!(spiral_replay, spiral_continuation);
    assert_eq!(faceted_replay, faceted_continuation);
    let replay_smoke = DecodedSmoke {
        schema: 1,
        domain: "uor-r4.conversation-entity-spin-path-decoded-smoke/1",
        operator_cid: operator_cid.clone(),
        cases: vec![
            DecodedCase {
                partition_id: sealed[0].0,
                target_hex: hex::encode(sealed[0].2),
                continuation: &spiral_replay,
            },
            DecodedCase {
                partition_id: sealed[1].0,
                target_hex: hex::encode(sealed[1].2),
                continuation: &faceted_replay,
            },
        ],
        real_correct,
        disabled_correct,
        permuted_correct,
        reversed_correct,
        support_mismatches: 0,
        work_mismatches: 0,
        terminal: "RETAIN_CONVERSATION_ENTITY_SPIN_PATH_ATTENTION_CONTINUE_BOUNDED_GLOBAL",
    };
    assert_eq!(serde_json::to_vec(&replay_smoke).unwrap(), decoded_bytes);
    assert_eq!(reloaded.to_bytes().unwrap(), operator_bytes);

    println!(
        "table_cid={}\nbase_overlay_cid={}\noperator_bytes={}\noperator_cid={operator_cid}\nconstruction_codec_kappa={}\nconstruction_vocabulary_kappa={}\nconstruction_route_manifest_kappa={}\naudit_codec_kappa={}\naudit_vocabulary_kappa={}\naudit_left_route_manifest_kappa={}\naudit_right_route_manifest_kappa={}\nspin_map_kappa={}\ngrammar_kappa={}\nrouting_policy_kappa={}\nhierarchy_audit_policy_kappa={}\nh4_root_table_kappa={}\nh4_multiplication_table_kappa={}\nglobal_epoch={}\ntarget_free_census_cid={target_free_cid}\ndecoded_smoke_cid={decoded_cid}\nreal=2/2\nconversation_disabled=1/2\ncross_turn_binding_permuted=0/2\nbinding_rows_reversed=2/2\nsupport_mismatches=0\nwork_mismatches=0\nterminal=RETAIN_CONVERSATION_ENTITY_SPIN_PATH_ATTENTION_CONTINUE_BOUNDED_GLOBAL",
        table.artifact_cid(),
        base_overlay.artifact_cid(),
        operator_bytes.len(),
        reloaded.codec_kappa(),
        reloaded.vocabulary_kappa(),
        reloaded.route_manifest_kappa(),
        hierarchy.codec_kappa.as_str(),
        hierarchy.vocabulary_kappa.as_str(),
        hierarchy.left_route_manifest_kappa.as_str(),
        hierarchy.right_route_manifest_kappa.as_str(),
        reloaded.spin_map_kappa(),
        reloaded.grammar_kappa(),
        reloaded.routing_policy_kappa(),
        reloaded.hierarchy_audit_policy_kappa(),
        reloaded.h4_root_table_kappa(),
        reloaded.h4_multiplication_table_kappa(),
        reloaded.global_epoch().unwrap(),
    );
}

#[test]
fn operator_rejects_binding_drift_tamper_malformed_scope_and_bounds() {
    let construction = construction_documents();
    let table = SourceFreeTable::compile(&construction).unwrap();
    let base_overlay = MultiscaleCountRadiusR4V1::compile(&table).unwrap();
    let operator =
        ConversationEntitySpinPathR4V1::compile(&table, &base_overlay, &construction).unwrap();
    let bytes = operator.to_bytes().unwrap();

    let mut tampered = bytes.clone();
    let final_index = tampered.len() - 1;
    tampered[final_index] ^= 1;
    assert!(ConversationEntitySpinPathR4V1::from_bytes(
        &table,
        &base_overlay,
        &construction,
        &tampered
    )
    .is_err());
    assert!(ConversationEntitySpinPathR4V1::from_bytes(
        &table,
        &base_overlay,
        &construction,
        &vec![0_u8; MAX_CONVERSATION_ENTITY_OPERATOR_BYTES + 1]
    )
    .is_err());

    let other_construction = vec![
        SourceDocument::new(
            "27",
            b"Nora carried the plain marker.\n\nNora opened the registry. Owen waited.\n\nThe active registry code is silver."
                .to_vec(),
        ),
        construction[1].clone(),
    ];
    assert!(ConversationEntitySpinPathR4V1::from_bytes(
        &table,
        &base_overlay,
        &other_construction,
        &bytes
    )
    .is_err());
    let other_table = SourceFreeTable::compile(&other_construction).unwrap();
    let other_overlay = MultiscaleCountRadiusR4V1::compile(&other_table).unwrap();
    assert!(ConversationEntitySpinPathR4V1::from_bytes(
        &other_table,
        &other_overlay,
        &other_construction,
        &bytes
    )
    .is_err());

    let mut reversed_construction = construction.clone();
    reversed_construction.reverse();
    let reversed =
        ConversationEntitySpinPathR4V1::compile(&table, &base_overlay, &reversed_construction)
            .unwrap();
    assert_eq!(reversed.to_bytes().unwrap(), bytes);

    assert!(operator
        .predict_matched(
            &table,
            &base_overlay,
            b"Mara carried the spiral marker.",
            FOCUS_TURN,
            ACTIVE_QUERY
        )
        .is_err());
    assert!(operator
        .predict_matched(
            &table,
            &base_overlay,
            b"Mara carried the spiral marker. Mara carried the faceted marker.",
            FOCUS_TURN,
            ACTIVE_QUERY
        )
        .is_err());
    assert!(operator
        .predict_matched(
            &table,
            &base_overlay,
            b"Mara carried the unknown marker. Iris carried the faceted marker.",
            FOCUS_TURN,
            ACTIVE_QUERY
        )
        .is_err());
    assert!(operator
        .predict_matched(
            &table,
            &base_overlay,
            SPIRAL_BINDING_TURN,
            b"Juno opened the registry. Iris waited.",
            ACTIVE_QUERY
        )
        .is_err());
    assert!(operator
        .predict_matched(
            &table,
            &base_overlay,
            SPIRAL_BINDING_TURN,
            b"Mara opened the registry. Mara waited.",
            ACTIVE_QUERY
        )
        .is_err());
    assert!(operator
        .predict_matched(
            &table,
            &base_overlay,
            SPIRAL_BINDING_TURN,
            FOCUS_TURN,
            b"The active registry code was"
        )
        .is_err());
    assert!(operator
        .predict_matched(
            &table,
            &base_overlay,
            b"Mara  carried the spiral marker. Iris carried the faceted marker.",
            FOCUS_TURN,
            ACTIVE_QUERY
        )
        .is_err());
    assert!(operator
        .predict_matched(
            &table,
            &base_overlay,
            SPIRAL_BINDING_TURN,
            b"Mara  opened the registry. Iris waited.",
            ACTIVE_QUERY
        )
        .is_err());
    assert!(operator
        .audit_hierarchy_pair(
            SPIRAL_BINDING_TURN,
            SPIRAL_BINDING_TURN,
            FOCUS_TURN,
            ACTIVE_QUERY
        )
        .is_err());

    let long_mara = vec![b'M'; MAX_CONVERSATION_ENTITY_BYTES];
    let mut excessive_binding = long_mara.clone();
    excessive_binding
        .extend_from_slice(b" carried the spiral marker. Iris carried the faceted marker.");
    assert!(excessive_binding.len() > MAX_CONVERSATION_ENTITY_BYTES);
    assert!(operator
        .predict_matched(
            &table,
            &base_overlay,
            &excessive_binding,
            FOCUS_TURN,
            ACTIVE_QUERY
        )
        .is_err());
    assert!(operator
        .continue_matched(
            &table,
            &base_overlay,
            SPIRAL_BINDING_TURN,
            FOCUS_TURN,
            ACTIVE_QUERY,
            0
        )
        .is_err());
}
