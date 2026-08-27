use std::collections::BTreeSet;
use std::sync::OnceLock;

use uor_r4_core::canonical_lexical_ingestion::{
    H4_BINARY_ICOSAHEDRAL_MULTIPLICATION_TABLE_KAPPA_REFERENCE,
    H4_BINARY_ICOSAHEDRAL_ROOT_TABLE_KAPPA_REFERENCE,
};
use uor_r4_core::recursive_geometric_attention::{
    run_a1r_associative_ordered_summary_probe, A1RAssociativeOrderedSummaryProbeReport,
    RETAIN_STATE_ONLY,
};

const FROZEN_A1R_REPORT_KAPPA: &str =
    "blake3:f0db7a5d5c81d51ebf3b4bf8a2715c4960ec16b14161e8bf7598d7b98c48c881";

fn report() -> &'static A1RAssociativeOrderedSummaryProbeReport {
    static REPORT: OnceLock<A1RAssociativeOrderedSummaryProbeReport> = OnceLock::new();
    REPORT
        .get_or_init(|| run_a1r_associative_ordered_summary_probe().expect("fixed #967 A1R probe"))
}

#[test]
fn exact_ordered_h4_fold_laws_and_collision_census_are_report_bound() {
    let report = report();
    assert_eq!(report.report_kappa, FROZEN_A1R_REPORT_KAPPA);
    assert!(!report.canonical_bytes().unwrap().is_empty());

    let body = &report.body;
    assert_eq!(
        body.fold_contract.h4_root_table_kappa,
        H4_BINARY_ICOSAHEDRAL_ROOT_TABLE_KAPPA_REFERENCE
    );
    assert_eq!(
        body.fold_contract.multiplication_table_kappa,
        H4_BINARY_ICOSAHEDRAL_MULTIPLICATION_TABLE_KAPPA_REFERENCE
    );
    assert_eq!(body.fold_contract.root_count, 120);
    assert!(body.fold_contract.opaque_index_distance_forbidden);
    assert!(body.fold_contract.all_states_reachable);
    assert_eq!(body.fold_contract.cayley_distances.len(), 120);
    assert!(body
        .fold_contract
        .cayley_distances
        .iter()
        .all(|entry| entry.distance_to_identity.is_some()));

    let laws = &body.fold_laws;
    assert!(laws.exact_table_identity);
    assert!(laws.exact_table_inverses);
    assert!(laws.exact_table_associativity);
    assert!(laws.exact_table_closure);
    assert_eq!(laws.identity_checks, 240);
    assert_eq!(laws.inverse_checks, 240);
    assert_eq!(laws.associativity_checks, 120 * 120 * 120);
    assert_eq!(laws.grouping_checks.len(), 6);
    let recursive = &laws.recursive_hierarchy_fixture;
    assert_eq!(
        (
            recursive.turn_count,
            recursive.paragraph_count,
            recursive.sentence_count,
            recursive.lexical_unit_count,
        ),
        (2, 3, 5, 9)
    );
    assert!(recursive.flat_equals_sentence_regrouped);
    assert!(recursive.flat_equals_paragraph_regrouped);
    assert!(recursive.flat_equals_recursive_conversation);
    assert!(recursive.all_regroupings_exact);
    assert!(laws.all_grouping_checks_exact);
    assert!(laws.all_laws_exact);
    assert!(laws.grouping_checks.iter().all(|check| {
        check.current_matches_leaf
            && check.previous_matches_leaf
            && check.last_two_matches_direct_fold
            && check.sentence_matches_flat_fold
            && check.paragraph_matches_sentence
            && check.conversation_matches_paragraph
            && check.regrouping_exact
    }));

    let census = &body.collision_census;
    assert_eq!(census.expected_permutations, 120);
    assert_eq!(census.examined_permutations, 120);
    assert_eq!(census.outcomes.len(), 120);
    assert_eq!(census.unique_states, 60);
    assert!(!census.collision_free);
    assert_eq!(census.collision_buckets.len(), 41);
    assert_eq!(census.largest_collision_bucket_size, 5);
    assert!(census
        .collision_buckets
        .iter()
        .all(|bucket| bucket.ordered_token_sequences.len() > 1));

    let work = &body.work_contract;
    assert_eq!(work.expected_permutation_census, 120);
    assert_eq!(work.exercised_permutation_census, 120);
    assert_eq!(work.expected_associativity_checks, 120 * 120 * 120);
    assert_eq!(work.exercised_associativity_checks, 120 * 120 * 120);
    assert!(!work.external_corpus_population_scan_performed);
    assert!(!work.source_model_or_teacher_run_performed);
}

