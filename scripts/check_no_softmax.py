#!/usr/bin/env python3
"""Ratchet gate for D23/D24: no softmax may be added to executable source.

D23 (owner, 2026-10-12) prohibits softmax in every executed path and recorded that
no interlock existed. This is that interlock. It cannot prove the tree is
softmax-free; it stops the count from growing so removal can proceed without new
violations landing underneath it.

Baseline: scripts/no-softmax-baseline.json maps repo-relative path -> number of
softmax sites when the ratchet was set. A file above its baseline, or a file
absent from the baseline with any hit, fails.

  python3 scripts/check_no_softmax.py                  # check (exit 1 on growth)
  python3 scripts/check_no_softmax.py --update-baseline # after REMOVING sites

--update-baseline refuses to raise any count, so it cannot be used to bless new
softmax. Dated records under docs/ are not scanned: they are evidence, not plans.
"""

from __future__ import annotations

import argparse
import json
import pathlib
import re
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent
BASELINE = ROOT / "scripts" / "no-softmax-baseline.json"
SCAN_DIRS = ("crates",)
SUFFIXES = (".rs", ".cu", ".cuh", ".wgsl")
PATTERN = re.compile(r"softmax", re.IGNORECASE)


def scan() -> dict[str, int]:
    found: dict[str, int] = {}
    for d in SCAN_DIRS:
        base = ROOT / d
        if not base.is_dir():
            continue
        for path in sorted(base.rglob("*")):
            if not path.is_file() or path.suffix not in SUFFIXES:
                continue
            if "target" in path.parts:
                continue
            try:
                text = path.read_text(encoding="utf-8", errors="replace")
            except OSError:
                continue
            n = len(PATTERN.findall(text))
            if n:
                found[str(path.relative_to(ROOT))] = n
    return found


def load_baseline() -> dict[str, int]:
    if not BASELINE.is_file():
        return {}
    return json.loads(BASELINE.read_text(encoding="utf-8"))


def main() -> int:
    ap = argparse.ArgumentParser(description="D23 softmax ratchet")
    ap.add_argument("--update-baseline", action="store_true")
    ap.add_argument("--quiet", action="store_true")
    args = ap.parse_args()

    current = scan()
    total = sum(current.values())

    if args.update_baseline:
        old = load_baseline()
        # First run sets the ratchet; afterwards it may only be lowered.
        raised = (
            {p: (old.get(p, 0), n) for p, n in current.items() if n > old.get(p, 0)}
            if old
            else {}
        )
        if not old:
            print("first run: setting the ratchet at the current counts")
        if raised:
            print("REFUSED: --update-baseline may only lower counts. Raised:", file=sys.stderr)
            for p, (o, n) in sorted(raised.items()):
                print(f"  {p}: {o} -> {n}", file=sys.stderr)
            print("Remove the new softmax, or justify it as an owner decision first.", file=sys.stderr)
            return 1
        BASELINE.write_text(json.dumps(current, indent=2, sort_keys=True) + "\n", encoding="utf-8")
        print(f"baseline written: {len(current)} files, {total} sites")
        return 0

    baseline = load_baseline()
    if not baseline:
        print("FAIL: no baseline. Run --update-baseline once to set the ratchet.", file=sys.stderr)
        return 1

    growth: list[str] = []
    for path, n in sorted(current.items()):
        base = baseline.get(path, 0)
        if n > base:
            growth.append(f"  {path}: {base} -> {n} (+{n - base})")

    removed = sum(max(0, baseline.get(p, 0) - current.get(p, 0)) for p in baseline)
    base_total = sum(baseline.values())

    if growth:
        print("FAIL: softmax added to executable source (D23 prohibits it):", file=sys.stderr)
        print("\n".join(growth), file=sys.stderr)
        print(f"\ntotal {base_total} -> {total}. Replace it with a geometric mechanism,", file=sys.stderr)
        print("or get an owner decision; do not raise the baseline.", file=sys.stderr)
        return 1

    if not args.quiet:
        print(f"OK: no softmax added. {total} remaining across {len(current)} files (baseline {base_total}).")
        if removed:
            print(f"    {removed} removed since the baseline: run --update-baseline to lock it in.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
