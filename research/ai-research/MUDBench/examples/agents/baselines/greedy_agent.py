"""Deterministic simple greedy baseline agent for MUDBench."""

from __future__ import annotations

import argparse
import random
import sys
from dataclasses import dataclass, field
from typing import Any, Sequence

from agents.protocol.action import ActionSubmission
from agents.protocol.observation import Observation
from agents.protocol.serialization import canonical_json_dumps, json_loads_object


_VERB_PRIORITY = {
    "attack": 6,
    "take": 5,
    "use": 4,
    "move": 3,
    "inspect": 2,
    "wait": 1,
}


@dataclass(frozen=True, slots=True)
class GreedyBaselineAgent:
    """Simple greedy baseline with deterministic verb-priority scoring."""

    seed: int
    _rng: random.Random = field(init=False, repr=False, compare=False)

    def __post_init__(self) -> None:
        object.__setattr__(self, "_rng", random.Random(self.seed))

    def decide(self, observation: Observation) -> ActionSubmission:
        if not observation.action_space:
            raise ValueError("Greedy baseline requires a non-empty action_space")

        scored_actions = [(self._score(action), action) for action in observation.action_space]
        best_score = max(score for score, _ in scored_actions)
        candidates = sorted(action for score, action in scored_actions if score == best_score)

        if len(candidates) == 1:
            selected = candidates[0]
        else:
            selected = candidates[self._rng.randrange(len(candidates))]

        return ActionSubmission(action=selected)

    def specification(self) -> dict[str, Any]:
        return {
            "agent_id": "greedy_baseline",
            "version": "v1",
            "category": "greedy",
            "description": "Greedy valid-action baseline using fixed verb priorities.",
            "deterministic": True,
            "parameters": {"seed": self.seed, "priority": dict(_VERB_PRIORITY)},
            "supported_scenarios": ["*"],
        }

    @staticmethod
    def _score(action: str) -> int:
        verb = action.strip().split(maxsplit=1)[0].lower()
        return _VERB_PRIORITY.get(verb, 0)


def run_stdio_loop(agent: GreedyBaselineAgent) -> int:
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
    parser = argparse.ArgumentParser(prog="greedy_baseline_agent")
    parser.add_argument("--seed", type=int, default=0)
    args = parser.parse_args(argv)

    agent = GreedyBaselineAgent(seed=args.seed)
    try:
        return run_stdio_loop(agent)
    except (ValueError, KeyError, TypeError) as exc:
        print(f"greedy_baseline_agent error: {exc}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
