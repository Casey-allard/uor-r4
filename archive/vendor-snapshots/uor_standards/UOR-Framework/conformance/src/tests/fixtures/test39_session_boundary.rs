/// SHACL test 39: SessionBoundary and SessionBoundaryType vocabulary — Amendment 27.
pub const TEST39_SESSION_BOUNDARY: &str = r#"
@prefix rdf:   <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:   <http://www.w3.org/2002/07/owl#> .
@prefix xsd:   <http://www.w3.org/2001/XMLSchema#> .
@prefix state: <https://uor.foundation/state/> .

state:ex_boundary_39 a owl:NamedIndividual, state:SessionBoundary ;
    state:boundaryType    state:ContradictionBoundary ;
    state:boundaryReason  "Type contradiction detected at address 0x42" ;
    state:priorContext    state:ex_prior_ctx_39 ;
    state:freshContext    state:ex_fresh_ctx_39 .

state:ex_prior_ctx_39 a owl:NamedIndividual, state:Context .
state:ex_fresh_ctx_39 a owl:NamedIndividual, state:Context .
"#;
