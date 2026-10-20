/// SHACL fixture for observable:AxisProjectionObservable (wiki ADR-038).
///
/// AxisProjectionObservable is the closed-catalog extension carrying
/// axis-realized projection values from the substrate-extension surface
/// per ADR-030. The fixture provides one named individual asserted to
/// the class so the SHACL coverage validator can confirm the class is
/// represented in instance test data.
pub const TEST288_AXIS_PROJECTION_OBSERVABLE: &str = r#"
@prefix rdf:        <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:        <http://www.w3.org/2002/07/owl#> .
@prefix xsd:        <http://www.w3.org/2001/XMLSchema#> .
@prefix observable: <https://uor.foundation/observable/> .

<urn:test:axis_projection_obs_1> a owl:NamedIndividual , observable:AxisProjectionObservable ;
    observable:value "1"^^xsd:decimal .
"#;
