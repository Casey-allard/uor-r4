# MUDBench Leaderboard and Evaluation Reporting

## Document Status
Version: 0.1
Status: Draft
Purpose: Define how benchmark results are aggregated, compared, presented, and audited across agents and runs.

---

# 1. Why This Exists

You now have:

- scoring outputs
- replay logs
- deterministic runs
- versioned releases

Congratulations. You’ve created numbers.

Now comes the part humans consistently ruin:

**comparison.**

Without strict leaderboard rules:
- results get cherry-picked
- scores get misinterpreted
- weak agents look strong
- strong agents look inconsistent

This document prevents that.

---

# 2. Core Principles

Leaderboard systems must be:

- version-aware
- reproducible
- statistically grounded
- resistant to gaming
- transparent and auditable

If a score cannot be explained, it should not be ranked.

---

# 3. Leaderboard Scope

Leaderboards must always be scoped by:

- benchmark_version
- scoring_version
- scenario set
- evaluation protocol

Example:

Leaderboard: MUDBench v0.1.0 (canonical scenarios)

Scores across different versions are NOT comparable.

---

# 4. Evaluation Unit

The atomic unit is a **run**.

Each run produces:
- capability vector
- composite score
- replay artifact
- metadata

Leaderboards must never rank a single run directly.

That would be absurdly noisy.

---

# 5. Aggregation Strategy

Each agent must be evaluated across multiple runs.

Minimum:

- N >= 5 runs per scenario
- multiple seeds

Aggregate metrics:

- mean score
- standard deviation
- worst-case (optional)
- best-case (optional)

---

# 6. Primary Ranking Metric

Default ranking metric:

composite_mean

Tie-breakers:

1. composite_std (lower is better)
2. worst_case_score
3. efficiency score

---

# 7. Capability Breakdown Reporting

Each leaderboard entry must include:

- navigation
- memory
- planning
- tactical
- social
- efficiency

This prevents:

- overfitting to composite score
- hiding weaknesses

---

# 8. Statistical Reporting

Each agent entry must report:

- mean score
- std deviation
- number of runs
- confidence interval (optional)

Example:

{
  "agent_id": "agent_X",
  "composite_mean": 0.72,
  "composite_std": 0.04,
  "runs": 10
}

---

# 9. Scenario-Level Reporting

Scores must also be broken down per scenario.

This allows:

- capability isolation
- failure diagnosis
- scenario-specific benchmarking

---

# 10. Replay Linking

Every leaderboard entry must link to:

- at least one representative replay
- ideally multiple runs

Leaderboard without replay = unverifiable claims

---

# 11. Anti-Gaming Rules

Prevent leaderboard manipulation:

- require multiple seeds
- reject single-run submissions
- detect abnormal variance
- flag suspicious patterns

Examples:

- extreme best-case only reporting
- deterministic exploitation of single seed

---

# 12. Submission Requirements

Each submission must include:

- agent identifier
- version of agent
- benchmark version
- run artifacts
- replay logs
- scorecards

Incomplete submissions are rejected.

---

# 13. Version Separation

Leaderboards must be partitioned by version.

Example:

- v0.1.0 leaderboard
- v0.2.0 leaderboard

Do NOT mix them.

Ever.

---

# 14. Historical Tracking

Leaderboards should maintain:

- historical rankings
- performance over time
- version transitions

This enables:

- progress tracking
- regression detection

---

# 15. Public vs Private Leaderboards

## Public

- visible rankings
- replay access (possibly delayed)
- anonymized agents optional

## Private

- internal testing
- experimental agents
- unpublished scenarios

---

# 16. Evaluation Protocol Modes

## 16.1 Fixed Benchmark Mode

- canonical scenarios
- fixed seeds
- official leaderboard

## 16.2 Extended Evaluation Mode

- additional scenarios
- stress tests
- not leaderboard-authoritative

---

# 17. Reporting Format

Leaderboard entries should be machine-readable.

Example:

{
  "agent_id": "agent_X",
  "version": "1.2",
  "benchmark_version": "0.1.0",
  "composite_mean": 0.72,
  "composite_std": 0.04,
  "capabilities": {
    "navigation": 0.80,
    "memory": 0.65,
    "planning": 0.70,
    "tactical": 0.60,
    "social": 0.55,
    "efficiency": 0.75
  }
}

---

# 18. Visualization Guidelines

Leaderboards should support:

- ranking tables
- capability radar charts
- score distributions
- scenario breakdowns

Visuals must reflect actual data, not smoothed nonsense.

---

# 19. Confidence and Uncertainty

Leaderboard positions must reflect uncertainty.

Agents with overlapping confidence intervals should not be treated as meaningfully different.

---

# 20. Disqualification Rules

Reject results that:

- fail replay validation
- violate determinism
- show inconsistent scoring
- exploit scoring bugs

---

# 21. Future Extensions

- ELO-style ranking systems
- head-to-head evaluation
- multi-agent tournament ladders
- longitudinal performance curves

---

# 22. Closing Statement

Leaderboards do not measure intelligence.

They measure performance under constraints.

If the constraints are weak, the leaderboard lies.

If the reporting is weak, humans will misread it anyway.

This document exists to reduce both problems.
