"""Prepare, fit, evaluate or independently replay the one #1063 curriculum."""

from __future__ import annotations

import argparse
import json
from pathlib import Path

from . import contract


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("command", choices=("prepare", "fit", "run", "verify"))
    parser.add_argument("root", type=Path)
    parser.add_argument("--frames-root", type=Path)
    args = parser.parse_args()
    if args.command == "prepare":
        if args.frames_root is None:
            parser.error("prepare requires --frames-root")
        result = contract.prepare(args.root, args.frames_root)
    elif args.command == "fit":
        from .training import fit

        result = fit(args.root, contract.validate_preparation(args.root))
    else:
        from . import campaign

        result = getattr(campaign, args.command)(args.root)
    print(json.dumps(result, sort_keys=True, separators=(",", ":")), flush=True)


if __name__ == "__main__":
    main()
