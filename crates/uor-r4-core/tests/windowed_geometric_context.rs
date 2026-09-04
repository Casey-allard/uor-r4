use uor_r4_core::canonical_lexical_ingestion::{
    canonical_global_epoch, validate_h4_binary_icosahedral_closure, CanonicalLexicalCodec,
    CanonicalRouteArtifact, ConversationInput, ParagraphInput, TurnInput,
};
use uor_r4_core::local_geometric_generation::{LocalGenerationControl, LocalGeometricGenerator};
use uor_r4_core::prime_route_attention::GeometricAddress;
use uor_r4_core::prime_route_geometric_attention::{
    GeometricAttentionArtifact, PathAttentionContextV1, PathLeaseControl,
    WINDOWED_PATH_ATTENTION_MAX_UNITS,
};

fn fixture() -> (
    CanonicalLexicalCodec,
    CanonicalRouteArtifact,
    GeometricAttentionArtifact,
) {
    let global = vec![b"gg".to_vec()];
    let input = ConversationInput {
        identity_scope: "windowed-context-v1".to_owned(),
        global_epoch: canonical_global_epoch(&global).unwrap(),
        global_snapshot_units: global,
        turns: vec![TurnInput {
            turn_id: "turn-1".to_owned(),
            paragraphs: vec![ParagraphInput {
                sentences: ["uu ll", "vv rr", "aa", "bb", "cc", "dd", "qq"]
                    .iter()
                    .map(|text| text.as_bytes().to_vec())
                    .collect(),
            }],
        }],
    };
    let codec = CanonicalLexicalCodec::compile(&input).unwrap();
    let artifact = CanonicalRouteArtifact::ingest(&codec, &input).unwrap();
    let attention = GeometricAttentionArtifact::compile_from_manifest_witnesses(
        &artifact.embedded_spin_manifest().unwrap(),
    )
    .unwrap();
    (codec, artifact, attention)
}

fn address(
    codec: &CanonicalLexicalCodec,
    artifact: &CanonicalRouteArtifact,
    word: &str,
) -> GeometricAddress {
    let id = codec.encode(0, 0, word.as_bytes()).unwrap().units[0].unit_id;
    artifact.lexical_route_address(id).unwrap().unwrap()
}

#[test]
fn short_context_preserves_historical_selection_and_rejects_invalid_writes() {
    let (codec, artifact, attention) = fixture();
    let table = validate_h4_binary_icosahedral_closure().unwrap();
    assert_eq!(PathAttentionContextV1::default().window_units(), 8);
    assert!(PathAttentionContextV1::new(0).is_err());
    assert!(PathAttentionContextV1::new(WINDOWED_PATH_ATTENTION_MAX_UNITS + 1).is_err());
    for words in [["aa", "bb", "dd", "qq"], ["bb", "aa", "dd", "qq"]] {
        let history = words
            .iter()
            .map(|word| address(&codec, &artifact, word))
            .collect::<Vec<_>>();
        let legacy = attention
            .causal_path_state_from_history(&history, &table)
            .unwrap();
        let mut windowed = attention
            .windowed_path_state_from_history(&history, &table, PathAttentionContextV1::default())
            .unwrap();
        for control in [
            PathLeaseControl::FullPath,
            PathLeaseControl::LastOnly,
            PathLeaseControl::StateDisabled,
        ] {
            let old = attention
                .select_path_or_abstain(&legacy, &table, control)
                .unwrap();
            let new = attention
                .select_windowed_path_or_abstain(&windowed, control)
                .unwrap();
            assert_eq!(old.support, new.support);
            assert_eq!(old.path_geometry_evaluations, new.path_geometry_evaluations);
            assert_eq!(
                old.selected.as_ref().map(|candidate| &candidate.next),
                new.selected.as_ref().map(|candidate| &candidate.next)
            );
            assert_eq!((old.tie, old.abstained), (new.tie, new.abstained));
            for (left, right) in old.candidates.iter().zip(&new.candidates) {
                assert_eq!(left.next, right.next);
                assert_eq!(left.cost.angular_shell, right.cost.angular_shell);
                assert_eq!(u16::from(left.cost.lease_age), right.cost.lease_age);
                assert_eq!(u32::from(left.best_prefix_index), right.best_prefix_index);
            }
        }
        let before = windowed.clone();
        let mut wrong = history[0].clone();
        wrong.payload_cid = format!("blake3:{}", "0".repeat(64));
        assert!(attention
            .observe_windowed_path(&mut windowed, wrong)
            .is_err());
        assert_eq!(
            windowed, before,
            "invalid observation must not evict or advance"
        );
    }
}

