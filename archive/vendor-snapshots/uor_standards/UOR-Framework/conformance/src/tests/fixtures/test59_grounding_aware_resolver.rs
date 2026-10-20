/// SHACL test 59: Grounding-aware resolver — GroundingAwareResolver with
/// usedGrounding (Amendment 33).
pub const TEST59_GROUNDING_AWARE_RESOLVER: &str = r#"
@prefix rdf:        <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:        <http://www.w3.org/2002/07/owl#> .
@prefix resolver:   <https://uor.foundation/resolver/> .
@prefix state:      <https://uor.foundation/state/> .

# 1. GroundingAwareResolver
resolver:ex_sar_59 a owl:NamedIndividual, resolver:GroundingAwareResolver ;
    resolver:usedGrounding state:ex_sc_59 .

# 2. Referenced GroundedContext
state:ex_sc_59 a owl:NamedIndividual, state:GroundedContext .
"#;
