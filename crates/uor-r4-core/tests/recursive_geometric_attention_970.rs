use std::collections::{BTreeMap, BTreeSet};

use uor_r4_core::canonical_lexical_ingestion::validate_h4_binary_icosahedral_closure;
use uor_r4_core::prime_route_attention::{
    zeta_grid_kappa, ZETA_GRID_KAPPA_REFERENCE, ZETA_GRID_REVISION,
};
use uor_r4_core::recursive_geometric_attention::{
    run_a1p_candidate_relative_identifiability_probe, A1PExactR4HeatmapClassKey,
    A1PFixtureObservation, NOT_RUN_IDENTIFIABILITY_HARD_STOP,
    RETAIN_H4_STATE_ONLY_ADVANCE_MULTICHANNEL_A1Q,
};

const CONSTRUCTION_FIXTURE_KAPPA: &str =
    "blake3:fb5f27fc1107f527d616f32affa8eba1746a2f60cfdb95ddbb21a0e493299652";
const VALIDATION_FIXTURE_KAPPA: &str =
    "blake3:ecbe8b404e7542d801ff4b4e66c91a41f90158d84efa484dc4edb53aff38b602";
const INHERITED_A1R_REPORT_KAPPA: &str =
    "blake3:f0db7a5d5c81d51ebf3b4bf8a2715c4960ec16b14161e8bf7598d7b98c48c881";
const PAIRED_H4_R4_HEATMAP_CONTRACT_KAPPA: &str =
    "blake3:2daacf538c022fab9580d1e124af6c18d0b06da04604fbc962a01bda57f08a98";
const PAIRED_H4_UNIVERSE_KAPPA: &str =
    "blake3:dca725c0ec6060166bcd0023df956e1ff029661b5fa7800ccb9f20808712b796";
const A1P_REPORT_KAPPA: &str =
    "blake3:5f9239150dea8c0c27c4dfa6ad2e4d0068bc3d18afc127b315c0ec358ceddb3f";

const DOWNSTREAM_CONTROL_CONTRACT: [(&str, bool); 10] = [
    ("full-paired-h4-r4-heatmap", true),
    ("current-only", true),
    ("additive-summary-compiled-scorer", true),
    ("factor-count-only", true),
    ("deterministic-geometry-permutation", true),
    ("candidate-relabeling", true),
    ("prime-assignment-permutation", true),
    ("hierarchy-disabled", true),
    ("exact-recall-only", true),
    ("placement-intervention", false),
];

fn square_zphi([a, b]: [i64; 2]) -> [i64; 2] {
    [a * a + b * b, 2 * a * b + b * b]
}

fn expected_landmark_output(key: &A1PExactR4HeatmapClassKey) -> Option<u8> {
    match (key.sin_zphi_numerator, key.cos_zphi_numerator) {
        ([2, 0], [0, 0]) | ([-2, 0], [0, 0]) => Some(1),
        ([0, 0], [2, 0]) | ([0, 0], [-2, 0]) => Some(0),
        _ => None,
    }
}

fn independent_s4_parity(observation: &A1PFixtureObservation, role_order: &[String]) -> String {
    assert_eq!(observation.history.len(), role_order.len() + 1);
    assert_eq!(observation.history.last().map(String::as_str), Some("qq"));
    let permutation = observation.history[..role_order.len()]
        .iter()
        .map(|token| {
            role_order
                .iter()
                .position(|role| role == token)
                .expect("frozen history role")
        })
        .collect::<Vec<_>>();
    assert_eq!(
        permutation.iter().copied().collect::<BTreeSet<_>>().len(),
        role_order.len()
    );
    let inversions = permutation
        .iter()
        .enumerate()
        .map(|(left_index, left)| {
            permutation[left_index + 1..]
                .iter()
                .filter(|right| left > *right)
                .count()
        })
        .sum::<usize>();
    if inversions % 2 == 0 { "EVEN" } else { "ODD" }.to_owned()
}

