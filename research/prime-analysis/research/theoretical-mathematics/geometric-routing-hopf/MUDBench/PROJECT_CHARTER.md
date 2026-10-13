# MUDBench Project Charter

## Document Status
- Version: 0.1
- Status: Draft for repo initialization
- Purpose: Establish the mission, scope, goals, non-goals, and governing principles for MUDBench

---

## 1. Project Name

**MUDBench**

Working subtitle:

> A persistent multi-agent text-world benchmark for evaluating language agents on long-horizon reasoning, memory, planning, social interaction, and adversarial adaptation.

---

## 2. Mission

MUDBench exists to provide a rigorous, interpretable, and extensible environment for evaluating AI agents in a persistent text-based world that captures capabilities poorly measured by narrow or static benchmarks.

The project is intended to test how well agents can:

- navigate partially observable environments
- remember world state over long horizons
- form and execute multi-step plans
- interact strategically with other agents
- adapt to changing conditions
- operate efficiently under time, action, and token constraints
- recover from failure in persistent environments

MUDBench should serve as both:

1. a serious benchmark platform for researchers and model developers
2. a hosted competitive environment for agent builders and hobbyists

---

## 3. Vision

The long-term vision is to create a benchmark ecosystem where language agents can be evaluated in environments that resemble live systems rather than isolated question-answer tasks.

MUDBench should become:

- a standard benchmark for persistent language-agent evaluation
- a platform for multi-agent competitions and tournaments
- a tool for analyzing cognitive profiles across different models and agent architectures
- a foundation for future research into routing, memory, social reasoning, and adversarial robustness in language agents

---

## 4. Why This Project Exists

Current benchmarks often fail to capture the full behavioral profile of an agent because they are:

- too short-horizon
- too static
- too narrow in capability scope
- too sanitized
- too single-agent
- too easily overfit

A persistent MUD-style environment is useful because it naturally combines:

- sequential decision-making
- partial observability
- natural language interaction
- structured action grounding
- multi-agent competition and cooperation
- emergent behavior
- interpretable replay logs

This makes MUDBench especially suitable for evaluating real-world agent behavior.

---

## 5. Core Product Statement

MUDBench is **not primarily a game**.

MUDBench is a **benchmark platform built on a persistent text-world simulation**.

The world exists to support meaningful evaluation, not the other way around.

Entertainment value is welcome, but benchmark integrity takes priority.

---

## 6. Primary Goals

### 6.1 Benchmark Integrity
Provide reproducible, auditable, and standardized evaluations for language agents.

### 6.2 Persistent World Evaluation
Measure how agents behave in environments that evolve over time and preserve consequences.

### 6.3 Multi-Dimensional Capability Measurement
Produce structured capability profiles instead of a single opaque leaderboard number.

### 6.4 Human Interpretability
Ensure that runs can be understood through logs, replays, metrics, and observable behavior.

### 6.5 Extensibility
Support future benchmark modes, world modules, and experimental agent interfaces without requiring a total rewrite.

### 6.6 Public Competition
Enable safe, fair, and interesting public competitions between submitted agents.

---

## 7. Non-Goals

The following are explicitly out of scope for the initial project:

### 7.1 Full MMORPG Development
MUDBench is not intended to become a content-heavy fantasy MMO.

### 7.2 Endless Lore or Narrative First Design
Narrative flavor may exist, but benchmark mechanics come first.

### 7.3 General-Purpose RL Training Platform
The initial focus is evaluation and competition, not full reinforcement learning training infrastructure.

### 7.4 Unbounded Sandbox Simulation
The environment should be constrained enough to support meaningful scoring and auditing.

### 7.5 Human-Centric Monetization Features at v0
Cosmetics, subscriptions, and marketplace systems are not part of the initial build.

---

## 8. Design Principles

### 8.1 Benchmark First
All world design decisions must preserve evaluability and fairness.

### 8.2 Persistent but Controlled
Persistence is essential, but persistence must remain measurable, resettable, and auditable.

### 8.3 Structured and Observable
Every meaningful action should produce a traceable outcome.

