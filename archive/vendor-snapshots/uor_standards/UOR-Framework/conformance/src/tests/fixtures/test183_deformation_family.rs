//! SHACL test 183: `type:DeformationFamily`.

/// Instance graph for Test 183: DeformationFamily with familyParameter
/// and familyPreservesCompleteness.
pub const TEST183_DEFORMATION_FAMILY: &str = r#"
@prefix rdf:  <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:  <http://www.w3.org/2002/07/owl#> .
@prefix xsd:  <http://www.w3.org/2001/XMLSchema#> .
@prefix type: <https://uor.foundation/type/> .

type:ex_df_183 a owl:NamedIndividual, type:DeformationFamily ;
    type:familyParameter type:ex_ct_183 ;
    type:familyPreservesCompleteness "true"^^xsd:boolean .

type:ex_ct_183 a owl:NamedIndividual, type:CompleteType .
"#;
