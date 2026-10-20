//! SHACL test 182: `observable:WhiteheadProduct`.

/// Instance graph for Test 182: WhiteheadProduct with whiteheadTrivial.
pub const TEST182_WHITEHEAD_PRODUCT: &str = r#"
@prefix rdf:        <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:        <http://www.w3.org/2002/07/owl#> .
@prefix xsd:        <http://www.w3.org/2001/XMLSchema#> .
@prefix observable: <https://uor.foundation/observable/> .

observable:ex_wp_182 a owl:NamedIndividual, observable:WhiteheadProduct ;
    observable:whiteheadTrivial "false"^^xsd:boolean .
"#;
