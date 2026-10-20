/// SHACL fixture for trace:TraceMetrics.
pub const TEST156_TRACE_METRICS: &str = r#"
@prefix rdf:   <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:   <http://www.w3.org/2002/07/owl#> .
@prefix xsd:   <http://www.w3.org/2001/XMLSchema#> .
@prefix trace: <https://uor.foundation/trace/> .

<urn:test:trace_metrics_1> a owl:NamedIndividual , trace:TraceMetrics ;
    trace:stepCount "10"^^xsd:nonNegativeInteger ;
    trace:totalRingDistance "42"^^xsd:nonNegativeInteger ;
    trace:totalHammingDistance "15"^^xsd:nonNegativeInteger .
"#;
