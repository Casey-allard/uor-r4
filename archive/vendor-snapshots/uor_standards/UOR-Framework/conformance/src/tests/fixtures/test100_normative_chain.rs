/// SHACL test 100: Full normative chain — Trace to GeodesicTrace to
/// GeodesicCertificate to GeodesicEvidenceBundle with both sub-predicates
/// (Amendment 40). End-to-end normative stack integrity at Q1.
pub const TEST100_NORMATIVE_CHAIN: &str = r#"
@prefix rdf:   <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:   <http://www.w3.org/2002/07/owl#> .
@prefix xsd:   <http://www.w3.org/2001/XMLSchema#> .
@prefix trace: <https://uor.foundation/trace/> .
@prefix cert:  <https://uor.foundation/cert/> .

# 1. GeodesicTrace at Q1
trace:ex_gt_100 a owl:NamedIndividual, trace:GeodesicTrace ;
    trace:isGeodesic          "true"^^xsd:boolean ;
    trace:isAR1Ordered        "true"^^xsd:boolean ;
    trace:isDC10Selected      "true"^^xsd:boolean ;
    trace:geodesicCertificate cert:ex_gc_100 .

# 2. GeodesicCertificate linking trace and evidence bundle
cert:ex_gc_100 a owl:NamedIndividual, cert:GeodesicCertificate ;
    cert:certifiedGeodesic <https://uor.foundation/trace/geodesic_Q1> ;
    cert:evidenceBundle    cert:ex_geb_100 .

# 3. GeodesicEvidenceBundle with both sub-predicates
cert:ex_geb_100 a owl:NamedIndividual, cert:GeodesicEvidenceBundle ;
    cert:isAR1Ordered   "true"^^xsd:boolean ;
    cert:isDC10Selected "true"^^xsd:boolean .
"#;
