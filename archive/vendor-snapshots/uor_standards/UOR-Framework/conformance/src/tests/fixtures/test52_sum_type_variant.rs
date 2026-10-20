/// SHACL test 52: Sum type variant — SumType with variant assertions
/// (Amendment 31, ST_1–ST_2).
pub const TEST52_SUM_TYPE_VARIANT: &str = r#"
@prefix rdf:        <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:        <http://www.w3.org/2002/07/owl#> .
@prefix xsd:        <http://www.w3.org/2001/XMLSchema#> .
@prefix type:       <https://uor.foundation/type/> .

# 1. SumType with two variant types
type:ex_sum_52 a owl:NamedIndividual, type:SumType ;
    type:component type:ex_var_a_52 ;
    type:component type:ex_var_b_52 .

# 2. Variant types
type:ex_var_a_52 a owl:NamedIndividual, type:TypeDefinition .
type:ex_var_b_52 a owl:NamedIndividual, type:TypeDefinition .
"#;
