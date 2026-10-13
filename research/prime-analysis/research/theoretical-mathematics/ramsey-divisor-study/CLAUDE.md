# ramsey — CLAUDE.md

Read by Claude/Anthropic agents on workspace entry.
This file is repo-local and aligned to the Ramsey research mission in this repo.

---

## Project identity

This is a mathematical research project on Ramsey-style graph coloring.
Current focus: test whether phi/Fibonacci-inspired recursive constructions offer
useful empirical structure for small instances compared with baseline methods.

Code is an experiment harness, not a theorem proof.

---

## Mandatory startup sequence

Run from repo root:

```bash
pwd
ls
git status
```

If available:

```bash
make state
```

---

## Core research scope

Primary question:
- Do recursive families (especially phi/Fibonacci-inspired) delay forced
  monochromatic patterns better than random and simple partitions at small n?

Required comparison families for exploratory studies:
- Fibonacci block growth
- Lucas-like growth
- Balanced binary recursive partitions
- Prime-anchor inspired partitioning
- Random baseline

---

## Hard research rules

1. Do not overclaim. Exploratory evidence is not proof.
2. Keep runs reproducible (fixed seeds, explicit configs, deterministic ordering).
3. Record both positive and negative results.
4. If signal is weak, mixed, or unstable, state that directly.
5. Avoid broad refactors; prefer smallest changes that advance experiments.

---

## Minimum measurement expectations

For each family and size range, capture:
- largest monochromatic clique (or equivalent avoided-threshold proxy)
- largest monochromatic independent set (or equivalent avoided-threshold proxy)
- cheap adjacency spectral indicators where feasible
- recurring ratio or partition signatures, if any

---

## Deliverables expectation

Unless the user says otherwise, provide:
- executable scripts in `scripts/` or `tasks/`
- machine-readable outputs in `results/` (CSV/JSON preferred)
- short writeup summarizing what appears to be signal vs noise
- explicit recommendation: `KEEP`, `KILL`, or `REFINE`
- limitations and uncertainty notes

---

## If uncertain

Do the smallest reproducible screen experiment first, publish artifacts, then
decide whether follow-up confirmation runs are justified.
