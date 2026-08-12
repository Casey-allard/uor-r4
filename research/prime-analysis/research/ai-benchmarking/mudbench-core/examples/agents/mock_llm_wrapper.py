"""Mock LLM-style local wrapper example for MUDBench."""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

_SRC_PATH = Path(__file__).resolve().parents[2] / "src"
if str(_SRC_PATH) not in sys.path:
    sys.path.insert(0, str(_SRC_PATH))


def _read_observation():
    from agents.protocol.observation import Observation

    raw_line = sys.stdin.readline()
    if not raw_line:
        return None
    payload = json.loads(raw_line)
    if not isinstance(payload, dict):
        raise ValueError("observation_payload_must_be_object")
    return Observation.from_dict(payload)


def _build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(prog="mock_llm_wrapper", add_help=False)
    parser.add_argument(
        "--emit-invalid-first",
        action="store_true",
        help="Emit one deterministic malformed first response so the bounded repair path is exercised.",
    )
    parser.add_argument(
        "--emit-invalid-always",
        action="store_true",
        help="Emit deterministic malformed output on both attempts so the fail-closed fallback path is exercised.",
    )
    parser.add_argument(
        "--persistent-session",
        action="store_true",
        help="Keep reading observations from stdin and return one bounded action JSON per line.",
    )
    return parser


def main() -> int:
    try:
        from agents.llm_runtime import run_mock_benchmark_llm_turn

        args = _build_parser().parse_args()
        turn_index = 0
        while True:
            observation = _read_observation()
            if observation is None:
                break
            result = run_mock_benchmark_llm_turn(
                observation,
                actor_id=f"mock-wrapper-turn-{turn_index}",
                emit_invalid_first=args.emit_invalid_first and turn_index == 0,
                emit_invalid_always=args.emit_invalid_always,
            )
            sys.stdout.write(
                json.dumps(
                    result.action_submission.to_dict(),
                    sort_keys=True,
                    separators=(",", ":"),
                    ensure_ascii=True,
                )
            )
            sys.stdout.write("\n")
            sys.stdout.flush()
            turn_index += 1
            if not args.persistent_session:
                break
        return 0
    except (ValueError, TypeError, json.JSONDecodeError) as exc:
        print(f"mock_llm_wrapper error: {exc}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
