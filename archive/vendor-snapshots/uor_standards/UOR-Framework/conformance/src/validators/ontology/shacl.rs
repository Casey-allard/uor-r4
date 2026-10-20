//! SHACL validator.
//!
//! Validates the 100 OWL instance test graphs against the UOR SHACL shapes.
//! Each test graph is defined as a Turtle string in `tests/fixtures/`.
//! Validation checks structural constraints without a full SHACL engine:
//! - Required properties are present
//! - Type assertions are correct
//! - Cardinality minimums are met

use uor_ontology::counts;

use crate::report::{ConformanceReport, TestResult};
use crate::tests;

/// Runs all 110 SHACL instance conformance tests.
pub fn validate() -> ConformanceReport {
    let mut report = ConformanceReport::new();
    let before_tests = report.results.len();

    run_test("test1_ring", tests::fixtures::TEST1_RING, &mut report);
    run_test(
        "test2_primitives",
        tests::fixtures::TEST2_PRIMITIVES,
        &mut report,
    );
    run_test(
        "test3_term_graph",
        tests::fixtures::TEST3_TERM_GRAPH,
        &mut report,
    );
    run_test(
        "test4_state_lifecycle",
        tests::fixtures::TEST4_STATE_LIFECYCLE,
        &mut report,
    );
    run_test(
        "test5_partition",
        tests::fixtures::TEST5_PARTITION,
        &mut report,
    );
    run_test(
        "test6_critical_identity",
        tests::fixtures::TEST6_CRITICAL_IDENTITY,
        &mut report,
    );
    run_test(
        "test7_end_to_end",
        tests::fixtures::TEST7_END_TO_END,
        &mut report,
    );
    run_test(
        "test8_free_rank",
        tests::fixtures::TEST8_FREE_RANK,
        &mut report,
    );
    run_test(
        "test9_constraint_algebra",
        tests::fixtures::TEST9_CONSTRAINT_ALGEBRA,
        &mut report,
    );
    run_test(
        "test10_iterative_resolution",
        tests::fixtures::TEST10_ITERATIVE_RESOLUTION,
        &mut report,
    );
    run_test(
        "test11_composition",
        tests::fixtures::TEST11_COMPOSITION,
        &mut report,
    );
    run_test(
        "test12_factorization",
        tests::fixtures::TEST12_FACTORIZATION,
        &mut report,
    );
    run_test(
        "test13_canonical_form",
        tests::fixtures::TEST13_CANONICAL_FORM,
        &mut report,
    );
    run_test(
        "test14_content_addressing",
        tests::fixtures::TEST14_CONTENT_ADDRESSING,
        &mut report,
    );
    run_test(
        "test15_boolean_sat",
        tests::fixtures::TEST15_BOOLEAN_SAT,
        &mut report,
    );
    run_test(
        "test16_algebraic_identities",
        tests::fixtures::TEST16_ALGEBRAIC_IDENTITIES,
        &mut report,
    );
    run_test(
        "test17_inter_algebra_maps",
        tests::fixtures::TEST17_INTER_ALGEBRA_MAPS,
        &mut report,
    );
    run_test(
        "test18_analytical_completeness",
        tests::fixtures::TEST18_ANALYTICAL_COMPLETENESS,
        &mut report,
    );
    run_test(
        "test19_homological_pipeline",
        tests::fixtures::TEST19_HOMOLOGICAL_PIPELINE,
        &mut report,
    );
    run_test(
        "test20_sheaf_consistency",
        tests::fixtures::TEST20_SHEAF_CONSISTENCY,
        &mut report,
    );
    run_test(
        "test21_topological_delta",
        tests::fixtures::TEST21_TOPOLOGICAL_DELTA,
        &mut report,
    );
    run_test(
        "test22_index_bridge",
        tests::fixtures::TEST22_INDEX_BRIDGE,
        &mut report,
    );
    run_test(
        "test23_identity_grounding",
        tests::fixtures::TEST23_IDENTITY_GROUNDING,
        &mut report,
    );
    run_test(
        "test24_verification_domain",
        tests::fixtures::TEST24_VERIFICATION_DOMAIN,
        &mut report,
    );
    run_test(
        "test25_geometric_character",
        tests::fixtures::TEST25_GEOMETRIC_CHARACTER,
        &mut report,
    );
    run_test(
        "test26_complexity_class",
        tests::fixtures::TEST26_COMPLEXITY_CLASS,
        &mut report,
    );
    run_test(
        "test27_rewrite_rule",
        tests::fixtures::TEST27_REWRITE_RULE,
        &mut report,
    );
    run_test(
        "test28_measurement_unit",
        tests::fixtures::TEST28_MEASUREMENT_UNIT,
        &mut report,
    );
    run_test(
        "test29_triad_projection",
        tests::fixtures::TEST29_TRIAD_PROJECTION,
        &mut report,
    );
    run_test(
        "test30_proof_coverage",
        tests::fixtures::TEST30_PROOF_COVERAGE,
        &mut report,
    );
    run_test(
        "test31_witt_level",
        tests::fixtures::TEST31_WITT_LEVEL,
        &mut report,
    );
    run_test(
        "test32_arc_grounding",
        tests::fixtures::TEST32_ARC_GROUNDING,
        &mut report,
    );
    run_test(
        "test33_graph_gaps",
        tests::fixtures::TEST33_GRAPH_GAPS,
        &mut report,
    );
    run_test(
        "test34_completeness_candidate",
        tests::fixtures::TEST34_COMPLETENESS_CANDIDATE,
        &mut report,
    );
    run_test(
        "test35_completeness_certificate",
        tests::fixtures::TEST35_COMPLETENESS_CERTIFICATE,
        &mut report,
    );
    run_test(
        "test36_w16_ring",
        tests::fixtures::TEST36_W16_RING,
        &mut report,
    );
    run_test(
        "test37_witt_level_binding",
        tests::fixtures::TEST37_WITT_LEVEL_BINDING,
        &mut report,
    );
    run_test(
        "test38_session_lifecycle",
        tests::fixtures::TEST38_SESSION_LIFECYCLE,
        &mut report,
    );
    run_test(
        "test39_session_boundary",
        tests::fixtures::TEST39_SESSION_BOUNDARY,
        &mut report,
    );
    run_test(
        "test40_type_synthesis_goal",
        tests::fixtures::TEST40_TYPE_SYNTHESIS_GOAL,
        &mut report,
    );
    run_test(
        "test41_synthesis_result",
        tests::fixtures::TEST41_SYNTHESIS_RESULT,
        &mut report,
    );
    run_test(
        "test42_witt_lift",
        tests::fixtures::TEST42_WITT_LIFT,
        &mut report,
    );
    run_test(
        "test43_spectral_sequence",
        tests::fixtures::TEST43_SPECTRAL_SEQUENCE,
        &mut report,
    );
    run_test(
        "test44_monodromy_flat",
        tests::fixtures::TEST44_MONODROMY_FLAT,
        &mut report,
    );
    run_test(
        "test45_monodromy_twisted",
        tests::fixtures::TEST45_MONODROMY_TWISTED,
        &mut report,
    );
    run_test(
        "test46_monodromy_pipeline",
        tests::fixtures::TEST46_MONODROMY_PIPELINE,
        &mut report,
    );
    run_test(
        "test47_thermo_pipeline",
        tests::fixtures::TEST47_THERMO_PIPELINE,
        &mut report,
    );
    run_test(
        "test48_phase_diagram",
        tests::fixtures::TEST48_PHASE_DIAGRAM,
        &mut report,
    );
    run_test(
        "test49_reversible_resolution",
        tests::fixtures::TEST49_REVERSIBLE_RESOLUTION,
        &mut report,
    );
    run_test(
        "test50_jacobian_resolver",
        tests::fixtures::TEST50_JACOBIAN_RESOLVER,
        &mut report,
    );
    run_test(
        "test51_product_type_pipeline",
        tests::fixtures::TEST51_PRODUCT_TYPE_PIPELINE,
        &mut report,
    );
    run_test(
        "test52_sum_type_variant",
        tests::fixtures::TEST52_SUM_TYPE_VARIANT,
        &mut report,
    );
    run_test(
        "test53_superposed_site",
        tests::fixtures::TEST53_SUPERPOSED_SITE,
        &mut report,
    );
    run_test(
        "test54_grounded_context",
        tests::fixtures::TEST54_GROUNDED_CONTEXT,
        &mut report,
    );
    run_test(
        "test55_grounding_witness",
        tests::fixtures::TEST55_GROUNDING_WITNESS,
        &mut report,
    );
    run_test(
        "test56_domain_grounding_record",
        tests::fixtures::TEST56_DOMAIN_GROUNDING_RECORD,
        &mut report,
    );
    run_test(
        "test57_grounding_phase",
        tests::fixtures::TEST57_GROUNDING_PHASE,
        &mut report,
    );
    run_test(
        "test58_grounding_certificate",
        tests::fixtures::TEST58_GROUNDING_CERTIFICATE,
        &mut report,
    );
    run_test(
        "test59_grounding_aware_resolver",
        tests::fixtures::TEST59_GROUNDING_AWARE_RESOLVER,
        &mut report,
    );
    run_test(
        "test60_impossibility_witness",
        tests::fixtures::TEST60_IMPOSSIBILITY_WITNESS,
        &mut report,
    );
    run_test(
        "test61_morphospace_record",
        tests::fixtures::TEST61_MORPHOSPACE_RECORD,
        &mut report,
    );
    run_test(
        "test62_morphospace_boundary",
        tests::fixtures::TEST62_MORPHOSPACE_BOUNDARY,
        &mut report,
    );
    run_test(
        "test63_forbidden_signature",
        tests::fixtures::TEST63_FORBIDDEN_SIGNATURE,
        &mut report,
    );
    run_test(
        "test64_achievability_status",
        tests::fixtures::TEST64_ACHIEVABILITY_STATUS,
        &mut report,
    );
    run_test(
        "test65_geodesic_trace",
        tests::fixtures::TEST65_GEODESIC_TRACE,
        &mut report,
    );
    run_test(
        "test66_geodesic_certificate",
        tests::fixtures::TEST66_GEODESIC_CERTIFICATE,
        &mut report,
    );
    run_test(
        "test67_geodesic_violation",
        tests::fixtures::TEST67_GEODESIC_VIOLATION,
        &mut report,
    );
    run_test(
        "test68_geodesic_validator",
        tests::fixtures::TEST68_GEODESIC_VALIDATOR,
        &mut report,
    );
    run_test(
        "test69_geodesic_ordered",
        tests::fixtures::TEST69_GEODESIC_ORDERED,
        &mut report,
    );
    run_test(
        "test70_measurement_resolver",
        tests::fixtures::TEST70_MEASUREMENT_RESOLVER,
        &mut report,
    );
    run_test(
        "test71_measurement_event",
        tests::fixtures::TEST71_MEASUREMENT_EVENT,
        &mut report,
    );
    run_test(
        "test72_measurement_certificate",
        tests::fixtures::TEST72_MEASUREMENT_CERTIFICATE,
        &mut report,
    );
    run_test(
        "test73_collapsed_site_state",
        tests::fixtures::TEST73_COLLAPSED_SITE_STATE,
        &mut report,
    );
    run_test(
        "test74_quantum_thermodynamic",
        tests::fixtures::TEST74_QUANTUM_THERMODYNAMIC,
        &mut report,
    );
    // Amendment 37: Gap Closure tests
    run_test(
        "test75_partition_product",
        tests::fixtures::TEST75_PARTITION_PRODUCT,
        &mut report,
    );
    run_test(
        "test76_partition_coproduct",
        tests::fixtures::TEST76_PARTITION_COPRODUCT,
        &mut report,
    );
    run_test(
        "test77_geodesic_evidence",
        tests::fixtures::TEST77_GEODESIC_EVIDENCE,
        &mut report,
    );
    run_test(
        "test78_born_rule",
        tests::fixtures::TEST78_BORN_RULE,
        &mut report,
    );
    run_test(
        "test79_measurement_outcome",
        tests::fixtures::TEST79_MEASUREMENT_OUTCOME,
        &mut report,
    );
    run_test(
        "test80_partition_exhaustive",
        tests::fixtures::TEST80_PARTITION_EXHAUSTIVE,
        &mut report,
    );
    run_test(
        "test81_dihedral_algebra",
        tests::fixtures::TEST81_DIHEDRAL_ALGEBRA,
        &mut report,
    );
    run_test(
        "test82_level_successor",
        tests::fixtures::TEST82_LEVEL_SUCCESSOR,
        &mut report,
    );
    run_test(
        "test83_amplitude_normalization",
        tests::fixtures::TEST83_AMPLITUDE_NORMALIZATION,
        &mut report,
    );
    run_test(
        "test84_enum_variant",
        tests::fixtures::TEST84_ENUM_VARIANT,
        &mut report,
    );
    // Amendment 39: Q1 Lift fixtures
    run_test(
        "test85_w16_ring_grounding",
        tests::fixtures::TEST85_W16_RING_GROUNDING,
        &mut report,
    );
    run_test(
        "test86_witt_lift_trivial",
        tests::fixtures::TEST86_WITT_LIFT_TRIVIAL,
        &mut report,
    );
    run_test(
        "test87_spectral_convergence",
        tests::fixtures::TEST87_SPECTRAL_CONVERGENCE,
        &mut report,
    );
    run_test(
        "test88_lift_obstruction_nontrivial",
        tests::fixtures::TEST88_LIFT_OBSTRUCTION_NONTRIVIAL,
        &mut report,
    );
    run_test(
        "test89_lift_refinement_suggestion",
        tests::fixtures::TEST89_LIFT_REFINEMENT_SUGGESTION,
        &mut report,
    );
    run_test(
        "test90_resolved_lift",
        tests::fixtures::TEST90_RESOLVED_LIFT,
        &mut report,
    );
    // Amendment 39: Q1 Inverse Pipeline fixtures
    run_test(
        "test91_synthesis_goal_w16",
        tests::fixtures::TEST91_SYNTHESIS_GOAL_W16,
        &mut report,
    );
    run_test(
        "test92_synthesis_checkpoint",
        tests::fixtures::TEST92_SYNTHESIS_CHECKPOINT,
        &mut report,
    );
    run_test(
        "test93_synthesis_signature",
        tests::fixtures::TEST93_SYNTHESIS_SIGNATURE,
        &mut report,
    );
    run_test(
        "test94_synthesized_type",
        tests::fixtures::TEST94_SYNTHESIZED_TYPE,
        &mut report,
    );
    run_test(
        "test95_unreachable_signature",
        tests::fixtures::TEST95_UNREACHABLE_SIGNATURE,
        &mut report,
    );
    // Amendment 40: Q1 Normative Certification fixtures
    run_test(
        "test96_geodesic_trace_w16",
        tests::fixtures::TEST96_GEODESIC_TRACE_W16,
        &mut report,
    );
    run_test(
        "test97_evidence_bundle_ar1",
        tests::fixtures::TEST97_EVIDENCE_BUNDLE_AR1,
        &mut report,
    );
    run_test(
        "test98_evidence_bundle_dc10",
        tests::fixtures::TEST98_EVIDENCE_BUNDLE_DC10,
        &mut report,
    );
    run_test(
        "test99_measurement_born_w16",
        tests::fixtures::TEST99_MEASUREMENT_BORN_W16,
        &mut report,
    );
    run_test(
        "test100_normative_chain",
        tests::fixtures::TEST100_NORMATIVE_CHAIN,
        &mut report,
    );
    // Amendment 41: Tower chain test fixtures
    run_test(
        "test101_lift_chain_flat",
        tests::fixtures::TEST101_LIFT_CHAIN_FLAT,
        &mut report,
    );
    run_test(
        "test102_lift_chain_twisted",
        tests::fixtures::TEST102_LIFT_CHAIN_TWISTED,
        &mut report,
    );
    run_test(
        "test103_obstruction_chain_empty",
        tests::fixtures::TEST103_OBSTRUCTION_CHAIN_EMPTY,
        &mut report,
    );
    run_test(
        "test104_obstruction_chain_nontrivial",
        tests::fixtures::TEST104_OBSTRUCTION_CHAIN_NONTRIVIAL,
        &mut report,
    );
    run_test(
        "test105_lift_chain_certificate",
        tests::fixtures::TEST105_LIFT_CHAIN_CERTIFICATE,
        &mut report,
    );
    run_test(
        "test106_chain_audit_trail",
        tests::fixtures::TEST106_CHAIN_AUDIT_TRAIL,
        &mut report,
    );
    run_test(
        "test107_tower_resolver",
        tests::fixtures::TEST107_TOWER_RESOLVER,
        &mut report,
    );
    run_test(
        "test108_inductive_proof",
        tests::fixtures::TEST108_INDUCTIVE_PROOF,
        &mut report,
    );
    run_test(
        "test109_validity_scope",
        tests::fixtures::TEST109_VALIDITY_SCOPE,
        &mut report,
    );
    run_test(
        "test110_tower_roundtrip",
        tests::fixtures::TEST110_TOWER_ROUNDTRIP,
        &mut report,
    );
    // Amendment 43: Cryptographic Primitive Pinning
    run_test(
        "test111_address_crypto_pinning",
        tests::fixtures::TEST111_ADDRESS_CRYPTO_PINNING,
        &mut report,
    );
    run_test(
        "test112_address_canonical_bytes",
        tests::fixtures::TEST112_ADDRESS_CANONICAL_BYTES,
        &mut report,
    );
    // Amendment 44: Structural Gap Closure SHACL tests
    run_test(
        "test113_carry_constraint_pinning",
        tests::fixtures::TEST113_CARRY_CONSTRAINT_PINNING,
        &mut report,
    );
    run_test(
        "test114_joint_satisfiability",
        tests::fixtures::TEST114_JOINT_SATISFIABILITY,
        &mut report,
    );
    run_test(
        "test115_dihedral_inverse_order",
        tests::fixtures::TEST115_DIHEDRAL_INVERSE_ORDER,
        &mut report,
    );
    run_test(
        "test116_constraint_expressiveness",
        tests::fixtures::TEST116_CONSTRAINT_EXPRESSIVENESS,
        &mut report,
    );
    run_test(
        "test117_sumtype_topology",
        tests::fixtures::TEST117_SUMTYPE_TOPOLOGY,
        &mut report,
    );
    run_test(
        "test118_synthesis_reachability",
        tests::fixtures::TEST118_SYNTHESIS_REACHABILITY,
        &mut report,
    );
    run_test(
        "test119_obstruction_termination",
        tests::fixtures::TEST119_OBSTRUCTION_TERMINATION,
        &mut report,
    );
    run_test(
        "test120_coefficient_ring",
        tests::fixtures::TEST120_COEFFICIENT_RING,
        &mut report,
    );
    run_test(
        "test121_gluing_feedback",
        tests::fixtures::TEST121_GLUING_FEEDBACK,
        &mut report,
    );
    run_test(
        "test122_session_grounding",
        tests::fixtures::TEST122_SESSION_GROUNDING,
        &mut report,
    );
    run_test(
        "test123_amplitude_index",
        tests::fixtures::TEST123_AMPLITUDE_INDEX,
        &mut report,
    );
    // Amendment 46: SHACL fixture coverage gap closure
    run_test("test124_glyph", tests::fixtures::TEST124_GLYPH, &mut report);
    run_test(
        "test125_dihedral_group",
        tests::fixtures::TEST125_DIHEDRAL_GROUP,
        &mut report,
    );
    run_test(
        "test126_validity_scope_kind",
        tests::fixtures::TEST126_VALIDITY_SCOPE_KIND,
        &mut report,
    );
    run_test(
        "test127_witt_level_resolver",
        tests::fixtures::TEST127_WITT_LEVEL_RESOLVER,
        &mut report,
    );
    run_test(
        "test128_stratum_observable",
        tests::fixtures::TEST128_STRATUM_OBSERVABLE,
        &mut report,
    );
    run_test(
        "test129_metric_observable",
        tests::fixtures::TEST129_METRIC_OBSERVABLE,
        &mut report,
    );
    run_test(
        "test130_path_observable",
        tests::fixtures::TEST130_PATH_OBSERVABLE,
        &mut report,
    );
    run_test(
        "test131_reduction_observable",
        tests::fixtures::TEST131_REDUCTION_OBSERVABLE,
        &mut report,
    );
    run_test(
        "test132_holonomy_observable",
        tests::fixtures::TEST132_HOLONOMY_OBSERVABLE,
        &mut report,
    );
    run_test(
        "test133_incompatibility_metric",
        tests::fixtures::TEST133_INCOMPATIBILITY_METRIC,
        &mut report,
    );
    run_test(
        "test134_stratum_value",
        tests::fixtures::TEST134_STRATUM_VALUE,
        &mut report,
    );
    run_test(
        "test135_stratum_delta",
        tests::fixtures::TEST135_STRATUM_DELTA,
        &mut report,
    );
    run_test(
        "test136_stratum_trajectory",
        tests::fixtures::TEST136_STRATUM_TRAJECTORY,
        &mut report,
    );
    run_test(
        "test137_path_length",
        tests::fixtures::TEST137_PATH_LENGTH,
        &mut report,
    );
    run_test(
        "test138_total_variation",
        tests::fixtures::TEST138_TOTAL_VARIATION,
        &mut report,
    );
    run_test(
        "test139_winding_number",
        tests::fixtures::TEST139_WINDING_NUMBER,
        &mut report,
    );
    run_test(
        "test140_reduction_length",
        tests::fixtures::TEST140_REDUCTION_LENGTH,
        &mut report,
    );
    run_test(
        "test141_reduction_count",
        tests::fixtures::TEST141_REDUCTION_COUNT,
        &mut report,
    );
    run_test(
        "test142_catastrophe_threshold",
        tests::fixtures::TEST142_CATASTROPHE_THRESHOLD,
        &mut report,
    );
    run_test(
        "test143_catastrophe_count",
        tests::fixtures::TEST143_CATASTROPHE_COUNT,
        &mut report,
    );
    run_test(
        "test144_commutator",
        tests::fixtures::TEST144_COMMUTATOR,
        &mut report,
    );
    run_test(
        "test145_curvature_flux",
        tests::fixtures::TEST145_CURVATURE_FLUX,
        &mut report,
    );
    run_test(
        "test146_parallel_transport",
        tests::fixtures::TEST146_PARALLEL_TRANSPORT,
        &mut report,
    );
    run_test(
        "test147_reduction_entropy",
        tests::fixtures::TEST147_REDUCTION_ENTROPY,
        &mut report,
    );
    run_test(
        "test148_phase_boundary_type",
        tests::fixtures::TEST148_PHASE_BOUNDARY_TYPE,
        &mut report,
    );
    run_test(
        "test149_face_map",
        tests::fixtures::TEST149_FACE_MAP,
        &mut report,
    );
    run_test(
        "test150_nerve_functor",
        tests::fixtures::TEST150_NERVE_FUNCTOR,
        &mut report,
    );
    run_test(
        "test151_chain_functor",
        tests::fixtures::TEST151_CHAIN_FUNCTOR,
        &mut report,
    );
    run_test(
        "test152_restriction_map",
        tests::fixtures::TEST152_RESTRICTION_MAP,
        &mut report,
    );
    run_test(
        "test153_coherence_proof",
        tests::fixtures::TEST153_COHERENCE_PROOF,
        &mut report,
    );
    run_test(
        "test154_derivation_step",
        tests::fixtures::TEST154_DERIVATION_STEP,
        &mut report,
    );
    run_test(
        "test155_computation_step",
        tests::fixtures::TEST155_COMPUTATION_STEP,
        &mut report,
    );
    run_test(
        "test156_trace_metrics",
        tests::fixtures::TEST156_TRACE_METRICS,
        &mut report,
    );
    run_test(
        "test157_isometry_certificate",
        tests::fixtures::TEST157_ISOMETRY_CERTIFICATE,
        &mut report,
    );
    run_test(
        "test158_lift_chain_certificate",
        tests::fixtures::TEST158_LIFT_CHAIN_CERTIFICATE,
        &mut report,
    );
    run_test(
        "test159_chain_audit_trail",
        tests::fixtures::TEST159_CHAIN_AUDIT_TRAIL,
        &mut report,
    );
    run_test(
        "test160_shared_context",
        tests::fixtures::TEST160_SHARED_CONTEXT,
        &mut report,
    );
    run_test(
        "test161_execution_policy",
        tests::fixtures::TEST161_EXECUTION_POLICY,
        &mut report,
    );
    run_test(
        "test162_session_composition",
        tests::fixtures::TEST162_SESSION_COMPOSITION,
        &mut report,
    );
    run_test(
        "test163_distributed_grounding",
        tests::fixtures::TEST163_DISTRIBUTED_GROUNDING,
        &mut report,
    );
    // Amendment 49: User-space class coverage fixtures
    run_test(
        "test164_embedding",
        tests::fixtures::TEST164_EMBEDDING,
        &mut report,
    );
    run_test(
        "test165_action",
        tests::fixtures::TEST165_ACTION,
        &mut report,
    );
    run_test(
        "test166_session_boundary_type",
        tests::fixtures::TEST166_SESSION_BOUNDARY_TYPE,
        &mut report,
    );
    run_test(
        "test167_metric_axis",
        tests::fixtures::TEST167_METRIC_AXIS,
        &mut report,
    );
    run_test(
        "test168_witt_carry",
        tests::fixtures::TEST168_WITT_CARRY,
        &mut report,
    );
    run_test(
        "test169_arithmetic_valuation",
        tests::fixtures::TEST169_ARITHMETIC_VALUATION,
        &mut report,
    );
    // Amendment 54: Homotopy Nerve
    run_test(
        "test170_kan_complex",
        tests::fixtures::TEST170_KAN_COMPLEX,
        &mut report,
    );
    run_test(
        "test171_postnikov_truncation",
        tests::fixtures::TEST171_POSTNIKOV_TRUNCATION,
        &mut report,
    );
    run_test(
        "test172_homotopy_group",
        tests::fixtures::TEST172_HOMOTOPY_GROUP,
        &mut report,
    );
    run_test(
        "test173_homotopy_end_to_end",
        tests::fixtures::TEST173_HOMOTOPY_END_TO_END,
        &mut report,
    );
    // Amendment 55: Homotopy Pipeline
    run_test(
        "test174_homotopy_resolver",
        tests::fixtures::TEST174_HOMOTOPY_RESOLVER,
        &mut report,
    );
    run_test(
        "test175_homotopy_pipeline",
        tests::fixtures::TEST175_HOMOTOPY_PIPELINE,
        &mut report,
    );
    // Amendment 56: Moduli Space
    run_test(
        "test176_moduli_space",
        tests::fixtures::TEST176_MODULI_SPACE,
        &mut report,
    );
    run_test(
        "test177_deformation_complex",
        tests::fixtures::TEST177_DEFORMATION_COMPLEX,
        &mut report,
    );
    run_test(
        "test178_holonomy_stratum",
        tests::fixtures::TEST178_HOLONOMY_STRATUM,
        &mut report,
    );
    run_test(
        "test179_moduli_end_to_end",
        tests::fixtures::TEST179_MODULI_END_TO_END,
        &mut report,
    );
    // Amendment 57: Moduli Resolver
    run_test(
        "test180_moduli_resolver",
        tests::fixtures::TEST180_MODULI_RESOLVER,
        &mut report,
    );
    run_test(
        "test181_stratification_record",
        tests::fixtures::TEST181_STRATIFICATION_RECORD,
        &mut report,
    );
    run_test(
        "test182_whitehead_product",
        tests::fixtures::TEST182_WHITEHEAD_PRODUCT,
        &mut report,
    );
    run_test(
        "test183_deformation_family",
        tests::fixtures::TEST183_DEFORMATION_FAMILY,
        &mut report,
    );
    run_test(
        "test184_versal_deformation",
        tests::fixtures::TEST184_VERSAL_DEFORMATION,
        &mut report,
    );
    // Amendment 58: Carry Algebra
    run_test(
        "test185_carry_chain",
        tests::fixtures::TEST185_CARRY_CHAIN,
        &mut report,
    );
    run_test(
        "test186_carry_event",
        tests::fixtures::TEST186_CARRY_EVENT,
        &mut report,
    );
    run_test(
        "test187_carry_profile",
        tests::fixtures::TEST187_CARRY_PROFILE,
        &mut report,
    );
    run_test(
        "test188_encoding_configuration",
        tests::fixtures::TEST188_ENCODING_CONFIGURATION,
        &mut report,
    );
    run_test(
        "test189_encoding_quality",
        tests::fixtures::TEST189_ENCODING_QUALITY,
        &mut report,
    );
    run_test(
        "test190_base_metric",
        tests::fixtures::TEST190_BASE_METRIC,
        &mut report,
    );
    run_test(
        "test191_grounding_observable",
        tests::fixtures::TEST191_GROUNDING_OBSERVABLE,
        &mut report,
    );
    run_test(
        "test192_euler_characteristic",
        tests::fixtures::TEST192_EULER_CHARACTERISTIC,
        &mut report,
    );
    run_test(
        "test193_galois_connection",
        tests::fixtures::TEST193_GALOIS_CONNECTION,
        &mut report,
    );
    run_test(
        "test194_nerve_operations",
        tests::fixtures::TEST194_NERVE_OPERATIONS,
        &mut report,
    );
    // Amendment 61: Structural Types
    run_test(
        "test195_scalar_symbol_type",
        tests::fixtures::TEST195_SCALAR_SYMBOL_TYPE,
        &mut report,
    );
    run_test(
        "test196_sequence_tuple_type",
        tests::fixtures::TEST196_SEQUENCE_TUPLE_TYPE,
        &mut report,
    );
    run_test(
        "test197_graph_tree_type",
        tests::fixtures::TEST197_GRAPH_TREE_TYPE,
        &mut report,
    );
    // Amendment 62: Composed Operations
    run_test(
        "test198_composed_operation",
        tests::fixtures::TEST198_COMPOSED_OPERATION,
        &mut report,
    );
    run_test(
        "test199_dispatch_operation",
        tests::fixtures::TEST199_DISPATCH_OPERATION,
        &mut report,
    );
    run_test(
        "test200_inference_operation",
        tests::fixtures::TEST200_INFERENCE_OPERATION,
        &mut report,
    );
    run_test(
        "test201_accumulation_operation",
        tests::fixtures::TEST201_ACCUMULATION_OPERATION,
        &mut report,
    );
    run_test(
        "test202_lease_partition_operation",
        tests::fixtures::TEST202_LEASE_PARTITION_OPERATION,
        &mut report,
    );
    run_test(
        "test203_session_composition_operation",
        tests::fixtures::TEST203_SESSION_COMPOSITION_OPERATION,
        &mut report,
    );
    // Amendment 63: Reduction Core
    run_test(
        "test204_euler_reduction",
        tests::fixtures::TEST204_EULER_REDUCTION,
        &mut report,
    );
    run_test(
        "test205_reduction_step",
        tests::fixtures::TEST205_REDUCTION_STEP,
        &mut report,
    );
    run_test(
        "test206_reduction_state",
        tests::fixtures::TEST206_REDUCTION_STATE,
        &mut report,
    );
    run_test(
        "test207_phase_gate",
        tests::fixtures::TEST207_PHASE_GATE,
        &mut report,
    );
    run_test("test208_epoch", tests::fixtures::TEST208_EPOCH, &mut report);
    run_test(
        "test209_predicate_expression",
        tests::fixtures::TEST209_PREDICATE_EXPRESSION,
        &mut report,
    );
    run_test(
        "test210_guard_expression",
        tests::fixtures::TEST210_GUARD_EXPRESSION,
        &mut report,
    );
    run_test(
        "test211_transition_effect",
        tests::fixtures::TEST211_TRANSITION_EFFECT,
        &mut report,
    );
    run_test(
        "test212_service_window",
        tests::fixtures::TEST212_SERVICE_WINDOW,
        &mut report,
    );
    run_test(
        "test213_reduction_transaction",
        tests::fixtures::TEST213_REDUCTION_TRANSACTION,
        &mut report,
    );
    run_test(
        "test214_feasibility_result",
        tests::fixtures::TEST214_FEASIBILITY_RESULT,
        &mut report,
    );
    run_test(
        "test215_lease_state",
        tests::fixtures::TEST215_LEASE_STATE,
        &mut report,
    );
    run_test(
        "test216_managed_lease",
        tests::fixtures::TEST216_MANAGED_LEASE,
        &mut report,
    );
    run_test(
        "test217_back_pressure",
        tests::fixtures::TEST217_BACK_PRESSURE,
        &mut report,
    );
    run_test(
        "test218_deferred_query",
        tests::fixtures::TEST218_DEFERRED_QUERY,
        &mut report,
    );
    run_test(
        "test219_convergence_level",
        tests::fixtures::TEST219_CONVERGENCE_LEVEL,
        &mut report,
    );
    run_test(
        "test220_hopf_fiber",
        tests::fixtures::TEST220_HOPF_FIBER,
        &mut report,
    );
    run_test(
        "test221_convergence_residual",
        tests::fixtures::TEST221_CONVERGENCE_RESIDUAL,
        &mut report,
    );
    run_test(
        "test222_commutative_subspace",
        tests::fixtures::TEST222_COMMUTATIVE_SUBSPACE,
        &mut report,
    );
    run_test(
        "test223_associative_subalgebra",
        tests::fixtures::TEST223_ASSOCIATIVE_SUBALGEBRA,
        &mut report,
    );
    run_test(
        "test224_normed_division_algebra",
        tests::fixtures::TEST224_NORMED_DIVISION_ALGEBRA,
        &mut report,
    );
    run_test(
        "test225_cayley_dickson",
        tests::fixtures::TEST225_CAYLEY_DICKSON,
        &mut report,
    );
    run_test(
        "test226_multiplication_table",
        tests::fixtures::TEST226_MULTIPLICATION_TABLE,
        &mut report,
    );
    run_test(
        "test227_algebra_commutator",
        tests::fixtures::TEST227_ALGEBRA_COMMUTATOR,
        &mut report,
    );
    run_test(
        "test228_algebra_associator",
        tests::fixtures::TEST228_ALGEBRA_ASSOCIATOR,
        &mut report,
    );
    run_test(
        "test229_interaction_context",
        tests::fixtures::TEST229_INTERACTION_CONTEXT,
        &mut report,
    );
    run_test(
        "test230_commutator_state",
        tests::fixtures::TEST230_COMMUTATOR_STATE,
        &mut report,
    );
    run_test(
        "test231_associator_state",
        tests::fixtures::TEST231_ASSOCIATOR_STATE,
        &mut report,
    );
    run_test(
        "test232_three_way_site",
        tests::fixtures::TEST232_THREE_WAY_SITE,
        &mut report,
    );
    run_test(
        "test233_negotiation_trace",
        tests::fixtures::TEST233_NEGOTIATION_TRACE,
        &mut report,
    );
    run_test(
        "test234_interaction_nerve",
        tests::fixtures::TEST234_INTERACTION_NERVE,
        &mut report,
    );
    run_test(
        "test235_monoidal_product",
        tests::fixtures::TEST235_MONOIDAL_PRODUCT,
        &mut report,
    );
    run_test(
        "test236_monoidal_unit",
        tests::fixtures::TEST236_MONOIDAL_UNIT,
        &mut report,
    );
    run_test(
        "test237_monoidal_associator",
        tests::fixtures::TEST237_MONOIDAL_ASSOCIATOR,
        &mut report,
    );
    run_test(
        "test238_structural_operad",
        tests::fixtures::TEST238_STRUCTURAL_OPERAD,
        &mut report,
    );
    run_test(
        "test239_operad_composition",
        tests::fixtures::TEST239_OPERAD_COMPOSITION,
        &mut report,
    );
    run_test(
        "test240_operad_end_to_end",
        tests::fixtures::TEST240_OPERAD_END_TO_END,
        &mut report,
    );
    run_test(
        "test241_associator_triple",
        tests::fixtures::TEST241_ASSOCIATOR_TRIPLE,
        &mut report,
    );
    run_test(
        "test242_mutual_model_trace",
        tests::fixtures::TEST242_MUTUAL_MODEL_TRACE,
        &mut report,
    );
    run_test(
        "test243_interaction_composition",
        tests::fixtures::TEST243_INTERACTION_COMPOSITION,
        &mut report,
    );
    run_test(
        "test244_sublease_transfer",
        tests::fixtures::TEST244_SUBLEASE_TRANSFER,
        &mut report,
    );
    run_test(
        "test245_predicate_subclasses",
        tests::fixtures::TEST245_PREDICATE_SUBCLASSES,
        &mut report,
    );
    run_test(
        "test246_phase_rotation",
        tests::fixtures::TEST246_PHASE_ROTATION,
        &mut report,
    );
    run_test(
        "test247_rollback_transition",
        tests::fixtures::TEST247_ROLLBACK_TRANSITION,
        &mut report,
    );
    run_test(
        "test248_epoch_boundary",
        tests::fixtures::TEST248_EPOCH_BOUNDARY,
        &mut report,
    );
    run_test(
        "test249_property_bind_advance",
        tests::fixtures::TEST249_PROPERTY_BIND_ADVANCE,
        &mut report,
    );
    run_test(
        "test250_pipeline_outcome",
        tests::fixtures::TEST250_PIPELINE_OUTCOME,
        &mut report,
    );
    run_test(
        "test251_preflight_checkpoint",
        tests::fixtures::TEST251_PREFLIGHT_CHECKPOINT,
        &mut report,
    );
    run_test(
        "test252_compile_unit",
        tests::fixtures::TEST252_COMPILE_UNIT,
        &mut report,
    );
    run_test(
        "test253_expression_types",
        tests::fixtures::TEST253_EXPRESSION_TYPES,
        &mut report,
    );
    run_test(
        "test254_proof_derivation_types",
        tests::fixtures::TEST254_PROOF_DERIVATION_TYPES,
        &mut report,
    );
    run_test(
        "test255_effect_types",
        tests::fixtures::TEST255_EFFECT_TYPES,
        &mut report,
    );
    run_test(
        "test256_predicate_types",
        tests::fixtures::TEST256_PREDICATE_TYPES,
        &mut report,
    );
    run_test(
        "test257_parallel_types",
        tests::fixtures::TEST257_PARALLEL_TYPES,
        &mut report,
    );
    run_test(
        "test258_stream_types",
        tests::fixtures::TEST258_STREAM_TYPES,
        &mut report,
    );
    run_test(
        "test259_failure_types",
        tests::fixtures::TEST259_FAILURE_TYPES,
        &mut report,
    );
    run_test(
        "test260_linear_types",
        tests::fixtures::TEST260_LINEAR_TYPES,
        &mut report,
    );
    run_test(
        "test261_recursion_types",
        tests::fixtures::TEST261_RECURSION_TYPES,
        &mut report,
    );
    run_test(
        "test262_region_types",
        tests::fixtures::TEST262_REGION_TYPES,
        &mut report,
    );
    run_test(
        "test263_boundary_types",
        tests::fixtures::TEST263_BOUNDARY_TYPES,
        &mut report,
    );
    run_test(
        "test264_conformance_types",
        tests::fixtures::TEST264_CONFORMANCE_TYPES,
        &mut report,
    );
    run_test(
        "test265_user_types",
        tests::fixtures::TEST265_USER_TYPES,
        &mut report,
    );
    run_test(
        "test266_witness_datum",
        tests::fixtures::TEST266_WITNESS_DATUM,
        &mut report,
    );
    run_test(
        "test267_compile_unit_builder",
        tests::fixtures::TEST267_COMPILE_UNIT_BUILDER,
        &mut report,
    );
    run_test(
        "test268_violation_kind",
        tests::fixtures::TEST268_VIOLATION_KIND,
        &mut report,
    );
    // Amendment 95: Workstreams 1, 3, 4, 5, 8
    run_test(
        "test269_predicate_individuals",
        tests::fixtures::TEST269_PREDICATE_INDIVIDUALS,
        &mut report,
    );
    run_test(
        "test270_constraint_subclasses",
        tests::fixtures::TEST270_CONSTRAINT_SUBCLASSES,
        &mut report,
    );
    run_test(
        "test271_host_value_types",
        tests::fixtures::TEST271_HOST_VALUE_TYPES,
        &mut report,
    );
    run_test(
        "test272_boundary_map_individuals",
        tests::fixtures::TEST272_BOUNDARY_MAP_INDIVIDUALS,
        &mut report,
    );
    run_test(
        "test273_type_definition_sum",
        tests::fixtures::TEST273_TYPE_DEFINITION_SUM,
        &mut report,
    );
    run_test(
        "test274_witness_types",
        tests::fixtures::TEST274_WITNESS_TYPES,
        &mut report,
    );
    run_test(
        "test275_reduction_advance",
        tests::fixtures::TEST275_REDUCTION_ADVANCE,
        &mut report,
    );
    run_test(
        "test276_witness_site_budget",
        tests::fixtures::TEST276_WITNESS_SITE_BUDGET,
        &mut report,
    );
    run_test(
        "test277_inhabitance_verdict",
        tests::fixtures::TEST277_INHABITANCE_VERDICT,
        &mut report,
    );
    run_test(
        "test278_v021_cleanup_bounds",
        tests::fixtures::TEST278_V021_CLEANUP_BOUNDS,
        &mut report,
    );
    run_test(
        "test279_multiplication_certificate",
        tests::fixtures::TEST279_MULTIPLICATION_CERTIFICATE,
        &mut report,
    );
    run_test(
        "test280_bridge_completion",
        tests::fixtures::TEST280_BRIDGE_COMPLETION,
        &mut report,
    );
    // Product/Coproduct Completion Amendment SHACL fixtures.
    run_test(
        "test285_cartesian_partition_product",
        tests::fixtures::TEST285_CARTESIAN_PARTITION_PRODUCT,
        &mut report,
    );
    run_test(
        "test286_tag_site",
        tests::fixtures::TEST286_TAG_SITE,
        &mut report,
    );
    run_test(
        "test287_layout_invariant",
        tests::fixtures::TEST287_LAYOUT_INVARIANT,
        &mut report,
    );
    // Wiki ADR-038: AxisProjectionObservable closed-catalog extension.
    run_test(
        "test288_axis_projection_observable",
        tests::fixtures::TEST288_AXIS_PROJECTION_OBSERVABLE,
        &mut report,
    );
    // Wiki ADR-049: SpectralObservable closed-catalog extension.
    run_test(
        "test289_spectral_observable",
        tests::fixtures::TEST289_SPECTRAL_OBSERVABLE,
        &mut report,
    );

    // Verify test fixture count matches expected
    let test_count = report.results.len() - before_tests;
    if test_count == counts::SHACL_TESTS {
        report.push(TestResult::pass(
            "ontology/shacl/test_count",
            format!("SHACL test count matches expected: {}", counts::SHACL_TESTS),
        ));
    } else {
        report.push(TestResult::fail(
            "ontology/shacl/test_count",
            format!(
                "Expected {} SHACL tests but ran {}",
                counts::SHACL_TESTS,
                test_count
            ),
        ));
    }

    report
}

