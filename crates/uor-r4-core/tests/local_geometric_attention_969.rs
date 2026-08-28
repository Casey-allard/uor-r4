use serde::Serialize;

use uor_r4_core::canonical_lexical_ingestion::{
    canonical_global_epoch, validate_h4_binary_icosahedral_closure, CanonicalLexicalCodec,
    CanonicalRouteArtifact, ConversationInput, ParagraphInput, TurnInput,
};
use uor_r4_core::prime_route_attention::GeometricAddress;
use uor_r4_core::prime_route_geometric_attention::{
    AttentionQueryPolicy, AttentionRowKey, AttentionRowSource, AttentionSourceCounts,
    GeometricAttentionArtifact, H4S3AngularShell, PathLeaseAttentionTrace, PathLeaseControl,
    PathLeaseCost, ATTENTION_ADJACENT_SPIN_ROWS, ATTENTION_ROWS_PER_QUERY,
};

const REGISTERED_TOKENS: [&str; 10] = ["aa", "bb", "cc", "dd", "gg", "ll", "qq", "rr", "uu", "vv"];
const CONSTRUCTION_SENTENCES: [&[&str]; 7] = [
    &["uu", "ll"],
    &["vv", "rr"],
    &["aa"],
    &["bb"],
    &["cc"],
    &["dd"],
    &["qq"],
];
const LEFT_HISTORY: [&str; 4] = ["aa", "bb", "dd", "qq"];
const RIGHT_HISTORY: [&str; 4] = ["bb", "aa", "dd", "qq"];

const CONSTRUCTION_ARTIFACT_KAPPA: &str =
    "blake3:2b70588d654c8e8bb2d8ab063f41853d45a21487d742ff7567f93a42cfb9011b";
const ATTENTION_MANIFEST_KAPPA: &str =
    "blake3:1c77c4103732964af6776f1dfcabc8b2a9191eea875a8ba205c36ebbf5618a99";
const SMOKE_FIXTURE_IDENTITY_BYTES: &[u8] =
    b"uor-r4.local-geometric-attention-smoke/1\naa bb dd qq|bb aa dd qq\nll,rr\n2";
const SMOKE_FIXTURE_KAPPA: &str =
    "blake3:cc36b703b95bf1da11f2691ed91bbe94a81a4385f0eff8483cb9402191f46332";
const SMOKE_RECORD_KAPPA: &str =
    "blake3:60360a9e22a56ea4af363e43f7103bb8104d015d58feb582d921fc17afaf207f";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct SmokeRun {
    smoke_fixture_kappa: String,
    construction_artifact_kappa: String,
    attention_manifest_kappa: String,
    h4_root_table_kappa: String,
    h4_multiplication_table_kappa: String,
    left_continuation: Vec<Vec<u8>>,
    right_continuation: Vec<Vec<u8>>,
    left_full_costs: Vec<Vec<(Vec<u8>, PathLeaseCost)>>,
    right_full_costs: Vec<Vec<(Vec<u8>, PathLeaseCost)>>,
    last_only_outputs: [Option<Vec<u8>>; 2],
    state_disabled_outputs: [Option<Vec<u8>>; 2],
    path_geometry_evaluations_per_step: [usize; 2],
    maximum_hierarchy_changes_per_event: usize,
}

fn conversation_input(scope: &str, sentences: &[&[&str]]) -> ConversationInput {
    let global_snapshot_units = vec![b"gg".to_vec()];
    ConversationInput {
        identity_scope: scope.to_owned(),
        global_epoch: canonical_global_epoch(&global_snapshot_units).unwrap(),
        global_snapshot_units,
        turns: vec![TurnInput {
            turn_id: "turn-0001".to_owned(),
            paragraphs: vec![ParagraphInput {
                sentences: sentences
                    .iter()
                    .map(|sentence| sentence.join(" ").into_bytes())
                    .collect(),
            }],
        }],
    }
}

fn registered_input() -> ConversationInput {
    let owned_sentences = REGISTERED_TOKENS
        .iter()
        .map(|token| vec![*token])
        .collect::<Vec<_>>();
    let sentences = owned_sentences
        .iter()
        .map(Vec::as_slice)
        .collect::<Vec<_>>();
    conversation_input("issue-952/a1-registration-only", &sentences)
}

