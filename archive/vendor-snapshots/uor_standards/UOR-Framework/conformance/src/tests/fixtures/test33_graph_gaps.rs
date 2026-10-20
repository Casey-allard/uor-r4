/// SHACL test 33: Graph gap closure — Gap B, D, E, G individuals and properties.
pub const TEST33_GRAPH_GAPS: &str = r#"
@prefix rdf:      <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:      <http://www.w3.org/2002/07/owl#> .
@prefix xsd:      <http://www.w3.org/2001/XMLSchema#> .
@prefix resolver: <https://uor.foundation/resolver/> .
@prefix cohomology:<https://uor.foundation/cohomology/> .
@prefix type:     <https://uor.foundation/type/> .
@prefix cert:     <https://uor.foundation/cert/> .
@prefix observable:<https://uor.foundation/observable/> .
@prefix morphism: <https://uor.foundation/morphism/> .
@prefix u:        <https://uor.foundation/u/> .

# Gap B: nerveEulerCharacteristic on ResolutionState
resolver:ex_resolutionState a owl:NamedIndividual, resolver:ResolutionState ;
    resolver:isComplete                "false"^^xsd:boolean ;
    resolver:iterationCount            "2"^^xsd:nonNegativeInteger ;
    resolver:nerveEulerCharacteristic  "8"^^xsd:integer .

# Gap B: GluingObstruction with addressesSuggestion
cohomology:ex_gluingObstruction a owl:NamedIndividual, cohomology:GluingObstruction ;
    cohomology:addressesSuggestion resolver:ex_refinementSuggestion .

# Gap D: CompleteType and CompletenessCertificate
type:ex_completeType a owl:NamedIndividual, type:CompleteType .

cert:ex_completenessCert a owl:NamedIndividual, cert:CompletenessCertificate ;
    cert:certifiedType type:ex_completeType ;
    cert:verified      "true"^^xsd:boolean .

# Gap E: ThermoObservable subclasses and Nats unit
observable:ex_thermoObs a owl:NamedIndividual, observable:ThermoObservable ;
    observable:value "1.386"^^xsd:decimal .

observable:ex_residualEntropy a owl:NamedIndividual, observable:ResidualEntropy ;
    observable:value    "0.693"^^xsd:decimal ;
    observable:hasUnit  observable:Nats .

# Gap G: GroundingCertificate
morphism:ex_groundingCert a owl:NamedIndividual, morphism:GroundingCertificate ;
    morphism:groundingCertMap     morphism:ex_groundingMap ;
    morphism:groundingCertAddress u:addr_42 .
"#;
