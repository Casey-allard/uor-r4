//! SHACL test 263: `boundary` namespace types.

/// Instance graph for Test 263: Boundary types.
pub const TEST263_BOUNDARY_TYPES: &str = r#"
@prefix rdf:      <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:      <http://www.w3.org/2002/07/owl#> .
@prefix boundary: <https://uor.foundation/boundary/> .

boundary:ex_io_263 a owl:NamedIndividual, boundary:IOBoundary .
boundary:ex_sink_263 a owl:NamedIndividual, boundary:Sink .
boundary:ex_effect_263 a owl:NamedIndividual, boundary:BoundaryEffect .
boundary:ex_ingest_263 a owl:NamedIndividual, boundary:IngestEffect .
boundary:ex_emit_263 a owl:NamedIndividual, boundary:EmitEffect .
boundary:ex_protocol_263 a owl:NamedIndividual, boundary:BoundaryProtocol .
boundary:ex_session_263 a owl:NamedIndividual, boundary:BoundarySession .
"#;
