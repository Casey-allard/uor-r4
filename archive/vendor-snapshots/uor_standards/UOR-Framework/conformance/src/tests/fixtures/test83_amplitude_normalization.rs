/// SHACL test 83: SuperposedSiteState with normalizationVerified (Amendment 37, Gap 1).
pub const TEST83_AMPLITUDE_NORMALIZATION: &str = r#"
@prefix rdf:  <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:  <http://www.w3.org/2002/07/owl#> .
@prefix xsd:  <http://www.w3.org/2001/XMLSchema#> .
@prefix type: <https://uor.foundation/type/> .

type:ex_sfs_83 a owl:NamedIndividual, type:SuperposedSiteState ;
    type:amplitude "0.707"^^xsd:decimal ;
    type:normalizationVerified "true"^^xsd:boolean .
"#;
