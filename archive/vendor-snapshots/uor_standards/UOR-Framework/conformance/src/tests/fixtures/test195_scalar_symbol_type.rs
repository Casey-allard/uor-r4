//! SHACL test 195: `type:PrimitiveType` structural instances (ScalarType, SymbolType).

/// Instance graph for Test 195: ScalarType and SymbolType structural type individuals.
pub const TEST195_SCALAR_SYMBOL_TYPE: &str = r#"
@prefix rdf:  <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:  <http://www.w3.org/2002/07/owl#> .
@prefix xsd:  <http://www.w3.org/2001/XMLSchema#> .
@prefix type: <https://uor.foundation/type/> .

type:ScalarType a owl:NamedIndividual, type:PrimitiveType ;
    type:structuralSiteCount "n (quantization bits)" ;
    type:structuralGrounding "quantize(value, range, bits) produces ring element where d_R reflects value proximity" .

type:SymbolType a owl:NamedIndividual, type:PrimitiveType ;
    type:structuralSiteCount "ceil(log2(|alphabet|))" ;
    type:structuralGrounding "argmin_{encoding} sum d_delta over observed pairs (CY_5)" .
"#;
