//! Ontology inventory validator.
//!
//! Verifies that the built ontology artifact contains the correct counts
//! as defined in [`uor_ontology::counts`].

use std::collections::{HashMap, HashSet};
use std::path::Path;

use anyhow::{Context, Result};
use serde_json::Value;
use uor_ontology::counts;
use uor_ontology::model::{IndividualValue, PropertyKind, Space};
use uor_ontology::Ontology;

use crate::report::{ConformanceReport, TestResult};

/// Validates the ontology inventory counts in the built JSON-LD artifact.
///
/// # Errors
///
/// Returns an error if the artifact file cannot be read or parsed.
pub fn validate(artifacts: &Path) -> Result<ConformanceReport> {
    let mut report = ConformanceReport::new();

    // Also validate against the live spec (no file I/O needed)
    validate_spec_counts(&mut report);

    // Hardening: three-space classification
    validate_space_classification(&mut report);

    // Hardening: individual completeness
    validate_individual_completeness(&mut report);

    // Hardening: identity completeness (all op:Identity individuals have lhs/rhs/forAll)
    validate_identity_completeness(&mut report);

    // Hardening: identity grounding (all op:Identity individuals have typed verification properties)
    validate_identity_grounding(&mut report);

    // v3.2: proof coverage and quantum scope
    validate_proof_coverage(&mut report);
    validate_quantum_scope(&mut report);

    // Amendment 23: vocabulary individual validators
    validate_verification_domain_individuals(&mut report);
    validate_geometric_character_individuals(&mut report);
    validate_complexity_class_individuals(&mut report);
    validate_rewrite_rule_individuals(&mut report);
    validate_measurement_unit_individuals(&mut report);
    validate_coordinate_kind_individuals(&mut report);
    validate_surface_symmetry_identity(&mut report);
    validate_saturation_phase_individuals(&mut report);
    validate_achievability_status_individuals(&mut report);
    // Amendment 37: Gap closure validators
    validate_superposition_identity_coverage(&mut report);
    validate_partition_tensor_product(&mut report);
    validate_geodesic_decomposition(&mut report);
    validate_born_rule_coverage(&mut report);
    validate_enum_variant_alignment(&mut report);
    // Amendment 38-40: Q1 Conformance Coverage validators
    validate_q1_ring_grounding(&mut report);
    validate_lift_obstruction_paths(&mut report);
    validate_synthesis_q1_coverage(&mut report);
    validate_evidence_bundle_properties(&mut report);
    validate_normative_chain_integrity(&mut report);
    // Amendment 41: Tower chain vocabulary validators
    validate_validity_scope_individuals(&mut report);
    validate_tower_chain_vocabulary(&mut report);
    // Amendment 42: EBNF grammar alignment
    validate_ebnf_grammar_alignment(&mut report);
    // Amendment 43: Cryptographic Primitive Pinning
    validate_crypto_pinning_vocabulary(&mut report);
    // Amendment 44: Structural Gap Closures
    validate_carry_constraint_pinning(&mut report);
    validate_joint_satisfiability_coverage(&mut report);
    validate_dihedral_algebra_completeness(&mut report);
    validate_constraint_expressiveness(&mut report);
    validate_sumtype_topology(&mut report);
    validate_synthesis_reachability(&mut report);
    validate_obstruction_termination(&mut report);
    validate_coefficient_ring_grounding(&mut report);
    validate_gluing_feedback(&mut report);
    validate_session_saturation_bridge(&mut report);
    validate_amplitude_index_characterization(&mut report);

    // Amendment 45: Self-Auditing Meta-Validators
    validate_certificate_issuance_coverage(&mut report);
    validate_identity_proof_bijection(&mut report);
    validate_shacl_fixture_coverage(&mut report);

    // Amendment 47: Model-derived meta-validators (Rules 4-8)
    validate_property_assertion_iris(&mut report);
    validate_property_kind_conformance(&mut report);
    validate_functional_cardinality(&mut report);
    validate_iri_ref_targets(&mut report);
    validate_datatype_range_conformance(&mut report);

    // Amendment 49: Scope binding cross-checks
    validate_forall_scope_alignment(&mut report);

    // Amendment 53: Witt-Carry Formalization
    validate_witt_carry_vocabulary(&mut report);
    validate_ostrowski_derivation_chain(&mut report);

    // Amendment 54: Homotopy Nerve
    validate_homotopy_nerve_vocabulary(&mut report);
    validate_postnikov_bridge(&mut report);
    // Amendment 55: Homotopy Pipeline
    validate_homotopy_pipeline(&mut report);
    // Amendment 56: Moduli Space
    validate_moduli_vocabulary(&mut report);
    validate_deformation_complex(&mut report);
    // Amendment 57: Moduli Resolver
    validate_moduli_resolver_vocabulary(&mut report);
    // Amendment 58: Carry Algebra
    validate_carry_algebra_vocabulary(&mut report);
    // Amendment 59: Named Base Metrics
    validate_named_base_metrics(&mut report);
    // Amendment 60: Galois Connection + Nerve Operations
    validate_galois_nerve_vocabulary(&mut report);
    // Amendment 61: Structural Types
    validate_structural_types_vocabulary(&mut report);
    // Amendment 62: Composed Operations
    validate_composed_ops_vocabulary(&mut report);
    // Amendment 63: Reduction Core
    validate_reduction_core_vocabulary(&mut report);
    // Amendment 64: Reduction Expansion
    validate_reduction_expansion_vocabulary(&mut report);
    // Amendment 65: Reduction Completion
    validate_reduction_completion_vocabulary(&mut report);
    // Amendment 66: Convergence Tower
    validate_convergence_tower_vocabulary(&mut report);
    // ADR-059: convergence-tower codomain stratification of κ-derivation
    validate_convergence_codomain_stratification(&mut report);
    // Amendment 67: Division Algebras
    validate_division_algebras_vocabulary(&mut report);
    // Amendment 68: Interaction Algebra
    validate_interaction_algebra_vocabulary(&mut report);
    // Amendment 69: Monoidal Composition
    validate_monoidal_composition_vocabulary(&mut report);
    // Amendment 70: Operad Composition
    validate_operad_composition_vocabulary(&mut report);

    // Amendment 84: CompileUnit
    validate_compile_unit_vocabulary(&mut report);
    validate_compile_unit_identities(&mut report);

    // Amendment 85: InductiveProof acyclicity
    validate_inductive_proof_acyclicity(&mut report);

    // Amendment 86: EmpiricalVerification elimination
    validate_no_empirical_verification(&mut report);

    // Amendment 87: Proof enrichment
    validate_proof_strategy_coverage(&mut report);
    validate_proof_strategy_vocabulary(&mut report);
    validate_proof_dependency_acyclicity(&mut report);

    // Amendment 89: Identity property typing
    validate_identity_properties_typed(&mut report);

    // Amendment 90: String property cleanup
    validate_removed_properties_absent(&mut report);
    validate_legitimate_string_properties_only(&mut report);

    // Amendment 91: Identity individual AST rewrite
    validate_identity_values_are_irirefs(&mut report);

    // Amendment 92: Proof derivation typing
    validate_formal_derivation_typed(&mut report);

    // Amendment 95: Workstreams 1, 3, 4, 5, 6, 8
    validate_predicate_registry(&mut report);
    validate_constraint_completion(&mut report);
    validate_host_value_sort(&mut report);
    validate_boundary_map_registry(&mut report);
    validate_type_definition_coverage(&mut report);
    validate_operad_retyping(&mut report);

    // Validate the built JSON-LD artifact
    let json_path = artifacts.join("uor.foundation.jsonld");
    if !json_path.exists() {
        report.push(TestResult::fail(
            "ontology/inventory",
            "uor.foundation.jsonld not found in artifacts directory",
        ));
        return Ok(report);
    }

    let content = std::fs::read_to_string(&json_path)
        .with_context(|| format!("Failed to read {}", json_path.display()))?;

    let value: Value = serde_json::from_str(&content)
        .with_context(|| format!("Failed to parse {} as JSON", json_path.display()))?;

    validate_json_inventory(&value, &mut report);

    Ok(report)
}

/// Validates inventory counts directly from the spec (no file I/O).
fn validate_spec_counts(report: &mut ConformanceReport) {
    let ontology = uor_ontology::Ontology::full();

    let ns_count = ontology.namespaces.len();
    let class_count = ontology.class_count();
    let property_count = ontology.property_count();
    let individual_count = ontology.individual_count();

    check_count(
        report,
        "namespaces",
        ns_count,
        counts::NAMESPACES,
        "ontology/inventory",
    );
    check_count(
        report,
        "classes",
        class_count,
        counts::CLASSES,
        "ontology/inventory",
    );
    check_count(
        report,
        "properties",
        property_count,
        counts::PROPERTIES,
        "ontology/inventory",
    );
    check_count(
        report,
        "individuals",
        individual_count,
        counts::INDIVIDUALS,
        "ontology/inventory",
    );
}

/// Checks a count matches the expected value.
pub(crate) fn check_count(
    report: &mut ConformanceReport,
    label: &str,
    actual: usize,
    expected: usize,
    validator: &str,
) {
    if actual == expected {
        report.push(TestResult::pass(
            validator,
            format!("Correct {} count: {}", label, actual),
        ));
    } else {
        report.push(TestResult::fail(
            validator,
            format!(
                "Wrong {} count: expected {}, got {}",
                label, expected, actual
            ),
        ));
    }
}

/// Validates that namespace space annotations follow the 3/10/3 classification.
///
/// Expected: 3 Kernel (u, schema, op), 10 Bridge, 3 User (type, state, morphism).
fn validate_space_classification(report: &mut ConformanceReport) {
    let ontology = uor_ontology::Ontology::full();
    let validator = "ontology/inventory/space_classification";

    let kernel: Vec<_> = ontology
        .namespaces
        .iter()
        .filter(|m| m.namespace.space == Space::Kernel)
        .map(|m| m.namespace.prefix)
        .collect();
    let bridge: Vec<_> = ontology
        .namespaces
        .iter()
        .filter(|m| m.namespace.space == Space::Bridge)
        .map(|m| m.namespace.prefix)
        .collect();
    let user: Vec<_> = ontology
        .namespaces
        .iter()
        .filter(|m| m.namespace.space == Space::User)
        .map(|m| m.namespace.prefix)
        .collect();

    if kernel.len() == counts::KERNEL_NAMESPACES {
        report.push(TestResult::pass(
            validator,
            format!(
                "Correct kernel-space count: {} ({:?})",
                counts::KERNEL_NAMESPACES,
                kernel
            ),
        ));
    } else {
        report.push(TestResult::fail(
            validator,
            format!(
                "Wrong kernel-space count: expected {}, got {} ({:?})",
                counts::KERNEL_NAMESPACES,
                kernel.len(),
                kernel
            ),
        ));
    }

    if bridge.len() == counts::BRIDGE_NAMESPACES {
        report.push(TestResult::pass(
            validator,
            format!(
                "Correct bridge-space count: {} ({:?})",
                counts::BRIDGE_NAMESPACES,
                bridge
            ),
        ));
    } else {
        report.push(TestResult::fail(
            validator,
            format!(
                "Wrong bridge-space count: expected {}, got {} ({:?})",
                counts::BRIDGE_NAMESPACES,
                bridge.len(),
                bridge
            ),
        ));
    }

    if user.len() == counts::USER_NAMESPACES {
        report.push(TestResult::pass(
            validator,
            format!(
                "Correct user-space count: {} ({:?})",
                counts::USER_NAMESPACES,
                user
            ),
        ));
    } else {
        report.push(TestResult::fail(
            validator,
            format!(
                "Wrong user-space count: expected {}, got {} ({:?})",
                counts::USER_NAMESPACES,
                user.len(),
                user
            ),
        ));
    }
}

/// Validates that every named individual has the minimum required property assertions.
fn validate_individual_completeness(report: &mut ConformanceReport) {
    let ontology = uor_ontology::Ontology::full();
    let validator = "ontology/inventory/individual_completeness";

    // Define minimum required properties per individual
    let requirements: &[(&str, &[&str])] = &[
        // 10 operations: all require arity
        ("https://uor.foundation/op/neg", &["op/arity"]),
        ("https://uor.foundation/op/bnot", &["op/arity"]),
        ("https://uor.foundation/op/succ", &["op/arity"]),
        ("https://uor.foundation/op/pred", &["op/arity"]),
        ("https://uor.foundation/op/add", &["op/arity"]),
        ("https://uor.foundation/op/sub", &["op/arity"]),
        ("https://uor.foundation/op/mul", &["op/arity"]),
        ("https://uor.foundation/op/xor", &["op/arity"]),
        ("https://uor.foundation/op/and", &["op/arity"]),
        ("https://uor.foundation/op/or", &["op/arity"]),
        // criticalIdentity: lhs, rhs, forAll
        (
            "https://uor.foundation/op/criticalIdentity",
            &["op/lhs", "op/rhs", "op/forAll"],
        ),
        // D2n: generatedBy, presentation
        (
            "https://uor.foundation/op/D2n",
            &["op/generatedBy", "op/presentation"],
        ),
        // pi1, zero: value
        ("https://uor.foundation/schema/pi1", &["schema/value"]),
        ("https://uor.foundation/schema/zero", &["schema/value"]),
        // MetricAxis individuals: type assertion only (no required properties)
        ("https://uor.foundation/type/verticalAxis", &[]),
        ("https://uor.foundation/type/horizontalAxis", &[]),
        ("https://uor.foundation/type/diagonalAxis", &[]),
        // criticalComposition: lawComponents, lawResult
        (
            "https://uor.foundation/morphism/criticalComposition",
            &["morphism/lawComponents", "morphism/lawResult"],
        ),
        // AD_1: addressing bijection — lhs, rhs, forAll
        (
            "https://uor.foundation/op/AD_1",
            &["op/lhs", "op/rhs", "op/forAll"],
        ),
        // AD_2: embedding coherence — lhs, rhs, forAll
        (
            "https://uor.foundation/op/AD_2",
            &["op/lhs", "op/rhs", "op/forAll"],
        ),
        // nerveFunctorN: type assertion only
        ("https://uor.foundation/homology/nerveFunctorN", &[]),
        // chainFunctorC: type assertion only
        ("https://uor.foundation/homology/chainFunctorC", &[]),
    ];

    let mut all_found = true;

    for (iri, required_props) in requirements {
        match ontology.find_individual(iri) {
            Some(individual) => {
                for prop_suffix in *required_props {
                    let full_prop = format!("https://uor.foundation/{prop_suffix}");
                    let has_prop = individual.properties.iter().any(|(k, _)| *k == full_prop);
                    if !has_prop {
                        report.push(TestResult::fail(
                            validator,
                            format!("Individual {} missing required property {}", iri, full_prop),
                        ));
                        all_found = false;
                    }
                }
            }
            None => {
                report.push(TestResult::fail(
                    validator,
                    format!("Named individual {} not found in ontology", iri),
                ));
                all_found = false;
            }
        }
    }

    if all_found {
        report.push(TestResult::pass(
            validator,
            format!(
                "All {} named individuals have required property assertions",
                requirements.len()
            ),
        ));
    }
}

/// Validates that all `op:Identity` individuals have lhs, rhs, and forAll properties,
/// and that every expected algebra prefix group is represented.
fn validate_identity_completeness(report: &mut ConformanceReport) {
    let ontology = uor_ontology::Ontology::full();
    let validator = "ontology/inventory/identity_completeness";

    let op_module = match ontology.find_namespace("op") {
        Some(m) => m,
        None => {
            report.push(TestResult::fail(validator, "op namespace not found"));
            return;
        }
    };

    let identity_type = "https://uor.foundation/op/Identity";
    let identities: Vec<_> = op_module
        .individuals
        .iter()
        .filter(|i| i.type_ == identity_type)
        .collect();

    let mut all_valid = true;
    for ind in &identities {
        let has_lhs = ind
            .properties
            .iter()
            .any(|(k, _)| *k == "https://uor.foundation/op/lhs");
        let has_rhs = ind
            .properties
            .iter()
            .any(|(k, _)| *k == "https://uor.foundation/op/rhs");
        let has_forall = ind
            .properties
            .iter()
            .any(|(k, _)| *k == "https://uor.foundation/op/forAll");
        if !has_lhs || !has_rhs || !has_forall {
            report.push(TestResult::fail(
                validator,
                format!("Identity {} missing lhs/rhs/forAll", ind.id),
            ));
            all_valid = false;
        }
    }

    // Verify expected algebra prefix groups are all present
    let expected_prefixes = [
        "R_A", "R_M", "B_", "X_", "D_", "U_", "AG_", "CA_", "C_", "CDI", "CL_", "CM_", "CR_", "F_",
        "FL_", "FPM_", "FS_", "RE_", "IR_", "SF_", "RD_", "SE_", "OO_", "CB_", "OB_M", "OB_C",
        "OB_H", "OB_P", "CT_", "CF_", "HG_", "T_C", "T_I", "T_E", "T_A", "AU_", "EF_", "AD_",
        "AA_", "AM_", "TH_", "AR_", "PD_", "RC_", "DC_", "HA_", "IT_", "phi_", "psi_",
        // Amendment 25: Completeness Certification
        "CC_",  // Amendment 26: Quantum Level Scaling
        "QL_",  // Amendment 27: Session-Scoped Resolution
        "GR_",  // Amendment 28: Type Synthesis
        "TS_",  // Amendment 29: Witt Level Spectral Sequence
        "WLS_", // Amendment 30: Monodromy Observables
        "MN_",  // Amendment 31: Product/Sum Type identities
        "PT_",  // Amendment 31: Sum Type identities
        "ST_",  // Amendment 33: Grounded State Limit
        "GS_",  // Amendment 34: Morphospace Achievability
        "MS_",  // Amendment 35: Computational Geodesic
        "GD_",  // Amendment 36: Measurement Boundary
        "QM_",  // Amendment 37: SuperpositionResolver identities
        "SP_",  // Amendment 41: Tower identities
        "WT_",  // Amendment 44: Structural Gap Closures
        "jsat_", "EXP_", "GO_", "COEFF_",
        // Amendment 46: Certificate Issuance Coverage
        "CIC_", "GC_", // Amendment 48: Multi-Session Coordination
        "MC_", // Amendment 53: Witt-Carry Formalization
        "WC_", "OA_", // Amendment 54: Homotopy Nerve
        "HT_", // Amendment 55: Homotopy Pipeline
        "HP_", // Amendment 56: Moduli Space
        "MD_", // Amendment 57: Moduli Resolver
        "MR_", // Amendment 58: Carry Algebra
        "CY_", // Amendment 59: Named Base Metrics
        "BM_", // Amendment 60: Galois Connection + Nerve Operations
        "GL_", "NV_", // Amendment 61: Structural Types
        "SD_", // Amendment 62: Composed Operations
        "DD_", "PI_", "PA_", "PL_", "PK_", "PP_", // Amendment 63: Reduction Core
        "PE_", "PM_", "ER_", // Amendment 64: Reduction Expansion
        "EA_", "OE_", "CS_", // Amendment 65: Reduction Completion
        "FA_", "SW_", "LS_", "TJ_", "AP_", // Amendment 66: Convergence Tower
        "EC_", // Amendment 67: Division Algebras
        "DA_", // Amendment 68: Interaction Algebra
        "IN_", "AS_", // Amendment 69: Monoidal Composition
        "MO_", // Amendment 70: Operad Composition
        "OP_", // Amendment 71: Effect Algebra
        "FX_", // Amendment 72: Predicate & Dispatch
        "PR_", // Amendment 73: Reduction Guard + Resolver Dispatch
        "CG_", "DIS_", // Amendment 74: Parallel Composition
        "PAR_", // Amendment 75: Higher-Order + Streams
        "HO_", "STR_", // Amendment 76: Failure Algebra
        "FLR_", // Amendment 77: Linear + Subtyping
        "LN_", "SB_", // Amendment 78: Bounded Recursion
        "BR_", // Amendment 79: Address Regions
        "RG_", // Amendment 81: IO Boundary
        "IO_",
    ];
    for prefix in &expected_prefixes {
        let has = identities.iter().any(|i| i.label.starts_with(prefix));
        if !has {
            report.push(TestResult::fail(
                validator,
                format!("No identity with prefix {} found", prefix),
            ));
            all_valid = false;
        }
    }

    if all_valid {
        report.push(TestResult::pass(
            validator,
            format!("{} identity individuals validated", identities.len()),
        ));
    }
}

/// Validates that all `op:Identity` individuals are grounded with at least one
/// `verificationDomain` (IriRef to a domain individual) and optionally
/// `verificationPathNote`.
fn validate_identity_grounding(report: &mut ConformanceReport) {
    let ontology = uor_ontology::Ontology::full();
    let validator = "ontology/inventory/identity_grounding";

    let identity_type = "https://uor.foundation/op/Identity";
    let domain_prop = "https://uor.foundation/op/verificationDomain";
    let valid_domains = [
        "https://uor.foundation/op/Enumerative",
        "https://uor.foundation/op/Algebraic",
        "https://uor.foundation/op/Geometric",
        "https://uor.foundation/op/Analytical",
        "https://uor.foundation/op/Thermodynamic",
        "https://uor.foundation/op/Topological",
        "https://uor.foundation/op/Pipeline",
        "https://uor.foundation/op/IndexTheoretic",
        "https://uor.foundation/op/SuperpositionDomain",
        "https://uor.foundation/op/QuantumThermodynamic",
        "https://uor.foundation/op/ArithmeticValuation",
        "https://uor.foundation/op/ComposedAlgebraic",
    ];

    let mut total = 0usize;
    let mut all_valid = true;

    for module in &ontology.namespaces {
        for ind in &module.individuals {
            if ind.type_ != identity_type {
                continue;
            }
            total += 1;

            // Check verificationDomain (at least one, each from closed set)
            let domains: Vec<_> = ind
                .properties
                .iter()
                .filter(|(k, _)| *k == domain_prop)
                .collect();

            if domains.is_empty() {
                report.push(TestResult::fail(
                    validator,
                    format!("Identity {} missing verificationDomain", ind.id),
                ));
                all_valid = false;
            } else {
                for (_, v) in &domains {
                    if let uor_ontology::IndividualValue::IriRef(iri) = v {
                        if !valid_domains.contains(iri) {
                            report.push(TestResult::fail(
                                validator,
                                format!(
                                    "Identity {} has invalid verificationDomain: {}",
                                    ind.id, iri
                                ),
                            ));
                            all_valid = false;
                        }
                    } else {
                        report.push(TestResult::fail(
                            validator,
                            format!("Identity {} verificationDomain is not an IriRef", ind.id),
                        ));
                        all_valid = false;
                    }
                }
            }
        }
    }

    if all_valid && total > 0 {
        report.push(TestResult::pass(
            validator,
            format!("All {} identities grounded with verificationDomain", total),
        ));
    }
}

