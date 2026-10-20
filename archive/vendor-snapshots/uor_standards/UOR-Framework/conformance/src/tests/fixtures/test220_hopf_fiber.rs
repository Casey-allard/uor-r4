//! SHACL test 220: `convergence:HopfFiber` instance.

/// Instance graph for Test 220: HopfFiber with dimension, total/base space.
pub const TEST220_HOPF_FIBER: &str = r#"
@prefix rdf:         <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:         <http://www.w3.org/2002/07/owl#> .
@prefix xsd:         <http://www.w3.org/2001/XMLSchema#> .
@prefix convergence: <https://uor.foundation/convergence/> .

convergence:fiber_example a owl:NamedIndividual, convergence:HopfFiber ;
    convergence:fiberDimension "1"^^xsd:nonNegativeInteger ;
    convergence:totalSpace "S³" ;
    convergence:baseSpace "S²" ;
    convergence:fiberSphere "S¹" .
"#;
