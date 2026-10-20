//! SHACL test 179: End-to-end moduli pipeline.

/// Instance graph for Test 179: ModuliSpace -> DeformationComplex -> HolonomyStratum -> ModuliTowerMap.
pub const TEST179_MODULI_END_TO_END: &str = r#"
@prefix rdf:      <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:      <http://www.w3.org/2002/07/owl#> .
@prefix xsd:      <http://www.w3.org/2001/XMLSchema#> .
@prefix type:     <https://uor.foundation/type/> .
@prefix homology: <https://uor.foundation/homology/> .
@prefix schema:   <https://uor.foundation/schema/> .

type:ex_moduli_179 a owl:NamedIndividual, type:ModuliSpace ;
    type:moduliWittLevel schema:Q0 ;
    type:moduliDimension    "4"^^xsd:nonNegativeInteger .

homology:ex_def_179 a owl:NamedIndividual, homology:DeformationComplex ;
    homology:tangentDimension     "4"^^xsd:nonNegativeInteger ;
    homology:obstructionDimension "0"^^xsd:nonNegativeInteger .

type:ex_stratum_179 a owl:NamedIndividual, type:HolonomyStratum ;
    type:stratumCodimension "0"^^xsd:nonNegativeInteger ;
    type:stratumModuli      type:ex_moduli_179 .

type:ex_tower_179 a owl:NamedIndividual, type:ModuliTowerMap ;
    type:towerMapSource type:ex_moduli_179 .
"#;
