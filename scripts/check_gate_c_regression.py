#!/usr/bin/env python3
"""Gate C trend check (issue #77; reworked 2026-07-30, process fix).

Two modes, chosen automatically:

REGRESSION (pin unchanged vs base): the measured row must not regress
beyond the alarm thresholds relative to the pinned record — the
original trend alarm.

RE-PIN SELF-CONSISTENCY (this revision's pin differs from the base
revision's pin): the PR is deliberately accepting a new row (semantic
change + era note, maintainer-priced). The check then verifies the NEW
pin honestly matches the measured row within the alarm thresholds in
BOTH directions — a stale, optimistic, or fat-fingered re-pin fails.

Why: pinned-absolute-only checking coupled every accepted semantic
change to merge-queue ordering gymnastics (2026-07-30: two PRs + a
queue simulation + manual sequencing for one re-pin). With
self-consistency mode, a re-pin travels IN the PR that changes the
semantics, any merge order works, and the alarm still catches every
unintended regression — an intended one is exactly a pin diff with an
era note, which is what maintainer review reads.

Usage:
  check_gate_c_regression.py <score_report.json> [--base-pin <path>]

--base-pin: the base revision's gate_c_pinned.json (CI passes
`git show` output; preflight passes origin/main's copy). Omitted →
regression mode against the working-tree pin (original behavior).
"""
import sys
import json
import os

_PIN_PATH = os.path.join(
    os.path.dirname(os.path.dirname(os.path.abspath(__file__))),
    "docs", "transformerless", "gate_c_pinned.json",
)


def load(path):
    with open(path) as f:
        return json.load(f)


def main():
    args = sys.argv[1:]
    if not args:
        print("Usage: check_gate_c_regression.py <path/to/score_report.json> [--base-pin <path>]")
        sys.exit(1)
    report_path = args[0]
    base_pin_path = None
    if "--base-pin" in args:
        base_pin_path = args[args.index("--base-pin") + 1]

    pin = load(_PIN_PATH)
    report = load(report_path)
    rule12 = report.get("gate_c", {}).get("rule12_precedence")
    if not rule12:
        print("Error: 'rule12_precedence' metrics not found in the report.")
        sys.exit(1)
    top1 = rule12.get("top1_agreement", 0.0)
    bpt = rule12.get("bits_per_token", 0.0)
    pin_top1 = pin["rule12_top1_agreement"]
    pin_bpt = pin["rule12_bits_per_token"]
    max_drop = pin["alarm"]["top1_drop_abs"]
    max_worsen = pin["alarm"]["bits_regress_abs"]

    repin = False
    if base_pin_path:
        try:
            base = load(base_pin_path)
            repin = (
                base.get("rule12_top1_agreement") != pin_top1
                or base.get("rule12_bits_per_token") != pin_bpt
            )
        except (OSError, json.JSONDecodeError):
            print("note: base pin unreadable; falling back to regression mode")

    print(f"Gate C (Rule 1+2) Current: top-1={top1:.4f} ({top1*100:.1f}%), bits/token={bpt:.4f}")
    print(f"Gate C (Rule 1+2) Pinned : top-1={pin_top1:.4f} ({pin_top1*100:.1f}%), bits/token={pin_bpt:.4f}")

    failed = False
    if repin:
        print("Mode: RE-PIN SELF-CONSISTENCY (pin differs from base revision — era-note re-pin)")
        if abs(top1 - pin_top1) > max_drop:
            print(f"🚨 RE-PIN MISMATCH: measured top-1 differs from the new pin by {abs(top1 - pin_top1)*100:.2f} points (> {max_drop*100:.1f})")
            failed = True
        if abs(bpt - pin_bpt) > max_worsen:
            print(f"🚨 RE-PIN MISMATCH: measured bits/token differs from the new pin by {abs(bpt - pin_bpt):.4f} (> {max_worsen:.2f})")
            failed = True
    else:
        print("Mode: REGRESSION (pin unchanged vs base)")
        if top1 < (pin_top1 - max_drop):
            print(f"🚨 REGRESSION ALARM: top-1 agreement dropped by more than {max_drop*100:.1f} points!")
            print(f"   Delta: {(top1 - pin_top1)*100:.2f} points")
            failed = True
        if bpt > (pin_bpt + max_worsen):
            print(f"🚨 REGRESSION ALARM: bits/token worsened by more than {max_worsen:.2f}!")
            print(f"   Delta: {bpt - pin_bpt:+.4f} bits/token")
            failed = True

    if failed:
        print("Gate C regression check FAILED.")
        sys.exit(1)
    print("Gate C regression check PASSED.")
    sys.exit(0)


if __name__ == "__main__":
    main()
