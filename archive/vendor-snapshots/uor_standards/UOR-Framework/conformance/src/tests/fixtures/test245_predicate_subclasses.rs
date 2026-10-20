//! SHACL test 245: reduction predicate subclasses.

/// Instance graph for Test 245: Ten predicate subclasses.
pub const TEST245_PREDICATE_SUBCLASSES: &str = r#"
@prefix rdf:     <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:     <http://www.w3.org/2002/07/owl#> .
@prefix xsd:     <http://www.w3.org/2001/XMLSchema#> .
@prefix reduction: <https://uor.foundation/reduction/> .

reduction:ex_comparison a owl:NamedIndividual, reduction:ComparisonPredicate ;
    reduction:comparisonField "freeRank" ;
    reduction:comparisonOperator ">=" ;
    reduction:comparisonValue "3" .

reduction:ex_conjunction a owl:NamedIndividual, reduction:ConjunctionPredicate ;
    reduction:conjuncts "p1" ;
    reduction:conjuncts "p2" .

reduction:ex_disjunction a owl:NamedIndividual, reduction:DisjunctionPredicate ;
    reduction:disjuncts "q1" ;
    reduction:disjuncts "q2" .

reduction:ex_negation a owl:NamedIndividual, reduction:NegationPredicate ;
    reduction:negatedPredicate "p1" .

reduction:ex_membership a owl:NamedIndividual, reduction:MembershipPredicate ;
    reduction:membershipSet "active_sites" ;
    reduction:membershipElement "site_7" .

reduction:ex_grounding a owl:NamedIndividual, reduction:GroundingPredicate ;
    reduction:groundingThreshold "0.75" .

reduction:ex_coverage a owl:NamedIndividual, reduction:SiteCoveragePredicate ;
    reduction:coverageTarget "all_critical" .

reduction:ex_equals a owl:NamedIndividual, reduction:EqualsPredicate ;
    reduction:equalityLeft "x" ;
    reduction:equalityRight "y" .

reduction:ex_non_null a owl:NamedIndividual, reduction:NonNullPredicate ;
    reduction:nonNullField "resolver_output" .

reduction:ex_query_subtype a owl:NamedIndividual, reduction:QuerySubtypePredicate ;
    reduction:queryTypeRef "SessionQuery" .
"#;