/// Validates the 9 VerificationDomain vocabulary individuals.
fn validate_verification_domain_individuals(report: &mut ConformanceReport) {
    let ontology = uor_ontology::Ontology::full();
    let validator = "ontology/inventory/verification_domain";

    let domain_iris = [
        "https://uor.foundation/op/Enumerative",
        "https://uor.foundation/op/Algebraic",
        "https://uor.foundation/op/Geometric",
        "https://uor.foundation/op/Analytical",
        "https://uor.foundation/op/Thermodynamic",
        "https://uor.foundation/op/Topological",
        "https://uor.foundation/op/Pipeline",
        "https://uor.foundation/op/IndexTheoretic",
        "https://uor.foundation/op/SuperpositionDomain",
        "https://uor.foundation/op/QuantumThermodynamic",
        "https://uor.foundation/op/ArithmeticValuation",
    ];

    let mut all_found = true;
    for iri in &domain_iris {
        if ontology.find_individual(iri).is_none() {
            report.push(TestResult::fail(
                validator,
                format!("VerificationDomain individual {} not found", iri),
            ));
            all_found = false;
        }
    }

    if all_found {
        report.push(TestResult::pass(
            validator,
            "All 11 VerificationDomain individuals present",
        ));
    }
}

/// Validates the 9 GeometricCharacter vocabulary individuals and that all
/// 10 operation individuals have `hasGeometricCharacter`.
fn validate_geometric_character_individuals(report: &mut ConformanceReport) {
    let ontology = uor_ontology::Ontology::full();
    let validator = "ontology/inventory/geometric_character";

    let gc_iris = [
        "https://uor.foundation/op/RingReflection",
        "https://uor.foundation/op/HypercubeReflection",
        "https://uor.foundation/op/Rotation",
        "https://uor.foundation/op/RotationInverse",
        "https://uor.foundation/op/Translation",
        "https://uor.foundation/op/Scaling",
        "https://uor.foundation/op/HypercubeTranslation",
        "https://uor.foundation/op/HypercubeProjection",
        "https://uor.foundation/op/HypercubeJoin",
    ];

    let mut all_valid = true;
    for iri in &gc_iris {
        if ontology.find_individual(iri).is_none() {
            report.push(TestResult::fail(
                validator,
                format!("GeometricCharacter individual {} not found", iri),
            ));
            all_valid = false;
        }
    }

    // Check all 10 operations have hasGeometricCharacter
    let gc_prop = "https://uor.foundation/op/hasGeometricCharacter";
    let op_types = ["Involution", "UnaryOp", "BinaryOp"];
    if let Some(op_module) = ontology.find_namespace("op") {
        for ind in &op_module.individuals {
            let type_local = ind.type_.rsplit('/').next().unwrap_or("");
            if op_types.contains(&type_local) {
                let has_gc = ind.properties.iter().any(|(k, _)| *k == gc_prop);
                if !has_gc {
                    report.push(TestResult::fail(
                        validator,
                        format!("Operation {} missing hasGeometricCharacter", ind.id),
                    ));
                    all_valid = false;
                }
            }
        }
    }

    if all_valid {
        report.push(TestResult::pass(
            validator,
            "All 9 GeometricCharacter individuals + 10 operations validated",
        ));
    }
}

/// Validates the 4 ComplexityClass vocabulary individuals.
fn validate_complexity_class_individuals(report: &mut ConformanceReport) {
    let ontology = uor_ontology::Ontology::full();
    let validator = "ontology/inventory/complexity_class";

    let cc_iris = [
        "https://uor.foundation/resolver/ConstantTime",
        "https://uor.foundation/resolver/LogarithmicTime",
        "https://uor.foundation/resolver/LinearTime",
        "https://uor.foundation/resolver/ExponentialTime",
    ];

    let mut all_found = true;
    for iri in &cc_iris {
        if ontology.find_individual(iri).is_none() {
            report.push(TestResult::fail(
                validator,
                format!("ComplexityClass individual {} not found", iri),
            ));
            all_found = false;
        }
    }

    if all_found {
        report.push(TestResult::pass(
            validator,
            "All 4 ComplexityClass individuals present",
        ));
    }
}

/// Validates the 6 RewriteRule vocabulary individuals and CriticalIdentityRule
/// `groundedIn` assertion.
fn validate_rewrite_rule_individuals(report: &mut ConformanceReport) {
    let ontology = uor_ontology::Ontology::full();
    let validator = "ontology/inventory/rewrite_rule";

    let rr_iris = [
        "https://uor.foundation/derivation/CriticalIdentityRule",
        "https://uor.foundation/derivation/InvolutionRule",
        "https://uor.foundation/derivation/AssociativityRule",
        "https://uor.foundation/derivation/CommutativityRule",
        "https://uor.foundation/derivation/IdentityElementRule",
        "https://uor.foundation/derivation/NormalizationRule",
    ];

    let mut all_valid = true;
    for iri in &rr_iris {
        if ontology.find_individual(iri).is_none() {
            report.push(TestResult::fail(
                validator,
                format!("RewriteRule individual {} not found", iri),
            ));
            all_valid = false;
        }
    }

    // Check CriticalIdentityRule has groundedIn → criticalIdentity
    if let Some(cir) =
        ontology.find_individual("https://uor.foundation/derivation/CriticalIdentityRule")
    {
        let has_grounded = cir.properties.iter().any(|(k, v)| {
            *k == "https://uor.foundation/derivation/groundedIn"
                && matches!(
                    v,
                    uor_ontology::IndividualValue::IriRef(
                        "https://uor.foundation/op/criticalIdentity"
                    )
                )
        });
        if !has_grounded {
            report.push(TestResult::fail(
                validator,
                "CriticalIdentityRule missing groundedIn → op:criticalIdentity",
            ));
            all_valid = false;
        }
    }

    if all_valid {
        report.push(TestResult::pass(
            validator,
            "All 6 RewriteRule individuals + groundedIn assertion validated",
        ));
    }
}

/// Validates the 4 MeasurementUnit vocabulary individuals.
fn validate_measurement_unit_individuals(report: &mut ConformanceReport) {
    let ontology = uor_ontology::Ontology::full();
    let validator = "ontology/inventory/measurement_unit";

    let mu_iris = [
        "https://uor.foundation/observable/Bits",
        "https://uor.foundation/observable/RingSteps",
        "https://uor.foundation/observable/Dimensionless",
        "https://uor.foundation/observable/Nats",
    ];

    let mut all_found = true;
    for iri in &mu_iris {
        if ontology.find_individual(iri).is_none() {
            report.push(TestResult::fail(
                validator,
                format!("MeasurementUnit individual {} not found", iri),
            ));
            all_found = false;
        }
    }

    if all_found {
        report.push(TestResult::pass(
            validator,
            "All 4 MeasurementUnit individuals present",
        ));
    }
}

/// Validates the 3 TriadProjection vocabulary individuals.
fn validate_coordinate_kind_individuals(report: &mut ConformanceReport) {
    let ontology = uor_ontology::Ontology::full();
    let validator = "ontology/inventory/coordinate_kind";

    let ck_iris = [
        "https://uor.foundation/query/TwoAdicValuation",
        "https://uor.foundation/query/WalshHadamardImage",
        "https://uor.foundation/query/Address",
    ];

    let mut all_found = true;
    for iri in &ck_iris {
        if ontology.find_individual(iri).is_none() {
            report.push(TestResult::fail(
                validator,
                format!("TriadProjection individual {} not found", iri),
            ));
            all_found = false;
        }
    }

    if all_found {
        report.push(TestResult::pass(
            validator,
            "All 3 TriadProjection individuals present",
        ));
    }
}

/// Validates that every `op:Identity` individual has a corresponding proof individual
/// with `proof:provesIdentity` linking to it.
fn validate_proof_coverage(report: &mut ConformanceReport) {
    let ontology = uor_ontology::Ontology::full();
    let validator = "ontology/inventory/proof_coverage";

    let identity_type = "https://uor.foundation/op/Identity";
    let proves_prop = "https://uor.foundation/proof/provesIdentity";

    // Collect all identity IRIs across all namespaces
    let mut identity_iris: Vec<&str> = Vec::new();
    for module in &ontology.namespaces {
        for ind in &module.individuals {
            if ind.type_ == identity_type {
                identity_iris.push(ind.id);
            }
        }
    }

    // Collect all IRIs that are targets of proof:provesIdentity
    let mut proved_iris: HashSet<&str> = HashSet::new();
    if let Some(proof_module) = ontology.find_namespace("proof") {
        for ind in &proof_module.individuals {
            for (k, v) in ind.properties {
                if *k == proves_prop {
                    if let uor_ontology::IndividualValue::IriRef(iri) = v {
                        proved_iris.insert(iri);
                    }
                }
            }
        }
    }

    let uncovered: Vec<_> = identity_iris
        .iter()
        .filter(|iri| !proved_iris.contains(**iri))
        .collect();

    if uncovered.is_empty() {
        report.push(TestResult::pass(
            validator,
            format!(
                "All {} identities covered by proof individuals",
                identity_iris.len()
            ),
        ));
    } else {
        report.push(TestResult::fail(
            validator,
            format!(
                "{} identities not covered by proof individuals: {:?}",
                uncovered.len(),
                &uncovered[..uncovered.len().min(5)]
            ),
        ));
    }
}

/// Validates quantum scope consistency: `ComputationCertificate` individuals
/// must have `atWittLevel` and must NOT have `universalScope`;
/// `AxiomaticDerivation` individuals must have `universalScope` and must NOT
/// have `atWittLevel`.
fn validate_quantum_scope(report: &mut ConformanceReport) {
    let ontology = uor_ontology::Ontology::full();
    let validator = "ontology/inventory/quantum_scope";

    let cert_type = "https://uor.foundation/proof/ComputationCertificate";
    let crit_type = "https://uor.foundation/proof/CriticalIdentityProof";
    let axiomatic_type = "https://uor.foundation/proof/AxiomaticDerivation";
    let inductive_type = "https://uor.foundation/proof/InductiveProof";
    let at_ql_prop = "https://uor.foundation/proof/atWittLevel";
    let univ_prop = "https://uor.foundation/proof/universalScope";

    let mut all_valid = true;
    let mut cert_count = 0usize;
    let mut axiomatic_count = 0usize;
    let mut inductive_count = 0usize;

    if let Some(proof_module) = ontology.find_namespace("proof") {
        for ind in &proof_module.individuals {
            let is_cert = ind.type_ == cert_type || ind.type_ == crit_type;
            let is_axiomatic = ind.type_ == axiomatic_type;
            let is_inductive = ind.type_ == inductive_type;

            if is_cert {
                cert_count += 1;
                let has_ql = ind.properties.iter().any(|(k, _)| *k == at_ql_prop);
                let has_univ = ind.properties.iter().any(|(k, _)| *k == univ_prop);
                if !has_ql {
                    report.push(TestResult::fail(
                        validator,
                        format!("ComputationCertificate {} missing atWittLevel", ind.id),
                    ));
                    all_valid = false;
                }
                if has_univ {
                    report.push(TestResult::fail(
                        validator,
                        format!(
                            "ComputationCertificate {} should not have universalScope",
                            ind.id
                        ),
                    ));
                    all_valid = false;
                }
            }

            if is_axiomatic {
                axiomatic_count += 1;
                let has_univ = ind.properties.iter().any(|(k, _)| *k == univ_prop);
                let has_ql = ind.properties.iter().any(|(k, _)| *k == at_ql_prop);
                if !has_univ {
                    report.push(TestResult::fail(
                        validator,
                        format!("AxiomaticDerivation {} missing universalScope", ind.id),
                    ));
                    all_valid = false;
                }
                if has_ql {
                    report.push(TestResult::fail(
                        validator,
                        format!("AxiomaticDerivation {} should not have atWittLevel", ind.id),
                    ));
                    all_valid = false;
                }
            }

            if is_inductive {
                inductive_count += 1;
                let has_univ = ind.properties.iter().any(|(k, _)| *k == univ_prop);
                let has_ql = ind.properties.iter().any(|(k, _)| *k == at_ql_prop);
                if !has_univ {
                    report.push(TestResult::fail(
                        validator,
                        format!("InductiveProof {} missing universalScope", ind.id),
                    ));
                    all_valid = false;
                }
                if has_ql {
                    report.push(TestResult::fail(
                        validator,
                        format!("InductiveProof {} should not have atWittLevel", ind.id),
                    ));
                    all_valid = false;
                }
            }

            // Amendment 47/86: Exhaustive type guard — flag unrecognized proof types
            // (EmpiricalVerification branch removed in Amendment 86)
            let known_exempt = [
                "https://uor.foundation/proof/ImpossibilityWitness",
                "https://uor.foundation/proof/MorphospaceRecord",
                "https://uor.foundation/proof/ProofStrategy",
            ];
            if !is_cert && !is_axiomatic && !is_inductive && !known_exempt.contains(&ind.type_) {
                report.push(TestResult::fail(
                    validator,
                    format!(
                        "Proof individual {} has unrecognized type {} \
                         — add validation branch or exemption",
                        ind.id, ind.type_
                    ),
                ));
                all_valid = false;
            }
        }
    }

    if all_valid && (cert_count + axiomatic_count + inductive_count) > 0 {
        report.push(TestResult::pass(
            validator,
            format!(
                "Quantum scope valid: {} computation certificates, \
                 {} axiomatic derivations, {} inductive proofs",
                cert_count, axiomatic_count, inductive_count
            ),
        ));
    }
}

/// Returns true if a JSON-LD node's `@type` field contains the given value.
///
/// Handles both string and array forms of `@type`.
fn node_has_type(node: &Value, target: &str) -> bool {
    match node.get("@type") {
        Some(Value::String(t)) => t == target,
        Some(Value::Array(types)) => types.iter().any(|t| t.as_str() == Some(target)),
        _ => false,
    }
}

/// Validates inventory counts from the JSON-LD graph.
fn validate_json_inventory(value: &Value, report: &mut ConformanceReport) {
    let graph = match value.get("@graph").and_then(|g| g.as_array()) {
        Some(g) => g,
        None => {
            report.push(TestResult::fail(
                "ontology/inventory",
                "JSON-LD artifact missing @graph array",
            ));
            return;
        }
    };

    // Count classes (type owl:Class)
    let class_count = graph
        .iter()
        .filter(|node| node_has_type(node, "owl:Class"))
        .count();

    // Count properties (owl:DatatypeProperty | owl:ObjectProperty | owl:AnnotationProperty)
    let property_count = graph
        .iter()
        .filter(|node| {
            node_has_type(node, "owl:DatatypeProperty")
                || node_has_type(node, "owl:ObjectProperty")
                || node_has_type(node, "owl:AnnotationProperty")
        })
        .count();

    // Count named individuals (owl:NamedIndividual)
    let individual_count = graph
        .iter()
        .filter(|node| node_has_type(node, "owl:NamedIndividual"))
        .count();

    check_count(
        report,
        "classes (JSON-LD)",
        class_count,
        counts::CLASSES,
        "ontology/inventory",
    );
    check_count(
        report,
        "properties (JSON-LD)",
        property_count,
        counts::PROPERTIES,
        "ontology/inventory",
    );
    check_count(
        report,
        "individuals (JSON-LD)",
        individual_count,
        counts::INDIVIDUALS,
        "ontology/inventory",
    );
}

/// Validates the Surface Symmetry identity and its proof individual.
fn validate_surface_symmetry_identity(report: &mut ConformanceReport) {
    let ontology = uor_ontology::Ontology::full();
    let validator = "ontology/inventory/surface_symmetry";

    let identity_iri = "https://uor.foundation/op/surfaceSymmetry";
    let proof_iri = "https://uor.foundation/proof/prf_surfaceSymmetry";

    let mut all_found = true;

    if ontology.find_individual(identity_iri).is_none() {
        report.push(TestResult::fail(
            validator,
            "op:surfaceSymmetry individual not found",
        ));
        all_found = false;
    }

    if ontology.find_individual(proof_iri).is_none() {
        report.push(TestResult::fail(
            validator,
            "proof:prf_surfaceSymmetry individual not found",
        ));
        all_found = false;
    }

    if all_found {
        report.push(TestResult::pass(
            validator,
            "op:surfaceSymmetry and proof:prf_surfaceSymmetry present",
        ));
    }
}

/// Validates the 3 GroundingPhase vocabulary individuals.
fn validate_saturation_phase_individuals(report: &mut ConformanceReport) {
    let ontology = uor_ontology::Ontology::full();
    let validator = "ontology/inventory/saturation_phase";

    let sp_iris = [
        "https://uor.foundation/state/Open",
        "https://uor.foundation/state/PartialGrounding",
        "https://uor.foundation/state/FullGrounding",
    ];

    let mut all_found = true;
    for iri in &sp_iris {
        if ontology.find_individual(iri).is_none() {
            report.push(TestResult::fail(
                validator,
                format!("GroundingPhase individual {} not found", iri),
            ));
            all_found = false;
        }
    }

    if all_found {
        report.push(TestResult::pass(
            validator,
            "All 3 GroundingPhase individuals present",
        ));
    }
}

/// Validates the 2 AchievabilityStatus vocabulary individuals.
fn validate_achievability_status_individuals(report: &mut ConformanceReport) {
    let ontology = uor_ontology::Ontology::full();
    let validator = "ontology/inventory/achievability_status";

    let as_iris = [
        "https://uor.foundation/observable/Achievable",
        "https://uor.foundation/observable/Forbidden",
    ];

    let mut all_found = true;
    for iri in &as_iris {
        if ontology.find_individual(iri).is_none() {
            report.push(TestResult::fail(
                validator,
                format!("AchievabilityStatus individual {} not found", iri),
            ));
            all_found = false;
        }
    }

    if all_found {
        report.push(TestResult::pass(
            validator,
            "All 2 AchievabilityStatus individuals present",
        ));
    }
}

fn validate_superposition_identity_coverage(report: &mut ConformanceReport) {
    let ontology = uor_ontology::Ontology::full();
    let validator = "ontology/inventory/superposition_identity";
    let sp_ids = [
        "https://uor.foundation/op/SP_1",
        "https://uor.foundation/op/SP_2",
        "https://uor.foundation/op/SP_3",
        "https://uor.foundation/op/SP_4",
    ];
    let mut all_found = true;
    for iri in &sp_ids {
        if ontology.find_individual(iri).is_none() {
            report.push(TestResult::fail(
                validator,
                format!("SP identity {} not found", iri),
            ));
            all_found = false;
        }
    }
    if all_found {
        report.push(TestResult::pass(validator, "All 4 SP_ identities present"));
    }
}

fn validate_partition_tensor_product(report: &mut ConformanceReport) {
    let ontology = uor_ontology::Ontology::full();
    let validator = "ontology/inventory/partition_tensor";
    let class_iris = [
        "https://uor.foundation/partition/PartitionProduct",
        "https://uor.foundation/partition/PartitionCoproduct",
    ];
    let mut all_found = true;
    for iri in &class_iris {
        if ontology.find_class(iri).is_none() {
            report.push(TestResult::fail(
                validator,
                format!("Class {} not found", iri),
            ));
            all_found = false;
        }
    }
    if all_found {
        report.push(TestResult::pass(
            validator,
            "PartitionProduct and PartitionCoproduct classes present",
        ));
    }
}

fn validate_geodesic_decomposition(report: &mut ConformanceReport) {
    let ontology = uor_ontology::Ontology::full();
    let validator = "ontology/inventory/geodesic_decomposition";
    if ontology
        .find_individual("https://uor.foundation/op/GD_6")
        .is_some()
    {
        report.push(TestResult::pass(
            validator,
            "GD_6 geodesic decomposition identity present",
        ));
    } else {
        report.push(TestResult::fail(validator, "GD_6 identity not found"));
    }
}

fn validate_born_rule_coverage(report: &mut ConformanceReport) {
    let ontology = uor_ontology::Ontology::full();
    let validator = "ontology/inventory/born_rule";
    let mut all_found = true;
    if ontology
        .find_individual("https://uor.foundation/op/QM_5")
        .is_none()
    {
        report.push(TestResult::fail(validator, "QM_5 identity not found"));
        all_found = false;
    }
    if ontology
        .find_class("https://uor.foundation/cert/BornRuleVerification")
        .is_none()
    {
        report.push(TestResult::fail(
            validator,
            "BornRuleVerification class not found",
        ));
        all_found = false;
    }
    if all_found {
        report.push(TestResult::pass(
            validator,
            "QM_5 identity and BornRuleVerification class present",
        ));
    }
}

fn validate_enum_variant_alignment(report: &mut ConformanceReport) {
    let ontology = uor_ontology::Ontology::full();
    let validator = "ontology/inventory/enum_variant";
    let domain_iris = [
        "https://uor.foundation/op/Enumerative",
        "https://uor.foundation/op/Algebraic",
        "https://uor.foundation/op/Geometric",
        "https://uor.foundation/op/Analytical",
        "https://uor.foundation/op/Thermodynamic",
        "https://uor.foundation/op/Topological",
        "https://uor.foundation/op/Pipeline",
        "https://uor.foundation/op/IndexTheoretic",
        "https://uor.foundation/op/SuperpositionDomain",
        "https://uor.foundation/op/QuantumThermodynamic",
        "https://uor.foundation/op/ArithmeticValuation",
    ];
    let ev_prop = "https://uor.foundation/op/enumVariant";
    let mut all_valid = true;
    for iri in &domain_iris {
        if let Some(ind) = ontology.find_individual(iri) {
            let has_ev = ind.properties.iter().any(|(k, _)| *k == ev_prop);
            if !has_ev {
                report.push(TestResult::fail(
                    validator,
                    format!("VerificationDomain {} missing enumVariant", iri),
                ));
                all_valid = false;
            }
        }
    }
    if all_valid {
        report.push(TestResult::pass(
            validator,
            "All 11 VerificationDomain individuals have enumVariant",
        ));
    }
}

