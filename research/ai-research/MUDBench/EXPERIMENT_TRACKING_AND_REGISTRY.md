# MUDBench Experiment Tracking and Registry Specification

## Document Status
Version: 0.1
Status: Draft
Purpose: Define how experiments, runs, agents, and artifacts are tracked, linked, and reproducibly registered across MUDBench.

---

# 1. Why This Exists

At this point, the system can:

- run simulations
- generate replays
- compute scores
- store artifacts

And yet, without a registry:

- results become detached from their origin
- experiments cannot be reproduced reliably
- comparisons become ambiguous

This document defines the canonical system for experiment tracking.

---

# 2. Core Principles

All experiment tracking must be:

- deterministic
- versioned
- fully traceable
- reproducible
- immutable (append-only)

If you cannot trace a result to its exact inputs, it does not count.

---

# 3. Key Concepts

## 3.1 Experiment

A logical grouping of runs designed to test a hypothesis.

## 3.2 Run

A single execution instance of:

- agent
- scenario
- seed
- configuration

## 3.3 Artifact

Any output produced by a run:

- replay file
- scorecard
- telemetry
- logs

## 3.4 Registry

A structured index that links experiments → runs → artifacts.

---

# 4. Experiment Schema

```
{
  "experiment_id": str,
  "description": str,
  "created_at": timestamp,
  "author": str,
  "hypothesis": str,
  "benchmark_version": str,
  "scoring_version": str,
  "tags": [str]
}
```

---

# 5. Run Schema

```
{
  "run_id": str,
  "experiment_id": str,
  "agent_id": str,
  "agent_version": str,
  "scenario_id": str,
  "scenario_version": str,
  "seed": int,
  "status": "completed | failed",
  "start_time": timestamp,
  "end_time": timestamp,
  "artifacts": {
    "replay_path": str,
    "scorecard_path": str,
    "telemetry_path": str
  }
}
```

---

# 6. Artifact Registry

Artifacts must be indexed and linked.

```
{
  "artifact_id": str,
  "run_id": str,
  "type": "replay | scorecard | telemetry",
  "path": str,
  "hash": str,
  "created_at": timestamp
}
```

---

# 7. Directory Structure

```
/registry/
  experiments.json
  runs.json
  artifacts.json
```

Optional sharding for scale:

```
/registry/experiments/
/registry/runs/
/registry/artifacts/
```

---

# 8. Versioning Requirements

Each experiment and run must include:

- benchmark_version
- scoring_version
- scenario_version
- agent_version

No version → no validity.

---

# 9. Immutability Rules

- experiments are append-only
- runs cannot be modified after completion
- artifacts are immutable

Corrections require new entries.

---

# 10. Traceability Guarantees

Every score must be traceable to:

- exact run
- exact agent version
- exact scenario
- exact seed
- exact replay

Missing linkage invalidates the result.

---

# 11. Determinism Validation

Each run must be reproducible:

- same inputs → identical replay
- identical scorecard

Validation process:

- rerun with same config
- compare hashes

Mismatch → registry entry invalid

---

# 12. Experiment Lifecycle

1. Create experiment entry
2. Register runs
3. Execute runs
4. Store artifacts
5. Validate determinism
6. Mark experiment complete

---

# 13. Query Capabilities

Registry must support queries like:

- all runs for experiment X
- all runs for agent Y
- best performance for scenario Z
- runs by version

---

# 14. Failure Handling

Failed runs must be recorded:

- status = failed
- include error logs
- do not discard

Failure data is part of truth.

---

# 15. Integration Points

Registry integrates with:

- Data Schema and Storage
- CI Pipeline
- Leaderboard System
- Replay System

It must not embed logic from those systems.

---

# 16. Hashing Requirements

All artifacts must include:

- content hash (SHA-1 or SHA-256)
- used for integrity verification

---

# 17. Scaling Considerations

For large-scale experiments:

- shard registry
- use indexed storage (e.g., Parquet)
- support distributed writes (controlled)

---

# 18. Security Constraints

- no arbitrary file references
- validate paths
- enforce access boundaries

---

# 19. Anti-Patterns

Do not allow:

- orphaned runs (no experiment_id)
- missing artifacts
- mutable entries
- implicit versioning

These destroy reproducibility.

---

# 20. Future Extensions

- experiment comparison tools
- automated regression detection
- lineage graphs
- experiment dashboards

---

# 21. Closing Statement

You are not tracking experiments for convenience.

You are tracking them so that:

- results can be trusted
- claims can be verified
- progress can be measured

Without a registry, you are guessing.

With it, you are doing science.