/// Runs a single SHACL test against the provided Turtle instance graph.
fn run_test(name: &str, turtle_src: &str, report: &mut ConformanceReport) {
    let validator = format!("ontology/shacl/{}", name);

    // Structural validation: the Turtle must be non-empty and syntactically plausible
    if turtle_src.trim().is_empty() {
        report.push(TestResult::fail(
            validator.clone(),
            "Instance graph is empty",
        ));
        return;
    }

    if !turtle_src.contains("@prefix") {
        report.push(TestResult::fail(
            validator.clone(),
            "Instance graph missing @prefix declarations",
        ));
        return;
    }

    // Run test-specific structural checks
    let result = match name {
        "test1_ring" => validate_ring(turtle_src),
        "test2_primitives" => validate_primitives(turtle_src),
        "test3_term_graph" => validate_term_graph(turtle_src),
        "test4_state_lifecycle" => validate_state_lifecycle(turtle_src),
        "test5_partition" => validate_partition(turtle_src),
        "test6_critical_identity" => validate_critical_identity(turtle_src),
        "test7_end_to_end" => validate_end_to_end(turtle_src),
        "test8_free_rank" => validate_free_rank(turtle_src),
        "test9_constraint_algebra" => validate_constraint_algebra(turtle_src),
        "test10_iterative_resolution" => validate_iterative_resolution(turtle_src),
        "test11_composition" => validate_composition(turtle_src),
        "test12_factorization" => validate_factorization(turtle_src),
        "test13_canonical_form" => validate_canonical_form(turtle_src),
        "test14_content_addressing" => validate_content_addressing(turtle_src),
        "test15_boolean_sat" => validate_boolean_sat(turtle_src),
        "test16_algebraic_identities" => validate_algebraic_identities(turtle_src),
        "test17_inter_algebra_maps" => validate_inter_algebra_maps(turtle_src),
        "test18_analytical_completeness" => validate_analytical_completeness(turtle_src),
        "test19_homological_pipeline" => validate_homological_pipeline(turtle_src),
        "test20_sheaf_consistency" => validate_sheaf_consistency(turtle_src),
        "test21_topological_delta" => validate_topological_delta(turtle_src),
        "test22_index_bridge" => validate_index_bridge(turtle_src),
        "test23_identity_grounding" => validate_identity_grounding_shacl(turtle_src),
        "test24_verification_domain" => validate_verification_domain_shacl(turtle_src),
        "test25_geometric_character" => validate_geometric_character_shacl(turtle_src),
        "test26_complexity_class" => validate_complexity_class_shacl(turtle_src),
        "test27_rewrite_rule" => validate_rewrite_rule_shacl(turtle_src),
        "test28_measurement_unit" => validate_measurement_unit_shacl(turtle_src),
        "test29_triad_projection" => validate_coordinate_kind_shacl(turtle_src),
        "test30_proof_coverage" => validate_proof_coverage_shacl(turtle_src),
        "test31_witt_level" => validate_quantum_level_shacl(turtle_src),
        "test32_arc_grounding" => validate_arc_grounding_shacl(turtle_src),
        "test33_graph_gaps" => validate_graph_gaps_shacl(turtle_src),
        "test34_completeness_candidate" => validate_completeness_candidate_shacl(turtle_src),
        "test35_completeness_certificate" => validate_completeness_certificate_shacl(turtle_src),
        "test36_w16_ring" => validate_q1_ring_shacl(turtle_src),
        "test37_witt_level_binding" => validate_quantum_level_binding_shacl(turtle_src),
        "test38_session_lifecycle" => validate_session_lifecycle_shacl(turtle_src),
        "test39_session_boundary" => validate_session_boundary_shacl(turtle_src),
        "test40_type_synthesis_goal" => validate_type_synthesis_goal_shacl(turtle_src),
        "test41_synthesis_result" => validate_synthesis_result_shacl(turtle_src),
        "test42_witt_lift" => validate_quantum_lift_shacl(turtle_src),
        "test43_spectral_sequence" => validate_spectral_sequence_shacl(turtle_src),
        "test44_monodromy_flat" => validate_monodromy_flat_shacl(turtle_src),
        "test45_monodromy_twisted" => validate_monodromy_twisted_shacl(turtle_src),
        "test46_monodromy_pipeline" => validate_monodromy_pipeline_shacl(turtle_src),
        "test47_thermo_pipeline" => validate_thermo_pipeline_shacl(turtle_src),
        "test48_phase_diagram" => validate_phase_diagram_shacl(turtle_src),
        "test49_reversible_resolution" => validate_reversible_resolution_shacl(turtle_src),
        "test50_jacobian_resolver" => validate_jacobian_resolver_shacl(turtle_src),
        "test51_product_type_pipeline" => validate_product_type_pipeline_shacl(turtle_src),
        "test52_sum_type_variant" => validate_sum_type_variant_shacl(turtle_src),
        "test53_superposed_site" => validate_superposed_site_shacl(turtle_src),
        "test54_grounded_context" => validate_saturated_context_shacl(turtle_src),
        "test55_grounding_witness" => validate_saturation_witness_shacl(turtle_src),
        "test56_domain_grounding_record" => validate_domain_saturation_record_shacl(turtle_src),
        "test57_grounding_phase" => validate_saturation_phase_shacl(turtle_src),
        "test58_grounding_certificate" => validate_saturation_certificate_shacl(turtle_src),
        "test59_grounding_aware_resolver" => validate_saturation_aware_resolver_shacl(turtle_src),
        "test60_impossibility_witness" => validate_impossibility_witness_shacl(turtle_src),
        "test61_morphospace_record" => validate_morphospace_record_shacl(turtle_src),
        "test62_morphospace_boundary" => validate_morphospace_boundary_shacl(turtle_src),
        "test63_forbidden_signature" => validate_forbidden_signature_shacl(turtle_src),
        "test64_achievability_status" => validate_achievability_status_shacl(turtle_src),
        "test65_geodesic_trace" => validate_geodesic_trace_shacl(turtle_src),
        "test66_geodesic_certificate" => validate_geodesic_certificate_shacl(turtle_src),
        "test67_geodesic_violation" => validate_geodesic_violation_shacl(turtle_src),
        "test68_geodesic_validator" => validate_geodesic_validator_shacl(turtle_src),
        "test69_geodesic_ordered" => validate_geodesic_ordered_shacl(turtle_src),
        "test70_measurement_resolver" => validate_measurement_resolver_shacl(turtle_src),
        "test71_measurement_event" => validate_measurement_event_shacl(turtle_src),
        "test72_measurement_certificate" => validate_measurement_certificate_shacl(turtle_src),
        "test73_collapsed_site_state" => validate_collapsed_site_state_shacl(turtle_src),
        "test74_quantum_thermodynamic" => validate_quantum_thermodynamic_shacl(turtle_src),
        "test75_partition_product" => validate_basic_turtle(turtle_src),
        "test76_partition_coproduct" => validate_basic_turtle(turtle_src),
        "test77_geodesic_evidence" => validate_basic_turtle(turtle_src),
        "test78_born_rule" => validate_basic_turtle(turtle_src),
        "test79_measurement_outcome" => validate_basic_turtle(turtle_src),
        "test80_partition_exhaustive" => validate_basic_turtle(turtle_src),
        "test81_dihedral_algebra" => validate_basic_turtle(turtle_src),
        "test82_level_successor" => validate_basic_turtle(turtle_src),
        "test83_amplitude_normalization" => validate_basic_turtle(turtle_src),
        "test84_enum_variant" => validate_basic_turtle(turtle_src),
        "test85_w16_ring_grounding" => validate_basic_turtle(turtle_src),
        "test86_witt_lift_trivial" => validate_basic_turtle(turtle_src),
        "test87_spectral_convergence" => validate_basic_turtle(turtle_src),
        "test88_lift_obstruction_nontrivial" => validate_basic_turtle(turtle_src),
        "test89_lift_refinement_suggestion" => validate_basic_turtle(turtle_src),
        "test90_resolved_lift" => validate_basic_turtle(turtle_src),
        "test91_synthesis_goal_w16" => validate_basic_turtle(turtle_src),
        "test92_synthesis_checkpoint" => validate_basic_turtle(turtle_src),
        "test93_synthesis_signature" => validate_basic_turtle(turtle_src),
        "test94_synthesized_type" => validate_basic_turtle(turtle_src),
        "test95_unreachable_signature" => validate_basic_turtle(turtle_src),
        "test96_geodesic_trace_w16" => validate_basic_turtle(turtle_src),
        "test97_evidence_bundle_ar1" => validate_basic_turtle(turtle_src),
        "test98_evidence_bundle_dc10" => validate_basic_turtle(turtle_src),
        "test99_measurement_born_w16" => validate_basic_turtle(turtle_src),
        "test100_normative_chain" => validate_basic_turtle(turtle_src),
        "test101_lift_chain_flat" => validate_basic_turtle(turtle_src),
        "test102_lift_chain_twisted" => validate_basic_turtle(turtle_src),
        "test103_obstruction_chain_empty" => validate_basic_turtle(turtle_src),
        "test104_obstruction_chain_nontrivial" => validate_basic_turtle(turtle_src),
        "test105_lift_chain_certificate" => validate_basic_turtle(turtle_src),
        "test106_chain_audit_trail" => validate_basic_turtle(turtle_src),
        "test107_tower_resolver" => validate_basic_turtle(turtle_src),
        "test108_inductive_proof" => validate_basic_turtle(turtle_src),
        "test109_validity_scope" => validate_basic_turtle(turtle_src),
        "test110_tower_roundtrip" => validate_basic_turtle(turtle_src),
        "test111_address_crypto_pinning" => validate_basic_turtle(turtle_src),
        "test112_address_canonical_bytes" => validate_basic_turtle(turtle_src),
        "test113_carry_constraint_pinning" => validate_basic_turtle(turtle_src),
        "test114_joint_satisfiability" => validate_basic_turtle(turtle_src),
        "test115_dihedral_inverse_order" => validate_basic_turtle(turtle_src),
        "test116_constraint_expressiveness" => validate_basic_turtle(turtle_src),
        "test117_sumtype_topology" => validate_basic_turtle(turtle_src),
        "test118_synthesis_reachability" => validate_basic_turtle(turtle_src),
        "test119_obstruction_termination" => validate_basic_turtle(turtle_src),
        "test120_coefficient_ring" => validate_basic_turtle(turtle_src),
        "test121_gluing_feedback" => validate_basic_turtle(turtle_src),
        "test122_session_grounding" => validate_basic_turtle(turtle_src),
        "test123_amplitude_index" => validate_basic_turtle(turtle_src),
        // Amendment 46: SHACL fixture coverage gap closure
        "test124_glyph" => validate_basic_turtle(turtle_src),
        "test125_dihedral_group" => validate_basic_turtle(turtle_src),
        "test126_validity_scope_kind" => validate_basic_turtle(turtle_src),
        "test127_witt_level_resolver" => validate_basic_turtle(turtle_src),
        "test128_stratum_observable" => validate_basic_turtle(turtle_src),
        "test129_metric_observable" => validate_basic_turtle(turtle_src),
        "test130_path_observable" => validate_basic_turtle(turtle_src),
        "test131_reduction_observable" => validate_basic_turtle(turtle_src),
        "test132_holonomy_observable" => validate_basic_turtle(turtle_src),
        "test133_incompatibility_metric" => validate_basic_turtle(turtle_src),
        "test134_stratum_value" => validate_basic_turtle(turtle_src),
        "test135_stratum_delta" => validate_basic_turtle(turtle_src),
        "test136_stratum_trajectory" => validate_basic_turtle(turtle_src),
        "test137_path_length" => validate_basic_turtle(turtle_src),
        "test138_total_variation" => validate_basic_turtle(turtle_src),
        "test139_winding_number" => validate_basic_turtle(turtle_src),
        "test140_reduction_length" => validate_basic_turtle(turtle_src),
        "test141_reduction_count" => validate_basic_turtle(turtle_src),
        "test142_catastrophe_threshold" => validate_basic_turtle(turtle_src),
        "test143_catastrophe_count" => validate_basic_turtle(turtle_src),
        "test144_commutator" => validate_basic_turtle(turtle_src),
        "test145_curvature_flux" => validate_basic_turtle(turtle_src),
        "test146_parallel_transport" => validate_basic_turtle(turtle_src),
        "test147_reduction_entropy" => validate_basic_turtle(turtle_src),
        "test148_phase_boundary_type" => validate_basic_turtle(turtle_src),
        "test149_face_map" => validate_basic_turtle(turtle_src),
        "test150_nerve_functor" => validate_basic_turtle(turtle_src),
        "test151_chain_functor" => validate_basic_turtle(turtle_src),
        "test152_restriction_map" => validate_basic_turtle(turtle_src),
        "test153_coherence_proof" => validate_basic_turtle(turtle_src),
        "test154_derivation_step" => validate_basic_turtle(turtle_src),
        "test155_computation_step" => validate_basic_turtle(turtle_src),
        "test156_trace_metrics" => validate_basic_turtle(turtle_src),
        "test157_isometry_certificate" => validate_basic_turtle(turtle_src),
        "test158_lift_chain_certificate" => validate_basic_turtle(turtle_src),
        "test159_chain_audit_trail" => validate_basic_turtle(turtle_src),
        "test160_shared_context" => validate_basic_turtle(turtle_src),
        "test161_execution_policy" => validate_basic_turtle(turtle_src),
        "test162_session_composition" => validate_basic_turtle(turtle_src),
        "test163_distributed_grounding" => validate_basic_turtle(turtle_src),
        "test164_embedding" => validate_basic_turtle(turtle_src),
        "test165_action" => validate_basic_turtle(turtle_src),
        "test166_session_boundary_type" => validate_basic_turtle(turtle_src),
        "test167_metric_axis" => validate_basic_turtle(turtle_src),
        "test168_witt_carry" => validate_basic_turtle(turtle_src),
        "test169_arithmetic_valuation" => validate_basic_turtle(turtle_src),
        "test170_kan_complex" => validate_basic_turtle(turtle_src),
        "test171_postnikov_truncation" => validate_basic_turtle(turtle_src),
        "test172_homotopy_group" => validate_basic_turtle(turtle_src),
        "test173_homotopy_end_to_end" => {
            check_contains(turtle_src, "homology:KanComplex", "Missing KanComplex")
                .and_then(|()| {
                    check_contains(
                        turtle_src,
                        "homology:PostnikovTruncation",
                        "Missing PostnikovTruncation",
                    )
                })
                .and_then(|()| {
                    check_contains(
                        turtle_src,
                        "observable:HomotopyGroup",
                        "Missing HomotopyGroup",
                    )
                })
        }
        "test174_homotopy_resolver" => validate_basic_turtle(turtle_src),
        "test175_homotopy_pipeline" => validate_basic_turtle(turtle_src),
        "test176_moduli_space" => validate_basic_turtle(turtle_src),
        "test177_deformation_complex" => validate_basic_turtle(turtle_src),
        "test178_holonomy_stratum" => validate_basic_turtle(turtle_src),
        "test179_moduli_end_to_end" => {
            check_contains(turtle_src, "type:ModuliSpace", "Missing ModuliSpace")
                .and_then(|()| {
                    check_contains(
                        turtle_src,
                        "homology:DeformationComplex",
                        "Missing DeformationComplex",
                    )
                })
                .and_then(|()| {
                    check_contains(
                        turtle_src,
                        "type:HolonomyStratum",
                        "Missing HolonomyStratum",
                    )
                })
        }
        "test180_moduli_resolver" => validate_basic_turtle(turtle_src),
        "test181_stratification_record" => validate_basic_turtle(turtle_src),
        "test182_whitehead_product" => validate_basic_turtle(turtle_src),
        "test183_deformation_family" => validate_basic_turtle(turtle_src),
        "test184_versal_deformation" => validate_basic_turtle(turtle_src),
        "test185_carry_chain" => validate_basic_turtle(turtle_src),
        "test186_carry_event" => validate_basic_turtle(turtle_src),
        "test187_carry_profile" => validate_basic_turtle(turtle_src),
        "test188_encoding_configuration" => validate_basic_turtle(turtle_src),
        "test189_encoding_quality" => validate_basic_turtle(turtle_src),
        "test190_base_metric" => validate_basic_turtle(turtle_src),
        "test191_grounding_observable" => validate_basic_turtle(turtle_src),
        "test192_euler_characteristic" => validate_basic_turtle(turtle_src),
        "test193_galois_connection" => validate_basic_turtle(turtle_src),
        "test194_nerve_operations" => validate_basic_turtle(turtle_src),
        "test198_composed_operation" => validate_basic_turtle(turtle_src),
        "test199_dispatch_operation" => validate_basic_turtle(turtle_src),
        "test200_inference_operation" => validate_basic_turtle(turtle_src),
        "test201_accumulation_operation" => validate_basic_turtle(turtle_src),
        "test202_lease_partition_operation" => validate_basic_turtle(turtle_src),
        "test203_session_composition_operation" => validate_basic_turtle(turtle_src),
        "test204_euler_reduction" => validate_basic_turtle(turtle_src),
        "test205_reduction_step" => validate_basic_turtle(turtle_src),
        "test206_reduction_state" => validate_basic_turtle(turtle_src),
        "test207_phase_gate" => validate_basic_turtle(turtle_src),
        "test208_epoch" => validate_basic_turtle(turtle_src),
        "test209_predicate_expression" => validate_basic_turtle(turtle_src),
        "test210_guard_expression" => validate_basic_turtle(turtle_src),
        "test211_transition_effect" => validate_basic_turtle(turtle_src),
        "test212_service_window" => validate_basic_turtle(turtle_src),
        "test213_reduction_transaction" => validate_basic_turtle(turtle_src),
        "test214_feasibility_result" => validate_basic_turtle(turtle_src),
        "test215_lease_state" => validate_basic_turtle(turtle_src),
        "test216_managed_lease" => validate_basic_turtle(turtle_src),
        "test217_back_pressure" => validate_basic_turtle(turtle_src),
        "test218_deferred_query" => validate_basic_turtle(turtle_src),
        "test219_convergence_level" => validate_basic_turtle(turtle_src),
        "test220_hopf_fiber" => validate_basic_turtle(turtle_src),
        "test221_convergence_residual" => validate_basic_turtle(turtle_src),
        "test222_commutative_subspace" => validate_basic_turtle(turtle_src),
        "test223_associative_subalgebra" => validate_basic_turtle(turtle_src),
        "test224_normed_division_algebra" => validate_basic_turtle(turtle_src),
        "test225_cayley_dickson" => validate_basic_turtle(turtle_src),
        "test226_multiplication_table" => validate_basic_turtle(turtle_src),
        "test227_algebra_commutator" => validate_basic_turtle(turtle_src),
        "test228_algebra_associator" => validate_basic_turtle(turtle_src),
        "test229_interaction_context" => validate_basic_turtle(turtle_src),
        "test230_commutator_state" => validate_basic_turtle(turtle_src),
        "test231_associator_state" => validate_basic_turtle(turtle_src),
        "test232_three_way_site" => validate_basic_turtle(turtle_src),
        "test233_negotiation_trace" => validate_basic_turtle(turtle_src),
        "test234_interaction_nerve" => validate_basic_turtle(turtle_src),
        "test235_monoidal_product" => validate_basic_turtle(turtle_src),
        "test236_monoidal_unit" => validate_basic_turtle(turtle_src),
        "test237_monoidal_associator" => validate_basic_turtle(turtle_src),
        "test238_structural_operad" => validate_basic_turtle(turtle_src),
        "test239_operad_composition" => validate_basic_turtle(turtle_src),
        "test240_operad_end_to_end" => validate_basic_turtle(turtle_src),
        "test241_associator_triple" => validate_basic_turtle(turtle_src),
        "test242_mutual_model_trace" => validate_basic_turtle(turtle_src),
        "test243_interaction_composition" => validate_basic_turtle(turtle_src),
        "test244_sublease_transfer" => validate_basic_turtle(turtle_src),
        "test245_predicate_subclasses" => validate_basic_turtle(turtle_src),
        "test246_phase_rotation" => validate_basic_turtle(turtle_src),
        "test247_rollback_transition" => validate_basic_turtle(turtle_src),
        "test248_epoch_boundary" => validate_basic_turtle(turtle_src),
        "test249_property_bind_advance" => validate_basic_turtle(turtle_src),
        "test250_pipeline_outcome" => validate_basic_turtle(turtle_src),
        "test251_preflight_checkpoint" => validate_basic_turtle(turtle_src),
        "test252_compile_unit" => validate_basic_turtle(turtle_src),
        "test253_expression_types" => validate_basic_turtle(turtle_src),
        "test254_proof_derivation_types" => validate_basic_turtle(turtle_src),
        "test255_effect_types" => validate_basic_turtle(turtle_src),
        "test256_predicate_types" => validate_basic_turtle(turtle_src),
        "test257_parallel_types" => validate_basic_turtle(turtle_src),
        "test258_stream_types" => validate_basic_turtle(turtle_src),
        "test259_failure_types" => validate_basic_turtle(turtle_src),
        "test260_linear_types" => validate_basic_turtle(turtle_src),
        "test261_recursion_types" => validate_basic_turtle(turtle_src),
        "test262_region_types" => validate_basic_turtle(turtle_src),
        "test263_boundary_types" => validate_basic_turtle(turtle_src),
        "test264_conformance_types" => validate_basic_turtle(turtle_src),
        "test265_user_types" => validate_basic_turtle(turtle_src),
        "test266_witness_datum" => validate_basic_turtle(turtle_src),
        "test267_compile_unit_builder" => validate_basic_turtle(turtle_src),
        "test268_violation_kind" => validate_basic_turtle(turtle_src),
        "test269_predicate_individuals" => validate_basic_turtle(turtle_src),
        "test270_constraint_subclasses" => validate_basic_turtle(turtle_src),
        "test271_host_value_types" => validate_basic_turtle(turtle_src),
        "test272_boundary_map_individuals" => validate_basic_turtle(turtle_src),
        "test273_type_definition_sum" => validate_basic_turtle(turtle_src),
        "test274_witness_types" => validate_basic_turtle(turtle_src),
        "test275_reduction_advance" => validate_basic_turtle(turtle_src),
        "test276_witness_site_budget" => validate_basic_turtle(turtle_src),
        _ => Ok(()),
    };

    match result {
        Ok(()) => report.push(TestResult::pass(
            validator,
            format!("Instance graph {} passes SHACL structural validation", name),
        )),
        Err(msg) => report.push(TestResult::fail(validator, msg)),
    }
}

