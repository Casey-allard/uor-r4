//! Test 2: All 10 named operation individuals.
//!
//! Validates: `op:neg`, `op:bnot`, `op:succ`, `op:pred`, `op:add`, `op:sub`,
//! `op:mul`, `op:xor`, `op:and`, `op:or` with correct types.
//! `op:succ` has `op:composedOf` pointing to `rdf:List(op:neg, op:bnot)`.

/// Instance graph for Test 2: Operation primitives.
pub const TEST2_PRIMITIVES: &str = r#"
@prefix rdf:  <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:  <http://www.w3.org/2002/07/owl#> .
@prefix xsd:  <http://www.w3.org/2001/XMLSchema#> .
@prefix op:   <https://uor.foundation/op/> .

# Unary involutions
op:neg
    a                   owl:NamedIndividual, op:Involution, op:UnaryOp, op:Operation ;
    op:arity            "1"^^xsd:nonNegativeInteger ;
    op:geometricCharacter "ring-reflection" .

op:bnot
    a                   owl:NamedIndividual, op:Involution, op:UnaryOp, op:Operation ;
    op:arity            "1"^^xsd:nonNegativeInteger ;
    op:geometricCharacter "hypercube-reflection" .

# succ is the composition neg ∘ bnot
op:succ
    a                   owl:NamedIndividual, op:UnaryOp, op:Operation ;
    op:arity            "1"^^xsd:nonNegativeInteger ;
    op:composedOf       ( op:neg op:bnot ) .

op:pred
    a                   owl:NamedIndividual, op:UnaryOp, op:Operation ;
    op:arity            "1"^^xsd:nonNegativeInteger .

# Binary operations
op:add
    a                   owl:NamedIndividual, op:BinaryOp, op:Operation ;
    op:arity            "2"^^xsd:nonNegativeInteger ;
    op:commutative      true ;
    op:associative      true .

op:sub
    a                   owl:NamedIndividual, op:BinaryOp, op:Operation ;
    op:arity            "2"^^xsd:nonNegativeInteger .

op:mul
    a                   owl:NamedIndividual, op:BinaryOp, op:Operation ;
    op:arity            "2"^^xsd:nonNegativeInteger ;
    op:commutative      true ;
    op:associative      true .

op:xor
    a                   owl:NamedIndividual, op:BinaryOp, op:Operation ;
    op:arity            "2"^^xsd:nonNegativeInteger ;
    op:commutative      true ;
    op:associative      true .

op:and
    a                   owl:NamedIndividual, op:BinaryOp, op:Operation ;
    op:arity            "2"^^xsd:nonNegativeInteger ;
    op:commutative      true ;
    op:associative      true .

op:or
    a                   owl:NamedIndividual, op:BinaryOp, op:Operation ;
    op:arity            "2"^^xsd:nonNegativeInteger ;
    op:commutative      true ;
    op:associative      true .
"#;
