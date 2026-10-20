//! SHACL test 241: `interaction:AssociatorTriple`.

/// Instance graph for Test 241: AssociatorTriple.
pub const TEST241_ASSOCIATOR_TRIPLE: &str = r#"
@prefix rdf:         <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:         <http://www.w3.org/2002/07/owl#> .
@prefix xsd:         <http://www.w3.org/2001/XMLSchema#> .
@prefix interaction: <https://uor.foundation/interaction/> .

interaction:ex_triple_241 a owl:NamedIndividual, interaction:AssociatorTriple ;
    interaction:tripleEntityA "entity_A" ;
    interaction:tripleEntityB "entity_B" ;
    interaction:tripleEntityC "entity_C" .
"#;
