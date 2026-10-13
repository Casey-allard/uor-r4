"""Deterministic random baseline agent for MUDBench."""

from __future__ import annotations

import argparse
import random
import sys
from dataclasses import dataclass, field
from typing import Any, Sequence

from agents.protocol.action import ActionSubmission
from agents.protocol.observation import Observation
from agents.protocol.serialization import canonical_json_dumps, json_loads_object


@dataclass(frozen=True, slots=True)
class RandomBaselineAgent:
    """Random baseline that selects uniformly from valid actions."""

    seed: int
    _rng: random.Random = field(init=False, repr=False, compare=False)

    def __post_init__(self) -> None:
        object.__setattr__(self, "_rng", random.Random(self.seed))

    def decide(self, observation: Observation) -> ActionSubmission:
        if not observation.action_space:
            raise ValueError("Random baseline requires a non-empty action_space")
        selected = self._rng.choice(observation.action_space)
        return ActionSubmission(action=selected)

    def specification(self) -> dict[str, Any]:
        return {
            "agent_id": "random_baseline",
            "version": "v1",
            "category": "random",
            "description": "Uniform random valid-action baseline with seeded RNG.",
            "deterministic": True,
            "parameters": {"seed": self.seed},
            "supported_scenarios": ["*"],
        }


def run_stdio_loop(agent: RandomBaselineAgent) -> int:
    """Read observation JSON lines from stdin and write action JSON lines to stdout."""
    for raw_line in sys.stdin:
        line = raw_line.strip()
        if not line:
            continue
        payload = json_loads_object(line)
        observation = Observation.from_dict(payload)
        action = agent.decide(observation)
        sys.stdout.write(canonical_json_dumps(action.to_dict()) + "\n")
        sys.stdout.flush()
    return 0


def main(argv: Sequence[str] | None = None) -> int:
    parser = argparse.ArgumentParser(prog="random_baseline_agent")
    parser.add_argument("--seed", type=int, default=0)
    args = parser.parse_args(argv)

    agent = RandomBaselineAgent(seed=args.seed)
    try:
        return run_stdio_loop(agent)
    except (ValueError, KeyError, TypeError) as exc:
        print(f"random_baseline_agent error: {exc}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
