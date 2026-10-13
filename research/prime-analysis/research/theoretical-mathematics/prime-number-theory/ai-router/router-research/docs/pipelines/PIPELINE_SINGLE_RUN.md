# Pipeline: Single Run

## Input
- Router CLI flags + `seed`.

## Steps
1. Execute `python hyperbolic_router_so8.py ...`.
2. Capture stdout to `results/raw/*.log`.
3. Ensure final `__JSON_SUMMARY__` line exists.

## Output
- raw log
- machine-readable summary line
