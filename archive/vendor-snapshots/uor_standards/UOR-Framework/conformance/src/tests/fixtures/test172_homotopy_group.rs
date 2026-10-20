//! SHACL test 172: `observable:HomotopyGroup` for dimensions k=0, k=1, k=2.

/// Instance graph for Test 172: HomotopyGroup.
pub const TEST172_HOMOTOPY_GROUP: &str = r#"
@prefix rdf:        <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:        <http://www.w3.org/2002/07/owl#> .
@prefix xsd:        <http://www.w3.org/2001/XMLSchema#> .
@prefix observable: <https://uor.foundation/observable/> .
@prefix type:       <https://uor.foundation/type/> .

observable:ex_pi0_172 a owl:NamedIndividual, observable:HomotopyGroup ;
    observable:homotopyDimension "0"^^xsd:nonNegativeInteger ;
    observable:homotopyRank      "1"^^xsd:nonNegativeInteger ;
    observable:homotopyBasepoint type:ex_constraint_172 .

observable:ex_pi1_172 a owl:NamedIndividual, observable:HomotopyGroup ;
    observable:homotopyDimension "1"^^xsd:nonNegativeInteger ;
    observable:homotopyRank      "0"^^xsd:nonNegativeInteger ;
    observable:homotopyBasepoint type:ex_constraint_172 .

observable:ex_pi2_172 a owl:NamedIndividual, observable:HomotopyGroup ;
    observable:homotopyDimension "2"^^xsd:nonNegativeInteger ;
    observable:homotopyRank      "0"^^xsd:nonNegativeInteger ;
    observable:homotopyBasepoint type:ex_constraint_172 .

type:ex_constraint_172 a owl:NamedIndividual, type:Constraint .
"#;
