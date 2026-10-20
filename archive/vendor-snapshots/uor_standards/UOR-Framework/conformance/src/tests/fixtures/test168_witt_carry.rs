//! SHACL test 168: `op:WC_1` — Witt coordinate identification identity.
//!
//! Validates that the WC_ series identity individuals are present as
//! named individuals of type `op:Identity`.

/// Instance graph for Test 168: op:WC_1 Witt-carry identity.
pub const TEST168_WITT_CARRY: &str = r#"
@prefix rdf:  <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:  <http://www.w3.org/2002/07/owl#> .
@prefix op:   <https://uor.foundation/op/> .

op:WC_1
    a owl:NamedIndividual, op:Identity .
"#;
