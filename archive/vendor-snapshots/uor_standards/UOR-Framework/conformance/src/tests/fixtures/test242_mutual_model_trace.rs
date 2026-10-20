//! SHACL test 242: `interaction:MutualModelTrace`.

/// Instance graph for Test 242: MutualModelTrace.
pub const TEST242_MUTUAL_MODEL_TRACE: &str = r#"
@prefix rdf:         <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:         <http://www.w3.org/2002/07/owl#> .
@prefix xsd:         <http://www.w3.org/2001/XMLSchema#> .
@prefix interaction: <https://uor.foundation/interaction/> .

interaction:ex_mutual_242 a owl:NamedIndividual, interaction:MutualModelTrace ;
    interaction:modelConvergent "true"^^xsd:boolean .
"#;
