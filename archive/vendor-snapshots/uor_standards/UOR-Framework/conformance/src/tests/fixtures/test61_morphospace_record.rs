/// SHACL test 61: Morphospace record — MorphospaceRecord with
/// achievabilityStatus, verifiedAtLevel, morphospaceRecord (Amendment 34).
pub const TEST61_MORPHOSPACE_RECORD: &str = r#"
@prefix rdf:        <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:        <http://www.w3.org/2002/07/owl#> .
@prefix xsd:        <http://www.w3.org/2001/XMLSchema#> .
@prefix observable: <https://uor.foundation/observable/> .
@prefix schema:     <https://uor.foundation/schema/> .

# 1. MorphospaceRecord with status and level
observable:ex_mr_61 a owl:NamedIndividual, observable:MorphospaceRecord ;
    observable:achievabilityStatus observable:Achievable ;
    observable:verifiedAtLevel schema:Q1 ;
    observable:morphospaceRecord observable:ex_mr_61 .
"#;