fn validate_ring(src: &str) -> Result<(), String> {
    check_contains(src, "schema:Ring", "Missing schema:Ring type assertion")?;
    check_contains(
        src,
        "schema:ringWittLength",
        "Missing schema:ringWittLength property",
    )?;
    check_contains(src, "schema:modulus", "Missing schema:modulus property")?;
    Ok(())
}

fn validate_primitives(src: &str) -> Result<(), String> {
    check_contains(src, "op:neg", "Missing op:neg individual reference")?;
    check_contains(src, "op:bnot", "Missing op:bnot individual reference")?;
    check_contains(src, "op:succ", "Missing op:succ individual reference")?;
    check_contains(src, "op:composedOf", "Missing op:composedOf property usage")?;
    Ok(())
}

fn validate_term_graph(src: &str) -> Result<(), String> {
    check_contains(src, "schema:Application", "Missing schema:Application")?;
    check_contains(src, "schema:Literal", "Missing schema:Literal")?;
    check_contains(src, "schema:denotes", "Missing schema:denotes property")?;
    Ok(())
}

fn validate_state_lifecycle(src: &str) -> Result<(), String> {
    check_contains(src, "state:Context", "Missing state:Context")?;
    check_contains(src, "state:Binding", "Missing state:Binding")?;
    check_contains(src, "state:Transition", "Missing state:Transition")?;
    Ok(())
}

