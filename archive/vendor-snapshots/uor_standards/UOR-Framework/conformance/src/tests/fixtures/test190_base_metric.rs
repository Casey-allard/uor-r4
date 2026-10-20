//! SHACL test 190: `observable:BaseMetric`.

/// Instance graph for Test 190: BaseMetric with domain and range.
pub const TEST190_BASE_METRIC: &str = r#"
@prefix rdf:        <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:        <http://www.w3.org/2002/07/owl#> .
@prefix xsd:        <http://www.w3.org/2001/XMLSchema#> .
@prefix observable: <https://uor.foundation/observable/> .

observable:ex_bm_190 a owl:NamedIndividual, observable:BaseMetric ;
    observable:metricDomain "pair of ring elements" ;
    observable:metricRange "non-negative integer" .
"#;