### 8.4 Multiple World Types
MUDBench should support both:
- randomized benchmark worlds for official evaluation
- persistent arena worlds for longitudinal learning and competition

### 8.5 Capability Profiles Over Single Scores
The system should emphasize per-dimension performance over oversimplified rankings.

### 8.6 Deterministic Core, Variable Content
Simulation rules should remain deterministic where possible, while environment instances can vary by seed.

### 8.7 Small Core, Expand Later
The first implementation should be minimal, stable, and extensible.

### 8.8 Replayability and Auditability
Every official run should be replayable and reviewable.

---

## 9. Intended Users

### 9.1 Researchers
Researchers need:
- benchmark rigor
- reproducibility
- controlled experiments
- multi-run score comparisons
- replayable logs

### 9.2 Model Developers
Developers need:
- clear APIs
- fair scoring
- visible strengths and weaknesses
- cost and latency metrics
- submission workflows

### 9.3 Hobbyists and Competitors
Hobbyists need:
- accessible agent submission
- replays
- rankings
- fun public competitions

### 9.4 Internal Builders
Project contributors need:
- strong architecture docs
- modular subsystem boundaries
- deterministic core behavior
- taskable implementation plans for coding agents

---

## 10. Product Modes

MUDBench is expected to support at least two top-level modes.

### 10.1 Official Benchmark Mode
Used for standardized evaluation.

Characteristics:
- hidden seeds
- controlled action budgets
- locked scoring
- replay verification
- randomized or curated benchmark scenarios

### 10.2 Persistent Arena Mode
Used for longitudinal learning, competition, and public engagement.

Characteristics:
- fixed or semi-fixed world
- ongoing world state
- repeated participation
- evolving strategies
- long-term score histories

---

## 11. Core Capability Areas

The project is intended to evaluate, at minimum, the following capability dimensions:

- navigation
- memory
- planning
- adaptation
- combat and tactical reasoning
- social reasoning
- deception resistance
- communication quality
- resource management
- efficiency under constraints

These may later be expanded or decomposed into finer submetrics.

---

## 12. Success Criteria for v0

The first working version of MUDBench will be considered successful if it can:

1. run a stable text-world simulation
2. support at least one official benchmark scenario
3. support at least two agents interacting in the same world
4. produce structured observations and accept constrained actions
5. log complete replay traces
6. calculate a basic but meaningful capability scorecard
7. support deterministic replays from logged events
8. run from a documented repository with clear module boundaries

---

## 13. Risks and Failure Modes

Key project risks include:

- building too much world content before finalizing benchmark logic
- overcomplicating the action interface
- creating scores that are noisy or easily gamed
- allowing public competition features to distort benchmark integrity
- failing to keep the implementation modular enough for agent-led development
- letting nostalgia override scientific clarity

These risks should be actively managed through documentation, phase gates, and benchmark-first design reviews.

---

## 14. Governance Rules for Design Decisions

When there is a conflict between priorities, the project should prefer:

1. benchmark integrity over novelty
2. interpretability over opaque complexity
3. modularity over convenience
4. deterministic behavior over hard-to-debug simulation randomness
5. constrained extensibility over uncontrolled feature sprawl
6. documented interfaces over implicit assumptions

---

## 15. Immediate Project Priorities

The first planning sequence for the repository is:

1. define the product clearly
2. define the system architecture
3. define the world model
4. define the agent interface
5. define the scoring model
6. define replay and telemetry standards
7. define implementation phases for coding agents

No implementation work should begin until the first four of these are documented.

---

## 16. Out-of-Scope Questions for Later

The following questions are intentionally deferred:

- final business model details
- pricing strategy
- hosted ladder season rules
- training-vs-evaluation product split
- human player integration
- builder/admin benchmark mode specifics
- advanced economy simulation
- procedural world generation complexity
- anti-collusion policy for public tournaments

These should be addressed in future product and operations documents, not in the initial charter.

---

## 17. Closing Statement

MUDBench should be built as a serious benchmark platform with enough structure to support scientific use and enough life to generate meaningful emergent behavior.

The world is not the point.
The measurable behavior of agents inside the world is the point.

That distinction should remain visible in every design and implementation choice.
