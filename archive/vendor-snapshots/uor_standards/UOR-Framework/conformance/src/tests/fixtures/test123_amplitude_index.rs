/// SHACL test 123: SuperposedSiteState amplitude index set —
/// demonstrates QM_6 identity grounding (Amendment 44).
pub const TEST123_AMPLITUDE_INDEX: &str = r#"
@prefix rdf:  <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .
@prefix owl:  <http://www.w3.org/2002/07/owl#> .
@prefix xsd:  <http://www.w3.org/2001/XMLSchema#> .
@prefix op:   <https://uor.foundation/op/> .
@prefix proof: <https://uor.foundation/proof/> .

op:QM_6 a owl:NamedIndividual, op:Identity ;
    rdfs:label "QM_6" ;
    op:lhs "amplitude index set of SuperposedSiteState over T" ;
    op:rhs "monotone pinning trajectories consistent with T" ;
    op:forAll "SuperposedSiteState over ConstrainedType T at Q_n" ;
    op:verificationDomain op:SuperpositionDomain ;
    op:universallyValid "true"^^xsd:boolean .

proof:prf_QM_6 a owl:NamedIndividual, proof:EmpiricalVerification ;
    proof:provesIdentity op:QM_6 ;
    proof:verified "true"^^xsd:boolean ;
    proof:quantumLevelRange "Q0-Q3" ;
    proof:verificationMethod "exhaustive trajectory enumeration over site lattice" .
"#;
