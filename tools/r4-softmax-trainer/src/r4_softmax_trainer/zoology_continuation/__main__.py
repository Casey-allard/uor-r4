"""Prepare, run, or verify the bounded #1057 checkpoint continuation."""

from __future__ import annotations

import argparse
import json
from pathlib import Path

from . import contract, development


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("command", choices=("prepare", "run", "verify"))
    parser.add_argument("root", type=Path)
    parser.add_argument("--parent-root", type=Path)
    args = parser.parse_args()
    if args.command == "prepare":
        if args.parent_root is None:
            parser.error("prepare requires --parent-root")
        result = contract.prepare(args.root, args.parent_root)
    else:
        result = getattr(development, args.command)(args.root)
    print(json.dumps(result, sort_keys=True, separators=(",", ":")), flush=True)


if __name__ == "__main__":
    main()
