//! Test 15: Boolean SAT case study.
//!
//! Validates: MetricQuery → EvaluationResolver → Partition → State
//! (Binding/Frame/Transition) → Certificate → Trace.

/// Instance graph for Test 15: Boolean SAT.
pub const TEST15_BOOLEAN_SAT: &str = r#"
@prefix rdf:        <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:        <http://www.w3.org/2002/07/owl#> .
@prefix xsd:        <http://www.w3.org/2001/XMLSchema#> .
@prefix query:      <https://uor.foundation/query/> .
@prefix resolver:   <https://uor.foundation/resolver/> .
@prefix partition:  <https://uor.foundation/partition/> .
@prefix state:      <https://uor.foundation/state/> .
@prefix cert:       <https://uor.foundation/cert/> .
@prefix trace:      <https://uor.foundation/trace/> .

# 1. Query — metric query for SAT evaluation
<https://uor.foundation/instance/sat/query>
    a               owl:NamedIndividual, query:MetricQuery .

# 2. Resolver — evaluation resolver
<https://uor.foundation/instance/sat/resolver>
    a                   owl:NamedIndividual, resolver:EvaluationResolver ;
    resolver:strategy   "boolean-evaluation" .

# 3. Partition
<https://uor.foundation/instance/sat/partition>
    a                       owl:NamedIndividual, partition:Partition ;
    partition:wittLength       "3"^^xsd:positiveInteger .

# 4. State — evaluation context with bindings and transitions
<https://uor.foundation/instance/sat/ctx>
    a               owl:NamedIndividual, state:Context ;
    state:wittLength   "3"^^xsd:nonNegativeInteger .

<https://uor.foundation/instance/sat/binding-x>
    a               owl:NamedIndividual, state:Binding ;
    state:variable  "x" ;
    state:value     "1"^^xsd:integer .

<https://uor.foundation/instance/sat/frame>
    a               owl:NamedIndividual, state:Frame ;
    state:binding   <https://uor.foundation/instance/sat/binding-x> .

<https://uor.foundation/instance/sat/transition>
    a               owl:NamedIndividual, state:Transition ;
    state:from      <https://uor.foundation/instance/sat/frame> ;
    state:to        <https://uor.foundation/instance/sat/frame-result> .

<https://uor.foundation/instance/sat/frame-result>
    a               owl:NamedIndividual, state:Frame .

# 5. Certificate
<https://uor.foundation/instance/sat/cert>
    a                   owl:NamedIndividual, cert:Certificate ;
    cert:certifies      <https://uor.foundation/instance/sat/partition> ;
    cert:verified       true .

# 6. Trace
<https://uor.foundation/instance/sat/trace>
    a                   owl:NamedIndividual, trace:ComputationTrace ;
    trace:certifiedBy   <https://uor.foundation/instance/sat/cert> .
"#;
