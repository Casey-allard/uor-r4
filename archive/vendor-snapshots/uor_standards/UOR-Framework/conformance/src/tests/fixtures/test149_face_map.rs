/// SHACL fixture for homology:FaceMap.
pub const TEST149_FACE_MAP: &str = r#"
@prefix rdf:      <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:      <http://www.w3.org/2002/07/owl#> .
@prefix xsd:      <http://www.w3.org/2001/XMLSchema#> .
@prefix homology: <https://uor.foundation/homology/> .

<urn:test:face_map_1> a owl:NamedIndividual , homology:FaceMap ;
    homology:removesVertex "0"^^xsd:nonNegativeInteger .
"#;
