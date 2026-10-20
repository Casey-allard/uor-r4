//! Test 277: v0.2.1 Inhabitance Verdict Instantiation.
//!
//! Validates SHACL coverage for the v0.2.1 ontology additions:
//! - cert:InhabitanceCertificate, proof:InhabitanceImpossibilityWitness
//! - trace:InhabitanceSearchTrace, derivation:InhabitanceStep / InhabitanceCheckpoint
//! - resolver:InhabitanceResolver and the three target deciders
//! - schema:ValueTuple
//! - reduction:FailureField (parametric PipelineFailure metadata)
//! - resolver:CertifyMapping (parametric Certify metadata)
//! - conformance:PreludeExport (parametric prelude membership)

/// Instance graph for Test 277: Inhabitance verdict.
pub const TEST277_INHABITANCE_VERDICT: &str = r#"
@prefix rdf:        <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:        <http://www.w3.org/2002/07/owl#> .
@prefix xsd:        <http://www.w3.org/2001/XMLSchema#> .
@prefix cert:       <https://uor.foundation/cert/> .
@prefix proof:      <https://uor.foundation/proof/> .
@prefix trace:      <https://uor.foundation/trace/> .
@prefix derivation: <https://uor.foundation/derivation/> .
@prefix resolver:   <https://uor.foundation/resolver/> .
@prefix schema:     <https://uor.foundation/schema/> .
@prefix predicate:  <https://uor.foundation/predicate/> .
@prefix reduction:  <https://uor.foundation/reduction/> .
@prefix conformance: <https://uor.foundation/conformance/> .

# 1. cert:InhabitanceCertificate (verified-true case)
<https://uor.foundation/instance/inhab/cert>
    a               owl:NamedIndividual, cert:InhabitanceCertificate ;
    cert:verified   "true"^^xsd:boolean .

# 2. schema:ValueTuple (the witness carrier)
<https://uor.foundation/instance/inhab/witness_tuple>
    a               owl:NamedIndividual, schema:ValueTuple .

# 3. proof:InhabitanceImpossibilityWitness
<https://uor.foundation/instance/inhab/witness>
    a               owl:NamedIndividual, proof:InhabitanceImpossibilityWitness ;
    proof:contradictionProof "by-contradiction over IH_1" .

# 4. trace:InhabitanceSearchTrace
<https://uor.foundation/instance/inhab/trace>
    a               owl:NamedIndividual, trace:InhabitanceSearchTrace .

# 5. derivation:InhabitanceStep
<https://uor.foundation/instance/inhab/step>
    a               owl:NamedIndividual, derivation:InhabitanceStep .

# 6. derivation:InhabitanceCheckpoint
<https://uor.foundation/instance/inhab/checkpoint>
    a               owl:NamedIndividual, derivation:InhabitanceCheckpoint ;
    derivation:checkpointIndex "0"^^xsd:integer .

# 7. resolver:InhabitanceResolver and the three target deciders
<https://uor.foundation/instance/inhab/resolver>
    a               owl:NamedIndividual, resolver:InhabitanceResolver .
<https://uor.foundation/instance/inhab/two_sat>
    a               owl:NamedIndividual, resolver:TwoSatDecider .
<https://uor.foundation/instance/inhab/horn_sat>
    a               owl:NamedIndividual, resolver:HornSatDecider .
<https://uor.foundation/instance/inhab/residual>
    a               owl:NamedIndividual, resolver:ResidualVerdictResolver .

# 8. reduction:FailureField (parametric PipelineFailure metadata)
<https://uor.foundation/instance/inhab/failure_field>
    a               owl:NamedIndividual, reduction:FailureField ;
    reduction:fieldName  "stage_iri" ;
    reduction:fieldType  "&'static str" .

# 9. resolver:CertifyMapping (parametric Certify metadata)
<https://uor.foundation/instance/inhab/certify_mapping>
    a               owl:NamedIndividual, resolver:CertifyMapping .

# 10. conformance:PreludeExport (parametric prelude membership)
<https://uor.foundation/instance/inhab/prelude_export>
    a               owl:NamedIndividual, conformance:PreludeExport .
"#;
