//! SHACL test 222: `convergence:CommutativeSubspace` instance.

/// Instance graph for Test 222: CommutativeSubspace with description.
pub const TEST222_COMMUTATIVE_SUBSPACE: &str = r#"
@prefix rdf:         <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:         <http://www.w3.org/2002/07/owl#> .
@prefix xsd:         <http://www.w3.org/2001/XMLSchema#> .
@prefix convergence: <https://uor.foundation/convergence/> .

convergence:subspace_example a owl:NamedIndividual, convergence:CommutativeSubspace ;
    convergence:subspaceDescription "U(1) subset of SU(2)" .
"#;
