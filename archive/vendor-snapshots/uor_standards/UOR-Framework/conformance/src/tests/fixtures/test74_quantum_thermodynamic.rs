/// SHACL test 74: Quantum thermodynamic domain — QuantumThermodynamicDomain
/// class instance with verification domain reference (Amendment 36).
pub const TEST74_QUANTUM_THERMODYNAMIC: &str = r#"
@prefix rdf:        <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:        <http://www.w3.org/2002/07/owl#> .
@prefix xsd:        <http://www.w3.org/2001/XMLSchema#> .
@prefix observable: <https://uor.foundation/observable/> .
@prefix op:         <https://uor.foundation/op/> .

# 1. QuantumThermodynamicDomain
observable:ex_qtd_74 a owl:NamedIndividual, observable:QuantumThermodynamicDomain .

# 2. Verification domain reference
op:ex_id_74 a owl:NamedIndividual, op:Identity ;
    op:verificationDomain op:Thermodynamic .
"#;
