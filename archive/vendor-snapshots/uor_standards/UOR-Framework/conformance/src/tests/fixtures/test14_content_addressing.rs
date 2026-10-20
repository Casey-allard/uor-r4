//! Test 14: Content addressing case study.
//!
//! Validates: Element → Observable taxonomy → InvolutionCertificate.

/// Instance graph for Test 14: Content addressing.
pub const TEST14_CONTENT_ADDRESSING: &str = r#"
@prefix rdf:        <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:        <http://www.w3.org/2002/07/owl#> .
@prefix xsd:        <http://www.w3.org/2001/XMLSchema#> .
@prefix u:          <https://uor.foundation/u/> .
@prefix schema:     <https://uor.foundation/schema/> .
@prefix observable: <https://uor.foundation/observable/> .
@prefix cert:       <https://uor.foundation/cert/> .
@prefix op:         <https://uor.foundation/op/> .

# A content-addressed datum
<https://uor.foundation/instance/ca/datum>
    a               owl:NamedIndividual, schema:Datum ;
    schema:value    "48656C6C6F" .

# Its content element
<https://uor.foundation/instance/ca/address>
    a               owl:NamedIndividual, u:Element ;
    u:digest        "sha256:2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824" ;
    u:addresses     <https://uor.foundation/instance/ca/datum> .

# Observable measurements across the tri-metric taxonomy
<https://uor.foundation/instance/ca/obs-ring>
    a                   owl:NamedIndividual, observable:RingMetric ;
    observable:value    "5"^^xsd:integer .

<https://uor.foundation/instance/ca/obs-hamming>
    a                   owl:NamedIndividual, observable:HammingMetric ;
    observable:value    "21"^^xsd:integer .

<https://uor.foundation/instance/ca/obs-curvature>
    a                   owl:NamedIndividual, observable:CurvatureObservable ;
    observable:value    "16"^^xsd:integer .

# Involution certificate for bnot on this value
<https://uor.foundation/instance/ca/cert>
    a                   owl:NamedIndividual, cert:InvolutionCertificate ;
    cert:operation      op:bnot ;
    cert:verified       true ;
    cert:certifies      <https://uor.foundation/instance/ca/datum> .
"#;
