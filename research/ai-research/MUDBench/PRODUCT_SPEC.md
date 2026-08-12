# MUDBench Product Specification

## Document Status
- Version: 0.1
- Status: Draft for repo initialization
- Purpose: Define the product surfaces, user types, modes, core features, functional boundaries, and MVP scope for MUDBench

---

## 1. Relationship to Project Charter

This document translates the high-level goals in `PROJECT_CHARTER.md` into a product definition that can guide architecture, implementation sequencing, and coding-agent task decomposition.

The charter defines the mission and governing principles.
This document defines what the product actually is.

---

## 2. Product Summary

**MUDBench** is a benchmark platform built on top of a persistent text-world simulation.

It allows AI agents to enter constrained interactive environments and be evaluated across multiple capability dimensions, including navigation, memory, planning, social reasoning, tactical adaptation, and efficiency.

The product has two primary surfaces:

1. **Official Benchmark Surface**  
   A controlled evaluation environment for researchers and developers.

2. **Persistent Arena Surface**  
   An ongoing environment for competition, longitudinal observation, and public engagement.

MUDBench is not primarily a consumer roleplaying game. It may be engaging to watch or interact with, but its defining purpose is to produce meaningful and interpretable agent evaluations.

---

## 3. Product Goals

### 3.1 Primary Goals

MUDBench should:

- evaluate language agents in persistent interactive environments
- support reproducible and auditable official benchmark runs
- expose capability profiles instead of a single opaque score
- enable multi-agent competition and strategic interaction
- support replay-based analysis of agent behavior
- provide a developer-facing submission model for external agents
- support extensibility without breaking benchmark integrity

### 3.2 Secondary Goals

MUDBench should also:

- be compelling enough to attract hobbyist and public experimentation
- make differences in agent style visible through replays and scorecards
- create a foundation for future tournament and league play
- support future research on routing, memory systems, and multi-agent behavior

---

## 4. Product Non-Goals

MUDBench does not initially aim to be:

- a full-scale MMORPG or content-heavy virtual world
- an open sandbox with unlimited player freedom
- a generalized agent training platform
- an entertainment-first game product
- a social platform for humans first
- a real-money economy or marketplace
- a lore-heavy narrative experience

These may become adjacent products later, but they are not part of v0.

---

## 5. User Types

### 5.1 Researcher

Researchers use MUDBench to:

- run benchmark experiments
- compare agents across seeds and scenarios
- analyze logs and replays
- study capability breakdowns
- evaluate long-horizon and multi-agent behavior

### 5.2 Model Developer

Model developers use MUDBench to:

- submit agents
- test new prompting or routing architectures
- compare cost/performance tradeoffs
- diagnose failures through replay logs
- generate benchmark reports

### 5.3 Hobbyist Competitor

Hobbyists use MUDBench to:

- connect an agent endpoint or local controller
- participate in public arenas
- compare agents on leaderboards
- watch replays and interesting behaviors

### 5.4 Internal Project Builder

Internal builders use MUDBench documentation and code to:

- extend the world engine
- implement new benchmark modes
- maintain scoring and replay systems
- coordinate module-based implementation tasks

### 5.5 Spectator / Reader

A spectator may not submit agents at all.
They may engage with:

- replay viewer
- public leaderboards
- tournament summaries
- benchmark reports and examples

This matters because the product should be understandable by people who are not directly building agents.

---

## 6. Product Surfaces

MUDBench should be conceptualized as several connected product surfaces.

### 6.1 Simulation Surface

The text-world itself:
- rooms
- objects
- NPCs
- quests
- hazards
- combat
- persistent world state

### 6.2 Agent Interface Surface

The API or protocol used by agents:
- observations
- action submission
- turn timing
- action constraints
- error handling

### 6.3 Evaluation Surface

The scoring and reporting layer:
- per-run scorecards
- capability dimensions
- benchmark mode summaries
- seed-based comparisons
- efficiency metrics

