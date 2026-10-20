//! SHACL test 176: `type:ModuliSpace` with quantum level and points.

/// Instance graph for Test 176: ModuliSpace.
pub const TEST176_MODULI_SPACE: &str = r#"
@prefix rdf:    <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:    <http://www.w3.org/2002/07/owl#> .
@prefix xsd:    <http://www.w3.org/2001/XMLSchema#> .
@prefix schema: <https://uor.foundation/schema/> .
@prefix type:   <https://uor.foundation/type/> .

type:ex_moduli_176 a owl:NamedIndividual, type:ModuliSpace ;
    type:moduliWittLevel schema:Q0 ;
    type:moduliDimension    "5"^^xsd:nonNegativeInteger ;
    type:moduliPoint        type:ex_ct_176 .

type:ex_ct_176 a owl:NamedIndividual, type:CompleteType .
"#;
