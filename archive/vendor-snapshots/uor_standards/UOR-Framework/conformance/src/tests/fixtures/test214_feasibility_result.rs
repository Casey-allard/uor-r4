//! SHACL test 214: `reduction:FeasibilityResult` instance.

/// Instance graph for Test 214: FeasibilityResult with feasibilityKind.
pub const TEST214_FEASIBILITY_RESULT: &str = r#"
@prefix rdf:     <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:     <http://www.w3.org/2002/07/owl#> .
@prefix xsd:     <http://www.w3.org/2001/XMLSchema#> .
@prefix reduction: <https://uor.foundation/reduction/> .

reduction:fr_example a owl:NamedIndividual, reduction:FeasibilityResult ;
    reduction:feasibilityKind "Feasible"^^xsd:string ;
    reduction:feasibilityWitness "all dispatchers reachable"^^xsd:string .
"#;
