/// SHACL test 27: Rewrite rule vocabulary — typed derivation rules.
pub const TEST27_REWRITE_RULE: &str = r#"
@prefix rdf:  <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:  <http://www.w3.org/2002/07/owl#> .
@prefix xsd:  <http://www.w3.org/2001/XMLSchema#> .
@prefix op:   <https://uor.foundation/op/> .
@prefix derivation: <https://uor.foundation/derivation/> .
@prefix schema:     <https://uor.foundation/schema/> .

# RewriteRule vocabulary individuals
derivation:CriticalIdentityRule a derivation:RewriteRule ;
    derivation:groundedIn op:criticalIdentity .

derivation:InvolutionRule a derivation:RewriteRule .
derivation:AssociativityRule a derivation:RewriteRule .
derivation:CommutativityRule a derivation:RewriteRule .
derivation:IdentityElementRule a derivation:RewriteRule .
derivation:NormalizationRule a derivation:RewriteRule .

# A two-step derivation using typed rewrite rules
<https://uor.foundation/instance/deriv-1>
    a derivation:Derivation ;
    derivation:originalTerm <https://uor.foundation/instance/term-neg-bnot-x> ;
    derivation:canonicalTerm <https://uor.foundation/instance/term-succ-x> ;
    derivation:step <https://uor.foundation/instance/step-1>,
                    <https://uor.foundation/instance/step-2> .

<https://uor.foundation/instance/step-1>
    a derivation:RewriteStep ;
    derivation:from <https://uor.foundation/instance/term-neg-bnot-x> ;
    derivation:to <https://uor.foundation/instance/term-succ-x> ;
    derivation:hasRewriteRule derivation:CriticalIdentityRule .

<https://uor.foundation/instance/step-2>
    a derivation:RewriteStep ;
    derivation:from <https://uor.foundation/instance/term-succ-x> ;
    derivation:to <https://uor.foundation/instance/term-succ-x> ;
    derivation:hasRewriteRule derivation:NormalizationRule .
"#;
