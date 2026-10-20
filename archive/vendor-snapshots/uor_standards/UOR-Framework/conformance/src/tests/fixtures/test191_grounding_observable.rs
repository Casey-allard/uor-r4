//! SHACL test 191: `observable:GroundingObservable`.

/// Instance graph for Test 191: GroundingObservable with numerator and denominator.
pub const TEST191_GROUNDING_OBSERVABLE: &str = r#"
@prefix rdf:        <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:        <http://www.w3.org/2002/07/owl#> .
@prefix xsd:        <http://www.w3.org/2001/XMLSchema#> .
@prefix observable: <https://uor.foundation/observable/> .

observable:ex_sat_191 a owl:NamedIndividual, observable:GroundingObservable ;
    observable:groundingNumerator "5"^^xsd:nonNegativeInteger ;
    observable:groundingDenominator "8"^^xsd:positiveInteger .
"#;
