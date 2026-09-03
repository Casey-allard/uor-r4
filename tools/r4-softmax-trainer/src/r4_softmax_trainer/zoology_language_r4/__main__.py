"""Prepare, run or replay the frozen learned-interface R4 comparison."""

from __future__ import annotations

import argparse
import json
from pathlib import Path

from . import contract


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("command", choices=("prepare", "run", "replay"))
    parser.add_argument("root", type=Path)
    parser.add_argument("--source-root", type=Path)
    parser.add_argument("--frame-root", type=Path)
    args = parser.parse_args()
    if args.command == "prepare":
        if args.source_root is None or args.frame_root is None:
            parser.error("prepare requires --source-root and --frame-root")
        result = contract.prepare(args.root, args.source_root, args.frame_root)
    else:
        from . import campaign

        result = (campaign.run if args.command == "run" else campaign.verify)(args.root)
    print(json.dumps(result, sort_keys=True, separators=(",", ":")), flush=True)


if __name__ == "__main__":
    main()