fn validate_partition(src: &str) -> Result<(), String> {
    check_contains(src, "partition:Partition", "Missing partition:Partition")?;
    check_contains(src, "partition:cardinality", "Missing cardinality property")?;
    Ok(())
}

fn validate_critical_identity(src: &str) -> Result<(), String> {
    check_contains(src, "op:criticalIdentity", "Missing op:criticalIdentity")?;
    check_contains(
        src,
        "proof:CriticalIdentityProof",
        "Missing proof:CriticalIdentityProof",
    )?;
    check_contains(
        src,
        "proof:provesIdentity",
        "Missing proof:provesIdentity property",
    )?;
    Ok(())
}

fn validate_end_to_end(src: &str) -> Result<(), String> {
    check_contains(src, "state:Context", "Missing state:Context")?;
    check_contains(src, "type:", "Missing type: namespace usage")?;
    check_contains(src, "resolver:", "Missing resolver: namespace usage")?;
    check_contains(src, "partition:", "Missing partition: namespace usage")?;
    check_contains(src, "observable:", "Missing observable: namespace usage")?;
    check_contains(src, "cert:", "Missing cert: namespace usage")?;
    check_contains(src, "trace:", "Missing trace: namespace usage")?;
    Ok(())
}

fn validate_free_rank(src: &str) -> Result<(), String> {
    check_contains(src, "partition:FreeRank", "Missing partition:FreeRank")?;
    check_contains(src, "partition:SiteIndex", "Missing partition:SiteIndex")?;
    check_contains(src, "partition:isClosed", "Missing partition:isClosed")?;
    check_contains(
        src,
        "partition:pinnedCount",
        "Missing partition:pinnedCount",
    )?;
    check_contains(src, "partition:freeRank", "Missing partition:freeRank")?;
    check_contains(src, "partition:hasSite", "Missing partition:hasSite")?;
    check_contains(
        src,
        "partition:SiteBinding",
        "Missing partition:SiteBinding",
    )?;
    Ok(())
}

