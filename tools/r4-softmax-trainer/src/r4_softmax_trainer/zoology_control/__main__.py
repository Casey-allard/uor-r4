"""Command-line lifecycle for ``python -m r4_softmax_trainer.zoology_control``."""

from __future__ import annotations

import argparse
import json
from collections.abc import Mapping
from pathlib import Path
from typing import Any

from .development import (
    execute_zoology_control,
    prepare_zoology_control,
    preflight_zoology_control,
    run_zoology_control,
    verify_zoology_control,
)


def _parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        prog="python -m r4_softmax_trainer.zoology_control",
        description="Create-once CPU lifecycle for the frozen #1047 Zoology control.",
    )
    commands = parser.add_subparsers(dest="command", required=True)

    prepare = commands.add_parser("prepare", help="bind open inputs and provenance")
    prepare.add_argument("root", type=Path)
    prepare.add_argument("--source-root", type=Path, required=True)
    prepare.add_argument("--predecessor-root", type=Path, required=True)

    for name, help_text in (
        ("preflight", "run C0 and measured CPU admission"),
        ("run", "execute the reached C1/C2 rungs once"),
        ("verify", "verify structure and CIDs without a long rescore"),
    ):
        command = commands.add_parser(name, help=help_text)
        command.add_argument("root", type=Path)

    execute = commands.add_parser("execute", help="prepare, preflight, run, verify")
    execute.add_argument("root", type=Path)
    execute.add_argument("--source-root", type=Path, required=True)
    execute.add_argument("--predecessor-root", type=Path, required=True)
    return parser


def _print(value: Mapping[str, Any]) -> None:
    print(json.dumps(value, sort_keys=True, separators=(",", ":")))


def main(argv: list[str] | None = None) -> None:
    arguments = _parser().parse_args(argv)
    if arguments.command == "prepare":
        result = prepare_zoology_control(
            arguments.root,
            source_root=arguments.source_root,
            predecessor_root=arguments.predecessor_root,
        )
    elif arguments.command == "preflight":
        result = preflight_zoology_control(arguments.root)
    elif arguments.command == "run":
        result = run_zoology_control(arguments.root)
    elif arguments.command == "verify":
        result = verify_zoology_control(arguments.root)
    elif arguments.command == "execute":
        result = execute_zoology_control(
            arguments.root,
            source_root=arguments.source_root,
            predecessor_root=arguments.predecessor_root,
        )
    else:  # pragma: no cover - argparse makes this unreachable.
        raise AssertionError(f"unknown command: {arguments.command}")
    _print(result)


if __name__ == "__main__":
    main()
