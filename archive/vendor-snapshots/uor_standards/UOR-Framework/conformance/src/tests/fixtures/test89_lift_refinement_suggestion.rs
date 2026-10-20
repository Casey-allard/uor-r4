/// SHACL test 89: LiftRefinementSuggestion with obstructionClass — Q1 failure
/// path (Amendment 39). Validates IncrementalCompletenessResolver rejection.
pub const TEST89_LIFT_REFINEMENT_SUGGESTION: &str = r#"
@prefix rdf:        <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:        <http://www.w3.org/2002/07/owl#> .
@prefix xsd:        <http://www.w3.org/2001/XMLSchema#> .
@prefix resolver:   <https://uor.foundation/resolver/> .
@prefix observable: <https://uor.foundation/observable/> .
@prefix partition:  <https://uor.foundation/partition/> .

resolver:ex_lrs_89 a owl:NamedIndividual, resolver:LiftRefinementSuggestion ;
    resolver:liftSitePosition partition:ex_site_89 ;
    resolver:obstructionClass  observable:ex_oc_89 .

partition:ex_site_89 a owl:NamedIndividual, partition:SiteIndex .

observable:ex_oc_89 a owl:NamedIndividual, observable:LiftObstructionClass .
"#;
