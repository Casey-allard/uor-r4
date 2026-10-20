//! SHACL test 271: schema:SurfaceSymbol and host value types.

/// Instance graph for Test 271: SurfaceSymbol, HostValue,
/// HostStringLiteral, and HostBooleanLiteral instances.
pub const TEST271_HOST_VALUE_TYPES: &str = r#"
@prefix rdf:    <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:    <http://www.w3.org/2002/07/owl#> .
@prefix schema: <https://uor.foundation/schema/> .

schema:ex_surface_symbol_271 a owl:NamedIndividual, schema:SurfaceSymbol .
schema:ex_host_value_271     a owl:NamedIndividual, schema:HostValue .
schema:ex_host_string_271    a owl:NamedIndividual, schema:HostStringLiteral .
schema:ex_host_boolean_271   a owl:NamedIndividual, schema:HostBooleanLiteral .
"#;
