//! Test 1: Ring substrate at Witt level n.
//!
//! Validates: `schema:Ring` with all required properties (ringWittLength, modulus,
//! generator, negation, complement) and valid generator/involution links.

/// Instance graph for Test 1: Ring substrate.
pub const TEST1_RING: &str = r#"
@prefix rdf:    <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:    <http://www.w3.org/2002/07/owl#> .
@prefix xsd:    <http://www.w3.org/2001/XMLSchema#> .
@prefix schema: <https://uor.foundation/schema/> .
@prefix op:     <https://uor.foundation/op/> .

# The ring R_8 = Z/256Z (n=8)
<https://uor.foundation/instance/ring-R8>
    a                   owl:NamedIndividual, schema:Ring ;
    schema:ringWittLength  "8"^^xsd:nonNegativeInteger ;
    schema:modulus      "256"^^xsd:nonNegativeInteger ;
    schema:generator    op:neg ;
    schema:negation     op:neg ;
    schema:complement   op:bnot .

# Verify that the generator (op:neg) is an Involution
op:neg  a   owl:NamedIndividual, op:Involution, op:UnaryOp, op:Operation .
op:bnot a   owl:NamedIndividual, op:Involution, op:UnaryOp, op:Operation .
"#;
