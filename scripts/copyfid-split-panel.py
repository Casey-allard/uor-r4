#!/usr/bin/env python3
"""Split a request panel into chunks `lut-chat` accepts.

`lut-chat` refuses a panel with more than 128 requests:
    a request panel needs 1 to 128 requests with distinct ids,
    each with 1 to 8 nonblank user turns

The tool is not patched; the panel is split instead. Chunks keep the original
ids, so every chunk's `chat.json` can be merged by id and scored against the
one `expected.json`. Chunks are written to a fresh directory (they are derived
inputs of a sealed panel, so they must not go under the sealed root).

    copyfid-split-panel.py --panel PANEL_DIR --out CHUNK_DIR [--size 128]
"""

from __future__ import annotations

import argparse
import json
import os
import sys


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--panel", required=True)
    parser.add_argument("--out", required=True)
    parser.add_argument("--size", type=int, default=128)
    args = parser.parse_args()

    with open(os.path.join(args.panel, "requests.json")) as handle:
        requests = json.load(handle)
    if not requests:
        print("empty panel", file=sys.stderr)
        return 2
    if os.path.exists(args.out):
        print(f"refusing: {args.out} already exists", file=sys.stderr)
        return 2
    os.makedirs(args.out)
    written = []
    for index in range(0, len(requests), args.size):
        chunk = requests[index : index + args.size]
        path = os.path.join(args.out, f"chunk-{index // args.size:02d}.json")
        with open(path, "w") as handle:
            json.dump(chunk, handle, indent=1)
        written.append((path, len(chunk)))
    print(f"{len(requests)} requests -> {len(written)} chunks in {args.out}")
    for path, count in written:
        print(f"  {path} ({count})")
    return 0


if __name__ == "__main__":
    sys.exit(main())
