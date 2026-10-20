/// SHACL fixture for resolver:WittLevelResolver.
pub const TEST127_WITT_LEVEL_RESOLVER: &str = r#"
@prefix rdf:      <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:      <http://www.w3.org/2002/07/owl#> .
@prefix resolver: <https://uor.foundation/resolver/> .

<urn:test:ql_resolver_1> a owl:NamedIndividual , resolver:WittLevelResolver .
"#;
