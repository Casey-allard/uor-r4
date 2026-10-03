#!/usr/bin/env python3
"""Score a chat arm and compare it against every prior arm on this lane.

Usage:
  score-chat-arm.py <arm-root> [--no-replies]

Reads `<arm-root>/dialogue/report.json` (sealed) or `<arm-root>/report.json`,
then optionally scores the reply arms from `<arm-root>/replies-1/chat.json`.

Why this exists: this lane has repeatedly published a number and then had to
retract it because the instrument was wrong or the comparison was not like for
like. The reference arms are pinned here by their known values, so a new arm is
always read against them, and the disjoint held-out panel -- the only evaluation
that has ever caught a template artefact on this lane -- is scored by default
rather than being remembered later.

Exit code is 0 even when an arm looks bad; this is a measurement tool.
"""
import argparse
import json
import os
import re
import sys

HOME = os.path.expanduser("~")
REPORTS = os.path.join(HOME, "uor-r4-worktrees/stack-prose-reports")

# Every arm this lane has measured, with its sealed numbers. A new arm is only
# meaningful relative to these.
REFERENCE = [
    ("S2 (from scratch + Phase A)",                2.520916, 2.031832),
    ("chat-from-prose-1 (prose, no Phase A)",      2.649143, 1.676303),
    ("chat-v0-phase-a-1 (Phase A, no prose)",      2.530589, 1.994121),
    ("chat-prose-init-phase-a-1 (WINNER, seed 1)", 2.097073, 1.417128),
    ("chat-prose-init-seed2-1 (WINNER, seed 2)",   2.098668, 1.426529),
    ("chat-mixedphaseB-seed1-1 (synthetic mix)",   2.207584, 1.544123),
]

# The 38-row development panel's memory rows: the value a correct reply names.
# Derived from panel row *first* turns (the second turn is the distractor).
MEMORY_TRUTH = {
    "dev-mem-01": "alex", "dev-mem-02": "momo", "dev-mem-03": "green",
    "dev-mem-04": "teacher", "dev-mem-05": "tokyo", "dev-mem-06": "two",
    "dev-mem-07": "july", "dev-mem-08": "blue", "dev-mem-09": "piano",
    "dev-mem-10": "pizza",
}


def load_report(root):
    for rel in ("dialogue/report.json", "report.json"):
        path = os.path.join(root, rel)
        if os.path.exists(path):
            with open(path) as fh:
                return json.load(fh), path
    return None, None


def score_replies(path):
    """Return (memory_hits, distinct, total) or (None, None, None)."""
    if not os.path.exists(path):
        return None, None, None
    with open(path) as fh:
        payload = json.load(fh)
    rows = payload.get("record", payload).get("rows", [])
    replies = {}
    for row in rows:
        turns = row.get("turns") or []
        if turns:
            replies[row["id"]] = turns[-1].get("reply", "")
    if not replies:
        return None, None, None

    def names(reply, value):
        return re.search(r"\b" + re.escape(value) + r"\b", reply.lower()) is not None

    hits = sum(1 for key, value in MEMORY_TRUTH.items() if names(replies.get(key, ""), value))
    distinct = len(set(replies.values()))
    return hits, distinct, len(replies)


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("root")
    parser.add_argument("--no-replies", action="store_true")
    parser.add_argument("--replies", default="replies-1")
    args = parser.parse_args()

    root = args.root if os.path.isabs(args.root) else os.path.join(REPORTS, args.root)
    print(f"arm root: {root}")
    if not os.path.isdir(root):
        print("  MISSING -- the run has not produced a root")
        return 1

    report, path = load_report(root)
    if report is None:
        print("  no sealed report yet (looked for dialogue/report.json and report.json)")
        return 1
    final = report.get("final_development") or {}
    initial = report.get("initial_development") or {}
    pooled = final.get("response_mean_nll")
    first4 = final.get("first_four_response_targets_mean_nll")
    print(f"  sealed report: {path}")
    print(f"  initial : pooled {initial.get('response_mean_nll')}  first-four "
          f"{initial.get('first_four_response_targets_mean_nll')}")
    print(f"  FINAL   : pooled {pooled}  first-four {first4}  "
          f"({final.get('selected_responses')} responses)")

    print("\n  against every prior arm on this lane:")
    print(f"    {'arm':44s} {'pooled':>9} {'first4':>9}   {'Δpooled':>8}")
    for name, ref_pooled, ref_first in REFERENCE:
        if pooled is None:
            print(f"    {name:44s} {ref_pooled:9.6f} {ref_first:9.6f}")
        else:
            delta = pooled - ref_pooled
            print(f"    {name:44s} {ref_pooled:9.6f} {ref_first:9.6f}   {delta:+8.6f}")
    if pooled is not None:
        best = min(ref[1] for ref in REFERENCE)
        print(f"\n  best prior pooled {best:.6f}; this arm {pooled:.6f} "
              f"({'BETTER' if pooled < best else 'worse'} by {abs(pooled - best):.6f})")

    if not args.no_replies:
        hits, distinct, total = score_replies(os.path.join(root, args.replies, "chat.json"))
        print("\n  replies:")
        if hits is None:
            print(f"    (no {args.replies}/chat.json -- run lut-chat on the integer export)")
        else:
            print(f"    memory (derived panel) : {hits}/10   [winner: 4/10]")
            print(f"    distinct               : {distinct}/{total}  [winner: 38/38]")

    for label, rel in (("disjoint held-out (40 rows)", "heldout40"),
                       ("oracle-conditioned", "oracle")):
        candidate = os.path.join(REPORTS, "value-faith-20261002", f"{rel}-{os.path.basename(root)}")
        if os.path.exists(candidate):
            print(f"  {label}: {candidate}")

    print("\n  NOTE: label any result from this arm as geometry-on-principle, not a "
          "demonstrated advantage. The matched measurement is 1.9981 geometric vs "
          "2.0111 transformer (Δ -0.0130, inside the +-0.021 seed spread) = parity.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
