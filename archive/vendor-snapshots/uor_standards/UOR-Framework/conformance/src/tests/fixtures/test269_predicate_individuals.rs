//! SHACL test 269: predicate:Predicate canonical individuals.

/// Instance graph for Test 269: All 12 predicate registry individuals.
pub const TEST269_PREDICATE_INDIVIDUALS: &str = r#"
@prefix rdf:       <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:       <http://www.w3.org/2002/07/owl#> .
@prefix predicate: <https://uor.foundation/predicate/> .

predicate:always             a owl:NamedIndividual, predicate:Predicate .
predicate:never              a owl:NamedIndividual, predicate:Predicate .
predicate:isZero             a owl:NamedIndividual, predicate:TypePredicate .
predicate:isUnit             a owl:NamedIndividual, predicate:TypePredicate .
predicate:isOdd              a owl:NamedIndividual, predicate:TypePredicate .
predicate:isEven             a owl:NamedIndividual, predicate:TypePredicate .
predicate:isInvolution       a owl:NamedIndividual, predicate:TypePredicate .
predicate:sitePinned        a owl:NamedIndividual, predicate:SitePredicate .
predicate:siteFree          a owl:NamedIndividual, predicate:SitePredicate .
predicate:contradictionReached a owl:NamedIndividual, predicate:StatePredicate .
predicate:budgetExhausted    a owl:NamedIndividual, predicate:StatePredicate .
predicate:reductionConverged   a owl:NamedIndividual, predicate:StatePredicate .
"#;
