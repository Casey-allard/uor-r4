/// SHACL test 49: Reversible resolution — FreeRank + SiteIndex +
/// ancillaSite + reversibleStrategy (Amendment 31, RC_1–RC_4).
pub const TEST49_REVERSIBLE_RESOLUTION: &str = r#"
@prefix rdf:        <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:        <http://www.w3.org/2002/07/owl#> .
@prefix xsd:        <http://www.w3.org/2001/XMLSchema#> .
@prefix partition:  <https://uor.foundation/partition/> .

# 1. SiteIndex with ancilla pairing
partition:ex_fc_49 a owl:NamedIndividual, partition:SiteIndex ;
    partition:ancillaSite partition:ex_ancilla_49 .

# 2. Ancilla site coordinate
partition:ex_ancilla_49 a owl:NamedIndividual, partition:SiteIndex .

# 3. FreeRank with reversible strategy
partition:ex_fb_49 a owl:NamedIndividual, partition:FreeRank ;
    partition:siteCount "4"^^xsd:nonNegativeInteger ;
    partition:reversibleStrategy "true"^^xsd:boolean .
"#;
