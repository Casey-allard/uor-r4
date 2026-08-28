//! Frozen non-copy paragraph probe for #973 `ParagraphEntitySpinPathR4V1`.
//!
//! The fixture is synthetic. D3 separation prevents held-out prompt bytes from
//! entering fitting; it does not establish natural-distribution transfer.

use serde::Serialize;

use uor_r4_core::canonical_lexical_ingestion::canonical_lexical_piece_bytes;
use uor_r4_core::higher_scope_geometric_attention::{
    PriorSentenceCountRadiusAbstention, PriorSentenceCountRadiusR4V1,
};
use uor_r4_core::paragraph_entity_spin_path_attention::{
    MatchedParagraphEntitySpinPathContinuation, MatchedParagraphEntitySpinPathPrediction,
    ParagraphEntitySpinPathCost, ParagraphEntitySpinPathR4V1,
};
use uor_r4_core::prime_route_geometric_attention::H4S3AngularShell;
use uor_r4_core::source_free_table::{
    d3_is_held_out, BackoffOrder, ContinuationStop, MultiscaleCountRadiusR4V1, SourceDocument,
    SourceFreeTable, BOS_TOKEN,
};

const STRIPED_PRIOR: &[u8] = b"Mara carried the striped marker. Iris carried the dotted marker.";
const DOTTED_PRIOR: &[u8] = b"Mara carried the dotted marker. Iris carried the striped marker.";
const ACTIVE_QUERY: &[u8] = b"For Mara the registry code is";

fn construction_documents() -> Vec<SourceDocument> {
    vec![
        SourceDocument::new(
            "20",
            b"Nora carried the striped marker.\n\nFor Nora the registry code is amber.".to_vec(),
        ),
        SourceDocument::new(
            "21",
            b"Owen carried the dotted marker.\n\nFor Owen the registry code is cobalt.".to_vec(),
        ),
    ]
}

fn prompt(prior: &[u8]) -> Vec<u8> {
    let mut bytes = prior.to_vec();
    bytes.extend_from_slice(b"\n\n");
    bytes.extend_from_slice(ACTIVE_QUERY);
    bytes
}

fn context(table: &SourceFreeTable, prior: &[u8]) -> Vec<u32> {
    let mut context = vec![BOS_TOKEN];
    context.extend(table.encode_text(&prompt(prior)).unwrap());
    context
}

fn decoded(table: &SourceFreeTable, token: u32) -> Vec<u8> {
    table.decode_tokens(&[token]).unwrap()
}

fn decoded_support(table: &SourceFreeTable, tokens: &[u32]) -> Vec<Vec<u8>> {
    tokens.iter().map(|token| decoded(table, *token)).collect()
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
    h4_root_table_kappa: String,
    h4_multiplication_table_kappa: String,
    cases: Vec<TargetFreeCase<'a>>,
    teacher_calls: u64,
    provider_calls: u64,
    source_weight_reads: u64,
    future_unit_reads: u64,
    target_reads: u64,
}

#[derive(Serialize)]
struct TargetFreeCase<'a> {
    partition_id: &'static str,
    prompt_cid: String,
    prediction: &'a MatchedParagraphEntitySpinPathPrediction,
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
    continuation: &'a MatchedParagraphEntitySpinPathContinuation,
}

