//! Test 13: Canonical form case study.
//!
//! Validates: RepresentationQuery → CanonicalFormResolver → Derivation
//! (RewriteStep chain) → TermMetrics.

/// Instance graph for Test 13: Canonical form.
pub const TEST13_CANONICAL_FORM: &str = r#"
@prefix rdf:        <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:        <http://www.w3.org/2002/07/owl#> .
@prefix xsd:        <http://www.w3.org/2001/XMLSchema#> .
@prefix query:      <https://uor.foundation/query/> .
@prefix resolver:   <https://uor.foundation/resolver/> .
@prefix derivation: <https://uor.foundation/derivation/> .
@prefix schema:     <https://uor.foundation/schema/> .

# 1. Query — representation query
<https://uor.foundation/instance/canon/query>
    a               owl:NamedIndividual, query:RepresentationQuery .

# 2. Resolver — canonical form resolver
<https://uor.foundation/instance/canon/resolver>
    a                   owl:NamedIndividual, resolver:CanonicalFormResolver ;
    resolver:strategy   "canonical-form-rewriting" .

# 3. Derivation with rewrite steps
<https://uor.foundation/instance/canon/derivation>
    a                       owl:NamedIndividual, derivation:Derivation ;
    derivation:originalTerm <https://uor.foundation/instance/canon/term-orig> ;
    derivation:canonicalTerm <https://uor.foundation/instance/canon/term-canon> ;
    derivation:step         <https://uor.foundation/instance/canon/step-1> ;
    derivation:step         <https://uor.foundation/instance/canon/step-2> ;
    derivation:termMetrics  <https://uor.foundation/instance/canon/metrics> .

<https://uor.foundation/instance/canon/term-orig>
    a               owl:NamedIndividual, schema:Application .

<https://uor.foundation/instance/canon/term-canon>
    a               owl:NamedIndividual, schema:Literal .

# Rewrite step 1: apply critical identity
<https://uor.foundation/instance/canon/step-1>
    a                       owl:NamedIndividual, derivation:RewriteStep ;
    derivation:from         <https://uor.foundation/instance/canon/term-orig> ;
    derivation:to           <https://uor.foundation/instance/canon/term-mid> ;
    derivation:hasRewriteRule  derivation:CriticalIdentityRule .

<https://uor.foundation/instance/canon/term-mid>
    a               owl:NamedIndividual, schema:Application .

# Rewrite step 2: normalize
<https://uor.foundation/instance/canon/step-2>
    a                       owl:NamedIndividual, derivation:RewriteStep ;
    derivation:from         <https://uor.foundation/instance/canon/term-mid> ;
    derivation:to           <https://uor.foundation/instance/canon/term-canon> ;
    derivation:hasRewriteRule  derivation:InvolutionRule .

# Term metrics
<https://uor.foundation/instance/canon/metrics>
    a                       owl:NamedIndividual, derivation:TermMetrics ;
    derivation:stepCount    "2"^^xsd:nonNegativeInteger ;
    derivation:termSize     "3"^^xsd:nonNegativeInteger .
"#;