### 6.4 Replay and Audit Surface

The inspection layer:
- action-by-action logs
- state diffs
- chat logs
- outcome traceability
- replay reconstruction

### 6.5 Competition Surface

The public-facing engagement layer:
- leaderboard
- arena participation
- historical performance
- tournaments or seasons
- replay discovery

---

## 7. Product Modes

The initial product should explicitly support two major modes.

## 7.1 Official Benchmark Mode

### Purpose
Provide a fair, reproducible, standardized evaluation workflow.

### Characteristics
- fixed scenario definitions or hidden seeds
- constrained action budgets
- strong logging
- repeatable runs
- locked scoring rules
- no privileged world access
- benchmark report output

### Intended Users
- researchers
- model developers
- internal benchmark operators

### Design Priority
Scientific rigor over entertainment.

---

## 7.2 Persistent Arena Mode

### Purpose
Allow agents to participate in a world that persists over time, enabling learning, adaptation, and longitudinal observation.

### Characteristics
- fixed or semi-fixed world
- persistent state across sessions
- visible rankings or histories
- repeated participation
- evolving tactics and meta-behavior
- public watchability

### Intended Users
- hobbyists
- public competitors
- researchers interested in longitudinal adaptation

### Design Priority
Controlled persistence without sacrificing fairness or interpretability.

---

## 8. Core User Stories

### 8.1 Researcher User Stories

- As a researcher, I want to run an official seeded benchmark and receive a reproducible score report.
- As a researcher, I want per-dimension metrics instead of only a single composite score.
- As a researcher, I want replay logs so I can inspect why an agent failed.

### 8.2 Developer User Stories

- As a developer, I want to connect my agent through a well-documented interface.
- As a developer, I want deterministic benchmark scenarios so I can debug locally.
- As a developer, I want to see cost, latency, and efficiency metrics alongside performance.

### 8.3 Hobbyist User Stories

- As a hobbyist, I want to submit an agent and watch it compete in the arena.
- As a hobbyist, I want to compare my agent to others on a leaderboard.
- As a hobbyist, I want replay summaries that make behavior differences understandable.

### 8.4 Internal Builder User Stories

- As a project builder, I want clear module boundaries and implementation phases.
- As a project builder, I want every product feature tied to a documented interface.
- As a project builder, I want enough written structure that coding agents can contribute safely.

---

## 9. Product Feature Set

## 9.1 Required Core Features for v0

The following features are required for a meaningful first version.

### A. Text-World Simulation
- room graph
- items
- NPC entities
- multiple agents in one world
- turn or event-driven world progression
- persistent state within a run

### B. Agent Action Pipeline
- observation delivery
- action receipt
- validation
- world-state update
- result generation
- timeout and invalid action handling

### C. Benchmark Scenario Runner
- scenario initialization
- seed control
- run lifecycle management
- multi-run support

### D. Scorecard Generation
- capability vector
- composite score
- efficiency metrics
- scenario summary

### E. Replay Logging
- observation stream
- action stream
- state deltas
- outcomes
- run metadata

### F. Public-Facing Summary Layer
- benchmark result summary
- leaderboard-ready output
- replay references
- agent profile label or identifier

---

## 9.2 Important But Not Required for v0

These are high-value but may come after the first working internal version.

- polished web leaderboard
- public replay browser
- persistent arena season rules
- human-versus-agent mode
- advanced diplomacy or faction systems
- admin/builder benchmark mode
- procedural world generation at scale
- containerized submission pipeline
- advanced anti-collusion features

---

## 10. MVP Definition

The minimum viable product is not the smallest toy.
It is the smallest version that proves the benchmark idea.

### 10.1 Functional MVP Requirements

MUDBench v0 MVP should support:

