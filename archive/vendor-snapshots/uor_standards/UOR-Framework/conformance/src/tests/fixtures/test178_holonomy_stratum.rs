//! SHACL test 178: `type:HolonomyStratum` -- flat and twisted strata.

/// Instance graph for Test 178: HolonomyStratum.
pub const TEST178_HOLONOMY_STRATUM: &str = r#"
@prefix rdf:        <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:        <http://www.w3.org/2002/07/owl#> .
@prefix xsd:        <http://www.w3.org/2001/XMLSchema#> .
@prefix type:       <https://uor.foundation/type/> .
@prefix observable: <https://uor.foundation/observable/> .

type:ex_flat_178 a owl:NamedIndividual, type:HolonomyStratum ;
    type:stratumCodimension    "0"^^xsd:nonNegativeInteger ;
    type:stratumHolonomyClass  observable:ex_mc_178 ;
    type:stratumModuli         type:ex_moduli_178 .

type:ex_twisted_178 a owl:NamedIndividual, type:HolonomyStratum ;
    type:stratumCodimension    "2"^^xsd:nonNegativeInteger ;
    type:stratumHolonomyClass  observable:ex_mc2_178 ;
    type:stratumModuli         type:ex_moduli_178 .

type:ex_moduli_178 a owl:NamedIndividual, type:ModuliSpace .

observable:ex_mc_178 a owl:NamedIndividual, observable:MonodromyClass .
observable:ex_mc2_178 a owl:NamedIndividual, observable:MonodromyClass .
"#;
