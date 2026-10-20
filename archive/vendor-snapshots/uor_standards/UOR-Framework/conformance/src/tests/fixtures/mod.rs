//! Instance graph test fixtures for SHACL conformance validation.
//!
//! Each constant holds a Turtle 1.1 RDF graph that represents a valid
//! instance of UOR ontology terms, used to verify SHACL shape constraints.

mod test100_normative_chain;
mod test101_lift_chain_flat;
mod test102_lift_chain_twisted;
mod test103_obstruction_chain_empty;
mod test104_obstruction_chain_nontrivial;
mod test105_lift_chain_certificate;
mod test106_chain_audit_trail;
mod test107_tower_resolver;
mod test108_inductive_proof;
mod test109_validity_scope;
mod test10_iterative_resolution;
mod test110_tower_roundtrip;
mod test111_address_crypto_pinning;
mod test112_address_canonical_bytes;
mod test113_carry_constraint_pinning;
mod test114_joint_satisfiability;
mod test115_dihedral_inverse_order;
mod test116_constraint_expressiveness;
mod test117_sumtype_topology;
mod test118_synthesis_reachability;
mod test119_obstruction_termination;
mod test11_composition;
mod test120_coefficient_ring;
mod test121_gluing_feedback;
mod test122_session_grounding;
mod test123_amplitude_index;
mod test124_glyph;
mod test125_dihedral_group;
mod test126_validity_scope_kind;
mod test127_witt_level_resolver;
mod test128_stratum_observable;
mod test129_metric_observable;
mod test12_factorization;
mod test130_path_observable;
mod test131_reduction_observable;
mod test132_holonomy_observable;
mod test133_incompatibility_metric;
mod test134_stratum_value;
mod test135_stratum_delta;
mod test136_stratum_trajectory;
mod test137_path_length;
mod test138_total_variation;
mod test139_winding_number;
mod test13_canonical_form;
mod test140_reduction_length;
mod test141_reduction_count;
mod test142_catastrophe_threshold;
mod test143_catastrophe_count;
mod test144_commutator;
mod test145_curvature_flux;
mod test146_parallel_transport;
mod test147_reduction_entropy;
mod test148_phase_boundary_type;
mod test149_face_map;
mod test14_content_addressing;
mod test150_nerve_functor;
mod test151_chain_functor;
mod test152_restriction_map;
mod test153_coherence_proof;
mod test154_derivation_step;
mod test155_computation_step;
mod test156_trace_metrics;
mod test157_isometry_certificate;
mod test158_lift_chain_certificate;
mod test159_chain_audit_trail;
mod test15_boolean_sat;
mod test160_shared_context;
mod test161_execution_policy;
mod test162_session_composition;
mod test163_distributed_grounding;
mod test164_embedding;
mod test165_action;
mod test166_session_boundary_type;
mod test167_metric_axis;
mod test168_witt_carry;
mod test169_arithmetic_valuation;
mod test16_algebraic_identities;
mod test170_kan_complex;
mod test171_postnikov_truncation;
mod test172_homotopy_group;
mod test173_homotopy_end_to_end;
mod test174_homotopy_resolver;
mod test175_homotopy_pipeline;
mod test176_moduli_space;
mod test177_deformation_complex;
mod test178_holonomy_stratum;
mod test179_moduli_end_to_end;
mod test17_inter_algebra_maps;
mod test180_moduli_resolver;
mod test181_stratification_record;
mod test182_whitehead_product;
mod test183_deformation_family;
mod test184_versal_deformation;
mod test185_carry_chain;
mod test186_carry_event;
mod test187_carry_profile;
mod test188_encoding_configuration;
mod test189_encoding_quality;
mod test18_analytical_completeness;
mod test190_base_metric;
mod test191_grounding_observable;
mod test192_euler_characteristic;
mod test193_galois_connection;
mod test194_nerve_operations;
mod test195_scalar_symbol_type;
mod test196_sequence_tuple_type;
mod test197_graph_tree_type;
mod test198_composed_operation;
mod test199_dispatch_operation;
mod test19_homological_pipeline;
mod test1_ring;
mod test200_inference_operation;
mod test201_accumulation_operation;
mod test202_lease_partition_operation;
mod test203_session_composition_operation;
mod test204_euler_reduction;
mod test205_reduction_step;
mod test206_reduction_state;
mod test207_phase_gate;
mod test208_epoch;
mod test209_predicate_expression;
mod test20_sheaf_consistency;
mod test210_guard_expression;
mod test211_transition_effect;
mod test212_service_window;
mod test213_reduction_transaction;
mod test214_feasibility_result;
mod test215_lease_state;
mod test216_managed_lease;
mod test217_back_pressure;
mod test218_deferred_query;
mod test219_convergence_level;
mod test21_topological_delta;
mod test220_hopf_fiber;
mod test221_convergence_residual;
mod test222_commutative_subspace;
mod test223_associative_subalgebra;
mod test224_normed_division_algebra;
mod test225_cayley_dickson;
mod test226_multiplication_table;
mod test227_algebra_commutator;
mod test228_algebra_associator;
mod test229_interaction_context;
mod test22_index_bridge;
mod test230_commutator_state;
mod test231_associator_state;
mod test232_three_way_site;
mod test233_negotiation_trace;
mod test234_interaction_nerve;
mod test235_monoidal_product;
mod test236_monoidal_unit;
mod test237_monoidal_associator;
mod test238_structural_operad;
mod test239_operad_composition;
mod test23_identity_grounding;
mod test240_operad_end_to_end;
mod test241_associator_triple;
mod test242_mutual_model_trace;
mod test243_interaction_composition;
mod test244_sublease_transfer;
mod test245_predicate_subclasses;
mod test246_phase_rotation;
mod test247_rollback_transition;
mod test248_epoch_boundary;
mod test249_property_bind_advance;
mod test24_verification_domain;
mod test250_pipeline_outcome;
mod test251_preflight_checkpoint;
mod test252_compile_unit;
mod test253_expression_types;
mod test254_proof_derivation_types;
mod test255_effect_types;
mod test256_predicate_types;
mod test257_parallel_types;
mod test258_stream_types;
mod test259_failure_types;
mod test25_geometric_character;
mod test260_linear_types;
mod test261_recursion_types;
mod test262_region_types;
mod test263_boundary_types;
mod test264_conformance_types;
mod test265_user_types;
mod test266_witness_datum;
mod test267_compile_unit_builder;
mod test268_violation_kind;
mod test269_predicate_individuals;
mod test26_complexity_class;
mod test270_constraint_subclasses;
mod test271_host_value_types;
mod test272_boundary_map_individuals;
mod test273_type_definition_sum;
mod test274_witness_types;
mod test275_reduction_advance;
mod test276_witness_site_budget;
mod test277_inhabitance_verdict;
mod test278_v021_cleanup_bounds;
mod test279_multiplication_certificate;
mod test27_rewrite_rule;
mod test280_bridge_completion;
// Product/Coproduct Completion Amendment — fixtures for the three new
// kernel/bridge classes added by the amendment. Numbers 281–284 were
// already taken (test281_impossibility_certificates pre-existed); these
// land at 285/286/287, the next free range.
mod test281_impossibility_certificates;
mod test285_cartesian_partition_product;
mod test286_tag_site;
mod test287_layout_invariant;
mod test288_axis_projection_observable;
mod test289_spectral_observable;
mod test28_measurement_unit;
mod test290_value_threshold_observable;
mod test29_triad_projection;
mod test2_primitives;
mod test30_proof_coverage;
mod test31_witt_level;
mod test32_arc_grounding;
mod test33_graph_gaps;
mod test34_completeness_candidate;
mod test35_completeness_certificate;
mod test36_w16_ring;
mod test37_witt_level_binding;
mod test38_session_lifecycle;
mod test39_session_boundary;
mod test3_term_graph;
mod test40_type_synthesis_goal;
mod test41_synthesis_result;
mod test42_witt_lift;
mod test43_spectral_sequence;
mod test44_monodromy_flat;
mod test45_monodromy_twisted;
mod test46_monodromy_pipeline;
mod test47_thermo_pipeline;
mod test48_phase_diagram;
mod test49_reversible_resolution;
mod test4_state_lifecycle;
mod test50_jacobian_resolver;
mod test51_product_type_pipeline;
mod test52_sum_type_variant;
mod test53_superposed_site;
mod test54_grounded_context;
mod test55_grounding_witness;
mod test56_domain_grounding_record;
mod test57_grounding_phase;
mod test58_grounding_certificate;
mod test59_grounding_aware_resolver;
mod test5_partition;
mod test60_impossibility_witness;
mod test61_morphospace_record;
mod test62_morphospace_boundary;
mod test63_forbidden_signature;
mod test64_achievability_status;
mod test65_geodesic_trace;
mod test66_geodesic_certificate;
mod test67_geodesic_violation;
mod test68_geodesic_validator;
mod test69_geodesic_ordered;
mod test6_critical_identity;
mod test70_measurement_resolver;
mod test71_measurement_event;
mod test72_measurement_certificate;
mod test73_collapsed_site_state;
mod test74_quantum_thermodynamic;
mod test75_partition_product;
mod test76_partition_coproduct;
mod test77_geodesic_evidence;
mod test78_born_rule;
mod test79_measurement_outcome;
mod test7_end_to_end;
mod test80_partition_exhaustive;
mod test81_dihedral_algebra;
mod test82_level_successor;
mod test83_amplitude_normalization;
mod test84_enum_variant;
mod test85_w16_ring_grounding;
mod test86_witt_lift_trivial;
mod test87_spectral_convergence;
mod test88_lift_obstruction_nontrivial;
mod test89_lift_refinement_suggestion;
mod test8_free_rank;
mod test90_resolved_lift;
mod test91_synthesis_goal_w16;
mod test92_synthesis_checkpoint;
mod test93_synthesis_signature;
mod test94_synthesized_type;
mod test95_unreachable_signature;
mod test96_geodesic_trace_w16;
mod test97_evidence_bundle_ar1;
mod test98_evidence_bundle_dc10;
mod test99_measurement_born_w16;
mod test9_constraint_algebra;

