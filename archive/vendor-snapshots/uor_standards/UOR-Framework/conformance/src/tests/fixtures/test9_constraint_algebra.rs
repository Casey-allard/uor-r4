//! Test 9: Constraint algebra (Amendment 10).
//!
//! Validates: `type:ResidueConstraint` + `type:CompositeConstraint` +
//! `type:MetricAxis` individuals, `type:metricAxis`, `type:hasConstraint`.

/// Instance graph for Test 9: Constraint algebra.
pub const TEST9_CONSTRAINT_ALGEBRA: &str = r#"
@prefix rdf:    <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:    <http://www.w3.org/2002/07/owl#> .
@prefix xsd:    <http://www.w3.org/2001/XMLSchema#> .
@prefix type:   <https://uor.foundation/type/> .

# A residue constraint: x ≡ 1 (mod 4)
<https://uor.foundation/instance/constraint-res-mod4>
    a               owl:NamedIndividual, type:ResidueConstraint ;
    type:modulus    "4"^^xsd:positiveInteger ;
    type:residue   "1"^^xsd:nonNegativeInteger ;
    type:metricAxis type:verticalAxis ;
    type:crossingCost "0"^^xsd:nonNegativeInteger .

# A carry constraint
<https://uor.foundation/instance/constraint-carry>
    a               owl:NamedIndividual, type:CarryConstraint ;
    type:carryPattern "1010" ;
    type:metricAxis type:horizontalAxis .

# A depth constraint
<https://uor.foundation/instance/constraint-depth>
    a               owl:NamedIndividual, type:DepthConstraint ;
    type:minDepth   "1"^^xsd:nonNegativeInteger ;
    type:maxDepth   "3"^^xsd:nonNegativeInteger ;
    type:metricAxis type:diagonalAxis .

# A composite constraint combining residue + carry
<https://uor.foundation/instance/constraint-composite>
    a                   owl:NamedIndividual, type:CompositeConstraint ;
    type:composedFrom   <https://uor.foundation/instance/constraint-res-mod4> ;
    type:composedFrom   <https://uor.foundation/instance/constraint-carry> ;
    type:crossingCost   "1"^^xsd:nonNegativeInteger .

# A constrained type using the composite constraint
<https://uor.foundation/instance/type-constrained-u8>
    a                   owl:NamedIndividual, type:ConstrainedType ;
    type:baseType       <https://uor.foundation/instance/type-u8> ;
    type:hasConstraint  <https://uor.foundation/instance/constraint-composite> .

<https://uor.foundation/instance/type-u8>
    a                   owl:NamedIndividual, type:PrimitiveType ;
    type:bitWidth       "8"^^xsd:positiveInteger .
"#;
