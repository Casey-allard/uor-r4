//! Test 7: End-to-end cycle.
//!
//! Validates a full resolution cycle:
//! Context → Type → Resolver → Partition → Observable → Cert → Trace → Transform.

/// Instance graph for Test 7: End-to-end resolution cycle.
pub const TEST7_END_TO_END: &str = r#"
@prefix rdf:        <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:        <http://www.w3.org/2002/07/owl#> .
@prefix xsd:        <http://www.w3.org/2001/XMLSchema#> .
@prefix state:      <https://uor.foundation/state/> .
@prefix type:       <https://uor.foundation/type/> .
@prefix resolver:   <https://uor.foundation/resolver/> .
@prefix partition:  <https://uor.foundation/partition/> .
@prefix observable: <https://uor.foundation/observable/> .
@prefix cert:       <https://uor.foundation/cert/> .
@prefix trace:      <https://uor.foundation/trace/> .
@prefix morphism:   <https://uor.foundation/morphism/> .
@prefix schema:     <https://uor.foundation/schema/> .

# 1. Context — the evaluation environment
<https://uor.foundation/instance/e2e/ctx>
    a               owl:NamedIndividual, state:Context ;
    state:wittLength   "8"^^xsd:nonNegativeInteger .

# 2. Type — the type of the value being resolved
<https://uor.foundation/instance/e2e/type-u8>
    a               owl:NamedIndividual, type:PrimitiveType ;
    type:bitWidth   "8"^^xsd:nonNegativeInteger .

# 3. Resolver — resolves the type in the ring
<https://uor.foundation/instance/e2e/resolver>
    a                   owl:NamedIndividual, resolver:Resolver ;
    resolver:inputType  <https://uor.foundation/instance/e2e/type-u8> ;
    resolver:strategy   "dihedral-factorization" .

# 4. Partition — the partition of R_8 produced by the resolver
<https://uor.foundation/instance/e2e/partition>
    a                   owl:NamedIndividual, partition:Partition ;
    partition:wittLength   "8"^^xsd:nonNegativeInteger .

# 5. Observable — a metric measurement on the resolved value
<https://uor.foundation/instance/e2e/observable>
    a                   owl:NamedIndividual, observable:Observable ;
    observable:value    "42"^^xsd:integer ;
    observable:source   <https://uor.foundation/instance/e2e/partition> .

# 6. Certificate — attests the resolution is correct
<https://uor.foundation/instance/e2e/cert>
    a                   owl:NamedIndividual, cert:Certificate ;
    cert:certifies      <https://uor.foundation/instance/e2e/observable> .

# 7. Trace — the computation trace of the resolution
<https://uor.foundation/instance/e2e/trace>
    a                   owl:NamedIndividual, trace:ComputationTrace ;
    trace:certifiedBy   <https://uor.foundation/instance/e2e/cert> .

# 8. Transform — the morphism applied during resolution
<https://uor.foundation/instance/e2e/transform>
    a                       owl:NamedIndividual, morphism:Transform ;
    morphism:source         <https://uor.foundation/instance/e2e/type-u8> ;
    morphism:trace          <https://uor.foundation/instance/e2e/trace> .
"#;
