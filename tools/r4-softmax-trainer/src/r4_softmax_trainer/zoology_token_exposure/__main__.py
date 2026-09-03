"""Prepare, run or replay the frozen construction-only exposure diagnostic."""

import argparse
import json
from pathlib import Path

from . import contract


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("command", choices=("prepare", "run", "replay"))
    parser.add_argument("root", type=Path)
    parser.add_argument("--source-root", type=Path)
    args = parser.parse_args()
    if args.command == "prepare":
        if args.source_root is None:
            parser.error("prepare requires --source-root pointing to retained #1079")
        result = contract.prepare(args.root, args.source_root)
    else:
        from . import campaign

        result = (campaign.run if args.command == "run" else campaign.verify)(args.root)
    print(json.dumps(result, sort_keys=True, separators=(",", ":")), flush=True)
    if result.get("diagnosis_permitted") is False:
        raise SystemExit(2)


if __name__ == "__main__":
    main()
