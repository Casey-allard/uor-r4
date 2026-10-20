/// SHACL test 19: Homological algebra pipeline — simplicial complexes through homology.
pub const TEST19_HOMOLOGICAL_PIPELINE: &str = r#"
@prefix rdf:        <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:        <http://www.w3.org/2002/07/owl#> .
@prefix xsd:        <http://www.w3.org/2001/XMLSchema#> .
@prefix homology:   <https://uor.foundation/homology/> .
@prefix type:       <https://uor.foundation/type/> .
@prefix partition:  <https://uor.foundation/partition/> .
@prefix observable: <https://uor.foundation/observable/> .

homology:s0 a homology:Simplex ;
    homology:dimension "0"^^xsd:integer ;
    homology:vertex "v0" .

homology:s1 a homology:Simplex ;
    homology:dimension "0"^^xsd:integer ;
    homology:vertex "v1" .

homology:s2 a homology:Simplex ;
    homology:dimension "0"^^xsd:integer ;
    homology:vertex "v2" .

homology:e01 a homology:Simplex ;
    homology:dimension "1"^^xsd:integer ;
    homology:vertex "v0" , "v1" .

homology:e12 a homology:Simplex ;
    homology:dimension "1"^^xsd:integer ;
    homology:vertex "v1" , "v2" .

homology:e02 a homology:Simplex ;
    homology:dimension "1"^^xsd:integer ;
    homology:vertex "v0" , "v2" .

homology:tri012 a homology:Simplex ;
    homology:dimension "2"^^xsd:integer ;
    homology:vertex "v0" , "v1" , "v2" .

homology:K a homology:SimplicialComplex ;
    homology:hasSimplex homology:s0 , homology:s1 , homology:s2 ,
        homology:e01 , homology:e12 , homology:e02 , homology:tri012 .

homology:C0 a homology:ChainGroup ;
    homology:degree "0"^^xsd:integer .

homology:C1 a homology:ChainGroup ;
    homology:degree "1"^^xsd:integer .

homology:C2 a homology:ChainGroup ;
    homology:degree "2"^^xsd:integer .

homology:d1 a homology:BoundaryOperator ;
    homology:sourceGroup homology:C1 ;
    homology:targetGroup homology:C0 .

homology:d2 a homology:BoundaryOperator ;
    homology:sourceGroup homology:C2 ;
    homology:targetGroup homology:C1 .

homology:chain_K a homology:ChainComplex ;
    homology:hasChainGroup homology:C0 , homology:C1 , homology:C2 ;
    homology:hasBoundary homology:d1 , homology:d2 .

homology:H0 a homology:HomologyGroup ;
    homology:homologyDegree "0"^^xsd:integer ;
    homology:bettiNumber "1"^^xsd:integer .

homology:H1 a homology:HomologyGroup ;
    homology:homologyDegree "1"^^xsd:integer ;
    homology:bettiNumber "0"^^xsd:integer .

homology:H2 a homology:HomologyGroup ;
    homology:homologyDegree "2"^^xsd:integer ;
    homology:bettiNumber "0"^^xsd:integer .
"#;
