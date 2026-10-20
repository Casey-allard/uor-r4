//! SHACL test 262: `region` namespace types.

/// Instance graph for Test 262: Region types.
pub const TEST262_REGION_TYPES: &str = r#"
@prefix rdf:    <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:    <http://www.w3.org/2002/07/owl#> .
@prefix region: <https://uor.foundation/region/> .

region:ex_address_262 a owl:NamedIndividual, region:AddressRegion .
region:ex_bound_262 a owl:NamedIndividual, region:RegionBound .
region:ex_metric_262 a owl:NamedIndividual, region:LocalityMetric .
region:ex_working_262 a owl:NamedIndividual, region:WorkingSet .
region:ex_alloc_262 a owl:NamedIndividual, region:RegionAllocation .
"#;
