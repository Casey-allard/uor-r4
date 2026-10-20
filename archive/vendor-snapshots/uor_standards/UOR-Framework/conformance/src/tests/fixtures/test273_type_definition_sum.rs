//! SHACL test 273: type:SumType canonical individuals.

/// Instance graph for Test 273: EitherType and OptionType individuals.
pub const TEST273_TYPE_DEFINITION_SUM: &str = r#"
@prefix rdf:  <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:  <http://www.w3.org/2002/07/owl#> .
@prefix type: <https://uor.foundation/type/> .

type:EitherType a owl:NamedIndividual, type:SumType .
type:OptionType a owl:NamedIndividual, type:SumType .
"#;
