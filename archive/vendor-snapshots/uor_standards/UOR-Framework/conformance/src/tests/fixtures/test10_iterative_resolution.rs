//! Test 10: Iterative resolution (Amendment 11).
//!
//! Validates: `resolver:ResolutionState` + `resolver:RefinementSuggestion` +
//! `derivation:RefinementStep` chain showing the resolution-as-learning loop.

/// Instance graph for Test 10: Iterative resolution.
pub const TEST10_ITERATIVE_RESOLUTION: &str = r#"
@prefix rdf:        <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:        <http://www.w3.org/2002/07/owl#> .
@prefix xsd:        <http://www.w3.org/2001/XMLSchema#> .
@prefix resolver:   <https://uor.foundation/resolver/> .
@prefix type:       <https://uor.foundation/type/> .
@prefix partition:  <https://uor.foundation/partition/> .
@prefix derivation: <https://uor.foundation/derivation/> .

# A resolver performing iterative resolution
<https://uor.foundation/instance/resolver-iter>
    a                       owl:NamedIndividual, resolver:DihedralFactorizationResolver ;
    resolver:strategy       "iterative-dihedral-factorization" ;
    resolver:resolutionState <https://uor.foundation/instance/state-iter-2> .

# Resolution state after 2 iterations (not yet complete)
<https://uor.foundation/instance/state-iter-2>
    a                       owl:NamedIndividual, resolver:ResolutionState ;
    resolver:isComplete     false ;
    resolver:iterationCount "2"^^xsd:nonNegativeInteger ;
    resolver:convergenceRate "0.5"^^xsd:decimal ;
    resolver:siteDeficit   <https://uor.foundation/instance/deficit-budget> ;
    resolver:suggestion     <https://uor.foundation/instance/suggestion-1> .

# The deficit budget: 2 sites still free
<https://uor.foundation/instance/deficit-budget>
    a                       owl:NamedIndividual, partition:FreeRank ;
    partition:totalSites   "8"^^xsd:nonNegativeInteger ;
    partition:pinnedCount   "6"^^xsd:nonNegativeInteger ;
    partition:freeRank      "2"^^xsd:nonNegativeInteger ;
    partition:isClosed      false .

# A refinement suggestion
<https://uor.foundation/instance/suggestion-1>
    a                       owl:NamedIndividual, resolver:RefinementSuggestion ;
    resolver:suggestedAxis  type:horizontalAxis ;
    resolver:suggestedClass type:CarryConstraint .

# A refinement step in the derivation
<https://uor.foundation/instance/refinement-step-1>
    a                       owl:NamedIndividual, derivation:RefinementStep ;
    derivation:previousType <https://uor.foundation/instance/type-partial> ;
    derivation:appliedConstraint <https://uor.foundation/instance/constraint-carry> ;
    derivation:refinedType  <https://uor.foundation/instance/type-narrowed> ;
    derivation:sitesClosed "2"^^xsd:nonNegativeInteger .

<https://uor.foundation/instance/type-partial>
    a                       owl:NamedIndividual, type:ConstrainedType .

<https://uor.foundation/instance/constraint-carry>
    a                       owl:NamedIndividual, type:CarryConstraint .

<https://uor.foundation/instance/type-narrowed>
    a                       owl:NamedIndividual, type:ConstrainedType .
"#;
