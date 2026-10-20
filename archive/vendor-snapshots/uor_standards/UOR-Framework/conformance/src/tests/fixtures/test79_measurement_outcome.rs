/// SHACL test 79: MeasurementOutcome with outcomeValue and outcomeProbability (Amendment 37, Gap 10).
pub const TEST79_MEASUREMENT_OUTCOME: &str = r#"
@prefix rdf:   <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:   <http://www.w3.org/2002/07/owl#> .
@prefix xsd:   <http://www.w3.org/2001/XMLSchema#> .
@prefix trace: <https://uor.foundation/trace/> .

trace:ex_mo_79 a owl:NamedIndividual, trace:MeasurementOutcome ;
    trace:outcomeValue "0"^^xsd:nonNegativeInteger ;
    trace:outcomeProbability "0.5"^^xsd:decimal .
"#;
