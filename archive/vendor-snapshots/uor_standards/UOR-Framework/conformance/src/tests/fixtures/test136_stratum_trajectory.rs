/// SHACL fixture for observable:StratumTrajectory.
pub const TEST136_STRATUM_TRAJECTORY: &str = r#"
@prefix rdf:        <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:        <http://www.w3.org/2002/07/owl#> .
@prefix xsd:        <http://www.w3.org/2001/XMLSchema#> .
@prefix observable: <https://uor.foundation/observable/> .

<urn:test:stratum_traj_1> a owl:NamedIndividual , observable:StratumTrajectory ;
    observable:value "4"^^xsd:decimal .
"#;
