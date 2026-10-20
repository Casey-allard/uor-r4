/// SHACL test 64: Achievability status individuals — Achievable, Forbidden
/// (Amendment 34).
pub const TEST64_ACHIEVABILITY_STATUS: &str = r#"
@prefix rdf:        <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:        <http://www.w3.org/2002/07/owl#> .
@prefix observable: <https://uor.foundation/observable/> .

# 1. AchievabilityStatus individuals
observable:Achievable a owl:NamedIndividual, observable:AchievabilityStatus .
observable:Forbidden a owl:NamedIndividual, observable:AchievabilityStatus .
"#;
