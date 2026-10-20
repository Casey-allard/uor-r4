/// SHACL test 53: Superposed site state — SuperposedSiteState + amplitude +
/// SuperpositionResolver (Amendment 32, RC_5).
pub const TEST53_SUPERPOSED_SITE: &str = r#"
@prefix rdf:        <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:        <http://www.w3.org/2002/07/owl#> .
@prefix xsd:        <http://www.w3.org/2001/XMLSchema#> .
@prefix type:       <https://uor.foundation/type/> .
@prefix resolver:   <https://uor.foundation/resolver/> .

# 1. SuperposedSiteState with amplitude
type:ex_sfs_53 a owl:NamedIndividual, type:SuperposedSiteState ;
    type:amplitude "0.707"^^xsd:decimal .

# 2. SuperpositionResolver
resolver:ex_sr_53 a owl:NamedIndividual, resolver:SuperpositionResolver .
"#;
