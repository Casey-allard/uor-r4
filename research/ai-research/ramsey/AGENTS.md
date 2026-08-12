# ramsey — Agent Instructions (all agents)

This repository is for mathematical research on small Ramsey-style graph coloring,
with a focus on testing whether phi/Fibonacci-inspired recursive constructions
show practical signal versus brute force and random baselines.

Code is an experimental harness. Do not overclaim mathematical conclusions.

---

## Mission

Investigate whether structured recursive constructions can delay forced
monochromatic structure in 2-color edge colorings at small n.

Primary question:
- Do phi/Fibonacci/Lucas-like recursive patterns produce repeatable empirical
  advantages over random and simple partition baselines?

---

## Session start (always do first)

Run these from repo root:

```bash
pwd
ls
git status
```

If custom checks exist, run:

```bash
make state
```

---

## Scope and boundaries

- Work only in this repository root (no external project assumptions).
- Keep changes small and task-focused.
- Do not rewrite history or use destructive git commands.
- Do not introduce heavyweight dependencies unless justified in writing.
- Preserve reproducibility: fixed seeds, deterministic ordering, explicit config.

---

## Research rules

Before running or changing an experiment, state:
1. What family/construction is being tested.
2. What metric defines signal.
3. What would count as "no signal."

Minimum expected families for comparative exploratory studies:
- Fibonacci block growth
- Lucas-like growth
- Balanced binary recursive partitions
- Prime-anchor inspired partitioning
- Random baseline

Minimum expected outputs:
- Largest monochromatic clique (or avoided threshold proxy)
- Largest monochromatic independent set (or avoided threshold proxy)
- Cheap spectral indicators from adjacency matrices when feasible
- Notes on recurring partition/ratio signatures

---

## Validation rules

- Run the narrowest relevant checks before finalizing.
- For experiment edits: run at least one smoke experiment and verify artifacts.
- Do not report success without showing command outputs or generated files.

---

## Deliverable format (default)

For experiment tasks, provide:
- `scripts/` or `tasks/` code
- machine-readable tables in `results/` (CSV/JSON)
- short writeup in `docs/` or `results/reports/`
- explicit recommendation: `KEEP`, `KILL`, or `REFINE`
- a plain statement of uncertainty / limits

---

## Truthfulness standard

- Exploratory evidence is not proof.
- Report negative or null results clearly.
- Distinguish measured outcomes from interpretation.

If the data is weak or mixed, default recommendation should be `REFINE` or `KILL`.
