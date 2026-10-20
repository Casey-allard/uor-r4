//! SHACL test 169: `op:ArithmeticValuation` verification domain.
//!
//! Validates that the ArithmeticValuation verification domain individual
//! is present as a named individual of type `op:VerificationDomain`.

/// Instance graph for Test 169: op:ArithmeticValuation domain.
pub const TEST169_ARITHMETIC_VALUATION: &str = r#"
@prefix rdf:  <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:  <http://www.w3.org/2002/07/owl#> .
@prefix op:   <https://uor.foundation/op/> .

op:ArithmeticValuation
    a owl:NamedIndividual, op:VerificationDomain .
"#;
