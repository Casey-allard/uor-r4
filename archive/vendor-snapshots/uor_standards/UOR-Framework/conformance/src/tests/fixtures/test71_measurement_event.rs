/// SHACL test 71: Measurement event — MeasurementEvent with measurementEvent,
/// preCollapseEntropy, postCollapseLandauerCost, collapseStep (Amendment 36).
/// v0.2.2 Phase A: extended to include observable:LandauerBudget — the sealed
/// carrier for accumulated Landauer cost backed by the new ontology class.
pub const TEST71_MEASUREMENT_EVENT: &str = r#"
@prefix rdf:        <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:        <http://www.w3.org/2002/07/owl#> .
@prefix xsd:        <http://www.w3.org/2001/XMLSchema#> .
@prefix observable: <https://uor.foundation/observable/> .
@prefix trace:      <https://uor.foundation/trace/> .

# 1. MeasurementEvent observable
observable:ex_me_71 a owl:NamedIndividual, observable:MeasurementEvent ;
    observable:measurementEvent trace:ex_step_71 ;
    observable:preCollapseEntropy "3.2"^^xsd:decimal ;
    observable:postCollapseLandauerCost "0.8"^^xsd:decimal ;
    observable:collapseStep "5"^^xsd:integer .

# 2. Referenced step
trace:ex_step_71 a owl:NamedIndividual, trace:ComputationTrace .

# 3. v0.2.2 Phase A: LandauerBudget instance carrying accumulated Landauer
#    cost in nats. Backs the Rust enforcement::LandauerBudget newtype that
#    holds one of the two clocks of UorTime.
observable:ex_landauer_budget_71 a owl:NamedIndividual, observable:LandauerBudget ;
    observable:landauerNats "0.8"^^xsd:decimal .
"#;