#[test]
fn target_free_spin_path_census_then_sealed_decoded_controls_are_load_bearing() {
    let construction = construction_documents();
    assert!(construction
        .iter()
        .all(|document| !d3_is_held_out(&document.id)));
    assert!(d3_is_held_out("26"));
    assert!(d3_is_held_out("38"));

    let held_out = [
        SourceDocument::new("26", prompt(STRIPED_PRIOR)),
        SourceDocument::new("38", prompt(DOTTED_PRIOR)),
    ];
    for document in &held_out {
        assert!(construction.iter().all(|source| {
            source.id != document.id && source.text_cid() != document.text_cid()
        }));
        assert!(!document
            .text
            .windows(b" amber".len())
            .any(|window| window == b" amber"));
        assert!(!document
            .text
            .windows(b" cobalt".len())
            .any(|window| window == b" cobalt"));
        assert!(!document
            .text
            .windows(b"Nora".len())
            .any(|window| window == b"Nora"));
        assert!(!document
            .text
            .windows(b"Owen".len())
            .any(|window| window == b"Owen"));
    }
    let mut striped_multiset = canonical_lexical_piece_bytes(&held_out[0].text).unwrap();
    let mut dotted_multiset = canonical_lexical_piece_bytes(&held_out[1].text).unwrap();
    striped_multiset.sort();
    dotted_multiset.sort();
    assert_eq!(striped_multiset, dotted_multiset);

    let table = SourceFreeTable::compile(&construction).unwrap();
    let base_overlay = MultiscaleCountRadiusR4V1::compile(&table).unwrap();
    let operator =
        ParagraphEntitySpinPathR4V1::compile(&table, &base_overlay, &construction).unwrap();
    let table_bytes = table.to_bytes();
    let base_overlay_bytes = base_overlay.to_bytes();
    let operator_bytes = operator.to_bytes().unwrap();
    let operator_cid = operator.artifact_cid().unwrap();
    let table = SourceFreeTable::from_bytes(&table_bytes).unwrap();
    let base_overlay = MultiscaleCountRadiusR4V1::from_bytes(&table, &base_overlay_bytes).unwrap();
    assert_eq!(table.to_bytes(), table_bytes);
    assert_eq!(base_overlay.to_bytes(), base_overlay_bytes);
    let reloaded = ParagraphEntitySpinPathR4V1::from_bytes(
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

    // Target-free census. No expected continuation is constructed above this
    // point, and neither admitted candidate occurs in either prompt.
    let striped = reloaded
        .predict_matched(&table, &base_overlay, STRIPED_PRIOR, ACTIVE_QUERY)
        .unwrap();
    let dotted = reloaded
        .predict_matched(&table, &base_overlay, DOTTED_PRIOR, ACTIVE_QUERY)
        .unwrap();
    for prediction in [&striped, &dotted] {
        assert_eq!(prediction.local.order, BackoffOrder::Trigram);
        assert_eq!(prediction.local.max_count, 1);
        assert!(prediction.local.geometry_reachable);
        assert_eq!(
            decoded_support(&table, &prediction.local.max_count_tie_tokens),
            vec![b" amber".to_vec(), b" cobalt".to_vec()]
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
        assert_eq!(decoded(&table, prediction.local.geometric_token), b" amber");
        assert_eq!(prediction.prior_candidate_occurrences, 0);
        assert!(prediction.support_matched);
        assert!(prediction.work_matched);
        assert_eq!(prediction.operator_abstention, None);
        assert_eq!(prediction.teacher_calls, 0);
        assert_eq!(prediction.provider_calls, 0);
        assert_eq!(prediction.source_weight_reads, 0);
        assert_eq!(prediction.future_unit_reads, 0);
        assert_eq!(prediction.target_reads, 0);
        assert_eq!(prediction.candidate_evidence.len(), 2);
        assert_eq!(prediction.real.work.prior_fact_slots_scanned, 2);
        assert_eq!(prediction.real.work.entity_key_comparisons, 2);
        assert_eq!(prediction.real.work.descriptor_row_comparisons, 2);
        assert_eq!(prediction.real.work.stored_spin_leaf_reads, 4);
        assert_eq!(prediction.real.work.h4_product_table_reads, 6);
        assert_eq!(prediction.real.work.h4_inverse_table_reads, 2);
        assert_eq!(prediction.real.work.phase_additions, 12);
        assert_eq!(prediction.real.work.phase_distance_reads, 4);
        assert_eq!(prediction.real.work.angular_shell_reads, 2);
        assert_eq!(prediction.real.work.cost_comparisons, 2);
        assert_eq!(prediction.real.work.final_choice_operations, 1);
        assert_eq!(prediction.real.work, prediction.paragraph_disabled.work);
        assert_eq!(
            prediction.real.work,
            prediction.entity_binding_permuted.work
        );
        assert_eq!(prediction.real.work, prediction.fact_order_reversed.work);
        for candidate in &prediction.local.tie_evidence {
            assert_eq!(candidate.count, 1);
            assert_eq!(candidate.coordinates.trigram_q32, 1_u64 << 31);
            assert_eq!(candidate.coordinates.bigram_q32, 1_u64 << 31);
            assert_eq!(candidate.coordinates.unigram_q32, 143_165_576);
            assert_eq!(candidate.coordinates.depth_q32, 3_u64 << 30);
            assert_eq!(candidate.radius, 19_620_161_960_467_810_368);
        }
    }
    assert_eq!(striped.local.tie_evidence, dotted.local.tie_evidence);
    assert_eq!(striped.local.geometric_work, dotted.local.geometric_work);
    assert_eq!(context(&table, STRIPED_PRIOR).len() - 1, 29);
    assert_eq!(context(&table, DOTTED_PRIOR).len() - 1, 29);

    assert_eq!(decoded(&table, striped.real.token), b" amber");
    assert_eq!(decoded(&table, dotted.real.token), b" cobalt");
    assert_eq!(decoded(&table, striped.paragraph_disabled.token), b" amber");
    assert_eq!(decoded(&table, dotted.paragraph_disabled.token), b" amber");
    assert_eq!(
        decoded(&table, striped.entity_binding_permuted.token),
        b" cobalt"
    );
    assert_eq!(
        decoded(&table, dotted.entity_binding_permuted.token),
        b" amber"
    );
    assert_eq!(striped.fact_order_reversed.token, striped.real.token);
    assert_eq!(dotted.fact_order_reversed.token, dotted.real.token);
    assert_ne!(striped.real.token, dotted.real.token);

    let prototypes = reloaded.prototype_traces().unwrap();
    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        hex::decode(&prototypes[0].candidate_hex).unwrap(),
        b" amber"
    );
    assert_eq!(
        hex::decode(&prototypes[1].candidate_hex).unwrap(),
        b" cobalt"
    );
    assert_eq!(
        hex::decode(&prototypes[0].descriptor_hex).unwrap(),
        b"striped"
    );
    assert_eq!(
        hex::decode(&prototypes[1].descriptor_hex).unwrap(),
        b"dotted"
    );
    assert_eq!(prototypes[0].leaves.len(), 4);
    assert_eq!(prototypes[1].leaves.len(), 4);
    for (prototype, payload) in prototypes.iter().zip([b"amber".as_slice(), b"cobalt"]) {
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
    }
    assert_eq!(
        prototypes[0].path_state.h4_coordinate,
        prototypes[1].path_state.h4_coordinate
    );
    assert_eq!(prototypes[0].path_state.fiber_q29, 160_683_320);
    assert_eq!(prototypes[0].path_state.torsion_q29, -15_268_680);
    assert_eq!(prototypes[1].path_state.fiber_q29, 144_614_988);
    assert_eq!(prototypes[1].path_state.torsion_q29, -13_741_812);
    let striped_leaf = &prototypes[0].leaves[2];
    let dotted_leaf = &prototypes[1].leaves[2];
    assert_eq!(striped_leaf.s3_q30, dotted_leaf.s3_q30);
    assert_eq!(striped_leaf.hopf_q30, dotted_leaf.hopf_q30);
    assert_eq!(
        striped_leaf.mapped_h4_coordinate,
        dotted_leaf.mapped_h4_coordinate
    );
    assert_ne!(striped_leaf.fiber_q29, dotted_leaf.fiber_q29);
    assert_ne!(striped_leaf.torsion_q29, dotted_leaf.torsion_q29);
    assert_eq!(striped_leaf.payload_cid, canonical_payload_cid(b"striped"));
    assert_eq!(dotted_leaf.payload_cid, canonical_payload_cid(b"dotted"));
    assert!(striped_leaf.prime >= 2);
    assert!(dotted_leaf.prime >= 2);

    let zero_cost = ParagraphEntitySpinPathCost {
        angular_shell: H4S3AngularShell::Coincident,
        fiber_distance_q29: 0,
        torsion_distance_q29: 0,
    };
    let nonmatching_cost = ParagraphEntitySpinPathCost {
        angular_shell: H4S3AngularShell::Coincident,
        fiber_distance_q29: 16_068_332,
        torsion_distance_q29: 1_526_868,
    };
    assert_eq!(striped.candidate_evidence[0].real.measured_cost, zero_cost);
    assert_eq!(
        striped.candidate_evidence[1].real.measured_cost,
        nonmatching_cost
    );
    assert_eq!(
        dotted.candidate_evidence[0].real.measured_cost,
        nonmatching_cost
    );
    assert_eq!(dotted.candidate_evidence[1].real.measured_cost, zero_cost);
    assert!(striped.candidate_evidence.iter().all(|candidate| candidate
        .real
        .measured_cost
        .angular_shell
        == H4S3AngularShell::Coincident));
    assert!(dotted.candidate_evidence.iter().all(|candidate| candidate
        .real
        .measured_cost
        .angular_shell
        == H4S3AngularShell::Coincident));
    for prediction in [&striped, &dotted] {
        for candidate in &prediction.candidate_evidence {
            assert_eq!(
                candidate.real.ranking_cost,
                Some(candidate.real.measured_cost)
            );
            assert_eq!(candidate.paragraph_disabled.ranking_cost, None);
            assert_eq!(
                candidate.entity_binding_permuted.ranking_cost,
                Some(candidate.entity_binding_permuted.measured_cost)
            );
            assert_eq!(
                candidate.fact_order_reversed.ranking_cost,
                Some(candidate.fact_order_reversed.measured_cost)
            );
        }
        assert_eq!(prediction.paragraph_disabled.minimum_cost, None);
    }

    let prior_copy = PriorSentenceCountRadiusR4V1::compile(&table, &base_overlay).unwrap();
    for prior in [STRIPED_PRIOR, DOTTED_PRIOR] {
        let copy = prior_copy
            .predict_matched(&table, &base_overlay, &context(&table, prior))
            .unwrap();
        assert_eq!(
            copy.operator_abstention,
            Some(PriorSentenceCountRadiusAbstention::NoPriorCandidateOccurrence)
        );
        assert_eq!(decoded(&table, copy.real.token), b" amber");
    }

    let target_free_census = TargetFreeCensus {
        schema: 1,
        domain: "uor-r4.paragraph-entity-spin-path-target-free-census/1",
        table_cid: table.artifact_cid(),
        base_overlay_cid: base_overlay.artifact_cid(),
        operator_cid: operator_cid.clone(),
        codec_kappa: reloaded.codec_kappa().to_owned(),
        vocabulary_kappa: reloaded.vocabulary_kappa().to_owned(),
        route_manifest_kappa: reloaded.route_manifest_kappa().to_owned(),
        spin_map_kappa: reloaded.spin_map_kappa().to_owned(),
        grammar_kappa: reloaded.grammar_kappa().to_owned(),
        routing_policy_kappa: reloaded.routing_policy_kappa().to_owned(),
        h4_root_table_kappa: reloaded.h4_root_table_kappa().to_owned(),
        h4_multiplication_table_kappa: reloaded.h4_multiplication_table_kappa().to_owned(),
        cases: vec![
            TargetFreeCase {
                partition_id: "26",
                prompt_cid: format!("blake3:{}", blake3::hash(&held_out[0].text).to_hex()),
                prediction: &striped,
            },
            TargetFreeCase {
                partition_id: "38",
                prompt_cid: format!("blake3:{}", blake3::hash(&held_out[1].text).to_hex()),
                prediction: &dotted,
            },
        ],
        teacher_calls: 0,
        provider_calls: 0,
        source_weight_reads: 0,
        future_unit_reads: 0,
        target_reads: 0,
    };
    let target_free_bytes = serde_json::to_vec(&target_free_census).unwrap();
    let target_free_cid = format!("blake3:{}", blake3::hash(&target_free_bytes).to_hex());
    let striped_census_replay = reloaded
        .predict_matched(&table, &base_overlay, STRIPED_PRIOR, ACTIVE_QUERY)
        .unwrap();
    let dotted_census_replay = reloaded
        .predict_matched(&table, &base_overlay, DOTTED_PRIOR, ACTIVE_QUERY)
        .unwrap();
    let replay_target_free_census = TargetFreeCensus {
        schema: 1,
        domain: "uor-r4.paragraph-entity-spin-path-target-free-census/1",
        table_cid: table.artifact_cid(),
        base_overlay_cid: base_overlay.artifact_cid(),
        operator_cid: operator_cid.clone(),
        codec_kappa: reloaded.codec_kappa().to_owned(),
        vocabulary_kappa: reloaded.vocabulary_kappa().to_owned(),
        route_manifest_kappa: reloaded.route_manifest_kappa().to_owned(),
        spin_map_kappa: reloaded.spin_map_kappa().to_owned(),
        grammar_kappa: reloaded.grammar_kappa().to_owned(),
        routing_policy_kappa: reloaded.routing_policy_kappa().to_owned(),
        h4_root_table_kappa: reloaded.h4_root_table_kappa().to_owned(),
        h4_multiplication_table_kappa: reloaded.h4_multiplication_table_kappa().to_owned(),
        cases: vec![
            TargetFreeCase {
                partition_id: "26",
                prompt_cid: format!("blake3:{}", blake3::hash(&held_out[0].text).to_hex()),
                prediction: &striped_census_replay,
            },
            TargetFreeCase {
                partition_id: "38",
                prompt_cid: format!("blake3:{}", blake3::hash(&held_out[1].text).to_hex()),
                prediction: &dotted_census_replay,
            },
        ],
        teacher_calls: 0,
        provider_calls: 0,
        source_weight_reads: 0,
        future_unit_reads: 0,
        target_reads: 0,
    };
    let replay_target_free_bytes = serde_json::to_vec(&replay_target_free_census).unwrap();
    assert_eq!(striped_census_replay, striped);
    assert_eq!(dotted_census_replay, dotted);
    assert_eq!(replay_target_free_bytes, target_free_bytes);
    assert_eq!(
        format!(
            "blake3:{}",
            blake3::hash(&replay_target_free_bytes).to_hex()
        ),
        target_free_cid
    );

    // Sealed labels are joined only after the target-free operator and census
    // bytes above have been frozen.
    let sealed = [
        ("26", STRIPED_PRIOR, b" amber.".as_slice()),
        ("38", DOTTED_PRIOR, b" cobalt.".as_slice()),
    ];
    let striped_continuation = reloaded
        .continue_matched(&table, &base_overlay, sealed[0].1, ACTIVE_QUERY, 3)
        .unwrap();
    let dotted_continuation = reloaded
        .continue_matched(&table, &base_overlay, sealed[1].1, ACTIVE_QUERY, 3)
        .unwrap();
    let continuations = [&striped_continuation, &dotted_continuation];
    let mut real_correct = 0_u32;
    let mut disabled_correct = 0_u32;
    let mut permuted_correct = 0_u32;
    let mut reversed_correct = 0_u32;
    for (index, continuation) in continuations.iter().enumerate() {
        let target = sealed[index].2;
        real_correct += u32::from(continuation.real.decoded == target);
        disabled_correct += u32::from(continuation.paragraph_disabled.decoded == target);
        permuted_correct += u32::from(continuation.entity_binding_permuted.decoded == target);
        reversed_correct += u32::from(continuation.fact_order_reversed.decoded == target);
        for arm in [
            &continuation.real,
            &continuation.paragraph_disabled,
            &continuation.entity_binding_permuted,
            &continuation.fact_order_reversed,
        ] {
            assert_eq!(arm.stop, ContinuationStop::EndOfDocument);
            assert_eq!(arm.tokens.len(), 2);
        }
        assert!(continuation.first_decision.support_matched);
        assert!(continuation.first_decision.work_matched);
    }
    assert_eq!(striped_continuation.real.decoded, b" amber.");
    assert_eq!(dotted_continuation.real.decoded, b" cobalt.");
    assert_eq!(striped_continuation.paragraph_disabled.decoded, b" amber.");
    assert_eq!(dotted_continuation.paragraph_disabled.decoded, b" amber.");
    assert_eq!(
        striped_continuation.entity_binding_permuted.decoded,
        b" cobalt."
    );
    assert_eq!(
        dotted_continuation.entity_binding_permuted.decoded,
        b" amber."
    );
    assert_eq!(striped_continuation.fact_order_reversed.decoded, b" amber.");
    assert_eq!(dotted_continuation.fact_order_reversed.decoded, b" cobalt.");
    assert_eq!(real_correct, 2);
    assert_eq!(disabled_correct, 1);
    assert_eq!(permuted_correct, 0);
    assert_eq!(reversed_correct, 2);

    let decoded_smoke = DecodedSmoke {
        schema: 1,
        domain: "uor-r4.paragraph-entity-spin-path-decoded-smoke/1",
        operator_cid: operator_cid.clone(),
        cases: vec![
            DecodedCase {
                partition_id: sealed[0].0,
                target_hex: hex::encode(sealed[0].2),
                continuation: &striped_continuation,
            },
            DecodedCase {
                partition_id: sealed[1].0,
                target_hex: hex::encode(sealed[1].2),
                continuation: &dotted_continuation,
            },
        ],
        real_correct,
        disabled_correct,
        permuted_correct,
        reversed_correct,
        support_mismatches: 0,
        work_mismatches: 0,
        terminal: "RETAIN_PARAGRAPH_ENTITY_SPIN_PATH_ATTENTION_CONTINUE_CONVERSATION",
    };
    let decoded_bytes = serde_json::to_vec(&decoded_smoke).unwrap();
    let decoded_cid = format!("blake3:{}", blake3::hash(&decoded_bytes).to_hex());

    let striped_replay = reloaded
        .continue_matched(&table, &base_overlay, STRIPED_PRIOR, ACTIVE_QUERY, 3)
        .unwrap();
    let dotted_replay = reloaded
        .continue_matched(&table, &base_overlay, DOTTED_PRIOR, ACTIVE_QUERY, 3)
        .unwrap();
    assert_eq!(striped_replay, striped_continuation);
    assert_eq!(dotted_replay, dotted_continuation);
    let replay_smoke = DecodedSmoke {
        schema: 1,
        domain: "uor-r4.paragraph-entity-spin-path-decoded-smoke/1",
        operator_cid: operator_cid.clone(),
        cases: vec![
            DecodedCase {
                partition_id: sealed[0].0,
                target_hex: hex::encode(sealed[0].2),
                continuation: &striped_replay,
            },
            DecodedCase {
                partition_id: sealed[1].0,
                target_hex: hex::encode(sealed[1].2),
                continuation: &dotted_replay,
            },
        ],
        real_correct,
        disabled_correct,
        permuted_correct,
        reversed_correct,
        support_mismatches: 0,
        work_mismatches: 0,
        terminal: "RETAIN_PARAGRAPH_ENTITY_SPIN_PATH_ATTENTION_CONTINUE_CONVERSATION",
    };
    assert_eq!(serde_json::to_vec(&replay_smoke).unwrap(), decoded_bytes);
    assert_eq!(reloaded.to_bytes().unwrap(), operator_bytes);

    println!(
        "table_cid={}\nbase_overlay_cid={}\noperator_bytes={}\noperator_cid={operator_cid}\ncodec_kappa={}\nvocabulary_kappa={}\nroute_manifest_kappa={}\nspin_map_kappa={}\ngrammar_kappa={}\nrouting_policy_kappa={}\nh4_root_table_kappa={}\nh4_multiplication_table_kappa={}\ntarget_free_census_cid={target_free_cid}\ndecoded_smoke_cid={decoded_cid}\nreal=2/2\nparagraph_disabled=1/2\nentity_binding_permuted=0/2\nfact_order_reversed=2/2\nsupport_mismatches=0\nwork_mismatches=0\nterminal=RETAIN_PARAGRAPH_ENTITY_SPIN_PATH_ATTENTION_CONTINUE_CONVERSATION",
        table.artifact_cid(),
        base_overlay.artifact_cid(),
        operator_bytes.len(),
        reloaded.codec_kappa(),
        reloaded.vocabulary_kappa(),
        reloaded.route_manifest_kappa(),
        reloaded.spin_map_kappa(),
        reloaded.grammar_kappa(),
        reloaded.routing_policy_kappa(),
        reloaded.h4_root_table_kappa(),
        reloaded.h4_multiplication_table_kappa(),
    );
}

#[test]
fn operator_rejects_binding_drift_tamper_and_malformed_scope() {
    let construction = construction_documents();
    let table = SourceFreeTable::compile(&construction).unwrap();
    let base_overlay = MultiscaleCountRadiusR4V1::compile(&table).unwrap();
    let operator =
        ParagraphEntitySpinPathR4V1::compile(&table, &base_overlay, &construction).unwrap();
    let bytes = operator.to_bytes().unwrap();

    let mut tampered = bytes.clone();
    let final_index = tampered.len() - 1;
    tampered[final_index] ^= 1;
    assert!(ParagraphEntitySpinPathR4V1::from_bytes(
        &table,
        &base_overlay,
        &construction,
        &tampered
    )
    .is_err());

    let other_construction = vec![
        SourceDocument::new(
            "20",
            b"Nora carried the plain marker.\n\nFor Nora the registry code is amber.".to_vec(),
        ),
        SourceDocument::new(
            "21",
            b"Owen carried the dotted marker.\n\nFor Owen the registry code is cobalt.".to_vec(),
        ),
    ];
    assert!(ParagraphEntitySpinPathR4V1::from_bytes(
        &table,
        &base_overlay,
        &other_construction,
        &bytes
    )
    .is_err());
    let other_table = SourceFreeTable::compile(&other_construction).unwrap();
    let other_overlay = MultiscaleCountRadiusR4V1::compile(&other_table).unwrap();
    assert!(ParagraphEntitySpinPathR4V1::from_bytes(
        &other_table,
        &other_overlay,
        &other_construction,
        &bytes
    )
    .is_err());

    let mut reversed_construction = construction.clone();
    reversed_construction.reverse();
    let reversed =
        ParagraphEntitySpinPathR4V1::compile(&table, &base_overlay, &reversed_construction)
            .unwrap();
    assert_eq!(reversed.to_bytes().unwrap(), bytes);

    assert!(operator
        .predict_matched(
            &table,
            &base_overlay,
            b"Mara carried the striped marker.",
            ACTIVE_QUERY
        )
        .is_err());
    assert!(operator
        .predict_matched(
            &table,
            &base_overlay,
            b"Mara carried the striped marker. Mara carried the dotted marker.",
            ACTIVE_QUERY
        )
        .is_err());
    assert!(operator
        .predict_matched(
            &table,
            &base_overlay,
            b"Mara carried the unknown marker. Iris carried the dotted marker.",
            ACTIVE_QUERY
        )
        .is_err());
    assert!(operator
        .predict_matched(
            &table,
            &base_overlay,
            STRIPED_PRIOR,
            b"For Mara the registry code was"
        )
        .is_err());
    assert!(operator
        .predict_matched(
            &table,
            &base_overlay,
            b"Mara  carried the striped marker. Iris carried the dotted marker.",
            ACTIVE_QUERY
        )
        .is_err());
    assert!(operator
        .predict_matched(
            &table,
            &base_overlay,
            STRIPED_PRIOR,
            b"For  Mara the registry code is"
        )
        .is_err());
    assert!(operator
        .predict_matched(
            &table,
            &base_overlay,
            STRIPED_PRIOR,
            b"\tFor Mara the registry code is"
        )
        .is_err());
    let excessive = vec![b'x'; 1025];
    assert!(operator
        .predict_matched(&table, &base_overlay, &excessive, ACTIVE_QUERY)
        .is_err());
    let aggregate_entity = vec![b'M'; 500];
    let mut aggregate_prior = aggregate_entity.clone();
    aggregate_prior
        .extend_from_slice(b" carried the striped marker. Iris carried the dotted marker.");
    let mut aggregate_query = b"For ".to_vec();
    aggregate_query.extend_from_slice(&aggregate_entity);
    aggregate_query.extend_from_slice(b" the registry code is");
    assert!(aggregate_prior.len() < 1024);
    assert!(aggregate_query.len() < 1024);
    assert!(aggregate_prior.len() + 2 + aggregate_query.len() > 1024);
    assert!(operator
        .predict_matched(&table, &base_overlay, &aggregate_prior, &aggregate_query)
        .is_err());
    let long_entity = vec![b'x'; 80];
    let mut unit_excess_prior = long_entity.clone();
    unit_excess_prior
        .extend_from_slice(b" carried the striped marker. Iris carried the dotted marker.");
    let mut unit_excess_query = b"For ".to_vec();
    unit_excess_query.extend_from_slice(&long_entity);
    unit_excess_query.extend_from_slice(b" the registry code is");
    assert!(operator
        .predict_matched(
            &table,
            &base_overlay,
            &unit_excess_prior,
            &unit_excess_query
        )
        .is_err());
}
