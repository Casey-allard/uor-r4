//! SHACL test 219: `convergence:ConvergenceLevel` instance.

/// Instance graph for Test 219: ConvergenceLevel with dimension and Betti signature.
pub const TEST219_CONVERGENCE_LEVEL: &str = r#"
@prefix rdf:         <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:         <http://www.w3.org/2002/07/owl#> .
@prefix xsd:         <http://www.w3.org/2001/XMLSchema#> .
@prefix convergence: <https://uor.foundation/convergence/> .

convergence:level_example a owl:NamedIndividual, convergence:ConvergenceLevel ;
    convergence:algebraDimension "2"^^xsd:nonNegativeInteger ;
    convergence:bettiSignature "[1,1]" ;
    convergence:characteristicIdentity "feedback" ;
    convergence:levelName "C" .
"#;
