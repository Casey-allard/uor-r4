/// SHACL test 69: Geodesic ordered — GeodesicTrace with adiabaticallyOrdered,
/// jacobianAtStep (Amendment 35).
pub const TEST69_GEODESIC_ORDERED: &str = r#"
@prefix rdf:        <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:        <http://www.w3.org/2002/07/owl#> .
@prefix xsd:        <http://www.w3.org/2001/XMLSchema#> .
@prefix trace:      <https://uor.foundation/trace/> .
@prefix observable: <https://uor.foundation/observable/> .

# 1. GeodesicTrace with adiabatic ordering and jacobian
trace:ex_go_69 a owl:NamedIndividual, trace:GeodesicTrace ;
    trace:adiabaticallyOrdered "true"^^xsd:boolean ;
    trace:jacobianAtStep observable:ex_jac_69 ;
    trace:isGeodesic "true"^^xsd:boolean .

# 2. Referenced Jacobian
observable:ex_jac_69 a owl:NamedIndividual, observable:Jacobian .
"#;
