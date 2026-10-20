//! SHACL test 197: `type:ConstrainedType` structural instances (GraphType, TreeType).

/// Instance graph for Test 197: GraphType and TreeType structural type individuals.
pub const TEST197_GRAPH_TREE_TYPE: &str = r#"
@prefix rdf:  <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:  <http://www.w3.org/2002/07/owl#> .
@prefix xsd:  <http://www.w3.org/2001/XMLSchema#> .
@prefix type: <https://uor.foundation/type/> .

type:GraphType a owl:NamedIndividual, type:ConstrainedType ;
    type:structuralSiteCount "sum of node site counts + edge overhead" ;
    type:structuralGrounding "constraint nerve = graph nerve, beta_k equality" ;
    type:structuralConstraint "edge constraints: adjacency preserved under grounding" .

type:TreeType a owl:NamedIndividual, type:ConstrainedType ;
    type:structuralSiteCount "sum of node site counts" ;
    type:structuralGrounding "parent-child encoding with acyclicity constraint" ;
    type:structuralConstraint "beta_1=0 (acyclic), beta_0=1 (connected)" .
"#;
