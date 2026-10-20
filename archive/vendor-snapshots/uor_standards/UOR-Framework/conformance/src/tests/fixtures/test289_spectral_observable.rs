/// SHACL fixture for observable:SpectralObservable (wiki ADR-049).
///
/// SpectralObservable is the closed-catalog extension hosting
/// Walsh–Hadamard-parity-derived spectral readings of the σ-projection's
/// frequency-domain spectrum per ADR-049. The fixture provides one named
/// individual asserted to the class so the SHACL coverage validator
/// confirms the class is represented in instance test data.
pub const TEST289_SPECTRAL_OBSERVABLE: &str = r#"
@prefix rdf:        <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:        <http://www.w3.org/2002/07/owl#> .
@prefix xsd:        <http://www.w3.org/2001/XMLSchema#> .
@prefix observable: <https://uor.foundation/observable/> .

<urn:test:spectral_obs_1> a owl:NamedIndividual , observable:SpectralObservable ;
    observable:value "1"^^xsd:decimal .
"#;
