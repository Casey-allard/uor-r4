/// SHACL test 63: Forbidden signature — ForbiddenSignature with
/// targetForbidden (Amendment 34).
pub const TEST63_FORBIDDEN_SIGNATURE: &str = r#"
@prefix rdf:        <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:        <http://www.w3.org/2002/07/owl#> .
@prefix xsd:        <http://www.w3.org/2001/XMLSchema#> .
@prefix observable: <https://uor.foundation/observable/> .

# 1. ForbiddenSignature
observable:ex_fs_63 a owl:NamedIndividual, observable:ForbiddenSignature ;
    observable:targetForbidden observable:ex_sig_63 .

# 2. Referenced signature
observable:ex_sig_63 a owl:NamedIndividual, observable:SynthesisSignature .
"#;