fn token_address(
    codec: &CanonicalLexicalCodec,
    artifact: &CanonicalRouteArtifact,
    token: &str,
) -> GeometricAddress {
    let encoded = codec.encode(0, 0, token.as_bytes()).unwrap();
    assert_eq!(encoded.units.len(), 1);
    artifact
        .lexical_route_address(encoded.units[0].unit_id)
        .unwrap()
        .unwrap()
}

fn history_addresses(
    codec: &CanonicalLexicalCodec,
    artifact: &CanonicalRouteArtifact,
    history: &[&str],
) -> Vec<GeometricAddress> {
    history
        .iter()
        .map(|token| token_address(codec, artifact, token))
        .collect()
}

fn selected_payload(
    artifact: &CanonicalRouteArtifact,
    trace: &PathLeaseAttentionTrace,
) -> Option<Vec<u8>> {
    trace.selected.as_ref().map(|selected| {
        artifact
            .lexical_route_value_for_address(&selected.next)
            .unwrap()
            .expect("selected address has an exact codec inverse")
            .payload_bytes
    })
}

fn cost_signature(
    artifact: &CanonicalRouteArtifact,
    trace: &PathLeaseAttentionTrace,
) -> Vec<(Vec<u8>, PathLeaseCost)> {
    trace
        .candidates
        .iter()
        .map(|candidate| {
            let payload = artifact
                .lexical_route_value_for_address(&candidate.next)
                .unwrap()
                .expect("admitted address has an exact codec inverse")
                .payload_bytes;
            (payload, candidate.cost)
        })
        .collect()
}

fn support_signature(
    artifact: &CanonicalRouteArtifact,
    trace: &PathLeaseAttentionTrace,
) -> Vec<(Vec<u8>, AttentionSourceCounts)> {
    let mut signature = trace
        .candidates
        .iter()
        .map(|candidate| {
            let payload = artifact
                .lexical_route_value_for_address(&candidate.next)
                .unwrap()
                .expect("admitted address has an exact codec inverse")
                .payload_bytes;
            (payload, candidate.source_counts)
        })
        .collect::<Vec<_>>();
    signature.sort_by(|left, right| left.0.cmp(&right.0));
    signature
}

fn assert_adjacent_spin_fallback_contract(trace: &PathLeaseAttentionTrace) {
    let support = &trace.support;
    let policy = AttentionQueryPolicy::PrimaryThenAdjacentSpinFallbackV1;
    assert_eq!(support.query_policy, policy);
    assert_eq!(support.query_policy_kappa, policy.identity_kappa());
    assert!(support.fallback_active);
    assert_eq!(support.rows_read.len(), ATTENTION_ROWS_PER_QUERY);
    assert_eq!(
        support
            .rows_read
            .iter()
            .map(|row| row.slot_index)
            .collect::<Vec<_>>(),
        (0..ATTENTION_ROWS_PER_QUERY).collect::<Vec<_>>()
    );

    let primary_rows =
        &support.rows_read[..ATTENTION_ROWS_PER_QUERY - ATTENTION_ADJACENT_SPIN_ROWS];
    assert!(primary_rows.iter().all(|row| {
        row.source != AttentionRowSource::AdjacentSpin
            && row.consulted
            && !row.fallback_active
            && row.candidate_entries_available == 0
            && row.candidate_entries_examined == 0
            && row.candidate_entries_admitted == 0
    }));

    let adjacent_rows =
        &support.rows_read[ATTENTION_ROWS_PER_QUERY - ATTENTION_ADJACENT_SPIN_ROWS..];
    assert_eq!(adjacent_rows.len(), ATTENTION_ADJACENT_SPIN_ROWS);
    assert!(adjacent_rows.iter().all(|row| {
        row.source == AttentionRowSource::AdjacentSpin
            && matches!(&row.key, AttentionRowKey::AdjacentSpin(_))
            && row.consulted
            && row.fallback_active
    }));
    assert!(support
        .rows_read
        .iter()
        .all(|row| row.hit == row.physical_row_present));

    assert_eq!(support.candidate_entries_available, 2);
    assert_eq!(support.candidate_entries_examined, 2);
    assert_eq!(support.candidate_entries_admitted, 2);
    assert_eq!(
        support
            .rows_read
            .iter()
            .map(|row| row.candidate_entries_available)
            .sum::<usize>(),
        2
    );
    assert_eq!(
        support
            .rows_read
            .iter()
            .map(|row| row.candidate_entries_examined)
            .sum::<usize>(),
        2
    );
    assert_eq!(
        support
            .rows_read
            .iter()
            .map(|row| row.candidate_entries_admitted)
            .sum::<usize>(),
        2
    );
}