/// Validates W16Ring class exists (Amendment 39).
fn validate_q1_ring_grounding(report: &mut ConformanceReport) {
    let ontology = uor_ontology::Ontology::full();
    let validator = "ontology/inventory/q1_ring_grounding";
    let w16ring = "https://uor.foundation/schema/W16Ring";
    if ontology.find_class(w16ring).is_some() {
        report.push(TestResult::pass(validator, "W16Ring class present"));
    } else {
        report.push(TestResult::fail(validator, "W16Ring class not found"));
    }
}

/// Validates WittLift and LiftObstruction classes exist (Amendment 39).
fn validate_lift_obstruction_paths(report: &mut ConformanceReport) {
    let ontology = uor_ontology::Ontology::full();
    let validator = "ontology/inventory/lift_obstruction_paths";
    let class_iris = [
        "https://uor.foundation/type/WittLift",
        "https://uor.foundation/type/LiftObstruction",
    ];
    let mut all_found = true;
    for iri in &class_iris {
        if ontology.find_class(iri).is_none() {
            report.push(TestResult::fail(
                validator,
                format!("Class {} not found", iri),
            ));
            all_found = false;
        }
    }
    if all_found {
        report.push(TestResult::pass(
            validator,
            "WittLift and LiftObstruction classes present",
        ));
    }
}

/// Validates SynthesisCheckpoint class and properties exist (Amendment 38).
fn validate_synthesis_q1_coverage(report: &mut ConformanceReport) {
    let ontology = uor_ontology::Ontology::full();
    let validator = "ontology/inventory/synthesis_q1_coverage";
    let class_iri = "https://uor.foundation/derivation/SynthesisCheckpoint";
    let prop_iris = [
        "https://uor.foundation/derivation/checkpointStep",
        "https://uor.foundation/derivation/checkpointState",
    ];
    let class_found = ontology.find_class(class_iri).is_some();
    let props_found = prop_iris
        .iter()
        .all(|iri| ontology.find_property(iri).is_some());
    if class_found && props_found {
        report.push(TestResult::pass(
            validator,
            "SynthesisCheckpoint class and properties present",
        ));
    } else {
        report.push(TestResult::fail(
            validator,
            "SynthesisCheckpoint class or properties missing",
        ));
    }
}

/// Validates isAR1Ordered and isDC10Selected properties exist (Amendment 38).
fn validate_evidence_bundle_properties(report: &mut ConformanceReport) {
    let ontology = uor_ontology::Ontology::full();
    let validator = "ontology/inventory/evidence_bundle_properties";
    let prop_iris = [
        "https://uor.foundation/cert/isAR1Ordered",
        "https://uor.foundation/cert/isDC10Selected",
    ];
    let mut all_found = true;
    for iri in &prop_iris {
        if ontology.find_property(iri).is_none() {
            report.push(TestResult::fail(
                validator,
                format!("Property {} not found", iri),
            ));
            all_found = false;
        }
    }
    if all_found {
        report.push(TestResult::pass(
            validator,
            "isAR1Ordered and isDC10Selected properties present",
        ));
    }
}

/// Validates the normative cert chain is connected (Amendment 40).
fn validate_normative_chain_integrity(report: &mut ConformanceReport) {
    let ontology = uor_ontology::Ontology::full();
    let validator = "ontology/inventory/normative_chain_integrity";
    let required = [
        "https://uor.foundation/cert/certifiedGeodesic",
        "https://uor.foundation/cert/evidenceBundle",
        "https://uor.foundation/cert/isAR1Ordered",
        "https://uor.foundation/cert/isDC10Selected",
    ];
    let all_found = required
        .iter()
        .all(|iri| ontology.find_property(iri).is_some());
    if all_found {
        report.push(TestResult::pass(
            validator,
            "Normative chain properties all present",
        ));
    } else {
        report.push(TestResult::fail(
            validator,
            "Normative chain incomplete: missing cert/ properties",
        ));
    }
}

/// Validates workspace-level inventory invariants (requires workspace root path).
///
/// Currently checks that the SHACL shapes file has exactly one NodeShape per class.
///
/// Validates that the 4 ValidityScopeKind individuals exist (Amendment 41).
fn validate_validity_scope_individuals(report: &mut ConformanceReport) {
    let ontology = uor_ontology::Ontology::full();
    let validator = "ontology/inventory/validity_scope_individuals";
    let scope_type = "https://uor.foundation/op/ValidityScopeKind";
    let expected = [
        "Universal",
        "ParametricLower",
        "ParametricRange",
        "LevelSpecific",
    ];

    if let Some(op_module) = ontology.find_namespace("op") {
        let scope_inds: Vec<&str> = op_module
            .individuals
            .iter()
            .filter(|i| i.type_ == scope_type)
            .map(|i| i.label)
            .collect();
        let mut all_found = true;
        for name in &expected {
            if !scope_inds.contains(name) {
                report.push(TestResult::fail(
                    validator,
                    format!("Missing ValidityScopeKind individual: {}", name),
                ));
                all_found = false;
            }
        }
        if all_found {
            report.push(TestResult::pass(
                validator,
                format!(
                    "All {} ValidityScopeKind individuals present",
                    expected.len()
                ),
            ));
        }
    } else {
        report.push(TestResult::fail(validator, "op/ namespace not found"));
    }
}

/// Validates Amendment 41 tower chain vocabulary exists.
fn validate_tower_chain_vocabulary(report: &mut ConformanceReport) {
    let ontology = uor_ontology::Ontology::full();
    let validator = "ontology/inventory/tower_chain_vocabulary";

    let expected_classes = [
        "https://uor.foundation/type/LiftChain",
        "https://uor.foundation/type/ObstructionChain",
        "https://uor.foundation/cert/LiftChainCertificate",
        "https://uor.foundation/cert/ChainAuditTrail",
        "https://uor.foundation/resolver/TowerCompletenessResolver",
        "https://uor.foundation/proof/InductiveProof",
        "https://uor.foundation/op/ValidityScopeKind",
    ];

    let all_classes: Vec<&str> = ontology
        .namespaces
        .iter()
        .flat_map(|m| m.classes.iter().map(|c| c.id))
        .collect();

    let mut all_found = true;
    for cls in &expected_classes {
        if !all_classes.contains(cls) {
            report.push(TestResult::fail(
                validator,
                format!("Missing Amendment 41 class: {}", cls),
            ));
            all_found = false;
        }
    }

    // Check WT_1 through WT_7 identities exist
    if let Some(op_module) = ontology.find_namespace("op") {
        for i in 1..=7 {
            let label = format!("WT_{}", i);
            let has = op_module
                .individuals
                .iter()
                .any(|ind| ind.label == label.as_str());
            if !has {
                report.push(TestResult::fail(
                    validator,
                    format!("Missing tower identity: {}", label),
                ));
                all_found = false;
            }
        }
    }

    if all_found {
        report.push(TestResult::pass(
            validator,
            "All Amendment 41 tower chain vocabulary present (7 classes, 7 WT_ identities)",
        ));
    }
}

/// Validates that the EBNF grammar operations align with the ontology's
/// PrimitiveOp individuals and WittLevel individuals.
fn validate_ebnf_grammar_alignment(report: &mut ConformanceReport) {
    let ontology = uor_ontology::Ontology::full();
    let validator = "ontology/inventory/ebnf_grammar_alignment";

    // The EBNF grammar references 15 PrimitiveOp operations (10 original +
    // 5 ADR-013/TR-08 substrate-amendment ops: Le, Lt, Ge, Gt, Concat) and
    // 4 Witt levels.
    let expected_ops = [
        "neg", "bnot", "succ", "pred", "add", "sub", "mul", "xor", "and", "or", "le", "lt", "ge",
        "gt", "concat",
    ];
    let expected_levels = ["W8", "W16", "W24", "W32"];

    let op_ns = ontology
        .namespaces
        .iter()
        .find(|m| m.namespace.prefix == "op");
    let schema_ns = ontology
        .namespaces
        .iter()
        .find(|m| m.namespace.prefix == "schema");

    let mut all_found = true;

    if let Some(op_mod) = op_ns {
        for op_name in &expected_ops {
            let iri = format!("https://uor.foundation/op/{}", op_name);
            if !op_mod.individuals.iter().any(|ind| ind.id == iri.as_str()) {
                report.push(TestResult::fail(
                    validator,
                    format!(
                        "EBNF grammar references op:{} but individual not found",
                        op_name
                    ),
                ));
                all_found = false;
            }
        }
    } else {
        report.push(TestResult::fail(validator, "op/ namespace not found"));
        return;
    }

    if let Some(schema_mod) = schema_ns {
        for level in &expected_levels {
            let iri = format!("https://uor.foundation/schema/{}", level);
            if !schema_mod
                .individuals
                .iter()
                .any(|ind| ind.id == iri.as_str())
            {
                report.push(TestResult::fail(
                    validator,
                    format!(
                        "EBNF grammar references schema:{} but individual not found",
                        level
                    ),
                ));
                all_found = false;
            }
        }
    } else {
        report.push(TestResult::fail(validator, "schema/ namespace not found"));
        return;
    }

    if all_found {
        report.push(TestResult::pass(
            validator,
            "EBNF grammar operations and quantum levels align with ontology (10 ops, 4 levels)",
        ));
    }
}

/// Validates that Amendment 43 cryptographic primitive pinning properties
/// exist with correct domain, range, and functionality.
fn validate_crypto_pinning_vocabulary(report: &mut ConformanceReport) {
    let ontology = uor_ontology::Ontology::full();
    let validator = "ontology/inventory/crypto_pinning_vocabulary";

    let u_ns = ontology
        .namespaces
        .iter()
        .find(|m| m.namespace.prefix == "u");

    let Some(u_mod) = u_ns else {
        report.push(TestResult::fail(validator, "u/ namespace not found"));
        return;
    };

    let mut all_valid = true;

    // Check u:digestAlgorithm
    let digest_alg = u_mod
        .properties
        .iter()
        .find(|p| p.id == "https://uor.foundation/u/digestAlgorithm");
    if let Some(prop) = digest_alg {
        if prop.domain != Some("https://uor.foundation/u/Element") {
            report.push(TestResult::fail(
                validator,
                "u:digestAlgorithm has incorrect domain",
            ));
            all_valid = false;
        }
        if !prop.functional {
            report.push(TestResult::fail(
                validator,
                "u:digestAlgorithm must be functional",
            ));
            all_valid = false;
        }
    } else {
        report.push(TestResult::fail(
            validator,
            "u:digestAlgorithm property not found",
        ));
        all_valid = false;
    }

    // Check u:canonicalBytes
    let canonical = u_mod
        .properties
        .iter()
        .find(|p| p.id == "https://uor.foundation/u/canonicalBytes");
    if let Some(prop) = canonical {
        if prop.domain != Some("https://uor.foundation/u/Element") {
            report.push(TestResult::fail(
                validator,
                "u:canonicalBytes has incorrect domain",
            ));
            all_valid = false;
        }
        if prop.range != "http://www.w3.org/2001/XMLSchema#hexBinary" {
            report.push(TestResult::fail(
                validator,
                "u:canonicalBytes must have range xsd:hexBinary",
            ));
            all_valid = false;
        }
        if !prop.functional {
            report.push(TestResult::fail(
                validator,
                "u:canonicalBytes must be functional",
            ));
            all_valid = false;
        }
    } else {
        report.push(TestResult::fail(
            validator,
            "u:canonicalBytes property not found",
        ));
        all_valid = false;
    }

    if all_valid {
        report.push(TestResult::pass(
            validator,
            "Amendment 43 crypto pinning vocabulary present (digestAlgorithm, canonicalBytes)",
        ));
    }
}

/// Validates carry constraint site-binding identities (Amendment 44, G7).
fn validate_carry_constraint_pinning(report: &mut ConformanceReport) {
    let ontology = uor_ontology::Ontology::full();
    let validator = "ontology/inventory/carry_constraint_pinning";
    let op_ns = ontology
        .namespaces
        .iter()
        .find(|m| m.namespace.prefix == "op");
    let Some(op_mod) = op_ns else {
        report.push(TestResult::fail(validator, "op/ namespace not found"));
        return;
    };
    let expected = ["CC_PINS", "CC_COST_SITE"];
    let mut all_found = true;
    for label in &expected {
        let has = op_mod.individuals.iter().any(|ind| ind.label == *label);
        if !has {
            report.push(TestResult::fail(
                validator,
                format!("Missing carry constraint pinning identity: {}", label),
            ));
            all_found = false;
        }
    }
    if all_found {
        report.push(TestResult::pass(
            validator,
            "Amendment 44 carry constraint pinning identities present (CC_PINS, CC_COST_SITE)",
        ));
    }
}

/// Validates nerve joint satisfiability identities (Amendment 44, G1).
fn validate_joint_satisfiability_coverage(report: &mut ConformanceReport) {
    let ontology = uor_ontology::Ontology::full();
    let validator = "ontology/inventory/joint_satisfiability_coverage";
    let op_ns = ontology
        .namespaces
        .iter()
        .find(|m| m.namespace.prefix == "op");
    let Some(op_mod) = op_ns else {
        report.push(TestResult::fail(validator, "op/ namespace not found"));
        return;
    };
    let expected = ["jsat_RR", "jsat_CR", "jsat_CC"];
    let mut all_found = true;
    for label in &expected {
        let has = op_mod.individuals.iter().any(|ind| ind.label == *label);
        if !has {
            report.push(TestResult::fail(
                validator,
                format!("Missing joint satisfiability identity: {}", label),
            ));
            all_found = false;
        }
    }
    if all_found {
        report.push(TestResult::pass(
            validator,
            "Amendment 44 joint satisfiability identities present (jsat_RR, jsat_CR, jsat_CC)",
        ));
    }
}

/// Validates dihedral algebra completeness (Amendment 44, G2).
fn validate_dihedral_algebra_completeness(report: &mut ConformanceReport) {
    let ontology = uor_ontology::Ontology::full();
    let validator = "ontology/inventory/dihedral_algebra_completeness";
    let op_ns = ontology
        .namespaces
        .iter()
        .find(|m| m.namespace.prefix == "op");
    let Some(op_mod) = op_ns else {
        report.push(TestResult::fail(validator, "op/ namespace not found"));
        return;
    };
    let expected = ["D_7", "D_8", "D_9"];
    let mut all_found = true;
    for label in &expected {
        let has = op_mod.individuals.iter().any(|ind| ind.label == *label);
        if !has {
            report.push(TestResult::fail(
                validator,
                format!("Missing dihedral algebra identity: {}", label),
            ));
            all_found = false;
        }
    }
    if all_found {
        report.push(TestResult::pass(
            validator,
            "Amendment 44 dihedral algebra identities present (D_7, D_8, D_9)",
        ));
    }
}

/// Validates constraint expressiveness boundary identities (Amendment 44, G5).
fn validate_constraint_expressiveness(report: &mut ConformanceReport) {
    let ontology = uor_ontology::Ontology::full();
    let validator = "ontology/inventory/constraint_expressiveness";
    let op_ns = ontology
        .namespaces
        .iter()
        .find(|m| m.namespace.prefix == "op");
    let Some(op_mod) = op_ns else {
        report.push(TestResult::fail(validator, "op/ namespace not found"));
        return;
    };
    let expected = ["EXP_1", "EXP_2", "EXP_3"];
    let mut all_found = true;
    for label in &expected {
        let has = op_mod.individuals.iter().any(|ind| ind.label == *label);
        if !has {
            report.push(TestResult::fail(
                validator,
                format!("Missing constraint expressiveness identity: {}", label),
            ));
            all_found = false;
        }
    }
    if all_found {
        report.push(TestResult::pass(
            validator,
            "Amendment 44 constraint expressiveness identities present (EXP_1, EXP_2, EXP_3)",
        ));
    }
}

/// Validates SumType topological identity algebra (Amendment 44, G4).
fn validate_sumtype_topology(report: &mut ConformanceReport) {
    let ontology = uor_ontology::Ontology::full();
    let validator = "ontology/inventory/sumtype_topology";
    let op_ns = ontology
        .namespaces
        .iter()
        .find(|m| m.namespace.prefix == "op");
    let Some(op_mod) = op_ns else {
        report.push(TestResult::fail(validator, "op/ namespace not found"));
        return;
    };
    let expected = ["ST_3", "ST_4", "ST_5"];
    let mut all_found = true;
    for label in &expected {
        let has = op_mod.individuals.iter().any(|ind| ind.label == *label);
        if !has {
            report.push(TestResult::fail(
                validator,
                format!("Missing SumType topology identity: {}", label),
            ));
            all_found = false;
        }
    }
    if all_found {
        report.push(TestResult::pass(
            validator,
            "Amendment 44 SumType topology identities present (ST_3, ST_4, ST_5)",
        ));
    }
}

/// Validates TypeSynthesis reachability domain completeness (Amendment 44, G3).
fn validate_synthesis_reachability(report: &mut ConformanceReport) {
    let ontology = uor_ontology::Ontology::full();
    let validator = "ontology/inventory/synthesis_reachability";
    let op_ns = ontology
        .namespaces
        .iter()
        .find(|m| m.namespace.prefix == "op");
    let Some(op_mod) = op_ns else {
        report.push(TestResult::fail(validator, "op/ namespace not found"));
        return;
    };
    let expected = ["TS_8", "TS_9", "TS_10"];
    let mut all_found = true;
    for label in &expected {
        let has = op_mod.individuals.iter().any(|ind| ind.label == *label);
        if !has {
            report.push(TestResult::fail(
                validator,
                format!("Missing synthesis reachability identity: {}", label),
            ));
            all_found = false;
        }
    }
    if all_found {
        report.push(TestResult::pass(
            validator,
            "Amendment 44 synthesis reachability identities present (TS_8, TS_9, TS_10)",
        ));
    }
}

/// Validates ObstructionChain termination guarantee (Amendment 44, G6).
fn validate_obstruction_termination(report: &mut ConformanceReport) {
    let ontology = uor_ontology::Ontology::full();
    let validator = "ontology/inventory/obstruction_termination";
    let op_ns = ontology
        .namespaces
        .iter()
        .find(|m| m.namespace.prefix == "op");
    let Some(op_mod) = op_ns else {
        report.push(TestResult::fail(validator, "op/ namespace not found"));
        return;
    };
    let expected = ["WT_8", "WT_9"];
    let mut all_found = true;
    for label in &expected {
        let has = op_mod.individuals.iter().any(|ind| ind.label == *label);
        if !has {
            report.push(TestResult::fail(
                validator,
                format!("Missing obstruction termination identity: {}", label),
            ));
            all_found = false;
        }
    }
    if all_found {
        report.push(TestResult::pass(
            validator,
            "Amendment 44 obstruction termination identities present (WT_8, WT_9)",
        ));
    }
}

/// Validates sheaf coefficient ring grounding (Amendment 44, G11).
fn validate_coefficient_ring_grounding(report: &mut ConformanceReport) {
    let ontology = uor_ontology::Ontology::full();
    let validator = "ontology/inventory/coefficient_ring_grounding";
    let op_ns = ontology
        .namespaces
        .iter()
        .find(|m| m.namespace.prefix == "op");
    let Some(op_mod) = op_ns else {
        report.push(TestResult::fail(validator, "op/ namespace not found"));
        return;
    };
    let has = op_mod.individuals.iter().any(|ind| ind.label == "COEFF_1");
    if has {
        report.push(TestResult::pass(
            validator,
            "Amendment 44 coefficient ring identity present (COEFF_1)",
        ));
    } else {
        report.push(TestResult::fail(
            validator,
            "Missing coefficient ring identity: COEFF_1",
        ));
    }
}

/// Validates GluingObstruction resolver feedback (Amendment 44, G9).
fn validate_gluing_feedback(report: &mut ConformanceReport) {
    let ontology = uor_ontology::Ontology::full();
    let validator = "ontology/inventory/gluing_feedback";
    let op_ns = ontology
        .namespaces
        .iter()
        .find(|m| m.namespace.prefix == "op");
    let Some(op_mod) = op_ns else {
        report.push(TestResult::fail(validator, "op/ namespace not found"));
        return;
    };
    let has = op_mod.individuals.iter().any(|ind| ind.label == "GO_1");
    if has {
        report.push(TestResult::pass(
            validator,
            "Amendment 44 gluing feedback identity present (GO_1)",
        ));
    } else {
        report.push(TestResult::fail(
            validator,
            "Missing gluing feedback identity: GO_1",
        ));
    }
}

/// Validates session saturation lifecycle bridge (Amendment 44, G8).
fn validate_session_saturation_bridge(report: &mut ConformanceReport) {
    let ontology = uor_ontology::Ontology::full();
    let validator = "ontology/inventory/session_saturation_bridge";
    let op_ns = ontology
        .namespaces
        .iter()
        .find(|m| m.namespace.prefix == "op");
    let Some(op_mod) = op_ns else {
        report.push(TestResult::fail(validator, "op/ namespace not found"));
        return;
    };
    let expected = ["GR_6", "GR_7"];
    let mut all_found = true;
    for label in &expected {
        let has = op_mod.individuals.iter().any(|ind| ind.label == *label);
        if !has {
            report.push(TestResult::fail(
                validator,
                format!("Missing session saturation identity: {}", label),
            ));
            all_found = false;
        }
    }
    if all_found {
        report.push(TestResult::pass(
            validator,
            "Amendment 44 session saturation identities present (SR_6, SR_7)",
        ));
    }
}

/// Validates SuperposedSiteState amplitude index characterization (Amendment 44, G10).
fn validate_amplitude_index_characterization(report: &mut ConformanceReport) {
    let ontology = uor_ontology::Ontology::full();
    let validator = "ontology/inventory/amplitude_index_characterization";
    let op_ns = ontology
        .namespaces
        .iter()
        .find(|m| m.namespace.prefix == "op");
    let Some(op_mod) = op_ns else {
        report.push(TestResult::fail(validator, "op/ namespace not found"));
        return;
    };
    let has = op_mod.individuals.iter().any(|ind| ind.label == "QM_6");
    if has {
        report.push(TestResult::pass(
            validator,
            "Amendment 44 amplitude index identity present (QM_6)",
        ));
    } else {
        report.push(TestResult::fail(
            validator,
            "Missing amplitude index identity: QM_6",
        ));
    }
}

