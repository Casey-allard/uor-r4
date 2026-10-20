//! SHACL test 261: `recursion` namespace types.

/// Instance graph for Test 261: Recursion types.
pub const TEST261_RECURSION_TYPES: &str = r#"
@prefix rdf:       <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:       <http://www.w3.org/2002/07/owl#> .
@prefix recursion: <https://uor.foundation/recursion/> .

recursion:ex_bounded_261 a owl:NamedIndividual, recursion:BoundedRecursion .
recursion:ex_measure_261 a owl:NamedIndividual, recursion:DescentMeasure .
recursion:ex_base_261 a owl:NamedIndividual, recursion:BaseCase .
recursion:ex_recursive_261 a owl:NamedIndividual, recursion:RecursiveCase .
recursion:ex_step_261 a owl:NamedIndividual, recursion:RecursiveStep .
recursion:ex_trace_261 a owl:NamedIndividual, recursion:RecursionTrace .
recursion:ex_structural_261 a owl:NamedIndividual, recursion:StructuralRecursion .
"#;
