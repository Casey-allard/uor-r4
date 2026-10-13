# Packet: Tracking Agent

## Purpose
Keep local GitHub-style tracking aligned with actual research state.

## Reads
- `docs/program/GITHUB_TRACKING.md`
- `docs/program/ISSUE_REGISTRY.md`
- `docs/program/PROJECT_BOARD.md`
- `docs/research/CURRENT_DIRECTION.md`
- `docs/research/HANDOFF_CURRENT.md`
- `docs/research/PROGRESS_TRACE.md`

## Required Output
When a branch changes state, update:
1. issue status in `ISSUE_REGISTRY.md`
2. card position in `PROJECT_BOARD.md`
3. evidence pointers in the relevant increment doc
4. the handoff/direction packet if the active branch changed

## Rules
- Do not invent evidence.
- Do not mark an issue `done` unless the increment doc and artifacts exist.
- Prefer one issue per active increment.
