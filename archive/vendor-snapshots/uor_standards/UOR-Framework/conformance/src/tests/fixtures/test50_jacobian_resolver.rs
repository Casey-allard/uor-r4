/// SHACL test 50: Jacobian-guided resolver — JacobianGuidedResolver +
/// ResolutionState + guidingJacobian (Amendment 31, DC_10).
pub const TEST50_JACOBIAN_RESOLVER: &str = r#"
@prefix rdf:        <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:        <http://www.w3.org/2002/07/owl#> .
@prefix xsd:        <http://www.w3.org/2001/XMLSchema#> .
@prefix resolver:   <https://uor.foundation/resolver/> .
@prefix observable: <https://uor.foundation/observable/> .

# 1. JacobianGuidedResolver instance
resolver:ex_jgr_50 a owl:NamedIndividual, resolver:JacobianGuidedResolver .

# 2. ResolutionState with guiding Jacobian
resolver:ex_rs_50 a owl:NamedIndividual, resolver:ResolutionState ;
    resolver:guidingJacobian observable:ex_jac_50 .

# 3. Jacobian observable
observable:ex_jac_50 a owl:NamedIndividual, observable:Jacobian ;
    observable:value "1.5"^^xsd:decimal .
"#;
