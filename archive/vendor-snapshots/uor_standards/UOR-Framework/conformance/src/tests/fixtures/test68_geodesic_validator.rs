/// SHACL test 68: Geodesic validator — GeodesicValidator (resolver) with
/// validateGeodesic (Amendment 35).
pub const TEST68_GEODESIC_VALIDATOR: &str = r#"
@prefix rdf:        <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:        <http://www.w3.org/2002/07/owl#> .
@prefix resolver:   <https://uor.foundation/resolver/> .
@prefix trace:      <https://uor.foundation/trace/> .

# 1. GeodesicValidator
resolver:ex_gval_68 a owl:NamedIndividual, resolver:GeodesicValidator ;
    resolver:validateGeodesic trace:ex_gt_68 .

# 2. Referenced GeodesicTrace
trace:ex_gt_68 a owl:NamedIndividual, trace:GeodesicTrace .
"#;
