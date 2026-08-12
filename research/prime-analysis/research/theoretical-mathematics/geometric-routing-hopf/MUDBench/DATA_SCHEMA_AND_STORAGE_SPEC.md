# MUDBench Data Schema and Storage Specification

## Document Status
Version: 0.1
Status: Draft
Purpose: Define how data is structured, stored, versioned, and accessed across MUDBench.

---

# 1. Why This Exists

You now have:
- simulation outputs
- replay logs
- scoring artifacts
- agent metadata

Without a strict schema:
- data becomes inconsistent
- reproducibility breaks
- analysis becomes unreliable

This document defines the canonical structure of all persisted data.

---

# 2. Core Principles

All data must be:
- deterministic
- versioned
- schema-validated
- immutable (once committed)
- reproducible from source

If data cannot be trusted, the benchmark is useless.

---

# 3. Data Categories

## 3.1 Core Data Types

- World State Snapshots
- Replay Logs
- Agent Actions
- Scorecards
- Telemetry Metrics
- Scenario Definitions
- Configuration Metadata

---

# 4. Canonical Data Formats

## 4.1 Preferred Formats

- JSON (human-readable, structured)
- NPZ (numerical arrays, compressed)
- Parquet (large-scale tabular data)

## 4.2 Constraints

- no ad-hoc formats
- no undocumented fields
- strict schema validation required

---

# 5. Replay Data Schema

Replay files must include:

```
{
  "seed": int,
  "scenario_id": str,
  "scenario_version": str,
  "steps": [
    {
      "t": int,
      "actions": [...],
      "events": [...],
      "state_hash": str
    }
  ]
}
```

---

# 6. Scorecard Schema

```
{
  "agent_id": str,
  "benchmark_version": str,
  "scores": {
    "navigation": float,
    "memory": float,
    "planning": float,
    "tactical": float,
    "social": float,
    "efficiency": float
  },
  "composite_score": float
}
```

---

# 7. Agent Metadata Schema

```
{
  "agent_id": str,
  "version": str,
  "parameters": dict,
  "training_info": dict (optional)
}
```

---

# 8. Storage Layout

```
/data/
  /replays/
  /scorecards/
  /agents/
  /telemetry/
  /scenarios/
```

Each directory must be version-scoped.

---

# 9. Versioning Requirements

All data must include:

- benchmark_version
- scoring_version
- scenario_version

Mismatch → data invalid for comparison

---

# 10. Immutability Rules

Once written:

- data must not be modified
- corrections require new version
- original artifacts must be preserved

---

# 11. Data Validation

All data must pass:

- schema validation
- determinism checks
- replay reconstruction checks

Invalid data must be rejected.

---

# 12. Access Patterns

Data access must be:

- read-only for analysis
- append-only for new runs
- version-aware

---

# 13. Compression and Efficiency

- large datasets must be compressed
- avoid duplication
- prefer columnar storage for analytics

---

# 14. Security Considerations

- no arbitrary code execution in data
- validate all inputs
- restrict file access boundaries

---

# 15. Future Extensions

- distributed storage
- streaming telemetry pipelines
- real-time analytics layers

---

# 16. Closing Statement

Data defines truth in this system.

If your data layer is sloppy, your conclusions will be fiction.
