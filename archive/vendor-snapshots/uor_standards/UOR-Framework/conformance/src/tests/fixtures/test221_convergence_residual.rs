//! SHACL test 221: `convergence:ConvergenceResidual` instance.

/// Instance graph for Test 221: ConvergenceResidual with Betti and dimension.
pub const TEST221_CONVERGENCE_RESIDUAL: &str = r#"
@prefix rdf:         <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:         <http://www.w3.org/2002/07/owl#> .
@prefix xsd:         <http://www.w3.org/2001/XMLSchema#> .
@prefix convergence: <https://uor.foundation/convergence/> .

convergence:residual_example a owl:NamedIndividual, convergence:ConvergenceResidual ;
    convergence:residualBetti "1" ;
    convergence:residualDimension "7"^^xsd:nonNegativeInteger .
"#;
