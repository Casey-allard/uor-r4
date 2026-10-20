//! SHACL test 177: `homology:DeformationComplex` subclassing ChainComplex.

/// Instance graph for Test 177: DeformationComplex.
pub const TEST177_DEFORMATION_COMPLEX: &str = r#"
@prefix rdf:      <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:      <http://www.w3.org/2002/07/owl#> .
@prefix xsd:      <http://www.w3.org/2001/XMLSchema#> .
@prefix homology: <https://uor.foundation/homology/> .
@prefix type:     <https://uor.foundation/type/> .

homology:ex_def_177 a owl:NamedIndividual, homology:DeformationComplex ;
    homology:deformationBase     type:ex_ct_177 ;
    homology:tangentDimension    "3"^^xsd:nonNegativeInteger ;
    homology:obstructionDimension "0"^^xsd:nonNegativeInteger .

type:ex_ct_177 a owl:NamedIndividual, type:CompleteType .
"#;
