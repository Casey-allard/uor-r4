/// SHACL fixture for op:DihedralGroup.
pub const TEST125_DIHEDRAL_GROUP: &str = r#"
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl: <http://www.w3.org/2002/07/owl#> .
@prefix op:  <https://uor.foundation/op/> .

<urn:test:dihedral_1> a owl:NamedIndividual , op:DihedralGroup .
"#;
