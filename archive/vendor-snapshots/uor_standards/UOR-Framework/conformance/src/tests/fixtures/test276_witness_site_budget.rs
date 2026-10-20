//! SHACL test 276: `conformance:WitnessSiteBudget` instance.

/// Instance graph for Test 276: WitnessSiteBudget (opaque site budget).
pub const TEST276_WITNESS_SITE_BUDGET: &str = r#"
@prefix rdf:         <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:         <http://www.w3.org/2002/07/owl#> .
@prefix conformance: <https://uor.foundation/conformance/> .

conformance:ex_site_budget_276 a owl:NamedIndividual, conformance:WitnessSiteBudget .
"#;
