"""Freeze, fit, evaluate or replay the single #1067 readout-placement candidate."""

from __future__ import annotations

import argparse
import json
from pathlib import Path

from . import contract


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("command", choices=("prepare", "fit", "run", "verify"))
    parser.add_argument("root", type=Path)
    parser.add_argument("--source-root", type=Path)
    args = parser.parse_args()
    if args.command == "prepare":
        if args.source_root is None:
            parser.error("prepare requires --source-root for the retained #1063 data")
        result = contract.prepare(args.root, args.source_root)
    elif args.command == "fit":
        from .training import fit

        result = fit(args.root, contract.validate_preparation(args.root))
    else:
        from . import campaign

        result = getattr(campaign, args.command)(args.root)
    print(json.dumps(result, sort_keys=True, separators=(",", ":")), flush=True)


if __name__ == "__main__":
    main()
