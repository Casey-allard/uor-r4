use uor_r4_core::canonical_lexical_ingestion::validate_h4_binary_icosahedral_closure;
use uor_r4_core::recursive_geometric_attention::{
    run_a1_0_ordered_state_probe, REDESIGN_ORDERED_ROUTE_SUMMARY,
};
use uor_r4_core::spiralcore_operator::{
    CL06_FINITE_COMPOSITION_KAPPA_REFERENCE, CL06_FINITE_GROUP_ORDER,
};

const FROZEN_S0_ARTIFACT_KAPPA: &str =
    "blake3:3f2043e15a32f6ef799c0073d0c714e3140449591b7d8a18069e39c5182662bd";

#[test]
fn exact_scaled_h4_table_closes_as_the_binary_icosahedral_group() {
    let table = validate_h4_binary_icosahedral_closure().expect("exact H4 closure");

    assert_eq!(table.root_count, 120);
    assert_eq!(table.product_count, 120 * 120);
    assert_eq!(table.inverse_indices.len(), 120);
    assert_eq!(table.multiplication_indices.len(), 120 * 120);
    assert!(table.unique_closure_exact);
    assert!(table.identity_exact);
    assert!(table.inverses_exact);
    assert!(table.associativity_exact);
    assert!(table.integer_only_no_rounding);
    assert_eq!(
        table.reproduce_multiplication_table_kappa().unwrap(),
        table.multiplication_table_kappa
    );

    for state in 0..120u16 {
        assert_eq!(
            table.product_index(table.identity_index, state),
            Some(state)
        );
        assert_eq!(
            table.product_index(state, table.identity_index),
            Some(state)
        );
        let inverse = table.inverse_indices[usize::from(state)];
        assert_eq!(
            table.product_index(state, inverse),
            Some(table.identity_index)
        );
        assert_eq!(
            table.product_index(inverse, state),
            Some(table.identity_index)
        );
    }
    assert_eq!(table.product_index(120, 0), None);
    assert_eq!(table.product_index(0, 120), None);
}

