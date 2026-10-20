//! SHACL test 196: `type:ProductType` structural instances (SequenceType, TupleType).

/// Instance graph for Test 196: SequenceType and TupleType structural type individuals.
pub const TEST196_SEQUENCE_TUPLE_TYPE: &str = r#"
@prefix rdf:  <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:  <http://www.w3.org/2002/07/owl#> .
@prefix xsd:  <http://www.w3.org/2001/XMLSchema#> .
@prefix type: <https://uor.foundation/type/> .

type:SequenceType a owl:NamedIndividual, type:ProductType ;
    type:structuralSiteCount "sum of element site counts" ;
    type:structuralGrounding "free monoid on element type, backbone constraint" ;
    type:structuralConstraint "backbone ordering: elements indexed by position" .

type:TupleType a owl:NamedIndividual, type:ProductType ;
    type:structuralSiteCount "sum of field site counts" ;
    type:structuralGrounding "site ordering follows CY_6 (optimal site ordering)" .
"#;