pub use test100_normative_chain::TEST100_NORMATIVE_CHAIN;
pub use test101_lift_chain_flat::TEST101_LIFT_CHAIN_FLAT;
pub use test102_lift_chain_twisted::TEST102_LIFT_CHAIN_TWISTED;
pub use test103_obstruction_chain_empty::TEST103_OBSTRUCTION_CHAIN_EMPTY;
pub use test104_obstruction_chain_nontrivial::TEST104_OBSTRUCTION_CHAIN_NONTRIVIAL;
pub use test105_lift_chain_certificate::TEST105_LIFT_CHAIN_CERTIFICATE;
pub use test106_chain_audit_trail::TEST106_CHAIN_AUDIT_TRAIL;
pub use test107_tower_resolver::TEST107_TOWER_RESOLVER;
pub use test108_inductive_proof::TEST108_INDUCTIVE_PROOF;
pub use test109_validity_scope::TEST109_VALIDITY_SCOPE;
pub use test10_iterative_resolution::TEST10_ITERATIVE_RESOLUTION;
pub use test110_tower_roundtrip::TEST110_TOWER_ROUNDTRIP;
pub use test111_address_crypto_pinning::TEST111_ADDRESS_CRYPTO_PINNING;
pub use test112_address_canonical_bytes::TEST112_ADDRESS_CANONICAL_BYTES;
pub use test113_carry_constraint_pinning::TEST113_CARRY_CONSTRAINT_PINNING;
pub use test114_joint_satisfiability::TEST114_JOINT_SATISFIABILITY;
pub use test115_dihedral_inverse_order::TEST115_DIHEDRAL_INVERSE_ORDER;
pub use test116_constraint_expressiveness::TEST116_CONSTRAINT_EXPRESSIVENESS;
pub use test117_sumtype_topology::TEST117_SUMTYPE_TOPOLOGY;
pub use test118_synthesis_reachability::TEST118_SYNTHESIS_REACHABILITY;
pub use test119_obstruction_termination::TEST119_OBSTRUCTION_TERMINATION;
pub use test11_composition::TEST11_COMPOSITION;
pub use test120_coefficient_ring::TEST120_COEFFICIENT_RING;
pub use test121_gluing_feedback::TEST121_GLUING_FEEDBACK;
pub use test122_session_grounding::TEST122_SESSION_GROUNDING;
pub use test123_amplitude_index::TEST123_AMPLITUDE_INDEX;
pub use test124_glyph::TEST124_GLYPH;
pub use test125_dihedral_group::TEST125_DIHEDRAL_GROUP;
pub use test126_validity_scope_kind::TEST126_VALIDITY_SCOPE_KIND;
pub use test127_witt_level_resolver::TEST127_WITT_LEVEL_RESOLVER;
pub use test128_stratum_observable::TEST128_STRATUM_OBSERVABLE;
pub use test129_metric_observable::TEST129_METRIC_OBSERVABLE;
pub use test12_factorization::TEST12_FACTORIZATION;
pub use test130_path_observable::TEST130_PATH_OBSERVABLE;
pub use test131_reduction_observable::TEST131_REDUCTION_OBSERVABLE;
pub use test132_holonomy_observable::TEST132_HOLONOMY_OBSERVABLE;
pub use test133_incompatibility_metric::TEST133_INCOMPATIBILITY_METRIC;
pub use test134_stratum_value::TEST134_STRATUM_VALUE;
pub use test135_stratum_delta::TEST135_STRATUM_DELTA;
pub use test136_stratum_trajectory::TEST136_STRATUM_TRAJECTORY;
pub use test137_path_length::TEST137_PATH_LENGTH;
pub use test138_total_variation::TEST138_TOTAL_VARIATION;
pub use test139_winding_number::TEST139_WINDING_NUMBER;
pub use test13_canonical_form::TEST13_CANONICAL_FORM;
pub use test140_reduction_length::TEST140_REDUCTION_LENGTH;
pub use test141_reduction_count::TEST141_REDUCTION_COUNT;
pub use test142_catastrophe_threshold::TEST142_CATASTROPHE_THRESHOLD;
pub use test143_catastrophe_count::TEST143_CATASTROPHE_COUNT;
pub use test144_commutator::TEST144_COMMUTATOR;
pub use test145_curvature_flux::TEST145_CURVATURE_FLUX;
pub use test146_parallel_transport::TEST146_PARALLEL_TRANSPORT;
pub use test147_reduction_entropy::TEST147_REDUCTION_ENTROPY;
pub use test148_phase_boundary_type::TEST148_PHASE_BOUNDARY_TYPE;
pub use test149_face_map::TEST149_FACE_MAP;
pub use test14_content_addressing::TEST14_CONTENT_ADDRESSING;
pub use test150_nerve_functor::TEST150_NERVE_FUNCTOR;
pub use test151_chain_functor::TEST151_CHAIN_FUNCTOR;
pub use test152_restriction_map::TEST152_RESTRICTION_MAP;
pub use test153_coherence_proof::TEST153_COHERENCE_PROOF;
pub use test154_derivation_step::TEST154_DERIVATION_STEP;
pub use test155_computation_step::TEST155_COMPUTATION_STEP;
pub use test156_trace_metrics::TEST156_TRACE_METRICS;
pub use test157_isometry_certificate::TEST157_ISOMETRY_CERTIFICATE;
pub use test158_lift_chain_certificate::TEST158_LIFT_CHAIN_CERTIFICATE;
pub use test159_chain_audit_trail::TEST159_CHAIN_AUDIT_TRAIL;
pub use test15_boolean_sat::TEST15_BOOLEAN_SAT;
pub use test160_shared_context::TEST160_SHARED_CONTEXT;
pub use test161_execution_policy::TEST161_EXECUTION_POLICY;
pub use test162_session_composition::TEST162_SESSION_COMPOSITION;
pub use test163_distributed_grounding::TEST163_DISTRIBUTED_GROUNDING;
pub use test164_embedding::TEST164_EMBEDDING;
pub use test165_action::TEST165_ACTION;
pub use test166_session_boundary_type::TEST166_SESSION_BOUNDARY_TYPE;
pub use test167_metric_axis::TEST167_METRIC_AXIS;
pub use test168_witt_carry::TEST168_WITT_CARRY;
pub use test169_arithmetic_valuation::TEST169_ARITHMETIC_VALUATION;
pub use test16_algebraic_identities::TEST16_ALGEBRAIC_IDENTITIES;
pub use test170_kan_complex::TEST170_KAN_COMPLEX;
pub use test171_postnikov_truncation::TEST171_POSTNIKOV_TRUNCATION;
pub use test172_homotopy_group::TEST172_HOMOTOPY_GROUP;
pub use test173_homotopy_end_to_end::TEST173_HOMOTOPY_END_TO_END;
pub use test174_homotopy_resolver::TEST174_HOMOTOPY_RESOLVER;
pub use test175_homotopy_pipeline::TEST175_HOMOTOPY_PIPELINE;
pub use test176_moduli_space::TEST176_MODULI_SPACE;
pub use test177_deformation_complex::TEST177_DEFORMATION_COMPLEX;
pub use test178_holonomy_stratum::TEST178_HOLONOMY_STRATUM;
pub use test179_moduli_end_to_end::TEST179_MODULI_END_TO_END;
pub use test17_inter_algebra_maps::TEST17_INTER_ALGEBRA_MAPS;
pub use test180_moduli_resolver::TEST180_MODULI_RESOLVER;
pub use test181_stratification_record::TEST181_STRATIFICATION_RECORD;
pub use test182_whitehead_product::TEST182_WHITEHEAD_PRODUCT;
pub use test183_deformation_family::TEST183_DEFORMATION_FAMILY;
pub use test184_versal_deformation::TEST184_VERSAL_DEFORMATION;
pub use test185_carry_chain::TEST185_CARRY_CHAIN;
pub use test186_carry_event::TEST186_CARRY_EVENT;
pub use test187_carry_profile::TEST187_CARRY_PROFILE;
pub use test188_encoding_configuration::TEST188_ENCODING_CONFIGURATION;
pub use test189_encoding_quality::TEST189_ENCODING_QUALITY;
pub use test18_analytical_completeness::TEST18_ANALYTICAL_COMPLETENESS;
pub use test190_base_metric::TEST190_BASE_METRIC;
pub use test191_grounding_observable::TEST191_GROUNDING_OBSERVABLE;
pub use test192_euler_characteristic::TEST192_EULER_CHARACTERISTIC;
pub use test193_galois_connection::TEST193_GALOIS_CONNECTION;
pub use test194_nerve_operations::TEST194_NERVE_OPERATIONS;
pub use test195_scalar_symbol_type::TEST195_SCALAR_SYMBOL_TYPE;
pub use test196_sequence_tuple_type::TEST196_SEQUENCE_TUPLE_TYPE;
pub use test197_graph_tree_type::TEST197_GRAPH_TREE_TYPE;
pub use test198_composed_operation::TEST198_COMPOSED_OPERATION;
pub use test199_dispatch_operation::TEST199_DISPATCH_OPERATION;
pub use test19_homological_pipeline::TEST19_HOMOLOGICAL_PIPELINE;
pub use test1_ring::TEST1_RING;
pub use test200_inference_operation::TEST200_INFERENCE_OPERATION;
pub use test201_accumulation_operation::TEST201_ACCUMULATION_OPERATION;
pub use test202_lease_partition_operation::TEST202_LEASE_PARTITION_OPERATION;
pub use test203_session_composition_operation::TEST203_SESSION_COMPOSITION_OPERATION;
pub use test204_euler_reduction::TEST204_EULER_REDUCTION;
pub use test205_reduction_step::TEST205_REDUCTION_STEP;
pub use test206_reduction_state::TEST206_REDUCTION_STATE;
pub use test207_phase_gate::TEST207_PHASE_GATE;
pub use test208_epoch::TEST208_EPOCH;
pub use test209_predicate_expression::TEST209_PREDICATE_EXPRESSION;
pub use test20_sheaf_consistency::TEST20_SHEAF_CONSISTENCY;
pub use test210_guard_expression::TEST210_GUARD_EXPRESSION;
pub use test211_transition_effect::TEST211_TRANSITION_EFFECT;
pub use test212_service_window::TEST212_SERVICE_WINDOW;
pub use test213_reduction_transaction::TEST213_REDUCTION_TRANSACTION;
pub use test214_feasibility_result::TEST214_FEASIBILITY_RESULT;
pub use test215_lease_state::TEST215_LEASE_STATE;
pub use test216_managed_lease::TEST216_MANAGED_LEASE;
pub use test217_back_pressure::TEST217_BACK_PRESSURE;
pub use test218_deferred_query::TEST218_DEFERRED_QUERY;
pub use test219_convergence_level::TEST219_CONVERGENCE_LEVEL;
pub use test21_topological_delta::TEST21_TOPOLOGICAL_DELTA;
pub use test220_hopf_fiber::TEST220_HOPF_FIBER;
pub use test221_convergence_residual::TEST221_CONVERGENCE_RESIDUAL;
pub use test222_commutative_subspace::TEST222_COMMUTATIVE_SUBSPACE;
pub use test223_associative_subalgebra::TEST223_ASSOCIATIVE_SUBALGEBRA;
pub use test224_normed_division_algebra::TEST224_NORMED_DIVISION_ALGEBRA;
pub use test225_cayley_dickson::TEST225_CAYLEY_DICKSON;
pub use test226_multiplication_table::TEST226_MULTIPLICATION_TABLE;
pub use test227_algebra_commutator::TEST227_ALGEBRA_COMMUTATOR;
pub use test228_algebra_associator::TEST228_ALGEBRA_ASSOCIATOR;
pub use test229_interaction_context::TEST229_INTERACTION_CONTEXT;
pub use test22_index_bridge::TEST22_INDEX_BRIDGE;
pub use test230_commutator_state::TEST230_COMMUTATOR_STATE;
pub use test231_associator_state::TEST231_ASSOCIATOR_STATE;
pub use test232_three_way_site::TEST232_THREE_WAY_SITE;
pub use test233_negotiation_trace::TEST233_NEGOTIATION_TRACE;
pub use test234_interaction_nerve::TEST234_INTERACTION_NERVE;
pub use test235_monoidal_product::TEST235_MONOIDAL_PRODUCT;
pub use test236_monoidal_unit::TEST236_MONOIDAL_UNIT;
pub use test237_monoidal_associator::TEST237_MONOIDAL_ASSOCIATOR;
pub use test238_structural_operad::TEST238_STRUCTURAL_OPERAD;
pub use test239_operad_composition::TEST239_OPERAD_COMPOSITION;
pub use test23_identity_grounding::TEST23_IDENTITY_GROUNDING;
pub use test240_operad_end_to_end::TEST240_OPERAD_END_TO_END;
pub use test241_associator_triple::TEST241_ASSOCIATOR_TRIPLE;
pub use test242_mutual_model_trace::TEST242_MUTUAL_MODEL_TRACE;
pub use test243_interaction_composition::TEST243_INTERACTION_COMPOSITION;
pub use test244_sublease_transfer::TEST244_SUBLEASE_TRANSFER;
pub use test245_predicate_subclasses::TEST245_PREDICATE_SUBCLASSES;
pub use test246_phase_rotation::TEST246_PHASE_ROTATION;
pub use test247_rollback_transition::TEST247_ROLLBACK_TRANSITION;
pub use test248_epoch_boundary::TEST248_EPOCH_BOUNDARY;
pub use test249_property_bind_advance::TEST249_PROPERTY_BIND_ADVANCE;
pub use test24_verification_domain::TEST24_VERIFICATION_DOMAIN;
pub use test250_pipeline_outcome::TEST250_PIPELINE_OUTCOME;
pub use test251_preflight_checkpoint::TEST251_PREFLIGHT_CHECKPOINT;
pub use test252_compile_unit::TEST252_COMPILE_UNIT;
pub use test253_expression_types::TEST253_EXPRESSION_TYPES;
pub use test254_proof_derivation_types::TEST254_PROOF_DERIVATION_TYPES;
pub use test255_effect_types::TEST255_EFFECT_TYPES;
pub use test256_predicate_types::TEST256_PREDICATE_TYPES;
pub use test257_parallel_types::TEST257_PARALLEL_TYPES;
pub use test258_stream_types::TEST258_STREAM_TYPES;
pub use test259_failure_types::TEST259_FAILURE_TYPES;
pub use test25_geometric_character::TEST25_GEOMETRIC_CHARACTER;
pub use test260_linear_types::TEST260_LINEAR_TYPES;
pub use test261_recursion_types::TEST261_RECURSION_TYPES;
pub use test262_region_types::TEST262_REGION_TYPES;
pub use test263_boundary_types::TEST263_BOUNDARY_TYPES;
pub use test264_conformance_types::TEST264_CONFORMANCE_TYPES;
pub use test265_user_types::TEST265_USER_TYPES;
pub use test266_witness_datum::TEST266_WITNESS_DATUM;
pub use test267_compile_unit_builder::TEST267_COMPILE_UNIT_BUILDER;
pub use test268_violation_kind::TEST268_VIOLATION_KIND;
pub use test269_predicate_individuals::TEST269_PREDICATE_INDIVIDUALS;
pub use test26_complexity_class::TEST26_COMPLEXITY_CLASS;
pub use test270_constraint_subclasses::TEST270_CONSTRAINT_SUBCLASSES;
pub use test271_host_value_types::TEST271_HOST_VALUE_TYPES;
pub use test272_boundary_map_individuals::TEST272_BOUNDARY_MAP_INDIVIDUALS;
pub use test273_type_definition_sum::TEST273_TYPE_DEFINITION_SUM;
pub use test274_witness_types::TEST274_WITNESS_TYPES;
pub use test275_reduction_advance::TEST275_REDUCTION_ADVANCE;
pub use test276_witness_site_budget::TEST276_WITNESS_SITE_BUDGET;
pub use test277_inhabitance_verdict::TEST277_INHABITANCE_VERDICT;
pub use test278_v021_cleanup_bounds::TEST278_V021_CLEANUP_BOUNDS;
pub use test279_multiplication_certificate::TEST279_MULTIPLICATION_CERTIFICATE;
pub use test27_rewrite_rule::TEST27_REWRITE_RULE;
pub use test280_bridge_completion::TEST280_BRIDGE_COMPLETION;
pub use test281_impossibility_certificates::TEST281_IMPOSSIBILITY_CERTIFICATES;
// Product/Coproduct Completion Amendment fixtures.
pub use test285_cartesian_partition_product::TEST285_CARTESIAN_PARTITION_PRODUCT;
pub use test286_tag_site::TEST286_TAG_SITE;
pub use test287_layout_invariant::TEST287_LAYOUT_INVARIANT;
// Wiki ADR-038: AxisProjectionObservable closed-catalog extension.
pub use test288_axis_projection_observable::TEST288_AXIS_PROJECTION_OBSERVABLE;
// Wiki ADR-049: SpectralObservable closed-catalog extension.
pub use test289_spectral_observable::TEST289_SPECTRAL_OBSERVABLE;
// Wiki ADR-040 + ADR-049 catalog correspondence: ValueThresholdObservable closed-catalog extension.
pub use test28_measurement_unit::TEST28_MEASUREMENT_UNIT;
pub use test290_value_threshold_observable::TEST290_VALUE_THRESHOLD_OBSERVABLE;
pub use test29_triad_projection::TEST29_TRIAD_PROJECTION;
pub use test2_primitives::TEST2_PRIMITIVES;
pub use test30_proof_coverage::TEST30_PROOF_COVERAGE;
pub use test31_witt_level::TEST31_WITT_LEVEL;
pub use test32_arc_grounding::TEST32_ARC_GROUNDING;
pub use test33_graph_gaps::TEST33_GRAPH_GAPS;
pub use test34_completeness_candidate::TEST34_COMPLETENESS_CANDIDATE;
pub use test35_completeness_certificate::TEST35_COMPLETENESS_CERTIFICATE;
pub use test36_w16_ring::TEST36_W16_RING;
pub use test37_witt_level_binding::TEST37_WITT_LEVEL_BINDING;
pub use test38_session_lifecycle::TEST38_SESSION_LIFECYCLE;
pub use test39_session_boundary::TEST39_SESSION_BOUNDARY;
pub use test3_term_graph::TEST3_TERM_GRAPH;
pub use test40_type_synthesis_goal::TEST40_TYPE_SYNTHESIS_GOAL;
pub use test41_synthesis_result::TEST41_SYNTHESIS_RESULT;
pub use test42_witt_lift::TEST42_WITT_LIFT;
pub use test43_spectral_sequence::TEST43_SPECTRAL_SEQUENCE;
pub use test44_monodromy_flat::TEST44_MONODROMY_FLAT;
pub use test45_monodromy_twisted::TEST45_MONODROMY_TWISTED;
pub use test46_monodromy_pipeline::TEST46_MONODROMY_PIPELINE;
pub use test47_thermo_pipeline::TEST47_THERMO_PIPELINE;
pub use test48_phase_diagram::TEST48_PHASE_DIAGRAM;
pub use test49_reversible_resolution::TEST49_REVERSIBLE_RESOLUTION;
pub use test4_state_lifecycle::TEST4_STATE_LIFECYCLE;
pub use test50_jacobian_resolver::TEST50_JACOBIAN_RESOLVER;
pub use test51_product_type_pipeline::TEST51_PRODUCT_TYPE_PIPELINE;
pub use test52_sum_type_variant::TEST52_SUM_TYPE_VARIANT;
pub use test53_superposed_site::TEST53_SUPERPOSED_SITE;
pub use test54_grounded_context::TEST54_GROUNDED_CONTEXT;
pub use test55_grounding_witness::TEST55_GROUNDING_WITNESS;
pub use test56_domain_grounding_record::TEST56_DOMAIN_GROUNDING_RECORD;
pub use test57_grounding_phase::TEST57_GROUNDING_PHASE;
pub use test58_grounding_certificate::TEST58_GROUNDING_CERTIFICATE;
pub use test59_grounding_aware_resolver::TEST59_GROUNDING_AWARE_RESOLVER;
pub use test5_partition::TEST5_PARTITION;
pub use test60_impossibility_witness::TEST60_IMPOSSIBILITY_WITNESS;
pub use test61_morphospace_record::TEST61_MORPHOSPACE_RECORD;
pub use test62_morphospace_boundary::TEST62_MORPHOSPACE_BOUNDARY;
pub use test63_forbidden_signature::TEST63_FORBIDDEN_SIGNATURE;
pub use test64_achievability_status::TEST64_ACHIEVABILITY_STATUS;
pub use test65_geodesic_trace::TEST65_GEODESIC_TRACE;
pub use test66_geodesic_certificate::TEST66_GEODESIC_CERTIFICATE;
pub use test67_geodesic_violation::TEST67_GEODESIC_VIOLATION;
pub use test68_geodesic_validator::TEST68_GEODESIC_VALIDATOR;
pub use test69_geodesic_ordered::TEST69_GEODESIC_ORDERED;
pub use test6_critical_identity::TEST6_CRITICAL_IDENTITY;
pub use test70_measurement_resolver::TEST70_MEASUREMENT_RESOLVER;
pub use test71_measurement_event::TEST71_MEASUREMENT_EVENT;
pub use test72_measurement_certificate::TEST72_MEASUREMENT_CERTIFICATE;
pub use test73_collapsed_site_state::TEST73_COLLAPSED_SITE_STATE;
pub use test74_quantum_thermodynamic::TEST74_QUANTUM_THERMODYNAMIC;
pub use test75_partition_product::TEST75_PARTITION_PRODUCT;
pub use test76_partition_coproduct::TEST76_PARTITION_COPRODUCT;
pub use test77_geodesic_evidence::TEST77_GEODESIC_EVIDENCE;
pub use test78_born_rule::TEST78_BORN_RULE;
pub use test79_measurement_outcome::TEST79_MEASUREMENT_OUTCOME;
pub use test7_end_to_end::TEST7_END_TO_END;
pub use test80_partition_exhaustive::TEST80_PARTITION_EXHAUSTIVE;
pub use test81_dihedral_algebra::TEST81_DIHEDRAL_ALGEBRA;
pub use test82_level_successor::TEST82_LEVEL_SUCCESSOR;
pub use test83_amplitude_normalization::TEST83_AMPLITUDE_NORMALIZATION;
pub use test84_enum_variant::TEST84_ENUM_VARIANT;
pub use test85_w16_ring_grounding::TEST85_W16_RING_GROUNDING;
pub use test86_witt_lift_trivial::TEST86_WITT_LIFT_TRIVIAL;
pub use test87_spectral_convergence::TEST87_SPECTRAL_CONVERGENCE;
pub use test88_lift_obstruction_nontrivial::TEST88_LIFT_OBSTRUCTION_NONTRIVIAL;
pub use test89_lift_refinement_suggestion::TEST89_LIFT_REFINEMENT_SUGGESTION;
pub use test8_free_rank::TEST8_FREE_RANK;
pub use test90_resolved_lift::TEST90_RESOLVED_LIFT;
pub use test91_synthesis_goal_w16::TEST91_SYNTHESIS_GOAL_W16;
pub use test92_synthesis_checkpoint::TEST92_SYNTHESIS_CHECKPOINT;
pub use test93_synthesis_signature::TEST93_SYNTHESIS_SIGNATURE;
pub use test94_synthesized_type::TEST94_SYNTHESIZED_TYPE;
pub use test95_unreachable_signature::TEST95_UNREACHABLE_SIGNATURE;
pub use test96_geodesic_trace_w16::TEST96_GEODESIC_TRACE_W16;
pub use test97_evidence_bundle_ar1::TEST97_EVIDENCE_BUNDLE_AR1;
pub use test98_evidence_bundle_dc10::TEST98_EVIDENCE_BUNDLE_DC10;
pub use test99_measurement_born_w16::TEST99_MEASUREMENT_BORN_W16;
pub use test9_constraint_algebra::TEST9_CONSTRAINT_ALGEBRA;