fn validate_constraint_algebra(src: &str) -> Result<(), String> {
    check_contains(
        src,
        "type:ResidueConstraint",
        "Missing type:ResidueConstraint",
    )?;
    check_contains(
        src,
        "type:CompositeConstraint",
        "Missing type:CompositeConstraint",
    )?;
    check_contains(src, "type:metricAxis", "Missing type:metricAxis")?;
    check_contains(src, "type:hasConstraint", "Missing type:hasConstraint")?;
    check_contains(src, "type:verticalAxis", "Missing type:verticalAxis")?;
    Ok(())
}

fn validate_iterative_resolution(src: &str) -> Result<(), String> {
    check_contains(
        src,
        "resolver:ResolutionState",
        "Missing resolver:ResolutionState",
    )?;
    check_contains(src, "resolver:isComplete", "Missing resolver:isComplete")?;
    check_contains(
        src,
        "resolver:RefinementSuggestion",
        "Missing resolver:RefinementSuggestion",
    )?;
    check_contains(
        src,
        "derivation:RefinementStep",
        "Missing derivation:RefinementStep",
    )?;
    check_contains(
        src,
        "resolver:convergenceRate",
        "Missing resolver:convergenceRate",
    )?;
    Ok(())
}

fn validate_composition(src: &str) -> Result<(), String> {
    check_contains(
        src,
        "morphism:Composition",
        "Missing morphism:Composition class",
    )?;
    check_contains(
        src,
        "morphism:CompositionLaw",
        "Missing morphism:CompositionLaw",
    )?;
    check_contains(
        src,
        "morphism:criticalComposition",
        "Missing morphism:criticalComposition",
    )?;
    check_contains(src, "morphism:Identity", "Missing morphism:Identity")?;
    check_contains(src, "morphism:identityOn", "Missing morphism:identityOn")?;
    Ok(())
}

fn validate_factorization(src: &str) -> Result<(), String> {
    check_contains(src, "query:", "Missing query: namespace usage")?;
    check_contains(
        src,
        "resolver:DihedralFactorizationResolver",
        "Missing DihedralFactorizationResolver",
    )?;
    check_contains(src, "partition:FreeRank", "Missing partition:FreeRank")?;
    check_contains(src, "cert:certifies", "Missing cert:certifies")?;
    check_contains(src, "trace:certifiedBy", "Missing trace:certifiedBy")?;
    Ok(())
}

