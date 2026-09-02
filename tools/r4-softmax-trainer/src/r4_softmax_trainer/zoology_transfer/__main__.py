"""Create-once command interface for #1053, separate from historical runs."""

from __future__ import annotations

import argparse
import json
from pathlib import Path

from .contract import prepare_transfer
from .development import preflight_transfer, run_transfer, verify_transfer


def main() -> None:
    parser = argparse.ArgumentParser(
        description="CPU-only exact-#1045 transfer (#1053)"
    )
    commands = parser.add_subparsers(dest="command", required=True)
    prepare = commands.add_parser("prepare")
    prepare.add_argument("root", type=Path)
    for name in ("source-root", "predecessor-root", "release-root"):
        prepare.add_argument(f"--{name}", type=Path, required=True)
    for name in ("preflight", "run", "verify"):
        commands.add_parser(name).add_argument("root", type=Path)
    args = parser.parse_args()
    if args.command == "prepare":
        result = prepare_transfer(
            args.root,
            source_root=args.source_root,
            predecessor_root=args.predecessor_root,
            release_root=args.release_root,
        )
    else:
        result = {
            "preflight": preflight_transfer,
            "run": run_transfer,
            "verify": verify_transfer,
        }[args.command](args.root)
    print(json.dumps(result, sort_keys=True, separators=(",", ":")), flush=True)


if __name__ == "__main__":
    main()
