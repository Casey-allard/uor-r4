//! SHACL test 249: `reduction:PropertyBind` and `reduction:StageAdvance`.

/// Instance graph for Test 249: PropertyBind and StageAdvance.
pub const TEST249_PROPERTY_BIND_ADVANCE: &str = r#"
@prefix rdf:     <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:     <http://www.w3.org/2002/07/owl#> .
@prefix xsd:     <http://www.w3.org/2001/XMLSchema#> .
@prefix reduction: <https://uor.foundation/reduction/> .

reduction:ex_bind_249 a owl:NamedIndividual, reduction:PropertyBind ;
    reduction:bindTarget "site_3" ;
    reduction:bindValue "42" .

reduction:ex_advance_249 a owl:NamedIndividual, reduction:StageAdvance ;
    reduction:advanceFrom reduction:Declare ;
    reduction:advanceTo reduction:Factorize .
"#;
