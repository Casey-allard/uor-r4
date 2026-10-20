//! Test 280: v0.2.2 Phase E + T1.2 — bridge namespace completion SHACL coverage.
//!
//! Validates SHACL coverage for the Phase E ontology additions plus the T1.2
//! cleanup:
//! - cert:PartitionCertificate
//! - partition:PartitionComponent (enum class)
//! - observable:GroundingSigma
//! - observable:JacobianObservable
//! - derivation:DerivationTrace with traceEventCount
//! - conformance:InteractionShape (T1.2 cleanup)

/// Instance graph for Test 280: Phase E bridge namespace completion + T1.2.
pub const TEST280_BRIDGE_COMPLETION: &str = r#"
@prefix rdf:         <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:         <http://www.w3.org/2002/07/owl#> .
@prefix xsd:         <http://www.w3.org/2001/XMLSchema#> .
@prefix cert:        <https://uor.foundation/cert/> .
@prefix partition:   <https://uor.foundation/partition/> .
@prefix observable:  <https://uor.foundation/observable/> .
@prefix derivation:  <https://uor.foundation/derivation/> .
@prefix conformance: <https://uor.foundation/conformance/> .

# 1. cert:PartitionCertificate — attests a partition component classification.
<https://uor.foundation/instance/cert/partition_cert_example>
    a owl:NamedIndividual, cert:PartitionCertificate .

# 2. partition:PartitionComponent — enum class (the four individuals live
# in the ontology, but a fresh instance here demonstrates fixture coverage).
<https://uor.foundation/instance/partition/partition_component_example>
    a owl:NamedIndividual, partition:PartitionComponent .

# 3. observable:GroundingSigma — the grounding completion ratio.
<https://uor.foundation/instance/observable/grounding_sigma_example>
    a owl:NamedIndividual, observable:GroundingSigma .

# 4. observable:JacobianObservable — per-site Jacobian row.
<https://uor.foundation/instance/observable/jacobian_observable_example>
    a owl:NamedIndividual, observable:JacobianObservable .

# 5. derivation:DerivationTrace — ordered trace of rewrite steps.
<https://uor.foundation/instance/derivation/trace_example>
    a owl:NamedIndividual, derivation:DerivationTrace ;
    derivation:traceEventCount "42"^^xsd:nonNegativeInteger .

# 6. conformance:InteractionShape — the shape class backing the foundation's
# InteractionDeclarationBuilder (T1.2 cleanup).
<https://uor.foundation/instance/conformance/interaction_shape_example>
    a owl:NamedIndividual, conformance:InteractionShape .
"#;
