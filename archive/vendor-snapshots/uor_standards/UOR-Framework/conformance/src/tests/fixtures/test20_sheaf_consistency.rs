/// SHACL test 20: Sheaf and cohomological consistency checks.
pub const TEST20_SHEAF_CONSISTENCY: &str = r#"
@prefix rdf:        <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:        <http://www.w3.org/2002/07/owl#> .
@prefix xsd:        <http://www.w3.org/2001/XMLSchema#> .
@prefix cohomology: <https://uor.foundation/cohomology/> .
@prefix homology:   <https://uor.foundation/homology/> .

cohomology:F a cohomology:Sheaf ;
    cohomology:sheafOver homology:K ;
    cohomology:coefficientIn "Z" .

cohomology:stalk_v0 a cohomology:Stalk ;
    cohomology:hasStalks cohomology:F ;
    cohomology:stalkAt "v0" .

cohomology:stalk_v1 a cohomology:Stalk ;
    cohomology:hasStalks cohomology:F ;
    cohomology:stalkAt "v1" .

cohomology:sec_global a cohomology:Section ;
    cohomology:sheafOver cohomology:F .

cohomology:sec_U0 a cohomology:LocalSection ;
    cohomology:sheafOver cohomology:F .

cohomology:sec_U1 a cohomology:LocalSection ;
    cohomology:sheafOver cohomology:F .

cohomology:obstruction_01 a cohomology:GluingObstruction ;
    cohomology:obstructionClass "H^1(K, F)" .

cohomology:CC0 a cohomology:CochainGroup ;
    cohomology:cochainDegree "0"^^xsd:integer .

cohomology:CC1 a cohomology:CochainGroup ;
    cohomology:cochainDegree "1"^^xsd:integer .

cohomology:CC2 a cohomology:CochainGroup ;
    cohomology:cochainDegree "2"^^xsd:integer .

cohomology:delta0 a cohomology:CoboundaryOperator ;
    cohomology:coboundarySource cohomology:CC0 ;
    cohomology:coboundaryTarget cohomology:CC1 .

cohomology:delta1 a cohomology:CoboundaryOperator ;
    cohomology:coboundarySource cohomology:CC1 ;
    cohomology:coboundaryTarget cohomology:CC2 .

cohomology:cochain_K a cohomology:CochainComplex ;
    cohomology:hasCochainGroup cohomology:CC0 , cohomology:CC1 , cohomology:CC2 ;
    cohomology:hasCoboundary cohomology:delta0 , cohomology:delta1 .

cohomology:HH0 a cohomology:CohomologyGroup ;
    cohomology:cohomologyDegree "0"^^xsd:integer .

cohomology:HH1 a cohomology:CohomologyGroup ;
    cohomology:cohomologyDegree "1"^^xsd:integer .
"#;
