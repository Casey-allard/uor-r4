
# MUDBench Benchmark Scoring Specification

## Document Status
Version: 0.1
Status: Draft
Purpose: Define the scoring model used to evaluate agent performance in MUDBench benchmark runs.

---

# 1. Design Goals

The scoring system must be:

- interpretable
- resistant to trivial gaming
- reproducible across runs
- decomposable into capability dimensions
- compatible with replay inspection

A single opaque score is insufficient.

MUDBench scoring therefore produces a **capability vector** and a **composite score**.

---

# 2. Capability Dimensions

The benchmark evaluates agents across multiple capability dimensions.

Initial MVP dimensions:

1. Navigation
2. Memory
3. Planning
4. Tactical Competence
5. Social Interaction
6. Efficiency

Each dimension produces an independent normalized score.

---

# 3. Navigation Score

Measures the agent's ability to explore and map the environment.

Signals used:

- unique rooms visited
- efficient path usage
- avoidance of repeated loops
- discovery of key locations

Example metric:

navigation_score =
unique_rooms_visited / total_rooms_available

Modifiers may reward discovery of difficult locations.

---

# 4. Memory Score

Measures whether the agent remembers world state across time.

Signals used:

- revisiting known locations efficiently
- retrieving previously seen items
- correct reference to known NPCs
- completion of delayed objectives

Example metric:

memory_score =
successful_memory_tasks / total_memory_tasks

Memory tasks may include delayed retrieval or location recall.

---

# 5. Planning Score

Measures multi-step reasoning ability.

Signals used:

- completion of multi-step quests
- successful execution of long action chains
- avoidance of dead-end strategies

Example metric:

planning_score =
completed_plans / attempted_plans

Complex objectives carry higher weight.

---

# 6. Tactical Competence Score

Measures effectiveness in dynamic interactions such as combat.

Signals used:

- enemies defeated
- damage efficiency
- survival under threat
- correct use of equipment

Example metric:

tactical_score =
combat_successes / combat_encounters

Optional modifiers:

- damage efficiency
- strategic retreats
- correct item usage

---

# 7. Social Interaction Score

Measures interaction quality with NPCs and other agents.

Signals used:

- quest dialogue success
- cooperative actions
- trading behavior
- avoidance of hostile misfires

Example metric:

social_score =
successful_interactions / interaction_attempts

---

# 8. Efficiency Score

Measures performance relative to resource usage.

Signals used:

- steps used
- token usage (if available)
- latency
- unnecessary actions

Example metric:

efficiency_score =
optimal_steps / actual_steps_used

This score penalizes excessive wandering or redundant actions.

---

# 9. Capability Vector

Each run produces a capability vector:

[
  navigation,
  memory,
  planning,
  tactical,
  social,
  efficiency
]

Each value ranges from **0.0 to 1.0**.

---

# 10. Composite Score

The composite score summarizes overall performance.

Example formula:

composite_score =
weighted_sum(capability_vector)

Example weights:

navigation = 0.20
memory = 0.20
planning = 0.20
tactical = 0.15
social = 0.15
efficiency = 0.10

Weights may change across benchmark versions but must be documented.

---

# 11. Normalization

Scores must be normalized to ensure comparability across runs.

Normalization factors include:

- world size
- number of agents
- scenario difficulty
- action budgets

Normalization rules must be deterministic.

---

# 12. Benchmark Scenario Metrics

Each scenario defines additional scoring signals.

Example signals:

- specific quest completion
- artifact retrieval
- region exploration milestones
- boss defeat events

These scenario-specific signals feed capability dimensions.

---

# 13. Run Termination Conditions

A benchmark run may terminate when:

- step limit reached
- agent death
- objective completion
- timeout failures

The termination reason must be recorded in the final score report.

---

# 14. Anti-Gaming Safeguards

The scoring system must discourage trivial strategies.

Examples of safeguards:

- diminishing rewards for repeated actions
- loop detection penalties
- idle action penalties
- resource waste penalties

Runs that exploit scoring bugs may be invalidated.

---

# 15. Random Seeds

Benchmark scenarios use deterministic seeds.

Seeds allow:

- reproducibility
- controlled variability
- fair comparisons

Seed disclosure policies may vary between public and private benchmarks.

---

# 16. Multi-Agent Score Attribution

When multiple agents interact:

- scores are calculated individually
- shared events may contribute to multiple agents
- adversarial outcomes affect tactical metrics

Cooperation bonuses may be introduced in future versions.

---

# 17. Scorecard Output

Each run should produce a structured scorecard.

Example:

{
  "run_id": "abc123",
  "agent_id": "agent_7",
  "navigation": 0.83,
  "memory": 0.61,
  "planning": 0.74,
  "tactical": 0.55,
  "social": 0.48,
  "efficiency": 0.70,
  "composite": 0.69,
  "termination_reason": "step_limit"
}

---

# 18. Replay Alignment

All scoring signals must correspond to events visible in replay logs.

This ensures that:

- researchers can audit results
- scoring bugs can be diagnosed
- agent behavior can be interpreted

Hidden scoring logic is discouraged.

---

# 19. Benchmark Versioning

Each benchmark release must specify:

- scoring version
- capability definitions
- normalization rules
- scenario definitions

Example:

benchmark_version = "0.1"

This prevents score drift across releases.

---

# 20. Future Scoring Extensions

Future versions may introduce:

- deception detection
- coalition formation metrics
- negotiation success
- long-term strategy evaluation
- meta-learning performance

These should remain additive and versioned.

---

# 21. Closing Statement

A benchmark is only as good as its scoring model.

The scoring system must reward genuine capability rather than superficial tricks.

Every metric should correspond to observable behavior inside the simulation.
