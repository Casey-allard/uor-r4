# MUDBench Release and Versioning Strategy

## Document Status
Version: 0.1
Status: Draft
Purpose: Define how MUDBench versions, packages, and releases its benchmark system to ensure reproducibility, comparability, and long-term stability.

---

# 1. Why This Exists

You’ve built:

- a deterministic simulation system
- a benchmark scoring model
- a scenario library
- a CI validation pipeline

Now comes the part people usually mess up:

**releases.**

Without strict versioning:
- scores become incomparable
- results drift silently
- research becomes meaningless

---

# 2. Core Principles

All releases must be:

- versioned explicitly
- reproducible from source
- tied to exact specifications
- immutable once published
- traceable to CI artifacts

If two users cannot reproduce the same result from the same version, the release is invalid.

---

# 3. Versioning Scheme

MUDBench uses **semantic versioning**:

MAJOR.MINOR.PATCH

Example:
0.1.0

## 3.1 MAJOR Version

Increment when:

- breaking changes occur
- scoring model changes
- scenario definitions change incompatibly
- agent interface changes

Effect:
- results are NOT comparable across major versions

---

## 3.2 MINOR Version

Increment when:

- new features are added
- new scenarios are introduced
- backward-compatible improvements occur

Effect:
- results are comparable with caveats

---

## 3.3 PATCH Version

Increment when:

- bug fixes are applied
- non-functional improvements occur
- documentation updates are made

Effect:
- results remain fully comparable

---

# 4. Versioned Components

Each release must explicitly version:

- benchmark_version
- scoring_version
- scenario_versions
- protocol_version

Example:

{
  "benchmark_version": "0.1.0",
  "scoring_version": "0.1.0",
  "scenarios": {
    "navigation_forest_v1": "1.0",
    "memory_delayed_retrieval_v1": "1.0"
  },
  "protocol_version": "1.0"
}

---

# 5. Release Types

## 5.1 Experimental Release

- unstable
- not benchmark-authoritative
- used for development and testing

## 5.2 Benchmark Release

- fully validated
- CI-passing
- reproducible
- used for official evaluation

## 5.3 Archive Release

- frozen historical version
- used for long-term comparison

---

# 6. Release Artifacts

Each release must include:

- source snapshot (tagged commit)
- compiled artifacts (if applicable)
- scenario definitions
- scoring configuration
- replay validation logs
- benchmark report

Optional:

- example agents
- sample replays

---

# 7. Release Process

All releases follow:

1. Freeze changes
2. Run full CI pipeline
3. Validate determinism
4. Validate replay reconstruction
5. Validate scoring reproducibility
6. Tag release
7. Publish artifacts

If any validation fails, release is blocked.

---

# 8. Git Tagging Strategy

Tags must follow versioning scheme:

v0.1.0  
v0.1.1  
v0.2.0  

Rules:

- tags are immutable
- tags must map to CI-passing commits
- no retagging allowed

---

# 9. Reproducibility Requirements

Each release must guarantee:

- fixed random seeds
- pinned dependencies
- identical outputs across environments

Verification:

- rerun benchmark from release
- compare outputs byte-for-byte

Mismatch = invalid release

---

# 10. Benchmark Result Compatibility

Results are only comparable when:

- benchmark_version matches
- scoring_version matches
- scenario versions match

If any differ, results must be labeled:

"NOT COMPARABLE"

---

# 11. Scenario Versioning

Each scenario must include:

- scenario_id
- version
- changelog

Example:

navigation_forest_v1 → v1.0 → v1.1

Rules:

- changes affecting scoring → MINOR or MAJOR bump
- minor fixes → PATCH bump

---

# 12. Scoring Version Control

Scoring changes are high-risk.

Rules:

- any formula change → MAJOR bump
- weight change → MINOR bump
- bug fix → PATCH bump

All scoring changes must be documented.

---

# 13. Release Notes

Each release must include:

- version number
- summary of changes
- affected components
- compatibility notes
- migration guidance (if needed)

---

# 14. Backward Compatibility Policy

- PATCH: fully compatible
- MINOR: mostly compatible
- MAJOR: not compatible

Old versions must remain usable.

---

# 15. CI Integration

Releases must only be created from CI-passing builds.

CI must verify:

- determinism
- replay integrity
- scoring reproducibility

---

# 16. Artifact Storage

Releases should be stored in:

- Git tags (source)
- artifact storage (replays, reports)
- versioned datasets (if applicable)

Artifacts must not be overwritten.

---

# 17. Deprecation Policy

When versions are deprecated:

- mark as deprecated
- document reason
- provide migration path

Do not delete historical versions.

---

# 18. Future Extensions

- automated release pipelines
- versioned leaderboards
- cross-version benchmarking tools
- long-term reproducibility archives

---

# 19. Closing Statement

A benchmark without strict versioning becomes noise.

A benchmark with strict versioning becomes science.

Choose carefully which one you want.