fn validate_canonical_form(src: &str) -> Result<(), String> {
    check_contains(
        src,
        "resolver:CanonicalFormResolver",
        "Missing CanonicalFormResolver",
    )?;
    check_contains(
        src,
        "derivation:Derivation",
        "Missing derivation:Derivation",
    )?;
    check_contains(
        src,
        "derivation:RewriteStep",
        "Missing derivation:RewriteStep",
    )?;
    check_contains(
        src,
        "derivation:TermMetrics",
        "Missing derivation:TermMetrics",
    )?;
    check_contains(
        src,
        "derivation:originalTerm",
        "Missing derivation:originalTerm",
    )?;
    Ok(())
}

fn validate_content_addressing(src: &str) -> Result<(), String> {
    check_contains(src, "u:Element", "Missing u:Element")?;
    check_contains(
        src,
        "observable:RingMetric",
        "Missing observable:RingMetric",
    )?;
    check_contains(
        src,
        "observable:HammingMetric",
        "Missing observable:HammingMetric",
    )?;
    check_contains(
        src,
        "cert:InvolutionCertificate",
        "Missing cert:InvolutionCertificate",
    )?;
    check_contains(src, "cert:certifies", "Missing cert:certifies")?;
    Ok(())
}

fn validate_boolean_sat(src: &str) -> Result<(), String> {
    check_contains(
        src,
        "resolver:EvaluationResolver",
        "Missing EvaluationResolver",
    )?;
    check_contains(src, "state:Transition", "Missing state:Transition")?;
    check_contains(src, "state:Binding", "Missing state:Binding")?;
    check_contains(src, "cert:certifies", "Missing cert:certifies")?;
    check_contains(src, "trace:certifiedBy", "Missing trace:certifiedBy")?;
    Ok(())
}

fn validate_algebraic_identities(src: &str) -> Result<(), String> {
    check_contains(src, "op:Identity", "Missing op:Identity type assertion")?;
    check_contains(src, "op:lhs", "Missing op:lhs property")?;
    check_contains(src, "op:rhs", "Missing op:rhs property")?;
    check_contains(src, "op:forAll", "Missing op:forAll property")?;
    check_contains(src, "op:R_A1", "Missing ring identity R_A1")?;
    check_contains(src, "op:B_1", "Missing Boolean identity B_1")?;
    check_contains(src, "op:C_1", "Missing constraint identity C_1")?;
    check_contains(src, "op:F_1", "Missing site identity F_1")?;
    check_contains(src, "op:RE_1", "Missing resolution identity RE_1")?;
    check_contains(src, "op:OB_M1", "Missing observable identity OB_M1")?;
    check_contains(src, "op:T_C1", "Missing transform identity T_C1")?;
    check_contains(src, "op:AD_1", "Missing addressing identity AD_1")?;
    check_contains(src, "op:TH_1", "Missing thermodynamic identity TH_1")?;
    check_contains(src, "op:CA_1", "Missing carry identity CA_1")?;
    Ok(())
}

fn validate_inter_algebra_maps(src: &str) -> Result<(), String> {
    check_contains(src, "op:Identity", "Missing op:Identity type assertion")?;
    check_contains(src, "op:phi_1", "Missing phi_1 (Ring → Constraints)")?;
    check_contains(src, "op:phi_2", "Missing phi_2 (Constraints → Sites)")?;
    check_contains(src, "op:phi_3", "Missing phi_3 (Sites → Partition)")?;
    check_contains(src, "op:phi_4", "Missing phi_4 (Resolution Pipeline)")?;
    check_contains(src, "op:phi_5", "Missing phi_5 (Operations → Observables)")?;
    check_contains(src, "op:phi_6", "Missing phi_6 (Observables → Refinement)")?;
    Ok(())
}

fn validate_analytical_completeness(src: &str) -> Result<(), String> {
    check_contains(src, "obs:Jacobian", "Missing obs:Jacobian class")?;
    check_contains(
        src,
        "obs:TopologicalObservable",
        "Missing obs:TopologicalObservable",
    )?;
    check_contains(src, "obs:BettiNumber", "Missing obs:BettiNumber class")?;
    check_contains(src, "obs:SpectralGap", "Missing obs:SpectralGap class")?;
    check_contains(src, "resolver:CechNerve", "Missing resolver:CechNerve")?;
    check_contains(
        src,
        "op:DC_1",
        "Missing differential calculus identity DC_1",
    )?;
    check_contains(src, "op:HA_1", "Missing homological identity HA_1")?;
    check_contains(src, "op:IT_7a", "Missing index theorem identity IT_7a")?;
    check_contains(src, "op:IT_7d", "Missing completeness criterion IT_7d")?;
    Ok(())
}

fn validate_homological_pipeline(src: &str) -> Result<(), String> {
    check_contains(src, "homology:Simplex", "Missing homology:Simplex")?;
    check_contains(
        src,
        "homology:SimplicialComplex",
        "Missing homology:SimplicialComplex",
    )?;
    check_contains(src, "homology:ChainGroup", "Missing homology:ChainGroup")?;
    check_contains(
        src,
        "homology:BoundaryOperator",
        "Missing homology:BoundaryOperator",
    )?;
    check_contains(
        src,
        "homology:ChainComplex",
        "Missing homology:ChainComplex",
    )?;
    check_contains(
        src,
        "homology:HomologyGroup",
        "Missing homology:HomologyGroup",
    )?;
    Ok(())
}

fn validate_sheaf_consistency(src: &str) -> Result<(), String> {
    check_contains(src, "cohomology:Sheaf", "Missing cohomology:Sheaf")?;
    check_contains(src, "cohomology:Stalk", "Missing cohomology:Stalk")?;
    check_contains(src, "cohomology:Section", "Missing cohomology:Section")?;
    check_contains(
        src,
        "cohomology:GluingObstruction",
        "Missing cohomology:GluingObstruction",
    )?;
    check_contains(
        src,
        "cohomology:CochainComplex",
        "Missing cohomology:CochainComplex",
    )?;
    check_contains(
        src,
        "cohomology:CohomologyGroup",
        "Missing cohomology:CohomologyGroup",
    )?;
    Ok(())
}

fn validate_topological_delta(src: &str) -> Result<(), String> {
    check_contains(
        src,
        "morphism:TopologicalDelta",
        "Missing morphism:TopologicalDelta",
    )?;
    check_contains(
        src,
        "morphism:bettisBefore",
        "Missing morphism:bettisBefore",
    )?;
    check_contains(src, "morphism:bettisAfter", "Missing morphism:bettisAfter")?;
    check_contains(src, "morphism:eulerBefore", "Missing morphism:eulerBefore")?;
    check_contains(src, "morphism:eulerAfter", "Missing morphism:eulerAfter")?;
    check_contains(src, "morphism:nerveBefore", "Missing morphism:nerveBefore")?;
    check_contains(src, "morphism:nerveAfter", "Missing morphism:nerveAfter")?;
    Ok(())
}

fn validate_index_bridge(src: &str) -> Result<(), String> {
    check_contains(src, "op:Identity", "Missing op:Identity type assertion")?;
    check_contains(src, "op:phi_1", "Missing phi_1")?;
    check_contains(src, "op:phi_6", "Missing phi_6")?;
    check_contains(src, "op:psi_1", "Missing psi_1")?;
    check_contains(src, "op:psi_6", "Missing psi_6")?;
    check_contains(
        src,
        "op:verificationDomain",
        "Missing op:verificationDomain",
    )?;
    Ok(())
}

fn validate_identity_grounding_shacl(src: &str) -> Result<(), String> {
    check_contains(src, "op:R_A1", "Missing R_A1")?;
    check_contains(src, "op:C_1", "Missing C_1")?;
    check_contains(src, "op:F_1", "Missing F_1")?;
    check_contains(src, "op:DC_1", "Missing DC_1")?;
    check_contains(src, "op:psi_1", "Missing psi_1")?;
    check_contains(
        src,
        "op:verificationDomain",
        "Missing verificationDomain property",
    )?;
    check_contains(
        src,
        "op:verificationPathNote",
        "Missing verificationPathNote property",
    )?;
    Ok(())
}

fn validate_verification_domain_shacl(src: &str) -> Result<(), String> {
    check_contains(
        src,
        "op:VerificationDomain",
        "Missing VerificationDomain type",
    )?;
    check_contains(src, "op:verificationDomain", "Missing verificationDomain")?;
    check_contains(src, "op:Enumerative", "Missing Enumerative individual")?;
    Ok(())
}

fn validate_geometric_character_shacl(src: &str) -> Result<(), String> {
    check_contains(
        src,
        "op:GeometricCharacter",
        "Missing GeometricCharacter type",
    )?;
    check_contains(
        src,
        "op:hasGeometricCharacter",
        "Missing hasGeometricCharacter",
    )?;
    check_contains(
        src,
        "op:RingReflection",
        "Missing RingReflection individual",
    )?;
    check_contains(
        src,
        "op:HypercubeReflection",
        "Missing HypercubeReflection individual",
    )?;
    Ok(())
}

fn validate_complexity_class_shacl(src: &str) -> Result<(), String> {
    check_contains(
        src,
        "resolver:ComplexityClass",
        "Missing ComplexityClass type",
    )?;
    check_contains(
        src,
        "resolver:hasComplexityClass",
        "Missing hasComplexityClass",
    )?;
    check_contains(
        src,
        "resolver:ConstantTime",
        "Missing ConstantTime individual",
    )?;
    check_contains(src, "resolver:LinearTime", "Missing LinearTime individual")?;
    Ok(())
}

fn validate_rewrite_rule_shacl(src: &str) -> Result<(), String> {
    check_contains(src, "derivation:RewriteRule", "Missing RewriteRule type")?;
    check_contains(src, "derivation:hasRewriteRule", "Missing hasRewriteRule")?;
    check_contains(src, "derivation:groundedIn", "Missing groundedIn")?;
    check_contains(
        src,
        "derivation:CriticalIdentityRule",
        "Missing CriticalIdentityRule",
    )?;
    Ok(())
}

fn validate_measurement_unit_shacl(src: &str) -> Result<(), String> {
    check_contains(
        src,
        "observable:MeasurementUnit",
        "Missing MeasurementUnit type",
    )?;
    check_contains(src, "observable:hasUnit", "Missing hasUnit")?;
    check_contains(src, "observable:Bits", "Missing Bits individual")?;
    check_contains(src, "observable:RingSteps", "Missing RingSteps individual")?;
    Ok(())
}

fn validate_coordinate_kind_shacl(src: &str) -> Result<(), String> {
    check_contains(src, "query:TriadProjection", "Missing TriadProjection type")?;
    check_contains(
        src,
        "query:hasTriadProjection",
        "Missing hasTriadProjection",
    )?;
    check_contains(
        src,
        "query:TwoAdicValuation",
        "Missing TwoAdicValuation individual",
    )?;
    check_contains(
        src,
        "query:WalshHadamardImage",
        "Missing WalshHadamardImage individual",
    )?;
    Ok(())
}

fn validate_proof_coverage_shacl(src: &str) -> Result<(), String> {
    check_contains(
        src,
        "proof:ComputationCertificate",
        "Missing ComputationCertificate type",
    )?;
    check_contains(
        src,
        "proof:AxiomaticDerivation",
        "Missing AxiomaticDerivation type",
    )?;
    check_contains(
        src,
        "proof:CriticalIdentityProof",
        "Missing CriticalIdentityProof type",
    )?;
    check_contains(
        src,
        "proof:provesIdentity",
        "Missing provesIdentity property",
    )?;
    check_contains(src, "proof:atWittLevel", "Missing atWittLevel property")?;
    check_contains(
        src,
        "proof:universalScope",
        "Missing universalScope property",
    )?;
    check_contains(src, "proof:verified", "Missing verified property")?;
    check_contains(
        src,
        "op:MN_1",
        "Missing MN_1 identity reference (Amendment 30 proof coverage)",
    )?;
    check_contains(
        src,
        "op:MN_7",
        "Missing MN_7 identity reference (Amendment 30 proof coverage)",
    )?;
    Ok(())
}

fn validate_quantum_level_shacl(src: &str) -> Result<(), String> {
    check_contains(src, "schema:WittLevel", "Missing WittLevel type")?;
    check_contains(src, "schema:bitsWidth", "Missing bitsWidth property")?;
    check_contains(src, "schema:cycleSize", "Missing cycleSize property")?;
    check_contains(
        src,
        "schema:nextWittLevel",
        "Missing nextWittLevel property",
    )?;
    check_contains(src, "schema:W8", "Missing W8 individual")?;
    check_contains(src, "schema:W16", "Missing W16 individual")?;
    check_contains(src, "schema:W24", "Missing W24 individual")?;
    check_contains(src, "schema:W32", "Missing W32 individual")?;
    // W32 terminal condition: must NOT have nextWittLevel
    if let Some(w32_start) = src.find("schema:W32") {
        let w32_block = &src[w32_start..];
        let w32_end = w32_block.find("\n\n").unwrap_or(w32_block.len());
        let w32_section = &w32_block[..w32_end];
        if w32_section.contains("nextWittLevel") {
            return Err("W32 must not have nextWittLevel (terminal condition)".to_string());
        }
    }
    Ok(())
}

fn validate_arc_grounding_shacl(src: &str) -> Result<(), String> {
    check_contains(
        src,
        "morphism:GroundingMap",
        "Missing GroundingMap type assertion",
    )?;
    check_contains(
        src,
        "morphism:groundedAddress",
        "Missing groundedAddress property",
    )?;
    check_contains(
        src,
        "morphism:symbolConstraints",
        "Missing symbolConstraints property",
    )?;
    check_contains(
        src,
        "morphism:ProjectionMap",
        "Missing ProjectionMap type assertion",
    )?;
    check_contains(
        src,
        "morphism:projectionFrame",
        "Missing projectionFrame property",
    )?;
    check_contains(
        src,
        "morphism:projectionSource",
        "Missing projectionSource property",
    )?;
    check_contains(
        src,
        "morphism:roundTripCoherence",
        "Missing roundTripCoherence property",
    )?;
    check_contains(
        src,
        "query:RelationQuery",
        "Missing RelationQuery type assertion",
    )?;
    check_contains(src, "query:sourceAddress", "Missing sourceAddress property")?;
    check_contains(src, "query:relationType", "Missing relationType property")?;
    check_contains(src, "query:targetSite", "Missing targetSite property")?;
    check_contains(src, "query:groundingMap", "Missing groundingMap property")?;
    check_contains(src, "query:projectionMap", "Missing projectionMap property")?;
    Ok(())
}

