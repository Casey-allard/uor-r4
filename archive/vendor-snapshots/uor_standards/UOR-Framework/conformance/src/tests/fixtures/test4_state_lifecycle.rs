//! Test 4: State lifecycle — Context → Binding → Frame → Transition.
//!
//! Validates: state namespace classes with temporal sequencing and trace link.

/// Instance graph for Test 4: State lifecycle.
pub const TEST4_STATE_LIFECYCLE: &str = r#"
@prefix rdf:    <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:    <http://www.w3.org/2002/07/owl#> .
@prefix xsd:    <http://www.w3.org/2001/XMLSchema#> .
@prefix state:  <https://uor.foundation/state/> .
@prefix trace:  <https://uor.foundation/trace/> .
@prefix type:   <https://uor.foundation/type/> .

# Context: an evaluation environment
<https://uor.foundation/instance/ctx-1>
    a               owl:NamedIndividual, state:Context ;
    state:wittLength   "8"^^xsd:nonNegativeInteger ;
    state:capacity  "256"^^xsd:nonNegativeInteger .

# Binding: a name bound to a typed value in the context
<https://uor.foundation/instance/binding-x>
    a               owl:NamedIndividual, state:Binding ;
    state:address   "0x00"^^xsd:string ;
    state:content   "42"^^xsd:integer ;
    state:boundType <https://uor.foundation/instance/type-u8> ;
    state:timestamp "0"^^xsd:nonNegativeInteger .

<https://uor.foundation/instance/type-u8>
    a               owl:NamedIndividual, type:PrimitiveType ;
    type:bitWidth   "8"^^xsd:nonNegativeInteger .

# Frame: a snapshot of the context with active bindings
<https://uor.foundation/instance/frame-0>
    a                       owl:NamedIndividual, state:Frame ;
    state:context           <https://uor.foundation/instance/ctx-1> ;
    state:activeBindings    "1"^^xsd:nonNegativeInteger .

# Transition: from frame-0 to frame-1, adding a binding
<https://uor.foundation/instance/frame-1>
    a                       owl:NamedIndividual, state:Frame ;
    state:context           <https://uor.foundation/instance/ctx-1> ;
    state:activeBindings    "2"^^xsd:nonNegativeInteger .

<https://uor.foundation/instance/transition-0-1>
    a                       owl:NamedIndividual, state:Transition ;
    state:from              <https://uor.foundation/instance/frame-0> ;
    state:to                <https://uor.foundation/instance/frame-1> ;
    state:addedBindings     "1"^^xsd:nonNegativeInteger ;
    state:removedBindings   "0"^^xsd:nonNegativeInteger ;
    state:trace             <https://uor.foundation/instance/trace-1> .

# Trace: the computation trace associated with this transition
<https://uor.foundation/instance/trace-1>
    a               owl:NamedIndividual, trace:ComputationTrace .
"#;
