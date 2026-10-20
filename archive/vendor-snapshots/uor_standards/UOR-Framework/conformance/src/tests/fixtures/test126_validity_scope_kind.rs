/// SHACL fixture for op:ValidityScopeKind (enum class).
pub const TEST126_VALIDITY_SCOPE_KIND: &str = r#"
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl: <http://www.w3.org/2002/07/owl#> .
@prefix op:  <https://uor.foundation/op/> .

op:Universal a owl:NamedIndividual , op:ValidityScopeKind .
op:ParametricLower a owl:NamedIndividual , op:ValidityScopeKind .
op:ParametricRange a owl:NamedIndividual , op:ValidityScopeKind .
op:LevelSpecific a owl:NamedIndividual , op:ValidityScopeKind .
"#;