fn exercise_incremental_hierarchy(
    codec: &CanonicalLexicalCodec,
    history: &[&str],
    continuation: &[Vec<u8>],
) -> usize {
    let mut decoded_sequence = history.to_vec();
    for selected_payload in continuation {
        decoded_sequence.push(std::str::from_utf8(selected_payload).unwrap());
    }
    assert!((2..=8).contains(&decoded_sequence.len()));
    let input = conversation_input("issue-969/decoded-smoke", &[&decoded_sequence]);
    let artifact = CanonicalRouteArtifact::ingest(codec, &input).unwrap();
    let expected_current = artifact
        .attention_consumer_trace()
        .unwrap()
        .ordered_levels
        .into_iter()
        .find(|level| level.level == "current")
        .expect("decoded hierarchy exposes current route")
        .identity_kappa;
    let mut cursor = artifact.incremental_cursor().unwrap();
    let mut maximum_changed = 0usize;
    while let Some(delta) = cursor.apply_next().unwrap() {
        maximum_changed = maximum_changed.max(delta.changed_nodes.len());
    }
    assert_eq!(cursor.remaining_events(), 0);
    assert_eq!(
        cursor.state().current_route.as_deref(),
        Some(expected_current.as_str())
    );
    maximum_changed
}

fn run_smoke() -> SmokeRun {
    let smoke_fixture_kappa = format!(
        "blake3:{}",
        blake3::hash(SMOKE_FIXTURE_IDENTITY_BYTES).to_hex()
    );
    assert_eq!(smoke_fixture_kappa, SMOKE_FIXTURE_KAPPA);
    let codec = CanonicalLexicalCodec::compile(&registered_input()).unwrap();
    let construction = CanonicalRouteArtifact::ingest(
        &codec,
        &conversation_input("issue-952/a1-construction-only", &CONSTRUCTION_SENTENCES),
    )
    .unwrap();
    let manifest = construction.embedded_spin_manifest().unwrap();
    let attention = GeometricAttentionArtifact::compile_from_manifest_witnesses(&manifest).unwrap();
    let table = validate_h4_binary_icosahedral_closure().unwrap();

    assert_eq!(construction.manifest_kappa(), CONSTRUCTION_ARTIFACT_KAPPA);
    assert_eq!(attention.manifest_kappa(), ATTENTION_MANIFEST_KAPPA);

    let left_history = history_addresses(&codec, &construction, &LEFT_HISTORY);
    let right_history = history_addresses(&codec, &construction, &RIGHT_HISTORY);
    assert_eq!(
        left_history
            .iter()
            .map(|address| address.atom)
            .collect::<std::collections::BTreeSet<_>>(),
        right_history.iter().map(|address| address.atom).collect()
    );

    let mut left_state = attention
        .causal_path_state_from_history(&left_history, &table)
        .unwrap();
    let mut right_state = attention
        .causal_path_state_from_history(&right_history, &table)
        .unwrap();

    let left_full = attention
        .select_path_or_abstain(&left_state, &table, PathLeaseControl::FullPath)
        .unwrap();
    let right_full = attention
        .select_path_or_abstain(&right_state, &table, PathLeaseControl::FullPath)
        .unwrap();
    let left_last = attention
        .select_path_or_abstain(&left_state, &table, PathLeaseControl::LastOnly)
        .unwrap();
    let right_last = attention
        .select_path_or_abstain(&right_state, &table, PathLeaseControl::LastOnly)
        .unwrap();
    let left_disabled = attention
        .select_path_or_abstain(&left_state, &table, PathLeaseControl::StateDisabled)
        .unwrap();
    let right_disabled = attention
        .select_path_or_abstain(&right_state, &table, PathLeaseControl::StateDisabled)
        .unwrap();

    let expected_support = vec![
        (
            b"ll".to_vec(),
            AttentionSourceCounts {
                adjacent_spin: 1,
                ..AttentionSourceCounts::default()
            },
        ),
        (
            b"rr".to_vec(),
            AttentionSourceCounts {
                adjacent_spin: 1,
                ..AttentionSourceCounts::default()
            },
        ),
    ];
    for trace in [
        &left_full,
        &right_full,
        &left_last,
        &right_last,
        &left_disabled,
        &right_disabled,
    ] {
        assert_eq!(support_signature(&construction, trace), expected_support);
        assert_adjacent_spin_fallback_contract(trace);
        assert_eq!(trace.support.unique_candidates_before_ceiling, 2);
        assert_eq!(trace.candidates.len(), 2);
        assert_eq!(trace.memory_keys_per_candidate, 4);
        assert_eq!(trace.path_geometry_evaluations, 8);
    }

    let left_first = selected_payload(&construction, &left_full).unwrap();
    let right_first = selected_payload(&construction, &right_full).unwrap();
    assert_eq!(left_first, b"rr");
    assert_eq!(right_first, b"ll");
    assert_ne!(left_first, right_first);
    assert_eq!(left_full.selected.as_ref().unwrap().best_prefix_index, 1);
    assert_eq!(right_full.selected.as_ref().unwrap().best_prefix_index, 1);

    let left_first_costs = cost_signature(&construction, &left_full);
    let right_first_costs = cost_signature(&construction, &right_full);
    assert_eq!(
        left_first_costs,
        vec![
            (
                b"rr".to_vec(),
                PathLeaseCost {
                    angular_shell: H4S3AngularShell::Degrees36,
                    lease_age: 4,
                },
            ),
            (
                b"ll".to_vec(),
                PathLeaseCost {
                    angular_shell: H4S3AngularShell::Degrees60,
                    lease_age: 2,
                },
            ),
        ]
    );
    assert_eq!(
        right_first_costs,
        vec![
            (
                b"ll".to_vec(),
                PathLeaseCost {
                    angular_shell: H4S3AngularShell::Degrees36,
                    lease_age: 4,
                },
            ),
            (
                b"rr".to_vec(),
                PathLeaseCost {
                    angular_shell: H4S3AngularShell::Degrees60,
                    lease_age: 2,
                },
            ),
        ]
    );

    let last_only_outputs = [
        selected_payload(&construction, &left_last),
        selected_payload(&construction, &right_last),
    ];
    assert_eq!(last_only_outputs, [None, None]);
    assert!(left_last.tie && right_last.tie);

    let state_disabled_outputs = [
        selected_payload(&construction, &left_disabled),
        selected_payload(&construction, &right_disabled),
    ];
    assert_eq!(
        state_disabled_outputs,
        [Some(b"rr".to_vec()), Some(b"rr".to_vec())]
    );

    let left_selected = left_full.selected.as_ref().unwrap().next.clone();
    let right_selected = right_full.selected.as_ref().unwrap().next.clone();
    attention
        .observe_path(&mut left_state, left_selected.clone(), &table)
        .unwrap();
    attention
        .observe_path(&mut right_state, right_selected.clone(), &table)
        .unwrap();
    let mut rebuilt_left = left_history.clone();
    rebuilt_left.push(left_selected.clone());
    let mut rebuilt_right = right_history.clone();
    rebuilt_right.push(right_selected.clone());
    assert_eq!(
        left_state,
        attention
            .causal_path_state_from_history(&rebuilt_left, &table)
            .unwrap()
    );
    assert_eq!(
        right_state,
        attention
            .causal_path_state_from_history(&rebuilt_right, &table)
            .unwrap()
    );

    let left_second_trace = attention
        .select_path_or_abstain(&left_state, &table, PathLeaseControl::FullPath)
        .unwrap();
    let right_second_trace = attention
        .select_path_or_abstain(&right_state, &table, PathLeaseControl::FullPath)
        .unwrap();
    for trace in [&left_second_trace, &right_second_trace] {
        assert_eq!(support_signature(&construction, trace), expected_support);
        assert_adjacent_spin_fallback_contract(trace);
        assert_eq!(trace.support.unique_candidates_before_ceiling, 2);
        assert_eq!(trace.candidates.len(), 2);
        assert_eq!(trace.memory_keys_per_candidate, 5);
        assert_eq!(trace.path_geometry_evaluations, 10);
    }

    let left_second = selected_payload(&construction, &left_second_trace).unwrap();
    let right_second = selected_payload(&construction, &right_second_trace).unwrap();
    assert_eq!(left_second, b"ll");
    assert_eq!(right_second, b"rr");
    assert_ne!(left_first, left_second);
    assert_ne!(right_first, right_second);
    assert_eq!(
        left_second_trace
            .selected
            .as_ref()
            .unwrap()
            .best_prefix_index,
        0
    );
    assert_eq!(
        right_second_trace
            .selected
            .as_ref()
            .unwrap()
            .best_prefix_index,
        1
    );

    let left_second_costs = cost_signature(&construction, &left_second_trace);
    let right_second_costs = cost_signature(&construction, &right_second_trace);
    assert_eq!(
        left_second_costs,
        vec![
            (
                b"ll".to_vec(),
                PathLeaseCost {
                    angular_shell: H4S3AngularShell::Degrees36,
                    lease_age: 6,
                },
            ),
            (
                b"rr".to_vec(),
                PathLeaseCost {
                    angular_shell: H4S3AngularShell::Orthogonal,
                    lease_age: 3,
                },
            ),
        ]
    );
    assert_eq!(
        right_second_costs,
        vec![
            (
                b"rr".to_vec(),
                PathLeaseCost {
                    angular_shell: H4S3AngularShell::Degrees36,
                    lease_age: 5,
                },
            ),
            (
                b"ll".to_vec(),
                PathLeaseCost {
                    angular_shell: H4S3AngularShell::Degrees36,
                    lease_age: 6,
                },
            ),
        ]
    );

    let left_second_selected = left_second_trace.selected.as_ref().unwrap().next.clone();
    let right_second_selected = right_second_trace.selected.as_ref().unwrap().next.clone();
    attention
        .observe_path(&mut left_state, left_second_selected.clone(), &table)
        .unwrap();
    attention
        .observe_path(&mut right_state, right_second_selected.clone(), &table)
        .unwrap();
    rebuilt_left.push(left_second_selected);
    rebuilt_right.push(right_second_selected);
    assert_eq!(
        left_state,
        attention
            .causal_path_state_from_history(&rebuilt_left, &table)
            .unwrap()
    );
    assert_eq!(
        right_state,
        attention
            .causal_path_state_from_history(&rebuilt_right, &table)
            .unwrap()
    );
    assert_eq!(
        left_state
            .prefix_states()
            .iter()
            .copied()
            .collect::<std::collections::BTreeSet<_>>()
            .len(),
        left_state.prefix_states().len()
    );
    assert_eq!(
        right_state
            .prefix_states()
            .iter()
            .copied()
            .collect::<std::collections::BTreeSet<_>>()
            .len(),
        right_state.prefix_states().len()
    );

    let left_continuation = vec![left_first, left_second];
    let right_continuation = vec![right_first, right_second];
    assert_eq!(left_continuation, [b"rr".to_vec(), b"ll".to_vec()]);
    assert_eq!(right_continuation, [b"ll".to_vec(), b"rr".to_vec()]);

    let left_hierarchy_changes =
        exercise_incremental_hierarchy(&codec, &LEFT_HISTORY, &left_continuation);
    let right_hierarchy_changes =
        exercise_incremental_hierarchy(&codec, &RIGHT_HISTORY, &right_continuation);
    assert_eq!(left_hierarchy_changes, 2);
    assert_eq!(right_hierarchy_changes, 2);

    SmokeRun {
        smoke_fixture_kappa,
        construction_artifact_kappa: construction.manifest_kappa().to_owned(),
        attention_manifest_kappa: attention.manifest_kappa().to_owned(),
        h4_root_table_kappa: table.h4_root_table_kappa,
        h4_multiplication_table_kappa: table.multiplication_table_kappa,
        left_continuation,
        right_continuation,
        left_full_costs: vec![left_first_costs, left_second_costs],
        right_full_costs: vec![right_first_costs, right_second_costs],
        last_only_outputs,
        state_disabled_outputs,
        path_geometry_evaluations_per_step: [
            left_full.path_geometry_evaluations,
            left_second_trace.path_geometry_evaluations,
        ],
        maximum_hierarchy_changes_per_event: left_hierarchy_changes,
    }
}

#[test]
fn causal_r4_path_geometry_changes_a_two_unit_decoded_continuation() {
    let first = run_smoke();
    let second = run_smoke();
    assert_eq!(first, second);
    let first_bytes = serde_json::to_vec(&first).unwrap();
    let second_bytes = serde_json::to_vec(&second).unwrap();
    assert_eq!(first_bytes, second_bytes);
    assert_eq!(
        format!("blake3:{}", blake3::hash(&first_bytes).to_hex()),
        SMOKE_RECORD_KAPPA
    );
}
