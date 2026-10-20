/// SHACL test 28: Measurement unit vocabulary — typed observable units.
pub const TEST28_MEASUREMENT_UNIT: &str = r#"
@prefix rdf:  <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:  <http://www.w3.org/2002/07/owl#> .
@prefix xsd:  <http://www.w3.org/2001/XMLSchema#> .
@prefix observable: <https://uor.foundation/observable/> .

# MeasurementUnit vocabulary individuals
observable:Bits a observable:MeasurementUnit .
observable:RingSteps a observable:MeasurementUnit .
observable:Dimensionless a observable:MeasurementUnit .

# Observable instances with typed units
<https://uor.foundation/instance/hamming-obs>
    a observable:HammingMetric ;
    observable:value "3"^^xsd:decimal ;
    observable:hasUnit observable:Bits .

<https://uor.foundation/instance/ring-obs>
    a observable:RingMetric ;
    observable:value "42"^^xsd:decimal ;
    observable:hasUnit observable:RingSteps .

<https://uor.foundation/instance/betti-obs>
    a observable:BettiNumber ;
    observable:value "1"^^xsd:decimal ;
    observable:hasUnit observable:Dimensionless .
"#;