fn validate_graph_gaps_shacl(src: &str) -> Result<(), String> {
    // Gap B
    check_contains(
        src,
        "resolver:nerveEulerCharacteristic",
        "Missing nerveEulerCharacteristic property",
    )?;
    check_contains(
        src,
        "cohomology:addressesSuggestion",
        "Missing addressesSuggestion property",
    )?;
    // Gap D
    check_contains(
        src,
        "type:CompleteType",
        "Missing CompleteType type assertion",
    )?;
    check_contains(
        src,
        "cert:CompletenessCertificate",
        "Missing CompletenessCertificate type assertion",
    )?;
    check_contains(src, "cert:certifiedType", "Missing certifiedType property")?;
    // Gap E
    check_contains(
        src,
        "observable:ThermoObservable",
        "Missing ThermoObservable type assertion",
    )?;
    check_contains(
        src,
        "observable:ResidualEntropy",
        "Missing ResidualEntropy type assertion",
    )?;
    check_contains(src, "observable:Nats", "Missing Nats individual")?;
    // Gap G
    check_contains(
        src,
        "morphism:GroundingCertificate",
        "Missing GroundingCertificate type assertion",
    )?;
    check_contains(
        src,
        "morphism:groundingCertMap",
        "Missing groundingCertMap property",
    )?;
    Ok(())
}

fn validate_completeness_candidate_shacl(src: &str) -> Result<(), String> {
    check_contains(
        src,
        "type:CompletenessCandidate",
        "Missing CompletenessCandidate type assertion",
    )?;
    check_contains(
        src,
        "type:completenessCandidate",
        "Missing completenessCandidate property",
    )?;
    check_contains(
        src,
        "type:candidateNerve",
        "Missing candidateNerve property",
    )?;
    check_contains(
        src,
        "resolver:CompletenessResolver",
        "Missing CompletenessResolver type assertion",
    )?;
    check_contains(
        src,
        "resolver:completenessTarget",
        "Missing completenessTarget property",
    )?;
    check_contains(
        src,
        "type:CompletenessWitness",
        "Missing CompletenessWitness type assertion",
    )?;
    check_contains(
        src,
        "type:witnessConstraint",
        "Missing witnessConstraint property",
    )?;
    check_contains(src, "type:sitesClosed", "Missing sitesClosed property")?;
    Ok(())
}

fn validate_completeness_certificate_shacl(src: &str) -> Result<(), String> {
    check_contains(
        src,
        "cert:CompletenessCertificate",
        "Missing CompletenessCertificate type assertion",
    )?;
    check_contains(src, "cert:auditTrail", "Missing auditTrail property")?;
    check_contains(
        src,
        "cert:CompletenessAuditTrail",
        "Missing CompletenessAuditTrail type assertion",
    )?;
    check_contains(src, "cert:witnessCount", "Missing witnessCount property")?;
    check_contains(
        src,
        "type:CompleteType",
        "Missing CompleteType type assertion",
    )?;
    check_contains(src, "cert:certifiedType", "Missing certifiedType property")?;
    Ok(())
}

fn validate_q1_ring_shacl(src: &str) -> Result<(), String> {
    check_contains(src, "schema:W16Ring", "Missing W16Ring type assertion")?;
    check_contains(src, "schema:W16bitWidth", "Missing W16bitWidth property")?;
    check_contains(src, "schema:W16capacity", "Missing W16capacity property")?;
    check_contains(
        src,
        "schema:nextWittLevel",
        "Missing nextWittLevel property",
    )?;
    Ok(())
}

fn validate_quantum_level_binding_shacl(src: &str) -> Result<(), String> {
    check_contains(
        src,
        "op:WittLevelBinding",
        "Missing WittLevelBinding type assertion",
    )?;
    check_contains(
        src,
        "op:universallyValid",
        "Missing universallyValid property",
    )?;
    check_contains(
        src,
        "op:verifiedAtLevel",
        "Missing verifiedAtLevel property",
    )?;
    check_contains(src, "op:bindingLevel", "Missing bindingLevel property")?;
    Ok(())
}

fn validate_session_lifecycle_shacl(src: &str) -> Result<(), String> {
    check_contains(src, "state:Session", "Missing Session type assertion")?;
    check_contains(
        src,
        "state:sessionBindings",
        "Missing sessionBindings property",
    )?;
    check_contains(
        src,
        "state:sessionQueries",
        "Missing sessionQueries property",
    )?;
    check_contains(
        src,
        "state:BindingAccumulator",
        "Missing BindingAccumulator type assertion",
    )?;
    check_contains(
        src,
        "state:accumulatedBindings",
        "Missing accumulatedBindings property",
    )?;
    check_contains(
        src,
        "resolver:SessionResolver",
        "Missing SessionResolver type assertion",
    )?;
    check_contains(
        src,
        "resolver:sessionAccumulator",
        "Missing sessionAccumulator property",
    )?;
    check_contains(
        src,
        "query:SessionQuery",
        "Missing SessionQuery type assertion",
    )?;
    check_contains(
        src,
        "query:sessionMembership",
        "Missing sessionMembership property",
    )?;
    Ok(())
}

fn validate_session_boundary_shacl(src: &str) -> Result<(), String> {
    check_contains(
        src,
        "state:SessionBoundary",
        "Missing SessionBoundary type assertion",
    )?;
    check_contains(src, "state:boundaryType", "Missing boundaryType property")?;
    check_contains(
        src,
        "state:ContradictionBoundary",
        "Missing ContradictionBoundary individual",
    )?;
    check_contains(
        src,
        "state:boundaryReason",
        "Missing boundaryReason property",
    )?;
    check_contains(src, "state:priorContext", "Missing priorContext property")?;
    check_contains(src, "state:freshContext", "Missing freshContext property")?;
    Ok(())
}

fn validate_type_synthesis_goal_shacl(src: &str) -> Result<(), String> {
    check_contains(
        src,
        "type:TypeSynthesisGoal",
        "Missing TypeSynthesisGoal type assertion",
    )?;
    check_contains(
        src,
        "type:targetEulerCharacteristic",
        "Missing targetEulerCharacteristic property",
    )?;
    check_contains(
        src,
        "resolver:TypeSynthesisResolver",
        "Missing TypeSynthesisResolver type assertion",
    )?;
    check_contains(
        src,
        "resolver:synthesisGoal",
        "Missing synthesisGoal property",
    )?;
    Ok(())
}

fn validate_synthesis_result_shacl(src: &str) -> Result<(), String> {
    check_contains(
        src,
        "type:TypeSynthesisResult",
        "Missing TypeSynthesisResult type assertion",
    )?;
    check_contains(
        src,
        "type:SynthesizedType",
        "Missing SynthesizedType type assertion",
    )?;
    check_contains(
        src,
        "type:MinimalConstraintBasis",
        "Missing MinimalConstraintBasis type assertion",
    )?;
    check_contains(src, "type:basisSize", "Missing basisSize property")?;
    check_contains(
        src,
        "observable:SynthesisSignature",
        "Missing SynthesisSignature type assertion",
    )?;
    check_contains(
        src,
        "observable:realisedEuler",
        "Missing realisedEuler property",
    )?;
    check_contains(
        src,
        "derivation:SynthesisStep",
        "Missing SynthesisStep type assertion",
    )?;
    check_contains(src, "derivation:stepIndex", "Missing stepIndex property")?;
    Ok(())
}

fn validate_quantum_lift_shacl(src: &str) -> Result<(), String> {
    check_contains(src, "type:WittLift", "Missing WittLift type assertion")?;
    check_contains(src, "type:liftBase", "Missing liftBase property")?;
    check_contains(
        src,
        "type:liftTargetLevel",
        "Missing liftTargetLevel property",
    )?;
    check_contains(
        src,
        "type:LiftObstruction",
        "Missing LiftObstruction type assertion",
    )?;
    check_contains(
        src,
        "type:obstructionTrivial",
        "Missing obstructionTrivial property",
    )?;
    check_contains(
        src,
        "resolver:IncrementalCompletenessResolver",
        "Missing IncrementalCompletenessResolver type assertion",
    )?;
    check_contains(src, "resolver:liftTarget", "Missing liftTarget property")?;
    Ok(())
}

fn validate_spectral_sequence_shacl(src: &str) -> Result<(), String> {
    check_contains(
        src,
        "observable:SpectralSequencePage",
        "Missing SpectralSequencePage type assertion",
    )?;
    check_contains(src, "observable:pageIndex", "Missing pageIndex property")?;
    check_contains(
        src,
        "observable:differentialIsZero",
        "Missing differentialIsZero property",
    )?;
    check_contains(
        src,
        "observable:convergedAt",
        "Missing convergedAt property",
    )?;
    Ok(())
}

fn validate_monodromy_flat_shacl(src: &str) -> Result<(), String> {
    check_contains(src, "type:FlatType", "Missing FlatType type assertion")?;
    check_contains(src, "type:holonomyGroup", "Missing holonomyGroup property")?;
    check_contains(
        src,
        "type:monodromyClass",
        "Missing monodromyClass property",
    )?;
    check_contains(
        src,
        "observable:HolonomyGroup",
        "Missing HolonomyGroup type assertion",
    )?;
    check_contains(
        src,
        "observable:holonomyGroupOrder",
        "Missing holonomyGroupOrder property",
    )?;
    check_contains(
        src,
        "observable:Monodromy",
        "Missing Monodromy type assertion",
    )?;
    check_contains(
        src,
        "observable:monodromyLoop",
        "Missing monodromyLoop property",
    )?;
    check_contains(
        src,
        "observable:monodromyElement",
        "Missing monodromyElement property",
    )?;
    check_contains(
        src,
        "observable:isTrivialMonodromy",
        "Missing isTrivialMonodromy property",
    )?;
    check_contains(
        src,
        "observable:DihedralElement",
        "Missing DihedralElement type assertion",
    )?;
    check_contains(
        src,
        "observable:ClosedConstraintPath",
        "Missing ClosedConstraintPath type assertion",
    )?;
    check_contains(
        src,
        "observable:MonodromyClass",
        "Missing MonodromyClass type assertion",
    )?;
    Ok(())
}

fn validate_monodromy_twisted_shacl(src: &str) -> Result<(), String> {
    check_contains(
        src,
        "type:TwistedType",
        "Missing TwistedType type assertion",
    )?;
    check_contains(src, "type:holonomyGroup", "Missing holonomyGroup property")?;
    check_contains(
        src,
        "type:monodromyClass",
        "Missing monodromyClass property",
    )?;
    check_contains(
        src,
        "observable:HolonomyGroup",
        "Missing HolonomyGroup type assertion",
    )?;
    check_contains(
        src,
        "observable:holonomyGroup",
        "Missing holonomyGroup generator property",
    )?;
    check_contains(
        src,
        "observable:holonomyGroupOrder",
        "Missing holonomyGroupOrder property",
    )?;
    check_contains(
        src,
        "observable:Monodromy",
        "Missing Monodromy type assertion",
    )?;
    check_contains(
        src,
        "observable:monodromyLoop",
        "Missing monodromyLoop property",
    )?;
    check_contains(
        src,
        "observable:monodromyElement",
        "Missing monodromyElement property",
    )?;
    check_contains(
        src,
        "observable:isTrivialMonodromy",
        "Missing isTrivialMonodromy property",
    )?;
    check_contains(
        src,
        "observable:DihedralElement",
        "Missing DihedralElement type assertion",
    )?;
    check_contains(
        src,
        "observable:isIdentityElement",
        "Missing isIdentityElement property",
    )?;
    check_contains(
        src,
        "observable:ClosedConstraintPath",
        "Missing ClosedConstraintPath type assertion",
    )?;
    check_contains(
        src,
        "type:LiftObstruction",
        "Missing LiftObstruction type assertion",
    )?;
    check_contains(
        src,
        "type:obstructionTrivial",
        "Missing obstructionTrivial property",
    )?;
    check_contains(
        src,
        "observable:LiftObstructionClass",
        "Missing LiftObstructionClass type assertion",
    )?;
    check_contains(
        src,
        "observable:MonodromyClass",
        "Missing MonodromyClass type assertion",
    )?;
    Ok(())
}

fn validate_monodromy_pipeline_shacl(src: &str) -> Result<(), String> {
    // Resolver
    check_contains(
        src,
        "resolver:MonodromyResolver",
        "Missing MonodromyResolver type assertion",
    )?;
    check_contains(
        src,
        "resolver:monodromyTarget",
        "Missing monodromyTarget property",
    )?;
    check_contains(
        src,
        "resolver:holonomyResult",
        "Missing holonomyResult property",
    )?;
    // Input type
    check_contains(
        src,
        "type:ConstrainedType",
        "Missing ConstrainedType type assertion",
    )?;
    check_contains(
        src,
        "type:holonomyGroup",
        "Missing holonomyGroup property on ConstrainedType",
    )?;
    check_contains(
        src,
        "type:monodromyClass",
        "Missing monodromyClass property on ConstrainedType",
    )?;
    // Observables
    check_contains(
        src,
        "observable:ClosedConstraintPath",
        "Missing ClosedConstraintPath type assertion",
    )?;
    check_contains(src, "observable:pathLength", "Missing pathLength property")?;
    check_contains(
        src,
        "observable:Monodromy",
        "Missing Monodromy type assertion",
    )?;
    check_contains(
        src,
        "observable:monodromyLoop",
        "Missing monodromyLoop property",
    )?;
    check_contains(
        src,
        "observable:monodromyElement",
        "Missing monodromyElement property",
    )?;
    check_contains(
        src,
        "observable:isTrivialMonodromy",
        "Missing isTrivialMonodromy property",
    )?;
    check_contains(
        src,
        "observable:DihedralElement",
        "Missing DihedralElement type assertion",
    )?;
    check_contains(
        src,
        "observable:isIdentityElement",
        "Missing isIdentityElement property",
    )?;
    check_contains(
        src,
        "observable:elementOrder",
        "Missing elementOrder property",
    )?;
    check_contains(
        src,
        "observable:HolonomyGroup",
        "Missing HolonomyGroup type assertion",
    )?;
    check_contains(
        src,
        "observable:holonomyGroup",
        "Missing holonomyGroup generator property",
    )?;
    check_contains(
        src,
        "observable:holonomyGroupOrder",
        "Missing holonomyGroupOrder property",
    )?;
    check_contains(
        src,
        "observable:MonodromyClass",
        "Missing MonodromyClass type assertion",
    )?;
    // Output classification
    check_contains(
        src,
        "type:TwistedType",
        "Missing TwistedType type assertion",
    )?;
    Ok(())
}

