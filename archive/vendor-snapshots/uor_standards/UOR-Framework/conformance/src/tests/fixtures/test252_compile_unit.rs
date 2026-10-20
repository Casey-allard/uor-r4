//! SHACL test 252: `reduction:CompileUnit` with admission properties.

/// Instance graph for Test 252: CompileUnit with required properties.
pub const TEST252_COMPILE_UNIT: &str = r#"
@prefix rdf:         <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:         <http://www.w3.org/2002/07/owl#> .
@prefix xsd:         <http://www.w3.org/2001/XMLSchema#> .
@prefix schema:      <https://uor.foundation/schema/> .
@prefix op:          <https://uor.foundation/op/> .
@prefix u:           <https://uor.foundation/u/> .
@prefix reduction:     <https://uor.foundation/reduction/> .

reduction:ex_compile_unit_252 a owl:NamedIndividual, reduction:CompileUnit ;
    reduction:rootTerm         reduction:ex_root_term_252 ;
    reduction:unitWittLevel schema:Q0 ;
    reduction:targetDomains    op:Algebraic ;
    reduction:targetDomains    op:Pipeline ;
    reduction:thermodynamicBudget "6.0"^^xsd:decimal ;
    reduction:unitAddress      reduction:ex_address_252 .

reduction:ex_root_term_252 a owl:NamedIndividual, schema:Term .
reduction:ex_address_252   a owl:NamedIndividual, u:Element .
"#;
