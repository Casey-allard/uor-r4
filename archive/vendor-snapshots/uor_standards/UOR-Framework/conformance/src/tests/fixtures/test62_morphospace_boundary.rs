/// SHACL test 62: Morphospace boundary — MorphospaceBoundary with boundaryType
/// (Amendment 34).
pub const TEST62_MORPHOSPACE_BOUNDARY: &str = r#"
@prefix rdf:        <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:        <http://www.w3.org/2002/07/owl#> .
@prefix xsd:        <http://www.w3.org/2001/XMLSchema#> .
@prefix observable: <https://uor.foundation/observable/> .

# 1. MorphospaceBoundary
observable:ex_mb_62 a owl:NamedIndividual, observable:MorphospaceBoundary ;
    observable:boundaryType "topological"^^xsd:string .
"#;
