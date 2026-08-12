
# MUDBench System Architecture

## Document Status
Version: 0.1  
Status: Draft  
Purpose: Define the core system architecture for MUDBench, including the major subsystems, data flow, and execution model.

---

# 1. Architecture Overview

MUDBench is composed of several interacting subsystems.

The system must support:

- persistent world simulation
- multi-agent interaction
- constrained action pipelines
- reproducible benchmark runs
- replay logging and telemetry
- modular development for coding agents

The architecture must remain modular so subsystems can evolve independently.

---

# 2. Core Subsystems

MUDBench consists of five primary subsystems:

1. Agent Gateway  
2. Simulation Controller  
3. World Engine  
4. Evaluation Engine  
5. Replay and Telemetry System  

High level structure:

Agents  
↓  
Agent Gateway  
↓  
Simulation Controller  
↓  
World Engine  
↓  
Evaluation Engine  
↓  
Replay / Telemetry

---

# 3. Agent Gateway

The Agent Gateway is responsible for communication between agents and the simulation.

Responsibilities:

- receive agent actions
- validate actions
- enforce action constraints
- deliver observations
- manage timeouts

The gateway must isolate the simulation from malformed agent behavior.

Functions:

- action validation
- agent authentication (future)
- rate limiting
- error handling

---

# 4. Simulation Controller

The Simulation Controller orchestrates the run.

Responsibilities:

- initialize world state
- step the simulation loop
- collect agent actions
- dispatch actions to the world engine
- manage turn order or event order
- terminate runs when conditions are met

Simulation steps follow the loop:

collect actions  
→ validate actions  
→ update world state  
→ generate observations  
→ log events  
→ update scores

---

# 5. World Engine

The World Engine is the core simulation.

Responsibilities:

- maintain world state
- execute entity behaviors
- resolve interactions
- update environment state

The engine must support:

- rooms
- items
- NPCs
- agents
- quests
- combat

The world state must be deterministic when seeded.

---

# 6. Entity Model

All interactive objects in the world are entities.

Entity types include:

- rooms
- items
- NPCs
- agents

Entities share a common base structure.

Example entity model:

Entity
- id
- type
- location
- attributes
- components

Components may include:

- inventory
- health
- AI behavior
- quest state

---

# 7. Action Processing

Action handling pipeline:

1. Agent submits action
2. Gateway validates action
3. Controller schedules action
4. World Engine resolves action
5. Resulting events generated
6. Observations returned

Example action:

move(direction="north")

---

# 8. Event System

The world state evolves through events.

Examples:

- movement
- item pickup
- combat attack
- NPC dialogue
- quest update

Events produce:

- state changes
- observation updates
- replay log entries

---

# 9. Evaluation Engine

The Evaluation Engine calculates benchmark metrics.

Responsibilities:

- track run statistics
- compute capability scores
- aggregate performance metrics
- produce run summaries

Metrics include:

- exploration coverage
- quest completion
- combat performance
- survival time
- efficiency metrics

Evaluation runs continuously during simulation.

---

# 10. Replay and Telemetry

All runs must produce deterministic replay logs.

Replay logs include:

- observation stream
- action stream
- state changes
- run metadata

Replay logs allow:

- debugging
- research analysis
- public viewing

The replay system reconstructs runs from event logs.

---

# 11. Data Storage

MUDBench requires persistent storage for:

- replay logs
- benchmark results
- arena history
- agent identifiers

Recommended storage:

- structured database (PostgreSQL)
- log storage for event streams

---

# 12. Determinism Requirements

Official benchmark runs must be reproducible.

Requirements:

- deterministic world updates
- fixed random seeds
- ordered action resolution
- stable scoring algorithms

Non-deterministic behavior is allowed only in arena mode.

---

# 13. Error Handling

The system must handle the following gracefully:

- invalid agent actions
- agent timeouts
- malformed responses
- gateway failures

Agents that repeatedly violate constraints may be removed from a run.

---

# 14. Performance Considerations

The architecture should support:

- multiple concurrent simulations
- scalable agent counts
- bounded world update cost
- efficient event logging

The MVP does not need massive scale but should avoid architectural bottlenecks.

---

# 15. Modularity Requirements

Subsystems must remain loosely coupled.

World Engine must not depend directly on:

- evaluation logic
- replay logic
- agent communication logic

This separation allows safe extensions.

---

# 16. Architecture Evolution

Future versions may introduce:

- distributed simulation
- containerized agent execution
- advanced economy simulation
- procedural world generation
- tournament orchestration

The initial architecture must not prevent these extensions.

---

# 17. Immediate Next Documents

The following documents refine specific parts of the architecture:

1. WORLD_SPEC.md
2. AGENT_INTERFACE_SPEC.md
3. BENCHMARK_SCORING_SPEC.md
4. REPLAY_AND_TELEMETRY_SPEC.md

These documents define the implementation details for each subsystem.

---

# 18. Closing Statement

The architecture should remain simple, deterministic, and modular.

The simulation world is a tool for evaluation.

The clarity of the system design is more important than the richness of the world itself.
