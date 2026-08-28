use uor_r4_graph_certify::corpus_signed_transport_attention::{
    certify_corpus_signed_transport_pregeometry, ObservedCorpusCodecSplitReadiness,
    TERMINAL_PRECEDENCE, UNAVAILABLE_FRAME_OR_POPULATION,
};

const AUDITED_MAIN: &str = "5de08b7e21cdd100a89782f480dc129cef3cd04d";
const RAW_MANIFEST_BYTES_CID: &str =
    "blake3:bb5f446ce92df60f7824ed5a1f04ede385386e7a47b9c198ae83a5d0f907bab3";
const CORPUS_BYTES_CID: &str =
    "blake3:194db0eebf2d49823ece01ee935447a0cc9edeaf018454ceea480ce7590132cf";

fn cid(fill: char) -> String {
    format!("blake3:{}", fill.to_string().repeat(64))
}

/// Structural control only: these synthetic CIDs are not #986 population
/// evidence and cannot satisfy the independent lexical `O(x)` prerequisite.
fn synthetic_population_ready_control() -> ObservedCorpusCodecSplitReadiness {
    ObservedCorpusCodecSplitReadiness {
        audited_main_commit: AUDITED_MAIN.to_owned(),
        raw_manifest_bytes_cid: Some(cid('1')),
        declared_corpus_bytes_cid: Some(cid('2')),
        reproduced_corpus_bytes_cid: Some(cid('2')),
        codec_cid: Some(cid('3')),
        split_commitment_cid: Some(cid('4')),
        source_document_count: 48,
        construction_document_count: 32,
        held_out_document_count: 16,
        construction_lexical_route_count: 256,
        calibration_pair_count: 16,
        calibration_decision_count: 32,
        sealed_test_pair_count: 32,
        sealed_test_decision_count: 64,
        source_free_observation_corpus: true,
        canonical_codec_reproduces: true,
        document_cid_partition_verified: true,
        anti_recall_disjointness_verified: true,
        natural_candidate_rule_frozen: true,
        pair_selection_key_frozen: true,
    }
}

fn observed_source_facts() -> ObservedCorpusCodecSplitReadiness {
    ObservedCorpusCodecSplitReadiness {
        audited_main_commit: AUDITED_MAIN.to_owned(),
        raw_manifest_bytes_cid: Some(RAW_MANIFEST_BYTES_CID.to_owned()),
        declared_corpus_bytes_cid: Some(CORPUS_BYTES_CID.to_owned()),
        reproduced_corpus_bytes_cid: Some(CORPUS_BYTES_CID.to_owned()),
        codec_cid: None,
        split_commitment_cid: None,
        source_document_count: 3_000,
        construction_document_count: 2_404,
        held_out_document_count: 596,
        ..ObservedCorpusCodecSplitReadiness::default()
    }
}

#[test]
fn exact_spiralcore_control_does_not_fabricate_a_lexical_operator_map() {
    let report = certify_corpus_signed_transport_pregeometry(synthetic_population_ready_control());

    assert!(report.exact_population_available);
    assert!(report.spiralcore.exact_operator_and_table_reproduce);
    assert_eq!(report.spiralcore.chart_transport_status, "NOT_ESTABLISHED");
    assert_eq!(
        report.spiralcore.operator_semantic_status,
        "OPTIONAL_CONTROL_PENDING"
    );
    assert!(!report.spiralcore.cross_chart_transport_established);
    assert!(report.spiralcore.lexical_binding_manifest_cid.is_none());
    assert_eq!(report.spiralcore.verified_bound_lexical_route_count, 0);
    assert!(
        !report
            .spiralcore
            .complete_same_frame_lexical_operator_binding
    );
    assert!(!report.spiralcore.compiler_query_frame_identity_verified);
    assert_eq!(report.terminal, UNAVAILABLE_FRAME_OR_POPULATION);
    assert_eq!(report.terminal_precedence, TERMINAL_PRECEDENCE);
    assert_eq!(
        report.unavailable_reasons,
        ["COMPLETE_SAME_FRAME_LEXICAL_O_X_BINDING_UNAVAILABLE"]
    );
    assert!(!report.placement_started);
    assert!(!report.diffusion_started);
    assert!(!report.gate0_started);
    assert!(!report.calibration_started);
    assert!(!report.sealed_label_path_started);
}

#[test]
fn observed_source_facts_stop_at_the_same_presealed_terminal() {
    let report = certify_corpus_signed_transport_pregeometry(observed_source_facts());

    assert!(!report.exact_population_available);
    assert!(report.raw_manifest_bytes_cid_is_canonical);
    assert!(report.corpus_bytes_cid_reproduces);
    assert!(report.source_partition_counts_reproduce);
    assert_eq!(report.observed.source_document_count, 3_000);
    assert_eq!(report.observed.construction_document_count, 2_404);
    assert_eq!(report.observed.held_out_document_count, 596);
    assert!(report.observed.codec_cid.is_none());
    assert!(report.observed.split_commitment_cid.is_none());
    assert_eq!(report.terminal, UNAVAILABLE_FRAME_OR_POPULATION);
    assert!(report
        .unavailable_reasons
        .contains(&"EXACT_CORPUS_CODEC_SPLIT_POPULATION_UNAVAILABLE"));
    assert!(report
        .unavailable_reasons
        .contains(&"COMPLETE_SAME_FRAME_LEXICAL_O_X_BINDING_UNAVAILABLE"));
}

#[test]
fn canonical_json_and_cid_replay_byte_identically() {
    let observation = observed_source_facts();
    let first = certify_corpus_signed_transport_pregeometry(observation.clone());
    let second = certify_corpus_signed_transport_pregeometry(observation);

    assert_eq!(first, second);
    assert_eq!(first.canonical_json_bytes(), second.canonical_json_bytes());
    assert_eq!(first.cid(), second.cid());
    assert_eq!(
        first.canonical_json_bytes(),
        serde_json::to_vec(&first).expect("certificate serializes")
    );

    let decoded: serde_json::Value =
        serde_json::from_slice(&first.canonical_json_bytes()).expect("canonical JSON parses");
    assert_eq!(decoded["terminal"], UNAVAILABLE_FRAME_OR_POPULATION);
    assert_eq!(decoded["placement_started"], false);
    assert_eq!(decoded["sealed_label_path_started"], false);

    assert_eq!(
        first.cid(),
        "blake3:3fff541e4ac37193babaacd25227019fb401950ccdd936ab38ac46c6c2916337",
        "the unavailable prerequisite certificate CID is pinned"
    );
}
