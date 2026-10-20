//! SHACL test 181: `observable:StratificationRecord`.

/// Instance graph for Test 181: StratificationRecord.
pub const TEST181_STRATIFICATION_RECORD: &str = r#"
@prefix rdf:        <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:        <http://www.w3.org/2002/07/owl#> .
@prefix schema:     <https://uor.foundation/schema/> .
@prefix observable: <https://uor.foundation/observable/> .
@prefix type:       <https://uor.foundation/type/> .

observable:ex_strat_181 a owl:NamedIndividual, observable:StratificationRecord ;
    observable:stratificationLevel  schema:Q0 ;
    observable:stratificationStratum type:ex_stratum_181 .

type:ex_stratum_181 a owl:NamedIndividual, type:HolonomyStratum .
"#;
