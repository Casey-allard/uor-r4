# MUDBench Agent Baseline Library

## Document Status
Version: 0.1
Status: Draft
Purpose: Define a canonical set of baseline agents used for benchmarking, comparison, and regression testing.

---

# 1. Why This Exists

If you don’t define baselines, your benchmark turns into vibes and anecdotes.

Baselines provide:
- reference performance floors
- sanity checks for scoring
- regression detection
- comparative grounding for new agents

If a new agent cannot beat a baseline, something is wrong.

---

# 2. Core Principles

All baseline agents must be:

- deterministic
- simple
- reproducible
- well-documented
- versioned

Baselines are not meant to be impressive.
They are meant to be reliable.

---

# 3. Baseline Categories

## 3.1 Random Agent

Behavior:
- selects random valid action each step

Purpose:
- establish absolute lower bound
- detect scoring inflation

Constraints:
- must use seeded RNG
- must only select valid actions

---

## 3.2 Greedy Agent

Behavior:
- selects locally optimal action based on immediate reward signal

Purpose:
- test short-term optimization
- highlight lack of planning

---

## 3.3 Rule-Based Agent

Behavior:
- follows fixed heuristics per scenario

Examples:
- navigation: prefer unexplored nodes
- memory: store and retrieve key items
- combat: prioritize weakest enemy

Purpose:
- establish structured non-learning baseline

---

## 3.4 Scripted Optimal Agent (Scenario-Specific)

Behavior:
- uses hardcoded optimal or near-optimal strategy

Purpose:
- approximate upper bound for specific scenario
- validate scoring ceiling

Constraints:
- must be transparent
- must not rely on hidden state

---

## 3.5 Memory-Augmented Agent

Behavior:
- maintains internal state across steps
- recalls past observations

Purpose:
- test memory capability impact

---

## 3.6 Planning Agent (Shallow Search)

Behavior:
- performs limited-depth lookahead (e.g., BFS/DFS)

Purpose:
- test planning signal sensitivity

Constraints:
- bounded computation
- deterministic traversal order

---

# 4. Baseline Specification Format

Each baseline must be defined as:

```
{
  "agent_id": str,
  "version": str,
  "category": str,
  "description": str,
  "deterministic": true,
  "parameters": dict,
  "supported_scenarios": [str]
}
```

---

# 5. Implementation Rules

- must conform to AGENT_INTERFACE_SPEC
- must produce valid actions only
- must not access hidden world state
- must be replay-compatible

---

# 6. Evaluation Role

Baselines must be included in:

- CI validation runs
- regression tests
- leaderboard calibration

---

# 7. Regression Detection

Baseline performance must remain stable.

If a baseline score changes:

- investigate scoring drift
- investigate simulation changes

Baseline drift = system instability

---

# 8. Versioning

Each baseline must include:

- agent_id
- version
- changelog

Changes must be explicit and reproducible.

---

# 9. Storage Location

Recommended directory:

```
examples/agents/baselines/
```

---

# 10. Anti-Patterns

Do not:

- make baselines overly complex
- hide logic
- allow stochastic behavior without seed
- tune baselines to inflate performance

---

# 11. Future Extensions

- learned baselines (frozen models)
- hybrid heuristic + learning agents
- cross-scenario generalist baselines

---

# 12. Closing Statement

Baselines are the only thing standing between you and self-deception.

If they are weak, your benchmark lies.
If they are unstable, your benchmark drifts.
If they are missing, your benchmark is meaningless.
