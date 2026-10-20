//! SHACL test 228: `division:AlgebraAssociator` instance.

/// Instance graph for Test 228: AlgebraAssociator with formula.
pub const TEST228_ALGEBRA_ASSOCIATOR: &str = r#"
@prefix rdf:      <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:      <http://www.w3.org/2002/07/owl#> .
@prefix xsd:      <http://www.w3.org/2001/XMLSchema#> .
@prefix division: <https://uor.foundation/division/> .

division:test_associator a owl:NamedIndividual, division:AlgebraAssociator ;
    division:associatorFormula "[a,b,c] = (ab)c - a(bc)" .
"#;