#[test]
fn repaired_state_is_distinct_but_candidate_interaction_truthfully_retains_state_only() {
    let body = &report().body;
    assert_eq!(
        body.probe_status,
        "EXERCISED_FIXED_A1R_NEGATIVE_WITH_UNAVAILABLE_MATCHED_CONTROL"
    );
    assert_eq!(body.terminal_verdict, RETAIN_STATE_ONLY);
    assert_eq!(
        body.successor_effect,
        "A1Q_969_MUST_REMAIN_BLOCKED_BY_AN_EXACT_A1R_SUCCESSOR"
    );

    assert_eq!(body.scope_contrasts.len(), 3);
    for contrast in &body.scope_contrasts {
        assert!(contrast.scope_mask_exact);
        assert!(contrast.legacy_non_digest_summary_equal);
        assert_eq!(contrast.levels.len(), 7);
        for level in &contrast.levels {
            let expected_equal = matches!(
                level.level.as_str(),
                "current" | "previous" | "last-two" | "global"
            );
            assert_eq!(level.expected_equal, expected_equal);
            assert_eq!(level.observed_equal, expected_equal);
        }
    }

    assert!(body.global_order_fixture.lower_scope_inputs_equal);
    assert!(body.global_order_fixture.global_epoch_is_derived_identity);
    assert_ne!(
        body.global_order_fixture.left_global_epoch,
        body.global_order_fixture.right_global_epoch
    );
    assert!(body.global_order_fixture.candidate_rows_equal);
    assert!(body.global_order_fixture.candidate_support_equal);
    assert!(body.global_order_fixture.candidate_work_budget_equal);
    assert_eq!(
        body.global_order_fixture.left_support_denominator_kappa,
        body.global_order_fixture.right_support_denominator_kappa
    );
    assert!(body.global_order_fixture.only_global_state_differs);
    assert_eq!(body.global_order_fixture.levels.len(), 7);
    assert!(body
        .global_order_fixture
        .levels
        .iter()
        .all(|level| level.observed_equal == (level.level != "global")));

    assert_eq!(body.incremental_checks.len(), 6);
    assert!(body
        .incremental_checks
        .iter()
        .all(|check| check.prefix_clone_unchanged && check.exact_reproduction));
    assert!(body.support_invariants_exact);
    assert_eq!(body.support_pair_checks.len(), 3);
    assert!(body.support_pair_checks.iter().all(|pair| {
        pair.left_support_denominator_kappa == pair.right_support_denominator_kappa
            && pair.natural_candidate_union_equal
            && pair.candidate_source_counts_equal
            && pair.candidate_origins_equal
            && pair.row_source_outcomes_equal
            && pair.row_and_candidate_budgets_equal
            && pair.both_competing_targets_in_each_union
            && pair.exact_direct_rows_miss_both_sides
            && pair.legacy_additive_summary_equal
    }));

    let work = &body.work_contract;
    assert_eq!((work.exercised_contrasts, work.expected_contrasts), (3, 3));
    assert_eq!(
        (
            work.exercised_candidate_queries,
            work.expected_candidate_queries
        ),
        (6, 6)
    );
    assert_eq!(
        (
            work.exact_candidate_payload_inversions,
            work.exercised_candidate_payload_inversions,
            work.expected_candidate_payload_inversions
        ),
        (12, 12, 12)
    );
    assert_eq!(
        (
            work.exact_incremental_checks,
            work.exercised_incremental_checks,
            work.expected_incremental_checks
        ),
        (6, 6, 6)
    );
    assert_eq!(work.candidate_ceiling, 8);
    assert_eq!(work.maximum_admitted_candidates_observed, 2);
    assert_eq!(work.rows_per_query_ceiling, 7);
    assert_eq!(
        (work.exercised_row_reads, work.expected_row_reads),
        (42, 42)
    );
    assert_eq!(work.candidate_entry_ceiling_per_query, 56);
    assert_eq!(work.control_arms_per_query, 8);

    assert_eq!(body.candidate_queries.len(), 6);
    let required_controls = BTreeSet::from([
        "current-only",
        "deterministic-conjugation",
        "exact-recall-only",
        "existing-additive-summary",
        "factor-count",
        "full-ordered-hierarchy",
        "hierarchy-disabled",
        "inverse-h4-intervention",
    ]);
    for query in &body.candidate_queries {
        assert_eq!(query.rows.len(), 7);
        assert_eq!(query.candidate_entries_examined, 2);
        assert_eq!(query.candidate_entry_ceiling, 56);
        assert_eq!(query.unique_candidates_before_admission, 2);
        assert_eq!(query.unique_candidates_after_admission, 2);
        assert_eq!(query.retained_candidate_ceiling, 8);
        assert!(query.full_pre_admission_union_observed);
        assert_eq!(query.anchored_candidates_after_admission, 2);
        assert_eq!(query.required_anchored_candidates, 2);
        assert_eq!(query.exact_candidate_payload_inversions, 2);
        assert!(!query.admission_truncated_union);
        assert!(query.exact_direct_rows_miss);
        assert!(query.divisor_rows_miss);
        assert!(query.adjacent_spin_only_support);
        assert!(!query.target_injected);
        assert!(!query.future_events_visible);

        let payloads = query
            .candidate_support
            .iter()
            .map(|candidate| candidate.payload_bytes.as_slice())
            .collect::<BTreeSet<_>>();
        assert_eq!(
            payloads,
            BTreeSet::from([b"ll".as_slice(), b"rr".as_slice()])
        );
        assert!(query
            .candidate_support
            .iter()
            .all(|candidate| candidate.exact_payload_inversion));

        assert_eq!(query.controls.len(), 8);
        assert_eq!(
            query
                .controls
                .iter()
                .map(|control| control.control.as_str())
                .collect::<BTreeSet<_>>(),
            required_controls
        );

        let full = query
            .controls
            .iter()
            .find(|control| control.control == "full-ordered-hierarchy")
            .expect("full ordered hierarchy arm");
        assert_eq!(full.status, "EXERCISED_TIE_ABSTAIN");
        assert!(full.exercised);
        assert_eq!(full.minimum_energy, Some(2));
        assert_eq!(full.minimum_energy_candidates, ["ll", "rr"]);
        assert_eq!(
            full.canonical_address_tiebreak_rule,
            "lexicographically-smallest-address-kappa-diagnostic-only"
        );
        assert_eq!(full.canonical_address_tiebreak_token.as_deref(), Some("rr"));
        assert_eq!(full.selected_token, None);
        assert!(full.tie);
        assert!(full.abstained);
        assert!(!full.intended_target_selected);

        let additive = query
            .controls
            .iter()
            .find(|control| control.control == "existing-additive-summary")
            .expect("existing additive summary arm");
        assert_eq!(
            additive.status,
            "NOT_EXERCISED_NO_PREDECLARED_ADDITIVE_SCORER"
        );
        assert!(!additive.exercised);
        assert!(additive.legacy_additive_state.is_some());
        assert!(additive
            .candidates
            .iter()
            .all(|candidate| candidate.energy.is_none()));
        assert!(additive.minimum_energy_candidates.is_empty());
        assert!(additive.selected_token.is_none());
        assert!(!additive.tie);
        assert!(additive.abstained);

        let exact = query
            .controls
            .iter()
            .find(|control| control.control == "exact-recall-only")
            .expect("exact recall arm");
        assert_eq!(exact.status, "EXERCISED_NO_EXACT_HIT_ABSTAIN");
        assert!(exact.exercised);
        assert_eq!(exact.minimum_energy, None);
        assert!(exact.minimum_energy_candidates.is_empty());
        assert!(exact.selected_token.is_none());
        assert!(!exact.tie);
        assert!(exact.abstained);
    }

    let transition = &body.transition_readout_summary;
    assert_eq!(transition.exercised_queries, 6);
    assert_eq!(
        transition.queries_with_distinct_candidate_relative_states,
        6
    );
    assert_eq!(
        transition.queries_with_distinct_relative_states_but_equal_energy,
        6
    );
    assert_eq!(transition.paired_same_candidate_comparisons, 6);
    assert_eq!(
        transition.paired_same_candidate_relative_state_differences,
        5
    );
    assert!(transition.scalar_readout_degeneracy_observed);

    assert_eq!(
        body.scoring_summary.status,
        "EXERCISED_WITH_UNAVAILABLE_MATCHED_CONTROL"
    );
    assert_eq!(body.scoring_summary.required_queries, 6);
    assert_eq!(body.scoring_summary.exercised_queries, 6);
    assert_eq!(body.scoring_summary.full_strict_correct, 0);
    assert_eq!(body.scoring_summary.full_ties, 6);
    assert!(body.scoring_summary.all_required_controls_present);
    assert!(!body.scoring_summary.all_required_controls_exercised);
    assert!(body.scoring_summary.any_exercised_control_not_weaker);
    assert!(!body.scoring_summary.every_control_weaker);
    assert!(!body.scoring_summary.positive_contract_satisfied);

    assert!(body.claim_boundary.representation_repair_only);
    assert!(!body.claim_boundary.full_recursive_attention_qualified);
    assert!(!body.claim_boundary.generation_unblocked);
    assert!(!body.claim_boundary.correctness_established);
    assert!(!body.claim_boundary.reasoning_established);
    assert!(!body.claim_boundary.digest_distance_used_as_geometry);
    assert!(
        body.claim_boundary
            .all_identity_and_provenance_bits_excluded_from_geometry
    );
    assert!(!body.claim_boundary.candidate_support_or_admission_modified);
}