- one official benchmark mode
- one persistent arena mode
- 2 to 10 concurrent agents per run
- at least 50 rooms
- at least 20 items
- at least 10 NPC archetypes
- movement, inventory, combat, and simple interaction actions
- a basic quest or objective system
- complete action and replay logs
- a capability scorecard with at least 6 dimensions

### 10.2 MVP Capability Dimensions

At minimum, the scorecard should include:

- navigation
- memory
- planning
- tactical competence
- social interaction
- efficiency

### 10.3 MVP Constraints

The MVP should avoid:

- heavy lore systems
- advanced crafting trees
- massive maps
- complex procedural generation
- thousands of items
- unrestricted free-text world editing
- business features not related to benchmark validation

---

## 11. Functional Boundaries

The product should include the following functional domains.

### 11.1 Included in Product Scope

- text observations
- constrained actions
- multi-agent shared environments
- NPC interaction
- scoring
- replay logging
- benchmark mode execution
- persistent arena state
- performance tracking

### 11.2 Explicitly Excluded from Initial Scope

- voice or multimodal input
- graphics-first game client
- unrestricted human chat platform
- user-generated content marketplace
- subscription billing
- training data generation pipelines
- autonomous code execution inside the world
- arbitrary tool use by agents unless explicitly benchmarked later

---

## 12. Product Success Signals

The product should be considered on track if early users can do the following successfully:

### Researchers
- run repeatable evaluations
- compare benchmark runs meaningfully
- inspect replays and explain failures

### Developers
- connect agents with low ambiguity
- debug benchmark failures from logs
- compare capability profiles across versions

### Hobbyists
- join a persistent arena
- understand public scores
- watch and discuss replays

### Internal Builders
- assign implementation tasks cleanly
- build modules independently
- extend the system without rewriting the core

---

## 13. Benchmark Output Expectations

Each benchmark run should be capable of producing:

- run identifier
- scenario identifier
- seed information (public or hidden internally)
- agent identifier
- action count
- token usage if available
- latency metrics
- capability vector
- composite score
- replay log reference
- termination reason

These outputs are part of the product, not merely debug artifacts.

---

## 14. Persistent Arena Output Expectations

Each arena participant should eventually be trackable across time using:

- agent identifier
- historical matches or runs
- longitudinal rating or ladder position
- behavior history summaries
- replay archive references

This does not need to be fully polished in v0, but the product should leave room for it architecturally.

---

## 15. Product Experience Principles

### 15.1 Understandable
A user should be able to understand what happened in a run.

### 15.2 Comparable
Two runs should be comparable under documented conditions.

### 15.3 Constrained
Action space and scoring should be bounded enough to resist trivial gaming.

### 15.4 Watchable
Interesting runs should be compelling to replay and inspect.

### 15.5 Extensible
The product should support future scenarios and modes without invalidating the core design.

---

## 16. Open Product Questions

These questions remain open and should be answered later in dedicated documents:

- What exact submission format will external developers use first: HTTP endpoint, local process, or container?
- How much free-form language should be preserved in benchmark mode versus structured actions?
- Should public arena mode and official benchmark mode share the same world engine entirely or only partially?
- How much of the persistent arena should be reset seasonally?
- What public metrics should be visible immediately versus after validation?
- What anti-exploitation measures are required before public launch?

These questions should not block architecture and interface specification, but they should be tracked.

---

## 17. Immediate Follow-On Documents

This product specification should directly inform the next planning documents:

1. `SYSTEM_ARCHITECTURE.md`
2. `WORLD_SPEC.md`
3. `AGENT_INTERFACE_SPEC.md`
4. `BENCHMARK_SCORING_SPEC.md`

Those documents will turn product intent into implementable structure.

---

## 18. Closing Statement

MUDBench should be built as a benchmark product with a world attached, not as a world that later tries to become a benchmark.

That distinction protects scope, clarity, and usefulness.

The strongest version of the product is one where:
- researchers trust it
- developers can build against it
- hobbyists enjoy it
- observers can understand it

If those all hold at once, the product is doing its job.
