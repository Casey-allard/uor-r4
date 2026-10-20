/// SHACL test 51: Product type pipeline — ProductType with component
/// assertions (Amendment 31, PT_1–PT_4).
pub const TEST51_PRODUCT_TYPE_PIPELINE: &str = r#"
@prefix rdf:        <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:        <http://www.w3.org/2002/07/owl#> .
@prefix xsd:        <http://www.w3.org/2001/XMLSchema#> .
@prefix type:       <https://uor.foundation/type/> .
@prefix partition:  <https://uor.foundation/partition/> .

# 1. ProductType with two component types
type:ex_prod_51 a owl:NamedIndividual, type:ProductType ;
    type:component type:ex_comp_a_51 ;
    type:component type:ex_comp_b_51 .

# 2. Component types
type:ex_comp_a_51 a owl:NamedIndividual, type:TypeDefinition .
type:ex_comp_b_51 a owl:NamedIndividual, type:TypeDefinition .

# 3. FreeRank for the product
partition:ex_fb_51 a owl:NamedIndividual, partition:FreeRank ;
    partition:siteCount "6"^^xsd:nonNegativeInteger .
"#;
