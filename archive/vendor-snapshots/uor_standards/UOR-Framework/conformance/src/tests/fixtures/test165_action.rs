//! SHACL test 165: `morphism:Action` — group action on type space.
//!
//! An `Action` records the mechanism by which a group applies transforms
//! to a set. The dihedral group D_{2^n} acts on type space by isometries.

/// Instance graph for Test 165: morphism:Action.
pub const TEST165_ACTION: &str = r#"
@prefix rdf:      <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:      <http://www.w3.org/2002/07/owl#> .
@prefix xsd:      <http://www.w3.org/2001/XMLSchema#> .
@prefix morphism: <https://uor.foundation/morphism/> .
@prefix op:       <https://uor.foundation/op/> .

<https://uor.foundation/instance/dihedral_action_n>
    a                        owl:NamedIndividual, morphism:Action ;
    morphism:group           op:DihedralGroup ;
    morphism:actionIsometry  "true"^^xsd:boolean .
"#;
