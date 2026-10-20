/// SHACL test 281: impossibility certificates — the failure-path cert
/// carriers for resolver `certify` functions per target §4.2
/// `Certified<ImpossibilityWitness>`. Workstream C (v0.2.2 closure).
pub const TEST281_IMPOSSIBILITY_CERTIFICATES: &str = r#"
@prefix rdf:  <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:  <http://www.w3.org/2002/07/owl#> .
@prefix xsd:  <http://www.w3.org/2001/XMLSchema#> .
@prefix cert: <https://uor.foundation/cert/> .

# Generic impossibility: failure-path cert for any Phase D resolver.
cert:ex_generic_impossibility_281 a owl:NamedIndividual, cert:GenericImpossibilityCertificate .

# Inhabitance impossibility: failure-path cert for resolver::inhabitance::certify.
cert:ex_inhabitance_impossibility_281 a owl:NamedIndividual, cert:InhabitanceImpossibilityCertificate .
"#;
