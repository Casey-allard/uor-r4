# MUDBench Baseline Agents (v1)

This directory contains the first reference baselines used for calibration and regression checks.

## Agents

- `RandomBaselineAgent` (`random_agent.py`)
  - chooses a uniformly random action from `observation.action_space`
  - deterministic under seed (`--seed`)

- `GreedyBaselineAgent` (`greedy_agent.py`)
  - applies deterministic verb-priority scoring:
    - `attack > take > use > move > inspect > wait`
  - ties are resolved with seeded RNG (`--seed`)

## Interface and execution

Both baselines:

- consume observation JSON lines from STDIN
- produce action JSON lines on STDOUT
- conform to the protocol models in `src/agents/protocol`
- only choose actions from the provided `action_space`

## Example run

```bash
python examples/agents/baselines/random_agent.py --seed 7
python examples/agents/baselines/greedy_agent.py --seed 7
```