fn validate_thermo_pipeline_shacl(src: &str) -> Result<(), String> {
    check_contains(
        src,
        "observable:ThermoObservable",
        "Missing ThermoObservable",
    )?;
    check_contains(src, "observable:ResidualEntropy", "Missing ResidualEntropy")?;
    check_contains(
        src,
        "observable:hardnessEstimate",
        "Missing hardnessEstimate",
    )?;
    check_contains(src, "trace:ComputationTrace", "Missing ComputationTrace")?;
    check_contains(src, "trace:residualEntropy", "Missing residualEntropy link")?;
    Ok(())
}

fn validate_phase_diagram_shacl(src: &str) -> Result<(), String> {
    check_contains(
        src,
        "observable:CatastropheObservable",
        "Missing CatastropheObservable",
    )?;
    check_contains(src, "observable:phaseN", "Missing phaseN coordinate")?;
    check_contains(src, "observable:phaseG", "Missing phaseG coordinate")?;
    check_contains(
        src,
        "observable:phaseBoundaryType",
        "Missing phaseBoundaryType",
    )?;
    check_contains(src, "observable:onResonanceLine", "Missing onResonanceLine")?;
    Ok(())
}

fn validate_reversible_resolution_shacl(src: &str) -> Result<(), String> {
    check_contains(src, "partition:FreeRank", "Missing FreeRank")?;
    check_contains(src, "partition:SiteIndex", "Missing SiteIndex")?;
    check_contains(src, "partition:ancillaSite", "Missing ancillaSite")?;
    check_contains(
        src,
        "partition:reversibleStrategy",
        "Missing reversibleStrategy",
    )?;
    Ok(())
}

fn validate_jacobian_resolver_shacl(src: &str) -> Result<(), String> {
    check_contains(
        src,
        "resolver:JacobianGuidedResolver",
        "Missing JacobianGuidedResolver",
    )?;
    check_contains(src, "resolver:guidingJacobian", "Missing guidingJacobian")?;
    check_contains(src, "observable:Jacobian", "Missing Jacobian observable")?;
    Ok(())
}

fn validate_product_type_pipeline_shacl(src: &str) -> Result<(), String> {
    check_contains(src, "type:ProductType", "Missing ProductType")?;
    check_contains(src, "type:component", "Missing component property")?;
    check_contains(src, "type:TypeDefinition", "Missing TypeDefinition")?;
    Ok(())
}

fn validate_sum_type_variant_shacl(src: &str) -> Result<(), String> {
    check_contains(src, "type:SumType", "Missing SumType")?;
    check_contains(src, "type:component", "Missing component property")?;
    Ok(())
}

fn validate_superposed_site_shacl(src: &str) -> Result<(), String> {
    check_contains(
        src,
        "type:SuperposedSiteState",
        "Missing SuperposedSiteState",
    )?;
    check_contains(src, "type:amplitude", "Missing amplitude property")?;
    check_contains(
        src,
        "resolver:SuperpositionResolver",
        "Missing SuperpositionResolver",
    )?;
    Ok(())
}

fn validate_saturated_context_shacl(src: &str) -> Result<(), String> {
    check_contains(
        src,
        "state:GroundedContext",
        "Missing GroundedContext type assertion",
    )?;
    check_contains(
        src,
        "state:groundingDegree",
        "Missing groundingDegree property",
    )?;
    check_contains(
        src,
        "state:contextTemperature",
        "Missing contextTemperature property",
    )?;
    check_contains(src, "state:isGrounded", "Missing isGrounded property")?;
    check_contains(
        src,
        "state:groundingPhase",
        "Missing groundingPhase property",
    )?;
    Ok(())
}

fn validate_saturation_witness_shacl(src: &str) -> Result<(), String> {
    check_contains(
        src,
        "state:GroundingWitness",
        "Missing GroundingWitness type assertion",
    )?;
    check_contains(
        src,
        "state:witnessBinding",
        "Missing witnessBinding property",
    )?;
    check_contains(src, "state:witnessStep", "Missing witnessStep property")?;
    check_contains(
        src,
        "state:residualFreeRank",
        "Missing residualFreeRank property",
    )?;
    Ok(())
}

fn validate_domain_saturation_record_shacl(src: &str) -> Result<(), String> {
    check_contains(
        src,
        "observable:DomainGroundingRecord",
        "Missing DomainGroundingRecord type assertion",
    )?;
    check_contains(
        src,
        "observable:groundedContext",
        "Missing groundedContext property",
    )?;
    check_contains(
        src,
        "observable:groundedDomain",
        "Missing groundedDomain property",
    )?;
    check_contains(
        src,
        "observable:domainFreeRank",
        "Missing domainFreeRank property",
    )?;
    Ok(())
}

fn validate_saturation_phase_shacl(src: &str) -> Result<(), String> {
    check_contains(
        src,
        "state:GroundingPhase",
        "Missing GroundingPhase type assertion",
    )?;
    check_contains(src, "state:Open", "Missing Open individual")?;
    check_contains(
        src,
        "state:PartialGrounding",
        "Missing PartialGrounding individual",
    )?;
    check_contains(
        src,
        "state:FullGrounding",
        "Missing FullGrounding individual",
    )?;
    Ok(())
}

fn validate_saturation_certificate_shacl(src: &str) -> Result<(), String> {
    check_contains(
        src,
        "cert:GroundingCertificate",
        "Missing GroundingCertificate type assertion",
    )?;
    check_contains(
        src,
        "cert:certifiedGrounding",
        "Missing certifiedGrounding property",
    )?;
    check_contains(
        src,
        "cert:groundingWitness",
        "Missing groundingWitness property",
    )?;
    Ok(())
}

fn validate_saturation_aware_resolver_shacl(src: &str) -> Result<(), String> {
    check_contains(
        src,
        "resolver:GroundingAwareResolver",
        "Missing GroundingAwareResolver type assertion",
    )?;
    check_contains(
        src,
        "resolver:usedGrounding",
        "Missing usedGrounding property",
    )?;
    Ok(())
}

fn validate_impossibility_witness_shacl(src: &str) -> Result<(), String> {
    check_contains(
        src,
        "proof:ImpossibilityWitness",
        "Missing ImpossibilityWitness type assertion",
    )?;
    check_contains(
        src,
        "proof:forbidsSignature",
        "Missing forbidsSignature property",
    )?;
    check_contains(
        src,
        "proof:impossibilityReason",
        "Missing impossibilityReason property",
    )?;
    check_contains(
        src,
        "proof:impossibilityDomain",
        "Missing impossibilityDomain property",
    )?;
    Ok(())
}

fn validate_morphospace_record_shacl(src: &str) -> Result<(), String> {
    check_contains(
        src,
        "observable:MorphospaceRecord",
        "Missing MorphospaceRecord type assertion",
    )?;
    check_contains(
        src,
        "observable:achievabilityStatus",
        "Missing achievabilityStatus property",
    )?;
    check_contains(
        src,
        "observable:verifiedAtLevel",
        "Missing verifiedAtLevel property",
    )?;
    check_contains(
        src,
        "observable:morphospaceRecord",
        "Missing morphospaceRecord property",
    )?;
    Ok(())
}

fn validate_morphospace_boundary_shacl(src: &str) -> Result<(), String> {
    check_contains(
        src,
        "observable:MorphospaceBoundary",
        "Missing MorphospaceBoundary type assertion",
    )?;
    check_contains(
        src,
        "observable:boundaryType",
        "Missing boundaryType property",
    )?;
    Ok(())
}

fn validate_forbidden_signature_shacl(src: &str) -> Result<(), String> {
    check_contains(
        src,
        "observable:ForbiddenSignature",
        "Missing ForbiddenSignature type assertion",
    )?;
    check_contains(
        src,
        "observable:targetForbidden",
        "Missing targetForbidden property",
    )?;
    Ok(())
}

fn validate_achievability_status_shacl(src: &str) -> Result<(), String> {
    check_contains(
        src,
        "observable:AchievabilityStatus",
        "Missing AchievabilityStatus type assertion",
    )?;
    check_contains(
        src,
        "observable:Achievable",
        "Missing Achievable individual",
    )?;
    check_contains(src, "observable:Forbidden", "Missing Forbidden individual")?;
    Ok(())
}

fn validate_geodesic_trace_shacl(src: &str) -> Result<(), String> {
    check_contains(
        src,
        "trace:GeodesicTrace",
        "Missing GeodesicTrace type assertion",
    )?;
    check_contains(src, "trace:isGeodesic", "Missing isGeodesic property")?;
    check_contains(
        src,
        "trace:geodesicCertificate",
        "Missing geodesicCertificate property",
    )?;
    check_contains(
        src,
        "trace:stepEntropyCost",
        "Missing stepEntropyCost property",
    )?;
    check_contains(
        src,
        "trace:cumulativeEntropyCost",
        "Missing cumulativeEntropyCost property",
    )?;
    Ok(())
}

fn validate_geodesic_certificate_shacl(src: &str) -> Result<(), String> {
    check_contains(
        src,
        "cert:GeodesicCertificate",
        "Missing GeodesicCertificate type assertion",
    )?;
    check_contains(
        src,
        "cert:certifiedGeodesic",
        "Missing certifiedGeodesic property",
    )?;
    check_contains(src, "cert:geodesicTrace", "Missing geodesicTrace property")?;
    Ok(())
}

fn validate_geodesic_violation_shacl(src: &str) -> Result<(), String> {
    check_contains(
        src,
        "observable:GeodesicViolation",
        "Missing GeodesicViolation type assertion",
    )?;
    check_contains(
        src,
        "observable:violationReason",
        "Missing violationReason property",
    )?;
    Ok(())
}

fn validate_geodesic_validator_shacl(src: &str) -> Result<(), String> {
    check_contains(
        src,
        "resolver:GeodesicValidator",
        "Missing GeodesicValidator type assertion",
    )?;
    check_contains(
        src,
        "resolver:validateGeodesic",
        "Missing validateGeodesic property",
    )?;
    Ok(())
}

fn validate_geodesic_ordered_shacl(src: &str) -> Result<(), String> {
    check_contains(
        src,
        "trace:GeodesicTrace",
        "Missing GeodesicTrace type assertion",
    )?;
    check_contains(
        src,
        "trace:adiabaticallyOrdered",
        "Missing adiabaticallyOrdered property",
    )?;
    check_contains(
        src,
        "trace:jacobianAtStep",
        "Missing jacobianAtStep property",
    )?;
    Ok(())
}

fn validate_measurement_resolver_shacl(src: &str) -> Result<(), String> {
    check_contains(
        src,
        "resolver:MeasurementResolver",
        "Missing MeasurementResolver type assertion",
    )?;
    check_contains(
        src,
        "resolver:collapseAmplitude",
        "Missing collapseAmplitude property",
    )?;
    check_contains(
        src,
        "resolver:collapsedSite",
        "Missing collapsedSite property",
    )?;
    check_contains(
        src,
        "resolver:measurementOutcome",
        "Missing measurementOutcome property",
    )?;
    Ok(())
}

fn validate_measurement_event_shacl(src: &str) -> Result<(), String> {
    check_contains(
        src,
        "observable:MeasurementEvent",
        "Missing MeasurementEvent type assertion",
    )?;
    check_contains(
        src,
        "observable:measurementEvent",
        "Missing measurementEvent property",
    )?;
    check_contains(
        src,
        "observable:preCollapseEntropy",
        "Missing preCollapseEntropy property",
    )?;
    check_contains(
        src,
        "observable:postCollapseLandauerCost",
        "Missing postCollapseLandauerCost property",
    )?;
    check_contains(
        src,
        "observable:collapseStep",
        "Missing collapseStep property",
    )?;
    Ok(())
}

fn validate_measurement_certificate_shacl(src: &str) -> Result<(), String> {
    check_contains(
        src,
        "cert:MeasurementCertificate",
        "Missing MeasurementCertificate type assertion",
    )?;
    check_contains(
        src,
        "cert:certifiedMeasurement",
        "Missing certifiedMeasurement property",
    )?;
    check_contains(
        src,
        "cert:vonNeumannEntropy",
        "Missing vonNeumannEntropy property",
    )?;
    check_contains(src, "cert:landauerCost", "Missing landauerCost property")?;
    Ok(())
}

fn validate_collapsed_site_state_shacl(src: &str) -> Result<(), String> {
    check_contains(
        src,
        "type:CollapsedSiteState",
        "Missing CollapsedSiteState type assertion",
    )?;
    check_contains(src, "type:collapsedFrom", "Missing collapsedFrom property")?;
    check_contains(
        src,
        "type:survivingAmplitude",
        "Missing survivingAmplitude property",
    )?;
    Ok(())
}

fn validate_quantum_thermodynamic_shacl(src: &str) -> Result<(), String> {
    check_contains(
        src,
        "observable:QuantumThermodynamicDomain",
        "Missing QuantumThermodynamicDomain type assertion",
    )?;
    check_contains(
        src,
        "op:verificationDomain",
        "Missing verificationDomain property",
    )?;
    Ok(())
}

fn check_contains(src: &str, needle: &str, msg: &str) -> Result<(), String> {
    if src.contains(needle) {
        Ok(())
    } else {
        Err(msg.to_string())
    }
}

/// Validates that a Turtle test fixture is syntactically non-empty and
/// contains at least one `owl:NamedIndividual` type assertion.
fn validate_basic_turtle(src: &str) -> Result<(), String> {
    check_contains(
        src,
        "owl:NamedIndividual",
        "Missing owl:NamedIndividual type assertion",
    )
}
