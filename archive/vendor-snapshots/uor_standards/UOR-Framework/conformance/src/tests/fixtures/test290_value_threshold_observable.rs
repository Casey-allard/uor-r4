/// SHACL fixture for observable:ValueThresholdObservable (wiki ADR-040 + ADR-049).
///
/// ValueThresholdObservable is the closed-catalog extension hosting
/// byte-sequence threshold comparison readings of digests, realizing the
/// type:LexicographicLessEqBound bound-shape primitive's dispatch path
/// per ADR-040. Foundation's typed observable `LexicographicLessEqThreshold`
/// per ADR-049 falls under this subclass; the canonical search-cost
/// commitment alias `TargetCommitment = SingletonCommitment<…>` per ADR-048
/// consumes it.
pub const TEST290_VALUE_THRESHOLD_OBSERVABLE: &str = r#"
@prefix rdf:        <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:        <http://www.w3.org/2002/07/owl#> .
@prefix xsd:        <http://www.w3.org/2001/XMLSchema#> .
@prefix observable: <https://uor.foundation/observable/> .

<urn:test:value_threshold_obs_1> a owl:NamedIndividual , observable:ValueThresholdObservable ;
    observable:value "1"^^xsd:decimal .
"#;