#[test]
fn frozen_a1_0_gate_reaches_values_then_stops_before_the_scorer() {
    let first = run_a1_0_ordered_state_probe().expect("fixed A1.0 probe");
    let second = run_a1_0_ordered_state_probe().expect("deterministic A1.0 re-run");
    assert_eq!(first, second);
    assert_eq!(
        first.canonical_bytes().unwrap(),
        second.canonical_bytes().unwrap()
    );

    let body = &first.body;
    assert_eq!(body.probe_status, "EXERCISED_FIXED_A1_0_GATE");
    assert_eq!(body.terminal_verdict, REDESIGN_ORDERED_ROUTE_SUMMARY);
    assert_eq!(
        body.provenance.frozen_s0_artifact_kappa,
        FROZEN_S0_ARTIFACT_KAPPA
    );
    assert!(body.provenance.fixed_partition_kappa.starts_with("blake3:"));
    assert_eq!(
        body.fixed_partition.registered_vocabulary,
        ["aa", "bb", "cc", "dd", "gg", "ll", "qq", "rr", "uu", "vv"]
    );
    assert_eq!(body.fixed_partition.construction_sentences.len(), 7);
    assert_eq!(body.fixed_partition.evaluation_contrasts.len(), 3);
    assert!(body.fixed_partition.frozen_before_codec_compile);
    assert!(body.fixed_partition.frozen_before_candidate_compile);
    assert!(body.fixed_partition.frozen_before_summary_compile);
    assert!(!body.fixed_partition.targets_selected_from_observed_rows);
    assert!(
        !body
            .fixed_partition
            .evaluation_histories_enter_construction_manifest
    );

    assert_eq!(body.required_contrasts, 3);
    assert_eq!(body.colliding_contrasts, 3);
    assert!(body.all_required_contrasts_collide);
    assert!(body.construction_artifact.construction_only);
    assert!(body.construction_artifact.one_worker_child_manifest);
    assert_eq!(body.construction_artifact.retained_candidate_ceiling, 8);
    assert_eq!(body.construction_artifact.maximum_candidates_per_row, 8);

    assert_eq!(body.exact_h4_closure.root_count, 120);
    assert_eq!(body.exact_h4_closure.product_count, 120 * 120);
    assert_eq!(body.exact_h4_closure.associativity_checks, 120 * 120 * 120);
    assert_eq!(body.exact_h4_closure.inverse_count, 120);
    assert!(body.exact_h4_closure.multiplication_table_kappa_reproduces);
    assert!(body.exact_h4_closure.unique_closure_exact);
    assert!(body.exact_h4_closure.identity_exact);
    assert!(body.exact_h4_closure.inverses_exact);
    assert!(body.exact_h4_closure.associativity_exact);
    assert!(body.exact_h4_closure.integer_only_no_rounding);

    assert_eq!(
        body.spiralcore_control.unique_states,
        CL06_FINITE_GROUP_ORDER
    );
    assert_eq!(body.spiralcore_control.composition_entries, 64 * 64);
    assert_eq!(body.spiralcore_control.associativity_checks, 64 * 64 * 64);
    assert_eq!(body.spiralcore_control.two_sided_inverses, 64);
    assert!(body.spiralcore_control.noncommuting_ordered_pairs > 0);
    assert!(body.spiralcore_control.composition_table_kappa_reproduces);
    assert_eq!(
        body.spiralcore_control.composition_table_kappa,
        CL06_FINITE_COMPOSITION_KAPPA_REFERENCE
    );

    for contrast in &body.contrasts {
        assert!(contrast.ordered_state_collides);
        assert!(contrast.natural_candidate_union_equal);
        assert!(contrast.candidate_support_counts_equal);
        assert!(contrast.both_competing_targets_in_each_union);
        assert!(contrast.exact_direct_rows_miss_both_sides);
        assert_eq!(contrast.collision_census.included_field_count, 46);
        assert_eq!(contrast.collision_census.per_level_equal.len(), 7);
        assert!(contrast.collision_census.all_required_levels_present);
        assert!(contrast.collision_census.all_non_digest_fields_collide);
        assert!(contrast.collision_census.digest_identities_differ);
        assert!(!contrast.collision_census.digest_identity_used_for_verdict);

        for path in [
            &contrast.left_candidate_path,
            &contrast.right_candidate_path,
        ] {
            assert_eq!(path.rows.len(), 7);
            assert!(path.exact_direct_rows_miss);
            assert!(!path.admission_truncated_union);
            assert!(path.full_pre_admission_union_observed);
            assert_eq!(path.unique_candidates_before_admission, 2);
            assert_eq!(path.unique_candidates_after_admission, 2);
            assert_eq!(path.retained_candidate_ceiling, 8);
            assert_eq!(path.intended_target_pre_admission_reachable, Some(true));
            assert!(path.intended_target_post_admission_reachable);
            assert_eq!(path.intended_target_truncated_before_geometry, Some(false));
            assert!(!path.target_injected);
            assert!(!path.future_events_visible);
            assert_eq!(path.candidates.len(), 2);
            assert!(path
                .candidates
                .iter()
                .any(|candidate| { candidate.address_value.payload_bytes == b"ll" }));
            assert!(path
                .candidates
                .iter()
                .any(|candidate| { candidate.address_value.payload_bytes == b"rr" }));
            for candidate in &path.candidates {
                assert_eq!(
                    u32::from(candidate.address_value.registry_address_index),
                    candidate.address_value.lexical_unit_id
                );
                assert!(
                    usize::from(candidate.address_value.child_manifest_address_index)
                        < body.construction_artifact.child_manifest_addresses
                );
                assert_eq!(candidate.source_counts.last_one, 0);
                assert_eq!(candidate.source_counts.last_two, 0);
                assert_eq!(candidate.source_counts.ordered_sentence, 0);
                assert_eq!(candidate.source_counts.divisor, 0);
                assert!(candidate.source_counts.adjacent_spin > 0);
                assert_eq!(candidate.contributing_sources, ["adjacent-spin"]);
            }
            assert!(path
                .incremental_next_state
                .as_ref()
                .is_some_and(|state| state.exact_reproduction));
        }
    }

    assert!(!body.serving_boundary.source_model_weights_opened);
    assert_eq!(body.serving_boundary.teacher_forwards, 0);
    assert_eq!(body.serving_boundary.transformer_calls, 0);
    assert_eq!(body.serving_boundary.moe_calls, 0);
    assert_eq!(body.serving_boundary.learned_router_calls, 0);
    assert_eq!(body.serving_boundary.dense_intelligence_matrix_calls, 0);
    assert_eq!(body.serving_boundary.ollama_calls, 0);
    assert_eq!(body.serving_boundary.hosted_provider_calls, 0);
    assert!(!body.scorer_boundary.attention_scorer_implemented);
    assert!(!body.scorer_boundary.geometry_coefficients_tuned);
    assert!(!body.scorer_boundary.scorer_controls_exercised);
    assert_eq!(
        body.scorer_boundary.scorer_status,
        "NOT_IMPLEMENTED_PREDECLARED_ORDERED_STATE_STOP"
    );
    assert!(!body.scorer_boundary.digest_distance_used_as_geometry);
}
