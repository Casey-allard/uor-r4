//! SHACL test 223: `convergence:AssociativeSubalgebra` instance.

/// Instance graph for Test 223: AssociativeSubalgebra with description.
pub const TEST223_ASSOCIATIVE_SUBALGEBRA: &str = r#"
@prefix rdf:         <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:         <http://www.w3.org/2002/07/owl#> .
@prefix xsd:         <http://www.w3.org/2001/XMLSchema#> .
@prefix convergence: <https://uor.foundation/convergence/> .

convergence:subalgebra_example a owl:NamedIndividual, convergence:AssociativeSubalgebra ;
    convergence:subalgebraDescription "H subset of O" .
"#;
