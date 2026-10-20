//! SHACL test 224: `division:NormedDivisionAlgebra` instance.

/// Instance graph for Test 224: NormedDivisionAlgebra with dimension and commutativity.
pub const TEST224_NORMED_DIVISION_ALGEBRA: &str = r#"
@prefix rdf:      <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:      <http://www.w3.org/2002/07/owl#> .
@prefix xsd:      <http://www.w3.org/2001/XMLSchema#> .
@prefix division: <https://uor.foundation/division/> .

division:test_algebra a owl:NamedIndividual, division:NormedDivisionAlgebra ;
    division:algebraDimension "4"^^xsd:nonNegativeInteger ;
    division:isCommutative "false"^^xsd:boolean ;
    division:isAssociative "true"^^xsd:boolean ;
    division:basisElements "{1, i, j, k}" .
"#;
