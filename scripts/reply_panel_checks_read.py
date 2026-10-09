#!/usr/bin/env python3
"""First reading of the reply panel's deterministic checks, and the canned-reply control.

For #2029. CPU only: reads the frozen checks file, a sealed report and a canned-reply list;
generates nothing and grades nothing.

It reports three things:

  1. `check_pass` over the checked rows of the sealed replies — the deterministic sub-reading
     the new checks make possible, per kind and per category. This is a RETROSPECTIVE first
     reading on replies that already existed; the target was frozen before any model is run
     against these checks, and the panel is development evidence either way.
  2. THE CANNED-REPLY CONTROL: every memorised string in `reply-panel-canned.txt` evaluated
     against every check. A canned string must pass ZERO checks. (The same control is run
     independently by `chat-grade check constants=...`, whose `check_only_controls` uses the
     frozen grader's own `RowCheck::passes`; this script is the cross-check.)
  3. The best constant over the abstention rows, because a constant abstention passes an
     `abstain_exact` check by design — that is reported, not hidden.

The check semantics here are a port of `chat-grade`'s `RowCheck::passes` and
`abstention_fault` from `scripts/check_panel_v5_conformance.py` on main; `chat-grade check` is
the authoritative validation and is run separately.

    python3 scripts/reply_panel_checks_read.py --checks F.tsv --report R.json \
        --canned F.txt [--constants F.txt] [--out summary.json]
"""
import argparse
import collections
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "scripts"))

import check_panel_v5_conformance as port  # noqa: E402

DEFAULT_CONSTANTS = [
    "I'm not sure. Can you tell me more about what you mean?",
    "That sounds nice! Thank you for telling me.",
    "Once upon a time, there was a little girl named Lily. She liked to play outside.",
]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--checks", required=True)
    parser.add_argument("--report", required=True)
    parser.add_argument("--canned", required=True)
    parser.add_argument("--constants", help="extra constants to read the abstention rows against")
    parser.add_argument("--out")
    args = parser.parse_args()

    checks = port.parse_checks(open(args.checks, encoding="utf-8").read())
    report = json.load(open(args.report))
    canned = [line.rstrip("\n") for line in open(args.canned, encoding="utf-8")
              if line.strip() and not line.startswith("#")]
    constants = list(DEFAULT_CONSTANTS)
    if args.constants:
        constants += [line.rstrip("\n") for line in open(args.constants, encoding="utf-8")
                      if line.strip() and not line.startswith("#")]

    per_row, by_kind, by_category = [], collections.Counter(), collections.defaultdict(
        collections.Counter)
    for row in report["rows"]:
        check = checks.get(row["id"])
        if check is None:
            continue
        reply = row["conversation"][-1]["assistant"]
        users = [turn["user"] for turn in row["conversation"]]
        passed = port.passes(check, users, reply)
        per_row.append({"id": row["id"], "category": row["category"], "kind": check["kind"],
                        "check_pass": passed})
        by_kind[check["kind"]] += 1 if passed else 0
        by_category[row["category"]]["checked"] += 1
        by_category[row["category"]]["pass"] += 1 if passed else 0

    control = []
    for text in canned:
        hits = [row["id"] for row in per_row
                if port.passes(checks[row["id"]], [t["user"] for t in
                                                   next(r for r in report["rows"]
                                                        if r["id"] == row["id"])["conversation"]],
                               text)]
        control.append({"reply": text, "rows_passed": len(hits), "ids": hits[:8]})

    constants_read = []
    for text in constants:
        hits = sum(1 for row in per_row
                   if port.passes(checks[row["id"]],
                                  [t["user"] for t in next(r for r in report["rows"]
                                                           if r["id"] == row["id"])["conversation"]],
                                  text))
        constants_read.append({"reply": text, "rows_passed": hits})

    checked = len(per_row)
    passed = sum(1 for row in per_row if row["check_pass"])
    summary = {
        "checks_file": args.checks,
        "checked_rows": checked,
        "check_pass": passed,
        "check_pass_by_kind": {kind: {"checked": sum(1 for r in per_row if r["kind"] == kind),
                                      "pass": by_kind[kind]}
                               for kind in sorted({r["kind"] for r in per_row})},
        "check_pass_by_category": {k: dict(v) for k, v in by_category.items()},
        "canned_control_max_rows_passed": max((c["rows_passed"] for c in control), default=0),
        "canned_control": control,
        "canned_control_result": ("pass" if all(c["rows_passed"] == 0 for c in control)
                                  else "FAIL"),
        "constants_read": constants_read,
        "per_row": per_row,
    }
    if args.out:
        json.dump(summary, open(args.out, "w"), indent=1)
    print(json.dumps({k: v for k, v in summary.items() if k != "per_row"}, indent=1))
    return 0


if __name__ == "__main__":
    sys.exit(main())
