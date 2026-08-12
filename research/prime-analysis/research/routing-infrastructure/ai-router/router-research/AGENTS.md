# Agent Rules (Codex)

## Non-negotiables
- Keep diffs small and PR-shaped.
- Always keep the repo runnable.
- Every run produces:
  - a log in `results/raw/`
  - a parsed JSON in `results/parsed/`
  - an appended row in `results/summary.csv`
- Add/update `docs/DECISIONS.md` when conclusions change.

## Experiment protocol
- Screen (1 seed) -> confirm (2 seeds) -> finalize (4 seeds).
- No massive grid sweeps unless there is a decision gate.

## Primary mission
Cut runtime and automate. Stop making the human watch a terminal.
