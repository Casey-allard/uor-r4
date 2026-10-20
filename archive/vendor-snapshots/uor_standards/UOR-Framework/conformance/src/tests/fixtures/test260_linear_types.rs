//! SHACL test 260: `linear` namespace types.

/// Instance graph for Test 260: Linear site and resource types.
pub const TEST260_LINEAR_TYPES: &str = r#"
@prefix rdf:    <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:    <http://www.w3.org/2002/07/owl#> .
@prefix linear: <https://uor.foundation/linear/> .

linear:ex_site_260 a owl:NamedIndividual, linear:LinearSite .
linear:ex_effect_260 a owl:NamedIndividual, linear:LinearEffect .
linear:ex_trace_260 a owl:NamedIndividual, linear:LinearTrace .
linear:ex_budget_260 a owl:NamedIndividual, linear:LinearBudget .
linear:ex_lease_260 a owl:NamedIndividual, linear:LeaseAllocation .
linear:ex_affine_260 a owl:NamedIndividual, linear:AffineSite .
"#;
