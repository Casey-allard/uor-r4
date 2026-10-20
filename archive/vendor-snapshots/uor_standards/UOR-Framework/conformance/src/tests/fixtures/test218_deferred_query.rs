//! SHACL test 218: `reduction:DeferredQuerySet` instance.

/// Instance graph for Test 218: DeferredQuerySet with count and epoch.
pub const TEST218_DEFERRED_QUERY: &str = r#"
@prefix rdf:     <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:     <http://www.w3.org/2002/07/owl#> .
@prefix xsd:     <http://www.w3.org/2001/XMLSchema#> .
@prefix reduction: <https://uor.foundation/reduction/> .

reduction:dq_example a owl:NamedIndividual, reduction:DeferredQuerySet ;
    reduction:deferredCount "5"^^xsd:nonNegativeInteger ;
    reduction:deferralEpoch "3"^^xsd:nonNegativeInteger .
"#;
