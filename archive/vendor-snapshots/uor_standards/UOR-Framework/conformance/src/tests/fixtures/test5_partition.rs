//! Test 5: Partition with components summing to 2^n.
//!
//! Validates: `partition:Partition` with 4 components whose cardinality values
//! are powers of 2 consistent with the Witt level n.

/// Instance graph for Test 5: Partition structure.
pub const TEST5_PARTITION: &str = r#"
@prefix rdf:        <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:        <http://www.w3.org/2002/07/owl#> .
@prefix xsd:        <http://www.w3.org/2001/XMLSchema#> .
@prefix partition:  <https://uor.foundation/partition/> .

# The partition of R_4 (Z/16Z) into irreducible components
<https://uor.foundation/instance/partition-R4>
    a                   owl:NamedIndividual, partition:Partition ;
    partition:wittLength   "4"^^xsd:nonNegativeInteger ;
    partition:irreducibles  <https://uor.foundation/instance/irred-set-R4> ;
    partition:reducibles    <https://uor.foundation/instance/red-set-R4> ;
    partition:units         <https://uor.foundation/instance/unit-set-R4> ;
    partition:exterior      <https://uor.foundation/instance/ext-set-R4> .

# Irreducible set: elements that cannot be further factored
<https://uor.foundation/instance/irred-set-R4>
    a                       owl:NamedIndividual, partition:IrreducibleSet ;
    partition:cardinality   "4"^^xsd:nonNegativeInteger ;
    partition:density       "0.25"^^xsd:string .

# Reducible set
<https://uor.foundation/instance/red-set-R4>
    a                       owl:NamedIndividual, partition:ReducibleSet ;
    partition:cardinality   "8"^^xsd:nonNegativeInteger .

# Unit group: invertible elements (units of the ring)
<https://uor.foundation/instance/unit-set-R4>
    a                       owl:NamedIndividual, partition:UnitGroup ;
    partition:cardinality   "2"^^xsd:nonNegativeInteger .

# Complement: elements outside the kernel
<https://uor.foundation/instance/ext-set-R4>
    a                       owl:NamedIndividual, partition:Complement ;
    partition:cardinality   "2"^^xsd:nonNegativeInteger .

# Component members
<https://uor.foundation/instance/component-0>
    a                       owl:NamedIndividual, partition:Component ;
    partition:member        <https://uor.foundation/instance/irred-set-R4> .
"#;
