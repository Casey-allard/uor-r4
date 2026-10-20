//! SHACL test 227: `division:AlgebraCommutator` instance.

/// Instance graph for Test 227: AlgebraCommutator with formula.
pub const TEST227_ALGEBRA_COMMUTATOR: &str = r#"
@prefix rdf:      <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:      <http://www.w3.org/2002/07/owl#> .
@prefix xsd:      <http://www.w3.org/2001/XMLSchema#> .
@prefix division: <https://uor.foundation/division/> .

division:test_commutator a owl:NamedIndividual, division:AlgebraCommutator ;
    division:commutatorFormula "[a,b] = ab - ba" .
"#;
