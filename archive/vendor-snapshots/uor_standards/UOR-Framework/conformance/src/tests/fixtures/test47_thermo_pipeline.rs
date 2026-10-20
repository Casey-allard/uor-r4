/// SHACL test 47: Thermodynamic pipeline — ComputationTrace → ResidualEntropy +
/// hardnessEstimate + TH_1/TH_5/TH_9 grounding (Amendment 31).
pub const TEST47_THERMO_PIPELINE: &str = r#"
@prefix rdf:        <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:        <http://www.w3.org/2002/07/owl#> .
@prefix xsd:        <http://www.w3.org/2001/XMLSchema#> .
@prefix observable: <https://uor.foundation/observable/> .
@prefix trace:      <https://uor.foundation/trace/> .

# 1. ThermoObservable with hardnessEstimate (TH_9)
observable:ex_thermo_47 a owl:NamedIndividual, observable:ThermoObservable ;
    observable:value "3.14"^^xsd:decimal ;
    observable:hardnessEstimate "2.71"^^xsd:decimal .

# 2. ResidualEntropy (subclass of ThermoObservable) — TH_5
observable:ex_residual_47 a owl:NamedIndividual, observable:ResidualEntropy ;
    observable:value "0.693"^^xsd:decimal .

# 3. ComputationTrace with residualEntropy link — TH_1/TH_9 connection
trace:ex_trace_47 a owl:NamedIndividual, trace:ComputationTrace ;
    trace:residualEntropy observable:ex_residual_47 .
"#;
