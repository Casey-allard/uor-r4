//! SHACL test 240: Operad end-to-end — operad + composition together.

/// Instance graph for Test 240: Operad end-to-end with both classes.
pub const TEST240_OPERAD_END_TO_END: &str = r#"
@prefix rdf:    <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:    <http://www.w3.org/2002/07/owl#> .
@prefix xsd:    <http://www.w3.org/2001/XMLSchema#> .
@prefix operad: <https://uor.foundation/operad/> .

operad:test_e2e_operad a owl:NamedIndividual, operad:StructuralOperad ;
    operad:operadDescription "Eight structural types with nesting" .

operad:test_e2e_tree a owl:NamedIndividual, operad:OperadComposition ;
    operad:outerType "Tree" ;
    operad:innerType "Symbol" ;
    operad:composedType "Tree(Symbol)" ;
    operad:composedSiteCount "branching_factor * leaf_count" ;
    operad:composedGrounding "depth-first traversal" .
"#;
