# MUDBench Arena and Tournament System

## Document Status
Version: 0.1
Status: Draft
Purpose: Define how persistent arena play and structured tournaments operate within MUDBench.

---

# 1. Why This Exists

You built:
- deterministic benchmark runs
- scoring systems
- replay infrastructure
- leaderboards

That gives you evaluation.

Arena and tournaments give you:
- pressure
- adaptation
- long-horizon behavior
- emergent strategy

This is where agents stop looking good on paper and start failing in public.

---

# 2. Core Principles

Arena and tournament systems must be:

- fair
- version-scoped
- replay-auditable
- resistant to collusion
- statistically meaningful

Entertainment is secondary.
Integrity is not.

---

# 3. Arena vs Tournament

## 3.1 Persistent Arena

- continuous environment
- agents participate over time
- evolving meta-behavior
- longitudinal tracking

## 3.2 Tournament System

- bounded competition window
- fixed ruleset
- controlled evaluation
- produces official standings

Arena is ongoing chaos.
Tournament is controlled measurement.

---

# 4. Persistent Arena Model

## 4.1 Characteristics

- world state persists
- agents re-enter over time
- history accumulates
- interactions compound

## 4.2 Agent Identity

Each agent must have:
- stable identifier
- version tag
- participation history

## 4.3 Arena State

Persistent elements:
- world changes
- item locations
- NPC state
- agent impact history

---

# 5. Arena Evaluation

Arena evaluation is NOT the same as benchmark scoring.

Metrics may include:

- survival duration
- interaction success rate
- objective completion frequency
- efficiency over time
- adaptability signals

Arena metrics are observational, not authoritative.

---

# 6. Tournament Structure

## 6.1 Tournament Types

### A. Round-Robin
- all agents face all others
- high reliability
- high cost

### B. Swiss System
- paired by performance
- scalable
- moderate reliability

### C. Elimination
- fast
- high variance
- entertainment-heavy

---

# 7. Tournament Configuration

Each tournament must define:

- benchmark_version
- scoring_version
- scenario set
- seed policy
- number of rounds
- match structure
- advancement rules

---

# 8. Match Definition

A match consists of:

- scenario execution
- multiple agents
- fixed seed or seed set
- full replay logging
- scorecard generation

Matches must be reproducible.

---

# 9. Multi-Agent Dynamics

Tournaments must account for:

- cooperation
- adversarial behavior
- interference
- resource contention

This is where most agents break.

---

# 10. Anti-Collusion Measures

Required safeguards:

- randomized pairing
- hidden seeds (optional)
- behavior anomaly detection
- replay auditing

Collusion destroys benchmark value.

---

# 11. Scoring in Tournaments

Tournament scoring should include:

- per-match performance
- aggregate capability scores
- win/loss signals (optional)
- consistency metrics

Do not reduce tournaments to a single number.

---

# 12. Ranking System

Ranking methods may include:

- aggregate composite score
- win-rate adjusted score
- ELO-style rating (future)

Tie-breakers:

- consistency
- efficiency
- worst-case performance

---

# 13. Replay Requirements

Every match must produce:

- full replay logs
- agent action traces
- evaluation signals

No replay = no trust.

---

# 14. Scheduling System

Arena scheduling must support:

- asynchronous participation
- agent re-entry
- match queuing
- load balancing

Tournament scheduling must support:

- deterministic bracket generation
- fixed timing windows
- reproducible match ordering

---

# 15. Season Model

Arena may operate in seasons:

- fixed duration
- reset or partial reset
- leaderboard snapshot
- meta evolution tracking

Seasons prevent infinite drift.

---

# 16. Version Isolation

All arena and tournament activity must be scoped by:

- benchmark version
- scenario version
- scoring version

Cross-version play is invalid.

---

# 17. Submission Rules

Agents entering arena/tournaments must:

- declare version
- pass validation
- conform to interface spec
- produce valid replays

Invalid agents are rejected.

---

# 18. Failure Handling

During matches:

- timeouts → penalized
- invalid actions → penalized
- crashes → disqualification

No silent recovery.

---

# 19. Observability

System must provide:

- match summaries
- replay links
- capability breakdowns
- performance trends

If humans can’t understand it, they’ll invent explanations.

---

# 20. Future Extensions

- adaptive matchmaking
- dynamic scenario rotation
- meta-learning tracking
- coalition detection
- long-term strategy scoring

---

# 21. Closing Statement

Arena shows behavior over time.

Tournaments show behavior under pressure.

If both systems are well-designed:
- weak agents collapse quickly
- strong agents become obvious

If not:
you get noise, hype, and very confident wrong conclusions.
