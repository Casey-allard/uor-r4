
# MUDBench Replay and Telemetry Specification

## Document Status
Version: 0.1
Status: Draft
Purpose: Define how MUDBench records, reconstructs, and analyzes simulation runs through replay logs and telemetry data.

---

# 1. Design Goals

Replay and telemetry systems must support:

- deterministic run reconstruction
- auditability of benchmark results
- debugging of agent behavior
- research analysis of agent decisions
- public replay viewing

Every official benchmark run must produce a complete replay artifact.

---

# 2. Relationship to Architecture

Replay and telemetry are defined as a core subsystem in the system architecture.

The simulation pipeline generates events which are captured by the replay layer and stored for reconstruction.

Events originate from the world engine and evaluation engine and must be recorded without altering simulation behavior.

---

# 3. Replay Philosophy

A replay should represent a **complete causal trace** of the simulation.

The replay log must allow reconstruction of:

- observations delivered to agents
- actions taken by agents
- world state changes
- evaluation signals
- termination conditions

If two runs share the same seed and actions, they must replay identically.

---

# 4. Replay Log Structure

Each run produces a replay log consisting of ordered events.

High-level structure:

Run Metadata
→ Initial World State
→ Step Events
→ Termination Summary

Example structure:

ReplayLog
- run_metadata
- seed
- scenario_id
- agent_ids
- initial_world_state
- step_events
- final_scorecard

---

# 5. Run Metadata

Metadata describes the run configuration.

Example:

{
  "run_id": "run_8349",
  "benchmark_version": "0.1",
  "scenario_id": "forest_trial",
  "seed": 1837281,
  "agent_ids": ["agentA","agentB"],
  "start_time": "2026-01-10T13:41:22Z"
}

This information ensures traceability across runs.

---

# 6. Step Event Structure

Each simulation step produces an event block.

Example:

StepEvent
- step_number
- observation
- agent_action
- world_events
- evaluation_signals
- timestamp

Events must appear in strict order.

---

# 7. Observation Logging

The replay must record the exact observation sent to each agent.

Example:

{
  "agent_id": "agentA",
  "observation": {
    "location": "forest_path",
    "exits": ["north","south"],
    "entities": ["goblin"]
  }
}

This allows analysis of what the agent actually perceived.

---

# 8. Action Logging

The replay must capture the action chosen by each agent.

Example:

{
  "agent_id": "agentA",
  "action": "move north"
}

Actions must be recorded before world resolution occurs.

---

# 9. World Event Logging

The replay must capture resulting world events.

Examples:

- movement
- combat damage
- item pickup
- quest progress

Example event:

{
  "event_type": "movement",
  "entity": "agentA",
  "from": "forest_path",
  "to": "forest_clearing"
}

---

# 10. Evaluation Signals

Replay logs must record scoring signals emitted by the evaluation engine.

Examples:

- rooms_explored
- quest_completed
- combat_success
- resource_collected

These signals support later score verification.

---

# 11. Replay Reconstruction

The replay system must allow full reconstruction of the run.

Reconstruction steps:

1. initialize world state
2. apply recorded actions
3. replay world events
4. recompute observations
5. verify evaluation signals

The reconstructed run must match the original run outcome.

---

# 12. Telemetry Metrics

In addition to replay logs, the system should record telemetry metrics.

Examples:

- agent response latency
- token usage
- number of invalid actions
- action distribution
- step duration

Telemetry supports research and performance analysis.

---

# 13. Storage Strategy

Replay logs may be large and should be stored efficiently.

Recommended strategy:

- structured JSON logs for core events
- compressed storage for long runs
- metadata index for fast lookup

Database references may store summary data while logs remain in file storage.

---

# 14. Replay Viewer Support

The replay format should support visualization tools.

Possible viewer features:

- step-by-step playback
- room map visualization
- action timeline
- score breakdown overlay
- agent comparison mode

These tools help researchers and spectators understand behavior.

---

# 15. Public Replay Access

Some replays may be publicly visible.

Public replay policies may include:

- anonymized agent identifiers
- delayed release after tournaments
- summarized replay mode for spectators

Official benchmark runs may restrict replay visibility until validation is complete.

---

# 16. Integrity and Verification

Replay logs must support integrity checks.

Possible safeguards:

- checksum of replay file
- run signature hash
- deterministic verification tests

These checks ensure that benchmark results cannot be manipulated.

---

# 17. Debugging Support

Replay logs should support debugging tools.

Examples:

- event filtering
- anomaly detection
- agent decision inspection
- divergence comparison across runs

These tools are critical during development.

---

# 18. Data Retention

Replay retention policies may vary.

Suggested tiers:

Short-term:
- all runs stored

Medium-term:
- benchmark runs retained

Long-term:
- notable or tournament runs archived

---

# 19. Future Telemetry Extensions

Future telemetry may include:

- internal agent reasoning traces (optional)
- multi-agent communication analysis
- emergent behavior detection
- anomaly detection metrics
- strategy classification

These features should not break the base replay format.

---

# 20. Closing Statement

Replay logs are the foundation of interpretability in MUDBench.

A benchmark that cannot be inspected cannot be trusted.

The replay and telemetry system ensures that every score corresponds to observable behavior.
