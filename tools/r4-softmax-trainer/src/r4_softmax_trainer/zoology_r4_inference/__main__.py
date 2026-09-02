"""Prepare, execute, or independently replay the #1059 inference integration."""

from __future__ import annotations

import argparse
import json
from pathlib import Path

from . import contract


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("command", choices=("prepare", "run", "verify"))
    parser.add_argument("root", type=Path)
    parser.add_argument("--source-root", type=Path)
    parser.add_argument("--frames-root", type=Path)
    args = parser.parse_args()
    if args.command == "prepare":
        if args.source_root is None or args.frames_root is None:
            parser.error("prepare requires --source-root and --frames-root")
        result = contract.prepare(args.root, args.source_root, args.frames_root)
    else:
        from . import campaign

        result = getattr(campaign, args.command)(args.root)
    print(json.dumps(result, sort_keys=True, separators=(",", ":")), flush=True)


if __name__ == "__main__":
    main()