/// Returns all compiled SHACL fixture source strings for meta-validator
/// scanning (Amendment 45, Rule 3).
pub fn all_fixture_sources() -> Vec<&'static str> {
    vec![
        TEST1_RING,
        TEST2_PRIMITIVES,
        TEST3_TERM_GRAPH,
        TEST4_STATE_LIFECYCLE,
        TEST5_PARTITION,
        TEST6_CRITICAL_IDENTITY,
        TEST7_END_TO_END,
        TEST8_FREE_RANK,
        TEST9_CONSTRAINT_ALGEBRA,
        TEST10_ITERATIVE_RESOLUTION,
        TEST11_COMPOSITION,
        TEST12_FACTORIZATION,
        TEST13_CANONICAL_FORM,
        TEST14_CONTENT_ADDRESSING,
        TEST15_BOOLEAN_SAT,
        TEST16_ALGEBRAIC_IDENTITIES,
        TEST17_INTER_ALGEBRA_MAPS,
        TEST18_ANALYTICAL_COMPLETENESS,
        TEST19_HOMOLOGICAL_PIPELINE,
        TEST20_SHEAF_CONSISTENCY,
        TEST21_TOPOLOGICAL_DELTA,
        TEST22_INDEX_BRIDGE,
        TEST23_IDENTITY_GROUNDING,
        TEST24_VERIFICATION_DOMAIN,
        TEST25_GEOMETRIC_CHARACTER,
        TEST26_COMPLEXITY_CLASS,
        TEST27_REWRITE_RULE,
        TEST28_MEASUREMENT_UNIT,
        TEST29_TRIAD_PROJECTION,
        TEST30_PROOF_COVERAGE,
        TEST31_WITT_LEVEL,
        TEST32_ARC_GROUNDING,
        TEST33_GRAPH_GAPS,
        TEST34_COMPLETENESS_CANDIDATE,
        TEST35_COMPLETENESS_CERTIFICATE,
        TEST36_W16_RING,
        TEST37_WITT_LEVEL_BINDING,
        TEST38_SESSION_LIFECYCLE,
        TEST39_SESSION_BOUNDARY,
        TEST40_TYPE_SYNTHESIS_GOAL,
        TEST41_SYNTHESIS_RESULT,
        TEST42_WITT_LIFT,
        TEST43_SPECTRAL_SEQUENCE,
        TEST44_MONODROMY_FLAT,
        TEST45_MONODROMY_TWISTED,
        TEST46_MONODROMY_PIPELINE,
        TEST47_THERMO_PIPELINE,
        TEST48_PHASE_DIAGRAM,
        TEST49_REVERSIBLE_RESOLUTION,
        TEST50_JACOBIAN_RESOLVER,
        TEST51_PRODUCT_TYPE_PIPELINE,
        TEST52_SUM_TYPE_VARIANT,
        TEST53_SUPERPOSED_SITE,
        TEST54_GROUNDED_CONTEXT,
        TEST55_GROUNDING_WITNESS,
        TEST56_DOMAIN_GROUNDING_RECORD,
        TEST57_GROUNDING_PHASE,
        TEST58_GROUNDING_CERTIFICATE,
        TEST59_GROUNDING_AWARE_RESOLVER,
        TEST60_IMPOSSIBILITY_WITNESS,
        TEST61_MORPHOSPACE_RECORD,
        TEST62_MORPHOSPACE_BOUNDARY,
        TEST63_FORBIDDEN_SIGNATURE,
        TEST64_ACHIEVABILITY_STATUS,
        TEST65_GEODESIC_TRACE,
        TEST66_GEODESIC_CERTIFICATE,
        TEST67_GEODESIC_VIOLATION,
        TEST68_GEODESIC_VALIDATOR,
        TEST69_GEODESIC_ORDERED,
        TEST70_MEASUREMENT_RESOLVER,
        TEST71_MEASUREMENT_EVENT,
        TEST72_MEASUREMENT_CERTIFICATE,
        TEST73_COLLAPSED_SITE_STATE,
        TEST74_QUANTUM_THERMODYNAMIC,
        TEST75_PARTITION_PRODUCT,
        TEST76_PARTITION_COPRODUCT,
        TEST77_GEODESIC_EVIDENCE,
        TEST78_BORN_RULE,
        TEST79_MEASUREMENT_OUTCOME,
        TEST80_PARTITION_EXHAUSTIVE,
        TEST81_DIHEDRAL_ALGEBRA,
        TEST82_LEVEL_SUCCESSOR,
        TEST83_AMPLITUDE_NORMALIZATION,
        TEST84_ENUM_VARIANT,
        TEST85_W16_RING_GROUNDING,
        TEST86_WITT_LIFT_TRIVIAL,
        TEST87_SPECTRAL_CONVERGENCE,
        TEST88_LIFT_OBSTRUCTION_NONTRIVIAL,
        TEST89_LIFT_REFINEMENT_SUGGESTION,
        TEST90_RESOLVED_LIFT,
        TEST91_SYNTHESIS_GOAL_W16,
        TEST92_SYNTHESIS_CHECKPOINT,
        TEST93_SYNTHESIS_SIGNATURE,
        TEST94_SYNTHESIZED_TYPE,
        TEST95_UNREACHABLE_SIGNATURE,
        TEST96_GEODESIC_TRACE_W16,
        TEST97_EVIDENCE_BUNDLE_AR1,
        TEST98_EVIDENCE_BUNDLE_DC10,
        TEST99_MEASUREMENT_BORN_W16,
        TEST100_NORMATIVE_CHAIN,
        TEST101_LIFT_CHAIN_FLAT,
        TEST102_LIFT_CHAIN_TWISTED,
        TEST103_OBSTRUCTION_CHAIN_EMPTY,
        TEST104_OBSTRUCTION_CHAIN_NONTRIVIAL,
        TEST105_LIFT_CHAIN_CERTIFICATE,
        TEST106_CHAIN_AUDIT_TRAIL,
        TEST107_TOWER_RESOLVER,
        TEST108_INDUCTIVE_PROOF,
        TEST109_VALIDITY_SCOPE,
        TEST110_TOWER_ROUNDTRIP,
        TEST111_ADDRESS_CRYPTO_PINNING,
        TEST112_ADDRESS_CANONICAL_BYTES,
        TEST113_CARRY_CONSTRAINT_PINNING,
        TEST114_JOINT_SATISFIABILITY,
        TEST115_DIHEDRAL_INVERSE_ORDER,
        TEST116_CONSTRAINT_EXPRESSIVENESS,
        TEST117_SUMTYPE_TOPOLOGY,
        TEST118_SYNTHESIS_REACHABILITY,
        TEST119_OBSTRUCTION_TERMINATION,
        TEST120_COEFFICIENT_RING,
        TEST121_GLUING_FEEDBACK,
        TEST122_SESSION_GROUNDING,
        TEST123_AMPLITUDE_INDEX,
        TEST124_GLYPH,
        TEST125_DIHEDRAL_GROUP,
        TEST126_VALIDITY_SCOPE_KIND,
        TEST127_WITT_LEVEL_RESOLVER,
        TEST128_STRATUM_OBSERVABLE,
        TEST129_METRIC_OBSERVABLE,
        TEST130_PATH_OBSERVABLE,
        TEST131_REDUCTION_OBSERVABLE,
        TEST132_HOLONOMY_OBSERVABLE,
        TEST133_INCOMPATIBILITY_METRIC,
        TEST134_STRATUM_VALUE,
        TEST135_STRATUM_DELTA,
        TEST136_STRATUM_TRAJECTORY,
        TEST137_PATH_LENGTH,
        TEST138_TOTAL_VARIATION,
        TEST139_WINDING_NUMBER,
        TEST140_REDUCTION_LENGTH,
        TEST141_REDUCTION_COUNT,
        TEST142_CATASTROPHE_THRESHOLD,
        TEST143_CATASTROPHE_COUNT,
        TEST144_COMMUTATOR,
        TEST145_CURVATURE_FLUX,
        TEST146_PARALLEL_TRANSPORT,
        TEST147_REDUCTION_ENTROPY,
        TEST148_PHASE_BOUNDARY_TYPE,
        TEST149_FACE_MAP,
        TEST150_NERVE_FUNCTOR,
        TEST151_CHAIN_FUNCTOR,
        TEST152_RESTRICTION_MAP,
        TEST153_COHERENCE_PROOF,
        TEST154_DERIVATION_STEP,
        TEST155_COMPUTATION_STEP,
        TEST156_TRACE_METRICS,
        TEST157_ISOMETRY_CERTIFICATE,
        TEST158_LIFT_CHAIN_CERTIFICATE,
        TEST159_CHAIN_AUDIT_TRAIL,
        TEST160_SHARED_CONTEXT,
        TEST161_EXECUTION_POLICY,
        TEST162_SESSION_COMPOSITION,
        TEST163_DISTRIBUTED_GROUNDING,
        TEST164_EMBEDDING,
        TEST165_ACTION,
        TEST166_SESSION_BOUNDARY_TYPE,
        TEST167_METRIC_AXIS,
        TEST168_WITT_CARRY,
        TEST169_ARITHMETIC_VALUATION,
        TEST170_KAN_COMPLEX,
        TEST171_POSTNIKOV_TRUNCATION,
        TEST172_HOMOTOPY_GROUP,
        TEST173_HOMOTOPY_END_TO_END,
        TEST174_HOMOTOPY_RESOLVER,
        TEST175_HOMOTOPY_PIPELINE,
        TEST176_MODULI_SPACE,
        TEST177_DEFORMATION_COMPLEX,
        TEST178_HOLONOMY_STRATUM,
        TEST179_MODULI_END_TO_END,
        TEST180_MODULI_RESOLVER,
        TEST181_STRATIFICATION_RECORD,
        TEST182_WHITEHEAD_PRODUCT,
        TEST183_DEFORMATION_FAMILY,
        TEST184_VERSAL_DEFORMATION,
        TEST185_CARRY_CHAIN,
        TEST186_CARRY_EVENT,
        TEST187_CARRY_PROFILE,
        TEST188_ENCODING_CONFIGURATION,
        TEST189_ENCODING_QUALITY,
        TEST190_BASE_METRIC,
        TEST191_GROUNDING_OBSERVABLE,
        TEST192_EULER_CHARACTERISTIC,
        TEST193_GALOIS_CONNECTION,
        TEST194_NERVE_OPERATIONS,
        TEST195_SCALAR_SYMBOL_TYPE,
        TEST196_SEQUENCE_TUPLE_TYPE,
        TEST197_GRAPH_TREE_TYPE,
        TEST198_COMPOSED_OPERATION,
        TEST199_DISPATCH_OPERATION,
        TEST200_INFERENCE_OPERATION,
        TEST201_ACCUMULATION_OPERATION,
        TEST202_LEASE_PARTITION_OPERATION,
        TEST203_SESSION_COMPOSITION_OPERATION,
        TEST204_EULER_REDUCTION,
        TEST205_REDUCTION_STEP,
        TEST206_REDUCTION_STATE,
        TEST207_PHASE_GATE,
        TEST208_EPOCH,
        TEST209_PREDICATE_EXPRESSION,
        TEST210_GUARD_EXPRESSION,
        TEST211_TRANSITION_EFFECT,
        TEST212_SERVICE_WINDOW,
        TEST213_REDUCTION_TRANSACTION,
        TEST214_FEASIBILITY_RESULT,
        TEST215_LEASE_STATE,
        TEST216_MANAGED_LEASE,
        TEST217_BACK_PRESSURE,
        TEST218_DEFERRED_QUERY,
        TEST219_CONVERGENCE_LEVEL,
        TEST220_HOPF_FIBER,
        TEST221_CONVERGENCE_RESIDUAL,
        TEST222_COMMUTATIVE_SUBSPACE,
        TEST223_ASSOCIATIVE_SUBALGEBRA,
        TEST224_NORMED_DIVISION_ALGEBRA,
        TEST225_CAYLEY_DICKSON,
        TEST226_MULTIPLICATION_TABLE,
        TEST227_ALGEBRA_COMMUTATOR,
        TEST228_ALGEBRA_ASSOCIATOR,
        TEST229_INTERACTION_CONTEXT,
        TEST230_COMMUTATOR_STATE,
        TEST231_ASSOCIATOR_STATE,
        TEST232_THREE_WAY_SITE,
        TEST233_NEGOTIATION_TRACE,
        TEST234_INTERACTION_NERVE,
        TEST235_MONOIDAL_PRODUCT,
        TEST236_MONOIDAL_UNIT,
        TEST237_MONOIDAL_ASSOCIATOR,
        TEST238_STRUCTURAL_OPERAD,
        TEST239_OPERAD_COMPOSITION,
        TEST240_OPERAD_END_TO_END,
        TEST241_ASSOCIATOR_TRIPLE,
        TEST242_MUTUAL_MODEL_TRACE,
        TEST243_INTERACTION_COMPOSITION,
        TEST244_SUBLEASE_TRANSFER,
        TEST245_PREDICATE_SUBCLASSES,
        TEST246_PHASE_ROTATION,
        TEST247_ROLLBACK_TRANSITION,
        TEST248_EPOCH_BOUNDARY,
        TEST249_PROPERTY_BIND_ADVANCE,
        TEST250_PIPELINE_OUTCOME,
        TEST251_PREFLIGHT_CHECKPOINT,
        TEST252_COMPILE_UNIT,
        TEST253_EXPRESSION_TYPES,
        TEST254_PROOF_DERIVATION_TYPES,
        TEST255_EFFECT_TYPES,
        TEST256_PREDICATE_TYPES,
        TEST257_PARALLEL_TYPES,
        TEST258_STREAM_TYPES,
        TEST259_FAILURE_TYPES,
        TEST260_LINEAR_TYPES,
        TEST261_RECURSION_TYPES,
        TEST262_REGION_TYPES,
        TEST263_BOUNDARY_TYPES,
        TEST264_CONFORMANCE_TYPES,
        TEST265_USER_TYPES,
        TEST266_WITNESS_DATUM,
        TEST267_COMPILE_UNIT_BUILDER,
        TEST268_VIOLATION_KIND,
        TEST269_PREDICATE_INDIVIDUALS,
        TEST270_CONSTRAINT_SUBCLASSES,
        TEST271_HOST_VALUE_TYPES,
        TEST272_BOUNDARY_MAP_INDIVIDUALS,
        TEST273_TYPE_DEFINITION_SUM,
        TEST274_WITNESS_TYPES,
        TEST275_REDUCTION_ADVANCE,
        TEST276_WITNESS_SITE_BUDGET,
        TEST277_INHABITANCE_VERDICT,
        TEST278_V021_CLEANUP_BOUNDS,
        TEST279_MULTIPLICATION_CERTIFICATE,
        TEST280_BRIDGE_COMPLETION,
        TEST281_IMPOSSIBILITY_CERTIFICATES,
        TEST285_CARTESIAN_PARTITION_PRODUCT,
        TEST286_TAG_SITE,
        TEST287_LAYOUT_INVARIANT,
        TEST288_AXIS_PROJECTION_OBSERVABLE,
        TEST289_SPECTRAL_OBSERVABLE,
        TEST290_VALUE_THRESHOLD_OBSERVABLE,
    ]
}
