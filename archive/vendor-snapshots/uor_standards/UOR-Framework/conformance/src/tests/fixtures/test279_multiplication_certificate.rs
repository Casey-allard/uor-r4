//! Test 279: v0.2.2 Phases C.4 + D — multiplication resolver and parametric
//! constraint surface SHACL coverage.
//!
//! Phase C.4:
//! - cert:MultiplicationCertificate with splittingFactor, subMultiplicationCount,
//!   landauerCostNats evidence.
//! - resolver:MultiplicationResolver.
//! - linear:stackBudgetBytes on a LinearBudget.
//!
//! Phase D (Q4):
//! - type:BoundConstraint with the 3 new parametric properties.
//! - type:BoundShape (one instance).
//! - type:Conjunction with conjuncts property.
//! - 4 new observable subclasses (ValueModObservable, DerivationDepthObservable,
//!   CarryDepthObservable, FreeRankObservable).

/// Instance graph for Test 279: v0.2.2 Phases C.4 + D parametric surface.
pub const TEST279_MULTIPLICATION_CERTIFICATE: &str = r#"
@prefix rdf:        <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:        <http://www.w3.org/2002/07/owl#> .
@prefix xsd:        <http://www.w3.org/2001/XMLSchema#> .
@prefix cert:       <https://uor.foundation/cert/> .
@prefix resolver:   <https://uor.foundation/resolver/> .
@prefix linear:     <https://uor.foundation/linear/> .
@prefix type:       <https://uor.foundation/type/> .
@prefix observable: <https://uor.foundation/observable/> .
@prefix derivation: <https://uor.foundation/derivation/> .
@prefix carry:      <https://uor.foundation/carry/> .
@prefix partition:  <https://uor.foundation/partition/> .

# 1. cert:MultiplicationCertificate — resolver-issued multiplication cost proof.
<https://uor.foundation/instance/cert/mult_karatsuba_w4096>
    a                             owl:NamedIndividual, cert:MultiplicationCertificate ;
    cert:splittingFactor          "2"^^xsd:positiveInteger ;
    cert:subMultiplicationCount   "3"^^xsd:nonNegativeInteger ;
    cert:landauerCostNats         "0.0665"^^xsd:decimal .

# 2. resolver:MultiplicationResolver — the resolver class itself.
<https://uor.foundation/instance/resolver/mult_resolver>
    a                             owl:NamedIndividual, resolver:MultiplicationResolver .

# 3. linear:LinearBudget extended with linear:stackBudgetBytes.
<https://uor.foundation/instance/linear/mult_call_site>
    a                             owl:NamedIndividual, linear:LinearBudget ;
    linear:stackBudgetBytes       "16384"^^xsd:nonNegativeInteger .

# v0.2.2 Phase D: parametric constraint surface.

# 4. type:BoundShape — a closed-catalogue predicate form.
<https://uor.foundation/instance/type/example_shape>
    a                             owl:NamedIndividual, type:BoundShape .

# 5. Four new observable subclasses (SHACL fixture coverage).
<https://uor.foundation/instance/observable/example_value_mod>
    a                             owl:NamedIndividual, observable:ValueModObservable .

<https://uor.foundation/instance/derivation/example_depth_obs>
    a                             owl:NamedIndividual, derivation:DerivationDepthObservable .

<https://uor.foundation/instance/carry/example_carry_depth>
    a                             owl:NamedIndividual, carry:CarryDepthObservable .

<https://uor.foundation/instance/partition/example_free_rank>
    a                             owl:NamedIndividual, partition:FreeRankObservable .

# 6. type:BoundConstraint — parametric constraint with all 3 new properties.
<https://uor.foundation/instance/type/example_residue_bound>
    a                             owl:NamedIndividual, type:BoundConstraint ;
    type:boundObservable          <https://uor.foundation/instance/observable/example_value_mod> ;
    type:boundShape               <https://uor.foundation/instance/type/example_shape> ;
    type:boundArguments           "modulus=256;residue=0" .

# 7. type:Conjunction — composition of BoundConstraint instances.
<https://uor.foundation/instance/type/example_conjunction>
    a                             owl:NamedIndividual, type:Conjunction ;
    type:conjuncts                <https://uor.foundation/instance/type/example_residue_bound> .
"#;
