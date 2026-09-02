"""Command-line lifecycle for the frozen #1050 Zoology reproduction."""

from __future__ import annotations

import argparse
import json
from collections.abc import Mapping
from pathlib import Path
from typing import Any

from .development import (
    execute_release_reproduction,
    prepare_release_reproduction,
    preflight_release_reproduction,
    run_release_reproduction,
    verify_release_reproduction,
)


def _parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        prog="python -m r4_softmax_trainer.zoology_release",
        description="CPU-only exact-source Figure-2 attention reproduction (#1050).",
    )
    commands = parser.add_subparsers(dest="command", required=True)

    prepare = commands.add_parser("prepare", help="bind source, predecessor, and data")
    prepare.add_argument("root", type=Path)
    prepare.add_argument("--predecessor-root", type=Path, required=True)

    for name, help_text in (
        ("preflight", "repeat C0 and measure batch-512 CPU plans"),
        ("run", "execute the frozen learning-rate arms with early stop"),
        ("verify", "verify result envelopes and artifacts"),
    ):
        command = commands.add_parser(name, help=help_text)
        command.add_argument("root", type=Path)

    execute = commands.add_parser("execute", help="prepare, preflight, run, verify")
    execute.add_argument("root", type=Path)
    execute.add_argument("--predecessor-root", type=Path, required=True)
    return parser


def _print(value: Mapping[str, Any]) -> None:
    print(json.dumps(value, sort_keys=True, separators=(",", ":")))


def main(argv: list[str] | None = None) -> None:
    arguments = _parser().parse_args(argv)
    if arguments.command == "prepare":
        result = prepare_release_reproduction(
            arguments.root,
            predecessor_root=arguments.predecessor_root,
        )
    elif arguments.command == "preflight":
        result = preflight_release_reproduction(arguments.root)
    elif arguments.command == "run":
        result = run_release_reproduction(arguments.root)
    elif arguments.command == "verify":
        result = verify_release_reproduction(arguments.root)
    elif arguments.command == "execute":
        result = execute_release_reproduction(
            arguments.root,
            predecessor_root=arguments.predecessor_root,
        )
    else:  # pragma: no cover
        raise AssertionError(f"unknown command: {arguments.command}")
    _print(result)


if __name__ == "__main__":
    main()
