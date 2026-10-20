//! SHACL test 239: `operad:OperadComposition` instance.

/// Instance graph for Test 239: OperadComposition nesting Table(Tuple).
pub const TEST239_OPERAD_COMPOSITION: &str = r#"
@prefix rdf:    <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:    <http://www.w3.org/2002/07/owl#> .
@prefix xsd:    <http://www.w3.org/2001/XMLSchema#> .
@prefix operad: <https://uor.foundation/operad/> .

operad:test_composition a owl:NamedIndividual, operad:OperadComposition ;
    operad:outerType "Table" ;
    operad:innerType "Tuple" ;
    operad:composedType "Table(Tuple)" ;
    operad:composedSiteCount "n_rows * n_fields" ;
    operad:composedGrounding "row-major flattening" .
"#;
