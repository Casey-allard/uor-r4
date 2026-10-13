#!/usr/bin/env python3
"""Print the repo's canonical context packet in a fixed order."""

from __future__ import annotations

import argparse
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]

GROUPS = {
    "startup": [
        ROOT / "docs/SESSION_BOOTSTRAP.md",
        ROOT / "CORE_PROJECT_GOALS.md",
        ROOT / "docs/PROJECT_CONTEXT.md",
        ROOT / "docs/research/KILL_LIST_TRACKER.md",
        ROOT / "docs/research/ACTIVE_STATE.md",
        ROOT / "docs/routes/ROUTE_MATRIX.md",
        ROOT / "docs/DECISIONS.md",
        ROOT / "docs/research/SUPPORTING_EVIDENCE.md",
        ROOT / "docs/research/CURRENT_DIRECTION.md",
        ROOT / "docs/research/HANDOFF_CURRENT.md",
        ROOT / "docs/research/LIVE_WORKLOG.md",
    ],
    "theory": [
        ROOT / "CORE_PROJECT_GOALS.md",
        ROOT / "GEOMETRIC_COMPUTATION_HYPOTHESIS.md",
        ROOT / "geometric_routing_kill_tests.md",
        ROOT / "phase_transport_hypothesis.md",
        ROOT / "hyperbolic_router_math_review.md",
        ROOT / "spectral_emergence_moonshot.md",
        ROOT / "adaptive_field_computer_moonshot.md",
    ],
}


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Print canonical context file order for repo recovery."
    )
    parser.add_argument(
        "--group",
        choices=sorted(GROUPS),
        default="startup",
        help="Context packet group to print.",
    )
    parser.add_argument(
        "--cat",
        action="store_true",
        help="Print file contents instead of only listing file paths.",
    )
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    files = GROUPS[args.group]

    if not args.cat:
        print(f"{args.group} context files:")
        for path in files:
            print(path.relative_to(ROOT))
        return 0

    for path in files:
        print(f"# >>> {path.relative_to(ROOT)}")
        print(path.read_text())
        if not path.read_text().endswith("\n"):
            print()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
