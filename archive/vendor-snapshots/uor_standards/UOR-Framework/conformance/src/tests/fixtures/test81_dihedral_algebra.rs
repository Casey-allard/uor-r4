/// SHACL test 81: DihedralElement with rotationExponent and reflectionBit (Amendment 37, Gap 6).
pub const TEST81_DIHEDRAL_ALGEBRA: &str = r#"
@prefix rdf:        <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:        <http://www.w3.org/2002/07/owl#> .
@prefix xsd:        <http://www.w3.org/2001/XMLSchema#> .
@prefix observable: <https://uor.foundation/observable/> .

observable:ex_de_81 a owl:NamedIndividual, observable:DihedralElement ;
    observable:rotationExponent "3"^^xsd:nonNegativeInteger ;
    observable:reflectionBit "true"^^xsd:boolean .
"#;
