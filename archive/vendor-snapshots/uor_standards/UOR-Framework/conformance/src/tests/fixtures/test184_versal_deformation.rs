//! SHACL test 184: `type:VersalDeformation`.

/// Instance graph for Test 184: VersalDeformation with versalBase and
/// versalDimension.
pub const TEST184_VERSAL_DEFORMATION: &str = r#"
@prefix rdf:  <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:  <http://www.w3.org/2002/07/owl#> .
@prefix xsd:  <http://www.w3.org/2001/XMLSchema#> .
@prefix type: <https://uor.foundation/type/> .

type:ex_vd_184 a owl:NamedIndividual, type:VersalDeformation ;
    type:versalBase type:ex_ct_184 ;
    type:versalDimension "3"^^xsd:nonNegativeInteger .

type:ex_ct_184 a owl:NamedIndividual, type:CompleteType .
"#;
