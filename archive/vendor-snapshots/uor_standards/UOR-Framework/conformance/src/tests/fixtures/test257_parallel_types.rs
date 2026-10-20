//! SHACL test 257: `parallel` namespace types.

/// Instance graph for Test 257: Parallel product, certificate, and trace types.
pub const TEST257_PARALLEL_TYPES: &str = r#"
@prefix rdf:      <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:      <http://www.w3.org/2002/07/owl#> .
@prefix parallel: <https://uor.foundation/parallel/> .

parallel:ex_product_257 a owl:NamedIndividual, parallel:ParallelProduct .
parallel:ex_disjointness_cert_257 a owl:NamedIndividual, parallel:DisjointnessCertificate .
parallel:ex_sync_point_257 a owl:NamedIndividual, parallel:SynchronizationPoint .
parallel:ex_trace_257 a owl:NamedIndividual, parallel:ParallelTrace .
parallel:ex_partitioning_257 a owl:NamedIndividual, parallel:SitePartitioning .
"#;
