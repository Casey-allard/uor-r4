/// SHACL test 66: Geodesic certificate — GeodesicCertificate with
/// certifiedGeodesic, geodesicTrace (Amendment 35).
pub const TEST66_GEODESIC_CERTIFICATE: &str = r#"
@prefix rdf:        <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:        <http://www.w3.org/2002/07/owl#> .
@prefix cert:       <https://uor.foundation/cert/> .
@prefix trace:      <https://uor.foundation/trace/> .

# 1. GeodesicCertificate
cert:ex_gc_66 a owl:NamedIndividual, cert:GeodesicCertificate ;
    cert:certifiedGeodesic trace:ex_gt_66 ;
    cert:geodesicTrace trace:ex_gt_66 .

# 2. Referenced GeodesicTrace
trace:ex_gt_66 a owl:NamedIndividual, trace:GeodesicTrace .
"#;
