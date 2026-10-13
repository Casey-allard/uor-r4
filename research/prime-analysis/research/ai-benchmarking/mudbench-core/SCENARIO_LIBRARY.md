# MUDBench Scenario Library

## Document Status
Version: 0.1
Status: Draft
Purpose: Define the canonical set of benchmark scenarios used in MUDBench and their capability coverage.

---

# 1. Overview

The Scenario Library defines the **canonical benchmark scenarios**.

These scenarios:
- instantiate the Scenario Specification
- drive scoring signals
- define benchmark identity
- enable reproducible evaluation

---

# 2. Scenario Categories

## 2.1 Navigation

### navigation_forest_v1
- Goal: Explore and map a forest region
- Signals:
  - rooms_explored
  - path_efficiency
- Difficulty: Easy

---

## 2.2 Memory

### memory_delayed_retrieval_v1
- Goal: Retrieve an item seen earlier after delay
- Signals:
  - recall_accuracy
  - retrieval_success
- Difficulty: Medium

---

## 2.3 Planning

### planning_multi_step_quest_v1
- Goal: Complete a multi-step quest chain
- Signals:
  - plan_completion
  - step_efficiency
- Difficulty: Medium

---

## 2.4 Tactical

### tactical_combat_arena_v1
- Goal: Defeat enemies efficiently
- Signals:
  - combat_success
  - damage_efficiency
- Difficulty: Medium

---

## 2.5 Social

### social_trade_negotiation_v1
- Goal: Successfully trade with NPCs or agents
- Signals:
  - trade_success
  - dialogue_efficiency
- Difficulty: Medium

---

## 2.6 Mixed

### mixed_adversarial_competition_v1
- Goal: Compete with other agents for objectives
- Signals:
  - objective_completion
  - interference_success
- Difficulty: Hard

---

# 3. Capability Coverage Matrix

| Scenario                          | Nav | Mem | Plan | Tactical | Social | Eff |
|----------------------------------|-----|-----|------|----------|--------|-----|
| navigation_forest_v1             |  ✓  |     |      |          |        |  ✓  |
| memory_delayed_retrieval_v1      |     |  ✓  |  ✓   |          |        |     |
| planning_multi_step_quest_v1     |     |     |  ✓   |          |        |  ✓  |
| tactical_combat_arena_v1         |     |     |      |    ✓     |        |  ✓  |
| social_trade_negotiation_v1      |     |     |      |          |   ✓    |     |
| mixed_adversarial_competition_v1 |  ✓  |  ✓  |  ✓   |    ✓     |   ✓    |  ✓  |

---

# 4. Difficulty Tiers

- Easy: navigation_forest_v1
- Medium:
  - memory_delayed_retrieval_v1
  - planning_multi_step_quest_v1
  - tactical_combat_arena_v1
  - social_trade_negotiation_v1
- Hard:
  - mixed_adversarial_competition_v1

---

# 5. Anti-Exploitation Notes

## navigation_forest_v1
- Prevent loop farming
- Penalize redundant movement

## memory_delayed_retrieval_v1
- Prevent brute-force search resets
- Require delayed recall

## planning_multi_step_quest_v1
- Prevent partial completion scoring abuse

## tactical_combat_arena_v1
- Prevent stall tactics

## social_trade_negotiation_v1
- Prevent spam interaction loops

## mixed_adversarial_competition_v1
- Prevent collusion
- Penalize passive strategies

---

# 6. Versioning

Each scenario must include:
- scenario_id
- version
- changelog

Example:
navigation_forest_v1 → v1.0

---

# 7. Future Extensions

- procedural scenario generation
- adaptive difficulty scaling
- dynamic scenario composition

---

# 8. Closing Statement

The Scenario Library defines what intelligence looks like inside MUDBench.

A weak library produces meaningless scores.
A strong library produces real evaluation.
