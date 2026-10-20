//! SHACL test 171: `homology:PostnikovTruncation` with k-invariant.

/// Instance graph for Test 171: PostnikovTruncation.
pub const TEST171_POSTNIKOV_TRUNCATION: &str = r#"
@prefix rdf:      <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:      <http://www.w3.org/2002/07/owl#> .
@prefix xsd:      <http://www.w3.org/2001/XMLSchema#> .
@prefix homology: <https://uor.foundation/homology/> .

homology:ex_trunc_171 a owl:NamedIndividual, homology:PostnikovTruncation ;
    homology:truncationLevel  "2"^^xsd:nonNegativeInteger ;
    homology:truncationSource homology:ex_kan_171 ;
    homology:kInvariant       homology:ex_kinv_171 .

homology:ex_kan_171 a owl:NamedIndividual, homology:KanComplex .

homology:ex_kinv_171 a owl:NamedIndividual, homology:KInvariant ;
    homology:kInvariantTrivial "true"^^xsd:boolean .
"#;
