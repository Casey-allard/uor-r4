//! SHACL test 180: `resolver:ModuliResolver`.

/// Instance graph for Test 180: ModuliResolver.
pub const TEST180_MODULI_RESOLVER: &str = r#"
@prefix rdf:      <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:      <http://www.w3.org/2002/07/owl#> .
@prefix resolver: <https://uor.foundation/resolver/> .
@prefix type:     <https://uor.foundation/type/> .
@prefix homology: <https://uor.foundation/homology/> .

resolver:ex_mr_180 a owl:NamedIndividual, resolver:ModuliResolver ;
    resolver:moduliTarget      type:ex_ct_180 ;
    resolver:moduliDeformation homology:ex_def_180 .

type:ex_ct_180 a owl:NamedIndividual, type:CompleteType .

homology:ex_def_180 a owl:NamedIndividual, homology:DeformationComplex .
"#;
