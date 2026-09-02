from __future__ import annotations

import argparse
import json
from pathlib import Path

from . import contract, development


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("command", choices=("prepare", "preflight", "run", "verify"))
    parser.add_argument("root", type=Path)
    parser.add_argument("--predecessor-root", type=Path)
    args = parser.parse_args()
    if args.command == "prepare":
        if args.predecessor_root is None:
            parser.error("prepare requires --predecessor-root")
        result = contract.prepare(args.root, args.predecessor_root)
    else:
        result = getattr(development, args.command)(args.root)
    print(json.dumps(result, sort_keys=True, separators=(",", ":")), flush=True)


if __name__ == "__main__":
    main()