/// # Errors
///
/// Returns an error if the shapes file cannot be read.
pub fn validate_workspace(workspace: &Path) -> Result<ConformanceReport> {
    let mut report = ConformanceReport::new();

    let shapes_path = workspace.join("conformance/shapes/uor-shapes.ttl");
    let content = std::fs::read_to_string(&shapes_path)
        .with_context(|| format!("Failed to read {}", shapes_path.display()))?;

    let shape_count = content
        .lines()
        .filter(|l| l.contains("sh:NodeShape"))
        .count();
    let expected = uor_ontology::Ontology::full().class_count();

    check_count(
        &mut report,
        "SHACL shapes (NodeShape declarations)",
        shape_count,
        expected,
        "ontology/inventory/shapes_count",
    );

    Ok(report)
}

/// Amendment 45, Rule 1: Every direct `cert:Certificate` subclass must have
/// a governing `op:Identity` individual that references it.
fn validate_certificate_issuance_coverage(report: &mut ConformanceReport) {
    let ontology = uor_ontology::Ontology::full();
    let validator = "meta/certificate_issuance_coverage";
    let cert_root = "https://uor.foundation/cert/Certificate";

    // Collect ALL direct cert:Certificate subclasses across all namespaces
    let cert_subclasses: Vec<_> = ontology
        .namespaces
        .iter()
        .flat_map(|m| m.classes.iter())
        .filter(|c| c.subclass_of.contains(&cert_root))
        .collect();

    // Collect all op:Identity individuals
    let identity_type = "https://uor.foundation/op/Identity";
    let identities: Vec<_> = ontology
        .namespaces
        .iter()
        .flat_map(|m| m.individuals.iter())
        .filter(|i| i.type_ == identity_type)
        .collect();

    // Structural certificate subclasses that serve as containers rather
    // than identity targets — exempt from the "governing Identity" check.
    let structural_exempt: HashSet<&str> = [
        "https://uor.foundation/cert/TransformCertificate",
        "https://uor.foundation/cert/IsometryCertificate",
        "https://uor.foundation/cert/InvolutionCertificate",
        "https://uor.foundation/cert/CompletenessCertificate",
        "https://uor.foundation/cert/GroundingCertificate",
        "https://uor.foundation/cert/GeodesicCertificate",
        "https://uor.foundation/cert/MeasurementCertificate",
        "https://uor.foundation/cert/BornRuleVerification",
        "https://uor.foundation/cert/LiftChainCertificate",
        "https://uor.foundation/morphism/GroundingCertificate",
        "https://uor.foundation/parallel/DisjointnessCertificate",
        // v0.2.1: InhabitanceCertificate is governed by IH_1 whose lhs
        // and forAll reference the type by label, but the inheritance
        // path goes through proof:ComputationCertificate first; the
        // structural exemption captures the multi-parent case.
        "https://uor.foundation/cert/InhabitanceCertificate",
        // v0.2.2 T1.3 (cleanup): MultiplicationCertificate and
        // PartitionCertificate exemptions were REMOVED. OA_5 and PT_2
        // now explicitly cite the certificates in their rdfs:comment /
        // forAll text; the text-based governance loop below picks them up.
        // Workstream C (v0.2.2 closure): impossibility certificates are
        // the failure-path carriers for resolver `certify` functions.
        // They are minted only when the resolver's decision procedure
        // produces no success witness; no separate `op:Identity` governs
        // them — the governing identity is the resolver's own semantics.
        "https://uor.foundation/cert/GenericImpossibilityCertificate",
        "https://uor.foundation/cert/InhabitanceImpossibilityCertificate",
    ]
    .into();

    // v0.2.2 T1.3: after `rewrite_identity_ast_refs`, identity `lhs`/`rhs`/
    // `forAll` values are IriRef pointers to `schema:term_*` individuals whose
    // `literalValue` / `variableName` properties carry the original strings.
    // Build a lookup table from term-individual IRI to its stored text so we
    // can follow the reference when scanning for certificate governance.
    let term_literal_prop = "https://uor.foundation/schema/literalValue";
    let term_variable_prop = "https://uor.foundation/schema/variableName";
    let literal_expression_type = "https://uor.foundation/schema/LiteralExpression";
    let forall_declaration_type = "https://uor.foundation/schema/ForAllDeclaration";
    let term_text_by_iri: HashMap<&str, &str> = ontology
        .namespaces
        .iter()
        .flat_map(|m| m.individuals.iter())
        .filter(|i| i.type_ == literal_expression_type || i.type_ == forall_declaration_type)
        .filter_map(|i| {
            i.properties.iter().find_map(|(k, v)| {
                if *k == term_literal_prop || *k == term_variable_prop {
                    if let IndividualValue::Str(s) = v {
                        Some((i.id, *s))
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
        })
        .collect();

    let mut all_governed = true;
    for cert_class in &cert_subclasses {
        if structural_exempt.contains(cert_class.id) {
            continue;
        }
        let governed = identities.iter().any(|id| {
            id.properties
                .iter()
                .filter(|(k, _)| {
                    *k == "https://uor.foundation/op/lhs"
                        || *k == "https://uor.foundation/op/rhs"
                        || *k == "https://uor.foundation/op/forAll"
                })
                .any(|(_, v)| match v {
                    IndividualValue::Str(s) => {
                        s.contains(cert_class.label) || s.contains(cert_class.id)
                    }
                    IndividualValue::IriRef(iri) => {
                        // Direct class-IRI reference:
                        if iri.contains(cert_class.id) {
                            return true;
                        }
                        // v0.2.2 T1.3: follow `schema:term_*` refs to the
                        // underlying LiteralExpression/ForAllDeclaration text.
                        if let Some(text) = term_text_by_iri.get(*iri) {
                            text.contains(cert_class.label) || text.contains(cert_class.id)
                        } else {
                            false
                        }
                    }
                    IndividualValue::List(iris) => {
                        iris.iter().any(|iri| iri.contains(cert_class.id))
                    }
                    _ => false,
                })
        });

        if !governed {
            report.push_meta(TestResult::fail(
                validator,
                format!(
                    "cert:Certificate subclass {} has no governing Identity",
                    cert_class.label
                ),
            ));
            all_governed = false;
        }
    }

    if all_governed {
        report.push_meta(TestResult::pass(
            validator,
            format!(
                "All {} Certificate subclasses governed by Identities",
                cert_subclasses.len()
            ),
        ));
    }
}

/// Amendment 45, Rule 2: Strict bijection between `op:Identity` individuals
/// and proof individuals via `provesIdentity`.
fn validate_identity_proof_bijection(report: &mut ConformanceReport) {
    let ontology = uor_ontology::Ontology::full();
    let validator = "meta/identity_proof_bijection";

    let identity_type = "https://uor.foundation/op/Identity";
    let proves_prop = "https://uor.foundation/proof/provesIdentity";

    // All op:Identity IRIs
    let identity_iris: HashSet<&str> = ontology
        .namespaces
        .iter()
        .flat_map(|m| m.individuals.iter())
        .filter(|i| i.type_ == identity_type)
        .map(|i| i.id)
        .collect();

    // All provesIdentity targets from proof individuals
    let proved_iris: HashSet<&str> = ontology
        .namespaces
        .iter()
        .flat_map(|m| m.individuals.iter())
        .flat_map(|i| i.properties.iter())
        .filter(|(k, _)| *k == proves_prop)
        .filter_map(|(_, v)| {
            if let IndividualValue::IriRef(iri) = v {
                Some(*iri)
            } else {
                None
            }
        })
        .collect();

    let mut all_ok = true;

    // Direction 1: every Identity must be proved
    for iri in &identity_iris {
        if !proved_iris.contains(iri) {
            report.push_meta(TestResult::fail(
                validator,
                format!("Identity {} has no proof individual", iri),
            ));
            all_ok = false;
        }
    }

    // Direction 2: every proof target must be a valid Identity
    for iri in &proved_iris {
        if !identity_iris.contains(iri) {
            report.push_meta(TestResult::fail(
                validator,
                format!("Proof targets nonexistent Identity {}", iri),
            ));
            all_ok = false;
        }
    }

    // Amendment 47: Encoding guard — all provesIdentity values must be IriRef
    for ns in &ontology.namespaces {
        for ind in ns.individuals.iter() {
            for (k, v) in ind.properties.iter() {
                if *k == proves_prop && !matches!(v, IndividualValue::IriRef(_)) {
                    report.push_meta(TestResult::fail(
                        validator,
                        format!("{} has non-IriRef provesIdentity value", ind.label),
                    ));
                    all_ok = false;
                }
            }
        }
    }

    if all_ok {
        report.push_meta(TestResult::pass(
            validator,
            format!(
                "Identity-proof bijection holds: {} identities, {} proofs",
                identity_iris.len(),
                proved_iris.len()
            ),
        ));
    }
}

/// Amendment 45, Rule 3: Every kernel/bridge class must appear in at least
/// one SHACL fixture.
fn validate_shacl_fixture_coverage(report: &mut ConformanceReport) {
    let ontology = uor_ontology::Ontology::full();
    let validator = "meta/shacl_fixture_coverage";

    // Classes in kernel and bridge spaces only
    let required_classes: Vec<_> = ontology
        .namespaces
        .iter()
        .filter(|m| matches!(m.namespace.space, Space::Kernel | Space::Bridge))
        .flat_map(|m| {
            let prefix = m.namespace.prefix;
            m.classes.iter().map(move |c| (prefix, c))
        })
        .collect();

    let all_fixtures = crate::tests::fixtures::all_fixture_sources();

    let mut all_covered = true;
    for (prefix, class) in &required_classes {
        // Extract local name from IRI (everything after last '/')
        let local_name = class.id.rsplit('/').next().unwrap_or("");
        let prefixed_form = format!("{}:{}", prefix, local_name);

        let covered = all_fixtures.iter().any(|src| {
            src.contains(&prefixed_form) || src.contains(class.id) || src.contains(class.label)
        });

        if !covered {
            report.push_meta(TestResult::fail(
                validator,
                format!(
                    "Kernel/bridge class {} has no SHACL fixture coverage",
                    class.label
                ),
            ));
            all_covered = false;
        }
    }

    // Amendment 47: Second pass — cert:Certificate subclasses across ALL spaces
    let cert_root = "https://uor.foundation/cert/Certificate";
    let cert_subclasses: Vec<_> = ontology
        .namespaces
        .iter()
        .flat_map(|m| {
            let prefix = m.namespace.prefix;
            m.classes.iter().map(move |c| (prefix, c))
        })
        .filter(|(_, c)| c.subclass_of.contains(&cert_root))
        .collect();
    for (prefix, cert_class) in &cert_subclasses {
        let local_name = cert_class.id.rsplit('/').next().unwrap_or("");
        let prefixed_form = format!("{}:{}", prefix, local_name);
        let has_fixture = all_fixtures.iter().any(|src| {
            src.contains(&prefixed_form)
                || src.contains(cert_class.id)
                || src.contains(cert_class.label)
        });
        if !has_fixture {
            all_covered = false;
            report.push_meta(TestResult::fail(
                validator,
                format!(
                    "cert:Certificate subclass {} has no SHACL fixture",
                    cert_class.label
                ),
            ));
        }
    }

    if all_covered {
        report.push_meta(TestResult::pass(
            validator,
            format!(
                "All {} kernel/bridge classes and {} Certificate subclasses \
                 have SHACL fixture coverage",
                required_classes.len(),
                cert_subclasses.len()
            ),
        ));
    }

    // Amendment 49, Rule 3 extended: user-space class SHACL fixture mandate.
    // type/, state/, and morphism/ classes are Prism-implementer territory, but
    // their presence in the fixture suite must be enforced to prevent silent regression.
    let validator_user = "meta/shacl_fixture_coverage/user";
    let user_classes: Vec<_> = ontology
        .namespaces
        .iter()
        .filter(|m| matches!(m.namespace.space, Space::User))
        .flat_map(|m| {
            let prefix = m.namespace.prefix;
            m.classes.iter().map(move |c| (prefix, c))
        })
        .collect();
    let mut user_covered = true;
    for (prefix, class) in &user_classes {
        let local_name = class.id.rsplit('/').next().unwrap_or("");
        let prefixed_form = format!("{}:{}", prefix, local_name);
        let covered = all_fixtures.iter().any(|src| {
            src.contains(&prefixed_form) || src.contains(class.id) || src.contains(class.label)
        });
        if !covered {
            report.push_meta(TestResult::fail(
                validator_user,
                format!(
                    "User-space class {} has no SHACL fixture coverage",
                    class.label
                ),
            ));
            user_covered = false;
        }
    }
    if user_covered {
        report.push_meta(TestResult::pass(
            validator_user,
            format!(
                "All {} user-space classes have SHACL fixture coverage",
                user_classes.len()
            ),
        ));
    }
}

// ── Amendment 47: Model-Derived Meta-Validators (Rules 4-8) ─────────────

/// Rule 4: Every property key in individual assertions must be a known
/// property IRI in the ontology.
fn validate_property_assertion_iris(report: &mut ConformanceReport) {
    let ontology = uor_ontology::Ontology::full();
    let validator = "meta/property_assertion_iris";

    let known_props: HashSet<&str> = ontology
        .namespaces
        .iter()
        .flat_map(|m| m.properties.iter())
        .map(|p| p.id)
        .collect();

    let mut violations = Vec::new();
    for ns in &ontology.namespaces {
        for ind in &ns.individuals {
            for (k, _) in ind.properties.iter() {
                if !known_props.contains(k) {
                    violations.push(format!("{}: unknown property IRI {}", ind.label, k));
                }
            }
        }
    }

    if violations.is_empty() {
        report.push_meta(TestResult::pass(
            validator,
            "All individual property assertions use known property IRIs",
        ));
    } else {
        for v in &violations {
            report.push_meta(TestResult::fail(validator, v.clone()));
        }
    }
}

/// Rule 5: ObjectProperty assertions should use IriRef/List values;
/// DatatypeProperty assertions should use Str/Int/Bool values.
fn validate_property_kind_conformance(report: &mut ConformanceReport) {
    let ontology = uor_ontology::Ontology::full();
    let validator = "meta/property_kind_conformance";

    let prop_map: HashMap<&str, &uor_ontology::model::Property> = ontology
        .namespaces
        .iter()
        .flat_map(|m| m.properties.iter())
        .map(|p| (p.id, p))
        .collect();

    let mut violations = Vec::new();
    for ns in &ontology.namespaces {
        for ind in &ns.individuals {
            for (k, v) in ind.properties.iter() {
                if let Some(prop) = prop_map.get(k) {
                    match prop.kind {
                        PropertyKind::Object => {
                            if !matches!(v, IndividualValue::IriRef(_) | IndividualValue::List(_)) {
                                violations.push(format!(
                                    "{}: ObjectProperty {} has non-IriRef value {}",
                                    ind.label, prop.label, v
                                ));
                            }
                        }
                        PropertyKind::Datatype => {
                            if matches!(v, IndividualValue::IriRef(_) | IndividualValue::List(_)) {
                                violations.push(format!(
                                    "{}: DatatypeProperty {} has IriRef value {}",
                                    ind.label, prop.label, v
                                ));
                            }
                        }
                        PropertyKind::Annotation => {}
                    }
                }
            }
        }
    }

    if violations.is_empty() {
        report.push_meta(TestResult::pass(
            validator,
            "All individual property values match their declared property kind",
        ));
    } else {
        for v in &violations {
            report.push_meta(TestResult::fail(validator, v.clone()));
        }
    }
}

/// Rule 6: Functional properties must appear at most once per individual.
fn validate_functional_cardinality(report: &mut ConformanceReport) {
    let ontology = uor_ontology::Ontology::full();
    let validator = "meta/functional_cardinality";

    let functional_props: HashSet<&str> = ontology
        .namespaces
        .iter()
        .flat_map(|m| m.properties.iter())
        .filter(|p| p.functional)
        .map(|p| p.id)
        .collect();

    let mut violations = Vec::new();
    for ns in &ontology.namespaces {
        for ind in &ns.individuals {
            let mut prop_counts: HashMap<&str, usize> = HashMap::new();
            for (k, _) in ind.properties.iter() {
                if functional_props.contains(k) {
                    *prop_counts.entry(k).or_insert(0) += 1;
                }
            }
            for (k, count) in &prop_counts {
                if *count > 1 {
                    violations.push(format!(
                        "{}: functional property {} appears {} times",
                        ind.label, k, count
                    ));
                }
            }
        }
    }

    if violations.is_empty() {
        report.push_meta(TestResult::pass(
            validator,
            "All functional properties have at most one assertion per individual",
        ));
    } else {
        for v in &violations {
            report.push_meta(TestResult::fail(validator, v.clone()));
        }
    }
}

/// Rule 7: Every IriRef value must resolve to a known individual or class.
fn validate_iri_ref_targets(report: &mut ConformanceReport) {
    let ontology = uor_ontology::Ontology::full();
    let validator = "meta/iri_ref_targets";

    let known_individuals: HashSet<&str> = ontology
        .namespaces
        .iter()
        .flat_map(|m| m.individuals.iter())
        .map(|i| i.id)
        .collect();
    let known_classes: HashSet<&str> = ontology
        .namespaces
        .iter()
        .flat_map(|m| m.classes.iter())
        .map(|c| c.id)
        .collect();
    let known_properties: HashSet<&str> = ontology
        .namespaces
        .iter()
        .flat_map(|m| m.properties.iter())
        .map(|p| p.id)
        .collect();

    // Well-known external IRIs that are valid targets but not defined
    // in the ontology itself.
    let known_external: HashSet<&str> = [
        "http://www.w3.org/2001/XMLSchema#decimal",
        "http://www.w3.org/2001/XMLSchema#string",
        "http://www.w3.org/2001/XMLSchema#integer",
        "http://www.w3.org/2001/XMLSchema#nonNegativeInteger",
        "http://www.w3.org/2001/XMLSchema#positiveInteger",
        "http://www.w3.org/2001/XMLSchema#boolean",
    ]
    .into();

    let mut violations = Vec::new();
    for ns in &ontology.namespaces {
        for ind in &ns.individuals {
            for (k, v) in ind.properties.iter() {
                let iris: Vec<&str> = match v {
                    IndividualValue::IriRef(iri) => vec![iri],
                    IndividualValue::List(iris) => iris.to_vec(),
                    _ => vec![],
                };
                for iri in iris {
                    if !known_individuals.contains(iri)
                        && !known_classes.contains(iri)
                        && !known_properties.contains(iri)
                        && !known_external.contains(iri)
                    {
                        violations.push(format!(
                            "{}: property {} references unknown IRI {}",
                            ind.label, k, iri
                        ));
                    }
                }
            }
        }
    }

    if violations.is_empty() {
        report.push_meta(TestResult::pass(
            validator,
            "All IriRef values resolve to known individuals or classes",
        ));
    } else {
        for v in &violations {
            report.push_meta(TestResult::fail(validator, v.clone()));
        }
    }
}

/// Rule 8: Datatype property values must match their declared XSD range type.
fn validate_datatype_range_conformance(report: &mut ConformanceReport) {
    let ontology = uor_ontology::Ontology::full();
    let validator = "meta/datatype_range_conformance";

    let prop_map: HashMap<&str, &uor_ontology::model::Property> = ontology
        .namespaces
        .iter()
        .flat_map(|m| m.properties.iter())
        .map(|p| (p.id, p))
        .collect();

    let bool_ranges: HashSet<&str> =
        ["xsd:boolean", "http://www.w3.org/2001/XMLSchema#boolean"].into();
    let int_ranges: HashSet<&str> = [
        "xsd:integer",
        "xsd:nonNegativeInteger",
        "xsd:positiveInteger",
        "http://www.w3.org/2001/XMLSchema#integer",
        "http://www.w3.org/2001/XMLSchema#nonNegativeInteger",
        "http://www.w3.org/2001/XMLSchema#positiveInteger",
    ]
    .into();

    let mut violations = Vec::new();
    for ns in &ontology.namespaces {
        for ind in &ns.individuals {
            for (k, v) in ind.properties.iter() {
                if let Some(prop) = prop_map.get(k) {
                    if prop.kind != PropertyKind::Datatype {
                        continue;
                    }
                    if bool_ranges.contains(prop.range) && !matches!(v, IndividualValue::Bool(_)) {
                        violations.push(format!(
                            "{}: {} (range {}) expects Bool, got {}",
                            ind.label, prop.label, prop.range, v
                        ));
                    } else if int_ranges.contains(prop.range)
                        && !matches!(v, IndividualValue::Int(_))
                    {
                        violations.push(format!(
                            "{}: {} (range {}) expects Int, got {}",
                            ind.label, prop.label, prop.range, v
                        ));
                    }
                    // String-like ranges (xsd:string, xsd:anyURI, etc.) accept Str
                }
            }
        }
    }

    if violations.is_empty() {
        report.push_meta(TestResult::pass(
            validator,
            "All datatype property values match their declared XSD range",
        ));
    } else {
        for v in &violations {
            report.push_meta(TestResult::fail(validator, v.clone()));
        }
    }
}

/// Amendment 49: Validates structural consistency between formal `validityKind`
/// scope bindings and the presence of `validKMin`/`validKMax` properties on
/// `op:Identity` individuals.
///
/// Enforced invariants:
/// - `ParametricLower` requires `validKMin`
/// - `ParametricRange` requires both `validKMin` and `validKMax`
/// - `LevelSpecific` must not carry `validKMin` or `validKMax`
/// - `universallyValid = true` must not be combined with a non-Universal `validityKind`
fn validate_forall_scope_alignment(report: &mut ConformanceReport) {
    let ontology = uor_ontology::Ontology::full();
    let validator = "ontology/inventory/forall_scope_alignment";

    let identity_iri = "https://uor.foundation/op/Identity";
    let kind_prop = "https://uor.foundation/op/validityKind";
    let k_min_prop = "https://uor.foundation/op/validKMin";
    let k_max_prop = "https://uor.foundation/op/validKMax";
    let univ_prop = "https://uor.foundation/op/universallyValid";

    let parametric_lower = "https://uor.foundation/op/ParametricLower";
    let parametric_range = "https://uor.foundation/op/ParametricRange";
    let level_specific = "https://uor.foundation/op/LevelSpecific";
    let universal = "https://uor.foundation/op/Universal";

    let mut violations: Vec<String> = Vec::new();

    let identities: Vec<_> = ontology
        .namespaces
        .iter()
        .flat_map(|m| m.individuals.iter())
        .filter(|ind| ind.type_ == identity_iri)
        .collect();

    for ind in &identities {
        let kind_iri: Option<&str> = ind.properties.iter().find_map(|(k, v)| {
            if *k == kind_prop {
                if let uor_ontology::IndividualValue::IriRef(iri) = v {
                    return Some(*iri);
                }
            }
            None
        });
        let has_k_min = ind.properties.iter().any(|(k, _)| *k == k_min_prop);
        let has_k_max = ind.properties.iter().any(|(k, _)| *k == k_max_prop);
        let univ_true = ind.properties.iter().any(|(k, v)| {
            *k == univ_prop && matches!(v, uor_ontology::IndividualValue::Bool(true))
        });

        if let Some(kind) = kind_iri {
            // Check A: ParametricLower requires validKMin
            if kind == parametric_lower && !has_k_min {
                violations.push(format!(
                    "{}: validityKind=ParametricLower but validKMin is absent",
                    ind.label
                ));
            }
            // Check B: ParametricRange requires both validKMin and validKMax
            if kind == parametric_range && (!has_k_min || !has_k_max) {
                violations.push(format!(
                    "{}: validityKind=ParametricRange but validKMin/validKMax incomplete",
                    ind.label
                ));
            }
            // Check C: LevelSpecific must not carry range bounds
            if kind == level_specific && (has_k_min || has_k_max) {
                violations.push(format!(
                    "{}: validityKind=LevelSpecific must not carry validKMin/validKMax",
                    ind.label
                ));
            }
            // Check D: universallyValid=true contradicts a non-Universal validityKind
            if univ_true && kind != universal {
                violations.push(format!(
                    "{}: universallyValid=true contradicts validityKind <{}>",
                    ind.label, kind
                ));
            }
        }
    }

    if violations.is_empty() {
        report.push(TestResult::pass(
            validator,
            format!(
                "All {} op:Identity scope bindings are structurally consistent",
                identities.len()
            ),
        ));
    } else {
        for v in violations {
            report.push(TestResult::fail(validator, v));
        }
    }
}

/// Amendment 53: Validates the WC_ and OA_ identity families exist with
/// correct verification domains.
fn validate_witt_carry_vocabulary(report: &mut ConformanceReport) {
    let ontology = uor_ontology::Ontology::full();
    let validator = "ontology/inventory/witt_carry_vocabulary";

    let wc_ids: &[(&str, &str)] = &[
        (
            "https://uor.foundation/op/WC_1",
            "https://uor.foundation/op/Algebraic",
        ),
        (
            "https://uor.foundation/op/WC_2",
            "https://uor.foundation/op/Algebraic",
        ),
        (
            "https://uor.foundation/op/WC_3",
            "https://uor.foundation/op/Algebraic",
        ),
        (
            "https://uor.foundation/op/WC_4",
            "https://uor.foundation/op/Algebraic",
        ),
        (
            "https://uor.foundation/op/WC_5",
            "https://uor.foundation/op/IndexTheoretic",
        ),
        (
            "https://uor.foundation/op/WC_6",
            "https://uor.foundation/op/Analytical",
        ),
        (
            "https://uor.foundation/op/WC_7",
            "https://uor.foundation/op/Algebraic",
        ),
        (
            "https://uor.foundation/op/WC_8",
            "https://uor.foundation/op/Algebraic",
        ),
        (
            "https://uor.foundation/op/WC_9",
            "https://uor.foundation/op/Algebraic",
        ),
        (
            "https://uor.foundation/op/WC_10",
            "https://uor.foundation/op/Algebraic",
        ),
        (
            "https://uor.foundation/op/WC_11",
            "https://uor.foundation/op/Algebraic",
        ),
        (
            "https://uor.foundation/op/WC_12",
            "https://uor.foundation/op/Algebraic",
        ),
        (
            "https://uor.foundation/op/OA_1",
            "https://uor.foundation/op/ArithmeticValuation",
        ),
        (
            "https://uor.foundation/op/OA_2",
            "https://uor.foundation/op/ArithmeticValuation",
        ),
        (
            "https://uor.foundation/op/OA_3",
            "https://uor.foundation/op/ArithmeticValuation",
        ),
        (
            "https://uor.foundation/op/OA_4",
            "https://uor.foundation/op/ArithmeticValuation",
        ),
        (
            "https://uor.foundation/op/OA_5",
            "https://uor.foundation/op/ArithmeticValuation",
        ),
    ];

    let domain_prop = "https://uor.foundation/op/verificationDomain";
    let mut all_valid = true;

    for (iri, expected_domain) in wc_ids {
        match ontology.find_individual(iri) {
            Some(ind) => {
                let has_domain = ind.properties.iter().any(|(k, v)| {
                    *k == domain_prop
                        && matches!(
                            v,
                            IndividualValue::IriRef(d) if *d == *expected_domain
                        )
                });
                if !has_domain {
                    report.push(TestResult::fail(
                        validator,
                        format!("{} missing verificationDomain {}", iri, expected_domain),
                    ));
                    all_valid = false;
                }
            }
            None => {
                report.push(TestResult::fail(
                    validator,
                    format!("Identity {} not found", iri),
                ));
                all_valid = false;
            }
        }
    }

    if all_valid {
        report.push(TestResult::pass(
            validator,
            "All 17 WC_/OA_ identities present with correct verificationDomains",
        ));
    }
}

/// Amendment 53: Validates the Witt\u{2013}Ostrowski\u{2013}Landauer
/// derivation chain: WC_4 \u{2192} OA_1 \u{2192} OA_2 \u{2192} OA_3 all
/// exist with AxiomaticDerivation proofs.
fn validate_ostrowski_derivation_chain(report: &mut ConformanceReport) {
    let ontology = uor_ontology::Ontology::full();
    let validator = "ontology/inventory/ostrowski_derivation_chain";

    let chain = [
        (
            "https://uor.foundation/op/WC_4",
            "https://uor.foundation/proof/prf_WC_4",
        ),
        (
            "https://uor.foundation/op/OA_1",
            "https://uor.foundation/proof/prf_OA_1",
        ),
        (
            "https://uor.foundation/op/OA_2",
            "https://uor.foundation/proof/prf_OA_2",
        ),
        (
            "https://uor.foundation/op/OA_3",
            "https://uor.foundation/proof/prf_OA_3",
        ),
    ];

    let ad_type = "https://uor.foundation/proof/AxiomaticDerivation";
    let mut all_valid = true;

    for (identity_iri, proof_iri) in &chain {
        if ontology.find_individual(identity_iri).is_none() {
            report.push(TestResult::fail(
                validator,
                format!("Chain identity {} not found", identity_iri),
            ));
            all_valid = false;
            continue;
        }
        match ontology.find_individual(proof_iri) {
            Some(proof) => {
                if proof.type_ != ad_type {
                    report.push(TestResult::fail(
                        validator,
                        format!(
                            "Proof {} is {} (expected AxiomaticDerivation)",
                            proof_iri, proof.type_
                        ),
                    ));
                    all_valid = false;
                }
            }
            None => {
                report.push(TestResult::fail(
                    validator,
                    format!("Chain proof {} not found", proof_iri),
                ));
                all_valid = false;
            }
        }
    }

    if all_valid {
        report.push(TestResult::pass(
            validator,
            "Witt\u{2013}Ostrowski\u{2013}Landauer derivation chain complete",
        ));
    }
}

/// Amendment 54: Validates HT_1\u{2013}HT_8 identities with correct
/// verificationDomains (Topological for 1-5,8; IndexTheoretic for 6,7).
fn validate_homotopy_nerve_vocabulary(report: &mut ConformanceReport) {
    let ontology = uor_ontology::Ontology::full();
    let validator = "ontology/inventory/homotopy_nerve_vocabulary";

    let ht_ids: &[(&str, &str)] = &[
        (
            "https://uor.foundation/op/HT_1",
            "https://uor.foundation/op/Topological",
        ),
        (
            "https://uor.foundation/op/HT_2",
            "https://uor.foundation/op/Topological",
        ),
        (
            "https://uor.foundation/op/HT_3",
            "https://uor.foundation/op/Topological",
        ),
        (
            "https://uor.foundation/op/HT_4",
            "https://uor.foundation/op/Topological",
        ),
        (
            "https://uor.foundation/op/HT_5",
            "https://uor.foundation/op/Topological",
        ),
        (
            "https://uor.foundation/op/HT_6",
            "https://uor.foundation/op/IndexTheoretic",
        ),
        (
            "https://uor.foundation/op/HT_7",
            "https://uor.foundation/op/IndexTheoretic",
        ),
        (
            "https://uor.foundation/op/HT_8",
            "https://uor.foundation/op/Topological",
        ),
    ];

    let domain_prop = "https://uor.foundation/op/verificationDomain";
    let mut all_valid = true;

    for (iri, expected_domain) in ht_ids {
        match ontology.find_individual(iri) {
            Some(ind) => {
                let has_domain = ind.properties.iter().any(|(k, v)| {
                    *k == domain_prop
                        && matches!(
                            v,
                            IndividualValue::IriRef(d) if *d == *expected_domain
                        )
                });
                if !has_domain {
                    report.push(TestResult::fail(
                        validator,
                        format!("{} missing verificationDomain {}", iri, expected_domain),
                    ));
                    all_valid = false;
                }
            }
            None => {
                report.push(TestResult::fail(
                    validator,
                    format!("Identity {} not found", iri),
                ));
                all_valid = false;
            }
        }
    }

    if all_valid {
        report.push(TestResult::pass(
            validator,
            "All 8 HT_ identities present with correct verificationDomains",
        ));
    }
}

/// Amendment 54: Validates Postnikov bridge \u{2014} CechNerve has
/// KanComplex subclass and observable/ has a postnikovTruncation property.
fn validate_postnikov_bridge(report: &mut ConformanceReport) {
    let ontology = uor_ontology::Ontology::full();
    let validator = "ontology/inventory/postnikov_bridge";

    let kan_iri = "https://uor.foundation/homology/KanComplex";
    let nerve_found = ontology
        .namespaces
        .iter()
        .flat_map(|m| m.classes.iter())
        .any(|c| c.label == "CechNerve" && c.subclass_of.contains(&kan_iri));

    let postnikov_prop = ontology
        .namespaces
        .iter()
        .flat_map(|m| m.properties.iter())
        .any(|p| p.id.ends_with("/postnikovTruncation"));

    if nerve_found && postnikov_prop {
        report.push(TestResult::pass(
            validator,
            "Postnikov bridge: CechNerve subclasses KanComplex and postnikovTruncation property exists",
        ));
    } else {
        let mut msg = String::new();
        if !nerve_found {
            msg.push_str("CechNerve does not subclass KanComplex; ");
        }
        if !postnikov_prop {
            msg.push_str("postnikovTruncation property not found");
        }
        report.push(TestResult::fail(validator, msg));
    }
}

/// Amendment 55: Validates homotopy pipeline identities psi_7\u{2013}psi_9,
/// HP_1\u{2013}HP_4 with correct verificationDomains.
fn validate_homotopy_pipeline(report: &mut ConformanceReport) {
    let ontology = uor_ontology::Ontology::full();
    let validator = "ontology/inventory/homotopy_pipeline";

    let hp_ids: &[(&str, &str)] = &[
        (
            "https://uor.foundation/op/psi_7",
            "https://uor.foundation/op/Pipeline",
        ),
        (
            "https://uor.foundation/op/psi_8",
            "https://uor.foundation/op/Pipeline",
        ),
        (
            "https://uor.foundation/op/psi_9",
            "https://uor.foundation/op/Pipeline",
        ),
        (
            "https://uor.foundation/op/HP_1",
            "https://uor.foundation/op/Pipeline",
        ),
        (
            "https://uor.foundation/op/HP_2",
            "https://uor.foundation/op/Pipeline",
        ),
        (
            "https://uor.foundation/op/HP_3",
            "https://uor.foundation/op/IndexTheoretic",
        ),
        (
            "https://uor.foundation/op/HP_4",
            "https://uor.foundation/op/Analytical",
        ),
    ];

    let domain_prop = "https://uor.foundation/op/verificationDomain";
    let mut all_valid = true;

    for (iri, expected_domain) in hp_ids {
        match ontology.find_individual(iri) {
            Some(ind) => {
                let has_domain = ind.properties.iter().any(|(k, v)| {
                    *k == domain_prop
                        && matches!(
                            v,
                            IndividualValue::IriRef(d) if *d == *expected_domain
                        )
                });
                if !has_domain {
                    report.push(TestResult::fail(
                        validator,
                        format!("{} missing verificationDomain {}", iri, expected_domain),
                    ));
                    all_valid = false;
                }
            }
            None => {
                report.push(TestResult::fail(
                    validator,
                    format!("Identity {} not found", iri),
                ));
                all_valid = false;
            }
        }
    }

    if all_valid {
        report.push(TestResult::pass(
            validator,
            "All homotopy pipeline identities present with correct verificationDomains",
        ));
    }
}

/// Amendment 56: Validates MD_1\u{2013}MD_10 identities with correct
/// verificationDomains.
fn validate_moduli_vocabulary(report: &mut ConformanceReport) {
    let ontology = uor_ontology::Ontology::full();
    let validator = "ontology/inventory/moduli_vocabulary";

    let md_ids: &[(&str, &str)] = &[
        (
            "https://uor.foundation/op/MD_1",
            "https://uor.foundation/op/Algebraic",
        ),
        (
            "https://uor.foundation/op/MD_2",
            "https://uor.foundation/op/Algebraic",
        ),
        (
            "https://uor.foundation/op/MD_3",
            "https://uor.foundation/op/Algebraic",
        ),
        (
            "https://uor.foundation/op/MD_4",
            "https://uor.foundation/op/IndexTheoretic",
        ),
        (
            "https://uor.foundation/op/MD_5",
            "https://uor.foundation/op/Topological",
        ),
        (
            "https://uor.foundation/op/MD_6",
            "https://uor.foundation/op/Topological",
        ),
        (
            "https://uor.foundation/op/MD_7",
            "https://uor.foundation/op/Algebraic",
        ),
        (
            "https://uor.foundation/op/MD_8",
            "https://uor.foundation/op/IndexTheoretic",
        ),
        (
            "https://uor.foundation/op/MD_9",
            "https://uor.foundation/op/IndexTheoretic",
        ),
        (
            "https://uor.foundation/op/MD_10",
            "https://uor.foundation/op/IndexTheoretic",
        ),
    ];

    let domain_prop = "https://uor.foundation/op/verificationDomain";
    let mut all_valid = true;

    for (iri, expected_domain) in md_ids {
        match ontology.find_individual(iri) {
            Some(ind) => {
                let has_domain = ind.properties.iter().any(|(k, v)| {
                    *k == domain_prop
                        && matches!(
                            v,
                            IndividualValue::IriRef(d) if *d == *expected_domain
                        )
                });
                if !has_domain {
                    report.push(TestResult::fail(
                        validator,
                        format!("{} missing verificationDomain {}", iri, expected_domain),
                    ));
                    all_valid = false;
                }
            }
            None => {
                report.push(TestResult::fail(
                    validator,
                    format!("Identity {} not found", iri),
                ));
                all_valid = false;
            }
        }
    }

    if all_valid {
        report.push(TestResult::pass(
            validator,
            "All 10 MD_ identities present with correct verificationDomains",
        ));
    }
}

/// Amendment 56: Validates DeformationComplex class exists and subclasses
/// ChainComplex.
fn validate_deformation_complex(report: &mut ConformanceReport) {
    let ontology = uor_ontology::Ontology::full();
    let validator = "ontology/inventory/deformation_complex";

    let chain_iri = "https://uor.foundation/homology/ChainComplex";
    let found = ontology
        .namespaces
        .iter()
        .flat_map(|m| m.classes.iter())
        .any(|c| c.label == "DeformationComplex" && c.subclass_of.contains(&chain_iri));

    if found {
        report.push(TestResult::pass(
            validator,
            "DeformationComplex class exists and subclasses ChainComplex",
        ));
    } else {
        report.push(TestResult::fail(
            validator,
            "DeformationComplex class not found or does not subclass ChainComplex",
        ));
    }
}

/// Amendment 57: Validates MR_1\u{2013}MR_4 identities with correct
/// verificationDomains.
fn validate_moduli_resolver_vocabulary(report: &mut ConformanceReport) {
    let ontology = uor_ontology::Ontology::full();
    let validator = "ontology/inventory/moduli_resolver_vocabulary";

    let mr_ids: &[(&str, &str)] = &[
        (
            "https://uor.foundation/op/MR_1",
            "https://uor.foundation/op/Algebraic",
        ),
        (
            "https://uor.foundation/op/MR_2",
            "https://uor.foundation/op/Topological",
        ),
        (
            "https://uor.foundation/op/MR_3",
            "https://uor.foundation/op/Analytical",
        ),
        (
            "https://uor.foundation/op/MR_4",
            "https://uor.foundation/op/Algebraic",
        ),
    ];

    let domain_prop = "https://uor.foundation/op/verificationDomain";
    let mut all_valid = true;

    for (iri, expected_domain) in mr_ids {
        match ontology.find_individual(iri) {
            Some(ind) => {
                let has_domain = ind.properties.iter().any(|(k, v)| {
                    *k == domain_prop
                        && matches!(
                            v,
                            IndividualValue::IriRef(d) if *d == *expected_domain
                        )
                });
                if !has_domain {
                    report.push(TestResult::fail(
                        validator,
                        format!("{} missing verificationDomain {}", iri, expected_domain),
                    ));
                    all_valid = false;
                }
            }
            None => {
                report.push(TestResult::fail(
                    validator,
                    format!("Identity {} not found", iri),
                ));
                all_valid = false;
            }
        }
    }

    if all_valid {
        report.push(TestResult::pass(
            validator,
            "All 4 MR_ identities present with correct verificationDomains",
        ));
    }
}

/// Validates the carry algebra vocabulary (Amendment 58).
fn validate_carry_algebra_vocabulary(report: &mut ConformanceReport) {
    let ontology = uor_ontology::Ontology::full();
    let validator = "ontology/inventory/carry_algebra_vocabulary";

    let cy_ids: &[(&str, &str)] = &[
        (
            "https://uor.foundation/op/CY_1",
            "https://uor.foundation/op/Algebraic",
        ),
        (
            "https://uor.foundation/op/CY_2",
            "https://uor.foundation/op/Algebraic",
        ),
        (
            "https://uor.foundation/op/CY_3",
            "https://uor.foundation/op/Algebraic",
        ),
        (
            "https://uor.foundation/op/CY_4",
            "https://uor.foundation/op/Algebraic",
        ),
        (
            "https://uor.foundation/op/CY_5",
            "https://uor.foundation/op/Algebraic",
        ),
        (
            "https://uor.foundation/op/CY_6",
            "https://uor.foundation/op/Algebraic",
        ),
        (
            "https://uor.foundation/op/CY_7",
            "https://uor.foundation/op/Algebraic",
        ),
    ];

    let domain_prop = "https://uor.foundation/op/verificationDomain";
    let mut all_valid = true;

    for (iri, expected_domain) in cy_ids {
        match ontology.find_individual(iri) {
            Some(ind) => {
                let has_domain = ind.properties.iter().any(|(k, v)| {
                    *k == domain_prop
                        && matches!(
                            v,
                            IndividualValue::IriRef(d) if *d == *expected_domain
                        )
                });
                if !has_domain {
                    report.push(TestResult::fail(
                        validator,
                        format!("{} missing verificationDomain {}", iri, expected_domain),
                    ));
                    all_valid = false;
                }
            }
            None => {
                report.push(TestResult::fail(
                    validator,
                    format!("Identity {} not found", iri),
                ));
                all_valid = false;
            }
        }
    }

    if all_valid {
        report.push(TestResult::pass(
            validator,
            "All 7 CY_ identities present with correct verificationDomains",
        ));
    }
}

/// Validates the named base metrics vocabulary (Amendment 59).
fn validate_named_base_metrics(report: &mut ConformanceReport) {
    let ontology = uor_ontology::Ontology::full();
    let validator = "ontology/inventory/named_base_metrics";

    let bm_ids: &[(&str, &str)] = &[
        (
            "https://uor.foundation/op/BM_1",
            "https://uor.foundation/op/IndexTheoretic",
        ),
        (
            "https://uor.foundation/op/BM_2",
            "https://uor.foundation/op/IndexTheoretic",
        ),
        (
            "https://uor.foundation/op/BM_3",
            "https://uor.foundation/op/IndexTheoretic",
        ),
        (
            "https://uor.foundation/op/BM_4",
            "https://uor.foundation/op/IndexTheoretic",
        ),
        (
            "https://uor.foundation/op/BM_5",
            "https://uor.foundation/op/IndexTheoretic",
        ),
        (
            "https://uor.foundation/op/BM_6",
            "https://uor.foundation/op/IndexTheoretic",
        ),
    ];

    let domain_prop = "https://uor.foundation/op/verificationDomain";
    let mut all_valid = true;

    for (iri, expected_domain) in bm_ids {
        match ontology.find_individual(iri) {
            Some(ind) => {
                let has_domain = ind.properties.iter().any(|(k, v)| {
                    *k == domain_prop
                        && matches!(
                            v,
                            IndividualValue::IriRef(d) if *d == *expected_domain
                        )
                });
                if !has_domain {
                    report.push(TestResult::fail(
                        validator,
                        format!("{} missing verificationDomain {}", iri, expected_domain),
                    ));
                    all_valid = false;
                }
            }
            None => {
                report.push(TestResult::fail(
                    validator,
                    format!("Identity {} not found", iri),
                ));
                all_valid = false;
            }
        }
    }

    if all_valid {
        report.push(TestResult::pass(
            validator,
            "All 6 BM_ identities present with correct verificationDomains",
        ));
    }
}

/// Validates the Galois connection and nerve operations vocabulary (Amendment 60).
fn validate_galois_nerve_vocabulary(report: &mut ConformanceReport) {
    let ontology = uor_ontology::Ontology::full();
    let validator = "ontology/inventory/galois_nerve_vocabulary";

    let ids: &[(&str, &str)] = &[
        (
            "https://uor.foundation/op/GL_1",
            "https://uor.foundation/op/Topological",
        ),
        (
            "https://uor.foundation/op/GL_2",
            "https://uor.foundation/op/Topological",
        ),
        (
            "https://uor.foundation/op/GL_3",
            "https://uor.foundation/op/Topological",
        ),
        (
            "https://uor.foundation/op/GL_4",
            "https://uor.foundation/op/Topological",
        ),
        (
            "https://uor.foundation/op/NV_1",
            "https://uor.foundation/op/Topological",
        ),
        (
            "https://uor.foundation/op/NV_2",
            "https://uor.foundation/op/Topological",
        ),
        (
            "https://uor.foundation/op/NV_3",
            "https://uor.foundation/op/Topological",
        ),
        (
            "https://uor.foundation/op/NV_4",
            "https://uor.foundation/op/Topological",
        ),
    ];

    let domain_prop = "https://uor.foundation/op/verificationDomain";
    let mut all_valid = true;

    for (iri, expected_domain) in ids {
        match ontology.find_individual(iri) {
            Some(ind) => {
                let has_domain = ind.properties.iter().any(|(k, v)| {
                    *k == domain_prop
                        && matches!(
                            v,
                            IndividualValue::IriRef(d) if *d == *expected_domain
                        )
                });
                if !has_domain {
                    report.push(TestResult::fail(
                        validator,
                        format!("{} missing verificationDomain {}", iri, expected_domain),
                    ));
                    all_valid = false;
                }
            }
            None => {
                report.push(TestResult::fail(
                    validator,
                    format!("Identity {} not found", iri),
                ));
                all_valid = false;
            }
        }
    }

    if all_valid {
        report.push(TestResult::pass(
            validator,
            "All 8 GL_/NV_ identities present with correct verificationDomains",
        ));
    }
}

/// Validates that all 8 SD_ structural type identities are present with
/// correct Algebraic verification domain.
fn validate_structural_types_vocabulary(report: &mut ConformanceReport) {
    let ontology = uor_ontology::Ontology::full();
    let validator = "ontology/inventory/structural_types_vocabulary";

    let ids: &[(&str, &str)] = &[
        (
            "https://uor.foundation/op/SD_1",
            "https://uor.foundation/op/Algebraic",
        ),
        (
            "https://uor.foundation/op/SD_2",
            "https://uor.foundation/op/Algebraic",
        ),
        (
            "https://uor.foundation/op/SD_3",
            "https://uor.foundation/op/Algebraic",
        ),
        (
            "https://uor.foundation/op/SD_4",
            "https://uor.foundation/op/Algebraic",
        ),
        (
            "https://uor.foundation/op/SD_5",
            "https://uor.foundation/op/Algebraic",
        ),
        (
            "https://uor.foundation/op/SD_6",
            "https://uor.foundation/op/Algebraic",
        ),
        (
            "https://uor.foundation/op/SD_7",
            "https://uor.foundation/op/Algebraic",
        ),
        (
            "https://uor.foundation/op/SD_8",
            "https://uor.foundation/op/Algebraic",
        ),
    ];

    let domain_prop = "https://uor.foundation/op/verificationDomain";
    let mut all_valid = true;

    for (iri, expected_domain) in ids {
        match ontology.find_individual(iri) {
            Some(ind) => {
                let has_domain = ind.properties.iter().any(|(k, v)| {
                    *k == domain_prop
                        && matches!(
                            v,
                            IndividualValue::IriRef(d) if *d == *expected_domain
                        )
                });
                if !has_domain {
                    report.push(TestResult::fail(
                        validator,
                        format!("{} missing verificationDomain {}", iri, expected_domain),
                    ));
                    all_valid = false;
                }
            }
            None => {
                report.push(TestResult::fail(
                    validator,
                    format!("Identity {} not found", iri),
                ));
                all_valid = false;
            }
        }
    }

    if all_valid {
        report.push(TestResult::pass(
            validator,
            "All 8 SD_ identities present with correct verificationDomains",
        ));
    }
}

/// Validates that all 18 DD_/PI_/PA_/PL_/PK_/PP_ composed operation identities
/// are present with correct ComposedAlgebraic verification domain.
fn validate_composed_ops_vocabulary(report: &mut ConformanceReport) {
    let ontology = uor_ontology::Ontology::full();
    let validator = "ontology/inventory/composed_ops_vocabulary";

    let ids: &[(&str, &str)] = &[
        (
            "https://uor.foundation/op/DD_1",
            "https://uor.foundation/op/ComposedAlgebraic",
        ),
        (
            "https://uor.foundation/op/DD_2",
            "https://uor.foundation/op/ComposedAlgebraic",
        ),
        (
            "https://uor.foundation/op/PI_1",
            "https://uor.foundation/op/ComposedAlgebraic",
        ),
        (
            "https://uor.foundation/op/PI_2",
            "https://uor.foundation/op/ComposedAlgebraic",
        ),
        (
            "https://uor.foundation/op/PI_3",
            "https://uor.foundation/op/ComposedAlgebraic",
        ),
        (
            "https://uor.foundation/op/PI_4",
            "https://uor.foundation/op/ComposedAlgebraic",
        ),
        (
            "https://uor.foundation/op/PI_5",
            "https://uor.foundation/op/ComposedAlgebraic",
        ),
        (
            "https://uor.foundation/op/PA_1",
            "https://uor.foundation/op/ComposedAlgebraic",
        ),
        (
            "https://uor.foundation/op/PA_2",
            "https://uor.foundation/op/ComposedAlgebraic",
        ),
        (
            "https://uor.foundation/op/PA_3",
            "https://uor.foundation/op/ComposedAlgebraic",
        ),
        (
            "https://uor.foundation/op/PA_4",
            "https://uor.foundation/op/ComposedAlgebraic",
        ),
        (
            "https://uor.foundation/op/PA_5",
            "https://uor.foundation/op/ComposedAlgebraic",
        ),
        (
            "https://uor.foundation/op/PL_1",
            "https://uor.foundation/op/ComposedAlgebraic",
        ),
        (
            "https://uor.foundation/op/PL_2",
            "https://uor.foundation/op/ComposedAlgebraic",
        ),
        (
            "https://uor.foundation/op/PL_3",
            "https://uor.foundation/op/ComposedAlgebraic",
        ),
        (
            "https://uor.foundation/op/PK_1",
            "https://uor.foundation/op/ComposedAlgebraic",
        ),
        (
            "https://uor.foundation/op/PK_2",
            "https://uor.foundation/op/ComposedAlgebraic",
        ),
        (
            "https://uor.foundation/op/PP_1",
            "https://uor.foundation/op/ComposedAlgebraic",
        ),
    ];

    let domain_prop = "https://uor.foundation/op/verificationDomain";
    let mut all_valid = true;

    for (iri, expected_domain) in ids {
        match ontology.find_individual(iri) {
            Some(ind) => {
                let has_domain = ind.properties.iter().any(|(k, v)| {
                    *k == domain_prop
                        && matches!(
                            v,
                            IndividualValue::IriRef(d) if *d == *expected_domain
                        )
                });
                if !has_domain {
                    report.push(TestResult::fail(
                        validator,
                        format!("{} missing verificationDomain {}", iri, expected_domain),
                    ));
                    all_valid = false;
                }
            }
            None => {
                report.push(TestResult::fail(
                    validator,
                    format!("Identity {} not found", iri),
                ));
                all_valid = false;
            }
        }
    }

    if all_valid {
        report.push(TestResult::pass(
            validator,
            "All 18 DD_/PI_/PA_/PL_/PK_/PP_ identities present with correct verificationDomains",
        ));
    }
}

/// Validates the reduction core vocabulary (Amendment 63).
fn validate_reduction_core_vocabulary(report: &mut ConformanceReport) {
    let ontology = uor_ontology::Ontology::full();
    let validator = "ontology/inventory/reduction_core_vocabulary";

    let ids: &[(&str, &str)] = &[
        (
            "https://uor.foundation/op/PE_1",
            "https://uor.foundation/op/Pipeline",
        ),
        (
            "https://uor.foundation/op/PE_2",
            "https://uor.foundation/op/Pipeline",
        ),
        (
            "https://uor.foundation/op/PE_3",
            "https://uor.foundation/op/Pipeline",
        ),
        (
            "https://uor.foundation/op/PE_4",
            "https://uor.foundation/op/Pipeline",
        ),
        (
            "https://uor.foundation/op/PE_5",
            "https://uor.foundation/op/Pipeline",
        ),
        (
            "https://uor.foundation/op/PE_6",
            "https://uor.foundation/op/Pipeline",
        ),
        (
            "https://uor.foundation/op/PE_7",
            "https://uor.foundation/op/Pipeline",
        ),
        (
            "https://uor.foundation/op/PM_1",
            "https://uor.foundation/op/Pipeline",
        ),
        (
            "https://uor.foundation/op/PM_2",
            "https://uor.foundation/op/Pipeline",
        ),
        (
            "https://uor.foundation/op/PM_3",
            "https://uor.foundation/op/Pipeline",
        ),
        (
            "https://uor.foundation/op/PM_4",
            "https://uor.foundation/op/Pipeline",
        ),
        (
            "https://uor.foundation/op/PM_5",
            "https://uor.foundation/op/Pipeline",
        ),
        (
            "https://uor.foundation/op/PM_6",
            "https://uor.foundation/op/Pipeline",
        ),
        (
            "https://uor.foundation/op/PM_7",
            "https://uor.foundation/op/Pipeline",
        ),
        (
            "https://uor.foundation/op/ER_1",
            "https://uor.foundation/op/Pipeline",
        ),
        (
            "https://uor.foundation/op/ER_2",
            "https://uor.foundation/op/Pipeline",
        ),
    ];

    let domain_prop = "https://uor.foundation/op/verificationDomain";
    let mut all_valid = true;

    for (iri, expected_domain) in ids {
        match ontology.find_individual(iri) {
            Some(ind) => {
                let has_domain = ind.properties.iter().any(|(k, v)| {
                    *k == domain_prop
                        && matches!(
                            v,
                            IndividualValue::IriRef(d) if *d == *expected_domain
                        )
                });
                if !has_domain {
                    report.push(TestResult::fail(
                        validator,
                        format!("{} missing verificationDomain {}", iri, expected_domain),
                    ));
                    all_valid = false;
                }
            }
            None => {
                report.push(TestResult::fail(
                    validator,
                    format!("Identity {} not found", iri),
                ));
                all_valid = false;
            }
        }
    }

    if all_valid {
        report.push(TestResult::pass(
            validator,
            "All 16 PE_/PM_/ER_ reduction core identities present with correct verificationDomains",
        ));
    }
}

/// Validates the reduction expansion vocabulary (Amendment 64).
fn validate_reduction_expansion_vocabulary(report: &mut ConformanceReport) {
    let ontology = uor_ontology::Ontology::full();
    let validator = "ontology/inventory/reduction_expansion_vocabulary";

    let ids: &[(&str, &str)] = &[
        (
            "https://uor.foundation/op/ER_3",
            "https://uor.foundation/op/Pipeline",
        ),
        (
            "https://uor.foundation/op/ER_4",
            "https://uor.foundation/op/Pipeline",
        ),
        (
            "https://uor.foundation/op/EA_1",
            "https://uor.foundation/op/Pipeline",
        ),
        (
            "https://uor.foundation/op/EA_2",
            "https://uor.foundation/op/Pipeline",
        ),
        (
            "https://uor.foundation/op/EA_3",
            "https://uor.foundation/op/Pipeline",
        ),
        (
            "https://uor.foundation/op/EA_4",
            "https://uor.foundation/op/Pipeline",
        ),
        (
            "https://uor.foundation/op/OE_1",
            "https://uor.foundation/op/Pipeline",
        ),
        (
            "https://uor.foundation/op/OE_2",
            "https://uor.foundation/op/Pipeline",
        ),
        (
            "https://uor.foundation/op/OE_3",
            "https://uor.foundation/op/Pipeline",
        ),
        (
            "https://uor.foundation/op/OE_4a",
            "https://uor.foundation/op/Pipeline",
        ),
        (
            "https://uor.foundation/op/OE_4b",
            "https://uor.foundation/op/Pipeline",
        ),
        (
            "https://uor.foundation/op/OE_4c",
            "https://uor.foundation/op/Pipeline",
        ),
        (
            "https://uor.foundation/op/CS_1",
            "https://uor.foundation/op/Pipeline",
        ),
        (
            "https://uor.foundation/op/CS_2",
            "https://uor.foundation/op/Pipeline",
        ),
        (
            "https://uor.foundation/op/CS_3",
            "https://uor.foundation/op/Pipeline",
        ),
        (
            "https://uor.foundation/op/CS_4",
            "https://uor.foundation/op/Pipeline",
        ),
    ];

    let domain_prop = "https://uor.foundation/op/verificationDomain";
    let mut all_valid = true;

    for (iri, expected_domain) in ids {
        match ontology.find_individual(iri) {
            Some(ind) => {
                let has_domain = ind.properties.iter().any(|(k, v)| {
                    *k == domain_prop
                        && matches!(
                            v,
                            IndividualValue::IriRef(d) if *d == *expected_domain
                        )
                });
                if !has_domain {
                    report.push(TestResult::fail(
                        validator,
                        format!("{} missing verificationDomain {}", iri, expected_domain),
                    ));
                    all_valid = false;
                }
            }
            None => {
                report.push(TestResult::fail(
                    validator,
                    format!("Identity {} not found", iri),
                ));
                all_valid = false;
            }
        }
    }

    if all_valid {
        report.push(TestResult::pass(
            validator,
            "All 16 ER_/EA_/OE_/CS_ reduction expansion identities present with correct verificationDomains",
        ));
    }
}

/// Validates the reduction completion vocabulary (Amendment 65).
fn validate_reduction_completion_vocabulary(report: &mut ConformanceReport) {
    let ontology = uor_ontology::Ontology::full();
    let validator = "ontology/inventory/reduction_completion_vocabulary";

    let ids: &[(&str, &str)] = &[
        (
            "https://uor.foundation/op/CS_5",
            "https://uor.foundation/op/Pipeline",
        ),
        (
            "https://uor.foundation/op/FA_1",
            "https://uor.foundation/op/Pipeline",
        ),
        (
            "https://uor.foundation/op/FA_2",
            "https://uor.foundation/op/Pipeline",
        ),
        (
            "https://uor.foundation/op/FA_3",
            "https://uor.foundation/op/Pipeline",
        ),
        (
            "https://uor.foundation/op/SW_1",
            "https://uor.foundation/op/Pipeline",
        ),
        (
            "https://uor.foundation/op/SW_2",
            "https://uor.foundation/op/Pipeline",
        ),
        (
            "https://uor.foundation/op/SW_3",
            "https://uor.foundation/op/Pipeline",
        ),
        (
            "https://uor.foundation/op/SW_4",
            "https://uor.foundation/op/Pipeline",
        ),
        (
            "https://uor.foundation/op/LS_1",
            "https://uor.foundation/op/Pipeline",
        ),
        (
            "https://uor.foundation/op/LS_2",
            "https://uor.foundation/op/Pipeline",
        ),
        (
            "https://uor.foundation/op/LS_3",
            "https://uor.foundation/op/Pipeline",
        ),
        (
            "https://uor.foundation/op/LS_4",
            "https://uor.foundation/op/Pipeline",
        ),
        (
            "https://uor.foundation/op/TJ_1",
            "https://uor.foundation/op/Pipeline",
        ),
        (
            "https://uor.foundation/op/TJ_2",
            "https://uor.foundation/op/Pipeline",
        ),
        (
            "https://uor.foundation/op/TJ_3",
            "https://uor.foundation/op/Pipeline",
        ),
        (
            "https://uor.foundation/op/AP_1",
            "https://uor.foundation/op/Pipeline",
        ),
        (
            "https://uor.foundation/op/AP_2",
            "https://uor.foundation/op/Pipeline",
        ),
        (
            "https://uor.foundation/op/AP_3",
            "https://uor.foundation/op/Pipeline",
        ),
    ];

    let domain_prop = "https://uor.foundation/op/verificationDomain";
    let mut all_valid = true;

    for (iri, expected_domain) in ids {
        match ontology.find_individual(iri) {
            Some(ind) => {
                let has_domain = ind.properties.iter().any(|(k, v)| {
                    *k == domain_prop
                        && matches!(
                            v,
                            IndividualValue::IriRef(d) if *d == *expected_domain
                        )
                });
                if !has_domain {
                    report.push(TestResult::fail(
                        validator,
                        format!("{} missing verificationDomain {}", iri, expected_domain),
                    ));
                    all_valid = false;
                }
            }
            None => {
                report.push(TestResult::fail(
                    validator,
                    format!("Identity {} not found", iri),
                ));
                all_valid = false;
            }
        }
    }

    if all_valid {
        report.push(TestResult::pass(
            validator,
            "All 18 CS_5/FA_/SW_/LS_/TJ_/AP_ reduction completion identities present with correct verificationDomains",
        ));
    }
}

/// Validates the convergence tower vocabulary (Amendment 66).
fn validate_convergence_tower_vocabulary(report: &mut ConformanceReport) {
    let ontology = uor_ontology::Ontology::full();
    let validator = "ontology/inventory/convergence_tower_vocabulary";

    let ids: &[(&str, &str)] = &[
        (
            "https://uor.foundation/op/EC_1",
            "https://uor.foundation/op/Topological",
        ),
        (
            "https://uor.foundation/op/EC_2",
            "https://uor.foundation/op/Topological",
        ),
        (
            "https://uor.foundation/op/EC_3",
            "https://uor.foundation/op/Topological",
        ),
        (
            "https://uor.foundation/op/EC_4",
            "https://uor.foundation/op/Topological",
        ),
        (
            "https://uor.foundation/op/EC_4a",
            "https://uor.foundation/op/Topological",
        ),
        (
            "https://uor.foundation/op/EC_4b",
            "https://uor.foundation/op/Topological",
        ),
        (
            "https://uor.foundation/op/EC_4c",
            "https://uor.foundation/op/Topological",
        ),
        (
            "https://uor.foundation/op/EC_5",
            "https://uor.foundation/op/Topological",
        ),
    ];

    let domain_prop = "https://uor.foundation/op/verificationDomain";
    let mut all_valid = true;

    for (iri, expected_domain) in ids {
        match ontology.find_individual(iri) {
            Some(ind) => {
                let has_domain = ind.properties.iter().any(|(k, v)| {
                    *k == domain_prop
                        && matches!(
                            v,
                            IndividualValue::IriRef(d) if *d == *expected_domain
                        )
                });
                if !has_domain {
                    report.push(TestResult::fail(
                        validator,
                        format!("{} missing verificationDomain {}", iri, expected_domain),
                    ));
                    all_valid = false;
                }
            }
            None => {
                report.push(TestResult::fail(
                    validator,
                    format!("Identity {} not found", iri),
                ));
                all_valid = false;
            }
        }
    }

    if all_valid {
        report.push(TestResult::pass(
            validator,
            "All 8 EC_ convergence tower identities present with correct verificationDomains",
        ));
    }
}

/// ADR-059: pins the four `convergence:ConvergenceLevel` individuals as the
/// operator-geometry-coordinate codomain stratification of κ-derivation
/// (ADR-058). ADR-059 §4 commits the Hopf convergence tower (R / C / H / O at
/// division-algebra dimensions {1, 2, 4, 8}) as the coarse codomain
/// stratification; this validator guards the exact normative values the ADR
/// makes load-bearing — algebra dimension, characteristic identity, Hopf fiber
/// sphere, and the persistent residual Betti signature (β_0 = 1 and
/// β_{2^k − 1} = 1 at each level). Without this guard, a regression in the
/// ontology-generated tower would silently break ADR-059's codomain commitment.
fn validate_convergence_codomain_stratification(report: &mut ConformanceReport) {
    let ontology = uor_ontology::Ontology::full();
    let validator = "ontology/inventory/convergence_codomain_stratification";

    // (level IRI, algebra_dimension, characteristic_identity, fiber IRI,
    //  betti_signature). The four normed division algebras R/C/H/O per
    //  ADR-059 §4; the persistent Betti at index 2^k − 1 encodes the Hopf
    //  fibration's residual β = 1 at each level.
    let levels: &[(&str, i64, &str, &str, &str)] = &[
        (
            "https://uor.foundation/convergence/L0_State",
            1,
            "existence",
            "https://uor.foundation/convergence/hopf_S0",
            "[1]",
        ),
        (
            "https://uor.foundation/convergence/L1_Memory",
            2,
            "feedback",
            "https://uor.foundation/convergence/hopf_S1",
            "[1,1]",
        ),
        (
            "https://uor.foundation/convergence/L2_Agency",
            4,
            "choice",
            "https://uor.foundation/convergence/hopf_S3",
            "[1,0,0,1]",
        ),
        (
            "https://uor.foundation/convergence/L3_Self",
            8,
            "self-reference",
            "https://uor.foundation/convergence/hopf_S7",
            "[1,0,0,0,0,0,0,1]",
        ),
    ];

    let dim_prop = "https://uor.foundation/convergence/algebraDimension";
    let betti_prop = "https://uor.foundation/convergence/bettiSignature";
    let fiber_prop = "https://uor.foundation/convergence/fiberType";
    let identity_prop = "https://uor.foundation/convergence/characteristicIdentity";
    let mut all_valid = true;

    for (iri, dim, identity, fiber, betti) in levels {
        match ontology.find_individual(iri) {
            Some(ind) => {
                let prop_eq = |key: &str, expect: &IndividualValue| {
                    ind.properties.iter().any(|(k, v)| *k == key && v == expect)
                };
                if !prop_eq(dim_prop, &IndividualValue::Int(*dim)) {
                    report.push(TestResult::fail(
                        validator,
                        format!("{iri} algebraDimension is not {dim} (ADR-059 §4 tower)"),
                    ));
                    all_valid = false;
                }
                if !prop_eq(identity_prop, &IndividualValue::Str(identity)) {
                    report.push(TestResult::fail(
                        validator,
                        format!("{iri} characteristicIdentity is not \"{identity}\""),
                    ));
                    all_valid = false;
                }
                if !prop_eq(fiber_prop, &IndividualValue::IriRef(fiber)) {
                    report.push(TestResult::fail(
                        validator,
                        format!("{iri} fiberType is not {fiber}"),
                    ));
                    all_valid = false;
                }
                if !prop_eq(betti_prop, &IndividualValue::Str(betti)) {
                    report.push(TestResult::fail(
                        validator,
                        format!("{iri} bettiSignature is not {betti} (persistent β_{{2^k-1}} = 1)"),
                    ));
                    all_valid = false;
                }
            }
            None => {
                report.push(TestResult::fail(
                    validator,
                    format!("ConvergenceLevel {iri} not found (ADR-059 codomain stratification)"),
                ));
                all_valid = false;
            }
        }
    }

    // Exactly four levels — the four normed division algebras, no more.
    let level_count = ontology
        .find_namespace("convergence")
        .map(|ns| {
            ns.individuals
                .iter()
                .filter(|i| i.type_ == "https://uor.foundation/convergence/ConvergenceLevel")
                .count()
        })
        .unwrap_or(0);
    if level_count != 4 {
        report.push(TestResult::fail(
            validator,
            format!("expected exactly 4 ConvergenceLevel individuals, found {level_count}"),
        ));
        all_valid = false;
    }

    if all_valid {
        report.push(TestResult::pass(
            validator,
            "ADR-059: Hopf convergence tower {R,C,H,O} at dims {1,2,4,8} carries the \
             committed codomain stratification (identity, fiber sphere, persistent Betti)",
        ));
    }
}

fn validate_division_algebras_vocabulary(report: &mut ConformanceReport) {
    let ontology = uor_ontology::Ontology::full();
    let validator = "ontology/inventory/division_algebras_vocabulary";

    let ids: &[(&str, &str)] = &[
        (
            "https://uor.foundation/op/DA_1",
            "https://uor.foundation/op/Algebraic",
        ),
        (
            "https://uor.foundation/op/DA_2",
            "https://uor.foundation/op/Algebraic",
        ),
        (
            "https://uor.foundation/op/DA_3",
            "https://uor.foundation/op/Algebraic",
        ),
        (
            "https://uor.foundation/op/DA_4",
            "https://uor.foundation/op/Algebraic",
        ),
        (
            "https://uor.foundation/op/DA_5",
            "https://uor.foundation/op/Algebraic",
        ),
        (
            "https://uor.foundation/op/DA_6",
            "https://uor.foundation/op/Algebraic",
        ),
        (
            "https://uor.foundation/op/DA_7",
            "https://uor.foundation/op/Algebraic",
        ),
    ];

    let domain_prop = "https://uor.foundation/op/verificationDomain";
    let mut all_valid = true;

    for (iri, expected_domain) in ids {
        match ontology.find_individual(iri) {
            Some(ind) => {
                let has_domain = ind.properties.iter().any(|(k, v)| {
                    *k == domain_prop
                        && matches!(
                            v,
                            IndividualValue::IriRef(d) if *d == *expected_domain
                        )
                });
                if !has_domain {
                    report.push(TestResult::fail(
                        validator,
                        format!("{} missing verificationDomain {}", iri, expected_domain),
                    ));
                    all_valid = false;
                }
            }
            None => {
                report.push(TestResult::fail(
                    validator,
                    format!("Identity {} not found", iri),
                ));
                all_valid = false;
            }
        }
    }

    if all_valid {
        report.push(TestResult::pass(
            validator,
            "All 7 DA_ division algebra identities present with correct verificationDomains",
        ));
    }
}

fn validate_interaction_algebra_vocabulary(report: &mut ConformanceReport) {
    let ontology = uor_ontology::Ontology::full();
    let validator = "ontology/inventory/interaction_algebra_vocabulary";

    let ids: &[(&str, &str)] = &[
        (
            "https://uor.foundation/op/IN_1",
            "https://uor.foundation/op/ComposedAlgebraic",
        ),
        (
            "https://uor.foundation/op/IN_2",
            "https://uor.foundation/op/ComposedAlgebraic",
        ),
        (
            "https://uor.foundation/op/IN_3",
            "https://uor.foundation/op/ComposedAlgebraic",
        ),
        (
            "https://uor.foundation/op/IN_4",
            "https://uor.foundation/op/ComposedAlgebraic",
        ),
        (
            "https://uor.foundation/op/IN_5",
            "https://uor.foundation/op/ComposedAlgebraic",
        ),
        (
            "https://uor.foundation/op/IN_6",
            "https://uor.foundation/op/ComposedAlgebraic",
        ),
        (
            "https://uor.foundation/op/IN_7",
            "https://uor.foundation/op/ComposedAlgebraic",
        ),
        (
            "https://uor.foundation/op/IN_8",
            "https://uor.foundation/op/ComposedAlgebraic",
        ),
        (
            "https://uor.foundation/op/IN_9",
            "https://uor.foundation/op/ComposedAlgebraic",
        ),
        (
            "https://uor.foundation/op/AS_1",
            "https://uor.foundation/op/ComposedAlgebraic",
        ),
        (
            "https://uor.foundation/op/AS_2",
            "https://uor.foundation/op/ComposedAlgebraic",
        ),
        (
            "https://uor.foundation/op/AS_3",
            "https://uor.foundation/op/ComposedAlgebraic",
        ),
        (
            "https://uor.foundation/op/AS_4",
            "https://uor.foundation/op/ComposedAlgebraic",
        ),
    ];

    let domain_prop = "https://uor.foundation/op/verificationDomain";
    let mut all_valid = true;

    for (iri, expected_domain) in ids {
        match ontology.find_individual(iri) {
            Some(ind) => {
                let has_domain = ind.properties.iter().any(|(k, v)| {
                    *k == domain_prop
                        && matches!(
                            v,
                            IndividualValue::IriRef(d) if *d == *expected_domain
                        )
                });
                if !has_domain {
                    report.push(TestResult::fail(
                        validator,
                        format!("{} missing verificationDomain {}", iri, expected_domain),
                    ));
                    all_valid = false;
                }
            }
            None => {
                report.push(TestResult::fail(
                    validator,
                    format!("Identity {} not found", iri),
                ));
                all_valid = false;
            }
        }
    }

    if all_valid {
        report.push(TestResult::pass(
            validator,
            "All 13 IN_/AS_ interaction algebra identities present with correct verificationDomains",
        ));
    }
}

fn validate_monoidal_composition_vocabulary(report: &mut ConformanceReport) {
    let ontology = uor_ontology::Ontology::full();
    let validator = "ontology/inventory/monoidal_composition_vocabulary";

    // Verify 3 monoidal classes exist
    let expected_classes = [
        "https://uor.foundation/monoidal/MonoidalProduct",
        "https://uor.foundation/monoidal/MonoidalUnit",
        "https://uor.foundation/monoidal/MonoidalAssociator",
    ];

    let mut all_valid = true;
    for class_iri in &expected_classes {
        if ontology.find_class(class_iri).is_none() {
            report.push(TestResult::fail(
                validator,
                format!("Missing monoidal class: {}", class_iri),
            ));
            all_valid = false;
        }
    }

    // Verify 5 MO_ identities with Algebraic verification domain
    let ids = [
        (
            "https://uor.foundation/op/MO_1",
            "https://uor.foundation/op/Algebraic",
        ),
        (
            "https://uor.foundation/op/MO_2",
            "https://uor.foundation/op/Algebraic",
        ),
        (
            "https://uor.foundation/op/MO_3",
            "https://uor.foundation/op/Algebraic",
        ),
        (
            "https://uor.foundation/op/MO_4",
            "https://uor.foundation/op/Algebraic",
        ),
        (
            "https://uor.foundation/op/MO_5",
            "https://uor.foundation/op/Algebraic",
        ),
    ];

    let domain_prop = "https://uor.foundation/op/verificationDomain";

    for (iri, expected_domain) in ids {
        match ontology.find_individual(iri) {
            Some(ind) => {
                let has_domain = ind.properties.iter().any(|(k, v)| {
                    *k == domain_prop
                        && matches!(
                            v,
                            IndividualValue::IriRef(d) if *d == expected_domain
                        )
                });
                if !has_domain {
                    report.push(TestResult::fail(
                        validator,
                        format!("{} missing verificationDomain {}", iri, expected_domain),
                    ));
                    all_valid = false;
                }
            }
            None => {
                report.push(TestResult::fail(
                    validator,
                    format!("Identity {} not found", iri),
                ));
                all_valid = false;
            }
        }
    }

    if all_valid {
        report.push(TestResult::pass(
            validator,
            "All 3 monoidal classes and 5 MO_ identities present with correct verificationDomains",
        ));
    }
}

fn validate_operad_composition_vocabulary(report: &mut ConformanceReport) {
    let ontology = uor_ontology::Ontology::full();
    let validator = "ontology/inventory/operad_composition_vocabulary";

    // Verify 2 operad classes exist
    let expected_classes = [
        "https://uor.foundation/operad/StructuralOperad",
        "https://uor.foundation/operad/OperadComposition",
    ];

    let mut all_valid = true;
    for class_iri in &expected_classes {
        if ontology.find_class(class_iri).is_none() {
            report.push(TestResult::fail(
                validator,
                format!("Missing operad class: {}", class_iri),
            ));
            all_valid = false;
        }
    }

    // Verify 5 OP_ identities with Algebraic verification domain
    let ids = [
        (
            "https://uor.foundation/op/OP_1",
            "https://uor.foundation/op/Algebraic",
        ),
        (
            "https://uor.foundation/op/OP_2",
            "https://uor.foundation/op/Algebraic",
        ),
        (
            "https://uor.foundation/op/OP_3",
            "https://uor.foundation/op/Algebraic",
        ),
        (
            "https://uor.foundation/op/OP_4",
            "https://uor.foundation/op/Algebraic",
        ),
        (
            "https://uor.foundation/op/OP_5",
            "https://uor.foundation/op/Algebraic",
        ),
    ];

    let domain_prop = "https://uor.foundation/op/verificationDomain";

    for (iri, expected_domain) in ids {
        match ontology.find_individual(iri) {
            Some(ind) => {
                let has_domain = ind.properties.iter().any(|(k, v)| {
                    *k == domain_prop
                        && matches!(
                            v,
                            IndividualValue::IriRef(d) if *d == expected_domain
                        )
                });
                if !has_domain {
                    report.push(TestResult::fail(
                        validator,
                        format!("{} missing verificationDomain {}", iri, expected_domain),
                    ));
                    all_valid = false;
                }
            }
            None => {
                report.push(TestResult::fail(
                    validator,
                    format!("Identity {} not found", iri),
                ));
                all_valid = false;
            }
        }
    }

    if all_valid {
        report.push(TestResult::pass(
            validator,
            "All 2 operad classes and 5 OP_ identities present with correct verificationDomains",
        ));
    }
}

fn validate_compile_unit_vocabulary(report: &mut ConformanceReport) {
    let ontology = uor_ontology::Ontology::full();
    let validator = "ontology/inventory/compile_unit_vocabulary";

    let mut all_valid = true;

    // Check CompileUnit class
    if ontology
        .find_class("https://uor.foundation/reduction/CompileUnit")
        .is_none()
    {
        report.push(TestResult::fail(
            validator,
            "reduction:CompileUnit class not found",
        ));
        all_valid = false;
    }

    // Check 6 new properties
    let prop_iris = [
        "https://uor.foundation/reduction/rootTerm",
        "https://uor.foundation/reduction/unitWittLevel",
        "https://uor.foundation/reduction/targetDomains",
        "https://uor.foundation/reduction/thermodynamicBudget",
        "https://uor.foundation/reduction/unitAddress",
        "https://uor.foundation/reduction/preflightOrder",
    ];
    for iri in &prop_iris {
        if ontology.find_property(iri).is_none() {
            report.push(TestResult::fail(
                validator,
                format!("Property {} not found", iri),
            ));
            all_valid = false;
        }
    }

    // Check BudgetSolvencyCheck individual
    if ontology
        .find_individual("https://uor.foundation/reduction/BudgetSolvencyCheck")
        .is_none()
    {
        report.push(TestResult::fail(
            validator,
            "reduction:BudgetSolvencyCheck individual not found",
        ));
        all_valid = false;
    }

    // Check all 6 PreflightCheck individuals have preflightOrder
    let preflight_iris = [
        "https://uor.foundation/reduction/BudgetSolvencyCheck",
        "https://uor.foundation/reduction/FeasibilityCheck",
        "https://uor.foundation/reduction/DispatchCoverageCheck",
        "https://uor.foundation/reduction/PackageCoherenceCheck",
        "https://uor.foundation/reduction/PreflightTiming",
        "https://uor.foundation/reduction/RuntimeTiming",
    ];
    let order_prop = "https://uor.foundation/reduction/preflightOrder";
    for iri in &preflight_iris {
        match ontology.find_individual(iri) {
            Some(ind) => {
                let has_order = ind.properties.iter().any(|(k, _)| *k == order_prop);
                if !has_order {
                    report.push(TestResult::fail(
                        validator,
                        format!("{} missing preflightOrder", iri),
                    ));
                    all_valid = false;
                }
            }
            None => {
                report.push(TestResult::fail(
                    validator,
                    format!("PreflightCheck {} not found", iri),
                ));
                all_valid = false;
            }
        }
    }

    if all_valid {
        report.push(TestResult::pass(
            validator,
            "CompileUnit class, 6 properties, BudgetSolvencyCheck, and \
             preflightOrder on all 6 PreflightCheck individuals verified",
        ));
    }
}

fn validate_compile_unit_identities(report: &mut ConformanceReport) {
    let ontology = uor_ontology::Ontology::full();
    let validator = "ontology/inventory/compile_unit_identities";

    let ids: &[(&str, &str)] = &[
        (
            "https://uor.foundation/op/CS_6",
            "https://uor.foundation/op/Pipeline",
        ),
        (
            "https://uor.foundation/op/CS_7",
            "https://uor.foundation/op/Algebraic",
        ),
    ];

    let domain_prop = "https://uor.foundation/op/verificationDomain";
    let mut all_valid = true;

    for (iri, expected_domain) in ids {
        match ontology.find_individual(iri) {
            Some(ind) => {
                let has_domain = ind.properties.iter().any(|(k, v)| {
                    *k == domain_prop
                        && matches!(v, IndividualValue::IriRef(d) if *d == *expected_domain)
                });
                if !has_domain {
                    report.push(TestResult::fail(
                        validator,
                        format!("{} missing verificationDomain {}", iri, expected_domain),
                    ));
                    all_valid = false;
                }
            }
            None => {
                report.push(TestResult::fail(
                    validator,
                    format!("Identity {} not found", iri),
                ));
                all_valid = false;
            }
        }
    }

    if all_valid {
        report.push(TestResult::pass(
            validator,
            "CS_6 (Pipeline) and CS_7 (Algebraic) compile unit identities verified",
        ));
    }
}

/// Amendment 85: Validates that no `InductiveProof` has self-referential
/// `baseCase` or `inductiveStep` properties.
fn validate_inductive_proof_acyclicity(report: &mut ConformanceReport) {
    let ontology = uor_ontology::Ontology::full();
    let validator = "ontology/inventory/inductive_proof_acyclicity";

    let inductive_type = "https://uor.foundation/proof/InductiveProof";
    let base_case_prop = "https://uor.foundation/proof/baseCase";
    let step_prop = "https://uor.foundation/proof/inductiveStep";

    let mut violations = Vec::new();

    for ns in &ontology.namespaces {
        for ind in &ns.individuals {
            if ind.type_ != inductive_type {
                continue;
            }
            for (prop, val) in ind.properties {
                if *prop == base_case_prop || *prop == step_prop {
                    if let IndividualValue::IriRef(iri) = val {
                        if *iri == ind.id {
                            violations.push(format!(
                                "{} has self-referential {} (points to itself)",
                                ind.label,
                                if *prop == base_case_prop {
                                    "baseCase"
                                } else {
                                    "inductiveStep"
                                },
                            ));
                        }
                    }
                }
            }
        }
    }

    if violations.is_empty() {
        report.push(TestResult::pass(
            validator,
            "All InductiveProof baseCase/inductiveStep references are non-self-referential",
        ));
    } else {
        for v in &violations {
            report.push(TestResult::fail(validator, v.clone()));
        }
    }
}

/// Amendment 86: Validates that no `EmpiricalVerification` class or instances
/// exist in the ontology.
fn validate_no_empirical_verification(report: &mut ConformanceReport) {
    let ontology = uor_ontology::Ontology::full();

    // Check 1: No EmpiricalVerification class
    let ev_class_id = "https://uor.foundation/proof/EmpiricalVerification";
    let class_exists = ontology
        .namespaces
        .iter()
        .flat_map(|m| m.classes.iter())
        .any(|c| c.id == ev_class_id);

    if class_exists {
        report.push(TestResult::fail(
            "ontology/inventory/no_empirical_verification_class",
            "proof/EmpiricalVerification class still exists in the ontology",
        ));
    } else {
        report.push(TestResult::pass(
            "ontology/inventory/no_empirical_verification_class",
            "proof/EmpiricalVerification class has been removed",
        ));
    }

    // Check 2: No EmpiricalVerification instances
    let mut ev_instances = Vec::new();
    for ns in &ontology.namespaces {
        for ind in &ns.individuals {
            if ind.type_ == ev_class_id {
                ev_instances.push(ind.label.to_string());
            }
        }
    }

    if ev_instances.is_empty() {
        report.push(TestResult::pass(
            "ontology/inventory/no_empirical_verification_instances",
            "Zero EmpiricalVerification instances exist",
        ));
    } else {
        for label in &ev_instances {
            report.push(TestResult::fail(
                "ontology/inventory/no_empirical_verification_instances",
                format!("{} is still typed as EmpiricalVerification", label),
            ));
        }
    }
}

/// Amendment 87: Validates that every Proof individual has exactly one
/// `proof:strategy` property pointing to a `ProofStrategy` individual.
fn validate_proof_strategy_coverage(report: &mut ConformanceReport) {
    let ontology = uor_ontology::Ontology::full();
    let validator = "ontology/inventory/proof_strategy_coverage";

    let proof_superclass = "https://uor.foundation/proof/Proof";
    let strategy_prop = "https://uor.foundation/proof/strategy";

    // Collect all proof subclass IRIs
    let proof_subclasses: HashSet<&str> = ontology
        .namespaces
        .iter()
        .flat_map(|m| m.classes.iter())
        .filter(|c| c.subclass_of.contains(&proof_superclass))
        .map(|c| c.id)
        .chain(std::iter::once(proof_superclass))
        .collect();

    // Exempt types that are not proofs of identities
    let exempt_types: HashSet<&str> = [
        "https://uor.foundation/proof/MorphospaceRecord",
        "https://uor.foundation/proof/MorphospaceBoundary",
        "https://uor.foundation/proof/ProofStrategy",
        "https://uor.foundation/proof/WitnessData",
    ]
    .into_iter()
    .collect();

    let mut missing = Vec::new();

    if let Some(proof_module) = ontology.find_namespace("proof") {
        for ind in &proof_module.individuals {
            if exempt_types.contains(ind.type_) {
                continue;
            }
            if !proof_subclasses.contains(ind.type_) {
                continue;
            }
            let has_strategy = ind.properties.iter().any(|(k, _)| *k == strategy_prop);
            if !has_strategy {
                missing.push(ind.label.to_string());
            }
        }
    }

    if missing.is_empty() {
        report.push(TestResult::pass(
            validator,
            "Every Proof individual has a strategy property",
        ));
    } else {
        for label in &missing {
            report.push(TestResult::fail(
                validator,
                format!("{} missing proof:strategy", label),
            ));
        }
    }
}

/// Amendment 87: Validates that exactly 11 `ProofStrategy` individuals exist.
fn validate_proof_strategy_vocabulary(report: &mut ConformanceReport) {
    let ontology = uor_ontology::Ontology::full();
    let validator = "ontology/inventory/proof_strategy_vocabulary";

    let strategy_type = "https://uor.foundation/proof/ProofStrategy";
    let expected_count = 11;

    let count = ontology
        .namespaces
        .iter()
        .flat_map(|m| m.individuals.iter())
        .filter(|ind| ind.type_ == strategy_type)
        .count();

    if count == expected_count {
        report.push(TestResult::pass(
            validator,
            format!("Exactly {} ProofStrategy individuals exist", expected_count),
        ));
    } else {
        report.push(TestResult::fail(
            validator,
            format!(
                "Expected {} ProofStrategy individuals, found {}",
                expected_count, count
            ),
        ));
    }
}

/// Amendment 87: Validates that the `proof:dependsOn` relation forms a DAG
/// (no cycles in the dependency graph).
fn validate_proof_dependency_acyclicity(report: &mut ConformanceReport) {
    let ontology = uor_ontology::Ontology::full();
    let validator = "ontology/inventory/proof_dependency_acyclicity";

    let depends_on_prop = "https://uor.foundation/proof/dependsOn";
    let proves_prop = "https://uor.foundation/proof/provesIdentity";

    // Build proof ID -> identity ID mapping
    let mut proof_to_identity: HashMap<&str, &str> = HashMap::new();
    // Build identity ID -> set of dependency identity IDs
    let mut identity_deps: HashMap<&str, Vec<&str>> = HashMap::new();

    if let Some(proof_module) = ontology.find_namespace("proof") {
        for ind in &proof_module.individuals {
            let mut identity_id = None;
            let mut deps = Vec::new();

            for (prop, val) in ind.properties {
                if *prop == proves_prop {
                    if let IndividualValue::IriRef(iri) = val {
                        identity_id = Some(*iri);
                    }
                }
                if *prop == depends_on_prop {
                    if let IndividualValue::IriRef(iri) = val {
                        deps.push(*iri);
                    }
                }
            }

            if let Some(id) = identity_id {
                proof_to_identity.insert(ind.id, id);
                if !deps.is_empty() {
                    identity_deps.insert(id, deps);
                }
            }
        }
    }

    // DFS cycle detection on identity dependency graph
    let mut visited: HashSet<&str> = HashSet::new();
    let mut in_stack: HashSet<&str> = HashSet::new();
    let mut has_cycle = false;

    fn dfs<'a>(
        node: &'a str,
        deps: &HashMap<&'a str, Vec<&'a str>>,
        visited: &mut HashSet<&'a str>,
        in_stack: &mut HashSet<&'a str>,
    ) -> bool {
        if in_stack.contains(node) {
            return true; // cycle
        }
        if visited.contains(node) {
            return false;
        }
        visited.insert(node);
        in_stack.insert(node);

        if let Some(neighbors) = deps.get(node) {
            for &neighbor in neighbors {
                if dfs(neighbor, deps, visited, in_stack) {
                    return true;
                }
            }
        }
        in_stack.remove(node);
        false
    }

    for &identity in identity_deps.keys() {
        if dfs(identity, &identity_deps, &mut visited, &mut in_stack) {
            has_cycle = true;
            break;
        }
    }

    if has_cycle {
        report.push(TestResult::fail(
            validator,
            "Cycle detected in proof:dependsOn dependency graph",
        ));
    } else {
        report.push(TestResult::pass(
            validator,
            format!(
                "Proof dependency DAG is acyclic ({} identities with dependencies)",
                identity_deps.len()
            ),
        ));
    }
}

/// Amendment 89: Validates that op:lhs, op:rhs, and op:forAll are ObjectProperties
/// with the correct AST ranges.
fn validate_identity_properties_typed(report: &mut ConformanceReport) {
    let ontology = uor_ontology::Ontology::full();

    let checks = [
        (
            "https://uor.foundation/op/lhs",
            PropertyKind::Object,
            "schema/TermExpression",
            "ontology/inventory/identity_lhs_typed",
        ),
        (
            "https://uor.foundation/op/rhs",
            PropertyKind::Object,
            "schema/TermExpression",
            "ontology/inventory/identity_rhs_typed",
        ),
        (
            "https://uor.foundation/op/forAll",
            PropertyKind::Object,
            "schema/ForAllDeclaration",
            "ontology/inventory/identity_forall_typed",
        ),
    ];

    for (prop_iri, expected_kind, expected_range_suffix, validator) in checks {
        let found = ontology
            .namespaces
            .iter()
            .flat_map(|m| m.properties.iter())
            .find(|p| p.id == prop_iri);

        match found {
            Some(prop) => {
                let kind_ok = prop.kind == expected_kind;
                let range_ok = prop.range.contains(expected_range_suffix);

                if kind_ok && range_ok {
                    report.push(TestResult::pass(
                        validator,
                        format!(
                            "{} is {:?} with range containing {}",
                            prop_iri, expected_kind, expected_range_suffix
                        ),
                    ));
                } else {
                    if !kind_ok {
                        report.push(TestResult::fail(
                            validator,
                            format!(
                                "{} has kind {:?}, expected {:?}",
                                prop_iri, prop.kind, expected_kind
                            ),
                        ));
                    }
                    if !range_ok {
                        report.push(TestResult::fail(
                            validator,
                            format!(
                                "{} has range {}, expected to contain {}",
                                prop_iri, prop.range, expected_range_suffix
                            ),
                        ));
                    }
                }
            }
            None => {
                report.push(TestResult::fail(
                    validator,
                    format!("Property {} not found in ontology", prop_iri),
                ));
            }
        }
    }
}

/// Amendment 90: Validates that removed properties no longer exist.
fn validate_removed_properties_absent(report: &mut ConformanceReport) {
    let ontology = uor_ontology::Ontology::full();
    let validator = "ontology/inventory/removed_properties_absent";

    let removed_iris = [
        "https://uor.foundation/op/verificationPathNote",
        "https://uor.foundation/op/geometricCharacterNote",
        "https://uor.foundation/op/compositionLawRef",
        "https://uor.foundation/type/constraint",
        "https://uor.foundation/type/axisSignatureNote",
        "https://uor.foundation/type/decompositionRule",
    ];

    let all_prop_ids: HashSet<&str> = ontology
        .namespaces
        .iter()
        .flat_map(|m| m.properties.iter())
        .map(|p| p.id)
        .collect();

    let mut found = Vec::new();
    for iri in &removed_iris {
        if all_prop_ids.contains(*iri) {
            found.push(iri.to_string());
        }
    }

    if found.is_empty() {
        report.push(TestResult::pass(
            validator,
            format!("All {} removed properties are absent", removed_iris.len()),
        ));
    } else {
        for iri in &found {
            report.push(TestResult::fail(
                validator,
                format!("Property {} should have been removed but still exists", iri),
            ));
        }
    }
}

/// Amendment 90: Validates that only legitimate string properties remain.
fn validate_legitimate_string_properties_only(report: &mut ConformanceReport) {
    let ontology = uor_ontology::Ontology::full();
    let validator = "ontology/inventory/legitimate_string_properties_only";

    let xsd_string = "http://www.w3.org/2001/XMLSchema#string";

    let legitimate: HashSet<&str> = [
        "https://uor.foundation/u/glyph",
        "https://uor.foundation/u/digest",
        "https://uor.foundation/u/digestAlgorithm",
        "https://uor.foundation/state/contentAddress",
        "https://uor.foundation/reduction/stageName",
        "https://uor.foundation/reduction/pressureLevel",
        "https://uor.foundation/convergence/levelName",
        "https://uor.foundation/convergence/bettiSignature",
        "https://uor.foundation/state/boundaryReason",
        "https://uor.foundation/trace/violationReason",
        "https://uor.foundation/op/operatorComplexity",
        "https://uor.foundation/op/operatorSignature",
        "https://uor.foundation/op/presentation",
        "https://uor.foundation/proof/wittNote",
        "https://uor.foundation/proof/criticalIdentity",
        "https://uor.foundation/schema/variableName",
        "https://uor.foundation/schema/literalValue",
        "https://uor.foundation/schema/infixOperator",
        "https://uor.foundation/type/structuralSiteCount",
        "https://uor.foundation/type/structuralGrounding",
        "https://uor.foundation/type/structuralConstraint",
        "https://uor.foundation/observable/metricDomain",
        "https://uor.foundation/observable/metricRange",
        "https://uor.foundation/division/basisElements",
        "https://uor.foundation/division/adjoinedElement",
        "https://uor.foundation/division/conjugationRule",
        "https://uor.foundation/proof/forbidsSignature",
        "https://uor.foundation/proof/impossibilityReason",
        "https://uor.foundation/convergence/totalSpace",
        "https://uor.foundation/convergence/baseSpace",
        "https://uor.foundation/convergence/fiberSphere",
        "https://uor.foundation/convergence/characteristicIdentity",
        "https://uor.foundation/reduction/failureKind",
        "https://uor.foundation/reduction/preflightKind",
        "https://uor.foundation/reduction/transactionPolicy",
        "https://uor.foundation/reduction/leasePhase",
        "https://uor.foundation/reduction/feasibilityKind",
        "https://uor.foundation/reduction/feasibilityWitness",
        "https://uor.foundation/reduction/bindTarget",
        "https://uor.foundation/reduction/bindValue",
        "https://uor.foundation/reduction/predicateField",
        "https://uor.foundation/reduction/predicateOperator",
        "https://uor.foundation/reduction/predicateValue",
        // Amendment 95: Declarative enforcement
        "https://uor.foundation/conformance/shapeIri",
        "https://uor.foundation/conformance/constraintIri",
        "https://uor.foundation/conformance/propertyIri",
        "https://uor.foundation/conformance/expectedRange",
        "https://uor.foundation/conformance/effectName",
        "https://uor.foundation/conformance/ringMapping",
        "https://uor.foundation/conformance/leaseScope",
        "https://uor.foundation/conformance/productivityWitness",
        "https://uor.foundation/conformance/terminationWitness",
        "https://uor.foundation/conformance/disjointnessWitness",
        // Amendment 95: Host-value sort (Workstream 5)
        "https://uor.foundation/schema/hostString",
        // Amendment 96 (2026-04-12): symbolic phase-angle descriptors
        // on reduction/ individuals carry Greek symbol strings
        // ("Ω⁰", "Ω¹", "π/6", "e^{iπ/6}") rather than numeric decimals.
        // Range was xsd:decimal; changed to xsd:string to match the
        // value shape. See reduction.rs.
        "https://uor.foundation/reduction/phaseParameter",
        "https://uor.foundation/reduction/convergenceAngle",
        "https://uor.foundation/reduction/rotationSchedule",
        "https://uor.foundation/reduction/baseAngle",
        "https://uor.foundation/reduction/targetAngle",
        "https://uor.foundation/reduction/gateExpectedPhase",
        "https://uor.foundation/reduction/expectedPhase",
        "https://uor.foundation/reduction/phaseAngle",
        "https://uor.foundation/reduction/finalGrounding",
        "https://uor.foundation/reduction/thermodynamicBudget",
        // Amendment 96: observable/referencesClass is an
        // annotation-valued IRI reference (the metric describes a
        // class of phenomena, not a specific instance).
        "https://uor.foundation/observable/referencesClass",
        // v0.2.1: Inhabitance Verdict Instantiation
        "https://uor.foundation/proof/contradictionProof",
        // v0.2.1: Conformance backing properties
        "https://uor.foundation/predicate/terminationWitness",
        "https://uor.foundation/parallel/disjointnessWitness",
        "https://uor.foundation/stream/productivityWitness",
        "https://uor.foundation/state/leaseScope",
        // v0.2.1: Surface-grammar metadata for parametric ebnf emission
        "https://uor.foundation/conformance/surfaceForm",
        "https://uor.foundation/conformance/surfaceKeyword",
        "https://uor.foundation/conformance/surfaceProduction",
        "https://uor.foundation/conformance/exportRustName",
        // v0.2.1: PipelineFailure variant-field metadata
        "https://uor.foundation/reduction/fieldName",
        "https://uor.foundation/reduction/fieldType",
        // v0.2.2 Phase D (Q4): BoundConstraint boundArguments carries
        // the kind-specific parameters in canonical string form
        // ("modulus=256;residue=0", "bound=0", etc.). String is the
        // canonical serialization; typed accessors on the Rust side
        // unpack via per-type-alias constructors.
        "https://uor.foundation/type/boundArguments",
        // Product/Coproduct Completion Amendment: productCategoryLevel is
        // a closed-vocabulary discriminator ("partition_classification" or
        // "nerve_topology") at the Partition class — using xsd:string is
        // the canonical encoding since the value is a fixed identifier the
        // reader interprets, not a parsed numeric or structured type.
        "https://uor.foundation/partition/productCategoryLevel",
        // Product/Coproduct Completion Amendment: layoutRule on
        // foundation:LayoutInvariant carries the human-readable arithmetic
        // identity ("SITE_COUNT(A × B) = SITE_COUNT(A) + SITE_COUNT(B)"
        // etc.) for documentation and debugger inspection. Not parsed;
        // xsd:string is the canonical range.
        "https://uor.foundation/foundation/layoutRule",
    ]
    .into_iter()
    .collect();

    let mut violations = Vec::new();
    for ns in &ontology.namespaces {
        for prop in &ns.properties {
            if prop.range == xsd_string && !legitimate.contains(prop.id) {
                violations.push(format!("{} still has range xsd:string", prop.id));
            }
        }
    }

    if violations.is_empty() {
        report.push(TestResult::pass(
            validator,
            format!(
                "Only {} legitimate string properties remain",
                legitimate.len()
            ),
        ));
    } else {
        for v in &violations {
            report.push(TestResult::fail(validator, v.clone()));
        }
    }
}

/// Amendment 91: Validates that every op:Identity individual's lhs, rhs, and
/// forAll property assertions use `IndividualValue::IriRef`, not `Str`.
fn validate_identity_values_are_irirefs(report: &mut ConformanceReport) {
    let ontology = uor_ontology::Ontology::full();
    let validator = "ontology/inventory/identity_values_are_irirefs";

    let identity_type = "https://uor.foundation/op/Identity";
    let lhs_prop = "https://uor.foundation/op/lhs";
    let rhs_prop = "https://uor.foundation/op/rhs";
    let forall_prop = "https://uor.foundation/op/forAll";

    let mut violations = Vec::new();

    for ns in &ontology.namespaces {
        for ind in &ns.individuals {
            if ind.type_ != identity_type {
                continue;
            }
            for (prop, val) in ind.properties {
                if (*prop == lhs_prop || *prop == rhs_prop || *prop == forall_prop)
                    && matches!(val, IndividualValue::Str(_))
                {
                    violations.push(format!(
                        "{}: {} still uses IndividualValue::Str",
                        ind.label,
                        prop.rsplit('/').next().unwrap_or(prop)
                    ));
                }
            }
        }
    }

    if violations.is_empty() {
        report.push(TestResult::pass(
            validator,
            "All identity lhs/rhs/forAll values are IriRef (not Str)",
        ));
    } else {
        report.push(TestResult::fail(
            validator,
            format!(
                "{} identity property assertions still use Str instead of IriRef",
                violations.len()
            ),
        ));
    }
}

/// Amendment 92: Validates that `proof:formalDerivation` is an ObjectProperty
/// with range `proof/DerivationTerm`.
fn validate_formal_derivation_typed(report: &mut ConformanceReport) {
    let ontology = uor_ontology::Ontology::full();
    let validator = "ontology/inventory/formal_derivation_typed";

    let prop_iri = "https://uor.foundation/proof/formalDerivation";
    let found = ontology
        .namespaces
        .iter()
        .flat_map(|m| m.properties.iter())
        .find(|p| p.id == prop_iri);

    match found {
        Some(prop) => {
            let kind_ok = prop.kind == PropertyKind::Object;
            let range_ok = prop.range.contains("DerivationTerm");

            if kind_ok && range_ok {
                report.push(TestResult::pass(
                    validator,
                    "proof:formalDerivation is ObjectProperty with range DerivationTerm",
                ));
            } else {
                if !kind_ok {
                    report.push(TestResult::fail(
                        validator,
                        format!("formalDerivation has kind {:?}, expected Object", prop.kind),
                    ));
                }
                if !range_ok {
                    report.push(TestResult::fail(
                        validator,
                        format!(
                            "formalDerivation has range {}, expected DerivationTerm",
                            prop.range
                        ),
                    ));
                }
            }
        }
        None => {
            report.push(TestResult::fail(
                validator,
                "Property proof:formalDerivation not found",
            ));
        }
    }
}

// ── Amendment 95 validators ────────────────────────────────────────────────

/// Validates the 12 predicate registry individuals (Workstream 1).
fn validate_predicate_registry(report: &mut ConformanceReport) {
    let ontology = Ontology::full();
    let validator = "ontology/inventory/predicate_registry";
    let ns = ontology.find_namespace("predicate");
    match ns {
        Some(m) => {
            let expected = [
                "always",
                "never",
                "isZero",
                "isUnit",
                "isOdd",
                "isEven",
                "isInvolution",
                "sitePinned",
                "siteFree",
                "contradictionReached",
                "budgetExhausted",
                "reductionConverged",
            ];
            let mut all_found = true;
            for label in &expected {
                let iri = format!("https://uor.foundation/predicate/{label}");
                if !m.individuals.iter().any(|i| i.id == iri) {
                    all_found = false;
                }
            }
            if all_found {
                report.push(TestResult::pass(
                    validator,
                    format!(
                        "All {} predicate registry individuals present",
                        expected.len()
                    ),
                ));
            } else {
                report.push(TestResult::fail(
                    validator,
                    "Missing one or more predicate registry individuals",
                ));
            }
        }
        None => {
            report.push(TestResult::fail(validator, "predicate namespace not found"));
        }
    }
}

/// Validates the 3 new Constraint subclasses and 5 properties (Workstream 3).
fn validate_constraint_completion(report: &mut ConformanceReport) {
    let ontology = Ontology::full();
    let validator = "ontology/inventory/constraint_completion";
    let ns = ontology.find_namespace("type");
    match ns {
        Some(m) => {
            // v0.2.2 Phase D: the 7 enumerated Constraint subclasses from
            // Amendment 95 (Workstream 3) were folded into the parametric
            // BoundConstraint<O, B> + Conjunction surface. The check moved
            // from "the 3 Workstream 3 subclasses exist" to "the parametric
            // classes, their 4 backing properties, and the 11 former
            // subclass properties (re-parented to BoundConstraint /
            // Conjunction) are all present".
            let parametric_classes = [
                "https://uor.foundation/type/BoundConstraint",
                "https://uor.foundation/type/BoundShape",
                "https://uor.foundation/type/Conjunction",
            ];
            let classes_ok = parametric_classes
                .iter()
                .all(|iri| m.classes.iter().any(|c| c.id == *iri));
            let new_props = [
                "https://uor.foundation/type/hammingBound",
                "https://uor.foundation/type/siteIndex",
                "https://uor.foundation/type/siteValue",
                "https://uor.foundation/type/affineOffset",
                "https://uor.foundation/type/affineGenerator",
                "https://uor.foundation/type/boundObservable",
                "https://uor.foundation/type/boundShape",
                "https://uor.foundation/type/boundArguments",
                "https://uor.foundation/type/conjuncts",
            ];
            let props_ok = new_props
                .iter()
                .all(|iri| m.properties.iter().any(|p| p.id == *iri));
            let carry_ok = m.properties.iter().any(|p| {
                p.id == "https://uor.foundation/type/carryPattern"
                    && p.range == "https://uor.foundation/schema/Datum"
            });
            if classes_ok && props_ok && carry_ok {
                report.push(TestResult::pass(
                    validator,
                    "Constraint completion (parametric): BoundConstraint + \
                     BoundShape + Conjunction classes, 4 parametric + 5 \
                     kind-specific properties, carryPattern retyped",
                ));
            } else {
                report.push(TestResult::fail(
                    validator,
                    format!(
                        "Constraint completion incomplete: classes={classes_ok}, \
                         props={props_ok}, carry_retyped={carry_ok}"
                    ),
                ));
            }
        }
        None => {
            report.push(TestResult::fail(validator, "type namespace not found"));
        }
    }
}

/// Validates the host-value sort classes and properties (Workstream 5).
fn validate_host_value_sort(report: &mut ConformanceReport) {
    let ontology = Ontology::full();
    let validator = "ontology/inventory/host_value_sort";
    let ns = ontology.find_namespace("schema");
    match ns {
        Some(m) => {
            let new_classes = [
                "https://uor.foundation/schema/SurfaceSymbol",
                "https://uor.foundation/schema/HostValue",
                "https://uor.foundation/schema/HostStringLiteral",
                "https://uor.foundation/schema/HostBooleanLiteral",
            ];
            let classes_ok = new_classes
                .iter()
                .all(|iri| m.classes.iter().any(|c| c.id == *iri));
            let literal_ok = m.classes.iter().any(|c| {
                c.id == "https://uor.foundation/schema/Literal"
                    && c.subclass_of
                        .contains(&"https://uor.foundation/schema/SurfaceSymbol")
            });
            if classes_ok && literal_ok {
                report.push(TestResult::pass(
                    validator,
                    "Host-value sort: 4 classes present, Literal has SurfaceSymbol parent",
                ));
            } else {
                report.push(TestResult::fail(
                    validator,
                    format!(
                        "Host-value sort incomplete: classes={classes_ok}, \
                         literal_parent={literal_ok}"
                    ),
                ));
            }
        }
        None => {
            report.push(TestResult::fail(validator, "schema namespace not found"));
        }
    }
}

/// Validates the boundary map registry (Workstream 4).
fn validate_boundary_map_registry(report: &mut ConformanceReport) {
    let ontology = Ontology::full();
    let validator = "ontology/inventory/boundary_map_registry";
    let ns = ontology.find_namespace("morphism");
    match ns {
        Some(m) => {
            let new_classes = [
                "https://uor.foundation/morphism/Witness",
                "https://uor.foundation/morphism/GroundingWitness",
                "https://uor.foundation/morphism/ProjectionWitness",
                "https://uor.foundation/morphism/SymbolSequence",
                "https://uor.foundation/morphism/SequenceElement",
            ];
            let classes_ok = new_classes
                .iter()
                .all(|iri| m.classes.iter().any(|c| c.id == *iri));
            let map_individuals = [
                "https://uor.foundation/morphism/IntegerGroundingMap",
                "https://uor.foundation/morphism/Utf8GroundingMap",
                "https://uor.foundation/morphism/JsonGroundingMap",
                "https://uor.foundation/morphism/IntegerProjectionMap",
                "https://uor.foundation/morphism/Utf8ProjectionMap",
                "https://uor.foundation/morphism/JsonProjectionMap",
            ];
            let individuals_ok = map_individuals
                .iter()
                .all(|iri| m.individuals.iter().any(|i| i.id == *iri));
            let redomained_ok = m.properties.iter().any(|p| {
                p.id == "https://uor.foundation/morphism/surfaceSymbol"
                    && p.domain == Some("https://uor.foundation/morphism/GroundingWitness")
                    && p.range == "https://uor.foundation/schema/SurfaceSymbol"
            });
            if classes_ok && individuals_ok && redomained_ok {
                report.push(TestResult::pass(
                    validator,
                    "Boundary map registry: 5 classes, 6 individuals, re-domainings correct",
                ));
            } else {
                report.push(TestResult::fail(
                    validator,
                    format!(
                        "Boundary map registry incomplete: classes={classes_ok}, \
                         individuals={individuals_ok}, redomained={redomained_ok}"
                    ),
                ));
            }
        }
        None => {
            report.push(TestResult::fail(validator, "morphism namespace not found"));
        }
    }
}

/// Validates EitherType and OptionType as SumType individuals (Workstream 8).
fn validate_type_definition_coverage(report: &mut ConformanceReport) {
    let ontology = Ontology::full();
    let validator = "ontology/inventory/type_definition_coverage";
    let ns = ontology.find_namespace("type");
    match ns {
        Some(m) => {
            let either_ok = m.individuals.iter().any(|i| {
                i.id == "https://uor.foundation/type/EitherType"
                    && i.type_ == "https://uor.foundation/type/SumType"
            });
            let option_ok = m.individuals.iter().any(|i| {
                i.id == "https://uor.foundation/type/OptionType"
                    && i.type_ == "https://uor.foundation/type/SumType"
            });
            if either_ok && option_ok {
                report.push(TestResult::pass(
                    validator,
                    "EitherType and OptionType present as SumType individuals",
                ));
            } else {
                report.push(TestResult::fail(
                    validator,
                    format!(
                        "TypeDefinition coverage incomplete: either={either_ok}, \
                         option={option_ok}"
                    ),
                ));
            }
        }
        None => {
            report.push(TestResult::fail(validator, "type namespace not found"));
        }
    }
}

/// Validates composedGrounding range retyping (Workstream 6).
fn validate_operad_retyping(report: &mut ConformanceReport) {
    let ontology = Ontology::full();
    let validator = "ontology/inventory/operad_retyping";
    let ns = ontology.find_namespace("operad");
    match ns {
        Some(m) => {
            let grounding_ok = m.properties.iter().any(|p| {
                p.id == "https://uor.foundation/operad/composedGrounding"
                    && p.range == "https://uor.foundation/morphism/GroundingMap"
            });
            if grounding_ok {
                report.push(TestResult::pass(
                    validator,
                    "composedGrounding range is morphism:GroundingMap",
                ));
            } else {
                report.push(TestResult::fail(
                    validator,
                    "composedGrounding range is not morphism:GroundingMap",
                ));
            }
        }
        None => {
            report.push(TestResult::fail(validator, "operad namespace not found"));
        }
    }
}
