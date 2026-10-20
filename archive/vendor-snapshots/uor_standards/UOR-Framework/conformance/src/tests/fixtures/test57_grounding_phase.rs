/// SHACL test 57: Grounding phase individuals — Open,
/// PartialGrounding, FullGrounding (Amendment 33).
pub const TEST57_GROUNDING_PHASE: &str = r#"
@prefix rdf:        <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:        <http://www.w3.org/2002/07/owl#> .
@prefix state:      <https://uor.foundation/state/> .

# 1. GroundingPhase individuals
state:Open a owl:NamedIndividual, state:GroundingPhase .
state:PartialGrounding a owl:NamedIndividual, state:GroundingPhase .
state:FullGrounding a owl:NamedIndividual, state:GroundingPhase .
"#;