#[test]
fn sliding_windows_bound_memory_and_keep_the_exact_cumulative_frame() {
    let (codec, artifact, attention) = fixture();
    let table = validate_h4_binary_icosahedral_closure().unwrap();
    let words = ["aa", "bb", "dd", "qq"];
    let leaves = words
        .iter()
        .map(|word| address(&codec, &artifact, word))
        .collect::<Vec<_>>();
    let history = (0..4200)
        .map(|index| leaves[index % leaves.len()].clone())
        .collect::<Vec<_>>();
    let expected = attention
        .causal_ordered_state_from_history(&history, &table)
        .unwrap();
    let previous = attention
        .causal_ordered_state_from_history(&history[..history.len() - 1], &table)
        .unwrap();
    for width in [1, 8, 32, 128, 512, 4096] {
        let context = PathAttentionContextV1::new(width).unwrap();
        let mut state = attention
            .windowed_path_state_from_history(&history, &table, context)
            .unwrap();
        assert_eq!(state.fold_state(), expected.fold_state());
        assert_eq!(state.observed_routes(), 4200);
        assert_eq!(state.first_retained_prefix_index(), 4200 - width as u32);
        assert_eq!(
            state.retained_prefix_states().last(),
            Some(&previous.fold_state())
        );
        let first = attention
            .causal_ordered_state_from_history(&history[..4200 - width], &table)
            .unwrap();
        assert_eq!(
            state.retained_prefix_states().next(),
            Some(&first.fold_state())
        );
        let memory = state.memory();
        assert_eq!(memory.retained_prefix_keys, width);
        assert_eq!(
            memory.prefix_payload_bytes,
            width * memory.current_fold_bytes
        );
        let trace = attention
            .select_windowed_path_or_abstain(&state, PathLeaseControl::FullPath)
            .unwrap();
        assert!(!trace.candidates.is_empty());
        assert_eq!(
            trace.path_geometry_evaluations,
            trace.candidates.len() * width
        );
        assert_eq!(
            trace.group_table_lookups,
            2 * trace.path_geometry_evaluations + trace.candidates.len()
        );
        assert!(trace
            .candidates
            .iter()
            .all(|candidate| candidate.cost.lease_age <= (width + 1) as u16));
        let allocated = memory.allocated_prefix_bytes;
        attention
            .observe_windowed_path(&mut state, leaves[0].clone())
            .unwrap();
        assert_eq!(state.memory().allocated_prefix_bytes, allocated);
        assert_eq!(state.memory().retained_prefix_keys, width);
        assert_eq!(state.first_retained_prefix_index(), 4201 - width as u32);
    }
}

#[test]
fn decoded_prompt_and_output_limits_are_independent_of_working_context() {
    let (_, artifact, _) = fixture();
    let generator =
        LocalGeometricGenerator::from_canonical_bytes(&artifact.canonical_bytes().unwrap())
            .unwrap();
    let control = LocalGenerationControl::FullPath;
    let short = b"aa bb dd qq";
    let old = generator.generate(short, control, 2).unwrap();
    let new = generator
        .generate_with_context(short, control, 2, PathAttentionContextV1::default())
        .unwrap();
    assert_eq!(old.continuation_bytes, new.continuation_bytes);
    assert_eq!(old.stop_reason, new.stop_reason);
    for length in [32, 128, 512] {
        let prompt = (0..length)
            .map(|index| ["aa", "bb", "dd", "qq"][index % 4])
            .collect::<Vec<_>>()
            .join(" ");
        assert!(generator.generate(prompt.as_bytes(), control, 16).is_err());
        for width in [8, 32, 128, 512] {
            let report = generator
                .generate_with_context(
                    prompt.as_bytes(),
                    control,
                    16,
                    PathAttentionContextV1::new(width).unwrap(),
                )
                .unwrap();
            assert_eq!(report.prompt_units, length);
            assert_eq!(report.continuation_cap, 16);
            assert!(!report.steps.is_empty());
            assert_eq!(report.steps[0].observed_routes_before, length as u32);
            assert_eq!(
                report.steps[0].memory.retained_prefix_keys,
                length.min(width)
            );
            for step in &report.steps {
                assert!(step.memory.retained_prefix_keys <= width);
                assert_eq!(
                    step.path_geometry_evaluations,
                    step.candidate_count * step.memory.retained_prefix_keys
                );
                assert_eq!(
                    step.observed_routes_after - step.observed_routes_before,
                    u32::from(step.selected_lexical_unit_id.is_some())
                );
            }
            assert_eq!(
                report.canonical_bytes().unwrap(),
                generator
                    .generate_with_context(
                        prompt.as_bytes(),
                        control,
                        16,
                        PathAttentionContextV1::new(width).unwrap()
                    )
                    .unwrap()
                    .canonical_bytes()
                    .unwrap()
            );
        }
    }
}
