# State Model

## Definition

The UOR state model captures evaluation context — the bindings, frames, and
transitions that comprise a computation. The `state/` namespace provides four
mutually disjoint classes.

## Classes

| Class | Description |
|-------|-------------|
| {@class https://uor.foundation/state/Context} | Evaluation environment (Witt level, capacity) |
| {@class https://uor.foundation/state/Binding} | A name bound to a typed value |
| {@class https://uor.foundation/state/Frame} | Snapshot of context with active bindings |
| {@class https://uor.foundation/state/Transition} | Change from one frame to another |

These four classes are mutually `owl:disjointWith` — a `Context` is never a
`Binding`, a `Frame`, or a `Transition`.

## Context

A {@class https://uor.foundation/state/Context} is the runtime environment:

| Property | Description |
|----------|-------------|
| {@prop https://uor.foundation/state/wittLength} | Witt level n of the ring |
| {@prop https://uor.foundation/state/capacity} | Maximum number of bindings |
| {@prop https://uor.foundation/state/contentAddress} | Content address of the context |

## Binding

A {@class https://uor.foundation/state/Binding} associates an address with a value:

| Property | Description |
|----------|-------------|
| {@prop https://uor.foundation/state/address} | Address within the context |
| {@prop https://uor.foundation/state/content} | Bound value |
| {@prop https://uor.foundation/state/boundType} | Type of the bound value |
| {@prop https://uor.foundation/state/timestamp} | Step count when binding was created |

## Frame

A {@class https://uor.foundation/state/Frame} is an immutable snapshot:

| Property | Description |
|----------|-------------|
| {@prop https://uor.foundation/state/context} | The containing context (functional) |
| {@prop https://uor.foundation/state/activeBindings} | Count of active bindings |
| {@prop https://uor.foundation/state/constraint} | Optional constraint on the frame |

## Transition

A {@class https://uor.foundation/state/Transition} records a state change:

| Property | Description |
|----------|-------------|
| {@prop https://uor.foundation/state/from} | Source frame (functional) |
| {@prop https://uor.foundation/state/to} | Target frame (functional) |
| {@prop https://uor.foundation/state/addedBindings} | Count of bindings added |
| {@prop https://uor.foundation/state/removedBindings} | Count of bindings removed |
| {@prop https://uor.foundation/state/trace} | Computation trace for this transition |

## Example Lifecycle

```
Context (quantum=8) → Frame-0 (0 bindings) → Transition → Frame-1 (1 binding)
                                                ↓
                                          ComputationTrace
```

### Turtle Example

```turtle
# Context: 8-bit ring with capacity for 4 bindings
<https://uor.foundation/instance/ctx-r8>
    a                   state:Context ;
    state:quantum       "8"^^xsd:nonNegativeInteger ;
    state:capacity      "4"^^xsd:nonNegativeInteger .

# Binding: address 0 holds value 42 of type u8
<https://uor.foundation/instance/binding-0>
    a                   state:Binding ;
    state:address       "0"^^xsd:nonNegativeInteger ;
    state:content       "42"^^xsd:nonNegativeInteger ;
    state:boundType     <https://uor.foundation/instance/type-u8> ;
    state:timestamp     "1"^^xsd:nonNegativeInteger .

# Frame: snapshot with one active binding
<https://uor.foundation/instance/frame-1>
    a                   state:Frame ;
    state:context       <https://uor.foundation/instance/ctx-r8> ;
    state:activeBindings "1"^^xsd:nonNegativeInteger .

# Transition: from empty frame to frame with one binding
<https://uor.foundation/instance/transition-0-1>
    a                       state:Transition ;
    state:from              <https://uor.foundation/instance/frame-0> ;
    state:to                <https://uor.foundation/instance/frame-1> ;
    state:addedBindings     "1"^^xsd:nonNegativeInteger ;
    state:removedBindings   "0"^^xsd:nonNegativeInteger .
```

This lifecycle is validated by SHACL test `test4_state_lifecycle`. See
[Evaluation](evaluation.html) for how evaluation triggers state transitions.
