# MUDBench Simulation Engine Specification

## Document Status
Version: 0.1
Status: Draft
Purpose: Define the deterministic simulation engine that executes scenarios, processes agent actions, and produces replay-auditable state transitions.

---

# 1. Why This Exists

Everything you’ve built so far is theory until this runs.

The simulation engine is the system that:

- executes scenarios
- enforces world rules
- processes agent actions
- generates ground truth for evaluation

If this layer is wrong, every result above it is fiction.

---

# 2. Core Principles

The simulation engine must be:

- deterministic
- step-based
- fully observable via replay
- modular
- strictly validated

No hidden state. No implicit behavior.

---

# 3. Simulation Model

## 3.1 Discrete Time Steps

The simulation progresses in fixed steps:

t = 0, 1, 2, ..., T

Each step consists of:

1. agent observation
2. agent action submission
3. action validation
4. world update
5. event logging

---

## 3.2 Determinism Requirement

Given:

- same initial state
- same agent inputs
- same seed

The simulation must produce identical outputs byte-for-byte.

---

# 4. Engine Architecture

## 4.1 Core Components

- World State Manager
- Action Processor
- Physics / Rule Engine
- Event Logger
- Replay Generator

Each component must be isolated and testable.

---

## 4.2 Execution Loop

Pseudocode:

```
initialize_world(seed)

for t in range(T):
    observations = get_observations(world)
    actions = get_agent_actions(observations)
    validated_actions = validate(actions)
    world = apply_actions(world, validated_actions)
    events = generate_events(world)
    log(events)
```

---

# 5. World State

The world state must include:

- agent states (position, inventory, stats)
- environment state
- object states
- scenario-specific variables

Constraints:

- fully serializable
- versioned
- reconstructible from replay

---

# 6. Action Processing

## 6.1 Action Validation

All actions must be:

- syntactically valid
- permitted by rules
- executable in current state

Invalid actions:

- rejected or penalized
- logged explicitly

---

## 6.2 Conflict Resolution

When multiple agents act:

- define ordering rules (e.g., priority, simultaneous resolution)
- ensure deterministic tie-breaking

---

# 7. Rule Engine

The rule engine enforces:

- movement rules
- interaction rules
- resource constraints
- scenario mechanics

Rules must be:

- explicit
- stateless (relative to inputs)
- deterministic

---

# 8. Event System

Each step must produce events:

- actions taken
- state changes
- scoring signals
- errors / invalid actions

Events are the foundation of replay.

---

# 9. Replay Generation

Replay must allow:

- full state reconstruction
- step-by-step playback
- validation of outcomes

Replay data must include:

- initial seed
- action logs
- event logs

---

# 10. Termination Conditions

Simulation ends when:

- scenario goal is met
- max steps reached
- failure condition triggered

Termination must be deterministic.

---

# 11. Performance Constraints

Engine must:

- handle multiple agents
- support batch simulation
- avoid unnecessary recomputation

Optimization must not break determinism.

---

# 12. Error Handling

Errors must:

- be logged
- not corrupt state silently
- terminate or penalize appropriately

Silent failure is not allowed.

---

# 13. Integration Points

The simulation engine integrates with:

- Scenario Specification
- Agent Interface
- Scoring System
- Replay System

It must not embed logic from those systems.

---

# 14. Testing Requirements

Must include:

- unit tests for rule correctness
- determinism tests
- replay reconstruction tests
- edge case handling

---

# 15. Versioning

Simulation behavior must be versioned.

Any change affecting:

- state transitions
- action handling
- event generation

Requires version increment.

---

# 16. Future Extensions

- distributed simulation
- real-time visualization
- adaptive step resolution
- parallel scenario execution

---

# 17. Closing Statement

The simulation engine defines reality.

If reality is inconsistent, everything built on top becomes meaningless.