#[test]
fn paired_h4_r4_heatmap_identifiability_hard_stops_before_selection() {
    let first = run_a1p_candidate_relative_identifiability_probe()
        .expect("fixed #970 paired-H4/R4 heatmap probe");
    let second = run_a1p_candidate_relative_identifiability_probe()
        .expect("deterministic second #970 execution");
    let first_bytes = first.canonical_bytes().expect("canonical first report");
    let second_bytes = second.canonical_bytes().expect("canonical second report");
    assert_eq!(first, second);
    assert_eq!(first.report_kappa, second.report_kappa);
    assert_eq!(first_bytes, second_bytes);
    assert_eq!(first.schema, 2);
    assert_eq!(
        first.domain,
        "uor-r4.a1p-candidate-relative-identifiability-probe/2"
    );

    let body = &first.body;
    assert_eq!(body.contract_status, "VALID_PUBLIC_CONTRACT");
    assert_eq!(
        body.probe_status,
        "EXERCISED_FIXED_PAIRED_H4_R4_HEATMAP_IDENTIFIABILITY_HARD_STOP"
    );
    assert_eq!(
        body.terminal_verdict,
        RETAIN_H4_STATE_ONLY_ADVANCE_MULTICHANNEL_A1Q
    );
    assert_eq!(body.scalar_functions_searched, 0);
    assert_eq!(body.readout_artifacts_compiled, 0);
    assert!(!body.validation_selection_outputs_opened);

    let contract = &body.paired_h4_r4_heatmap_contract;
    assert_eq!(contract.expected_kappa, PAIRED_H4_R4_HEATMAP_CONTRACT_KAPPA);
    assert_eq!(contract.observed_kappa, PAIRED_H4_R4_HEATMAP_CONTRACT_KAPPA);
    assert!(contract.kappa_reproduces);
    assert_eq!(contract.contract.paired_h4_operands.len(), 2);
    assert_eq!(
        contract.contract.relative_product,
        "D(H,c)=X(H,c)*Y(P_c,c)^-1"
    );
    assert_eq!(contract.contract.r4_basis, ["1", "i", "j", "k"]);
    assert_eq!(contract.contract.coordinate_scale_denominator, 2);
    assert_eq!(contract.contract.activation_denominator, 4);
    assert_eq!(contract.contract.exact_ring, "Z[phi]");
    assert_eq!(contract.contract.zeta_grid_revision, ZETA_GRID_REVISION);
    assert_eq!(contract.contract.zeta_grid_kappa, ZETA_GRID_KAPPA_REFERENCE);
    assert_eq!(
        zeta_grid_kappa().expect("live fixed-zeta grid kappa"),
        ZETA_GRID_KAPPA_REFERENCE
    );
    assert_eq!(contract.contract.binary_landmarks.len(), 4);
    assert_eq!(
        contract.contract.radial_coupling_status,
        "STRUCTURAL_BINDING_ONLY_NO_ZETA_NLET_TO_PHI_EXPONENT_RULE"
    );
    assert_eq!(
        contract.contract.typed_geometry_equivalence,
        ["EUCLIDEAN_ROOT_2", "COMPLEX_2I", "RIEMANNIAN_0_2"]
    );
    assert!(contract.contract.intermediate_rule.contains("no threshold"));
    assert!(contract
        .contract
        .semantic_exclusions
        .contains(&"validation labels".to_owned()));
    assert!(contract
        .contract
        .semantic_exclusions
        .contains(&"raw zeta channel identity as scorer input".to_owned()));

    let fixtures = &body.fixture_contract;
    assert_eq!(
        fixtures.construction.expected_kappa,
        CONSTRUCTION_FIXTURE_KAPPA
    );
    assert_eq!(
        fixtures.construction.observed_kappa,
        CONSTRUCTION_FIXTURE_KAPPA
    );
    assert!(fixtures.construction.kappa_reproduces);
    assert_eq!(fixtures.validation.expected_kappa, VALIDATION_FIXTURE_KAPPA);
    assert_eq!(fixtures.validation.observed_kappa, VALIDATION_FIXTURE_KAPPA);
    assert!(fixtures.validation.kappa_reproduces);
    assert!(fixtures.parity_derived_from_history);
    assert!(fixtures.new_split_preparation_target_free);
    assert!(fixtures.labels_attached_after_preparation);
    assert_eq!(fixtures.construction_parity_audit.len(), 6);
    assert_eq!(fixtures.validation_parity_audit.len(), 6);
    for (observations, audits) in [
        (
            &fixtures.construction.observations,
            &fixtures.construction_parity_audit,
        ),
        (
            &fixtures.validation.observations,
            &fixtures.validation_parity_audit,
        ),
    ] {
        for observation in observations {
            let derived = independent_s4_parity(observation, &fixtures.construction.role_order);
            let audit = audits
                .iter()
                .find(|audit| audit.observation_id == observation.id)
                .expect("parity audit row");
            assert_eq!(derived, observation.permutation_parity);
            assert_eq!(audit.derived_parity, derived);
            assert_eq!(audit.declared_parity, derived);
            assert!(audit.exact_match);
        }
    }
    assert!(fixtures.histories_disjoint_across_all_splits);
    assert!(fixtures.trivial_map_rejected);
    assert!(fixtures.construction_rule_confirmed);
    assert!(fixtures.regression_labels_falsification_only);
    assert!(fixtures.no_a1q_fixture_consumed);

    let universe = &body.paired_h4_structural_universe;
    assert_eq!(universe.operand_count, 120);
    assert_eq!(universe.expected_ordered_pair_count, 14_400);
    assert_eq!(universe.enumerated_ordered_pair_count, 14_400);
    assert_eq!(universe.relative_image_count, 120);
    assert_eq!(universe.relative_image_rows.len(), 120);
    assert!(universe.exact_relative_image_complete);
    assert_eq!(universe.pair_universe_census_schema, 1);
    assert_eq!(universe.pair_universe_row_width_bytes, 6);
    assert!(universe
        .pair_universe_row_encoding
        .contains("x_u16be||y_u16be||D_u16be"));
    assert_eq!(universe.pair_universe_kappa, PAIRED_H4_UNIVERSE_KAPPA);
    assert_eq!(universe.pair_multiplicity_per_relative_image_row, 120);
    assert!(universe.uniform_pair_multiplicity_exact);
    assert!(universe.heatmap_class_count > 4);
    assert!(universe.heatmap_class_count <= 120);
    assert_eq!(universe.heatmap_class_count, 45);
    assert!(universe.typed_null_pair_count > 0);
    assert_eq!(universe.typed_null_pair_count % 120, 0);
    assert_eq!(universe.typed_null_pair_count, 480);
    assert!(universe.target_free);
    assert!(universe.integer_only_no_rounding);
    let table = validate_h4_binary_icosahedral_closure().expect("exact H4 closure");
    let mut relative_counts = BTreeMap::<u16, usize>::new();
    for x in 0..u16::try_from(table.root_count).expect("root count fits u16") {
        for y in 0..u16::try_from(table.root_count).expect("root count fits u16") {
            let inverse_y = table.inverse_indices[usize::from(y)];
            let relative = table
                .product_index(x, inverse_y)
                .expect("closed pair product");
            *relative_counts.entry(relative).or_default() += 1;
        }
    }
    assert_eq!(relative_counts.len(), 120);
    assert!(relative_counts.values().all(|count| *count == 120));

    let mut observed_landmark_outputs = BTreeSet::new();
    for entry in &universe.relative_image_rows {
        let relative_offset = entry.relative_state.opaque_table_offset;
        assert!(usize::from(relative_offset) < table.root_count);
        assert_eq!(
            entry.ordered_pair_multiplicity,
            relative_counts[&relative_offset]
        );
        assert_eq!(
            entry.relative_state.root_coordinate.scaled_zphi_quaternion[0],
            entry.heatmap_key.sin_zphi_numerator
        );
        assert_eq!(
            entry.relative_state.root_coordinate.scaled_zphi_quaternion[1],
            entry.heatmap_key.cos_zphi_numerator
        );
        assert_eq!(
            entry.heatmap_key.activation_zphi_numerator,
            square_zphi(entry.heatmap_key.sin_zphi_numerator)
        );
        assert_eq!(
            entry.binary_landmark_output,
            expected_landmark_output(&entry.heatmap_key)
        );
        if let Some(output) = entry.binary_landmark_output {
            observed_landmark_outputs.insert(output);
        }
    }
    assert_eq!(observed_landmark_outputs, BTreeSet::from([0, 1]));

    let inherited = &body.inherited_regression;
    assert_eq!(inherited.report_kappa, INHERITED_A1R_REPORT_KAPPA);
    assert_eq!(
        inherited.labels_status,
        "FALSIFICATION_ONLY_NON_PROMOTIONAL"
    );
    assert_eq!(inherited.query_denominator, 6);
    assert_eq!(inherited.candidate_decision_denominator, 12);
    assert_eq!(inherited.shortest_cayley_strict_selections, 0);
    assert_eq!(inherited.shortest_cayley_ties, 6);
    assert_eq!(inherited.shortest_cayley_abstentions, 6);
    assert_eq!(inherited.queries_with_distinct_candidate_relative_states, 6);
    assert_eq!(inherited.paired_same_candidate_comparisons, 6);
    assert_eq!(
        inherited.paired_same_candidate_relative_state_differences,
        5
    );
    assert!(inherited.aggregate_reproduces);

    assert_eq!(body.support_and_work.len(), 3);
    assert_eq!(
        body.support_and_work
            .iter()
            .map(|split| split.split.as_str())
            .collect::<BTreeSet<_>>(),
        BTreeSet::from(["A1R_REGRESSION", "CONSTRUCTION", "SEALED_VALIDATION"])
    );
    for split in &body.support_and_work {
        assert_eq!(split.histories, 6);
        assert_eq!(split.candidate_decisions, 12);
        assert_eq!(split.natural_candidates, ["ll", "rr"]);
        assert!(split.natural_candidate_union_exact);
        assert!(split.support_denominator_exact);
        assert_eq!(split.rows_per_query, 7);
        assert_eq!(split.row_reads, 42);
        assert_eq!(split.candidate_entries_examined, 12);
        assert_eq!(split.candidate_entry_ceiling_per_query, 56);
        assert_eq!(split.candidate_ceiling, 8);
        assert_eq!(split.maximum_admitted_candidates, 2);
        assert!(split.exact_direct_rows_miss);
        assert!(split.divisor_rows_miss);
        assert!(split.adjacent_spin_only_support);
        assert_eq!(split.exact_payload_inversions, 12);
        assert!(!split.target_injected);
        assert!(!split.future_events_visible);
        assert!(!split.admission_truncation_observed);
        assert!(split.contract_exact);
    }

    let heatmap = &body.paired_h4_r4_heatmap;
    assert_eq!(
        heatmap
            .classes
            .iter()
            .map(|class| class.members.len())
            .sum::<usize>(),
        36
    );
    assert_eq!(heatmap.coordinate_scale_denominator, 2);
    assert_eq!(heatmap.activation_denominator, 4);
    assert!(heatmap.integer_only_no_rounding);
    assert_eq!(
        heatmap.metrics.construction_decisions_covered.denominator,
        12
    );
    assert_eq!(
        heatmap
            .metrics
            .validation_decisions_covered_by_construction
            .denominator,
        12
    );
    assert_eq!(
        heatmap
            .metrics
            .validation_no_class_splitting_oracle_ceiling
            .denominator,
        12
    );
    assert_eq!(
        heatmap
            .metrics
            .validation_construction_transfer_selection_ceiling
            .denominator,
        6
    );
    assert!(heatmap.metrics.exact_class_aliasing_observed);
    assert_eq!(heatmap.metrics.exact_class_count, 14);
    assert_eq!(heatmap.metrics.inherited_class_count, 9);
    assert_eq!(heatmap.metrics.construction_class_count, 10);
    assert_eq!(heatmap.metrics.validation_class_count, 7);
    assert_eq!(heatmap.metrics.construction_impure_class_count, 0);
    assert!(heatmap.metrics.construction_classes_pure);
    assert_eq!(
        (
            heatmap
                .metrics
                .validation_decisions_covered_by_construction
                .numerator,
            heatmap
                .metrics
                .validation_decisions_covered_by_construction
                .denominator,
        ),
        (10, 12)
    );
    assert_eq!(
        (
            heatmap
                .metrics
                .validation_no_class_splitting_oracle_ceiling
                .numerator,
            heatmap
                .metrics
                .validation_no_class_splitting_oracle_ceiling
                .denominator,
        ),
        (10, 12)
    );
    assert_eq!(
        heatmap
            .metrics
            .validation_construction_transfer_selection_ceiling
            .numerator,
        0
    );
    assert_eq!(
        heatmap.metrics.incompatible_class_count_across_all_splits,
        8
    );
    assert!(!heatmap.metrics.transferable_construction_rule_exists);
    for class in &heatmap.classes {
        assert_eq!(
            class.heatmap_key.activation_zphi_numerator,
            square_zphi(class.heatmap_key.sin_zphi_numerator)
        );
        assert_eq!(
            class.binary_landmark_output,
            expected_landmark_output(&class.heatmap_key)
        );
        assert_eq!(
            class.typed_null_abstain,
            class.heatmap_key.chart_status == "TYPED_NULL_ABSTAIN"
        );
        for member in &class.members {
            let x = member.operands.x_interaction.opaque_table_offset;
            let y = member
                .operands
                .y_predecessor_interaction
                .opaque_table_offset;
            let d = member.operands.relative_projection.opaque_table_offset;
            let inverse_y = table.inverse_indices[usize::from(y)];
            assert_eq!(table.product_index(x, inverse_y), Some(d));
            assert_eq!(
                member
                    .operands
                    .relative_projection
                    .root_coordinate
                    .scaled_zphi_quaternion[0],
                class.heatmap_key.sin_zphi_numerator
            );
            assert_eq!(
                member
                    .operands
                    .relative_projection
                    .root_coordinate
                    .scaled_zphi_quaternion[1],
                class.heatmap_key.cos_zphi_numerator
            );
        }
    }
    let decisive_alias = heatmap
        .classes
        .iter()
        .find(|class| {
            class.members.iter().any(|member| {
                member.decision.observation_id == "A1R-R05"
                    && member.decision.candidate == "ll"
                    && member.decision.outcome == "SELECT"
            }) && class.members.iter().any(|member| {
                member.decision.observation_id == "A1R-R06"
                    && member.decision.candidate == "ll"
                    && member.decision.outcome == "REJECT"
            })
        })
        .expect("same exact heatmap class retains incompatible old-six outcomes");
    assert!(decisive_alias.incompatible_outcomes);
    for observation_id in ["A1R-R05", "A1R-R06"] {
        assert_eq!(
            decisive_alias
                .members
                .iter()
                .find(|member| {
                    member.decision.observation_id == observation_id
                        && member.decision.candidate == "ll"
                })
                .expect("decisive alias member")
                .operands
                .relative_projection
                .opaque_table_offset,
            70
        );
    }
    assert!(body.hard_stop_reasons.contains(&format!(
        "INCOMPATIBLE_EXACT_R4_HEATMAP_CLASS:{}",
        decisive_alias.class_id
    )));
    assert!(body
        .hard_stop_reasons
        .iter()
        .any(|reason| reason == "NO_COMPLETE_EXACT_R4_HEATMAP_CONSTRUCTION_TRANSFER"));
    assert_eq!(body.hard_stop_reasons.len(), 9);

    let old_d = &body.full_h4;
    assert!(old_d
        .class_definition
        .starts_with("SUPERSEDED_SCOPE_DIAGNOSTIC_ONLY_NOT_A1P_SCORER_KEY:"));
    assert_eq!(
        old_d
            .classes
            .iter()
            .map(|class| class.members.len())
            .sum::<usize>(),
        36
    );

    let additive = &body.additive;
    assert_eq!(
        additive
            .classes
            .iter()
            .map(|class| class.members.len())
            .sum::<usize>(),
        36
    );
    assert_eq!(additive.metrics.exact_class_count, 2);
    assert_eq!(additive.metrics.construction_impure_class_count, 2);
    assert!(!additive.metrics.construction_classes_pure);
    assert_eq!(
        additive
            .metrics
            .validation_construction_transfer_selection_ceiling
            .numerator,
        0
    );
    assert!(!additive.metrics.transferable_construction_rule_exists);
    assert!(additive
        .excluded_fields
        .contains(&"token-spelling".to_owned()));
    assert!(additive.excluded_fields.contains(&"prime".to_owned()));
    assert!(additive.excluded_fields.contains(&"kappas".to_owned()));
    assert!(additive.excluded_fields.contains(&"provenance".to_owned()));

    assert_eq!(
        body.downstream_controls
            .iter()
            .map(|control| (control.control.as_str(), control.required))
            .collect::<Vec<_>>(),
        DOWNSTREAM_CONTROL_CONTRACT
    );
    assert!(body.downstream_controls.iter().all(|control| {
        control.status == NOT_RUN_IDENTIFIABILITY_HARD_STOP
            && control.selections == 0
            && control.ties == 0
            && control.abstentions == 0
            && control.exact_hits == 0
            && control.support_equal.is_none()
            && control.work.validation_queries == 0
            && control.work.candidate_decisions == 0
            && control.work.row_reads == 0
            && control.work.candidate_entry_ceiling_per_query == 56
            && control.work.candidate_ceiling == 8
            && control.work.maximum_admitted_candidates == 0
    }));

    assert!(body.claim_boundary.identifiability_falsifier_only);
    assert!(body.claim_boundary.h4_state_retained_as_structural_only);
    assert!(
        body.claim_boundary
            .paired_h4_r4_heatmap_retained_as_structural_only
    );
    assert!(
        !body
            .claim_boundary
            .superseded_single_h4_diagnostic_is_terminal_evidence
    );
    assert!(!body.claim_boundary.attention_established);
    assert!(!body.claim_boundary.inference_established);
    assert!(!body.claim_boundary.generation_established);
    assert!(!body.claim_boundary.correctness_established);
    assert!(!body.claim_boundary.reasoning_established);
    assert!(!body.claim_boundary.semantic_value_established);
    assert!(!body.claim_boundary.validation_selection_performed);
    assert!(!body.claim_boundary.digest_or_kappa_used_as_geometry);
    assert!(!body.claim_boundary.opaque_table_offset_used_as_scalar);
    assert!(!body.claim_boundary.candidate_support_or_admission_modified);

    eprintln!("A1P_REPORT_KAPPA={}", first.report_kappa);
    eprintln!("PAIRED_HEATMAP_METRICS={:?}", heatmap.metrics);
    eprintln!(
        "STRUCTURAL_UNIVERSE_KAPPA={} heatmap_classes:{} typed_null_pairs:{}",
        universe.pair_universe_kappa, universe.heatmap_class_count, universe.typed_null_pair_count
    );
    eprintln!("ADDITIVE_METRICS={:?}", additive.metrics);
    for class in heatmap
        .classes
        .iter()
        .filter(|class| class.incompatible_outcomes)
    {
        eprintln!(
            "HEATMAP_ALIAS={} key={:?} members={:?}",
            class.class_id,
            class.heatmap_key,
            class
                .members
                .iter()
                .map(|member| (
                    member.decision.split.as_str(),
                    member.decision.observation_id.as_str(),
                    member.decision.candidate.as_str(),
                    member.decision.outcome.as_str(),
                    member.operands.relative_projection.opaque_table_offset,
                ))
                .collect::<Vec<_>>()
        );
    }

    assert_eq!(first.report_kappa, A1P_REPORT_KAPPA);
}
